//! # Term Command
//!
//! Spawns an interactive AI agent terminal session with optional isolation modes.
//!
//! ## Overview
//!
//! This command provides a quick way to launch any configured AI agent
//! in an interactive terminal session. It supports three isolation modes:
//!
//! - **Default**: Agent runs directly in the current working directory
//! - **Worktree (`--worktree`)**: Agent runs in an isolated git worktree on a separate branch
//! - **Sandbox (`--sandbox`)**: Agent runs inside a Docker container
//! - **Combined (`--sandbox --worktree`)**: Maximum isolation with both
//!
//! ## Usage
//!
//! ```bash
//! ckrv term                          # Interactive agent selection
//! ckrv term --agent claude-default   # Direct agent spawn
//! ckrv term --worktree               # Isolated worktree mode
//! ckrv term --sandbox                # Docker sandbox mode
//! ckrv term --sandbox --worktree     # Maximum isolation
//! ckrv term --worktree --name fix-auth  # Named session for resume
//! ckrv term --resume fix-auth        # Resume a named session
//! ckrv term --list-sessions          # List all sessions
//! ckrv term --cleanup fix-auth       # Remove a session
//! ```

// ============================================================
// IMPORTS
// ============================================================

// Standard library
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

// External crates
use chrono::{DateTime, Utc};
use clap::Args;
use dialoguer::{theme::ColorfulTheme, Input, MultiSelect, Select};
use serde::{Deserialize, Serialize};

// Workspace crates
use ckrv_git::{DefaultWorktreeManager, Worktree, WorktreeManager};
use ckrv_sandbox::{BindMount, DockerClient};

// Internal modules
use crate::services::agent_lookup::{load_agents_config, AgentConfig, AgentType};
use crate::ui::UiContext;

// ============================================================
// CONSTANTS
// ============================================================

// ============================================================
// TYPES
// ============================================================

/// Common agent options that can be selected interactively
#[derive(Debug, Clone)]
struct CommonOption {
    label: &'static str,
    action: OptionAction,
    description: &'static str,
    agents: &'static [AgentType],
}

/// Action to take when an option is selected
#[derive(Debug, Clone)]
enum OptionAction {
    Flag(&'static str),
    EnvVar(&'static str, &'static str),
}

/// All Claude-based agent types (native, OpenRouter, `GLM`)
const CLAUDE_AGENTS: &[AgentType] = &[
    AgentType::Claude,
    AgentType::ClaudeOpenRouter,
    AgentType::ClaudeGlm,
];

const COMMON_OPTIONS: &[CommonOption] = &[
    CommonOption {
        label: "Skip permissions",
        action: OptionAction::Flag("--dangerously-skip-permissions"),
        description: "Skip all permission prompts (dangerous!)",
        agents: CLAUDE_AGENTS,
    },
    CommonOption {
        label: "Continue session",
        action: OptionAction::Flag("--continue"),
        description: "Resume the most recent conversation",
        agents: CLAUDE_AGENTS,
    },
    CommonOption {
        label: "Agent teams",
        action: OptionAction::EnvVar("CLAUDE_CODE_EXPERIMENTAL_AGENT_TEAMS", "1"),
        description: "Enable experimental agent teams mode",
        agents: CLAUDE_AGENTS,
    },
    CommonOption {
        label: "Verbose output",
        action: OptionAction::Flag("--verbose"),
        description: "Enable verbose logging",
        agents: CLAUDE_AGENTS,
    },
    CommonOption {
        label: "JSON output",
        action: OptionAction::Flag("--output-format json"),
        description: "Output in JSON format",
        agents: CLAUDE_AGENTS,
    },
    CommonOption {
        label: "Full auto mode",
        action: OptionAction::Flag("--full-auto"),
        description: "Run autonomously without approval prompts",
        agents: &[AgentType::Codex],
    },
    CommonOption {
        label: "JSON output",
        action: OptionAction::Flag("--json"),
        description: "Output as newline-delimited JSON events",
        agents: &[AgentType::Codex],
    },
    CommonOption {
        label: "Auto mode",
        action: OptionAction::Flag("--auto"),
        description: "Run autonomously without approval prompts",
        agents: &[AgentType::KiloCode],
    },
];

/// Post-session action choices
#[derive(Debug, Clone, Copy, PartialEq)]
enum PostAction {
    Diff,
    Merge,
    Keep,
    Discard,
}

/// Session state for persistence
#[derive(Debug, Clone, Serialize, Deserialize)]
struct SessionState {
    /// Session name
    name: String,
    /// Agent ID used
    agent_id: String,
    /// Was session created with --sandbox?
    #[serde(default)]
    sandbox: bool,
    /// Was session created with --worktree?
    #[serde(default)]
    worktree: bool,
    /// Docker container ID (if --sandbox was used)
    container_id: Option<String>,
    /// Worktree path (if --worktree was used)
    worktree_path: Option<String>,
    /// Worktree branch name
    worktree_branch: Option<String>,
    /// Extra CLI args passed to the agent (e.g., --dangerously-skip-permissions)
    #[serde(default)]
    extra_args: Vec<String>,
    /// Extra environment variables set for the agent (e.g., CLAUDE_CODE_EXPERIMENTAL_AGENT_TEAMS=1)
    #[serde(default)]
    env_vars: Vec<(String, String)>,
    /// Creation timestamp
    created_at: DateTime<Utc>,
    /// Current status
    status: SessionStatus,
}

/// Session status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
enum SessionStatus {
    Active,
    Stopped,
    Merged,
    Discarded,
}

impl std::fmt::Display for SessionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Active => write!(f, "active"),
            Self::Stopped => write!(f, "stopped"),
            Self::Merged => write!(f, "merged"),
            Self::Discarded => write!(f, "discarded"),
        }
    }
}

// ============================================================
// CLI ARGS
// ============================================================

