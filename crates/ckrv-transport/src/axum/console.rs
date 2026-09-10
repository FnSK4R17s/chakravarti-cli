//! # Console Axum Routes
//!
//! Axum route wrappers for console handlers.
//!
//! Local command execution uses `spawn_blocking` inside
//! [`execute_command_handler`](crate::handlers::console::execute_command_handler)
//! so blocking `std::process::Command` does not starve the async runtime.
//! The sandbox path stays async.

use crate::handlers::console::{execute_command_handler, ExecuteCommandRequest};
use crate::state::AppState;
use axum::extract::State;
use axum::response::IntoResponse;
use axum::routing::post;
use axum::{Json, Router};

/// Execute a command in the project context.
async fn execute_command(
    State(state): State<AppState>,
    Json(request): Json<ExecuteCommandRequest>,
) -> impl IntoResponse {
    match execute_command_handler(&state, request).await {
        Ok(result) => Json(result).into_response(),
        Err(e) => e.into_response(),
    }
}

/// Create console routes.
pub fn routes() -> Router<AppState> {
    Router::new().route("/console/exec", post(execute_command))
}
