//! # Example Axum Routes
//!
//! Reference implementation showing the Axum wrapper pattern.
//!
//! This module demonstrates how to create Axum route wrappers for handlers.
//! Sync handlers are wrapped with `tokio::task::spawn_blocking` to match the
//! production route pattern.

use crate::error::TransportError;
use crate::handlers::example::{example_handler, get_example_info_handler, ExampleRequest};
use crate::state::AppState;
use axum::extract::State;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};

/// Handle POST /example request.
///
/// This wrapper:
/// 1. Extracts `AppState` from Axum's `State`
/// 2. Extracts JSON body into `ExampleRequest`
/// 3. Calls the transport-agnostic handler via `spawn_blocking`
/// 4. Converts the result to an Axum response
async fn example(
    State(state): State<AppState>,
    Json(request): Json<ExampleRequest>,
) -> impl IntoResponse {
    match tokio::task::spawn_blocking(move || example_handler(&state, request)).await {
        Ok(Ok(result)) => Json(result).into_response(),
        Ok(Err(e)) => e.into_response(),
        Err(e) => TransportError::Internal(format!("Task panicked: {e}")).into_response(),
    }
}

/// Handle GET /example request.
///
/// This shows a handler that takes no request body.
async fn get_example_info(State(state): State<AppState>) -> impl IntoResponse {
    match tokio::task::spawn_blocking(move || get_example_info_handler(&state)).await {
        Ok(Ok(result)) => Json(result).into_response(),
        Ok(Err(e)) => e.into_response(),
        Err(e) => TransportError::Internal(format!("Task panicked: {e}")).into_response(),
    }
}

/// Create example routes.
///
/// Returns a router with:
/// - `GET /example` - Get example info
/// - `POST /example` - Process example request
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/example", get(get_example_info))
        .route("/example", post(example))
}
