//! WebSocket-based interactive terminal for Docker containers.
//!
//! Provides bidirectional streaming between the browser and Docker exec.

use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State, Query,
    },
    response::IntoResponse,
};
use bollard::Docker;
use bollard::exec::{CreateExecOptions, StartExecOptions, StartExecResults};
use bollard::container::LogOutput;
use futures_util::{SinkExt, StreamExt};
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::Mutex;
use once_cell::sync::Lazy;
use tokio::sync::mpsc;
use tokio::io::AsyncWriteExt;

use crate::state::AppState;
use crate::api::agents::{AgentConfig, AgentType};

// Session store for container IDs
static TERMINAL_SESSIONS: Lazy<Mutex<HashMap<String, String>>> = Lazy::new(|| {
    Mutex::new(HashMap::new())
});

/// Query parameters for WebSocket connection
#[derive(Debug, Deserialize)]
pub struct TerminalQuery {
    pub session_id: String,
}

/// Request to start a terminal session
#[derive(Debug, Deserialize)]
pub struct StartTerminalRequest {
    pub session_id: String,
    pub agent: Option<AgentConfig>,
}

/// Handler for WebSocket upgrade
pub async fn terminal_ws(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    Query(query): Query<TerminalQuery>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_terminal(socket, state, query.session_id))
}

