//! # Agents Handler
//!
//! Handlers for agent configuration management.

use crate::error::TransportError;
use crate::state::AppState;
use crate::types::{
    AgentConfig, AgentType, DeleteAgentRequest, GlmConfig, KiloCodeConfig, KiloCodeModel,
    ListAgentsResponse, OpenRouterConfig, OpenRouterModel, SetDefaultAgentRequest,
    SetQaAgentRequest, SetTestWriterAgentRequest, TestAgentRequest, TestAgentResponse,
    UpsertAgentRequest,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

// ============================================================================
// Agent File Types
// ============================================================================

/// Full agent configuration as stored in file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentFileConfig {
    /// Unique identifier
    pub id: String,
    /// Display name
    pub name: String,
    /// Agent type
    #[serde(default)]
    pub agent_type: AgentType,
    /// Capability level (1-5, where 5 is strongest/most capable)
    #[serde(default = "default_level")]
    pub level: u8,
    /// Whether this is the default agent
    #[serde(default)]
    pub is_default: bool,
    /// Whether this is the QA/testing agent
    #[serde(default)]
    pub is_qa_agent: bool,
    /// Whether this is the test writer agent
    #[serde(default)]
    pub is_test_writer: bool,
    /// Whether this agent is enabled
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    /// Description
    pub description: Option<String>,
    /// OpenRouter configuration (for ClaudeOpenRouter type)
    pub openrouter: Option<OpenRouterFileConfig>,
    /// GLM Coding Plan configuration (for ClaudeGlm type)
    pub glm: Option<GlmFileConfig>,
    /// Kilo Code configuration (for KiloCode type)
    pub kilo: Option<KiloCodeFileConfig>,
    /// Custom CLI binary path (if not using default)
    pub binary_path: Option<String>,
    /// Additional CLI arguments
    pub extra_args: Option<Vec<String>>,
    /// Environment variables to set
    pub env_vars: Option<HashMap<String, String>>,
}

/// OpenRouter config as stored in file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenRouterFileConfig {
    pub api_key: Option<String>,
    pub model: String,
    pub base_url: Option<String>,
    pub max_tokens: Option<u32>,
    pub temperature: Option<f32>,
}

/// GLM config as stored in file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlmFileConfig {
    pub api_key: Option<String>,
    pub model: String,
    pub timeout_ms: Option<u32>,
}

/// Kilo Code config as stored in file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KiloCodeFileConfig {
    pub model: String,
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
    pub agents: Vec<AgentFileConfig>,
}

// ============================================================================
// Path Resolution
// ============================================================================

/// Get the path to the agents config file.
fn get_agents_path(state: &AppState) -> PathBuf {
    dirs::config_dir()
        .map(|d| d.join("chakravarti").join("agents.yaml"))
        .unwrap_or_else(|| state.project_root.join(".chakravarti").join("agents.yaml"))
}

/// Ensure default agents exist.
fn ensure_defaults(agents: &mut AgentsFile) {
    if agents.agents.is_empty() {
        agents.agents.push(AgentFileConfig {
            id: "claude-default".to_string(),
            name: "Claude Code".to_string(),
            agent_type: AgentType::Claude,
            level: 5,
            is_default: true,
            is_qa_agent: false,
            is_test_writer: false,
            enabled: true,
            description: Some("Default Claude Code CLI agent".to_string()),
            openrouter: None,
            glm: None,
            kilo: None,
            binary_path: None,
            extra_args: None,
            env_vars: None,
        });
    }
}

/// Load agents from config file.
pub fn load_agents(state: &AppState) -> AgentsFile {
    let path = get_agents_path(state);

    if let Ok(content) = fs::read_to_string(&path) {
        if let Ok(mut agents) = serde_yaml::from_str::<AgentsFile>(&content) {
            ensure_defaults(&mut agents);
            return agents;
        }
    }

    let mut default_agents = AgentsFile::default();
    ensure_defaults(&mut default_agents);
    default_agents
}

