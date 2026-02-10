//! # Term Command
//!
//! Spawns an interactive AI agent terminal session.
//!
//! ## Overview
//!
//! This command provides a quick way to launch any configured AI agent
//! in an interactive terminal session. It handles all the environment
//! variable setup required for different agent types (OpenRouter, Z.AI, etc).
//!
//! ## Usage
//!
//! ```bash
//! ckrv term           # Interactive agent selection with options
//! ckrv term --agent claude-default  # Directly spawn specific agent
//! ckrv term --list    # List available agents
//! ```

// Standard library
use std::process::Command;

// External crates
use clap::Args;
use dialoguer::{theme::ColorfulTheme, Input, MultiSelect, Select};

// Internal modules
use crate::services::agent_lookup::{load_agents_config, AgentConfig, AgentType};
use crate::ui::UiContext;

// ============================================================

/// Common agent options that can be selected interactively
#[derive(Debug, Clone)]
struct CommonOption {
    label: &'static str,
    action: OptionAction,
    description: &'static str,
    /// Which agent types this option applies to
    agents: &'static [AgentType],
}

/// Action to take when an option is selected
#[derive(Debug, Clone)]
enum OptionAction {
    /// Pass flag(s) to the command line
    Flag(&'static str),
    /// Set an environment variable
    EnvVar(&'static str, &'static str),
}

/// All Claude-based agent types (native, OpenRouter, GLM)
const CLAUDE_AGENTS: &[AgentType] = &[
    AgentType::Claude,
    AgentType::ClaudeOpenRouter,
    AgentType::ClaudeGlm,
];

const COMMON_OPTIONS: &[CommonOption] = &[
    // === Claude-specific options ===
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
    // === Codex-specific options ===
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
    // === Kilo Code-specific options ===
    CommonOption {
        label: "Auto mode",
        action: OptionAction::Flag("--auto"),
        description: "Run autonomously without approval prompts",
        agents: &[AgentType::KiloCode],
    },
];

// ============================================================

#[derive(Args, Debug)]
#[command(
    long_about = "Spawn an interactive AI agent terminal session.\n\n\
                  Quickly launch any configured agent (Claude, OpenRouter, Z.AI, Codex, Kilo Code) \
                  with the correct environment variables automatically configured.\n\n\
                  Without arguments, presents an interactive selection menu with options \
                  for common flags. Use -- to pass arguments directly for scripting.",
    after_help = "Examples:\n\
                  # Interactive selection with options prompt\n\
                  ckrv term\n\n\
                  # Launch specific agent (skips agent selection)\n\
                  ckrv term --agent my-openrouter-agent\n\n\
                  # Pass flags directly (scripting)\n\
                  ckrv term -- --dangerously-skip-permissions --continue\n\n\
                  # List available agents\n\
                  ckrv term --list"
)]
pub struct TermArgs {
    /// Agent ID to spawn directly (skips interactive agent selection)
    #[arg(short, long)]
    agent: Option<String>,

    /// List available agents and exit
    #[arg(short, long)]
    list: bool,

    /// Additional arguments to pass to the agent binary
    #[arg(last = true)]
    passthrough_args: Vec<String>,
}

// ============================================================

pub async fn execute(args: TermArgs, json: bool, ui: &UiContext) -> anyhow::Result<()> {
    // Load agents configuration
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

    // Handle --list flag
    if args.list {
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
            ui.markdown("## Available Agents\n");
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
        return Ok(());
    }

    // Find agent to spawn
    let agent = if let Some(agent_id) = &args.agent {
        // Direct agent selection
        enabled_agents
            .iter()
            .find(|a| a.id == *agent_id)
            .copied()
            .ok_or_else(|| anyhow::anyhow!("Agent '{}' not found or disabled", agent_id))?
    } else {
        // Interactive selection
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

        enabled_agents[selection]
    };

    // Collect extra arguments and env vars - either from passthrough or interactive prompt
    let (extra_args, prompt_env_vars) = if !args.passthrough_args.is_empty() {
        // Use passthrough args directly (scripting mode)
        (args.passthrough_args.clone(), Vec::new())
    } else if !json {
        // Interactive options prompt (filtered by agent type)
        let result = prompt_for_options(&agent.agent_type)?;
        (result.args, result.env_vars)
    } else {
        (Vec::new(), Vec::new())
    };

    // Build command based on agent type
    let (binary, mut env_vars) = build_agent_command(agent)?;

    // Add env vars from interactive prompt
    env_vars.extend(prompt_env_vars);

    if !json {
        ui.success("Spawning", &format!("{} ({})", agent.name, agent.id));
        println!("  Binary: {}", binary);
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
        println!(); // Blank line before spawning
    }

    // Spawn the agent process
    let mut cmd = Command::new(&binary);

    // Set environment variables
    for (key, value) in &env_vars {
        cmd.env(key, value);
    }

    // Add any extra args from agent config
    if let Some(config_args) = &agent.extra_args {
        cmd.args(config_args);
    }

    // Add any custom env vars from config
    if let Some(custom_env) = &agent.env_vars {
        for (key, value) in custom_env {
            cmd.env(key, value);
        }
    }

    // Add extra arguments (from interactive prompt or passthrough)
    if !extra_args.is_empty() {
        cmd.args(&extra_args);
    }

    // Execute and wait
    let status = cmd.status()?;

    if !status.success() {
        if let Some(code) = status.code() {
            std::process::exit(code);
        }
    }

    Ok(())
}