/// Start a terminal session (creates container if needed)
pub async fn start_terminal_session(
    State(state): State<AppState>,
    axum::Json(payload): axum::Json<StartTerminalRequest>,
) -> impl IntoResponse {
    // Check if session already exists
    {
        let sessions = TERMINAL_SESSIONS.lock().unwrap();
        if let Some(container_id) = sessions.get(&payload.session_id) {
            return axum::Json(super::session::StartSessionResponse {
                success: true,
                session_id: payload.session_id,
                container_id: Some(container_id.clone()),
                message: Some("Session already exists".to_string()),
            });
        }
    }

    // Create Docker client
    let docker = match Docker::connect_with_local_defaults() {
        Ok(d) => d,
        Err(e) => {
            return axum::Json(super::session::StartSessionResponse {
                success: false,
                session_id: payload.session_id,
                container_id: None,
                message: Some(format!("Docker not available: {}", e)),
            });
        }
    };

    // Create a container that stays alive with a shell
    let cwd = state.project_root.to_string_lossy().to_string();
    
    // Get host home for Claude config
    let host_home = std::env::var("HOME").unwrap_or_default();
    let container_home = "/home/claude";
    
    let mut binds = vec![
        format!("{}:/workspace", cwd),
    ];
    
    // Build environment variables based on agent type
    let mut env_vars = vec![format!("HOME={}", container_home)];
    
    let is_openrouter = payload.agent.as_ref()
        .map(|a| matches!(a.agent_type, AgentType::ClaudeOpenRouter))
        .unwrap_or(false);
    
    if is_openrouter {
        // OpenRouter configuration for Claude Code
        // See: https://openrouter.ai/docs/guides/guides/claude-code-integration
        if let Some(ref agent) = payload.agent {
            if let Some(ref openrouter_config) = agent.openrouter {
                // Required: Set base URL to OpenRouter
                env_vars.push("ANTHROPIC_BASE_URL=https://openrouter.ai/api".to_string());
                
                // Required: Set auth token to OpenRouter API key
                if let Some(ref api_key) = openrouter_config.api_key {
                    env_vars.push(format!("ANTHROPIC_AUTH_TOKEN={}", api_key));
                    env_vars.push(format!("OPENROUTER_API_KEY={}", api_key));
                }
                
                // Required: Explicitly blank out Anthropic API key to prevent conflicts
                env_vars.push("ANTHROPIC_API_KEY=".to_string());
                
                // Optional: Set default model if specified (e.g., z-ai/glm-4.7)
                if !openrouter_config.model.is_empty() {
                    env_vars.push(format!("ANTHROPIC_DEFAULT_SONNET_MODEL={}", openrouter_config.model));
                    // Also set opus and haiku to same model for consistency
                    env_vars.push(format!("ANTHROPIC_DEFAULT_OPUS_MODEL={}", openrouter_config.model));
                    env_vars.push(format!("ANTHROPIC_DEFAULT_HAIKU_MODEL={}", openrouter_config.model));
                }
                
                println!("OpenRouter agent configured: model={}", openrouter_config.model);
            }
        }
        
        // For OpenRouter, we do NOT mount Claude credentials
        println!("Terminal session using OpenRouter - skipping Claude credential mounts");
    } else {
        // For native Claude, mount credentials if they exist
        let claude_config = format!("{}/.claude.json", host_home);
        if std::path::Path::new(&claude_config).exists() {
            binds.push(format!("{}:{}/.claude.json", claude_config, container_home));
        }
        let claude_dir = format!("{}/.claude", host_home);
        if std::path::Path::new(&claude_dir).exists() {
            binds.push(format!("{}:{}/.claude", claude_dir, container_home));
        }
        println!("Terminal session using native Claude with mounted credentials");
    }
    
    // Add any custom env vars from agent config
    if let Some(ref agent) = payload.agent {
        if let Some(ref custom_env) = agent.env_vars {
            for (key, value) in custom_env {
                env_vars.push(format!("{}={}", key, value));
            }
        }
    }

    let container_name = format!("ckrv-term-{}", uuid::Uuid::new_v4());
    
    let config = bollard::container::Config {
        image: Some("ckrv-agent:latest".to_string()),
        cmd: Some(vec!["tail".to_string(), "-f".to_string(), "/dev/null".to_string()]),
        working_dir: Some("/workspace".to_string()),
        env: Some(env_vars),
        host_config: Some(bollard::models::HostConfig {
            binds: Some(binds),
            network_mode: Some("host".to_string()),
            ..Default::default()
        }),
        tty: Some(true),
        open_stdin: Some(true),
        ..Default::default()
    };

    let options = Some(bollard::container::CreateContainerOptions {
        name: container_name.clone(),
        platform: None,
    });

    match docker.create_container(options, config).await {
        Ok(container) => {
            // Start container
            if let Err(e) = docker.start_container::<String>(&container.id, None).await {
                return axum::Json(super::session::StartSessionResponse {
                    success: false,
                    session_id: payload.session_id,
                    container_id: None,
                    message: Some(format!("Failed to start container: {}", e)),
                });
            }

            // Store session
            {
                let mut sessions = TERMINAL_SESSIONS.lock().unwrap();
                sessions.insert(payload.session_id.clone(), container.id.clone());
            }

            println!("Terminal session started: {} -> {}", payload.session_id, container.id);

            axum::Json(super::session::StartSessionResponse {
                success: true,
                session_id: payload.session_id,
                container_id: Some(container.id),
                message: Some("Terminal session created".to_string()),
            })
        }
        Err(e) => axum::Json(super::session::StartSessionResponse {
            success: false,
            session_id: payload.session_id,
            container_id: None,
            message: Some(format!("Failed to create container: {}", e)),
        }),
    }
}

