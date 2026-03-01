//! Factory Droid CLI provider implementation.
//!
//! Factory Droid is Factory AI's autonomous software engineering CLI.
//! Uses file-based auth in `~/.factory/` and optional API key fallback.

use super::{AgentConfig, AgentOutput, AgentProvider, AgentType};
use anyhow::Result;
use bollard::models::Mount;
use std::path::Path;

/// Factory Droid CLI provider
#[derive(Debug, Default)]
pub struct FactoryDroidProvider;

impl FactoryDroidProvider {
    /// Create a new Factory Droid provider
    pub fn new() -> Self {
        Self
    }
}

impl AgentProvider for FactoryDroidProvider {
    fn name(&self) -> &str {
        "Factory Droid"
    }

    fn agent_type(&self) -> AgentType {
        AgentType::FactoryDroid
    }

    fn build_command(&self, prompt: &str, workdir: &Path, config: &AgentConfig) -> Vec<String> {
        let mut cmd = vec!["droid".to_string()];

        // Non-interactive execution with prompt
        cmd.push("run".to_string());
        cmd.push(prompt.to_string());

        // Auto-approve permissions for autonomous execution
        cmd.push("--auto".to_string());

        // Structured output for event parsing in stream mode
        if config.streaming {
            cmd.push("--format".to_string());
            cmd.push("json".to_string());
        }

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
        // Prefer file-based auth via ~/.factory; API key can be provided as fallback.
        vec!["FACTORY_API_KEY"]
    }

    fn config_mounts(&self, host_home: &str, container_home: &str) -> Vec<Mount> {
        let mut mounts = Vec::new();

        // Mount ~/.factory directory for Factory credentials and config
        let factory_dir = format!("{}/.factory", host_home);
        if std::path::Path::new(&factory_dir).exists() {
            mounts.push(Mount {
                target: Some(format!("{}/.factory", container_home)),
                source: Some(factory_dir),
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
