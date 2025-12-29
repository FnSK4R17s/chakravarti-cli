//! Docker/Podman client wrapper.

use std::collections::HashMap;
use std::time::Duration;

use bollard::container::{
    Config, CreateContainerOptions, LogOutput, LogsOptions, RemoveContainerOptions,
    StartContainerOptions, WaitContainerOptions,
};
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

        // Wait for container with timeout
        let wait_options = Some(WaitContainerOptions {
            condition: "not-running",
        });

        let start_time = std::time::Instant::now();
        let exit_code = tokio::select! {
            result = async {
                let mut stream = self.client.wait_container(&container.id, wait_options);
                if let Some(Ok(response)) = stream.next().await {
                    response.status_code
                } else {
                    -1
                }
            } => result,
            _ = tokio::time::sleep(timeout) => {
                // Kill on timeout
                let _ = self.client.kill_container::<String>(&container.id, None).await;
                -1
            }
        };

        let duration = start_time.elapsed();

        // Get logs
        let log_options = Some(LogsOptions::<String> {
            stdout: true,
            stderr: true,
            ..Default::default()
        });

        let mut stdout = String::new();
        let mut stderr = String::new();

        let mut log_stream = self.client.logs(&container.id, log_options);
        while let Some(Ok(log)) = log_stream.next().await {
            match log {
                LogOutput::StdOut { message } => {
                    stdout.push_str(&String::from_utf8_lossy(&message));
                }
                LogOutput::StdErr { message } => {
                    stderr.push_str(&String::from_utf8_lossy(&message));
                }
                _ => {}
            }
        }

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
