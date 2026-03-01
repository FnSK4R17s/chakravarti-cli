//! Gemini CLI provider implementation.

use super::{AgentConfig, AgentOutput, AgentProvider, AgentType};
use anyhow::Result;
use bollard::models::Mount;
use std::path::Path;

/// Google Gemini CLI provider
#[derive(Debug, Default)]
pub struct GeminiProvider;

impl GeminiProvider {
    /// Create a new Gemini provider
    pub fn new() -> Self {
        Self
    }
}

impl AgentProvider for GeminiProvider {
    fn name(&self) -> &str {
        "Gemini CLI"
    }

    fn agent_type(&self) -> AgentType {
        AgentType::Gemini
    }

    fn build_command(&self, prompt: &str, workdir: &Path, config: &AgentConfig) -> Vec<String> {
        let mut cmd = vec!["gemini".to_string()];

        // Add prompt for non-interactive execution
        cmd.push("--prompt".to_string());
        cmd.push(prompt.to_string());

        // Auto-approve tool actions for autonomous execution
        cmd.push("--yolo".to_string());

        // Add model override if specified
        if let Some(ref model) = config.model {
            cmd.push("--model".to_string());
            cmd.push(model.clone());
        }

        // Set working directory
        cmd.push("--cwd".to_string());
        cmd.push(workdir.to_string_lossy().to_string());

        // Disable interactive UI for machine-consumable output paths
        if !config.streaming {
            cmd.push("--quiet".to_string());
        }

        cmd
    }

    fn required_env_vars(&self) -> Vec<&str> {
        vec!["GEMINI_API_KEY"]
    }

    fn config_mounts(&self, host_home: &str, container_home: &str) -> Vec<Mount> {
        let mut mounts = Vec::new();

        // Mount ~/.gemini directory for Gemini credentials and config
        let gemini_dir = format!("{}/.gemini", host_home);
        if std::path::Path::new(&gemini_dir).exists() {
            mounts.push(Mount {
                target: Some(format!("{}/.gemini", container_home)),
                source: Some(gemini_dir),
                typ: Some(bollard::models::MountTypeEnum::BIND),
                read_only: Some(false),
                ..Default::default()
            });
        }

        // Mount Google API config directory if present
        let google_config_dir = format!("{}/.config/google", host_home);
        if std::path::Path::new(&google_config_dir).exists() {
            mounts.push(Mount {
                target: Some(format!("{}/.config/google", container_home)),
                source: Some(google_config_dir),
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
