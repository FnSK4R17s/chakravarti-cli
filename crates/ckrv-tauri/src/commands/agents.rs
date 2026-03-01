//! Agent commands for Tauri IPC.

// ============================================================
// Imports
// ============================================================

use crate::SharedState;
use ckrv_transport::handlers::agents::{
    delete_agent_handler, get_glm_models_handler, get_kilo_models_handler,
    get_openrouter_models_handler, list_agents_handler, set_default_agent_handler,
    set_qa_agent_handler, set_test_writer_agent_handler, test_agent_handler,
    upsert_agent_handler,
};
use ckrv_transport::types::{
    AgentConfig, DeleteAgentRequest, GlmModel, KiloCodeModel, OpenRouterModel,
    SetDefaultAgentRequest, SetQaAgentRequest, SetTestWriterAgentRequest, TestAgentRequest,
    TestAgentResponse, UpsertAgentRequest,
};
use serde::Serialize;
use tauri::State;

// ============================================================
// Types
// ============================================================

/// Response wrapper for list_agents to match frontend expectations.
#[derive(Serialize)]
pub struct ListAgentsWrapped {
    /// List of configured agents.
    agents: Vec<AgentConfig>,
}

/// Response wrapper for get_openrouter_models to match frontend expectations.
#[derive(Serialize)]
pub struct ModelsWrapped {
    /// Available OpenRouter models.
    models: Vec<OpenRouterModel>,
}

/// Response wrapper for get_kilo_models to match frontend expectations.
#[derive(Serialize)]
pub struct KiloModelsWrapped {
    /// Available Kilo Code models.
    models: Vec<KiloCodeModel>,
}

/// Response wrapper for get_glm_models to match frontend expectations.
#[derive(Serialize)]
pub struct GlmModelsWrapped {
    /// Available GLM models.
    models: Vec<GlmModel>,
}

// ============================================================
// Handlers
// ============================================================

/// List all configured agents.
#[tauri::command]
pub async fn list_agents(state: State<'_, SharedState>) -> Result<ListAgentsWrapped, String> {
    let app_state = state.read().await;
    list_agents_handler(&app_state)
        .await
        .map(|agents| ListAgentsWrapped { agents })
        .map_err(|e| e.to_string())
}

/// Get available OpenRouter models.
#[tauri::command]
pub async fn get_openrouter_models() -> Result<ModelsWrapped, String> {
    get_openrouter_models_handler()
        .await
        .map(|models| ModelsWrapped { models })
        .map_err(|e| e.to_string())
}

/// Create or update an agent configuration.
#[tauri::command]
pub async fn upsert_agent(
    state: State<'_, SharedState>,
    agent: AgentConfig,
) -> Result<AgentConfig, String> {
    let app_state = state.read().await;
    upsert_agent_handler(&app_state, UpsertAgentRequest { agent })
        .await
        .map_err(|e| e.to_string())
}

/// Delete an agent by ID.
#[tauri::command]
pub async fn delete_agent(state: State<'_, SharedState>, id: String) -> Result<(), String> {
    let app_state = state.read().await;
    delete_agent_handler(&app_state, DeleteAgentRequest { name: id })
        .await
        .map_err(|e| e.to_string())
}

/// Set the default agent.
#[tauri::command]
pub async fn set_default_agent(
    state: State<'_, SharedState>,
    id: String,
) -> Result<AgentConfig, String> {
    let app_state = state.read().await;
    set_default_agent_handler(&app_state, SetDefaultAgentRequest { name: id })
        .await
        .map_err(|e| e.to_string())
}

/// Set the QA agent.
#[tauri::command]
pub async fn set_qa_agent(
    state: State<'_, SharedState>,
    id: String,
) -> Result<AgentConfig, String> {
    let app_state = state.read().await;
    set_qa_agent_handler(&app_state, SetQaAgentRequest { name: id })
        .await
        .map_err(|e| e.to_string())
}

/// Set the test writer agent.
#[tauri::command]
pub async fn set_test_writer_agent(
    state: State<'_, SharedState>,
    id: String,
) -> Result<AgentConfig, String> {
    let app_state = state.read().await;
    set_test_writer_agent_handler(&app_state, SetTestWriterAgentRequest { name: id })
        .await
        .map_err(|e| e.to_string())
}

/// Test an agent configuration.
#[tauri::command]
pub async fn test_agent(agent: AgentConfig) -> Result<TestAgentResponse, String> {
    test_agent_handler(TestAgentRequest { agent })
        .await
        .map_err(|e| e.to_string())
}

/// Get available Kilo Code models.
#[tauri::command]
pub async fn get_kilo_models() -> Result<KiloModelsWrapped, String> {
    get_kilo_models_handler()
        .await
        .map(|models| KiloModelsWrapped { models })
        .map_err(|e| e.to_string())
}

/// Get available GLM models.
#[tauri::command]
pub async fn get_glm_models() -> Result<GlmModelsWrapped, String> {
    get_glm_models_handler()
        .await
        .map(|models| GlmModelsWrapped { models })
        .map_err(|e| e.to_string())
}
