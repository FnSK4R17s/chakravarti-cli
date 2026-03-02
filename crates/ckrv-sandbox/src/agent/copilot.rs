//! GitHub Copilot CLI provider implementation (via GitHub CLI).

use super::{AgentConfig, AgentOutput, AgentProvider, AgentType};
use anyhow::Result;
use bollard::models::Mount;
use std::path::Path;

/// GitHub Copilot provider using `gh copilot`.
#[derive(Debug, Default)]
pub struct GithubCopilotProvider;

impl GithubCopilotProvider {
    /// Create a new GitHub Copilot provider.
    pub fn new() -> Self {
        Self
    }
}

impl AgentProvider for GithubCopilotProvider {
    fn name(&self) -> &str {
        "GitHub Copilot"
    }

    fn agent_type(&self) -> AgentType {
        AgentType::GithubCopilot
    }

    fn build_command(&self, prompt: &str, _workdir: &Path, _config: &AgentConfig) -> Vec<String> {
        vec![
            "gh".to_string(),
            "copilot".to_string(),
            "suggest".to_string(),
            "-t".to_string(),
            "shell".to_string(),
            prompt.to_string(),
        ]
    }

    fn required_env_vars(&self) -> Vec<&str> {
        vec![]
    }

    fn config_mounts(&self, host_home: &str, container_home: &str) -> Vec<Mount> {
        let mut mounts = Vec::new();
        let gh_config = format!("{}/.config/gh", host_home);
        if std::path::Path::new(&gh_config).exists() {
            mounts.push(Mount {
                target: Some(format!("{}/.config/gh", container_home)),
                source: Some(gh_config),
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
