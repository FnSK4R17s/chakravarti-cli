//! # Plans Axum Routes
//!
//! Axum route wrappers for plan handlers.
//!
//! Routes match frontend expectations:
//! - GET /plans/detail?spec=X - Get plan for spec
//! - POST /plans/save - Save plan
//! - GET /plans/models - Get available models

use crate::handlers::plans::get_plan_handler;
use crate::state::AppState;
use axum::extract::{Query, State};
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::Deserialize;

/// Query params for GET /plans/detail.
#[derive(Deserialize)]
struct PlanDetailQuery {
    spec: String,
}

/// Get plan for a spec.
async fn get_plan(
    State(state): State<AppState>,
    Query(query): Query<PlanDetailQuery>,
) -> impl IntoResponse {
    match get_plan_handler(&state, query.spec) {
        Ok(plan) => Json(plan).into_response(),
        Err(e) => e.into_response(),
    }
}

/// Save plan request.
#[derive(Deserialize)]
#[allow(dead_code)]
struct SavePlanRequest {
    spec: String,
    content: Option<String>,
}

/// Save plan - placeholder.
async fn save_plan(
    State(_state): State<AppState>,
    Json(request): Json<SavePlanRequest>,
) -> impl IntoResponse {
    Json(serde_json::json!({
        "status": "ok",
        "spec": request.spec
    }))
}

/// Get available models for planning.
async fn get_models() -> impl IntoResponse {
    Json(serde_json::json!({
        "models": [
            {"id": "claude-sonnet-4", "name": "Claude Sonnet 4"},
            {"id": "claude-opus-4", "name": "Claude Opus 4"},
            {"id": "gemini-2.5-pro", "name": "Gemini 2.5 Pro"}
        ]
    }))
}

/// Create plan routes.
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/plans/detail", get(get_plan))
        .route("/plans/save", post(save_plan))
        .route("/plans/models", get(get_models))
}