/// Save agents to config file.
fn save_agents(state: &AppState, agents: &AgentsFile) -> Result<(), TransportError> {
    let path = get_agents_path(state);

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| TransportError::Internal(format!("Failed to create directory: {e}")))?;
    }

    let yaml = serde_yaml::to_string(agents)
        .map_err(|e| TransportError::Internal(format!("Failed to serialize: {e}")))?;
    fs::write(&path, yaml)
        .map_err(|e| TransportError::Internal(format!("Failed to write file: {e}")))?;

    Ok(())
}

// ============================================================================
// Conversions
// ============================================================================

impl From<AgentFileConfig> for AgentConfig {
    fn from(fc: AgentFileConfig) -> Self {
        let openrouter = fc.openrouter.map(|or| OpenRouterConfig {
            api_key: or.api_key,
            model: or.model,
            base_url: or.base_url,
            max_tokens: or.max_tokens,
            temperature: or.temperature,
        });

        let glm = fc.glm.map(|g| GlmConfig {
            api_key: g.api_key,
            model: g.model,
            timeout_ms: g.timeout_ms,
        });

        let kilo = fc.kilo.map(|k| KiloCodeConfig { model: k.model });

        AgentConfig {
            id: fc.id,
            name: fc.name,
            agent_type: fc.agent_type,
            level: fc.level,
            model: None,
            is_default: fc.is_default,
            is_qa_agent: fc.is_qa_agent,
            is_test_writer: fc.is_test_writer,
            enabled: fc.enabled,
            description: fc.description,
            openrouter,
            glm,
            kilo,
        }
    }
}

// ============================================================================
// Handlers
// ============================================================================

/// List all configured agents.
pub async fn list_agents_handler(state: &AppState) -> Result<ListAgentsResponse, TransportError> {
    let agents = load_agents(state);
    Ok(agents.agents.into_iter().map(Into::into).collect())
}

/// Create or update an agent.
pub async fn upsert_agent_handler(
    state: &AppState,
    request: UpsertAgentRequest,
) -> Result<AgentConfig, TransportError> {
    let mut agents = load_agents(state);

    // If this agent is being set as default, unset others
    if request.agent.is_default {
        for agent in &mut agents.agents {
            agent.is_default = false;
        }
    }

    // Convert to file config
    let file_config = AgentFileConfig {
        id: request.agent.id.clone(),
        name: request.agent.name.clone(),
        agent_type: request.agent.agent_type.clone(),
        level: request.agent.level,
        is_default: request.agent.is_default,
        is_qa_agent: request.agent.is_qa_agent,
        is_test_writer: request.agent.is_test_writer,
        enabled: request.agent.enabled,
        description: request.agent.description.clone(),
        openrouter: request
            .agent
            .openrouter
            .as_ref()
            .map(|or| OpenRouterFileConfig {
                api_key: or.api_key.clone(),
                model: or.model.clone(),
                base_url: or.base_url.clone(),
                max_tokens: or.max_tokens,
                temperature: or.temperature,
            }),
        glm: request.agent.glm.as_ref().map(|g| GlmFileConfig {
            api_key: g.api_key.clone(),
            model: g.model.clone(),
            timeout_ms: g.timeout_ms,
        }),
        kilo: request.agent.kilo.as_ref().map(|k| KiloCodeFileConfig {
            model: k.model.clone(),
        }),
        binary_path: None,
        extra_args: None,
        env_vars: None,
    };

    // Find existing or add new
    if let Some(existing) = agents.agents.iter_mut().find(|a| a.id == file_config.id) {
        *existing = file_config.clone();
    } else {
        agents.agents.push(file_config.clone());
    }

    save_agents(state, &agents)?;
    Ok(file_config.into())
}

/// Delete an agent by name.
pub async fn delete_agent_handler(
    state: &AppState,
    request: DeleteAgentRequest,
) -> Result<(), TransportError> {
    let mut agents = load_agents(state);

    // Don't allow deleting the default agent
    if let Some(agent) = agents.agents.iter().find(|a| a.id == request.name) {
        if agent.is_default {
            return Err(TransportError::BadRequest(
                "Cannot delete the default agent".to_string(),
            ));
        }
    }

    agents.agents.retain(|a| a.id != request.name);
    save_agents(state, &agents)?;

    Ok(())
}

