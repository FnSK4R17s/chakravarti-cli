//! Docker/Podman client wrapper.

use std::collections::HashMap;
use std::time::Duration;

use bollard::container::{
    Config, CreateContainerOptions, LogOutput, LogsOptions, RemoveContainerOptions,
    StartContainerOptions, WaitContainerOptions,
};
use bollard::exec::{CreateExecOptions, StartExecResults};
use bollard::image::CreateImageOptions;
use bollard::models::{HostConfig, Mount, MountTypeEnum};
use bollard::Docker;
use futures_util::StreamExt;

use crate::SandboxError;

/// Default Docker image for execution (contains Claude Code CLI).
pub const DEFAULT_IMAGE: &str = "ckrv-agent:latest";

/// Docker client wrapper.
pub struct DockerClient {
    client: Docker,
    default_image: String,
}

impl DockerClient {
    /// Create a new Docker client.
    ///
    /// # Errors
    ///
    /// Returns an error if Docker is not available.
    pub fn new() -> Result<Self, SandboxError> {
        let client = Docker::connect_with_local_defaults()
            .map_err(|e| SandboxError::RuntimeNotAvailable(e.to_string()))?;

        Ok(Self {
            client,
            default_image: DEFAULT_IMAGE.to_string(),
        })
    }

    /// Set the default image.
    pub fn set_image(&mut self, image: impl Into<String>) {
        self.default_image = image.into();
    }

    /// Check if Docker is available.
    pub async fn health_check(&self) -> Result<(), SandboxError> {
        self.client
            .ping()
            .await
            .map_err(|e| SandboxError::RuntimeNotAvailable(e.to_string()))?;
        Ok(())
    }

    /// Pull an image if not present.
    pub async fn ensure_image(&self, image: &str) -> Result<(), SandboxError> {
        // Check if image exists
        if self.client.inspect_image(image).await.is_ok() {
            return Ok(());
        }

        // Pull the image
        let options = Some(CreateImageOptions {
            from_image: image,
            ..Default::default()
        });

        let mut stream = self.client.create_image(options, None, None);
        while let Some(result) = stream.next().await {
            if let Err(e) = result {
                return Err(SandboxError::ImagePullFailed(e.to_string()));
            }
        }

        Ok(())
    }

