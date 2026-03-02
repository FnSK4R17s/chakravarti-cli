//! # Commands Axum Routes
//!
//! Axum route wrappers for command handlers.

use crate::handlers::commands::{
    run_diff_handler, run_execute_handler, run_fix_handler, run_git_init_handler, run_init_handler,
    run_plan_handler, run_promote_handler, run_spec_new_handler, run_spec_tasks_handler,
    run_verify_handler, DiffRequest, FixRequest, PromoteRequest, SpecNewRequest, VerifyRequest,
};
use crate::state::AppState;
use axum::extract::State;
use axum::response::IntoResponse;
use axum::routing::post;
use axum::{Json, Router};

/// Run init command.
async fn run_init(State(state): State<AppState>) -> impl IntoResponse {
    match run_init_handler(&state) {
        Ok(result) => Json(result).into_response(),
        Err(e) => e.into_response(),
    }
}

/// Run git init command.
async fn run_git_init(State(state): State<AppState>) -> impl IntoResponse {
    match run_git_init_handler(&state) {
        Ok(result) => Json(result).into_response(),
        Err(e) => e.into_response(),
    }
}

/// Run spec new command.
async fn run_spec_new(
    State(state): State<AppState>,
    Json(request): Json<SpecNewRequest>,
) -> impl IntoResponse {
    match run_spec_new_handler(&state, request) {
        Ok(result) => Json(result).into_response(),
        Err(e) => e.into_response(),
    }
}

/// Run spec tasks command.
async fn run_spec_tasks(State(state): State<AppState>) -> impl IntoResponse {
    match run_spec_tasks_handler(&state) {
        Ok(result) => Json(result).into_response(),
        Err(e) => e.into_response(),
    }
}

/// Run plan command.
async fn run_plan(State(state): State<AppState>) -> impl IntoResponse {
    match run_plan_handler(&state) {
        Ok(result) => Json(result).into_response(),
        Err(e) => e.into_response(),
    }
}

/// Run execute command.
async fn run_execute(State(state): State<AppState>) -> impl IntoResponse {
    match run_execute_handler(&state) {
        Ok(result) => Json(result).into_response(),
        Err(e) => e.into_response(),
    }
}

/// Run diff command.
async fn run_diff(
    State(state): State<AppState>,
    Json(request): Json<DiffRequest>,
) -> impl IntoResponse {
    match run_diff_handler(&state, request) {
        Ok(result) => Json(result).into_response(),
        Err(e) => e.into_response(),
    }
}

/// Run verify command.
async fn run_verify(
    State(state): State<AppState>,
    Json(request): Json<VerifyRequest>,
) -> impl IntoResponse {
    match run_verify_handler(&state, request) {
        Ok(result) => Json(result).into_response(),
        Err(e) => e.into_response(),
    }
}

/// Run promote command.
async fn run_promote(
    State(state): State<AppState>,
    Json(request): Json<PromoteRequest>,
) -> impl IntoResponse {
    match run_promote_handler(&state, request) {
        Ok(result) => Json(result).into_response(),
        Err(e) => e.into_response(),
    }
}

/// Run fix command.
async fn run_fix(
    State(state): State<AppState>,
    Json(request): Json<FixRequest>,
) -> impl IntoResponse {
    match run_fix_handler(&state, request) {
        Ok(result) => Json(result).into_response(),
        Err(e) => e.into_response(),
    }
}

/// Create command routes.
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/command/init", post(run_init))
        .route("/command/git-init", post(run_git_init))
        .route("/command/spec-new", post(run_spec_new))
        .route("/command/spec-tasks", post(run_spec_tasks))
        .route("/command/plan", post(run_plan))
        .route("/command/execute", post(run_execute))
        .route("/command/diff", post(run_diff))
        .route("/command/verify", post(run_verify))
        .route("/command/promote", post(run_promote))
        .route("/command/fix", post(run_fix))
}
