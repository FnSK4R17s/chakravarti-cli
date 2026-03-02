//! # History Axum Routes
//!
//! Axum route wrappers for history handlers.
//!
//! Routes match frontend expectations:
//! - GET /history/{spec} - List runs for spec
//! - POST /history/{spec} - Create new run
//! - GET /history/{spec}/{run_id} - Get run details
//! - PATCH /history/{spec}/{run_id} - Update run
//! - DELETE /history/{spec}/{run_id} - Delete run

use crate::handlers::history::{
    create_run_handler, delete_run_handler, get_run_handler, list_history_handler,
    update_run_handler,
};
use crate::state::AppState;
use crate::types::{CreateRunRequest, UpdateRunRequest};
use axum::extract::{Path, State};
use axum::response::IntoResponse;
use axum::routing::get;
use axum::{Json, Router};

/// List execution history for a spec.
async fn list_history(
    State(state): State<AppState>,
    Path(spec): Path<String>,
) -> impl IntoResponse {
    match list_history_handler(&state, spec) {
        Ok(runs) => Json(runs).into_response(),
        Err(e) => e.into_response(),
    }
}

/// Get run details.
async fn get_run(
    State(state): State<AppState>,
    Path((spec, run_id)): Path<(String, String)>,
) -> impl IntoResponse {
    match get_run_handler(&state, spec, run_id) {
        Ok(run) => Json(run).into_response(),
        Err(e) => e.into_response(),
    }
}

/// Create a new run.
async fn create_run(
    State(state): State<AppState>,
    Path(spec): Path<String>,
    Json(request): Json<CreateRunRequest>,
) -> impl IntoResponse {
    match create_run_handler(&state, spec, request) {
        Ok(run) => Json(run).into_response(),
        Err(e) => e.into_response(),
    }
}

/// Update a run.
async fn update_run(
    State(state): State<AppState>,
    Path((spec, run_id)): Path<(String, String)>,
    Json(request): Json<UpdateRunRequest>,
) -> impl IntoResponse {
    match update_run_handler(&state, spec, run_id, request) {
        Ok(run) => Json(run).into_response(),
        Err(e) => e.into_response(),
    }
}

/// Delete a run.
async fn delete_run_route(
    State(state): State<AppState>,
    Path((spec, run_id)): Path<(String, String)>,
) -> impl IntoResponse {
    match delete_run_handler(&state, spec, run_id) {
        Ok(()) => axum::http::StatusCode::NO_CONTENT.into_response(),
        Err(e) => e.into_response(),
    }
}

/// Create history routes.
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/history/{spec}", get(list_history).post(create_run))
        .route(
            "/history/{spec}/{run_id}",
            get(get_run).patch(update_run).delete(delete_run_route),
        )
}
