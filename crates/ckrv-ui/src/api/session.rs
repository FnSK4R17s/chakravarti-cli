//! Session-based sandbox execution API.
//!
//! Provides persistent container sessions where multiple commands
//! can be executed in the same environment.

use axum::{Json, response::IntoResponse, extract::State};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;
use once_cell::sync::Lazy;
use crate::state::AppState;
use ckrv_sandbox::docker::DockerClient;

// Global session store: Map<SessionID, ContainerID>
static SESSIONS: Lazy<Mutex<HashMap<String, String>>> = Lazy::new(|| {
    Mutex::new(HashMap::new())
});

/// Request to start a new session
#[derive(Debug, Deserialize)]
pub struct StartSessionRequest {
    pub session_id: String, // Usually agent_id or a unique modal ID
}

/// Response from starting a session
#[derive(Debug, Serialize)]
pub struct StartSessionResponse {
    pub success: bool,
    pub session_id: String,
    pub container_id: Option<String>,
    pub message: Option<String>,
}

/// Request to execute in a session
#[derive(Debug, Deserialize)]
pub struct ExecRequest {
    pub session_id: String,
    pub command: String,
}

/// Response from exec
#[derive(Debug, Serialize)]
pub struct ExecResponse {
    pub success: bool,
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
    pub duration_ms: u64,
    pub message: Option<String>,
}

/// Request to stop a session
#[derive(Debug, Deserialize)]
pub struct StopSessionRequest {
    pub session_id: String,
}

/// Response from stopping a session
#[derive(Debug, Serialize)]
pub struct StopSessionResponse {
    pub success: bool,
    pub message: Option<String>,
}

/// Start a persistent sandbox session
pub async fn start_session(
    State(state): State<AppState>,
    Json(payload): Json<StartSessionRequest>,
) -> impl IntoResponse {
    // Check if session already exists
    {
        let sessions = SESSIONS.lock().unwrap();
        if let Some(container_id) = sessions.get(&payload.session_id) {
            return Json(StartSessionResponse {
                success: true,
                session_id: payload.session_id,
                container_id: Some(container_id.clone()),
                message: Some("Session already exists".to_string()),
            });
        }
    }

    // Create Docker client
    let client = match DockerClient::new() {
        Ok(c) => c,
        Err(e) => {
            return Json(StartSessionResponse {
                success: false,
                session_id: payload.session_id,
                container_id: None,
                message: Some(format!("Docker not available: {}", e)),
            });
        }
    };

    // Create session container
    let cwd = state.project_root.to_string_lossy().to_string();
    let env: HashMap<String, String> = HashMap::new();

    match client.create_session("/workspace", &cwd, "/workspace", env).await {
        Ok(container_id) => {
            // Store session
            {
                let mut sessions = SESSIONS.lock().unwrap();
                sessions.insert(payload.session_id.clone(), container_id.clone());
            }

            println!("Session started: {} -> {}", payload.session_id, container_id);

            Json(StartSessionResponse {
                success: true,
                session_id: payload.session_id,
                container_id: Some(container_id),
                message: Some("Session created".to_string()),
            })
        }
        Err(e) => Json(StartSessionResponse {
            success: false,
            session_id: payload.session_id,
            container_id: None,
            message: Some(format!("Failed to create session: {}", e)),
        }),
    }
}

/// Execute a command in an existing session
pub async fn exec_in_session(
    Json(payload): Json<ExecRequest>,
) -> impl IntoResponse {
    // Look up container ID
    let container_id = {
        let sessions = SESSIONS.lock().unwrap();
        sessions.get(&payload.session_id).cloned()
    };

    let container_id = match container_id {
        Some(id) => id,
        None => {
            return Json(ExecResponse {
                success: false,
                stdout: String::new(),
                stderr: "No active session found. Start a session first.".to_string(),
                exit_code: -1,
                duration_ms: 0,
                message: Some("Session not found".to_string()),
            });
        }
    };

    // Create Docker client
    let client = match DockerClient::new() {
        Ok(c) => c,
        Err(e) => {
            return Json(ExecResponse {
                success: false,
                stdout: String::new(),
                stderr: format!("Docker error: {}", e),
                exit_code: -1,
                duration_ms: 0,
                message: Some("Docker not available".to_string()),
            });
        }
    };

    // Execute command
    let command = vec!["sh".to_string(), "-c".to_string(), payload.command];
    let env: HashMap<String, String> = HashMap::new();

    match client.exec_in_session(&container_id, command, env).await {
        Ok(result) => {
            println!("Exec in {}: exit_code={}", container_id, result.exit_code);
            Json(ExecResponse {
                success: result.exit_code == 0,
                stdout: result.stdout,
                stderr: result.stderr,
                exit_code: result.exit_code,
                duration_ms: result.duration_ms,
                message: None,
            })
        }
        Err(e) => Json(ExecResponse {
            success: false,
            stdout: String::new(),
            stderr: e.to_string(),
            exit_code: -1,
            duration_ms: 0,
            message: Some(format!("Execution failed: {}", e)),
        }),
    }
}

/// Stop and clean up a session
pub async fn stop_session(
    Json(payload): Json<StopSessionRequest>,
) -> impl IntoResponse {
    // Remove from store
    let container_id = {
        let mut sessions = SESSIONS.lock().unwrap();
        sessions.remove(&payload.session_id)
    };

    let container_id = match container_id {
        Some(id) => id,
        None => {
            return Json(StopSessionResponse {
                success: true,
                message: Some("Session not found (already stopped?)".to_string()),
            });
        }
    };

    // Stop container
    let client = match DockerClient::new() {
        Ok(c) => c,
        Err(e) => {
            return Json(StopSessionResponse {
                success: false,
                message: Some(format!("Docker error: {}", e)),
            });
        }
    };

    match client.stop_session(&container_id).await {
        Ok(()) => {
            println!("Session stopped: {} -> {}", payload.session_id, container_id);
            Json(StopSessionResponse {
                success: true,
                message: Some("Session stopped".to_string()),
            })
        }
        Err(e) => Json(StopSessionResponse {
            success: false,
            message: Some(format!("Failed to stop session: {}", e)),
        }),
    }
}