#[derive(Args, Debug)]
#[allow(clippy::struct_excessive_bools)]
#[command(
    long_about = "Spawn an interactive AI agent terminal session.\n\n\
                  Quickly launch any configured agent (Claude, OpenRouter, Z.AI, Codex, Kilo Code) \
                  with the correct environment variables automatically configured.\n\n\
                  ## Isolation Modes\n\n\
                  - **Default**: Agent runs directly in the current working directory\n\
                  - **--worktree**: Agent runs in an isolated git worktree on a separate branch.\n\
                    After the session, you can view diffs, merge changes, keep for later, or discard.\n\
                  - **--sandbox**: Agent runs inside a Docker container with credential mounts.\n\
                    Changes are isolated to the container filesystem.\n\
                  - **--sandbox --worktree**: Maximum isolation - worktree for code, container for execution.\n\n\
                  ## Session Management\n\n\
                  Use --name to create named sessions that can be resumed later with --resume.\n\
                  Session state is stored in .chakravarti/sessions/<name>.yaml.",
    after_help = "Examples:\n\
                  # Interactive selection with options prompt\n\
                  ckrv term\n\n\
                  # Launch specific agent (skips agent selection)\n\
                  ckrv term --agent my-openrouter-agent\n\n\
                  # Isolated worktree — changes on a branch, merge when ready\n\
                  ckrv term --worktree\n\n\
                  # Docker sandbox — agent in a container\n\
                  ckrv term --sandbox\n\n\
                  # Maximum isolation — worktree + sandbox\n\
                  ckrv term --sandbox --worktree\n\n\
                  # Named session for resume\n\
                  ckrv term --worktree --name fix-auth\n\n\
                  # Resume a session\n\
                  ckrv term --resume fix-auth\n\n\
                  # Session management\n\
                  ckrv term --list-sessions\n\
                  ckrv term --cleanup fix-auth\n\n\
                  # Pass flags directly (scripting)\n\
                  ckrv term -- --dangerously-skip-permissions --continue"
)]
pub struct TermArgs {
    /// Agent ID to spawn directly (skips interactive agent selection)
    #[arg(short, long)]
    agent: Option<String>,

    /// List available agents and exit
    #[arg(short, long)]
    list: bool,

    /// Run agent in an isolated git worktree
    #[arg(long)]
    worktree: bool,

    /// Run agent in a Docker sandbox container
    #[arg(long)]
    sandbox: bool,

    /// Name for this session (enables resume with --resume)
    #[arg(long)]
    name: Option<String>,

    /// Resume a session. Optionally pass a session name, or omit to select interactively.
    #[arg(long, conflicts_with = "agent", num_args = 0..=1, default_missing_value = "")]
    resume: Option<String>,

    /// List all sessions and exit
    #[arg(long)]
    list_sessions: bool,

    /// Clean up a session (removes worktree and state)
    #[arg(long)]
    cleanup: Option<String>,

    /// Output in JSON format (for --list and --list-sessions)
    #[arg(long)]
    json: bool,

    /// Additional arguments to pass to the agent binary
    #[arg(last = true)]
    passthrough_args: Vec<String>,
}

// ============================================================
// EXECUTE
// ============================================================

/// Execute the term command.
///
/// # Errors
///
/// Returns an error if:
/// - Agent configuration cannot be loaded
/// - No agents are configured
/// - The specified agent is not found
/// - Worktree creation fails
/// - Session state cannot be saved
/// - Agent execution fails
pub async fn execute(mut args: TermArgs, json: bool, ui: &UiContext) -> anyhow::Result<()> {
    let cwd = std::env::current_dir()?;

    // Handle --list flag (list agents)
    if args.list {
        return list_agents(&args, json);
    }

    // Handle --list-sessions flag
    if args.list_sessions {
        return list_sessions(&args, json, ui);
    }

    // Handle --cleanup flag
    if let Some(session_name) = &args.cleanup {
        return cleanup_session(session_name, ui, &cwd).await;
    }

    // Handle --resume flag
    if let Some(session_name) = &args.resume {
        if session_name.is_empty() {
            let sessions = get_all_sessions()?;
            if sessions.is_empty() {
                ui.info("No Sessions", "No sessions available to resume.");
                return Ok(());
            }

            let items: Vec<String> = sessions
                .iter()
                .map(|(name, state)| {
                    let age = format_age(state.created_at);
                    format!("{} [{}] {} — {}", name, state.status, state.agent_id, age)
                })
                .collect();

            let selection = Select::with_theme(&ColorfulTheme::default())
                .with_prompt("Select a session to resume")
                .items(&items)
                .interact()?;

            let selected_name = &sessions[selection].0;
            return resume_session(&args, selected_name, json, ui, &cwd).await;
        }

        return resume_session(&args, session_name, json, ui, &cwd).await;
    }

    // Normal execution flow
    let agents_config = load_agents_config()?;
    let enabled_agents: Vec<&AgentConfig> =
        agents_config.agents.iter().filter(|a| a.enabled).collect();

    if enabled_agents.is_empty() {
        if json {
            println!(r#"{{"error": "no_agents", "message": "No agents configured"}}"#);
        } else {
            ui.error(
                "No Agents",
                "No agents configured. Run `ckrv ui` to configure agents.",
            );
        }
        return Ok(());
    }

    // Find agent to spawn
    let agent: AgentConfig = if let Some(agent_id) = &args.agent {
        agents_config
            .agents
            .iter()
            .find(|a| a.id == *agent_id && a.enabled)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("Agent '{agent_id}' not found or disabled"))?
    } else {
        select_agent_interactively(&enabled_agents)?
    };

    // Collect extra arguments and env vars (includes interactively-selected term options)
    let prompt_result = collect_args_and_env(&args, &agent.agent_type, json)?;
    let extra_args = prompt_result.args;

    // Apply interactively-selected term-level options to args
    if prompt_result.worktree {
        args.worktree = true;
    }
    if prompt_result.sandbox {
        args.sandbox = true;
    }
    if prompt_result.session_name.is_some() && args.name.is_none() {
        args.name = prompt_result.session_name;
    }

    // Build command based on agent type
    let (binary, mut env_vars) = build_agent_command(&agent)?;

    env_vars.extend(prompt_result.env_vars);

    // Generate session name — either user-provided or auto-generated
    if args.name.is_none() {
        args.name = Some(generate_session_name());
    }
    let session_name = args.name.as_ref().unwrap();

    // Generate session ID (used for worktree branch naming)
    let session_id = generate_session_id(Some(session_name));

    // Create worktree if requested
    let worktree_info = if args.worktree {
        Some(create_worktree(&session_id, &cwd, ui, json)?)
    } else {
        None
    };

    // Determine working directory
    let working_dir = worktree_info
        .as_ref()
        .map_or_else(|| cwd.clone(), |wt| wt.path.clone());

    // Always create session state (no longer gated by --name)
    create_session_state(
        session_name,
        &agent.id,
        worktree_info.as_ref(),
        &cwd,
        args.sandbox,
        args.worktree,
        &extra_args,
        &env_vars,
    )?;

    // Display session info
    if !json {
        display_session_info(
            ui,
            &agent,
            &binary,
            &env_vars,
            &extra_args,
            worktree_info.as_ref(),
            args.sandbox,
        );
    }

    // Execute the agent
    let exit_status = if args.sandbox {
        execute_in_sandbox(
            &binary,
            &env_vars,
            &extra_args,
            &agent,
            &working_dir,
            &session_id,
            ui,
        )
        .await?
    } else {
        execute_locally(&binary, &env_vars, &extra_args, &agent, &working_dir)?
    };

    // Print session info after agent exits
    if !json {
        let session_name = args.name.as_ref().unwrap();
        println!();
        ui.info(
            "Session",
            &format!(
                "\"{}\" — resume with: ckrv term --resume {}",
                session_name, session_name
            ),
        );
    }

    // Handle post-session for worktree mode
    if let Some(ref wt) = worktree_info {
        handle_post_session(wt, &cwd, ui, &agent.name)?;
    }

    // Always update session status to stopped
    let session_name = args.name.as_ref().unwrap();
    update_session_status(session_name, SessionStatus::Stopped)?;

    // Exit with same code as agent
    if !exit_status.success() {
        if let Some(code) = exit_status.code() {
            std::process::exit(code);
        }
    }

    Ok(())
}

