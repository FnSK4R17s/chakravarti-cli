//! Amp CLI provider implementation.
//!
//! Amp is a coding agent CLI (https://ampcode.com/) with execute mode for
//! non-interactive automation. Auth can be provided via `AMP_API_KEY` or
//! existing settings in `~/.config/amp/settings.json`.

use super::{AgentConfig, AgentOutput, AgentProvider, AgentType};
use anyhow::Result;
use bollard::models::Mount;
use std::path::Path;

/// Amp CLI provider
#[derive(Debug, Default)]
pub struct AmpProvider;

impl AmpProvider {
    /// Create a new Amp provider
    pub fn new() -> Self {
        Self
    }
}

impl AgentProvider for AmpProvider {
    fn name(&self) -> &str {
        "Amp"
    }

    fn agent_type(&self) -> AgentType {
        AgentType::Amp
    }

    fn build_command(&self, prompt: &str, _workdir: &Path, config: &AgentConfig) -> Vec<String> {
        let mut cmd = vec!["amp".to_string()];

        // Non-interactive execution mode with prompt
        cmd.push("--execute".to_string());
        cmd.push(prompt.to_string());

        // Auto-approve all permissions for autonomous execution
        cmd.push("--dangerously-allow-all".to_string());

        // Disable IDE integrations in sandboxed execution
        cmd.push("--no-ide".to_string());
        cmd.push("--no-jetbrains".to_string());

        // Stream JSON for programmatic parsing if requested
        if config.streaming {
            cmd.push("--stream-json".to_string());
        }

        // AgentConfig::model is mapped to Amp mode override for compatibility.
        // Expected values include: deep, rush, smart, free.
        if let Some(ref model_or_mode) = config.model {
            cmd.push("--mode".to_string());
            cmd.push(model_or_mode.clone());
        }

        cmd
    }

    fn required_env_vars(&self) -> Vec<&str> {
        // Amp can authenticate with stored settings (~/.config/amp/settings.json)
        // or with AMP_API_KEY env var.
        vec![]
    }

    fn config_mounts(&self, host_home: &str, container_home: &str) -> Vec<Mount> {
        let mut mounts = Vec::new();

        // Mount ~/.config/amp for settings and auth state
        let amp_config_dir = format!("{}/.config/amp", host_home);
        if std::path::Path::new(&amp_config_dir).exists() {
            mounts.push(Mount {
                target: Some(format!("{}/.config/amp", container_home)),
                source: Some(amp_config_dir),
                typ: Some(bollard::models::MountTypeEnum::BIND),
                read_only: Some(false),
                ..Default::default()
            });
        }

        // Mount ~/.cache/amp for logs/cache used by Amp CLI
        let amp_cache_dir = format!("{}/.cache/amp", host_home);
        if std::path::Path::new(&amp_cache_dir).exists() {
            mounts.push(Mount {
                target: Some(format!("{}/.cache/amp", container_home)),
                source: Some(amp_cache_dir),
                typ: Some(bollard::models::MountTypeEnum::BIND),
                read_only: Some(false),
                ..Default::default()
            });
        }

        mounts
    }

    fn parse_output(&self, stdout: &str, stderr: &str, exit_code: i32) -> Result<AgentOutput> {
        Ok(AgentOutput {
            success: exit_code == 0,
            stdout: stdout.to_string(),
            stderr: stderr.to_string(),
            exit_code,
        })
    }
}
