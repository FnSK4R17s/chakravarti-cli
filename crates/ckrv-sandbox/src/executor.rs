//! Sandboxed command execution.

use std::{collections::HashMap, path::PathBuf, time::Duration};

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::{docker::DockerClient, AllowList, SandboxError};

/// Configuration for command execution.
#[derive(Debug, Clone)]
pub struct ExecuteConfig {
    /// Command to run.
    pub command: Vec<String>,
    /// Working directory inside sandbox.
    pub workdir: PathBuf,
    /// Worktree to mount.
    pub mount: PathBuf,
    /// Environment variables.
    pub env: HashMap<String, String>,
    /// Timeout.
    pub timeout: Duration,
    /// Keep container after execution (for debugging).
    pub keep_container: bool,
}

impl ExecuteConfig {
    /// Create a new config.
    #[must_use]
    pub fn new(command: impl Into<String>, mount: PathBuf) -> Self {
        Self {
            command: vec![command.into()],
            workdir: PathBuf::from("/workspace"),
            mount,
            env: HashMap::new(),
            timeout: Duration::from_secs(300),
            keep_container: false,
        }
    }

    /// Set command as shell command.
    #[must_use]
    pub fn shell(mut self, cmd: impl Into<String>) -> Self {
        self.command = vec!["sh".to_string(), "-c".to_string(), cmd.into()];
        self
    }

    /// Add environment variable.
    #[must_use]
    pub fn env(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.env.insert(key.into(), value.into());
        self
    }

    /// Set timeout.
    #[must_use]
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Keep container after execution (for debugging).
    #[must_use]
    pub fn with_keep_container(mut self, keep: bool) -> Self {
        self.keep_container = keep;
        self
    }
}

/// Result of command execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecuteResult {
    /// Exit code.
    pub exit_code: i32,
    /// Standard output.
    pub stdout: String,
    /// Standard error.
    pub stderr: String,
    /// Execution duration.
    pub duration_ms: u64,
}

impl ExecuteResult {
    /// Check if execution succeeded.
    #[must_use]
    pub fn success(&self) -> bool {
        self.exit_code == 0
    }

    /// Combined output.
    #[must_use]
    pub fn combined_output(&self) -> String {
        format!("{}\n{}", self.stdout, self.stderr)
    }
}

/// Trait for sandboxed execution.
#[async_trait]
pub trait Sandbox: Send + Sync {
    /// Execute a command in the sandbox.
    ///
    /// # Errors
    ///
    /// Returns an error if execution fails.
    async fn execute(&self, config: ExecuteConfig) -> Result<ExecuteResult, SandboxError>;

    /// Check if sandbox runtime is available.
    ///
    /// # Errors
    ///
    /// Returns an error if runtime is not available.
    async fn health_check(&self) -> Result<(), SandboxError>;
}

/// Docker-backed sandbox implementation.
pub struct DockerSandbox {
    client: DockerClient,
    allowlist: Box<dyn AllowList>,
}

impl DockerSandbox {
    /// Create a new Docker sandbox.
    ///
    /// # Errors
    ///
    /// Returns an error if Docker is not available.
    pub fn new(allowlist: impl AllowList + 'static) -> Result<Self, SandboxError> {
        let client = DockerClient::new()?;
        Ok(Self {
            client,
            allowlist: Box::new(allowlist),
        })
    }

    /// Create with default allowlist.
    ///
    /// # Errors
    ///
    /// Returns an error if Docker is not available.
    pub fn with_defaults() -> Result<Self, SandboxError> {
        Self::new(crate::allowlist::DefaultAllowList::default())
    }

    /// Get reference to inner Docker client.
    pub fn inner_client(&self) -> &DockerClient {
        &self.client
    }
}

#[async_trait]
impl Sandbox for DockerSandbox {
    async fn execute(&self, config: ExecuteConfig) -> Result<ExecuteResult, SandboxError> {
        // Check allowlist
        if !self.allowlist.is_allowed(&config.command) {
            return Err(SandboxError::CommandNotAllowed(config.command.join(" ")));
        }

        let workdir = config.workdir.to_string_lossy().to_string();
        let mount_source = config.mount.to_string_lossy().to_string();

        let output = self
            .client
            .execute(
                config.command,
                &workdir,
                &mount_source,
                &workdir,
                config.env,
                config.timeout,
                config.keep_container,
            )
            .await?;

        Ok(ExecuteResult {
            exit_code: output.exit_code,
            stdout: output.stdout,
            stderr: output.stderr,
            duration_ms: output.duration_ms,
        })
    }

    async fn health_check(&self) -> Result<(), SandboxError> {
        self.client.health_check().await
    }
}

/// Local (non-sandboxed) execution for development.
pub struct LocalSandbox;

impl LocalSandbox {
    /// Create a new local sandbox.
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl Default for LocalSandbox {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Sandbox for LocalSandbox {
    async fn execute(&self, config: ExecuteConfig) -> Result<ExecuteResult, SandboxError> {
        use std::process::Command;
        use std::time::Instant;

        let start = Instant::now();

        // If command starts with "sh -c", use it directly
        // Otherwise wrap in sh -c
        let output = if config.command.len() >= 2
            && config.command[0] == "sh"
            && config.command[1] == "-c"
        {
            Command::new(&config.command[0])
                .args(&config.command[1..])
                .current_dir(&config.mount)
                .envs(config.env)
                .output()
                .map_err(|e| SandboxError::ExecutionFailed(e.to_string()))?
        } else {
            Command::new("sh")
                .args(["-c", &config.command.join(" ")])
                .current_dir(&config.mount)
                .envs(config.env)
                .output()
                .map_err(|e| SandboxError::ExecutionFailed(e.to_string()))?
        };

        let duration = start.elapsed();

        Ok(ExecuteResult {
            exit_code: output.status.code().unwrap_or(-1),
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            duration_ms: duration.as_millis() as u64,
        })
    }

    async fn health_check(&self) -> Result<(), SandboxError> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_execute_config_new() {
        let config = ExecuteConfig::new("cargo test", PathBuf::from("/tmp"));
        assert_eq!(config.command, vec!["cargo test"]);
    }

    #[test]
    fn test_execute_config_shell() {
        let config =
            ExecuteConfig::new("", PathBuf::from("/tmp")).shell("echo hello && echo world");
        assert_eq!(config.command[0], "sh");
        assert_eq!(config.command[2], "echo hello && echo world");
    }

    #[test]
    fn test_execute_result_success() {
        let result = ExecuteResult {
            exit_code: 0,
            stdout: "ok".to_string(),
            stderr: String::new(),
            duration_ms: 100,
        };
        assert!(result.success());
    }

    #[test]
    fn test_execute_result_failure() {
        let result = ExecuteResult {
            exit_code: 1,
            stdout: String::new(),
            stderr: "error".to_string(),
            duration_ms: 50,
        };
        assert!(!result.success());
    }

    #[tokio::test]
    async fn test_local_sandbox_simple_command() {
        let dir = TempDir::new().expect("temp dir");
        let sandbox = LocalSandbox::new();

        let config = ExecuteConfig::new("", dir.path().to_path_buf()).shell("echo hello");

        let result = sandbox.execute(config).await.expect("execute");
        assert!(result.success());
        assert!(result.stdout.contains("hello"));
    }

    #[tokio::test]
    async fn test_local_sandbox_health_check() {
        let sandbox = LocalSandbox::new();
        assert!(sandbox.health_check().await.is_ok());
    }
}