// ============================================================
// HELPER FUNCTIONS
// ============================================================

fn list_agents(_args: &TermArgs, json: bool) -> anyhow::Result<()> {
    let agents_config = load_agents_config()?;
    let enabled_agents: Vec<&AgentConfig> =
        agents_config.agents.iter().filter(|a| a.enabled).collect();

    if json {
        let agents_json: Vec<_> = enabled_agents
            .iter()
            .map(|a| {
                serde_json::json!({
                    "id": a.id,
                    "name": a.name,
                    "type": format!("{:?}", a.agent_type),
                    "is_default": a.is_default,
                    "level": a.level,
                })
            })
            .collect();
        println!("{}", serde_json::to_string_pretty(&agents_json)?);
    } else {
        println!("## Available Agents\n");
        for agent in &enabled_agents {
            let type_badge = match agent.agent_type {
                AgentType::Claude => "claude",
                AgentType::ClaudeOpenRouter => "openrouter",
                AgentType::ClaudeGlm => "glm",
                AgentType::Codex => "codex",
                AgentType::KiloCode => "kilo",
            };
            let default_marker = if agent.is_default { " ★" } else { "" };
            println!(
                "  {} - {} [{}]{}",
                agent.id, agent.name, type_badge, default_marker
            );
        }
    }
    Ok(())
}

fn list_sessions(args: &TermArgs, json: bool, ui: &UiContext) -> anyhow::Result<()> {
    let sessions = get_all_sessions()?;

    if json {
        let sessions_json: Vec<_> = sessions
            .iter()
            .map(|(name, state)| {
                serde_json::json!({
                    "name": name,
                    "agent_id": state.agent_id,
                    "status": format!("{:?}", state.status).to_lowercase(),
                    "has_worktree": state.worktree_path.is_some(),
                    "has_container": state.container_id.is_some(),
                    "created_at": state.created_at.to_rfc3339(),
                })
            })
            .collect();
        println!("{}", serde_json::to_string_pretty(&sessions_json)?);
    } else {
        println!("## Active Sessions\n");
        if sessions.is_empty() {
            println!("  No sessions found.");
        } else {
            for (name, state) in &sessions {
                let mode = match (&state.worktree_path, &state.container_id) {
                    (Some(_), Some(_)) => "worktree+sandbox",
                    (Some(_), None) => "worktree",
                    (None, Some(_)) => "sandbox",
                    (None, None) => "local",
                };
                let age = format_age(state.created_at);
                println!(
                    "  {}  {}  [{}]  {}  {}",
                    name, state.agent_id, state.status, mode, age
                );
            }
        }
    }
    Ok(())
}

/// Load all sessions from the sessions directory.
///
/// Returns sessions sorted by creation time (most recent first).
fn get_all_sessions() -> anyhow::Result<Vec<(String, SessionState)>> {
    let sessions_dir = get_sessions_dir()?;
    if !sessions_dir.exists() {
        return Ok(Vec::new());
    }

    let mut sessions: Vec<(String, SessionState)> = Vec::new();
    for entry in std::fs::read_dir(&sessions_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().is_some_and(|e| e == "yaml") {
            if let Ok(content) = std::fs::read_to_string(&path) {
                if let Ok(state) = serde_yaml::from_str::<SessionState>(&content) {
                    if let Some(name) = path.file_stem().and_then(|s| s.to_str()) {
                        sessions.push((name.to_string(), state));
                    }
                }
            }
        }
    }

    sessions.sort_by(|a, b| b.1.created_at.cmp(&a.1.created_at));
    Ok(sessions)
}

async fn cleanup_session(session_name: &str, ui: &UiContext, cwd: &PathBuf) -> anyhow::Result<()> {
    let sessions_dir = get_sessions_dir()?;
    let session_path = sessions_dir.join(format!("{session_name}.yaml"));

    if !session_path.exists() {
        ui.error(
            "Session Not Found",
            &format!("Session '{session_name}' does not exist."),
        );
        return Err(anyhow::anyhow!("Session not found: {session_name}"));
    }

    let content = std::fs::read_to_string(&session_path)?;
    let state: SessionState = serde_yaml::from_str(&content)?;

    if let Some(container_id) = &state.container_id {
        ui.info(
            "Stopping Container",
            &format!("Stopping container {container_id}..."),
        );
        if let Ok(docker) = DockerClient::new() {
            let _ = docker.stop_session(container_id).await;
        }
    }

    if let Some(wt_path) = &state.worktree_path {
        let wt_path = PathBuf::from(wt_path);
        if wt_path.exists() {
            ui.info(
                "Removing Worktree",
                &format!("Removing worktree at {}...", wt_path.display()),
            );

            if let Some(branch) = &state.worktree_branch {
                let _ = Command::new("git")
                    .args(["worktree", "remove", "--force"])
                    .arg(&wt_path)
                    .current_dir(cwd)
                    .status();

                let _ = Command::new("git")
                    .args(["branch", "-D"])
                    .arg(branch)
                    .current_dir(cwd)
                    .status();
            } else {
                let _ = std::fs::remove_dir_all(&wt_path);
            }
        }
    }

    std::fs::remove_file(&session_path)?;
    ui.success(
        "Session Cleaned Up",
        &format!("Session '{session_name}' has been removed."),
    );

    Ok(())
}

