//! # Agent Types
//!
//! Types for agent configuration and management.

use serde::{Deserialize, Serialize};

#[cfg(feature = "typescript")]
use ts_rs::TS;

/// Type of agent execution backend.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(feature = "typescript", derive(TS))]
#[cfg_attr(feature = "typescript", ts(export))]
#[serde(rename_all = "snake_case")]
pub enum AgentType {
    /// Default Claude Code CLI
    Claude,
    /// Claude Code with custom OpenRouter API
    ClaudeOpenRouter,
    /// Claude Code with Z.AI GLM Coding Plan
    ClaudeGlm,
    /// OpenAI Codex CLI
    Codex,
    /// Kilo Code multi-provider CLI
    KiloCode,
    /// Factory Droid CLI
    FactoryDroid,
}

impl Default for AgentType {
    fn default() -> Self {
        Self::Claude
    }
}

/// Configuration for an AI agent.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "typescript", derive(TS))]
#[cfg_attr(feature = "typescript", ts(export))]
pub struct AgentConfig {
    /// Unique identifier for this agent (used by frontend as id)
    pub id: String,

    /// Human-readable display name (used by frontend as name)
    pub name: String,

    /// Agent backend type
    pub agent_type: AgentType,

    /// Capability level 1-5 (5 = strongest)
    #[serde(default = "default_level")]
    pub level: u8,

    /// Model identifier (e.g., "anthropic/claude-3-opus")
    pub model: Option<String>,

    /// Whether this is the default agent
    pub is_default: bool,

    /// Whether this agent is the QA agent
    #[serde(default)]
    pub is_qa_agent: bool,

    /// Whether this agent is the test writer
    #[serde(default)]
    pub is_test_writer: bool,

    /// Whether this agent is enabled
    pub enabled: bool,

    /// Optional description
    pub description: Option<String>,

    /// Agent-specific configuration (OpenRouter config)
    pub openrouter: Option<OpenRouterConfig>,

    /// Agent-specific configuration (GLM config)
    pub glm: Option<GlmConfig>,

    /// Agent-specific configuration (Kilo Code config)
    pub kilo: Option<KiloCodeConfig>,
}

fn default_level() -> u8 {
    3
}

/// OpenRouter-specific configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "typescript", derive(TS))]
#[cfg_attr(feature = "typescript", ts(export))]
pub struct OpenRouterConfig {
    /// API key
    pub api_key: Option<String>,

    /// Model ID on OpenRouter
    pub model: String,

    /// Optional base URL
    pub base_url: Option<String>,

    /// Max tokens
    pub max_tokens: Option<u32>,

    /// Temperature
    pub temperature: Option<f32>,
}

/// GLM-specific configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "typescript", derive(TS))]
#[cfg_attr(feature = "typescript", ts(export))]
pub struct GlmConfig {
    /// API key
    pub api_key: Option<String>,

    /// Model name
    pub model: String,

    /// Timeout in milliseconds
    pub timeout_ms: Option<u32>,
}

/// Kilo Code-specific configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "typescript", derive(TS))]
#[cfg_attr(feature = "typescript", ts(export))]
pub struct KiloCodeConfig {
    /// Model ID in provider/model format (e.g., "google/gemma-3-27b-it:free")
    pub model: String,
}

/// Available model from OpenRouter.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "typescript", derive(TS))]
#[cfg_attr(feature = "typescript", ts(export))]
pub struct OpenRouterModel {
    /// Model ID
    pub id: String,

    /// Model display name
    pub name: String,

    /// Model description
    pub description: Option<String>,

    /// Context window size
    pub context_length: Option<u32>,

    /// Pricing per 1M input tokens
    pub pricing_prompt: Option<String>,

    /// Pricing per 1M output tokens
    pub pricing_completion: Option<String>,
}

/// Available model from Kilo Code CLI.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "typescript", derive(TS))]
#[cfg_attr(feature = "typescript", ts(export))]
pub struct KiloCodeModel {
    /// Full model ID (e.g., "kilo/google/gemma-3-27b-it:free")
    pub id: String,

