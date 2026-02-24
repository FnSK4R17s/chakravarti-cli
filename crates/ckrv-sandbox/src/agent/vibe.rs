//! Mistral Vibe CLI provider implementation.
//!
//! Mistral Vibe is Mistral AI's first-party coding agent CLI (`vibe`).
//! This provider uses programmatic prompt mode (`--prompt`) with streaming
//! output suitable for autonomous sandbox execution.

use super::{AgentConfig, AgentOutput, AgentProvider, AgentType};
use anyhow::Result;
use bollard::models::Mount;
use std::path::Path;

/// Mistral Vibe CLI provider
#[derive(Debug, Default)]
pub struct MistralVibeProvider;

impl MistralVibeProvider {
    /// Create a new Mistral Vibe provider
    pub fn new() -> Self {
        Self
    }
}

impl AgentProvider for MistralVibeProvider {
    fn name(&self) -> &str {
        "Mistral Vibe"
    }

    fn agent_type(&self) -> AgentType {
        AgentType::MistralVibe
    }

    fn build_command(&self, prompt: &str, workdir: &Path, config: &AgentConfig) -> Vec<String> {
        let mut cmd = vec![
            "vibe".to_string(),
            "--prompt".to_string(),
            prompt.to_string(),
            "--output".to_string(),
            "streaming".to_string(),
            "--workdir".to_string(),
            workdir.to_string_lossy().to_string(),
        ];

        if let Some(max_turns) = config.max_turns {
            cmd.push("--max-turns".to_string());
            cmd.push(max_turns.to_string());
        }

        if let Some(max_price) = config.max_price {
            cmd.push("--max-price".to_string());
            cmd.push(max_price.to_string());
        }

        cmd
    }

    fn required_env_vars(&self) -> Vec<&str> {
        vec!["MISTRAL_API_KEY"]
    }

    fn config_mounts(&self, _host_home: &str, _container_home: &str) -> Vec<Mount> {
        // Vibe uses env-var auth; no credentials directory mount required.
        vec![]
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
