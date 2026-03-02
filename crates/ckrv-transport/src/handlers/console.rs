//! # Console Handler
//!
//! Handlers for interactive command execution.

use crate::error::TransportError;
use crate::state::AppState;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

// ============================================================
// Request/Response Types
// ============================================================

/// Command execution request.
#[derive(Debug, Deserialize)]
pub struct ExecuteCommandRequest {
    /// Optional agent ID for session tracking
    pub agent_id: Option<String>,
    /// Command to execute
    pub command: String,
    /// Working directory (optional)
    pub cwd: Option<String>,
    /// Environment variables (optional)
    pub env: Option<HashMap<String, String>>,
    /// Whether to use Docker sandbox
    pub use_sandbox: bool,
    /// Keep container alive for session
    #[serde(default)]
    pub keep_container: bool,
}

/// Command execution response.
#[derive(Debug, Serialize)]
pub struct ExecuteCommandResponse {
    /// Whether the command exited with code 0.
    pub success: bool,
    /// Standard output from the command.
    pub stdout: String,
    /// Standard error from the command.
    pub stderr: String,
    /// Process exit code.
    pub exit_code: i32,
    /// Additional status message.
    pub message: Option<String>,
}

// ============================================================
// Handlers
// ============================================================

/// Execute a command in the project context.
///
/// Supports both local execution and Docker sandbox execution.
pub async fn execute_command_handler(
    state: &AppState,
    request: ExecuteCommandRequest,
) -> Result<ExecuteCommandResponse, TransportError> {
    let cwd = request
        .cwd
        .map(PathBuf::from)
        .unwrap_or_else(|| state.project_root.clone());

    let env = request.env.unwrap_or_default();

    if request.use_sandbox {
        // Use Docker sandbox for isolation
        execute_in_sandbox(
            &request.command,
            &cwd,
            env,
            request.agent_id,
            request.keep_container,
        )
        .await
    } else {
        // Execute locally
        execute_locally(&request.command, &cwd, env)
    }
}

/// Execute command locally.
fn execute_locally(
    command: &str,
    cwd: &Path,
    env: HashMap<String, String>,
) -> Result<ExecuteCommandResponse, TransportError> {
    let output = std::process::Command::new("sh")
        .args(["-c", command])
        .current_dir(cwd)
        .envs(env)
        .output()
        .map_err(|e| TransportError::Internal(format!("Failed to execute command: {e}")))?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let exit_code = output.status.code().unwrap_or(-1);

    Ok(ExecuteCommandResponse {
        success: exit_code == 0,
        stdout,
        stderr,
        exit_code,
        message: None,
    })
}

/// Execute command in Docker sandbox.
async fn execute_in_sandbox(
    command: &str,
    cwd: &Path,
    env: HashMap<String, String>,
    _agent_id: Option<String>,
    keep_container: bool,
) -> Result<ExecuteCommandResponse, TransportError> {
    use ckrv_sandbox::{DefaultAllowList, DockerSandbox, ExecuteConfig, Sandbox};

    let config = ExecuteConfig {
        command: vec!["sh".to_string(), "-c".to_string(), command.to_string()],
        workdir: PathBuf::from("/workspace"),
        mount: cwd.to_path_buf(),
        env,
        timeout: std::time::Duration::from_secs(30),
        keep_container,
        extra_mounts: Vec::new(),
    };

    let allowlist = DefaultAllowList::default();
    let sandbox = DockerSandbox::new(allowlist)
        .map_err(|e| TransportError::Internal(format!("Failed to create sandbox: {e}")))?;

    // For now, execute without session support
    // Session support requires additional wiring in ckrv-sandbox
    let result = sandbox
        .execute(config)
        .await
        .map_err(|e| TransportError::Internal(format!("Sandbox execution failed: {e}")))?;

    Ok(ExecuteCommandResponse {
        success: result.exit_code == 0,
        stdout: result.stdout,
        stderr: result.stderr,
        exit_code: result.exit_code,
        message: None,
    })
}

// ============================================================
// Tests
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_execute_locally() {
        let cwd = std::env::current_dir().unwrap();
        let result = execute_locally("echo hello", &cwd, HashMap::new());
        assert!(result.is_ok());
        let response = result.unwrap();
        assert!(response.success);
        assert!(response.stdout.contains("hello"));
    }
}