async fn resume_session(
    _args: &TermArgs,
    session_name: &str,
    json: bool,
    ui: &UiContext,
    cwd: &PathBuf,
) -> anyhow::Result<()> {
    let sessions_dir = get_sessions_dir()?;
    let session_path = sessions_dir.join(format!("{session_name}.yaml"));

    if !session_path.exists() {
        ui.error(
            "Session Not Found",
            &format!("Session '{session_name}' does not exist."),
        );
        return Err(anyhow::anyhow!("Session not found: {session_name}"));
    }

    let content = std::fs::read_to_string(&session_path)?;
    let state: SessionState = serde_yaml::from_str(&content)?;

    if state.status == SessionStatus::Active {
        ui.warn(
            "Session Active",
            "This session may still be running. Use --cleanup to force remove.",
        );
    }

    let agents_config = load_agents_config()?;
    let agent = agents_config
        .agents
        .iter()
        .find(|a| a.id == state.agent_id && a.enabled)
        .cloned()
        .ok_or_else(|| anyhow::anyhow!("Agent '{}' not found or disabled", state.agent_id))?;

    let working_dir = match &state.worktree_path {
        Some(path) => {
            let p = PathBuf::from(path);
            if !p.exists() {
                return Err(anyhow::anyhow!(
                    "Worktree for session '{session_name}' no longer exists at {path}"
                ));
            }
            p
        }
        None => cwd.clone(),
    };

    let use_sandbox = state.sandbox;

    let (binary, mut env_vars) = build_agent_command(&agent)?;

    // Restore persisted extra args and env vars from the session
    let extra_args = if state.extra_args.is_empty() {
        // Fallback: if session was created before args were persisted, prompt
        let prompt_result = collect_args_and_env(_args, &agent.agent_type, json)?;
        env_vars.extend(prompt_result.env_vars);
        prompt_result.args
    } else {
        // Merge persisted env vars into the build_agent_command env vars
        env_vars.extend(state.env_vars.clone());
        state.extra_args.clone()
    };

    if !json {
        ui.success(
            "Resuming Session",
            &format!("{} ({})", agent.name, agent.id),
        );
        if state.worktree_path.is_some() {
            println!("  Worktree: {}", working_dir.display());
        }
        if use_sandbox {
            println!("  Mode: Docker sandbox");
        }
        if !env_vars.is_empty() {
            println!(
                "  Environment: {}",
                env_vars
                    .iter()
                    .map(|(k, _)| k.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            );
        }
        if !extra_args.is_empty() {
            println!("  Extra args: {}", extra_args.join(" "));
        }
        println!();
    }

    let mut state = state;
    state.status = SessionStatus::Active;
    save_session_state(session_name, &state)?;

    let exit_status = if use_sandbox {
        let session_id = format!("resume-{session_name}");
        execute_in_sandbox(
            &binary,
            &env_vars,
            &extra_args,
            &agent,
            &working_dir,
            &session_id,
            ui,
        )
        .await?
    } else {
        execute_locally(&binary, &env_vars, &extra_args, &agent, &working_dir)?
    };

    if state.worktree {
        let wt = Worktree {
            path: working_dir.clone(),
            branch: state.worktree_branch.clone().unwrap_or_default(),
            job_id: session_name.to_string(),
            attempt_id: "1".to_string(),
            base_commit: String::new(),
            status: ckrv_git::WorktreeStatus::Ready,
        };
        handle_post_session(&wt, cwd, ui, &agent.name)?;
    }

    update_session_status(session_name, SessionStatus::Stopped)?;

    if !exit_status.success() {
        if let Some(code) = exit_status.code() {
            std::process::exit(code);
        }
    }

    Ok(())
}

/// Select an agent interactively from the list.
///
/// # Errors
///
/// Returns an error if the interactive selection fails.
fn select_agent_interactively(enabled_agents: &[&AgentConfig]) -> anyhow::Result<AgentConfig> {
    let items: Vec<String> = enabled_agents
        .iter()
        .map(|a| {
            let type_badge = match a.agent_type {
                AgentType::Claude => "claude",
                AgentType::ClaudeOpenRouter => "openrouter",
                AgentType::ClaudeGlm => "glm",
                AgentType::Codex => "codex",
                AgentType::KiloCode => "kilo",
            };
            let default_marker = if a.is_default { " ★" } else { "" };
            format!("{} ({}) [{}]{}", a.name, a.id, type_badge, default_marker)
        })
        .collect();

    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Select an agent to spawn")
        .items(&items)
        .default(0)
        .interact()?;

    Ok((*enabled_agents[selection]).clone())
}

/// Collect command-line arguments and environment variables.
///
/// # Errors
///
/// Returns an error if the interactive prompt fails.
fn collect_args_and_env(
    args: &TermArgs,
    agent_type: &AgentType,
    json: bool,
) -> anyhow::Result<PromptResult> {
    if !args.passthrough_args.is_empty() {
        return Ok(PromptResult {
            args: args.passthrough_args.clone(),
            env_vars: Vec::new(),
            worktree: false,
            sandbox: false,
            session_name: None,
        });
    }

    if json {
        Ok(PromptResult {
            args: Vec::new(),
            env_vars: Vec::new(),
            worktree: false,
            sandbox: false,
            session_name: None,
        })
    } else {
        prompt_for_options(agent_type)
    }
}

/// Generate a session ID from an optional name.
fn generate_session_id(name: Option<&str>) -> String {
    name.map_or_else(
        || {
            let uuid: String = uuid::Uuid::new_v4().to_string().chars().take(8).collect();
            format!("term-{uuid}")
        },
        |n| format!("term-{n}"),
    )
}

/// Generate a memorable, terminal-safe session name.
///
/// Format: `adjective-animal-NNNN` (e.g., "brave-panda-4821", "swift-falcon-0137").
/// The 4-digit suffix prevents collisions across concurrent or rapid sessions.
fn generate_session_name() -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    const ADJECTIVES: &[&str] = &[
        "bold", "brave", "calm", "cool", "crisp", "deft", "fair", "fast", "fine", "firm", "fond",
        "free", "glad", "gold", "good", "holy", "keen", "kind", "lean", "live", "neat", "nice",
        "pure", "rare", "rich", "safe", "sage", "slim", "soft", "sure", "tall", "tidy", "true",
        "vast", "warm", "wide", "wild", "wise", "zany", "epic", "swift",
    ];

    const ANIMALS: &[&str] = &[
        "ape", "bat", "bear", "bison", "boar", "bull", "civet", "cobra", "crane", "crow", "deer",
        "dove", "eagle", "elephant", "fox", "frog", "gaur", "gecko", "goat", "hawk", "hare",
        "heron", "ibis", "jackal", "kite", "koel", "langur", "lion", "moth", "mongoose", "myna",
        "newt", "otter", "owl", "panda", "peacock", "rat", "rhino", "robin", "shrew", "stork",
        "tiger", "viper", "wolf",
    ];

    let mut hasher = DefaultHasher::new();
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos()
        .hash(&mut hasher);
    let hash = hasher.finish();

    let adj = ADJECTIVES[(hash as usize) % ADJECTIVES.len()];
    let animal = ANIMALS[((hash >> 16) as usize) % ANIMALS.len()];
    let suffix = (hash >> 32) % 10000;

    let (adj, animal) = if adj == "holy" {
        ("holy", "cow")
    } else {
        (adj, animal)
    };

    format!("{adj}-{animal}-{suffix:04}")
}

