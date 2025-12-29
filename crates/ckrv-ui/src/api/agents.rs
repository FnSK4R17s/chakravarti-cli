//! Agent management API endpoints

use axum::{extract::State, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use crate::state::AppState;

/// Agent type enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum AgentType {
    /// Default Claude Code CLI
    Claude,
    /// Claude Code with custom OpenRouter API
    ClaudeOpenRouter,
    /// Gemini CLI
    Gemini,
    /// OpenAI Codex CLI
    Codex,
    /// Cursor CLI
    Cursor,
    /// Amp CLI
    Amp,
    /// Qwen Code CLI
    QwenCode,
    /// Opencode CLI
    Opencode,
    /// Factory Droid
    FactoryDroid,
    /// GitHub Copilot (via CLI)
    Copilot,
}

impl Default for AgentType {
    fn default() -> Self {
        Self::Claude
    }
}

/// Model provider for OpenRouter-based agents
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenRouterConfig {
    /// API key for OpenRouter
    pub api_key: Option<String>,
    /// Model identifier (e.g., "moonshot/kimi-k2", "minimax/minimax-m1")
    pub model: String,
    /// Custom base URL (default: https://openrouter.ai/api/v1)
    pub base_url: Option<String>,
    /// Max tokens
    pub max_tokens: Option<u32>,
    /// Temperature
    pub temperature: Option<f32>,
}

/// Agent configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    /// Unique identifier
    pub id: String,
    /// Display name
    pub name: String,
    /// Agent type
    #[serde(default)]
    pub agent_type: AgentType,
    /// Whether this is the default agent
    #[serde(default)]
    pub is_default: bool,
    /// Whether this agent is enabled
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    /// Description
    pub description: Option<String>,
    /// OpenRouter configuration (for ClaudeOpenRouter type)
    pub openrouter: Option<OpenRouterConfig>,
    /// Custom CLI binary path (if not using default)
    pub binary_path: Option<String>,
    /// Additional CLI arguments
    pub extra_args: Option<Vec<String>>,
    /// Environment variables to set
    pub env_vars: Option<HashMap<String, String>>,
}

fn default_enabled() -> bool {
    true
}

/// Agents configuration file
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AgentsFile {
    pub agents: Vec<AgentConfig>,
}

/// Get the path to the agents config file
fn get_agents_path(state: &AppState) -> PathBuf {
    state.project_root.join(".chakravarti").join("agents.yaml")
}

/// Ensure default agents exist
fn ensure_defaults(agents: &mut AgentsFile) {
    if agents.agents.is_empty() {
        // Add default Claude Code agent
        agents.agents.push(AgentConfig {
            id: "claude-default".to_string(),
            name: "Claude Code".to_string(),
            agent_type: AgentType::Claude,
            is_default: true,
            enabled: true,
            description: Some("Default Claude Code CLI agent".to_string()),
            openrouter: None,
            binary_path: None,
            extra_args: None,
            env_vars: None,
        });
    }
}

/// Load agents from config file
fn load_agents(state: &AppState) -> AgentsFile {
    let path = get_agents_path(state);
    
    if let Ok(content) = fs::read_to_string(&path) {
        if let Ok(mut agents) = serde_yaml::from_str::<AgentsFile>(&content) {
            ensure_defaults(&mut agents);
            return agents;
        }
    }
    
    // Return default agents if file doesn't exist or is invalid
    let mut default_agents = AgentsFile::default();
    ensure_defaults(&mut default_agents);
    default_agents
}

/// Save agents to config file
fn save_agents(state: &AppState, agents: &AgentsFile) -> Result<(), String> {
    let path = get_agents_path(state);
    
    // Ensure .chakravarti directory exists
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("Failed to create directory: {}", e))?;
    }
    
    let yaml = serde_yaml::to_string(agents).map_err(|e| format!("Failed to serialize: {}", e))?;
    fs::write(&path, yaml).map_err(|e| format!("Failed to write file: {}", e))?;
    
    Ok(())
}

/// List all agents
pub async fn list_agents(State(state): State<AppState>) -> impl IntoResponse {
    let agents = load_agents(&state);
    Json(serde_json::json!({
        "success": true,
        "agents": agents.agents
    }))
}

/// Get available models from OpenRouter
#[derive(Debug, Serialize)]
pub struct OpenRouterModel {
    pub id: String,
    pub name: String,
    pub description: String,
    pub context_length: Option<u32>,
    pub pricing: Option<String>,
}

