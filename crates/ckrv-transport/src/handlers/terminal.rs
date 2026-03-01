//! # Terminal Handler
//!
//! Handlers for interactive terminal sessions with Docker containers.
//! Supports WebSocket-based bidirectional streaming between browser and Docker exec.

use crate::error::TransportError;
use crate::state::AppState;
use crate::types::{AgentConfig, AgentType};
use axum::extract::ws::{Message, WebSocket};
use bollard::container::LogOutput;
use bollard::exec::{CreateExecOptions, ResizeExecOptions, StartExecOptions, StartExecResults};
use bollard::Docker;
use futures_util::{SinkExt, StreamExt};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;
use tokio::io::AsyncWriteExt;
use tokio::sync::mpsc;

// ============================================================================
// Session Store
// ============================================================================

/// Session store for container IDs.
static TERMINAL_SESSIONS: Lazy<Mutex<HashMap<String, String>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

// ============================================================================
// Request/Response Types
// ============================================================================

/// Request to start a terminal session.
#[derive(Debug, Deserialize)]
pub struct StartTerminalRequest {
    /// Session ID for the terminal
    pub session_id: Option<String>,
    /// Agent configuration to use
    pub agent: Option<AgentConfig>,
    /// Terminal width in columns (from xterm.js)
    pub cols: Option<u16>,
    /// Terminal height in rows (from xterm.js)
    pub rows: Option<u16>,
}

/// Response from starting a terminal session.
#[derive(Debug, Serialize)]
pub struct StartTerminalResponse {
    /// Whether the operation succeeded
    pub success: bool,
    /// Session ID
    pub session_id: String,
    /// Container ID if created
    pub container_id: Option<String>,
    /// Message describing the result
    pub message: Option<String>,
}

/// Request to stop a terminal session.
#[derive(Debug, Deserialize)]
pub struct StopTerminalRequest {
    /// Session ID to stop
    pub session_id: String,
}

/// Response from stopping a terminal session.
#[derive(Debug, Serialize)]
pub struct StopTerminalResponse {
    /// Whether the operation succeeded
    pub success: bool,
    /// Message describing the result
    pub message: Option<String>,
}

// ============================================================================
// Handlers
// ============================================================================

