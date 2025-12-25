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

/// Default Docker image for execution.
pub const DEFAULT_IMAGE: &str = "rust:1.83-slim";

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
    ) -> Result<ExecutionOutput, SandboxError> {
        let image = &self.default_image;
        self.ensure_image(image).await?;

        let container_name = format!("ckrv-{}", uuid::Uuid::new_v4());

        // Convert env to Docker format
        let env_vec: Vec<String> = env.into_iter().map(|(k, v)| format!("{k}={v}")).collect();

        // Create mount
        let mounts = vec![Mount {
            target: Some(mount_target.to_string()),
            source: Some(mount_source.to_string()),
            typ: Some(MountTypeEnum::BIND),
            read_only: Some(false),
            ..Default::default()
        }];

        let config = Config {
            image: Some(image.to_string()),
            cmd: Some(command),
            working_dir: Some(workdir.to_string()),
            env: Some(env_vec),
            host_config: Some(HostConfig {
                mounts: Some(mounts),
                network_mode: Some("none".to_string()), // No network access
                memory: Some(512 * 1024 * 1024),        // 512MB limit
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

        // Cleanup container
        let remove_options = Some(RemoveContainerOptions {
            force: true,
            ..Default::default()
        });
        let _ = self
            .client
            .remove_container(&container.id, remove_options)
            .await;

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
        let result = DockerClient::new();
        // May fail if Docker is not available
        if result.is_ok() {
            let client = result.unwrap();
            assert!(client.health_check().await.is_ok());
        }
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