    /// Execute a command in a container.
    pub async fn execute(
        &self,
        command: Vec<String>,
        workdir: &str,
        mount_source: &str,
        mount_target: &str,
        env: HashMap<String, String>,
        timeout: Duration,
        keep_container: bool,
    ) -> Result<ExecutionOutput, SandboxError> {
        let image = &self.default_image;
        self.ensure_image(image).await?;

        let container_name = format!("ckrv-{}", uuid::Uuid::new_v4());

        tracing::info!(
            container_name = %container_name,
            image = %image,
            "Creating Docker container"
        );

        // Convert env to Docker format
        let mut env_vec: Vec<String> = env.into_iter().map(|(k, v)| format!("{k}={v}")).collect();

        // Mount Claude credentials if they exist
        let host_home = std::env::var("HOME").unwrap_or_default();
        
        // Use /home/claude as the container home directory (writable by any user)
        let container_home = "/home/claude".to_string();
        env_vec.push(format!("HOME={}", container_home));

        // Create mounts: workspace + Claude credentials
        let mut mounts = vec![
            // Workspace mount
            Mount {
                target: Some(mount_target.to_string()),
                source: Some(mount_source.to_string()),
                typ: Some(MountTypeEnum::BIND),
                read_only: Some(false),
                ..Default::default()
            },
        ];

        // Mount ~/.claude.json to container home
        let claude_config = format!("{}/.claude.json", host_home);
        if std::path::Path::new(&claude_config).exists() {
            tracing::debug!(path = %claude_config, "Mounting Claude config to {}", container_home);
            mounts.push(Mount {
                target: Some(format!("{}/.claude.json", container_home)),
                source: Some(claude_config),
                typ: Some(MountTypeEnum::BIND),
                read_only: Some(false), // Claude needs write access for token refresh
                ..Default::default()
            });
        }

        // Mount ~/.claude directory to container home
        let claude_dir = format!("{}/.claude", host_home);
        if std::path::Path::new(&claude_dir).exists() {
            tracing::debug!(path = %claude_dir, "Mounting Claude directory to {}", container_home);
            mounts.push(Mount {
                target: Some(format!("{}/.claude", container_home)),
                source: Some(claude_dir),
                typ: Some(MountTypeEnum::BIND),
                read_only: Some(false), // Claude needs write access for sessions
                ..Default::default()
            });
        }

        // Get current user UID:GID for proper permission handling
        // Use `id` command since libc is unsafe
        let uid_gid = std::process::Command::new("id")
            .args(["-u"])
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .map(|s| s.trim().to_string())
            .unwrap_or_else(|| "1000".to_string());

        let gid = std::process::Command::new("id")
            .args(["-g"])
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .map(|s| s.trim().to_string())
            .unwrap_or_else(|| "1000".to_string());

        let user_spec = format!("{}:{}", uid_gid, gid);

        let config = Config {
            image: Some(image.to_string()),
            cmd: Some(command),
            working_dir: Some(workdir.to_string()),
            user: Some(user_spec),
            env: Some(env_vec),
            host_config: Some(HostConfig {
                mounts: Some(mounts),
                network_mode: Some("host".to_string()), // Need network for Claude API
                memory: Some(1024 * 1024 * 1024),       // 1GB limit for Claude
                ..Default::default()
            }),
            ..Default::default()
        };

        let options = Some(CreateContainerOptions {
            name: container_name.clone(),
            platform: None,
        });

        // Create container
        let container = self
            .client
            .create_container(options, config)
            .await
            .map_err(|e| SandboxError::ContainerCreateFailed(e.to_string()))?;

        tracing::info!(
            container_id = %container.id,
            container_name = %container_name,
            "Container created, starting execution"
        );

        // Start container
        self.client
            .start_container(&container.id, None::<StartContainerOptions<String>>)
            .await
            .map_err(|e| SandboxError::ContainerStartFailed(e.to_string()))?;
        
        let start_time = std::time::Instant::now();

        // Get logs with following enabled for real-time streaming
        let log_options = Some(LogsOptions::<String> {
            follow: true,
            stdout: true,
            stderr: true,
            ..Default::default()
        });

        let mut stdout = String::new();
        let mut stderr = String::new();

        // Spawn a task to wait for the container exit code independently
        let client_clone = self.client.clone();
        let container_id_clone = container.id.clone();
        
        let wait_handle = tokio::spawn(async move {
            let wait_options = Some(WaitContainerOptions {
                 condition: "not-running",
            });
            let mut stream = client_clone.wait_container(&container_id_clone, wait_options);
            if let Some(Ok(response)) = stream.next().await {
                 response.status_code
            } else {
                 -1
            }
        });

        // Stream logs in the main task
        let mut log_stream = self.client.logs(&container.id, log_options);
        
        // Use a timeout for the *entire* execution, not just wait
        let log_collection = async {
            // Import Write trait for flush
            use std::io::Write;
            while let Some(Ok(log)) = log_stream.next().await {
                match log {
                    LogOutput::StdOut { message } => {
                        let s = String::from_utf8_lossy(&message);
                        print!("{}", s); // Stream to parent stdout
                        let _ = std::io::stdout().flush(); // Force flush
                        stdout.push_str(&s);
                    }
                    LogOutput::StdErr { message } => {
                        let s = String::from_utf8_lossy(&message);
                        eprint!("{}", s); // Stream to parent stderr
                        let _ = std::io::stderr().flush(); // Force flush
                        stderr.push_str(&s);
                    }
                    _ => {}
                }
            }
        };

        // Run log collection with timeout
        if let Err(_) = tokio::time::timeout(timeout, log_collection).await {
             // Timeout occurred
             let _ = self.client.kill_container::<String>(&container.id, None).await;
        }
        
        // Now wait for the exit code (it should be ready or close to ready)
        // We wrap this in a short timeout just in case
        let exit_code = match tokio::time::timeout(Duration::from_secs(5), wait_handle).await {
            Ok(Ok(code)) => code,
            _ => {
                 // Force kill if still running after log stream ended/timeout
                 let _ = self.client.kill_container::<String>(&container.id, None).await;
                 -1
            }
        };

        let duration = start_time.elapsed();

        // Cleanup container (unless keep_container is set)
        if keep_container {
            tracing::info!(
                container_id = %container.id,
                container_name = %container_name,
                "Keeping container for debugging. Remove manually with: docker rm -f {}",
                container_name
            );
        } else {
            let remove_options = Some(RemoveContainerOptions {
                force: true,
                ..Default::default()
            });
            let _ = self
                .client
                .remove_container(&container.id, remove_options)
                .await;
        }

        Ok(ExecutionOutput {
            exit_code: exit_code as i32,
            stdout,
            stderr,
            duration_ms: duration.as_millis() as u64,
        })
    }
    pub async fn create_session(
        &self,
        workdir: &str,
        mount_source: &str,
        mount_target: &str,
        env: HashMap<String, String>,
    ) -> Result<String, SandboxError> {
        let image = &self.default_image;
        self.ensure_image(image).await?;
        let container_name = format!("ckrv-session-{}", uuid::Uuid::new_v4());

        // Prepare Env and Mounts
        let mut env_vec: Vec<String> = env.into_iter().map(|(k, v)| format!("{k}={v}")).collect();
        let host_home = std::env::var("HOME").unwrap_or_default();
        let container_home = "/home/claude".to_string();
        env_vec.push(format!("HOME={}", container_home));

        let mut mounts = vec![
            Mount {
                target: Some(mount_target.to_string()),
                source: Some(mount_source.to_string()),
                typ: Some(MountTypeEnum::BIND),
                read_only: Some(false),
                ..Default::default()
            },
        ];

        let claude_config = format!("{}/.claude.json", host_home);
        if std::path::Path::new(&claude_config).exists() {
             mounts.push(Mount {
                target: Some(format!("{}/.claude.json", container_home)),
                source: Some(claude_config),
                typ: Some(MountTypeEnum::BIND),
                read_only: Some(false),
                ..Default::default()
            });
        }
        let claude_dir = format!("{}/.claude", host_home);
        if std::path::Path::new(&claude_dir).exists() {
            mounts.push(Mount {
                target: Some(format!("{}/.claude", container_home)),
                source: Some(claude_dir),
                typ: Some(MountTypeEnum::BIND),
                read_only: Some(false),
                ..Default::default()
            });
        }

        let config = Config {
            image: Some(image.to_string()),
            cmd: Some(vec!["tail".to_string(), "-f".to_string(), "/dev/null".to_string()]), 
            working_dir: Some(workdir.to_string()),
            env: Some(env_vec),
            host_config: Some(HostConfig {
                mounts: Some(mounts),
                network_mode: Some("host".to_string()),
                ..Default::default()
            }),
            ..Default::default()
        };

        let options = Some(CreateContainerOptions {
            name: container_name.clone(),
            platform: None,
        });

        let container = self.client.create_container(options, config).await
            .map_err(|e| SandboxError::ContainerCreateFailed(e.to_string()))?;
        
        self.client.start_container(&container.id, None::<StartContainerOptions<String>>).await
            .map_err(|e| SandboxError::ContainerStartFailed(e.to_string()))?;

        Ok(container.id)
    }