/// Start a terminal session (creates Docker container).
pub async fn start_terminal_handler(
    state: &AppState,
    request: StartTerminalRequest,
) -> Result<StartTerminalResponse, TransportError> {
    let session_id = request
        .session_id
        .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

    // Check if session already exists
    {
        let sessions = TERMINAL_SESSIONS.lock().unwrap();
        if let Some(container_id) = sessions.get(&session_id) {
            return Ok(StartTerminalResponse {
                success: true,
                session_id,
                container_id: Some(container_id.clone()),
                message: Some("Session already exists".to_string()),
            });
        }
    }

    // Create Docker client
    let docker = match Docker::connect_with_local_defaults() {
        Ok(d) => d,
        Err(e) => {
            return Ok(StartTerminalResponse {
                success: false,
                session_id,
                container_id: None,
                message: Some(format!("Docker not available: {e}")),
            });
        }
    };

    // Get paths
    let cwd = state.project_root.to_string_lossy().to_string();
    let host_home = std::env::var("HOME").unwrap_or_default();

    // Start with workspace bind
    let mut binds = vec![format!("{cwd}:/workspace")];

    // Determine agent type
    let is_openrouter = request
        .agent
        .as_ref()
        .map(|a| matches!(a.agent_type, AgentType::ClaudeOpenRouter))
        .unwrap_or(false);

    let is_glm = request
        .agent
        .as_ref()
        .map(|a| matches!(a.agent_type, AgentType::ClaudeGlm))
        .unwrap_or(false);

    let is_codex = request
        .agent
        .as_ref()
        .map(|a| matches!(a.agent_type, AgentType::Codex))
        .unwrap_or(false);

    let is_kilo = request
        .agent
        .as_ref()
        .map(|a| matches!(a.agent_type, AgentType::KiloCode))
        .unwrap_or(false);

    let is_gemini = request
        .agent
        .as_ref()
        .map(|a| matches!(a.agent_type, AgentType::Gemini))
        .unwrap_or(false);

    let is_cursor = request
        .agent
        .as_ref()
        .map(|a| matches!(a.agent_type, AgentType::Cursor))
        .unwrap_or(false);

    let is_amp = request
        .agent
        .as_ref()
        .map(|a| matches!(a.agent_type, AgentType::Amp))
        .unwrap_or(false);

    let is_qwen = request
        .agent
        .as_ref()
        .map(|a| matches!(a.agent_type, AgentType::Qwen))
        .unwrap_or(false);

    let is_opencode = request
        .agent
        .as_ref()
        .map(|a| matches!(a.agent_type, AgentType::Opencode))
        .unwrap_or(false);

    let is_factory_droid = request
        .agent
        .as_ref()
        .map(|a| matches!(a.agent_type, AgentType::FactoryDroid))
        .unwrap_or(false);

    let is_github_copilot = request
        .agent
        .as_ref()
        .map(|a| matches!(a.agent_type, AgentType::GithubCopilot))
        .unwrap_or(false);

    let is_mistral_vibe = request
        .agent
        .as_ref()
        .map(|a| matches!(a.agent_type, AgentType::MistralVibe))
        .unwrap_or(false);

    // Set container home based on agent type
    let container_home = if is_codex {
        "/home/codex"
    } else if is_kilo {
        "/home/kilo"
    } else if is_gemini {
        "/home/gemini"
    } else if is_cursor {
        "/home/cursor"
    } else if is_amp {
        "/home/amp"
    } else if is_qwen {
        "/home/qwen"
    } else if is_opencode {
        "/home/opencode"
    } else if is_factory_droid {
        "/home/factory"
    } else if is_github_copilot {
        "/home/copilot"
    } else if is_mistral_vibe {
        "/home/vibe"
    } else {
        "/home/claude"
    };

    // Build environment variables based on agent type
    let term_cols = request.cols.unwrap_or(120);
    let term_rows = request.rows.unwrap_or(30);
    let mut env_vars = vec![
        format!("HOME={container_home}"),
        // TERM is critical for TUI-based CLIs (Codex uses Ink, Kilo uses similar).
        // Without it, these tools can't detect terminal capabilities and render blank.
        "TERM=xterm-256color".to_string(),
        "COLORTERM=truecolor".to_string(),
        // Terminal size from xterm.js frontend (or sensible defaults)
        format!("COLUMNS={term_cols}"),
        format!("LINES={term_rows}"),
    ];

    // Select Docker image based on agent type
    let docker_image = if is_codex {
        "ghcr.io/fnsk4r17s/ckrv-codex:latest".to_string()
    } else if is_kilo {
        "ghcr.io/fnsk4r17s/ckrv-kilo:latest".to_string()
    } else if is_gemini {
        "ghcr.io/fnsk4r17s/ckrv-gemini:latest".to_string()
    } else if is_cursor {
        "ghcr.io/fnsk4r17s/ckrv-cursor:latest".to_string()
    } else if is_amp {
        "ghcr.io/fnsk4r17s/ckrv-amp:latest".to_string()
    } else if is_qwen {
        "ghcr.io/fnsk4r17s/ckrv-qwen:latest".to_string()
    } else if is_opencode {
        "ghcr.io/fnsk4r17s/ckrv-opencode:latest".to_string()
    } else if is_factory_droid {
        "ghcr.io/fnsk4r17s/ckrv-factory:latest".to_string()
    } else if is_github_copilot {
        "ghcr.io/fnsk4r17s/ckrv-copilot:latest".to_string()
    } else if is_mistral_vibe {
        "ghcr.io/fnsk4r17s/ckrv-vibe:latest".to_string()
    } else {
        "ghcr.io/fnsk4r17s/ckrv-claude:latest".to_string()
    };

    if is_codex {
        // Codex configuration - mount logged-in credentials
        let codex_dir = format!("{host_home}/.codex");
        if std::path::Path::new(&codex_dir).exists() {
            binds.push(format!("{codex_dir}:/home/codex/.codex"));
        }
        let openai_config = format!("{host_home}/.config/openai");
        if std::path::Path::new(&openai_config).exists() {
            binds.push(format!("{openai_config}:/home/codex/.config/openai"));
        }

        // Also pass OPENAI_API_KEY if available
        if let Ok(key) = std::env::var("OPENAI_API_KEY") {
            env_vars.push(format!("OPENAI_API_KEY={key}"));
        }

        tracing::info!("Terminal session using OpenAI Codex with mounted credentials");
    } else if is_glm {
        // GLM Coding Plan configuration for Claude Code
        // See: https://docs.z.ai/devpack/tool/claude#manual-configuration
        if let Some(ref agent) = request.agent {
            if let Some(ref glm_config) = agent.glm {
                // Required: Set base URL to Z.AI
                env_vars.push("ANTHROPIC_BASE_URL=https://api.z.ai/api/anthropic".to_string());

                // Required: Set auth token to Z.AI API key
                if let Some(ref api_key) = glm_config.api_key {
                    env_vars.push(format!("ANTHROPIC_AUTH_TOKEN={api_key}"));
                    env_vars.push(format!("ZAI_API_KEY={api_key}"));
                }

                // Required: Explicitly blank out Anthropic API key to prevent conflicts
                env_vars.push("ANTHROPIC_API_KEY=".to_string());

                // Set extended timeout for GLM
                env_vars.push(format!(
                    "API_TIMEOUT_MS={}",
                    glm_config.timeout_ms.unwrap_or(3000000)
                ));

                // Set default model if specified
                if !glm_config.model.is_empty() {
                    env_vars.push(format!(
                        "ANTHROPIC_DEFAULT_SONNET_MODEL={}",
                        glm_config.model
                    ));
                    env_vars.push(format!("ANTHROPIC_DEFAULT_OPUS_MODEL={}", glm_config.model));
                    env_vars.push(format!(
                        "ANTHROPIC_DEFAULT_HAIKU_MODEL={}",
                        glm_config.model
                    ));
                }

                tracing::info!(
                    "GLM Coding Plan agent configured: model={}",
                    glm_config.model
                );
            }
        }

        tracing::info!(
            "Terminal session using GLM Coding Plan - skipping Claude credential mounts"
        );
    } else if is_openrouter {
        // OpenRouter configuration for Claude Code
        // See: https://openrouter.ai/docs/guides/guides/claude-code-integration
        if let Some(ref agent) = request.agent {
            if let Some(ref openrouter_config) = agent.openrouter {
                // Required: Set base URL to OpenRouter
                env_vars.push("ANTHROPIC_BASE_URL=https://openrouter.ai/api".to_string());

                // Required: Set auth token to OpenRouter API key
                if let Some(ref api_key) = openrouter_config.api_key {
                    env_vars.push(format!("ANTHROPIC_AUTH_TOKEN={api_key}"));
                    env_vars.push(format!("OPENROUTER_API_KEY={api_key}"));
                }

                // Required: Explicitly blank out Anthropic API key to prevent conflicts
                env_vars.push("ANTHROPIC_API_KEY=".to_string());

                // Optional: Set default model if specified (e.g., z-ai/glm-4.7)
                if !openrouter_config.model.is_empty() {
                    env_vars.push(format!(
                        "ANTHROPIC_DEFAULT_SONNET_MODEL={}",
                        openrouter_config.model
                    ));
                    env_vars.push(format!(
                        "ANTHROPIC_DEFAULT_OPUS_MODEL={}",
                        openrouter_config.model
                    ));
                    env_vars.push(format!(
                        "ANTHROPIC_DEFAULT_HAIKU_MODEL={}",
                        openrouter_config.model
                    ));
                }

                tracing::info!(
                    "OpenRouter agent configured: model={}",
                    openrouter_config.model
                );
            }
        }

        tracing::info!("Terminal session using OpenRouter - skipping Claude credential mounts");
    } else if is_kilo {
        // Kilo Code configuration - mount config directory for file-based auth
        let kilo_config = format!("{}/.config/kilo", host_home);
        if std::path::Path::new(&kilo_config).exists() {
            binds.push(format!("{kilo_config}:/home/kilo/.config/kilo"));
        }

        tracing::info!("Terminal session using Kilo Code with mounted config");
    } else if is_gemini {
        // Gemini configuration - mount Gemini credentials/config directories
        let gemini_dir = format!("{}/.gemini", host_home);
        if std::path::Path::new(&gemini_dir).exists() {
            binds.push(format!("{gemini_dir}:/home/gemini/.gemini"));
        }
        let google_config = format!("{}/.config/google", host_home);
        if std::path::Path::new(&google_config).exists() {
            binds.push(format!("{google_config}:/home/gemini/.config/google"));
        }
        if let Ok(key) = std::env::var("GEMINI_API_KEY") {
            env_vars.push(format!("GEMINI_API_KEY={key}"));
        }

        tracing::info!("Terminal session using Gemini with mounted config");
    } else if is_cursor {
        // Cursor configuration - mount Cursor credentials/config
        let cursor_dir = format!("{}/.cursor", host_home);
        if std::path::Path::new(&cursor_dir).exists() {
            binds.push(format!("{cursor_dir}:/home/cursor/.cursor"));
        }
        let cursor_config = format!("{}/.config/cursor", host_home);
        if std::path::Path::new(&cursor_config).exists() {
            binds.push(format!("{cursor_config}:/home/cursor/.config/cursor"));
        }

        tracing::info!("Terminal session using Cursor with mounted config");
    } else if is_amp {
        // Amp configuration - mount Amp config directory
        let amp_config = format!("{}/.config/amp", host_home);
        if std::path::Path::new(&amp_config).exists() {
            binds.push(format!("{amp_config}:/home/amp/.config/amp"));
        }

        tracing::info!("Terminal session using Amp with mounted config");
    } else if is_qwen {
        // Qwen Code configuration - pass through API keys
        if let Ok(key) = std::env::var("OPENAI_API_KEY") {
            env_vars.push(format!("OPENAI_API_KEY={key}"));
        }
        if let Ok(key) = std::env::var("QWEN_AUTH_TOKEN") {
            env_vars.push(format!("QWEN_AUTH_TOKEN={key}"));
        }
        if let Ok(url) = std::env::var("OPENAI_BASE_URL") {
            env_vars.push(format!("OPENAI_BASE_URL={url}"));
        }

        tracing::info!("Terminal session using Qwen Code");
    } else if is_opencode {
        // Opencode configuration - mount config directory
        let opencode_config = format!("{}/.config/opencode", host_home);
        if std::path::Path::new(&opencode_config).exists() {
            binds.push(format!("{opencode_config}:/home/opencode/.config/opencode"));
        }

        tracing::info!("Terminal session using Opencode with mounted config");
    } else if is_factory_droid {
        // Factory Droid configuration - mount credentials and pass API key
        let factory_dir = format!("{}/.factory", host_home);
        if std::path::Path::new(&factory_dir).exists() {
            binds.push(format!("{factory_dir}:/home/factory/.factory"));
        }
        if let Ok(key) = std::env::var("FACTORY_API_KEY") {
            env_vars.push(format!("FACTORY_API_KEY={key}"));
        }

        tracing::info!("Terminal session using Factory Droid with mounted config");
    } else if is_github_copilot {
        // GitHub Copilot configuration - mount gh CLI config
        let gh_config = format!("{}/.config/gh", host_home);
        if std::path::Path::new(&gh_config).exists() {
            binds.push(format!("{gh_config}:/home/copilot/.config/gh"));
        }

        tracing::info!("Terminal session using GitHub Copilot with mounted config");
    } else if is_mistral_vibe {
        // Mistral Vibe configuration - pass API key
        if let Ok(key) = std::env::var("MISTRAL_API_KEY") {
            env_vars.push(format!("MISTRAL_API_KEY={key}"));
        }
        let vibe_config = format!("{}/.config/vibe", host_home);
        if std::path::Path::new(&vibe_config).exists() {
            binds.push(format!("{vibe_config}:/home/vibe/.config/vibe"));
        }

        tracing::info!("Terminal session using Mistral Vibe");
    } else {
        // For native Claude, mount credentials if they exist
        let claude_config = format!("{host_home}/.claude.json");
        if std::path::Path::new(&claude_config).exists() {
            binds.push(format!("{claude_config}:{container_home}/.claude.json"));
        }
        let claude_dir = format!("{host_home}/.claude");
        if std::path::Path::new(&claude_dir).exists() {
            binds.push(format!("{claude_dir}:{container_home}/.claude"));
        }

        // Pass ANTHROPIC_API_KEY if available
        if let Ok(key) = std::env::var("ANTHROPIC_API_KEY") {
            env_vars.push(format!("ANTHROPIC_API_KEY={key}"));
        }

        tracing::info!("Terminal session using native Claude with mounted credentials");
    }

    // Create container name
    let container_name = format!("ckrv-term-{}", uuid::Uuid::new_v4());

    // Build container config
    let config = bollard::container::Config {
        image: Some(docker_image),
        cmd: Some(vec![
            "tail".to_string(),
            "-f".to_string(),
            "/dev/null".to_string(),
        ]),
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
                return Ok(StartTerminalResponse {
                    success: false,
                    session_id,
                    container_id: None,
                    message: Some(format!("Failed to start container: {e}")),
                });
            }

            // Store session
            {
                let mut sessions = TERMINAL_SESSIONS.lock().unwrap();
                sessions.insert(session_id.clone(), container.id.clone());
            }

            tracing::info!(
                "Terminal session started: {} -> {}",
                session_id,
                container.id
            );

            Ok(StartTerminalResponse {
                success: true,
                session_id,
                container_id: Some(container.id),
                message: Some("Terminal session created".to_string()),
            })
        }
        Err(e) => Ok(StartTerminalResponse {
            success: false,
            session_id,
            container_id: None,
            message: Some(format!("Failed to create container: {e}")),
        }),
    }
}

