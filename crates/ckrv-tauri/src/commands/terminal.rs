//! Terminal commands for Tauri IPC
//!
//! This module provides terminal/shell capabilities using Docker containers for
//! sandboxed agent execution. Uses ckrv-sandbox for container management.

use crate::SharedState;
use ckrv_sandbox::DockerClient;
use ckrv_transport::types::{AgentConfig, AgentType};
use parking_lot::Mutex;
use serde::Serialize;
use std::collections::HashMap;
use std::sync::Arc;
use tauri::State;

/// Session state for managing active terminal sessions.
pub type TerminalSessions = Arc<Mutex<HashMap<String, TerminalSession>>>;

/// A terminal session with its Docker container.
pub struct TerminalSession {
    /// Container ID running the session
    pub container_id: String,
    /// Session ID
    pub session_id: String,
}

/// Response for terminal start.
#[derive(Debug, Serialize)]
pub struct TerminalStartResponse {
    pub session_id: String,
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub container_id: Option<String>,
    /// Mode indicator for frontend (tauri = use IPC, web = use WebSocket)  
    pub mode: String,
}

/// Response for terminal stop.
#[derive(Debug, Serialize)]
pub struct TerminalStopResponse {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

/// Terminal output event.
#[derive(Debug, Clone, Serialize)]
pub struct TerminalOutput {
    pub session_id: String,
    pub data: String,
    pub is_error: bool,
}

/// Convert transport AgentConfig to sandbox AgentConfig for container setup.
fn agent_to_sandbox_config(agent: &AgentConfig) -> HashMap<String, String> {
    let mut env = HashMap::new();

    match agent.agent_type {
        AgentType::ClaudeOpenRouter => {
            if let Some(ref or_config) = agent.openrouter {
                env.insert(
                    "ANTHROPIC_BASE_URL".to_string(),
                    "https://openrouter.ai/api".to_string(),
                );
                if let Some(ref api_key) = or_config.api_key {
                    env.insert("ANTHROPIC_AUTH_TOKEN".to_string(), api_key.clone());
                    env.insert("OPENROUTER_API_KEY".to_string(), api_key.clone());
                }
                env.insert("ANTHROPIC_API_KEY".to_string(), "".to_string());
                if !or_config.model.is_empty() {
                    env.insert(
                        "ANTHROPIC_DEFAULT_SONNET_MODEL".to_string(),
                        or_config.model.clone(),
                    );
                }
            }
        }
        AgentType::ClaudeGlm => {
            if let Some(ref glm_config) = agent.glm {
                env.insert(
                    "ANTHROPIC_BASE_URL".to_string(),
                    "https://api.z.ai/api/anthropic".to_string(),
                );
                if let Some(ref api_key) = glm_config.api_key {
                    env.insert("ANTHROPIC_AUTH_TOKEN".to_string(), api_key.clone());
                    env.insert("ZAI_API_KEY".to_string(), api_key.clone());
                }
                env.insert("ANTHROPIC_API_KEY".to_string(), "".to_string());
                if !glm_config.model.is_empty() {
                    env.insert(
                        "ANTHROPIC_DEFAULT_SONNET_MODEL".to_string(),
                        glm_config.model.clone(),
                    );
                }
            }
        }
        AgentType::Codex => {
            if let Ok(key) = std::env::var("OPENAI_API_KEY") {
                env.insert("OPENAI_API_KEY".to_string(), key);
            }
        }
        AgentType::Claude => {
            if let Ok(key) = std::env::var("ANTHROPIC_API_KEY") {
                env.insert("ANTHROPIC_API_KEY".to_string(), key);
            }
        }
        AgentType::KiloCode => {
            // Kilo Code uses file-based auth (~/.config/kilo/) - no env vars needed
        }
        AgentType::Gemini => {
            if let Ok(key) = std::env::var("GEMINI_API_KEY") {
                env.insert("GEMINI_API_KEY".to_string(), key);
            }
        }
    }

    // Set container home
    let home = if matches!(agent.agent_type, AgentType::Codex) {
        "/home/codex"
    } else if matches!(agent.agent_type, AgentType::KiloCode) {
        "/home/kilo"
    } else if matches!(agent.agent_type, AgentType::Gemini) {
        "/home/gemini"
    } else {
        "/home/claude"
    };
    env.insert("HOME".to_string(), home.to_string());

    // TERM is critical for TUI-based CLIs (Codex uses Ink, Kilo uses similar).
    // Without it, these tools can't detect terminal capabilities and render blank in xterm.js.
    env.insert("TERM".to_string(), "xterm-256color".to_string());
    env.insert("COLORTERM".to_string(), "truecolor".to_string());

    env
}

/// Start a terminal session with Docker container.
#[tauri::command(rename_all = "snake_case")]
pub async fn terminal_start(
    state: State<'_, SharedState>,
    sessions: State<'_, TerminalSessions>,
    session_id: String,
    agent: Option<AgentConfig>,
    #[allow(unused)] command: Option<String>,
    #[allow(unused)] args: Option<Vec<String>>,
    #[allow(unused)] cwd: Option<String>,
) -> Result<TerminalStartResponse, String> {
    // Check if session already exists
    {
        let sessions_guard = sessions.lock();
        if let Some(existing) = sessions_guard.get(&session_id) {
            return Ok(TerminalStartResponse {
                session_id,
                success: true,
                message: Some("Session already exists".to_string()),
                container_id: Some(existing.container_id.clone()),
                mode: "tauri".to_string(),
            });
        }
    }

    // Create Docker client
    let mut docker = match DockerClient::new() {
        Ok(d) => d,
        Err(e) => {
            return Ok(TerminalStartResponse {
                session_id,
                success: false,
                message: Some(format!("Docker not available: {e}")),
                container_id: None,
                mode: "tauri".to_string(),
            });
        }
    };

    // Get project root from app state
    let project_root = {
        let app_state = state.read().await;
        app_state.project_root.to_string_lossy().to_string()
    };

    // Set image based on agent type
    let is_codex = agent
        .as_ref()
        .map(|a| matches!(a.agent_type, AgentType::Codex))
        .unwrap_or(false);

    let is_kilo = agent
        .as_ref()
        .map(|a| matches!(a.agent_type, AgentType::KiloCode))
        .unwrap_or(false);

    let is_gemini = agent
        .as_ref()
        .map(|a| matches!(a.agent_type, AgentType::Gemini))
        .unwrap_or(false);

    let image = if is_codex {
        "ghcr.io/fnsk4r17s/ckrv-codex:latest"
    } else if is_kilo {
        "ghcr.io/fnsk4r17s/ckrv-kilo:latest"
    } else if is_gemini {
        "ghcr.io/fnsk4r17s/ckrv-gemini:latest"
    } else {
        "ghcr.io/fnsk4r17s/ckrv-claude:latest"
    };
    docker.set_image(image);

    // Build environment from agent config
    let env = agent
        .as_ref()
        .map(|a| agent_to_sandbox_config(a))
        .unwrap_or_default();

    // Create session container
    match docker
        .create_session("/workspace", &project_root, "/workspace", env, Vec::new())
        .await
    {
        Ok(container_id) => {
            tracing::info!(
                "Terminal session created: {} -> {}",
                session_id,
                container_id
            );

            // Store session
            {
                let mut sessions_guard = sessions.lock();
                sessions_guard.insert(
                    session_id.clone(),
                    TerminalSession {
                        container_id: container_id.clone(),
                        session_id: session_id.clone(),
                    },
                );
            }

            Ok(TerminalStartResponse {
                session_id,
                success: true,
                message: Some("Terminal session created".to_string()),
                container_id: Some(container_id),
                mode: "tauri".to_string(),
            })
        }
        Err(e) => Ok(TerminalStartResponse {
            session_id,
            success: false,
            message: Some(format!("Failed to create container: {e}")),
            container_id: None,
            mode: "tauri".to_string(),
        }),
    }
}

/// Stop a terminal session.
#[tauri::command(rename_all = "snake_case")]
pub async fn terminal_stop(
    sessions: State<'_, TerminalSessions>,
    session_id: String,
) -> Result<TerminalStopResponse, String> {
    // Remove from sessions
    let container_id = {
        let mut sessions_guard = sessions.lock();
        sessions_guard.remove(&session_id).map(|s| s.container_id)
    };

    let container_id = match container_id {
        Some(id) => id,
        None => {
            return Ok(TerminalStopResponse {
                success: true,
                message: Some("Session not found (already stopped?)".to_string()),
            });
        }
    };

    // Stop container using DockerClient
    let docker = match DockerClient::new() {
        Ok(d) => d,
        Err(e) => {
            return Ok(TerminalStopResponse {
                success: false,
                message: Some(format!("Docker error: {e}")),
            });
        }
    };

    match docker.stop_session(&container_id).await {
        Ok(()) => {
            tracing::info!(
                "Terminal session stopped: {} -> {}",
                session_id,
                container_id
            );
            Ok(TerminalStopResponse {
                success: true,
                message: Some("Session stopped".to_string()),
            })
        }
        Err(e) => Ok(TerminalStopResponse {
            success: false,
            message: Some(format!("Failed to stop session: {e}")),
        }),
    }
}

/// Send input to a terminal session (execute command in container).
#[tauri::command(rename_all = "snake_case")]
pub async fn terminal_write(
    sessions: State<'_, TerminalSessions>,
    session_id: String,
    data: String,
) -> Result<bool, String> {
    // Get container ID
    let container_id = {
        let sessions_guard = sessions.lock();
        sessions_guard
            .get(&session_id)
            .map(|s| s.container_id.clone())
    };

    let container_id = match container_id {
        Some(id) => id,
        None => return Err(format!("Session {} not found", session_id)),
    };

    // Execute command in container
    let docker = DockerClient::new().map_err(|e| format!("Docker error: {}", e))?;

    // For interactive terminal, we execute the input as a shell command
    let command = vec!["/bin/bash".to_string(), "-c".to_string(), data];

    match docker
        .exec_in_session(&container_id, command, HashMap::new())
        .await
    {
        Ok(_) => Ok(true),
        Err(e) => Err(format!("Failed to execute: {}", e)),
    }
}

/// Read output from a terminal session.
/// Note: For true interactive terminal, we'd need PTY support. This returns last command output.
#[tauri::command(rename_all = "snake_case")]
pub async fn terminal_read(
    sessions: State<'_, TerminalSessions>,
    session_id: String,
) -> Result<Option<TerminalOutput>, String> {
    // Get container ID
    let container_id = {
        let sessions_guard = sessions.lock();
        sessions_guard
            .get(&session_id)
            .map(|s| s.container_id.clone())
    };

    let _container_id = match container_id {
        Some(id) => id,
        None => return Ok(None),
    };

    // For now, return empty output - full PTY support would require more complex streaming
    // The web version uses WebSocket + Docker attach for real-time output
    Ok(None)
}

/// Check if a terminal session is still running.
#[tauri::command(rename_all = "snake_case")]
pub async fn terminal_is_running(
    sessions: State<'_, TerminalSessions>,
    session_id: String,
) -> Result<bool, String> {
    let sessions_guard = sessions.lock();
    Ok(sessions_guard.contains_key(&session_id))
}

/// List active terminal sessions.
#[tauri::command]
pub async fn terminal_list(sessions: State<'_, TerminalSessions>) -> Result<Vec<String>, String> {
    let sessions_guard = sessions.lock();
    Ok(sessions_guard.keys().cloned().collect())
}
