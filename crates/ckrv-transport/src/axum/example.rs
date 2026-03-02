//! # Example Axum Routes
//!
//! Reference implementation showing the Axum wrapper pattern.
//!
//! This module demonstrates how to create Axum route wrappers for handlers.

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
/// 3. Calls the transport-agnostic handler
/// 4. Converts the result to an Axum response
async fn example(
    State(state): State<AppState>,
    Json(request): Json<ExampleRequest>,
) -> impl IntoResponse {
    match example_handler(&state, request) {
        Ok(result) => Json(result).into_response(),
        Err(e) => e.into_response(),
    }
}

/// Handle GET /example request.
///
/// This shows a handler that takes no request body.
async fn get_example_info(State(state): State<AppState>) -> impl IntoResponse {
    match get_example_info_handler(&state) {
        Ok(result) => Json(result).into_response(),
        Err(e) => e.into_response(),
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
