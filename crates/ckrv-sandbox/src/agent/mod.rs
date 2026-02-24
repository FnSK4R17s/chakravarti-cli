//! Agent abstraction module for supporting multiple AI coding assistants.
//!
//! This module provides the `AgentProvider` trait that abstracts the differences
//! between AI agents (Claude Code, OpenAI Codex, etc.) allowing the sandbox
//! to work with any supported agent interchangeably.

mod amp;
mod claude;
mod codex;
mod kilo;
#[cfg(test)]
mod tests;

pub use amp::AmpProvider;
pub use claude::ClaudeProvider;
pub use codex::CodexProvider;
pub use kilo::KiloCodeProvider;

use anyhow::Result;
use bollard::models::Mount;
use std::path::Path;

/// Supported agent types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AgentType {
    /// Claude Code CLI (Anthropic) - default agent
    #[default]
    Claude,
    /// OpenAI Codex CLI
    Codex,
    /// Kilo Code CLI (multi-provider)
    KiloCode,
    /// Amp CLI
    Amp,
}

impl AgentType {
    /// Parse agent type from string
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "claude" | "claude-code" => Some(Self::Claude),
            "codex" | "openai" | "openai-codex" => Some(Self::Codex),
            "kilo" | "kilo-code" | "kilocode" => Some(Self::KiloCode),
            "amp" | "ampcode" => Some(Self::Amp),
            _ => None,
        }
    }

    /// Get the display name for this agent
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Claude => "Claude Code",
            Self::Codex => "OpenAI Codex",
            Self::KiloCode => "Kilo Code",
            Self::Amp => "Amp",
        }
    }
}

impl std::fmt::Display for AgentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

/// Configuration for agent execution
#[derive(Debug, Clone)]
pub struct AgentConfig {
    /// The type of agent to use
    pub agent_type: AgentType,
    /// Optional model override
    pub model: Option<String>,
    /// Whether to use streaming output
    pub streaming: bool,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            agent_type: AgentType::default(),
            model: None,
            streaming: true,
        }
    }
}

impl AgentConfig {
    /// Create a new agent config with specified type
    pub fn new(agent_type: AgentType) -> Self {
        Self {
            agent_type,
            ..Default::default()
        }
    }

    /// Set the model
    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.model = Some(model.into());
        self
    }

    /// Set streaming mode
    pub fn with_streaming(mut self, streaming: bool) -> Self {
        self.streaming = streaming;
        self
    }
}

/// Normalized output from agent execution
#[derive(Debug, Clone, Default)]
pub struct AgentOutput {
    /// Whether execution succeeded
    pub success: bool,
    /// Standard output content
    pub stdout: String,
    /// Standard error content
    pub stderr: String,
    /// Exit code from the agent process
    pub exit_code: i32,
}

/// Trait defining the interface for AI agent implementations.
///
/// Each agent (Claude, Codex, etc.) implements this trait to provide
/// agent-specific command construction, configuration mounting, and
/// output parsing.
pub trait AgentProvider: Send + Sync {
    /// Human-readable name for logging and UI display
    fn name(&self) -> &str;

    /// Get the agent type
    fn agent_type(&self) -> AgentType;

    /// Construct the CLI command for agent execution
    ///
    /// Returns a vector of command and arguments: [command, arg1, arg2, ...]
    fn build_command(&self, prompt: &str, workdir: &Path, config: &AgentConfig) -> Vec<String>;

    /// Environment variables required by this agent
    ///
    /// e.g., ["ANTHROPIC_API_KEY"] or ["OPENAI_API_KEY"]
    fn required_env_vars(&self) -> Vec<&str>;

    /// Docker mounts for agent-specific config files
    ///
    /// e.g., ~/.claude.json or ~/.codex/config.json
    fn config_mounts(&self, host_home: &str, container_home: &str) -> Vec<Mount>;

    /// Parse agent output into normalized format
    fn parse_output(&self, stdout: &str, stderr: &str, exit_code: i32) -> Result<AgentOutput>;
}

/// Create an agent provider for the given type
pub fn create_agent(agent_type: AgentType) -> Box<dyn AgentProvider> {
    match agent_type {
        AgentType::Claude => Box::new(ClaudeProvider::new()),
        AgentType::Codex => Box::new(CodexProvider::new()),
        AgentType::KiloCode => Box::new(KiloCodeProvider::new()),
        AgentType::Amp => Box::new(AmpProvider::new()),
    }
}

/// Get the default agent provider
pub fn default_agent() -> Box<dyn AgentProvider> {
    create_agent(AgentType::default())
}
