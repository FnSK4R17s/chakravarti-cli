//! Opencode CLI provider implementation.
//!
//! Opencode is an open-source coding agent CLI.

use super::{AgentConfig, AgentOutput, AgentProvider, AgentType};
use anyhow::Result;
use bollard::models::Mount;
use std::path::Path;

/// Opencode CLI provider
#[derive(Debug, Default)]
pub struct OpencodeProvider;

impl OpencodeProvider {
    /// Create a new Opencode provider
    pub fn new() -> Self {
        Self
    }
}

impl AgentProvider for OpencodeProvider {
    fn name(&self) -> &str {
        "Opencode"
    }

    fn agent_type(&self) -> AgentType {
        AgentType::Opencode
    }

    fn build_command(&self, prompt: &str, workdir: &Path, config: &AgentConfig) -> Vec<String> {
        let mut cmd = vec!["opencode".to_string()];

        // Non-interactive execution with prompt
        cmd.push("run".to_string());
        cmd.push(prompt.to_string());

        // Auto-approve all permissions for autonomous execution
        cmd.push("--auto".to_string());

        // Optional model override
        if let Some(ref model) = config.model {
            cmd.push("--model".to_string());
            cmd.push(model.clone());
        }

        // Set working directory
        cmd.push("--cwd".to_string());
        cmd.push(workdir.to_string_lossy().to_string());

        cmd
    }

    fn required_env_vars(&self) -> Vec<&str> {
        // Opencode typically uses file-based/project-local auth.
        vec![]
    }

    fn config_mounts(&self, host_home: &str, container_home: &str) -> Vec<Mount> {
        let mut mounts = Vec::new();

        // Mount ~/.config/opencode/ for CLI auth/config if present.
        let opencode_config_dir = format!("{}/.config/opencode", host_home);
        if std::path::Path::new(&opencode_config_dir).exists() {
            mounts.push(Mount {
                target: Some(format!("{}/.config/opencode", container_home)),
                source: Some(opencode_config_dir),
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
