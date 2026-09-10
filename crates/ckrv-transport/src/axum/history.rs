//! # History Axum Routes
//!
//! Axum route wrappers for history handlers.
//!
//! Uses `tokio::task::spawn_blocking` since handlers perform synchronous
//! filesystem I/O.
//!
//! Routes match frontend expectations:
//! - GET /history/{spec} - List runs for spec
//! - POST /history/{spec} - Create new run
//! - GET /history/{spec}/{run_id} - Get run details
//! - PATCH /history/{spec}/{run_id} - Update run
//! - DELETE /history/{spec}/{run_id} - Delete run

use crate::error::TransportError;
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
    match tokio::task::spawn_blocking(move || list_history_handler(&state, spec)).await {
        Ok(Ok(runs)) => Json(runs).into_response(),
        Ok(Err(e)) => e.into_response(),
        Err(e) => TransportError::Internal(format!("Task panicked: {e}")).into_response(),
    }
}

/// Get run details.
async fn get_run(
    State(state): State<AppState>,
    Path((spec, run_id)): Path<(String, String)>,
) -> impl IntoResponse {
    match tokio::task::spawn_blocking(move || get_run_handler(&state, spec, run_id)).await {
        Ok(Ok(run)) => Json(run).into_response(),
        Ok(Err(e)) => e.into_response(),
        Err(e) => TransportError::Internal(format!("Task panicked: {e}")).into_response(),
    }
}

/// Create a new run.
async fn create_run(
    State(state): State<AppState>,
    Path(spec): Path<String>,
    Json(request): Json<CreateRunRequest>,
) -> impl IntoResponse {
    match tokio::task::spawn_blocking(move || create_run_handler(&state, spec, request)).await {
        Ok(Ok(run)) => Json(run).into_response(),
        Ok(Err(e)) => e.into_response(),
        Err(e) => TransportError::Internal(format!("Task panicked: {e}")).into_response(),
    }
}

/// Update a run.
async fn update_run(
    State(state): State<AppState>,
    Path((spec, run_id)): Path<(String, String)>,
    Json(request): Json<UpdateRunRequest>,
) -> impl IntoResponse {
    match tokio::task::spawn_blocking(move || update_run_handler(&state, spec, run_id, request))
        .await
    {
        Ok(Ok(run)) => Json(run).into_response(),
        Ok(Err(e)) => e.into_response(),
        Err(e) => TransportError::Internal(format!("Task panicked: {e}")).into_response(),
    }
}

/// Delete a run.
async fn delete_run_route(
    State(state): State<AppState>,
    Path((spec, run_id)): Path<(String, String)>,
) -> impl IntoResponse {
    match tokio::task::spawn_blocking(move || delete_run_handler(&state, spec, run_id)).await {
        Ok(Ok(())) => axum::http::StatusCode::NO_CONTENT.into_response(),
        Ok(Err(e)) => e.into_response(),
        Err(e) => TransportError::Internal(format!("Task panicked: {e}")).into_response(),
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