    /// Execute a command in an existing session container.
    /// Supports interactive commands via PTY allocation.
    pub async fn exec_in_session(
        &self,
        container_id: &str,
        command: Vec<String>,
        _env: HashMap<String, String>,
    ) -> Result<ExecutionOutput, SandboxError> {
        let exec_config = CreateExecOptions {
            attach_stdout: Some(true),
            attach_stderr: Some(true),
            attach_stdin: Some(true),
            tty: Some(true), // Allocate PTY for interactive commands
            cmd: Some(command),
            ..Default::default()
        };

        let exec = self.client.create_exec(container_id, exec_config).await
            .map_err(|e| SandboxError::ExecutionFailed(format!("Create exec failed: {}", e)))?;

        let stream = self.client.start_exec(&exec.id, None).await
            .map_err(|e| SandboxError::ExecutionFailed(format!("Start exec failed: {}", e)))?;

        let mut stdout = String::new();
        let start_time = std::time::Instant::now();

        // With TTY enabled, both stdout and stderr come through the same stream
        if let StartExecResults::Attached { mut output, .. } = stream {
             while let Some(Ok(msg)) = output.next().await {
                match msg {
                    LogOutput::StdOut { message } => stdout.push_str(&String::from_utf8_lossy(&message)),
                    LogOutput::StdErr { message } => stdout.push_str(&String::from_utf8_lossy(&message)),
                    LogOutput::Console { message } => stdout.push_str(&String::from_utf8_lossy(&message)),
                    _ => {}
                }
             }
        }

        let duration = start_time.elapsed();
        
        let inspect = self.client.inspect_exec(&exec.id).await
             .map_err(|e| SandboxError::ExecutionFailed(format!("Inspect exec failed: {}", e)))?;

        Ok(ExecutionOutput {
            exit_code: inspect.exit_code.unwrap_or(-1) as i32,
            stdout,
            stderr: String::new(), // With TTY, stderr is merged into stdout
            duration_ms: duration.as_millis() as u64,
        })
    }

    /// Stop and remove a session container.
    pub async fn stop_session(&self, container_id: &str) -> Result<(), SandboxError> {
        self.client.remove_container(container_id, Some(RemoveContainerOptions { force: true, ..Default::default() })).await
            .map_err(|e| SandboxError::ExecutionFailed(format!("Failed to remove session: {}", e)))?;
        Ok(())
    }
}

/// Output from container execution.
#[derive(Debug, Clone)]
pub struct ExecutionOutput {
    /// Exit code.
    pub exit_code: i32,
    /// Standard output.
    pub stdout: String,
    /// Standard error.
    pub stderr: String,
    /// Duration in milliseconds.
    pub duration_ms: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_docker_client_creation() {
        let client = DockerClient::new().expect("Docker should be available");
        assert!(
            client.health_check().await.is_ok(),
            "Docker health check failed"
        );
    }

    #[test]
    fn test_execution_output_structure() {
        let output = ExecutionOutput {
            exit_code: 0,
            stdout: "success".to_string(),
            stderr: String::new(),
            duration_ms: 100,
        };

        assert_eq!(output.exit_code, 0);
        assert_eq!(output.stdout, "success");
    }
}
