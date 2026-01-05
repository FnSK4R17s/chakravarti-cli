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
    /// Capability level (1-5, where 5 is strongest/most capable)
    /// Used for task-to-agent matching based on complexity
    #[serde(default = "default_level")]
    pub level: u8,
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

fn default_level() -> u8 {
    3 // Default to mid-tier
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
    // Proactively use global config path to avoid storing secrets in repo
    dirs::config_dir()
        .map(|d| d.join("chakravarti").join("agents.yaml"))
        .unwrap_or_else(|| state.project_root.join(".chakravarti").join("agents.yaml"))
}

/// Ensure default agents exist
fn ensure_defaults(agents: &mut AgentsFile) {
    if agents.agents.is_empty() {
        // Add default Claude Code agent
        agents.agents.push(AgentConfig {
            id: "claude-default".to_string(),
            name: "Claude Code".to_string(),
            agent_type: AgentType::Claude,
            level: 5, // Default Claude is strongest
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
#[derive(Debug, Serialize, Clone)]
pub struct OpenRouterModel {
    pub id: String,
    pub name: String,
    pub description: String,
    pub context_length: Option<u32>,
    pub pricing: Option<String>,
}

/// OpenRouter API response structure
#[derive(Debug, Deserialize)]
struct OpenRouterApiResponse {
    data: Vec<OpenRouterApiModel>,
}

#[derive(Debug, Deserialize)]
struct OpenRouterApiModel {
    id: String,
    name: String,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    context_length: Option<u32>,
    #[serde(default)]
    pricing: Option<OpenRouterApiPricing>,
}

#[derive(Debug, Deserialize)]
struct OpenRouterApiPricing {
    prompt: Option<String>,
    completion: Option<String>,
}

/// Format pricing as a human-readable string
fn format_pricing(pricing: &OpenRouterApiPricing) -> String {
    let prompt = pricing.prompt.as_deref().unwrap_or("0");
    let completion = pricing.completion.as_deref().unwrap_or("0");
    
    // Convert from per-token to per-million tokens
    let prompt_f: f64 = prompt.parse().unwrap_or(0.0) * 1_000_000.0;
    let completion_f: f64 = completion.parse().unwrap_or(0.0) * 1_000_000.0;
    
    if prompt_f == 0.0 && completion_f == 0.0 {
        "Free".to_string()
    } else {
        format!("${:.2}/${:.2} per 1M tokens", prompt_f, completion_f)
    }
}

/// Filter to only include models suitable for coding tasks
fn is_coding_model(model: &OpenRouterApiModel) -> bool {
    let id = model.id.to_lowercase();
    
    // Include models from known good providers for coding
    let coding_providers = [
        "anthropic/claude",
        "openai/gpt-4",
        "openai/gpt-3.5",
        "openai/o1",
        "openai/o3",
        "openai/o4",
        "google/gemini",
        "deepseek/",
        "qwen/",
        "meta-llama/",
        "mistralai/",
        "minimax/",
        "moonshot/",
        "x-ai/grok",
        "z-ai/glm",          // Z.AI / Zhipu GLM models
        "zhipu/glm",
        "thudm/glm",         // THUDM GLM models
        "cohere/command",
        "nvidia/",           // NVIDIA models
        "01-ai/",            // Yi models
        "alibaba/",          // Alibaba models
        "bytedance/",        // ByteDance models  
        "amazon/",           // Amazon models
        "ai21/",             // AI21 models
        "inflection/",       // Inflection models
        "perplexity/",       // Perplexity models
        "databricks/",       // Databricks models
    ];
    
    // Exclude models that are primarily for chat/roleplay
    let exclude_patterns = [
        "mythomax",
        "roleplay",
        "remm",
        "weaver",
        "fimbulvetr",
        "noromaid",
        "psyfighter",
        "toppy",
    ];
    
    let is_from_good_provider = coding_providers.iter().any(|p| id.contains(p));
    let is_excluded = exclude_patterns.iter().any(|p| id.contains(p));
    
    is_from_good_provider && !is_excluded
}

/// Get models from OpenRouter API dynamically
pub async fn get_openrouter_models() -> impl IntoResponse {
    // Try to fetch from OpenRouter API
    match fetch_openrouter_models().await {
        Ok(models) => Json(serde_json::json!({
            "success": true,
            "models": models,
            "source": "openrouter_api"
        })),
        Err(e) => {
            eprintln!("Failed to fetch OpenRouter models: {}", e);
            // Return fallback curated list
            let fallback = get_fallback_models();
            Json(serde_json::json!({
                "success": true,
                "models": fallback,
                "source": "fallback",
                "warning": format!("Could not fetch from OpenRouter API: {}", e)
            }))
        }
    }
}

/// Fetch models from OpenRouter API
async fn fetch_openrouter_models() -> Result<Vec<OpenRouterModel>, String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;
    
    let response = client
        .get("https://openrouter.ai/api/v1/models")
        .header("Accept", "application/json")
        .send()
        .await
        .map_err(|e| format!("Failed to fetch models: {}", e))?;
    
    if !response.status().is_success() {
        return Err(format!("OpenRouter API returned status: {}", response.status()));
    }
    
    let api_response: OpenRouterApiResponse = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))?;
    
    // Transform all models (no filtering)
    let models: Vec<OpenRouterModel> = api_response
        .data
        .into_iter()
        .map(|m| {
            let pricing = m.pricing.as_ref().map(format_pricing);
            OpenRouterModel {
                id: m.id.clone(),
                name: m.name.clone(),
                description: m.description.unwrap_or_default(),
                context_length: m.context_length,
                pricing,
            }
        })
        .collect();
    
    // Sort by popularity/relevance (Claude and GPT models first)
    let mut sorted_models = models;
    sorted_models.sort_by(|a, b| {
        let priority = |id: &str| -> i32 {
            if id.contains("anthropic/claude-sonnet-4") { return 0; }
            if id.contains("anthropic/claude-opus-4") { return 1; }
            if id.contains("anthropic/claude") { return 2; }
            if id.contains("openai/gpt-4") { return 3; }
            if id.contains("openai/o3") { return 4; }
            if id.contains("google/gemini") { return 5; }
            if id.contains("deepseek") { return 6; }
            if id.contains("qwen") { return 7; }
            10
        };
        priority(&a.id).cmp(&priority(&b.id))
    });
    
    Ok(sorted_models)
}

/// Fallback curated list if API fails
fn get_fallback_models() -> Vec<OpenRouterModel> {
    vec![
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
            id: "deepseek/deepseek-r1".to_string(),
            name: "DeepSeek R1".to_string(),
            description: "DeepSeek's reasoning model with chain-of-thought".to_string(),
            context_length: Some(64000),
            pricing: Some("$0.55/$2.19 per 1M tokens".to_string()),
        },
        OpenRouterModel {
            id: "openai/gpt-4.1".to_string(),
            name: "GPT-4.1".to_string(),
            description: "OpenAI's GPT-4.1 model".to_string(),
            context_length: Some(1047576),
            pricing: Some("$2/$8 per 1M tokens".to_string()),
        },
    ]
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

