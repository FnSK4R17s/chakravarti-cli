//! # Session Handler
//!
//! Handlers for persistent Docker session management.

use crate::error::TransportError;
use crate::state::AppState;
use ckrv_sandbox::docker::DockerClient;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;

// ============================================================================
// Global Session Store
// ============================================================================

/// Global session store: Maps SessionID -> ContainerID.
static SESSIONS: Lazy<Mutex<HashMap<String, String>>> = Lazy::new(|| Mutex::new(HashMap::new()));

// ============================================================================
// Request/Response Types
// ============================================================================

/// Request to start a new session.
#[derive(Debug, Deserialize)]
pub struct StartSessionRequest {
    /// Unique identifier for the session.
    pub session_id: String,
}

/// Response from starting a session.
#[derive(Debug, Serialize)]
pub struct StartSessionResponse {
    /// Session identifier.
    pub session_id: String,
    /// Docker container ID if created.
    pub container_id: Option<String>,
}

/// Request to execute in a session.
#[derive(Debug, Deserialize)]
pub struct ExecRequest {
    /// Session to execute in.
    pub session_id: String,
    /// Shell command to run.
    pub command: String,
}

/// Response from exec.
#[derive(Debug, Serialize)]
pub struct ExecResponse {
    /// Whether the command exited with code 0.
    pub success: bool,
    /// Standard output from the command.
    pub stdout: String,
    /// Standard error from the command.
    pub stderr: String,
    /// Process exit code.
    pub exit_code: i32,
    /// Execution duration in milliseconds.
    pub duration_ms: u64,
}

/// Request to stop a session.
#[derive(Debug, Deserialize)]
pub struct StopSessionRequest {
    /// Session to stop.
    pub session_id: String,
}

// ============================================================================
// Handlers
// ============================================================================

/// Start a persistent sandbox session.
pub async fn start_session_handler(
    state: &AppState,
    request: StartSessionRequest,
) -> Result<StartSessionResponse, TransportError> {
    // Check if session already exists
    {
        let sessions = SESSIONS
            .lock()
            .map_err(|e| TransportError::Internal(format!("Session lock poisoned: {e}")))?;
        if let Some(container_id) = sessions.get(&request.session_id) {
            return Ok(StartSessionResponse {
                session_id: request.session_id,
                container_id: Some(container_id.clone()),
            });
        }
    }

    // Create Docker client
    let client = DockerClient::new()
        .map_err(|e| TransportError::ServiceUnavailable(format!("Docker not available: {e}")))?;

    // Create session container
    let cwd = state.project_root.to_string_lossy().to_string();
    let env: HashMap<String, String> = HashMap::new();

    let container_id = client
        .create_session("/workspace", &cwd, "/workspace", env, Vec::new())
        .await
        .map_err(|e| TransportError::Internal(format!("Failed to create session: {e}")))?;

    // Store session
    {
        let mut sessions = SESSIONS
            .lock()
            .map_err(|e| TransportError::Internal(format!("Session lock poisoned: {e}")))?;
        sessions.insert(request.session_id.clone(), container_id.clone());
    }

    Ok(StartSessionResponse {
        session_id: request.session_id,
        container_id: Some(container_id),
    })
}

/// Execute a command in an existing session.
pub async fn exec_in_session_handler(
    _state: &AppState,
    request: ExecRequest,
) -> Result<ExecResponse, TransportError> {
    // Look up container ID
    let container_id = {
        let sessions = SESSIONS
            .lock()
            .map_err(|e| TransportError::Internal(format!("Session lock poisoned: {e}")))?;
        sessions.get(&request.session_id).cloned()
    };

    let container_id = container_id.ok_or_else(|| {
        TransportError::NotFound("No active session found. Start a session first.".to_string())
    })?;

    // Create Docker client
    let client = DockerClient::new()
        .map_err(|e| TransportError::ServiceUnavailable(format!("Docker error: {e}")))?;

    // Execute command
    let command = vec!["sh".to_string(), "-c".to_string(), request.command];
    let env: HashMap<String, String> = HashMap::new();

    let result = client
        .exec_in_session(&container_id, command, env)
        .await
        .map_err(|e| TransportError::Internal(format!("Execution failed: {e}")))?;

    Ok(ExecResponse {
        success: result.exit_code == 0,
        stdout: result.stdout,
        stderr: result.stderr,
        exit_code: result.exit_code,
        duration_ms: result.duration_ms,
    })
}

/// Stop and clean up a session.
pub async fn stop_session_handler(
    _state: &AppState,
    request: StopSessionRequest,
) -> Result<(), TransportError> {
    // Remove from store
    let container_id = {
        let mut sessions = SESSIONS
            .lock()
            .map_err(|e| TransportError::Internal(format!("Session lock poisoned: {e}")))?;
        sessions.remove(&request.session_id)
    };

    let Some(container_id) = container_id else {
        return Ok(()); // Already stopped
    };

    // Stop container
    let client = DockerClient::new()
        .map_err(|e| TransportError::ServiceUnavailable(format!("Docker error: {e}")))?;

    client
        .stop_session(&container_id)
        .await
        .map_err(|e| TransportError::Internal(format!("Failed to stop session: {e}")))?;

    Ok(())
}

/// Get active session info.
pub fn get_session_handler(
    _state: &AppState,
    session_id: String,
) -> Result<Option<String>, TransportError> {
    let sessions = SESSIONS
        .lock()
        .map_err(|e| TransportError::Internal(format!("Session lock poisoned: {e}")))?;
    Ok(sessions.get(&session_id).cloned())
}

/// List all active sessions.
pub fn list_sessions_handler(_state: &AppState) -> Result<Vec<(String, String)>, TransportError> {
    let sessions = SESSIONS
        .lock()
        .map_err(|e| TransportError::Internal(format!("Session lock poisoned: {e}")))?;
    Ok(sessions
        .iter()
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect())
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[tokio::test]
    async fn test_list_sessions_handler() {
        let state = AppState::new(PathBuf::from("/tmp/test-session"));
        let result = list_sessions_handler(&state);
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }
}