/// Get popular models for OpenRouter
pub async fn get_openrouter_models() -> impl IntoResponse {
    // Curated list of popular coding models on OpenRouter
    let models = vec![
        OpenRouterModel {
            id: "anthropic/claude-sonnet-4".to_string(),
            name: "Claude Sonnet 4".to_string(),
            description: "Anthropic's Claude Sonnet 4 - excellent for coding".to_string(),
            context_length: Some(200000),
            pricing: Some("$3/$15 per 1M tokens".to_string()),
        },
        OpenRouterModel {
            id: "anthropic/claude-opus-4".to_string(),
            name: "Claude Opus 4".to_string(),
            description: "Anthropic's most capable model".to_string(),
            context_length: Some(200000),
            pricing: Some("$15/$75 per 1M tokens".to_string()),
        },
        OpenRouterModel {
            id: "google/gemini-2.5-pro-preview".to_string(),
            name: "Gemini 2.5 Pro".to_string(),
            description: "Google's latest Gemini Pro model".to_string(),
            context_length: Some(1000000),
            pricing: Some("$1.25/$10 per 1M tokens".to_string()),
        },
        OpenRouterModel {
            id: "moonshot/kimi-k2".to_string(),
            name: "Kimi K2".to_string(),
            description: "Moonshot's Kimi K2 - strong coding model".to_string(),
            context_length: Some(131072),
            pricing: Some("$0.60/$2.40 per 1M tokens".to_string()),
        },
        OpenRouterModel {
            id: "minimax/minimax-m1".to_string(),
            name: "MiniMax M1".to_string(),
            description: "MiniMax's coding-focused model".to_string(),
            context_length: Some(1000000),
            pricing: Some("$0.40/$1.60 per 1M tokens".to_string()),
        },
        OpenRouterModel {
            id: "deepseek/deepseek-r1".to_string(),
            name: "DeepSeek R1".to_string(),
            description: "DeepSeek's reasoning model with chain-of-thought".to_string(),
            context_length: Some(64000),
            pricing: Some("$0.55/$2.19 per 1M tokens".to_string()),
        },
        OpenRouterModel {
            id: "deepseek/deepseek-chat".to_string(),
            name: "DeepSeek V3".to_string(),
            description: "DeepSeek's latest chat model".to_string(),
            context_length: Some(64000),
            pricing: Some("$0.27/$1.10 per 1M tokens".to_string()),
        },
        OpenRouterModel {
            id: "qwen/qwen3-235b-a22b".to_string(),
            name: "Qwen 3 235B".to_string(),
            description: "Alibaba's Qwen 3 large model".to_string(),
            context_length: Some(131072),
            pricing: Some("$0.14/$0.50 per 1M tokens".to_string()),
        },
        OpenRouterModel {
            id: "zhipu/glm-4-long".to_string(),
            name: "GLM-4 Long".to_string(),
            description: "Zhipu's GLM-4 with long context".to_string(),
            context_length: Some(1000000),
            pricing: Some("$0.14/$0.28 per 1M tokens".to_string()),
        },
        OpenRouterModel {
            id: "openai/gpt-4.1".to_string(),
            name: "GPT-4.1".to_string(),
            description: "OpenAI's GPT-4.1 model".to_string(),
            context_length: Some(1047576),
            pricing: Some("$2/$8 per 1M tokens".to_string()),
        },
        OpenRouterModel {
            id: "openai/o3".to_string(),
            name: "OpenAI o3".to_string(),
            description: "OpenAI's o3 reasoning model".to_string(),
            context_length: Some(200000),
            pricing: Some("$2/$8 per 1M tokens".to_string()),
        },
        OpenRouterModel {
            id: "meta-llama/llama-4-maverick".to_string(),
            name: "Llama 4 Maverick".to_string(),
            description: "Meta's Llama 4 Maverick model".to_string(),
            context_length: Some(1000000),
            pricing: Some("$0.19/$0.49 per 1M tokens".to_string()),
        },
    ];
    
    Json(serde_json::json!({
        "success": true,
        "models": models
    }))
}

/// Create or update an agent
#[derive(Debug, Deserialize)]
pub struct UpsertAgentPayload {
    pub agent: AgentConfig,
}