/// Handle the WebSocket connection for interactive terminal
async fn handle_terminal(socket: WebSocket, _state: AppState, session_id: String) {
    // Look up container
    let container_id = {
        let sessions = TERMINAL_SESSIONS.lock().unwrap();
        sessions.get(&session_id).cloned()
    };

    let container_id = match container_id {
        Some(id) => id,
        None => {
            let (mut sender, _) = socket.split();
            let _ = sender.send(Message::Text("Error: No session found. Start a session first.".to_string().into())).await;
            return;
        }
    };

    // Connect to Docker
    let docker = match Docker::connect_with_local_defaults() {
        Ok(d) => d,
        Err(e) => {
            let (mut sender, _) = socket.split();
            let _ = sender.send(Message::Text(format!("Error: Docker connection failed: {}", e).into())).await;
            return;
        }
    };

    // Create exec instance for interactive shell
    let exec_config = CreateExecOptions {
        attach_stdin: Some(true),
        attach_stdout: Some(true),
        attach_stderr: Some(true),
        tty: Some(true),
        cmd: Some(vec!["/bin/bash".to_string(), "-l".to_string()]),
        ..Default::default()
    };

    let exec = match docker.create_exec(&container_id, exec_config).await {
        Ok(e) => e,
        Err(e) => {
            let (mut sender, _) = socket.split();
            let _ = sender.send(Message::Text(format!("Error: Failed to create exec: {}", e).into())).await;
            return;
        }
    };

    // Start exec with TTY
    let start_config = Some(StartExecOptions {
        detach: false,
        tty: true,
        ..Default::default()
    });

    let exec_result = match docker.start_exec(&exec.id, start_config).await {
        Ok(r) => r,
        Err(e) => {
            let (mut sender, _) = socket.split();
            let _ = sender.send(Message::Text(format!("Error: Failed to start exec: {}", e).into())).await;
            return;
        }
    };

    // Get the attached streams
    if let StartExecResults::Attached { mut output, mut input } = exec_result {
        let (mut ws_sender, mut ws_receiver) = socket.split();
        
        // Channel for coordinating shutdown
        let (shutdown_tx, mut shutdown_rx) = mpsc::channel::<()>(1);
        let shutdown_tx2 = shutdown_tx.clone();

        // Task: Docker output -> WebSocket
        let output_task = tokio::spawn(async move {
            loop {
                tokio::select! {
                    msg = output.next() => {
                        match msg {
                            Some(Ok(log)) => {
                                let text = match log {
                                    LogOutput::StdOut { message } => String::from_utf8_lossy(&message).to_string(),
                                    LogOutput::StdErr { message } => String::from_utf8_lossy(&message).to_string(),
                                    LogOutput::Console { message } => String::from_utf8_lossy(&message).to_string(),
                                    _ => continue,
                                };
                                if ws_sender.send(Message::Text(text.into())).await.is_err() {
                                    break;
                                }
                            }
                            Some(Err(_)) | None => break,
                        }
                    }
                    _ = shutdown_rx.recv() => break,
                }
            }
        });

        // Task: WebSocket input -> Docker stdin
        let input_task = tokio::spawn(async move {
            while let Some(Ok(msg)) = ws_receiver.next().await {
                match msg {
                    Message::Text(text) => {
                        if input.write_all(text.as_bytes()).await.is_err() {
                            break;
                        }
                    }
                    Message::Binary(data) => {
                        if input.write_all(&data).await.is_err() {
                            break;
                        }
                    }
                    Message::Close(_) => {
                        let _ = shutdown_tx2.send(()).await;
                        break;
                    }
                    _ => {}
                }
            }
        });

        // Wait for either task to complete
        let _ = tokio::select! {
            _ = output_task => {},
            _ = input_task => {},
        };
    }
}

/// Stop a terminal session
pub async fn stop_terminal_session(
    axum::Json(payload): axum::Json<super::session::StopSessionRequest>,
) -> impl IntoResponse {
    // Remove from store
    let container_id = {
        let mut sessions = TERMINAL_SESSIONS.lock().unwrap();
        sessions.remove(&payload.session_id)
    };

    let container_id = match container_id {
        Some(id) => id,
        None => {
            return axum::Json(super::session::StopSessionResponse {
                success: true,
                message: Some("Session not found (already stopped?)".to_string()),
            });
        }
    };

    // Stop and remove container
    let docker = match Docker::connect_with_local_defaults() {
        Ok(d) => d,
        Err(e) => {
            return axum::Json(super::session::StopSessionResponse {
                success: false,
                message: Some(format!("Docker error: {}", e)),
            });
        }
    };

    let remove_options = Some(bollard::container::RemoveContainerOptions {
        force: true,
        ..Default::default()
    });

    match docker.remove_container(&container_id, remove_options).await {
        Ok(()) => {
            println!("Terminal session stopped: {} -> {}", payload.session_id, container_id);
            axum::Json(super::session::StopSessionResponse {
                success: true,
                message: Some("Session stopped".to_string()),
            })
        }
        Err(e) => axum::Json(super::session::StopSessionResponse {
            success: false,
            message: Some(format!("Failed to stop session: {}", e)),
        }),
    }
}
