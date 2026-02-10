//! Kilo Code CLI provider implementation.
//!
//! Kilo Code is an open-source, multi-provider agentic CLI that supports
//! 30+ AI providers through a single interface. Uses file-based auth
//! stored in `~/.config/kilo/` (XDG-compliant).

use super::{AgentConfig, AgentOutput, AgentProvider, AgentType};
use anyhow::Result;
use bollard::models::Mount;
use std::path::Path;

/// Kilo Code CLI provider
///
/// Supports 30+ AI backends (Gemini, DeepSeek, Mistral, Qwen, etc.)
/// through a single integration. Uses `kilo run [prompt] --auto` for
/// non-interactive execution.
#[derive(Debug, Default)]
pub struct KiloCodeProvider;

impl KiloCodeProvider {
    /// Create a new Kilo Code provider
    pub fn new() -> Self {
        Self
    }
}

impl AgentProvider for KiloCodeProvider {
    fn name(&self) -> &str {
        "Kilo Code"
    }

    fn agent_type(&self) -> AgentType {
        AgentType::KiloCode
    }

    fn build_command(&self, prompt: &str, workdir: &Path, config: &AgentConfig) -> Vec<String> {
        let mut cmd = vec!["kilo".to_string()];

        // Non-interactive execution with prompt
        cmd.push("run".to_string());
        cmd.push(prompt.to_string());

        // Auto-approve all permissions for autonomous execution
        cmd.push("--auto".to_string());

        // Add streaming output format if enabled (NDJSON events)
        if config.streaming {
            cmd.push("--format".to_string());
            cmd.push("json".to_string());
        }

        // Add model override if specified (format: provider/model)
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
        // Kilo Code uses file-based auth (~/.config/kilo/config.json)
        // configured via `kilo auth`. No environment variables required.
        vec![]
    }

    fn config_mounts(&self, host_home: &str, container_home: &str) -> Vec<Mount> {
        let mut mounts = Vec::new();

        // Mount ~/.config/kilo/ directory for Kilo credentials and config
        let kilo_config_dir = format!("{}/.config/kilo", host_home);
        if std::path::Path::new(&kilo_config_dir).exists() {
            mounts.push(Mount {
                target: Some(format!("{}/.config/kilo", container_home)),
                source: Some(kilo_config_dir),
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
