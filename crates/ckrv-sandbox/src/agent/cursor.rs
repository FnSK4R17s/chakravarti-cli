//! Cursor CLI provider implementation.
//!
//! Cursor is an AI-powered code editor with a built-in CLI agent mode.
//! Uses its own authentication context (session-based via the Cursor app).
//! Config is stored in `~/.cursor/` or `~/.config/cursor/`.

use super::{AgentConfig, AgentOutput, AgentProvider, AgentType};
use anyhow::Result;
use bollard::models::Mount;
use std::path::Path;

/// Cursor CLI provider
///
/// Cursor's CLI agent mode uses `cursor --print [prompt]` for
/// non-interactive execution, similar to Claude Code.
#[derive(Debug, Default)]
pub struct CursorProvider;

impl CursorProvider {
    /// Create a new Cursor provider
    pub fn new() -> Self {
        Self
    }
}

impl AgentProvider for CursorProvider {
    fn name(&self) -> &str {
        "Cursor"
    }

    fn agent_type(&self) -> AgentType {
        AgentType::Cursor
    }

    fn build_command(&self, prompt: &str, workdir: &Path, config: &AgentConfig) -> Vec<String> {
        let mut cmd = vec!["cursor".to_string()];

        // Non-interactive execution with prompt
        cmd.push("--print".to_string());
        cmd.push(prompt.to_string());

        // Add model override if specified
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
        // Cursor uses its own session-based auth, no env vars required
        vec![]
    }

    fn config_mounts(&self, host_home: &str, container_home: &str) -> Vec<Mount> {
        let mut mounts = Vec::new();

        // Mount ~/.cursor/ directory for Cursor credentials and config
        let cursor_dir = format!("{}/.cursor", host_home);
        if std::path::Path::new(&cursor_dir).exists() {
            mounts.push(Mount {
                target: Some(format!("{}/.cursor", container_home)),
                source: Some(cursor_dir),
                typ: Some(bollard::models::MountTypeEnum::BIND),
                read_only: Some(false),
                ..Default::default()
            });
        }

        // Mount ~/.config/cursor/ directory (XDG-compliant location)
        let cursor_config_dir = format!("{}/.config/cursor", host_home);
        if std::path::Path::new(&cursor_config_dir).exists() {
            mounts.push(Mount {
                target: Some(format!("{}/.config/cursor", container_home)),
                source: Some(cursor_config_dir),
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
