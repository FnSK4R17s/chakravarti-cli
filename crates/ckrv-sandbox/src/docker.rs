//! Docker/Podman client wrapper.

// ============================================================
// IMPORTS
// ============================================================

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

// ============================================================
// CONSTANTS
// ============================================================

/// GHCR registry prefix for pre-built agent images.
pub const GHCR_PREFIX: &str = "ghcr.io/fnsk4r17s";

/// Default Docker image for execution (contains Claude Code CLI).
pub const DEFAULT_IMAGE: &str = "ghcr.io/fnsk4r17s/ckrv-agent:latest";

// ============================================================
// TYPES
// ============================================================

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
    pub fn set_image(&mut self, image: &str) {
        image.clone_into(&mut self.default_image);
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
        extra_mounts: Vec<crate::executor::BindMount>,
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

        // Terminal environment — Claude Code needs these to detect capabilities and enable tool use
        env_vec.push("TERM=xterm-256color".to_string());
        env_vec.push("COLORTERM=truecolor".to_string());
        env_vec.push("COLUMNS=120".to_string());
        env_vec.push("LINES=30".to_string());

        // Mount Claude credentials if they exist
        let _host_home = std::env::var("HOME").unwrap_or_default();

        // Use the agent user's home directory where .claude config lives
        let container_home = "/home/agent".to_string();
        env_vec.push(format!("HOME={}", container_home));

        // Create mounts: workspace + agent-specific credential mounts
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

        // Add agent-specific credential mounts (provided by caller)
        for bind in &extra_mounts {
            mounts.push(Mount {
                target: Some(bind.target.clone()),
                source: Some(bind.source.clone()),
                typ: Some(MountTypeEnum::BIND),
                read_only: Some(bind.read_only),
                ..Default::default()
            });
        }

        // Run as root so we can chown the workspace to the agent user,
        // then drop to the agent user via the entrypoint/command wrapper.
        // The image's default user is "agent" (uid 1001) but the workspace
        // mount is owned by the host user (typically uid 1000 = "node" in container).
        // We prepend a chown to fix permissions before running the actual command.
        //
        // The incoming `command` is typically ["sh", "-c", "claude -p '...' --flags"].
        // Extract the inner shell command (3rd element) to avoid double sh -c wrapping.
        let inner_cmd = if command.len() == 3 && command[0] == "sh" && command[1] == "-c" {
            command[2].clone()
        } else {
            command.join(" ")
        };
        let wrapped_command = vec![
            "sh".to_string(),
            "-c".to_string(),
            format!(
                "chown -R agent:agent {} 2>/dev/null; exec su -s /bin/sh agent -c '{}'",
                mount_target,
                inner_cmd.replace('\'', "'\\''"),
            ),
        ];

        let config = Config {
            image: Some(image.clone()),
            cmd: Some(wrapped_command),
            working_dir: Some(workdir.to_string()),
            user: Some("root".to_string()), // Root for chown, then su to agent
            tty: Some(true),
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
        if tokio::time::timeout(timeout, log_collection).await.is_err() {
            // Timeout occurred
            let _ = self
                .client
                .kill_container::<String>(&container.id, None)
                .await;
        }

        // Now wait for the exit code (it should be ready or close to ready)
        // We wrap this in a short timeout just in case
        let exit_code =
            if let Ok(Ok(code)) = tokio::time::timeout(Duration::from_secs(5), wait_handle).await {
                code
            } else {
                // Force kill if still running after log stream ended/timeout
                let _ = self
                    .client
                    .kill_container::<String>(&container.id, None)
                    .await;
                -1
            };

        let duration = start_time.elapsed();

        // Restore workspace ownership to host user after container exits.
        restore_workspace_ownership(mount_source);

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

    /// Execute a command in a container with real-time log streaming.
    ///
    /// Unlike `execute()`, this method calls `on_log` for each line of output
    /// as it arrives, enabling real-time streaming to the UI.
    pub async fn execute_streaming<F>(
        &self,
        command: Vec<String>,
        workdir: &str,
        mount_source: &str,
        mount_target: &str,
        env: HashMap<String, String>,
        timeout: Duration,
        keep_container: bool,
        extra_mounts: Vec<crate::executor::BindMount>,
        mut on_log: F,
    ) -> Result<ExecutionOutput, SandboxError>
    where
        F: FnMut(&str, bool) + Send, // (line, is_stderr)
    {
        let image = &self.default_image;
        self.ensure_image(image).await?;

        let container_name = format!("ckrv-{}", uuid::Uuid::new_v4());

        tracing::info!(
            container_name = %container_name,
            image = %image,
            "Creating Docker container with streaming"
        );

        // Convert env to Docker format
        let mut env_vec: Vec<String> = env.iter().map(|(k, v)| format!("{k}={v}")).collect();

        // Terminal environment — Claude Code needs these to detect capabilities and enable tool use
        env_vec.push("TERM=xterm-256color".to_string());
        env_vec.push("COLORTERM=truecolor".to_string());
        env_vec.push("COLUMNS=120".to_string());
        env_vec.push("LINES=30".to_string());

        // Mount credentials if they exist
        let _host_home = std::env::var("HOME").unwrap_or_default();

        // Use HOME from passed env, or default to /home/claude
        let container_home = env
            .get("HOME")
            .cloned()
            .unwrap_or_else(|| "/home/claude".to_string());

        // Only add HOME if not already in env
        if !env.contains_key("HOME") {
            env_vec.push(format!("HOME={}", container_home));
        }

        // Create mounts: workspace + agent-specific credential mounts
        let mut mounts = vec![Mount {
            target: Some(mount_target.to_string()),
            source: Some(mount_source.to_string()),
            typ: Some(MountTypeEnum::BIND),
            read_only: Some(false),
            ..Default::default()
        }];

        // Add agent-specific credential mounts (provided by caller)
        for bind in &extra_mounts {
            mounts.push(Mount {
                target: Some(bind.target.clone()),
                source: Some(bind.source.clone()),
                typ: Some(MountTypeEnum::BIND),
                read_only: Some(bind.read_only),
                ..Default::default()
            });
        }

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
            image: Some(image.clone()),
            cmd: Some(command),
            working_dir: Some(workdir.to_string()),
            user: Some(user_spec),
            tty: Some(true), // Claude Code needs a TTY for tool execution
            env: Some(env_vec),
            host_config: Some(HostConfig {
                mounts: Some(mounts),
                network_mode: Some("host".to_string()),
                memory: Some(1024 * 1024 * 1024),
                ..Default::default()
            }),
            ..Default::default()
        };

        let options = Some(CreateContainerOptions {
            name: container_name.clone(),
            platform: None,
        });

        let container = self
            .client
            .create_container(options, config)
            .await
            .map_err(|e| SandboxError::ContainerCreateFailed(e.to_string()))?;

        self.client
            .start_container(&container.id, None::<StartContainerOptions<String>>)
            .await
            .map_err(|e| SandboxError::ContainerStartFailed(e.to_string()))?;

        let start_time = std::time::Instant::now();

        let log_options = Some(LogsOptions::<String> {
            follow: true,
            stdout: true,
            stderr: true,
            ..Default::default()
        });

        let mut stdout = String::new();
        let mut stderr = String::new();

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

        let mut log_stream = self.client.logs(&container.id, log_options);

        let log_collection = async {
            while let Some(Ok(log)) = log_stream.next().await {
                match log {
                    LogOutput::StdOut { message } => {
                        let s = String::from_utf8_lossy(&message);
                        for line in s.lines() {
                            on_log(line, false);
                        }
                        stdout.push_str(&s);
                    }
                    LogOutput::StdErr { message } => {
                        let s = String::from_utf8_lossy(&message);
                        for line in s.lines() {
                            on_log(line, true);
                        }
                        stderr.push_str(&s);
                    }
                    _ => {}
                }
            }
        };

        if tokio::time::timeout(timeout, log_collection).await.is_err() {
            let _ = self
                .client
                .kill_container::<String>(&container.id, None)
                .await;
        }

        let exit_code =
            if let Ok(Ok(code)) = tokio::time::timeout(Duration::from_secs(5), wait_handle).await {
                code
            } else {
                let _ = self
                    .client
                    .kill_container::<String>(&container.id, None)
                    .await;
                -1
            };

        let duration = start_time.elapsed();

        // Restore workspace ownership to host user after container exits
        restore_workspace_ownership(mount_source);

        if keep_container {
            tracing::info!(container_id = %container.id, "Keeping container for debugging");
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

    /// Create a long-lived session container for interactive command execution.
    ///
    /// # Errors
    ///
    /// Returns an error if the container cannot be created or started.
    pub async fn create_session(
        &self,
        workdir: &str,
        mount_source: &str,
        mount_target: &str,
        env: HashMap<String, String>,
        extra_mounts: Vec<crate::executor::BindMount>,
    ) -> Result<String, SandboxError> {
        let image = &self.default_image;
        self.ensure_image(image).await?;
        let container_name = format!("ckrv-session-{}", uuid::Uuid::new_v4());

        // Prepare Env and Mounts
        let env_vec: Vec<String> = env.into_iter().map(|(k, v)| format!("{k}={v}")).collect();

        let mut mounts = vec![Mount {
            target: Some(mount_target.to_string()),
            source: Some(mount_source.to_string()),
            typ: Some(MountTypeEnum::BIND),
            read_only: Some(false),
            ..Default::default()
        }];

        // Add agent-specific credential mounts (provided by caller)
        for bind in &extra_mounts {
            mounts.push(Mount {
                target: Some(bind.target.clone()),
                source: Some(bind.source.clone()),
                typ: Some(MountTypeEnum::BIND),
                read_only: Some(bind.read_only),
                ..Default::default()
            });
        }

        // Get current user UID:GID for proper permission handling
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
            image: Some(image.clone()),
            cmd: Some(vec![
                "tail".to_string(),
                "-f".to_string(),
                "/dev/null".to_string(),
            ]),
            working_dir: Some(workdir.to_string()),
            user: Some(user_spec),
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

        let container = self
            .client
            .create_container(options, config)
            .await
            .map_err(|e| SandboxError::ContainerCreateFailed(e.to_string()))?;

        self.client
            .start_container(&container.id, None::<StartContainerOptions<String>>)
            .await
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

        let exec = self
            .client
            .create_exec(container_id, exec_config)
            .await
            .map_err(|e| SandboxError::ExecutionFailed(format!("Create exec failed: {}", e)))?;

        let stream = self
            .client
            .start_exec(&exec.id, None)
            .await
            .map_err(|e| SandboxError::ExecutionFailed(format!("Start exec failed: {}", e)))?;

        let mut stdout = String::new();
        let start_time = std::time::Instant::now();

        // With TTY enabled, both stdout and stderr come through the same stream
        if let StartExecResults::Attached { mut output, .. } = stream {
            while let Some(Ok(msg)) = output.next().await {
                match msg {
                    LogOutput::StdOut { message }
                    | LogOutput::StdErr { message }
                    | LogOutput::Console { message } => {
                        stdout.push_str(&String::from_utf8_lossy(&message));
                    }
                    LogOutput::StdIn { .. } => {}
                }
            }
        }

        let duration = start_time.elapsed();

        let inspect =
            self.client.inspect_exec(&exec.id).await.map_err(|e| {
                SandboxError::ExecutionFailed(format!("Inspect exec failed: {}", e))
            })?;

        Ok(ExecutionOutput {
            exit_code: inspect.exit_code.unwrap_or(-1) as i32,
            stdout,
            stderr: String::new(), // With TTY, stderr is merged into stdout
            duration_ms: duration.as_millis() as u64,
        })
    }

    /// Stop and remove a session container.
    pub async fn stop_session(&self, container_id: &str) -> Result<(), SandboxError> {
        self.client
            .remove_container(
                container_id,
                Some(RemoveContainerOptions {
                    force: true,
                    ..Default::default()
                }),
            )
            .await
            .map_err(|e| {
                SandboxError::ExecutionFailed(format!("Failed to remove session: {}", e))
            })?;
        Ok(())
    }
}

// ============================================================
// OUTPUT TYPES
// ============================================================

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

// ============================================================
// TESTS
// ============================================================

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

/// Restore workspace ownership to the host user after a container exits.
///
/// Container processes run as "agent" (uid 1001), so files they create are
/// owned by that uid. The host user needs ownership for git operations.
/// Uses a Docker alpine container since the host user can't chown files
/// owned by a different uid without root.
fn restore_workspace_ownership(mount_source: &str) {
    let owner = std::process::Command::new("id")
        .arg("-u")
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "1000".to_string());
    let group = std::process::Command::new("id")
        .arg("-g")
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "1000".to_string());
    let _ = std::process::Command::new("docker")
        .args([
            "run",
            "--rm",
            "-v",
            &format!("{mount_source}:/fix"),
            "alpine",
            "chown",
            "-R",
            &format!("{owner}:{group}"),
            "/fix",
        ])
        .status();
}
