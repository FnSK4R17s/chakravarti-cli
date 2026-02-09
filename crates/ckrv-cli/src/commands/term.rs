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
//! ckrv term           # Interactive agent selection
//! ckrv term --agent claude-default  # Directly spawn specific agent
//! ckrv term --list    # List available agents
//! ```

use crate::services::agent_lookup::{load_agents_config, AgentConfig, AgentType};
use crate::ui::UiContext;
use clap::Args;
use dialoguer::{theme::ColorfulTheme, Select};
use std::process::Command;

#[derive(Args, Debug)]
#[command(
    long_about = "Spawn an interactive AI agent terminal session.\n\n\
                  Quickly launch any configured agent (Claude, OpenRouter, Z.AI, Codex) \
                  with the correct environment variables automatically configured.\n\n\
                  Without arguments, presents an interactive selection menu.",
    after_help = "Examples:\n\
                  # Interactive selection\n\
                  ckrv term\n\n\
                  # Launch specific agent by ID\n\
                  ckrv term --agent my-openrouter-agent\n\n\
                  # List available agents\n\
                  ckrv term --list"
)]
pub struct TermArgs {
    /// Agent ID to spawn directly (skips interactive selection)
    #[arg(short, long)]
    agent: Option<String>,

    /// List available agents and exit
    #[arg(short, long)]
    list: bool,
}

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

    // Build command based on agent type
    let (binary, env_vars) = build_agent_command(agent)?;

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
        println!(); // Blank line before spawning
    }

    // Spawn the agent process
    let mut cmd = Command::new(&binary);

    // Set environment variables
    for (key, value) in &env_vars {
        cmd.env(key, value);
    }

    // Add any extra args from config
    if let Some(extra_args) = &agent.extra_args {
        cmd.args(extra_args);
    }

    // Add any custom env vars from config
    if let Some(custom_env) = &agent.env_vars {
        for (key, value) in custom_env {
            cmd.env(key, value);
        }
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
    }

    Ok((binary, env_vars))
}