/// Set the default agent.
pub async fn set_default_agent_handler(
    state: &AppState,
    request: SetDefaultAgentRequest,
) -> Result<AgentConfig, TransportError> {
    let mut agents = load_agents(state);

    let mut found = false;
    let mut result_agent = None;

    for agent in &mut agents.agents {
        if agent.id == request.name {
            agent.is_default = true;
            found = true;
            result_agent = Some(agent.clone());
        } else {
            agent.is_default = false;
        }
    }

    if !found {
        return Err(TransportError::NotFound(format!(
            "Agent not found: {}",
            request.name
        )));
    }

    save_agents(state, &agents)?;
    Ok(result_agent.unwrap().into())
}

/// Set the QA agent.
pub async fn set_qa_agent_handler(
    state: &AppState,
    request: SetQaAgentRequest,
) -> Result<AgentConfig, TransportError> {
    let mut agents = load_agents(state);

    let mut found = false;
    let mut result_agent = None;

    // Unset all QA agents, then set the new one
    for agent in &mut agents.agents {
        if agent.id == request.name {
            agent.is_qa_agent = true;
            found = true;
            result_agent = Some(agent.clone());
        } else {
            agent.is_qa_agent = false;
        }
    }

    if !found {
        return Err(TransportError::NotFound(format!(
            "Agent not found: {}",
            request.name
        )));
    }

    save_agents(state, &agents)?;
    Ok(result_agent.unwrap().into())
}

/// Set the test writer agent.
pub async fn set_test_writer_agent_handler(
    state: &AppState,
    request: SetTestWriterAgentRequest,
) -> Result<AgentConfig, TransportError> {
    let mut agents = load_agents(state);

    let mut found = false;
    let mut result_agent = None;

    // Unset all test writer agents, then set the new one
    for agent in &mut agents.agents {
        if agent.id == request.name {
            agent.is_test_writer = true;
            found = true;
            result_agent = Some(agent.clone());
        } else {
            agent.is_test_writer = false;
        }
    }

    if !found {
        return Err(TransportError::NotFound(format!(
            "Agent not found: {}",
            request.name
        )));
    }

    save_agents(state, &agents)?;
    Ok(result_agent.unwrap().into())
}

/// Test an agent configuration.
pub async fn test_agent_handler(
    request: TestAgentRequest,
) -> Result<TestAgentResponse, TransportError> {
    let result = match request.agent.agent_type {
        AgentType::Claude => {
            // Test Claude CLI
            let binary = "claude";
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
            // Test OpenRouter API configuration
            if let Some(ref config) = request.agent.openrouter {
                if config.api_key.is_none()
                    || config
                        .api_key
                        .as_ref()
                        .map(|k| k.is_empty())
                        .unwrap_or(true)
                {
                    Err("OpenRouter API key is required".to_string())
                } else {
                    Ok(format!(
                        "OpenRouter config valid for model: {}",
                        config.model
                    ))
                }
            } else {
                Err("OpenRouter configuration is required".to_string())
            }
        }
        AgentType::ClaudeGlm => {
            // Test GLM Coding Plan API
            if let Some(ref config) = request.agent.glm {
                if config.api_key.is_none()
                    || config
                        .api_key
                        .as_ref()
                        .map(|k| k.is_empty())
                        .unwrap_or(true)
                {
                    Err("Z.AI API key is required for GLM Coding Plan".to_string())
                } else {
                    Ok(format!(
                        "GLM Coding Plan config valid for model: {}",
                        config.model
                    ))
                }
            } else {
                Err("GLM configuration is required".to_string())
            }
        }
        AgentType::Codex => {
            // Test Codex CLI
            let binary = "codex";
            match std::process::Command::new(binary).arg("--version").output() {
                Ok(output) => {
                    if output.status.success() {
                        Ok("Codex CLI available".to_string())
                    } else {
                        Err("Codex CLI not responding correctly".to_string())
                    }
                }
                Err(e) => Err(format!("Codex CLI not found: {}", e)),
            }
        }
        AgentType::KiloCode => {
            // Test Kilo Code CLI
            let binary = "kilo";
            match std::process::Command::new(binary).arg("--version").output() {
                Ok(output) => {
                    if output.status.success() {
                        let version = String::from_utf8_lossy(&output.stdout);
                        Ok(format!("Kilo Code CLI available: {}", version.trim()))
                    } else {
                        Err("Kilo Code CLI not responding correctly".to_string())
                    }
                }
                Err(e) => Err(format!("Kilo Code CLI not found: {}", e)),
            }
        }
    };

    match result {
        Ok(message) => Ok(TestAgentResponse {
            success: true,
            message,
        }),
        Err(message) => Ok(TestAgentResponse {
            success: false,
            message,
        }),
    }
}

