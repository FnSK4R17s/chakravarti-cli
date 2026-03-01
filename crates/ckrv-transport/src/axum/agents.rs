//! # Agents Axum Routes
//!
//! Axum route wrappers for agent handlers.

use crate::handlers::agents::{
    delete_agent_handler, get_glm_models_handler, get_kilo_models_handler,
    get_openrouter_models_handler, list_agents_handler, set_default_agent_handler,
    set_qa_agent_handler, set_test_writer_agent_handler, test_agent_handler, upsert_agent_handler,
};
use crate::state::AppState;
use crate::types::{
    DeleteAgentRequest, SetDefaultAgentRequest, SetQaAgentRequest, SetTestWriterAgentRequest,
    TestAgentRequest, UpsertAgentRequest,
};
use axum::extract::State;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::Deserialize;

/// List all agents.
async fn list_agents(State(state): State<AppState>) -> impl IntoResponse {
    match list_agents_handler(&state).await {
        Ok(agents) => Json(serde_json::json!({ "agents": agents })).into_response(),
        Err(e) => e.into_response(),
    }
}

/// Create or update an agent.
async fn upsert_agent(
    State(state): State<AppState>,
    Json(request): Json<UpsertAgentRequest>,
) -> impl IntoResponse {
    match upsert_agent_handler(&state, request).await {
        Ok(agent) => Json(agent).into_response(),
        Err(e) => e.into_response(),
    }
}

/// Delete request body for POST /agents/delete.
#[derive(Deserialize)]
struct DeleteAgentBody {
    name: String,
}

/// Delete an agent.
async fn delete_agent(
    State(state): State<AppState>,
    Json(body): Json<DeleteAgentBody>,
) -> impl IntoResponse {
    let request = DeleteAgentRequest { name: body.name };
    match delete_agent_handler(&state, request).await {
        Ok(()) => axum::http::StatusCode::NO_CONTENT.into_response(),
        Err(e) => e.into_response(),
    }
}

/// Set default agent request body.
#[derive(Deserialize)]
struct SetDefaultBody {
    name: String,
}

/// Set default agent.
async fn set_default_agent(
    State(state): State<AppState>,
    Json(body): Json<SetDefaultBody>,
) -> impl IntoResponse {
    let request = SetDefaultAgentRequest { name: body.name };
    match set_default_agent_handler(&state, request).await {
        Ok(agent) => Json(agent).into_response(),
        Err(e) => e.into_response(),
    }
}

/// Set QA agent request body.
#[derive(Deserialize)]
struct SetQaBody {
    name: String,
}

/// Set QA agent.
async fn set_qa_agent(
    State(state): State<AppState>,
    Json(body): Json<SetQaBody>,
) -> impl IntoResponse {
    let request = SetQaAgentRequest { name: body.name };
    match set_qa_agent_handler(&state, request).await {
        Ok(agent) => Json(serde_json::json!({ "success": true, "agent": agent })).into_response(),
        Err(e) => e.into_response(),
    }
}

/// Set test writer agent request body.
#[derive(Deserialize)]
struct SetTestWriterBody {
    name: String,
}

/// Set test writer agent.
async fn set_test_writer_agent(
    State(state): State<AppState>,
    Json(body): Json<SetTestWriterBody>,
) -> impl IntoResponse {
    let request = SetTestWriterAgentRequest { name: body.name };
    match set_test_writer_agent_handler(&state, request).await {
        Ok(agent) => Json(serde_json::json!({ "success": true, "agent": agent })).into_response(),
        Err(e) => e.into_response(),
    }
}

/// Test an agent.
async fn test_agent(
    State(_state): State<AppState>,
    Json(request): Json<TestAgentRequest>,
) -> impl IntoResponse {
    match test_agent_handler(request).await {
        Ok(response) => Json(response).into_response(),
        Err(e) => e.into_response(),
    }
}

/// Get available OpenRouter models.
async fn get_models() -> impl IntoResponse {
    match get_openrouter_models_handler().await {
        Ok(models) => Json(serde_json::json!({ "models": models })).into_response(),
        Err(e) => e.into_response(),
    }
}

/// Get available Kilo Code models.
async fn get_kilo_models() -> impl IntoResponse {
    match get_kilo_models_handler().await {
        Ok(models) => Json(serde_json::json!({ "models": models })).into_response(),
        Err(e) => e.into_response(),
    }
}

/// Get available GLM Coding Plan models.
async fn get_glm_models() -> impl IntoResponse {
    match get_glm_models_handler().await {
        Ok(models) => Json(serde_json::json!({ "models": models })).into_response(),
        Err(e) => e.into_response(),
    }
}

/// Create agent routes.
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/agents", get(list_agents))
        .route("/agents/models", get(get_models))
        .route("/agents/kilo-models", get(get_kilo_models))
        .route("/agents/glm-models", get(get_glm_models))
        .route("/agents/upsert", post(upsert_agent))
        .route("/agents/delete", post(delete_agent))
        .route("/agents/set-default", post(set_default_agent))
        .route("/agents/set-qa", post(set_qa_agent))
        .route("/agents/set-test-writer", post(set_test_writer_agent))
        .route("/agents/test", post(test_agent))
}