pub async fn upsert_agent(
    State(state): State<AppState>,
    Json(payload): Json<UpsertAgentPayload>,
) -> impl IntoResponse {
    let mut agents = load_agents(&state);
    
    // If this agent is being set as default, unset others
    if payload.agent.is_default {
        for agent in &mut agents.agents {
            agent.is_default = false;
        }
    }
    
    // Find existing or add new
    if let Some(existing) = agents.agents.iter_mut().find(|a| a.id == payload.agent.id) {
        *existing = payload.agent.clone();
    } else {
        agents.agents.push(payload.agent.clone());
    }
    
    match save_agents(&state, &agents) {
        Ok(()) => Json(serde_json::json!({
            "success": true,
            "agent": payload.agent
        })),
        Err(e) => Json(serde_json::json!({
            "success": false,
            "message": e
        })),
    }
}

/// Delete an agent
#[derive(Debug, Deserialize)]
pub struct DeleteAgentPayload {
    pub id: String,
}

pub async fn delete_agent(
    State(state): State<AppState>,
    Json(payload): Json<DeleteAgentPayload>,
) -> impl IntoResponse {
    let mut agents = load_agents(&state);
    
    // Don't allow deleting the default agent
    if let Some(agent) = agents.agents.iter().find(|a| a.id == payload.id) {
        if agent.is_default {
            return Json(serde_json::json!({
                "success": false,
                "message": "Cannot delete the default agent"
            }));
        }
    }
    
    agents.agents.retain(|a| a.id != payload.id);
    
    match save_agents(&state, &agents) {
        Ok(()) => Json(serde_json::json!({
            "success": true
        })),
        Err(e) => Json(serde_json::json!({
            "success": false,
            "message": e
        })),
    }
}

/// Set default agent
#[derive(Debug, Deserialize)]
pub struct SetDefaultPayload {
    pub id: String,
}

pub async fn set_default_agent(
    State(state): State<AppState>,
    Json(payload): Json<SetDefaultPayload>,
) -> impl IntoResponse {
    let mut agents = load_agents(&state);
    
    // Unset all defaults, then set the new one
    let mut found = false;
    for agent in &mut agents.agents {
        if agent.id == payload.id {
            agent.is_default = true;
            found = true;
        } else {
            agent.is_default = false;
        }
    }
    
    if !found {
        return Json(serde_json::json!({
            "success": false,
            "message": "Agent not found"
        }));
    }
    
    match save_agents(&state, &agents) {
        Ok(()) => Json(serde_json::json!({
            "success": true
        })),
        Err(e) => Json(serde_json::json!({
            "success": false,
            "message": e
        })),
    }
}

/// Test agent connection
#[derive(Debug, Deserialize)]
pub struct TestAgentPayload {
    pub agent: AgentConfig,
}

pub async fn test_agent(Json(payload): Json<TestAgentPayload>) -> impl IntoResponse {
    // Test the agent configuration
    let result = match payload.agent.agent_type {
        AgentType::Claude => {
            // Test Claude CLI
            let binary = payload.agent.binary_path.as_deref().unwrap_or("claude");
            match std::process::Command::new(binary).arg("--version").output() {
                Ok(output) => {
                    if output.status.success() {
                        let version = String::from_utf8_lossy(&output.stdout);
                        Ok(format!("Claude CLI available: {}", version.trim()))
                    } else {
                        Err("Claude CLI not responding correctly".to_string())
                    }
                }
                Err(e) => Err(format!("Claude CLI not found: {}", e)),
            }
        }
        AgentType::ClaudeOpenRouter => {
            // Test OpenRouter API
            if let Some(ref config) = payload.agent.openrouter {
                if config.api_key.is_none() || config.api_key.as_ref().map(|k| k.is_empty()).unwrap_or(true) {
                    Err("OpenRouter API key is required".to_string())
                } else {
                    // For now, just validate the config exists
                    Ok(format!("OpenRouter config valid for model: {}", config.model))
                }
            } else {
                Err("OpenRouter configuration is required".to_string())
            }
        }
        AgentType::Gemini => {
            let binary = payload.agent.binary_path.as_deref().unwrap_or("gemini");
            match std::process::Command::new(binary).arg("--version").output() {
                Ok(_) => Ok("Gemini CLI available".to_string()),
                Err(e) => Err(format!("Gemini CLI not found: {}", e)),
            }
        }
        _ => {
            // Generic test for other agents
            Ok(format!("{:?} agent configured", payload.agent.agent_type))
        }
    };
    
    match result {
        Ok(message) => Json(serde_json::json!({
            "success": true,
            "message": message
        })),
        Err(message) => Json(serde_json::json!({
            "success": false,
            "message": message
        })),
    }
}

