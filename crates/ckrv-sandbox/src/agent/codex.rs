//! OpenAI Codex CLI provider implementation.

use super::{AgentConfig, AgentOutput, AgentProvider, AgentType};
use anyhow::Result;
use bollard::models::Mount;
use std::path::Path;

/// OpenAI Codex CLI provider
#[derive(Debug, Default)]
pub struct CodexProvider;

impl CodexProvider {
    /// Create a new Codex provider
    pub fn new() -> Self {
        Self
    }
}

impl AgentProvider for CodexProvider {
    fn name(&self) -> &str {
        "OpenAI Codex"
    }

    fn agent_type(&self) -> AgentType {
        AgentType::Codex
    }

    fn build_command(&self, prompt: &str, workdir: &Path, config: &AgentConfig) -> Vec<String> {
        let mut cmd = vec!["codex".to_string()];

        // Add prompt for non-interactive execution
        cmd.push("--print".to_string());
        cmd.push(prompt.to_string());

        // Add full-auto flag to skip all approvals
        cmd.push("--full-auto".to_string());

        // Add model override if specified
        if let Some(ref model) = config.model {
            cmd.push("--model".to_string());
            cmd.push(model.clone());
        }

        // Set working directory
        cmd.push("--cwd".to_string());
        cmd.push(workdir.to_string_lossy().to_string());

        // Add quiet mode for cleaner output
        cmd.push("--quiet".to_string());

        cmd
    }

    fn required_env_vars(&self) -> Vec<&str> {
        vec!["OPENAI_API_KEY"]
    }

    fn config_mounts(&self, host_home: &str, container_home: &str) -> Vec<Mount> {
        let mut mounts = Vec::new();

        // Mount ~/.codex directory for Codex config
        let codex_dir = format!("{}/.codex", host_home);
        if std::path::Path::new(&codex_dir).exists() {
            mounts.push(Mount {
                target: Some(format!("{}/.codex", container_home)),
                source: Some(codex_dir),
                typ: Some(bollard::models::MountTypeEnum::BIND),
                read_only: Some(false),
                ..Default::default()
            });
        }

        // Mount OpenAI config if present
        let openai_config = format!("{}/.config/openai", host_home);
        if std::path::Path::new(&openai_config).exists() {
            mounts.push(Mount {
                target: Some(format!("{}/.config/openai", container_home)),
                source: Some(openai_config),
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