fn create_worktree(
    session_id: &str,
    cwd: &std::path::Path,
    ui: &UiContext,
    json: bool,
) -> anyhow::Result<Worktree> {
    let manager = DefaultWorktreeManager::new(cwd)?;

    let job_id = session_id.strip_prefix("term-").unwrap_or(session_id);

    let worktree = manager.create(job_id, "1")?;

    if !json {
        ui.success("Created Worktree", &format!("Branch: {}", worktree.branch));
        println!("  Path: {}", worktree.path.display());
        println!();
    }

    Ok(worktree)
}

fn create_session_state(
    session_name: &str,
    agent_id: &str,
    worktree_info: Option<&Worktree>,
    _cwd: &std::path::Path,
    sandbox: bool,
    worktree: bool,
    extra_args: &[String],
    env_vars: &[(String, String)],
) -> anyhow::Result<()> {
    let sessions_dir = get_sessions_dir()?;
    std::fs::create_dir_all(&sessions_dir)?;

    let session_path = sessions_dir.join(format!("{session_name}.yaml"));

    if session_path.exists() {
        return Err(anyhow::anyhow!(
            "Session '{session_name}' already exists. Use --resume to continue or --cleanup to remove."
        ));
    }

    let state = SessionState {
        name: session_name.to_string(),
        agent_id: agent_id.to_string(),
        sandbox,
        worktree,
        container_id: None,
        worktree_path: worktree_info.map(|wt| wt.path.to_string_lossy().to_string()),
        worktree_branch: worktree_info.map(|wt| wt.branch.clone()),
        extra_args: extra_args.to_vec(),
        env_vars: env_vars.to_vec(),
        created_at: Utc::now(),
        status: SessionStatus::Active,
    };

    save_session_state(session_name, &state)
}

fn save_session_state(session_name: &str, state: &SessionState) -> anyhow::Result<()> {
    let sessions_dir = get_sessions_dir()?;
    std::fs::create_dir_all(&sessions_dir)?;
    let session_path = sessions_dir.join(format!("{session_name}.yaml"));
    let yaml = serde_yaml::to_string(state)?;
    std::fs::write(&session_path, yaml)?;
    Ok(())
}

fn update_session_status(session_name: &str, status: SessionStatus) -> anyhow::Result<()> {
    let sessions_dir = get_sessions_dir()?;
    let session_path = sessions_dir.join(format!("{session_name}.yaml"));

    if !session_path.exists() {
        return Ok(());
    }

    let content = std::fs::read_to_string(&session_path)?;
    let mut state: SessionState = serde_yaml::from_str(&content)?;
    state.status = status;
    save_session_state(session_name, &state)
}

fn get_sessions_dir() -> anyhow::Result<PathBuf> {
    let cwd = std::env::current_dir()?;
    Ok(cwd.join(".chakravarti").join("sessions"))
}

