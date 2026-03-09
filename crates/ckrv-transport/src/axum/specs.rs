//! # Specs Axum Routes
//!
//! Axum route wrappers for spec handlers.
//!
//! Routes match the frontend expectations:
//! - GET /specs - List all specs
//! - GET /specs/detail?name=X - Get spec details
//! - POST /specs/create - Create new spec
//! - POST /specs/save - Save spec
//! - GET /specs/{name}/validate - Validate spec
//! - POST /specs/{name}/design - Generate design
//! - POST /specs/{name}/tasks - Generate tasks
//! - GET /specs/{name}/clarifications - Get clarifications
//! - POST /specs/{name}/clarify - Answer clarifications
//!
//! Handlers that call synchronous filesystem/CLI operations use
//! `tokio::task::spawn_blocking` to avoid blocking the async runtime.

// ============================================================
// IMPORTS
// ============================================================

use crate::error::TransportError;
use crate::handlers::specs::{
    create_spec_handler, get_spec_handler, list_specs_handler, update_spec_handler,
};
use crate::state::AppState;
use crate::types::{CreateSpecRequest, UpdateSpecRequest};
use axum::extract::{Query, State};
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::Deserialize;

// ============================================================
// TYPES
// ============================================================

/// Query params for GET /specs/detail.
#[derive(Deserialize)]
struct SpecDetailQuery {
    name: String,
}

// ============================================================
// HANDLERS
// ============================================================

/// List all specs.
async fn list_specs(State(state): State<AppState>) -> impl IntoResponse {
    match tokio::task::spawn_blocking(move || list_specs_handler(&state)).await {
        Ok(Ok(specs)) => Json(specs).into_response(),
        Ok(Err(e)) => e.into_response(),
        Err(e) => TransportError::Internal(format!("Task panicked: {e}")).into_response(),
    }
}

/// Get spec by name (query param style).
///
/// Returns `{ success, spec, raw_yaml }` to match frontend expectations.
async fn get_spec_detail(
    State(state): State<AppState>,
    Query(query): Query<SpecDetailQuery>,
) -> impl IntoResponse {
    let name = query.name;
    match tokio::task::spawn_blocking(move || get_spec_handler(&state, name)).await {
        Ok(Ok(spec)) => {
            let raw_yaml = spec.raw_yaml.clone();
            Json(serde_json::json!({
                "success": true,
                "spec": spec,
                "raw_yaml": raw_yaml,
            }))
            .into_response()
        }
        Ok(Err(e)) => e.into_response(),
        Err(e) => TransportError::Internal(format!("Task panicked: {e}")).into_response(),
    }
}

/// Create a new spec.
async fn create_spec(
    State(state): State<AppState>,
    Json(request): Json<CreateSpecRequest>,
) -> impl IntoResponse {
    match tokio::task::spawn_blocking(move || create_spec_handler(&state, request)).await {
        Ok(Ok(spec)) => (axum::http::StatusCode::CREATED, Json(spec)).into_response(),
        Ok(Err(e)) => e.into_response(),
        Err(e) => TransportError::Internal(format!("Task panicked: {e}")).into_response(),
    }
}

/// Save spec request (includes name).
#[derive(Deserialize)]
struct SaveSpecRequest {
    name: String,
    raw_yaml: Option<String>,
}

/// Save spec (update with name in body).
async fn save_spec(
    State(state): State<AppState>,
    Json(request): Json<SaveSpecRequest>,
) -> impl IntoResponse {
    let name = request.name;
    let update_request = UpdateSpecRequest {
        raw_yaml: request.raw_yaml,
    };
    match tokio::task::spawn_blocking(move || update_spec_handler(&state, name, update_request))
        .await
    {
        Ok(Ok(spec)) => Json(spec).into_response(),
        Ok(Err(e)) => e.into_response(),
        Err(e) => TransportError::Internal(format!("Task panicked: {e}")).into_response(),
    }
}

/// Validate spec - placeholder.
async fn validate_spec(
    State(_state): State<AppState>,
    axum::extract::Path(name): axum::extract::Path<String>,
) -> impl IntoResponse {
    Json(serde_json::json!({
        "valid": true,
        "spec": name,
        "errors": []
    }))
}

/// Generate design - placeholder.
async fn generate_design(
    State(_state): State<AppState>,
    axum::extract::Path(name): axum::extract::Path<String>,
) -> impl IntoResponse {
    Json(serde_json::json!({
        "status": "ok",
        "spec": name,
        "message": "Design generation started"
    }))
}

/// Generate tasks - placeholder.
async fn generate_tasks(
    State(_state): State<AppState>,
    axum::extract::Path(name): axum::extract::Path<String>,
) -> impl IntoResponse {
    Json(serde_json::json!({
        "status": "ok",
        "spec": name,
        "message": "Task generation started"
    }))
}

/// Get clarifications - placeholder.
async fn get_clarifications(
    State(_state): State<AppState>,
    axum::extract::Path(name): axum::extract::Path<String>,
) -> impl IntoResponse {
    Json(serde_json::json!({
        "spec": name,
        "clarifications": []
    }))
}

/// Answer clarifications - placeholder.
async fn clarify(
    State(_state): State<AppState>,
    axum::extract::Path(name): axum::extract::Path<String>,
) -> impl IntoResponse {
    Json(serde_json::json!({
        "status": "ok",
        "spec": name
    }))
}

// ============================================================
// ROUTES
// ============================================================

/// Create spec routes.
pub fn routes() -> Router<AppState> {
    Router::new()
        // List and create
        .route("/specs", get(list_specs))
        .route("/specs/detail", get(get_spec_detail))
        .route("/specs/create", post(create_spec))
        .route("/specs/save", post(save_spec))
        // Per-spec operations
        .route("/specs/{name}/validate", get(validate_spec))
        .route("/specs/{name}/design", post(generate_design))
        .route("/specs/{name}/tasks", post(generate_tasks))
        .route("/specs/{name}/clarifications", get(get_clarifications))
        .route("/specs/{name}/clarify", post(clarify))
}
