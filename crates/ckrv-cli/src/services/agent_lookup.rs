//! Agent lookup service - find agents by role.

// ============================================================
// IMPORTS
// ============================================================

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

// ============================================================
// TYPES
// ============================================================

/// Agent type enumeration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum AgentType {
    Claude,
    ClaudeOpenRouter,
    ClaudeGlm,
    Codex,
    KiloCode,
    Cursor,
}

impl Default for AgentType {
    fn default() -> Self {
        Self::Claude
    }
}

/// OpenRouter configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenRouterConfig {
    /// API key for OpenRouter authentication.
    pub api_key: Option<String>,
    /// Model identifier (e.g., "anthropic/claude-3-opus").
    pub model: String,
    /// Custom base URL for OpenRouter API.
    pub base_url: Option<String>,
    /// Maximum tokens for completions.
    pub max_tokens: Option<u32>,
    /// Temperature for response generation.
    pub temperature: Option<f32>,
}

/// GLM configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GLMConfig {
    /// API key for GLM authentication.
    pub api_key: Option<String>,
    /// Model identifier for GLM.
    pub model: String,
    /// Request timeout in milliseconds.
    pub timeout_ms: Option<u32>,
}

/// Agent configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    /// Unique identifier for the agent.
    pub id: String,
    /// Display name for the agent.
    pub name: String,
    /// Type of agent (Claude, Codex, etc.).
    #[serde(default)]
    pub agent_type: AgentType,
    /// Agent capability level (1-5).
    #[serde(default = "default_level")]
    pub level: u8,
    /// Whether this is the default agent for execution.
    #[serde(default)]
    pub is_default: bool,
    /// Whether this agent is used for QA reviews.
    #[serde(default)]
    pub is_qa_agent: bool,
    /// Whether this agent writes tests.
    #[serde(default)]
    pub is_test_writer: bool,
    /// Whether this agent is enabled for use.
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    /// Optional description of the agent.
    pub description: Option<String>,
    /// OpenRouter configuration if using OpenRouter.
    pub openrouter: Option<OpenRouterConfig>,
    /// GLM configuration if using GLM models.
    pub glm: Option<GLMConfig>,
    /// Custom binary path for the agent executable.
    pub binary_path: Option<String>,
    /// Additional arguments to pass to the agent.
    pub extra_args: Option<Vec<String>>,
    /// Environment variables to set for the agent.
    pub env_vars: Option<HashMap<String, String>>,
}

fn default_level() -> u8 {
    3
}

fn default_enabled() -> bool {
    true
}

/// Agents configuration file.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AgentsFile {
    /// List of configured agents.
    pub agents: Vec<AgentConfig>,
}

// ============================================================
// IMPLEMENTATION
// ============================================================

/// Get the path to the agents config file.
pub fn get_agents_path() -> PathBuf {
    dirs::config_dir()
        .map(|d| d.join("chakravarti").join("agents.yaml"))
        .unwrap_or_else(|| PathBuf::from(".chakravarti/agents.yaml"))
}

/// Load agents configuration from file
pub fn load_agents_config() -> anyhow::Result<AgentsFile> {
    let path = get_agents_path();
    if !path.exists() {
        return Ok(AgentsFile::default());
    }
    let content = fs::read_to_string(&path)?;
    let agents: AgentsFile = serde_yaml::from_str(&content)?;
    Ok(agents)
}

/// Find the test writer agent (is_test_writer=true)
pub fn find_test_writer_agent() -> Option<AgentConfig> {
    load_agents_config()
        .ok()?
        .agents
        .into_iter()
        .find(|a| a.is_test_writer && a.enabled)
}

/// Find the QA agent (is_qa_agent=true)
pub fn find_qa_agent() -> Option<AgentConfig> {
    load_agents_config()
        .ok()?
        .agents
        .into_iter()
        .find(|a| a.is_qa_agent && a.enabled)
}

/// Find the default agent
pub fn find_default_agent() -> Option<AgentConfig> {
    load_agents_config()
        .ok()?
        .agents
        .into_iter()
        .find(|a| a.is_default && a.enabled)
}

/// Error message for missing test writer agent
pub fn test_writer_missing_message() -> String {
    format!(
        r#"No test writer agent configured.

To configure a test writer agent:
1. Open the Agent Manager in `ckrv ui`
2. Select an agent and enable "Test Writer" role
3. Or add to ~/.config/chakravarti/agents.yaml:

   agents:
     - id: test-writer
       name: Test Writer
       agent_type: claude
       is_test_writer: true
       enabled: true
"#
    )
}

/// Error message for missing QA agent
pub fn qa_agent_missing_message() -> String {
    format!(
        r#"No QA agent configured.

To configure a QA agent:
1. Open the Agent Manager in `ckrv ui`
2. Select an agent and enable "QA Agent" role
3. Or add to ~/.config/chakravarti/agents.yaml:

   agents:
     - id: qa-agent
       name: QA Agent
       agent_type: claude
       is_qa_agent: true
       enabled: true
"#
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserializes_cursor_agent_type() {
        let yaml = r#"
agents:
  - id: cursor-default
    name: Cursor CLI
    agent_type: cursor
    enabled: true
"#;

        let parsed: AgentsFile = serde_yaml::from_str(yaml).expect("cursor config should parse");
        assert_eq!(parsed.agents.len(), 1);
        assert_eq!(parsed.agents[0].agent_type, AgentType::Cursor);
    }
}