fn display_session_info(
    ui: &UiContext,
    agent: &AgentConfig,
    binary: &str,
    env_vars: &[(String, String)],
    extra_args: &[String],
    worktree_info: Option<&Worktree>,
    sandbox: bool,
) {
    ui.success("Spawning", &format!("{} ({})", agent.name, agent.id));
    println!("  Binary: {binary}");

    if let Some(wt) = worktree_info {
        println!("  Worktree: {}", wt.path.display());
        println!("  Branch: {}", wt.branch);
    }

    if sandbox {
        println!("  Mode: Docker sandbox");
    }

    if !env_vars.is_empty() {
        println!(
            "  Environment: {}",
            env_vars
                .iter()
                .map(|(k, _)| k.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        );
    }

    if !extra_args.is_empty() {
        println!("  Extra args: {}", extra_args.join(" "));
    }
    println!();
}

/// Map CLI `AgentType` to sandbox `AgentType`.
fn to_sandbox_agent_type(cli_type: &AgentType) -> ckrv_sandbox::AgentType {
    match cli_type {
        AgentType::Claude | AgentType::ClaudeOpenRouter | AgentType::ClaudeGlm => {
            ckrv_sandbox::AgentType::Claude
        }
        AgentType::Codex => ckrv_sandbox::AgentType::Codex,
        AgentType::KiloCode => ckrv_sandbox::AgentType::KiloCode,
    }
}

async fn execute_in_sandbox(
    binary: &str,
    env_vars: &[(String, String)],
    extra_args: &[String],
    agent: &AgentConfig,
    working_dir: &std::path::Path,
    _session_id: &str,
    ui: &UiContext,
) -> anyhow::Result<std::process::ExitStatus> {
    // Check Docker availability
    let mut docker = DockerClient::new().map_err(|e| {
        anyhow::anyhow!(
            "Docker is not available: {}\n\n\
             Please ensure Docker is installed and running:\n\
             - Install: https://docs.docker.com/get-docker/\n\
             - Start: Run 'docker info' to verify Docker is running",
            e
        )
    })?;

    // Health check
    docker.health_check().await.map_err(|_| {
        anyhow::anyhow!(
            "Docker health check failed.\n\n\
             Please ensure Docker daemon is running:\n\
             - Try: docker info\n\
             - On macOS: Open Docker Desktop\n\
             - On Linux: sudo systemctl start docker"
        )
    })?;

    // Set agent-specific Docker image
    let image = match &agent.agent_type {
        AgentType::Codex => "ckrv-codex:latest",
        AgentType::KiloCode => "ckrv-kilo:latest",
        _ => "ckrv-claude:latest",
    };
    docker.set_image(image);

    // Get agent provider for credential mounts
    let sandbox_type = to_sandbox_agent_type(&agent.agent_type);
    let agent_provider = ckrv_sandbox::create_agent(sandbox_type);

    // Get credential mounts - set container home based on agent type
    let host_home = std::env::var("HOME").unwrap_or_default();
    let container_home = match &agent.agent_type {
        AgentType::Codex => "/home/codex",
        AgentType::KiloCode => "/home/kilo",
        _ => "/home/claude",
    };
    let mounts = agent_provider.config_mounts(&host_home, &container_home);

    // Convert to BindMount
    let extra_mounts: Vec<BindMount> = mounts
        .into_iter()
        .filter_map(|m| {
            let source = m.source?;
            let target = m.target?;
            let read_only = m.read_only.unwrap_or(true);
            Some(BindMount {
                source,
                target,
                read_only,
            })
        })
        .collect();

    // Convert env vars to HashMap
    let mut env_map: HashMap<String, String> = env_vars.iter().cloned().collect();

    // Add HOME to env for the container
    env_map.insert("HOME".to_string(), container_home.to_string());

    // Add agent-specific env vars from config
    if let Some(custom_env) = &agent.env_vars {
        env_map.extend(custom_env.clone());
    }

    // Create container session - mount host path to /workspace in container
    let host_path = working_dir.to_string_lossy().to_string();
    let container_workdir = "/workspace";

    let container_id = docker
        .create_session(
            container_workdir,
            &host_path,
            container_workdir,
            env_map.clone(),
            extra_mounts,
        )
        .await
        .map_err(|e| anyhow::anyhow!("Failed to create Docker container: {e}"))?;

    ui.info(
        "Container Started",
        &format!("Container ID: {}...", &container_id[..12]),
    );

    // Set up signal handler for cleanup
    let container_id_clone = container_id.clone();
    let cleanup_done = Arc::new(AtomicBool::new(false));
    let cleanup_done_clone = cleanup_done.clone();

    tokio::spawn(async move {
        let _ = tokio::signal::ctrl_c().await;
        if !cleanup_done_clone.load(Ordering::SeqCst) {
            eprintln!("\nStopping container...");
            if let Ok(docker) = DockerClient::new() {
                let _ = docker.stop_session(&container_id_clone).await;
            }
        }
    });

    // Build agent command
    let mut agent_cmd: Vec<String> = vec![binary.to_string()];

    // Add extra args from agent config
    if let Some(config_args) = &agent.extra_args {
        agent_cmd.extend(config_args.clone());
    }

    // Add passthrough args
    agent_cmd.extend(extra_args.to_vec());

    // Build docker exec command with env vars
    let mut docker_args = vec![
        "exec".to_string(),
        "-it".to_string(),
        "-w".to_string(),
        container_workdir.to_string(),
    ];

    // Pass env vars explicitly via -e flags
    for (key, value) in env_vars {
        docker_args.push("-e".to_string());
        docker_args.push(format!("{}={}", key, value));
    }

    // Add TERM for TUI rendering
    docker_args.push("-e".to_string());
    docker_args.push("TERM=xterm-256color".to_string());
    docker_args.push("-e".to_string());
    docker_args.push("COLORTERM=truecolor".to_string());

    // Add container ID
    docker_args.push(container_id.clone());

    // Execute via docker exec -it for interactive PTY
    let status = Command::new("docker")
        .args(&docker_args)
        .args(&agent_cmd)
        .status();

    // Mark cleanup as done (successful completion)
    cleanup_done.store(true, Ordering::SeqCst);

    // Stop container
    if let Err(e) = docker.stop_session(&container_id).await {
        ui.warn("Cleanup Warning", &format!("Failed to stop container: {e}"));
    }

    status.map_err(|e| anyhow::anyhow!("Failed to execute in container: {e}"))
}

fn execute_locally(
    binary: &str,
    env_vars: &[(String, String)],
    extra_args: &[String],
    agent: &AgentConfig,
    working_dir: &std::path::Path,
) -> anyhow::Result<std::process::ExitStatus> {
    let mut cmd = Command::new(binary);

    for (key, value) in env_vars {
        cmd.env(key, value);
    }

    cmd.current_dir(working_dir);

    if let Some(config_args) = &agent.extra_args {
        cmd.args(config_args);
    }

    if let Some(custom_env) = &agent.env_vars {
        for (key, value) in custom_env {
            cmd.env(key, value);
        }
    }

    if !extra_args.is_empty() {
        cmd.args(extra_args);
    }

    cmd.status()
        .map_err(|e| anyhow::anyhow!("Failed to spawn agent: {e}"))
}

fn handle_post_session(
    worktree: &Worktree,
    cwd: &std::path::Path,
    ui: &UiContext,
    agent_name: &str,
) -> anyhow::Result<()> {
    // Check for tracked file modifications
    let has_tracked_changes = Command::new("git")
        .args(["diff", "--quiet", "HEAD"])
        .current_dir(&worktree.path)
        .status()
        .is_ok_and(|s| !s.success());

    // Check for untracked (new) files created by the agent
    let has_untracked = Command::new("git")
        .args(["status", "--porcelain"])
        .current_dir(&worktree.path)
        .output()
        .map(|o| !o.stdout.is_empty())
        .unwrap_or(false);

    let has_changes = has_tracked_changes || has_untracked;

    if !has_changes {
        ui.info("No Changes", "Agent made no changes to the worktree.");
        return Ok(());
    }

    loop {
        let action = post_session_prompt()?;

        match action {
            PostAction::Diff => {
                show_diff(&worktree.path)?;
            }
            PostAction::Merge => {
                merge_worktree(worktree, cwd, ui, agent_name)?;
                return Ok(());
            }
            PostAction::Keep => {
                ui.success(
                    "Worktree Kept",
                    &format!("Worktree preserved at: {}", worktree.path.display()),
                );
                println!("  Branch: {}", worktree.branch);
                println!("\n  To merge later: git merge {}", worktree.branch);
                println!("  To remove: ckrv term --cleanup <session-name>");
                return Ok(());
            }
            PostAction::Discard => {
                discard_worktree(worktree, cwd, ui)?;
                return Ok(());
            }
        }
    }
}

fn post_session_prompt() -> anyhow::Result<PostAction> {
    let items = [
        "View diff",
        "Merge into current branch",
        "Keep worktree for later",
        "Discard all changes",
    ];

    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("What would you like to do with the changes?")
        .items(&items)
        .default(0)
        .interact()?;

    match selection {
        0 => Ok(PostAction::Diff),
        1 => Ok(PostAction::Merge),
        2 => Ok(PostAction::Keep),
        3 => Ok(PostAction::Discard),
        _ => Ok(PostAction::Keep),
    }
}

fn show_diff(worktree_path: &std::path::Path) -> anyhow::Result<()> {
    println!("\n--- Changes in worktree ---\n");

    // Show tracked file changes
    let status = Command::new("git")
        .args(["diff", "HEAD"])
        .current_dir(worktree_path)
        .status()?;

    if !status.success() {
        println!("(No diff available)");
    }

    // Show untracked (new) files
    let untracked = Command::new("git")
        .args(["ls-files", "--others", "--exclude-standard"])
        .current_dir(worktree_path)
        .output()?;

    let untracked_files = String::from_utf8_lossy(&untracked.stdout);
    if !untracked_files.is_empty() {
        println!("\nNew files (untracked):");
        for file in untracked_files.lines() {
            println!("  + {file}");
        }
    }

    println!("\n---------------------------\n");
    Ok(())
}

/// Prompt the user for a commit message
fn prompt_commit_message(worktree: &Worktree, agent_name: &str) -> anyhow::Result<String> {
    let auto_msg = format!(
        "feat(term): {} session changes via {}",
        worktree.job_id, agent_name
    );

    let items = [
        "Write custom message...",
        &format!("Use auto-generated: \"{auto_msg}\""),
    ];

    let choice = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Commit message")
        .items(&items)
        .default(0)
        .interact()?;

    match choice {
        0 => {
            let msg: String = Input::with_theme(&ColorfulTheme::default())
                .with_prompt("Enter commit message")
                .interact_text()?;
            Ok(msg)
        }
        _ => Ok(auto_msg),
    }
}

fn merge_worktree(
    worktree: &Worktree,
    cwd: &std::path::Path,
    ui: &UiContext,
    agent_name: &str,
) -> anyhow::Result<()> {
    ui.info(
        "Committing Changes",
        &format!("Committing changes in {}...", worktree.branch),
    );

    // Git add
    let add_status = Command::new("git")
        .args(["add", "."])
        .current_dir(&worktree.path)
        .status()?;

    if !add_status.success() {
        return Err(anyhow::anyhow!("Failed to stage changes in worktree"));
    }

    // Check if there are staged changes
    let has_staged = Command::new("git")
        .args(["diff", "--staged", "--quiet"])
        .current_dir(&worktree.path)
        .status()
        .map(|s| !s.success())
        .unwrap_or(false);

    if has_staged {
        let commit_msg = prompt_commit_message(worktree, agent_name)?;
        let commit_status = Command::new("git")
            .args(["commit", "-m", &commit_msg])
            .current_dir(&worktree.path)
            .status()?;

        if !commit_status.success() {
            return Err(anyhow::anyhow!("Failed to commit changes in worktree"));
        }
    }

    ui.info(
        "Merging",
        &format!("Merging {} into current branch...", worktree.branch),
    );

    let merge_status = Command::new("git")
        .args(["merge", "--no-ff", "--no-edit", &worktree.branch])
        .current_dir(cwd)
        .status()?;

    if !merge_status.success() {
        // Check for conflicts
        let has_conflicts = Command::new("git")
            .args(["diff", "--check"])
            .current_dir(cwd)
            .status()
            .map(|s| !s.success())
            .unwrap_or(false);

        if has_conflicts {
            ui.error(
                "Merge Conflicts",
                "Conflicts detected. Please resolve manually:",
            );
            println!("  git status                    # See conflicted files");
            println!("  git mergetool                 # Resolve conflicts");
            println!("  git commit                    # Complete merge");
            println!("  git merge --abort             # Abort and try again");
            return Err(anyhow::anyhow!("Merge conflicts need manual resolution"));
        }

        return Err(anyhow::anyhow!("Failed to merge worktree branch"));
    }

    ui.success(
        "Merged",
        &format!("Successfully merged {}", worktree.branch),
    );

    // Cleanup worktree
    let manager = DefaultWorktreeManager::new(cwd)?;
    manager.cleanup(worktree)?;

    ui.success("Cleaned Up", "Worktree removed");
    Ok(())
}

fn discard_worktree(
    worktree: &Worktree,
    cwd: &std::path::Path,
    ui: &UiContext,
) -> anyhow::Result<()> {
    ui.info("Discarding", "Removing worktree and branch...");

    let manager = DefaultWorktreeManager::new(cwd)?;
    manager.cleanup(worktree)?;

    ui.success("Discarded", "Worktree and branch removed");
    Ok(())
}

fn format_age(created_at: DateTime<Utc>) -> String {
    let now = Utc::now();
    let diff = now.signed_duration_since(created_at);

    if diff.num_minutes() < 1 {
        "just now".to_string()
    } else if diff.num_minutes() < 60 {
        format!("{}m ago", diff.num_minutes())
    } else if diff.num_hours() < 24 {
        format!("{}h ago", diff.num_hours())
    } else {
        format!("{}d ago", diff.num_days())
    }
}

/// Result of interactive options prompt
struct PromptResult {
    args: Vec<String>,
    env_vars: Vec<(String, String)>,
    /// Whether the user selected worktree isolation mode interactively
    worktree: bool,
    /// Whether the user selected sandbox isolation mode interactively
    sandbox: bool,
    /// Session name if provided interactively
    session_name: Option<String>,
}

/// Prompt user interactively for common options and custom args.
fn prompt_for_options(agent_type: &AgentType) -> anyhow::Result<PromptResult> {
    let theme = ColorfulTheme::default();
    let mut args: Vec<String> = Vec::new();
    let mut env_vars: Vec<(String, String)> = Vec::new();
    let mut worktree = false;
    let mut sandbox = false;
    let mut session_name: Option<String> = None;

    let applicable: Vec<&CommonOption> = COMMON_OPTIONS
        .iter()
        .filter(|opt| opt.agents.contains(agent_type))
        .collect();

    let launch_choice = Select::with_theme(&theme)
        .with_prompt("Launch options")
        .items(&["Launch directly", "Configure options..."])
        .default(0)
        .interact()?;

    if launch_choice == 0 {
        return Ok(PromptResult {
            args,
            env_vars,
            worktree,
            sandbox,
            session_name,
        });
    }

    let term_options = [
        "Worktree isolation - Run agent in an isolated git worktree branch",
        "Docker sandbox - Run agent inside a Docker container",
        "Name session - Create a named session for resume later",
    ];

    let term_selections = MultiSelect::with_theme(&theme)
        .with_prompt("Isolation modes (Space to toggle, Enter to confirm)")
        .items(&term_options)
        .interact()?;

    for idx in &term_selections {
        match idx {
            0 => worktree = true,
            1 => sandbox = true,
            2 => {
                let name: String = Input::with_theme(&theme)
                    .with_prompt("Session name")
                    .interact_text()?;
                if !name.trim().is_empty() {
                    session_name = Some(name.trim().to_string());
                }
            }
            _ => {}
        }
    }

    if !applicable.is_empty() {
        let items: Vec<String> = applicable
            .iter()
            .map(|opt| format!("{} - {}", opt.label, opt.description))
            .collect();

        let selections = MultiSelect::with_theme(&theme)
            .with_prompt("Agent options (Space to toggle, Enter to confirm)")
            .items(&items)
            .interact()?;

        for idx in selections {
            match &applicable[idx].action {
                OptionAction::Flag(flag) => {
                    for part in flag.split_whitespace() {
                        args.push(part.to_string());
                    }
                }
                OptionAction::EnvVar(key, value) => {
                    env_vars.push(((*key).to_string(), (*value).to_string()));
                }
            }
        }
    }

    let custom: String = Input::with_theme(&theme)
        .with_prompt("Additional arguments (or press Enter to skip)")
        .allow_empty(true)
        .interact_text()?;

    if !custom.trim().is_empty() {
        for arg in shell_words::split(&custom)? {
            args.push(arg);
        }
    }

    Ok(PromptResult {
        args,
        env_vars,
        worktree,
        sandbox,
        session_name,
    })
}

/// Build command binary and environment variables for an agent
fn build_agent_command(agent: &AgentConfig) -> anyhow::Result<(String, Vec<(String, String)>)> {
    let binary = agent
        .binary_path
        .clone()
        .unwrap_or_else(|| match agent.agent_type {
            AgentType::Claude | AgentType::ClaudeOpenRouter | AgentType::ClaudeGlm => {
                "claude".to_string()
            }
            AgentType::Codex => "codex".to_string(),
            AgentType::KiloCode => "kilo".to_string(),
        });

    let mut env_vars: Vec<(String, String)> = Vec::new();

    match agent.agent_type {
        AgentType::Claude => {
            // Native Claude - no extra env vars needed
        }
        AgentType::ClaudeOpenRouter => {
            let config = agent
                .openrouter
                .as_ref()
                .ok_or_else(|| anyhow::anyhow!("OpenRouter agent missing 'openrouter' config"))?;

            let base_url = config
                .base_url
                .clone()
                .unwrap_or_else(|| "https://openrouter.ai/api".to_string());
            env_vars.push(("ANTHROPIC_BASE_URL".to_string(), base_url));

            if let Some(api_key) = &config.api_key {
                env_vars.push(("ANTHROPIC_AUTH_TOKEN".to_string(), api_key.clone()));
            } else if let Ok(key) = std::env::var("OPENROUTER_API_KEY") {
                env_vars.push(("ANTHROPIC_AUTH_TOKEN".to_string(), key));
            }

            env_vars.push(("ANTHROPIC_API_KEY".to_string(), String::new()));

            env_vars.push((
                "ANTHROPIC_DEFAULT_SONNET_MODEL".to_string(),
                config.model.clone(),
            ));
            env_vars.push((
                "ANTHROPIC_DEFAULT_OPUS_MODEL".to_string(),
                config.model.clone(),
            ));
            env_vars.push((
                "ANTHROPIC_DEFAULT_HAIKU_MODEL".to_string(),
                config.model.clone(),
            ));
        }
        AgentType::ClaudeGlm => {
            let config = agent
                .glm
                .as_ref()
                .ok_or_else(|| anyhow::anyhow!("GLM agent missing 'glm' config"))?;

            env_vars.push((
                "ANTHROPIC_BASE_URL".to_string(),
                "https://api.z.ai/api/anthropic".to_string(),
            ));

            if let Some(api_key) = &config.api_key {
                env_vars.push(("ANTHROPIC_AUTH_TOKEN".to_string(), api_key.clone()));
            } else if let Ok(key) = std::env::var("ZAI_API_KEY") {
                env_vars.push(("ANTHROPIC_AUTH_TOKEN".to_string(), key));
            }

            env_vars.push(("ANTHROPIC_API_KEY".to_string(), String::new()));

            env_vars.push((
                "ANTHROPIC_DEFAULT_SONNET_MODEL".to_string(),
                config.model.clone(),
            ));
            env_vars.push((
                "ANTHROPIC_DEFAULT_OPUS_MODEL".to_string(),
                config.model.clone(),
            ));
            env_vars.push((
                "ANTHROPIC_DEFAULT_HAIKU_MODEL".to_string(),
                config.model.clone(),
            ));

            let timeout = config.timeout_ms.unwrap_or(3_000_000);
            env_vars.push(("API_TIMEOUT_MS".to_string(), timeout.to_string()));
        }
        AgentType::Codex => {
            // Native Codex - no extra env vars needed
        }
        AgentType::KiloCode => {
            // Kilo Code uses file-based auth (~/.config/kilo/) - no extra env vars needed
        }
    }

    Ok((binary, env_vars))
}