/// Result of interactive options prompt
struct PromptResult {
    args: Vec<String>,
    env_vars: Vec<(String, String)>,
}

/// Prompt user interactively for common options and custom args.
///
/// Options are filtered based on the selected agent type. If no options
/// are available for the agent, the options screen is skipped entirely.
fn prompt_for_options(agent_type: &AgentType) -> anyhow::Result<PromptResult> {
    let theme = ColorfulTheme::default();
    let mut args: Vec<String> = Vec::new();
    let mut env_vars: Vec<(String, String)> = Vec::new();

    // Filter options to only those applicable to this agent type
    let applicable: Vec<&CommonOption> = COMMON_OPTIONS
        .iter()
        .filter(|opt| opt.agents.contains(agent_type))
        .collect();

    // If no options available for this agent, skip the options screen
    if applicable.is_empty() {
        return Ok(PromptResult { args, env_vars });
    }

    // First ask if user wants to configure options or launch directly
    let launch_choice = Select::with_theme(&theme)
        .with_prompt("Launch options")
        .items(&["Launch directly", "Configure options..."])
        .default(0)
        .interact()?;

    // If "Launch directly" selected, return empty result
    if launch_choice == 0 {
        return Ok(PromptResult { args, env_vars });
    }

    // Build selection items with descriptions
    let items: Vec<String> = applicable
        .iter()
        .map(|opt| format!("{} - {}", opt.label, opt.description))
        .collect();

    // Multi-select for applicable options
    let selections = MultiSelect::with_theme(&theme)
        .with_prompt("Select options (Space to toggle, Enter to confirm)")
        .items(&items)
        .interact()?;

    // Process selected options
    for idx in selections {
        match &applicable[idx].action {
            OptionAction::Flag(flag) => {
                // Split flag in case it has a value (e.g., "--output-format json")
                for part in flag.split_whitespace() {
                    args.push(part.to_string());
                }
            }
            OptionAction::EnvVar(key, value) => {
                env_vars.push(((*key).to_string(), (*value).to_string()));
            }
        }
    }

    // Prompt for custom args
    let custom: String = Input::with_theme(&theme)
        .with_prompt("Additional arguments (or press Enter to skip)")
        .allow_empty(true)
        .interact_text()?;

    if !custom.trim().is_empty() {
        // Parse custom args (handle quoted strings properly)
        for arg in shell_words::split(&custom)? {
            args.push(arg);
        }
    }

    Ok(PromptResult { args, env_vars })
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
            // Per https://openrouter.ai/docs/guides/guides/claude-code-integration
            let config = agent
                .openrouter
                .as_ref()
                .ok_or_else(|| anyhow::anyhow!("OpenRouter agent missing 'openrouter' config"))?;

            // Set base URL (must NOT include /v1 suffix)
            let base_url = config
                .base_url
                .clone()
                .unwrap_or_else(|| "https://openrouter.ai/api".to_string());
            env_vars.push(("ANTHROPIC_BASE_URL".to_string(), base_url));

            // Set API key for auth
            if let Some(api_key) = &config.api_key {
                env_vars.push(("ANTHROPIC_AUTH_TOKEN".to_string(), api_key.clone()));
            } else if let Ok(key) = std::env::var("OPENROUTER_API_KEY") {
                env_vars.push(("ANTHROPIC_AUTH_TOKEN".to_string(), key));
            }

            // CRITICAL: Must be explicitly empty to prevent Anthropic auth
            env_vars.push(("ANTHROPIC_API_KEY".to_string(), String::new()));

            // Set model on all tiers for consistency
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

            // Z.AI base URL
            env_vars.push((
                "ANTHROPIC_BASE_URL".to_string(),
                "https://api.z.ai/api/anthropic".to_string(),
            ));

            // Set API key if configured
            if let Some(api_key) = &config.api_key {
                env_vars.push(("ANTHROPIC_AUTH_TOKEN".to_string(), api_key.clone()));
            } else if let Ok(key) = std::env::var("ZAI_API_KEY") {
                env_vars.push(("ANTHROPIC_AUTH_TOKEN".to_string(), key));
            }

            // CRITICAL: Must be explicitly empty to prevent Anthropic auth
            env_vars.push(("ANTHROPIC_API_KEY".to_string(), String::new()));

            // Set model on all tiers for consistency
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

            // Set timeout
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
