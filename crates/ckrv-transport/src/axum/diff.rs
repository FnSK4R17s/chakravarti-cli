//! # Diff Axum Routes
//!
//! Axum route wrappers for diff handlers.
//!
//! Uses `tokio::task::spawn_blocking` since handlers perform synchronous
//! Git CLI operations.
//!
//! Routes match frontend expectations:
//! - GET /diff - Get diff
//! - GET /diff/branches - Get branches for diff

use crate::error::TransportError;
use crate::handlers::diff::{get_branches_handler, get_diff_handler, DiffQuery};
use crate::state::AppState;
use axum::extract::{Query, State};
use axum::response::IntoResponse;
use axum::routing::get;
use axum::{Json, Router};

/// Get diff.
async fn get_diff(
    State(state): State<AppState>,
    Query(query): Query<DiffQuery>,
) -> impl IntoResponse {
    match tokio::task::spawn_blocking(move || get_diff_handler(&state, query)).await {
        Ok(Ok(diff)) => Json(diff).into_response(),
        Ok(Err(e)) => e.into_response(),
        Err(e) => TransportError::Internal(format!("Task panicked: {e}")).into_response(),
    }
}

/// Get branches for diff.
async fn get_branches(State(state): State<AppState>) -> impl IntoResponse {
    match tokio::task::spawn_blocking(move || get_branches_handler(&state)).await {
        Ok(Ok(branches)) => Json(branches).into_response(),
        Ok(Err(e)) => e.into_response(),
        Err(e) => TransportError::Internal(format!("Task panicked: {e}")).into_response(),
    }
}

/// Create diff routes.
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/diff", get(get_diff))
        .route("/diff/branches", get(get_branches))
}
