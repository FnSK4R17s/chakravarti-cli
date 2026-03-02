//! # Diff Axum Routes
//!
//! Axum route wrappers for diff handlers.
//!
//! Routes match frontend expectations:
//! - GET /diff - Get diff
//! - GET /diff/branches - Get branches for diff

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
    match get_diff_handler(&state, query) {
        Ok(diff) => Json(diff).into_response(),
        Err(e) => e.into_response(),
    }
}

/// Get branches for diff.
async fn get_branches(State(state): State<AppState>) -> impl IntoResponse {
    match get_branches_handler(&state) {
        Ok(branches) => Json(branches).into_response(),
        Err(e) => e.into_response(),
    }
}

/// Create diff routes.
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/diff", get(get_diff))
        .route("/diff/branches", get(get_branches))
}