// ============================================================================
// Kilo Code Models
// ============================================================================

/// Get models from Kilo Code CLI by running `kilo models`.
pub async fn get_kilo_models_handler() -> Result<Vec<KiloCodeModel>, TransportError> {
    match fetch_kilo_models().await {
        Ok(models) => Ok(models),
        Err(_) => Ok(get_fallback_kilo_models()),
    }
}

/// Fetch models by running `kilo models` command.
async fn fetch_kilo_models() -> Result<Vec<KiloCodeModel>, TransportError> {
    let output = tokio::process::Command::new("kilo")
        .arg("models")
        .output()
        .await
        .map_err(|e| {
            TransportError::ServiceUnavailable(format!("Failed to run 'kilo models': {e}"))
        })?;

    if !output.status.success() {
        return Err(TransportError::ServiceUnavailable(
            "'kilo models' command failed".to_string(),
        ));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let models: Vec<KiloCodeModel> = stdout
        .lines()
        .filter(|line| !line.trim().is_empty())
        .filter_map(|line| {
            let line = line.trim();
            // Format variants:
            //   kilo/provider/model-name:free  (3 segments, e.g. kilo/google/gemma-3-27b-it:free)
            //   kilo/model-name:free           (2 segments, e.g. kilo/corethink:free)
            //   kilo/model-name                (2 segments, no tag, e.g. kilo/giga-potato)
            let stripped = line.strip_prefix("kilo/")?;

            let (provider, model_name) = if let Some(slash_pos) = stripped.find('/') {
                // Has a provider segment: provider/model-name
                (&stripped[..slash_pos], &stripped[slash_pos + 1..])
            } else {
                // No provider segment: treat entire remainder as model name, provider = "kilo"
                ("kilo", stripped)
            };

            let free = model_name.ends_with(":free") || !model_name.contains(':');

            Some(KiloCodeModel {
                id: line.to_string(),
                provider: provider.to_string(),
                name: model_name.to_string(),
                free,
            })
        })
        .collect();

    Ok(models)
}

/// Fallback list if `kilo models` fails.
fn get_fallback_kilo_models() -> Vec<KiloCodeModel> {
    vec![
        KiloCodeModel {
            id: "kilo/deepseek/deepseek-r1-0528:free".to_string(),
            provider: "deepseek".to_string(),
            name: "deepseek-r1-0528:free".to_string(),
            free: true,
        },
        KiloCodeModel {
            id: "kilo/google/gemma-3-27b-it:free".to_string(),
            provider: "google".to_string(),
            name: "gemma-3-27b-it:free".to_string(),
            free: true,
        },
        KiloCodeModel {
            id: "kilo/qwen/qwen3-coder:free".to_string(),
            provider: "qwen".to_string(),
            name: "qwen3-coder:free".to_string(),
            free: true,
        },
    ]
}

// ============================================================================
// OpenRouter API
// ============================================================================

/// OpenRouter API response structure.
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

/// Format pricing as a human-readable string.
fn format_pricing(pricing: &OpenRouterApiPricing) -> String {
    let prompt = pricing.prompt.as_deref().unwrap_or("0");
    let completion = pricing.completion.as_deref().unwrap_or("0");

    let prompt_f: f64 = prompt.parse().unwrap_or(0.0) * 1_000_000.0;
    let completion_f: f64 = completion.parse().unwrap_or(0.0) * 1_000_000.0;

    if prompt_f == 0.0 && completion_f == 0.0 {
        "Free".to_string()
    } else {
        format!("${prompt_f:.2}/${completion_f:.2} per 1M tokens")
    }
}

/// Get models from OpenRouter API.
pub async fn get_openrouter_models_handler() -> Result<Vec<OpenRouterModel>, TransportError> {
    match fetch_openrouter_models().await {
        Ok(models) => Ok(models),
        Err(_) => Ok(get_fallback_models()),
    }
}

/// Fetch models from OpenRouter API.
async fn fetch_openrouter_models() -> Result<Vec<OpenRouterModel>, TransportError> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| TransportError::Internal(format!("Failed to create HTTP client: {e}")))?;

    let response = client
        .get("https://openrouter.ai/api/v1/models")
        .header("Accept", "application/json")
        .send()
        .await
        .map_err(|e| TransportError::ServiceUnavailable(format!("Failed to fetch models: {e}")))?;

    if !response.status().is_success() {
        return Err(TransportError::ServiceUnavailable(format!(
            "OpenRouter API returned status: {}",
            response.status()
        )));
    }

    let api_response: OpenRouterApiResponse = response
        .json()
        .await
        .map_err(|e| TransportError::Internal(format!("Failed to parse response: {e}")))?;

    let mut models: Vec<OpenRouterModel> = api_response
        .data
        .into_iter()
        .map(|m| {
            let pricing_prompt = m.pricing.as_ref().and_then(|p| p.prompt.clone());
            let pricing_completion = m.pricing.as_ref().and_then(|p| p.completion.clone());
            OpenRouterModel {
                id: m.id,
                name: m.name,
                description: m.description,
                context_length: m.context_length,
                pricing_prompt,
                pricing_completion,
            }
        })
        .collect();

    // Sort by popularity
    models.sort_by(|a, b| {
        let priority = |id: &str| -> i32 {
            if id.contains("anthropic/claude-sonnet-4") {
                return 0;
            }
            if id.contains("anthropic/claude-opus-4") {
                return 1;
            }
            if id.contains("anthropic/claude") {
                return 2;
            }
            if id.contains("openai/gpt-4") {
                return 3;
            }
            if id.contains("google/gemini") {
                return 5;
            }
            if id.contains("deepseek") {
                return 6;
            }
            10
        };
        priority(&a.id).cmp(&priority(&b.id))
    });

    Ok(models)
}