    /// Provider name (e.g., "google", "meta-llama")
    pub provider: String,

    /// Model name without provider prefix (e.g., "gemma-3-27b-it:free")
    pub name: String,

    /// Whether the model is free
    pub free: bool,
}

/// Available model from Z.AI GLM Coding Plan.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "typescript", derive(TS))]
#[cfg_attr(feature = "typescript", ts(export))]
pub struct GlmModel {
    /// Model ID (e.g., "glm-4.7")
    pub id: String,

    /// Human-readable model name (e.g., "GLM-4.7")
    pub name: String,

    /// Context window size in tokens
    pub context_length: Option<u32>,
}

// ============================================================================
// Request/Response Types
// ============================================================================

/// Request to create or update an agent.
#[derive(Debug, Clone, Deserialize)]
#[cfg_attr(feature = "typescript", derive(TS))]
#[cfg_attr(feature = "typescript", ts(export))]
pub struct UpsertAgentRequest {
    /// Agent configuration to create/update
    pub agent: AgentConfig,
}

/// Request to delete an agent.
#[derive(Debug, Clone, Deserialize)]
#[cfg_attr(feature = "typescript", derive(TS))]
#[cfg_attr(feature = "typescript", ts(export))]
pub struct DeleteAgentRequest {
    /// Name of agent to delete
    pub name: String,
}

/// Request to set the default agent.
#[derive(Debug, Clone, Deserialize)]
#[cfg_attr(feature = "typescript", derive(TS))]
#[cfg_attr(feature = "typescript", ts(export))]
pub struct SetDefaultAgentRequest {
    /// Name of agent to set as default
    pub name: String,
}

/// Request to set the QA agent.
#[derive(Debug, Clone, Deserialize)]
#[cfg_attr(feature = "typescript", derive(TS))]
#[cfg_attr(feature = "typescript", ts(export))]
pub struct SetQaAgentRequest {
    /// Name of agent to set as QA agent
    pub name: String,
}

/// Request to set the test writer agent.
#[derive(Debug, Clone, Deserialize)]
#[cfg_attr(feature = "typescript", derive(TS))]
#[cfg_attr(feature = "typescript", ts(export))]
pub struct SetTestWriterAgentRequest {
    /// Name of agent to set as test writer
    pub name: String,
}

/// Request to test an agent.
#[derive(Debug, Clone, Deserialize)]
#[cfg_attr(feature = "typescript", derive(TS))]
#[cfg_attr(feature = "typescript", ts(export))]
pub struct TestAgentRequest {
    /// Agent configuration to test
    pub agent: AgentConfig,
}

/// Response from testing an agent.
#[derive(Debug, Clone, Serialize)]
#[cfg_attr(feature = "typescript", derive(TS))]
#[cfg_attr(feature = "typescript", ts(export))]
pub struct TestAgentResponse {
    /// Whether the test was successful
    pub success: bool,
    /// Result message
    pub message: String,
}

/// Response from list agents.
pub type ListAgentsResponse = Vec<AgentConfig>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_type_serialization() {
        let agent_type = AgentType::ClaudeOpenRouter;
        let json = serde_json::to_string(&agent_type).expect("serialization failed");
        // snake_case converts ClaudeOpenRouter to claude_open_router
        assert_eq!(json, "\"claude_open_router\"");
    }

    #[test]
    fn test_agent_config_serialization() {
        let config = AgentConfig {
            id: "default".to_string(),
            name: "Default Agent".to_string(),
            agent_type: AgentType::Claude,
            level: 3,
            model: None,
            is_default: true,
            is_qa_agent: false,
            is_test_writer: false,
            enabled: true,
            description: None,
            openrouter: None,
            glm: None,
            kilo: None,
        };

        let json = serde_json::to_string(&config).expect("serialization failed");
        assert!(json.contains("\"agent_type\":\"claude\""));
    }
}
