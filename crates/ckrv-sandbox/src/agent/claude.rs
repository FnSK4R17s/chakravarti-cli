//! Claude Code CLI provider implementation.

use super::{AgentConfig, AgentOutput, AgentProvider, AgentType};
use anyhow::Result;
use bollard::models::Mount;
use std::path::Path;

/// Claude Code CLI provider
#[derive(Debug, Default)]
pub struct ClaudeProvider;

impl ClaudeProvider {
    /// Create a new Claude provider
    pub fn new() -> Self {
        Self
    }
}

impl AgentProvider for ClaudeProvider {
    fn name(&self) -> &str {
        "Claude Code"
    }

    fn agent_type(&self) -> AgentType {
        AgentType::Claude
    }

    fn build_command(&self, prompt: &str, workdir: &Path, config: &AgentConfig) -> Vec<String> {
        let mut cmd = vec!["claude".to_string()];

        // Add print mode for non-interactive execution
        cmd.push("--print".to_string());
        cmd.push(prompt.to_string());

        // Add streaming output format if enabled
        if config.streaming {
            cmd.push("--output-format".to_string());
            cmd.push("stream-json".to_string());
            cmd.push("--verbose".to_string());
        }

        // Skip permission prompts for automated execution
        cmd.push("--dangerously-skip-permissions".to_string());

        // Add model override if specified
        if let Some(ref model) = config.model {
            // Model is set via environment variables for Claude
            // ANTHROPIC_DEFAULT_SONNET_MODEL, etc.
            tracing::debug!(model = %model, "Model override specified (set via env vars)");
        }

        // Set working directory
        cmd.push("--cwd".to_string());
        cmd.push(workdir.to_string_lossy().to_string());

        cmd
    }

    fn required_env_vars(&self) -> Vec<&str> {
        vec![
            "ANTHROPIC_API_KEY",
            // Alternative auth methods
            // "ANTHROPIC_AUTH_TOKEN",
        ]
    }

    fn config_mounts(&self, host_home: &str, container_home: &str) -> Vec<Mount> {
        let mut mounts = Vec::new();

        // Mount ~/.claude.json for Claude config
        let claude_config = format!("{}/.claude.json", host_home);
        if std::path::Path::new(&claude_config).exists() {
            mounts.push(Mount {
                target: Some(format!("{}/.claude.json", container_home)),
                source: Some(claude_config),
                typ: Some(bollard::models::MountTypeEnum::BIND),
                read_only: Some(false),
                ..Default::default()
            });
        }

        // Mount ~/.claude directory for credentials
        let claude_dir = format!("{}/.claude", host_home);
        if std::path::Path::new(&claude_dir).exists() {
            mounts.push(Mount {
                target: Some(format!("{}/.claude", container_home)),
                source: Some(claude_dir),
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