/// Fallback curated list if API fails.
fn get_fallback_models() -> Vec<OpenRouterModel> {
    vec![
        OpenRouterModel {
            id: "anthropic/claude-sonnet-4".to_string(),
            name: "Claude Sonnet 4".to_string(),
            description: Some("Anthropic's Claude Sonnet 4 - excellent for coding".to_string()),
            context_length: Some(200000),
            pricing_prompt: Some("$3".to_string()),
            pricing_completion: Some("$15".to_string()),
        },
        OpenRouterModel {
            id: "anthropic/claude-opus-4".to_string(),
            name: "Claude Opus 4".to_string(),
            description: Some("Anthropic's most capable model".to_string()),
            context_length: Some(200000),
            pricing_prompt: Some("$15".to_string()),
            pricing_completion: Some("$75".to_string()),
        },
        OpenRouterModel {
            id: "google/gemini-2.5-pro-preview".to_string(),
            name: "Gemini 2.5 Pro".to_string(),
            description: Some("Google's latest Gemini Pro model".to_string()),
            context_length: Some(1000000),
            pricing_prompt: Some("$1.25".to_string()),
            pricing_completion: Some("$10".to_string()),
        },
        OpenRouterModel {
            id: "deepseek/deepseek-r1".to_string(),
            name: "DeepSeek R1".to_string(),
            description: Some("DeepSeek's reasoning model with chain-of-thought".to_string()),
            context_length: Some(64000),
            pricing_prompt: Some("$0.55".to_string()),
            pricing_completion: Some("$2.19".to_string()),
        },
    ]
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_list_agents_handler() {
        let state = AppState::new(std::path::PathBuf::from("/tmp/test-agents"));
        let result = list_agents_handler(&state).await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_get_fallback_models() {
        let models = get_fallback_models();
        assert!(!models.is_empty());
        assert!(models.iter().any(|m| m.id.contains("claude")));
    }
}
