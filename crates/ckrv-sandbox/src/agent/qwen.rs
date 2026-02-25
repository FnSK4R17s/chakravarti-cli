//! Qwen Code provider implementation.
//!
//! Supports both:
//! - Native Qwen CLI mode (`qwen` command)
//! - OpenAI-compatible API mode via `qwen chat` flags

use super::{AgentConfig, AgentOutput, AgentProvider, AgentType};
use anyhow::Result;
use bollard::models::Mount;
use std::path::Path;

/// Qwen Code provider
#[derive(Debug, Default)]
pub struct QwenProvider;

impl QwenProvider {
    /// Create a new Qwen provider
    pub fn new() -> Self {
        Self
    }
}

impl AgentProvider for QwenProvider {
    fn name(&self) -> &str {
        "qwen-code"
    }

    fn agent_type(&self) -> AgentType {
        AgentType::Qwen
    }

    fn build_command(&self, prompt: &str, workdir: &Path, config: &AgentConfig) -> Vec<String> {
        let mut cmd = vec!["qwen".to_string()];

        // Use non-interactive mode for automated execution.
        cmd.push("--yes".to_string());
        cmd.push("--approval-mode=auto".to_string());

        // API mode uses OpenAI-compatible endpoint/model options.
        if config.use_api {
            if let Some(ref model) = config.model {
                cmd.push("--model".to_string());
                cmd.push(model.clone());
            } else {
                cmd.push("--model".to_string());
                cmd.push("qwen/qwen3-coder".to_string());
            }

            if let Some(ref base_url) = config.api_base_url {
                cmd.push("--base-url".to_string());
                cmd.push(base_url.clone());
            }
        } else if let Some(ref model) = config.model {
            // CLI mode still allows explicit model selection.
            cmd.push("--model".to_string());
            cmd.push(model.clone());
        }

        // Set working directory.
        cmd.push("--cwd".to_string());
        cmd.push(workdir.to_string_lossy().to_string());

        // Prompt should be the final positional argument.
        cmd.push(prompt.to_string());

        cmd
    }

    fn required_env_vars(&self) -> Vec<&str> {
        vec!["OPENAI_API_KEY", "QWEN_AUTH_TOKEN", "OPENAI_BASE_URL"]
    }

    fn config_mounts(&self, host_home: &str, container_home: &str) -> Vec<Mount> {
        let mut mounts = Vec::new();

        // Mount ~/.qwen directory for Qwen credentials and config.
        let qwen_dir = format!("{}/.qwen", host_home);
        if std::path::Path::new(&qwen_dir).exists() {
            mounts.push(Mount {
                target: Some(format!("{}/.qwen", container_home)),
                source: Some(qwen_dir),
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