/// Handle WebSocket connection for interactive terminal.
pub async fn handle_terminal_ws(socket: WebSocket, session_id: String) {
    // Look up container
    let container_id = {
        let sessions = TERMINAL_SESSIONS.lock().unwrap();
        sessions.get(&session_id).cloned()
    };

    let container_id = match container_id {
        Some(id) => id,
        None => {
            let (mut sender, _) = socket.split();
            let _ = sender
                .send(Message::Text(
                    "Error: No session found. Start a session first.".into(),
                ))
                .await;
            return;
        }
    };

    // Connect to Docker
    let docker = match Docker::connect_with_local_defaults() {
        Ok(d) => d,
        Err(e) => {
            let (mut sender, _) = socket.split();
            let _ = sender
                .send(Message::Text(
                    format!("Error: Docker connection failed: {e}").into(),
                ))
                .await;
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
        // Set TERM so TUI-based CLIs (Codex/Ink, Kilo) can detect terminal capabilities.
        // Without this, they render blank or produce garbled output in xterm.js.
        env: Some(vec![
            "TERM=xterm-256color".to_string(),
            "COLORTERM=truecolor".to_string(),
        ]),
        ..Default::default()
    };

    let exec = match docker.create_exec(&container_id, exec_config).await {
        Ok(e) => e,
        Err(e) => {
            let (mut sender, _) = socket.split();
            let _ = sender
                .send(Message::Text(
                    format!("Error: Failed to create exec: {e}").into(),
                ))
                .await;
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
            let _ = sender
                .send(Message::Text(
                    format!("Error: Failed to start exec: {e}").into(),
                ))
                .await;
            return;
        }
    };

    // Get the attached streams
    if let StartExecResults::Attached {
        mut output,
        mut input,
    } = exec_result
    {
        let (mut ws_sender, mut ws_receiver) = socket.split();

        // Channel for coordinating shutdown
        let (shutdown_tx, mut shutdown_rx) = mpsc::channel::<()>(1);
        let shutdown_tx2 = shutdown_tx.clone();

        // Hold the exec ID for resize operations
        let exec_id = exec.id.clone();

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

        // Clone docker + exec_id for the input task
        let docker_for_input = docker.clone();
        let exec_id_for_input = exec_id.clone();

        // Task: WebSocket input -> Docker stdin (handles both text input and resize messages)
        let input_task = tokio::spawn(async move {
            while let Some(Ok(msg)) = ws_receiver.next().await {
                match msg {
                    Message::Text(ref text) => {
                        // Check if this is a resize JSON message from the frontend
                        if let Ok(resize) = serde_json::from_str::<serde_json::Value>(text.as_ref())
                        {
                            if resize.get("type").and_then(|t| t.as_str()) == Some("resize") {
                                let cols =
                                    resize.get("cols").and_then(|c| c.as_u64()).unwrap_or(120)
                                        as u16;
                                let rows = resize.get("rows").and_then(|r| r.as_u64()).unwrap_or(30)
                                    as u16;
                                let _ = docker_for_input
                                    .resize_exec(
                                        &exec_id_for_input,
                                        ResizeExecOptions {
                                            width: cols,
                                            height: rows,
                                        },
                                    )
                                    .await;
                                continue;
                            }
                        }
                        // Regular text input
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

/// Stop a terminal session.
pub async fn stop_terminal_handler(
    request: StopTerminalRequest,
) -> Result<StopTerminalResponse, TransportError> {
    // Remove from store
    let container_id = {
        let mut sessions = TERMINAL_SESSIONS.lock().unwrap();
        sessions.remove(&request.session_id)
    };

    let container_id = match container_id {
        Some(id) => id,
        None => {
            return Ok(StopTerminalResponse {
                success: true,
                message: Some("Session not found (already stopped?)".to_string()),
            });
        }
    };

    // Stop and remove container
    let docker = match Docker::connect_with_local_defaults() {
        Ok(d) => d,
        Err(e) => {
            return Ok(StopTerminalResponse {
                success: false,
                message: Some(format!("Docker error: {e}")),
            });
        }
    };

    let remove_options = Some(bollard::container::RemoveContainerOptions {
        force: true,
        ..Default::default()
    });

    match docker.remove_container(&container_id, remove_options).await {
        Ok(()) => {
            tracing::info!(
                "Terminal session stopped: {} -> {}",
                request.session_id,
                container_id
            );
            Ok(StopTerminalResponse {
                success: true,
                message: Some("Session stopped".to_string()),
            })
        }
        Err(e) => Ok(StopTerminalResponse {
            success: false,
            message: Some(format!("Failed to stop session: {e}")),
        }),
    }
}
