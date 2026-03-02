//! # Test Axum Routes
//!
//! Axum route wrappers for test handlers.
//!
//! Routes match frontend expectations:
//! - GET /test/agent - Get test writer agent
//! - POST /test/run - Run tests
//! - POST /test/plan - Create test plan
//! - GET /test/plan-status - Get test plan status
//! - POST /test/write - Write tests
//! - GET /test/write-status - Get test write status
//! - GET /test/coverage - Get coverage
//! - POST /test/fix - Fix failing tests
//! - POST /test/generate - Generate tests

use crate::handlers::test::{
    create_test_plan_handler, fix_tests_handler, generate_tests_handler, get_coverage_handler,
    get_plan_status_handler, get_test_writer_agent_handler, get_write_status_handler,
    run_tests_handler, write_tests_handler, GenerateTestsRequest, RunTestsRequest, TestFixRequest,
    TestPlanRequest, TestWriteRequest,
};
use crate::state::AppState;
use axum::extract::State;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};

/// Get test writer agent.
async fn get_test_agent(State(state): State<AppState>) -> impl IntoResponse {
    match get_test_writer_agent_handler(&state) {
        Ok(agent) => Json(serde_json::json!({ "agent": agent })).into_response(),
        Err(e) => e.into_response(),
    }
}

/// Run tests.
async fn run_tests(
    State(state): State<AppState>,
    Json(request): Json<RunTestsRequest>,
) -> impl IntoResponse {
    match run_tests_handler(&state, request) {
        Ok(result) => {
            // Map to old API format expected by frontend
            let test_result = serde_json::json!({
                "total": result.summary.total,
                "passed": result.summary.passed,
                "failed": result.summary.failed,
                "skipped": result.summary.skipped,
                "duration_ms": result.summary.duration_ms,
                "failures": result.results.iter().filter(|r| matches!(r.status, crate::handlers::test::TestStatus::Failed)).collect::<Vec<_>>(),
                "framework": "unknown"
            });
            Json(serde_json::json!({
                "success": result.summary.failed == 0,
                "result": test_result,
                "error": null
            }))
            .into_response()
        }
        Err(e) => Json(serde_json::json!({
            "success": false,
            "result": null,
            "error": e.to_string()
        }))
        .into_response(),
    }
}

/// Create test plan.
async fn create_test_plan(
    State(state): State<AppState>,
    Json(request): Json<TestPlanRequest>,
) -> impl IntoResponse {
    match create_test_plan_handler(&state, request).await {
        Ok(result) => Json(result).into_response(),
        Err(e) => e.into_response(),
    }
}

/// Get test plan status.
async fn get_plan_status(State(state): State<AppState>) -> impl IntoResponse {
    match get_plan_status_handler(&state) {
        Ok(status) => Json(status).into_response(),
        Err(e) => e.into_response(),
    }
}

/// Write tests.
async fn write_tests(
    State(state): State<AppState>,
    Json(request): Json<TestWriteRequest>,
) -> impl IntoResponse {
    match write_tests_handler(&state, request).await {
        Ok(result) => Json(result).into_response(),
        Err(e) => e.into_response(),
    }
}

/// Get write status.
async fn get_write_status(State(state): State<AppState>) -> impl IntoResponse {
    match get_write_status_handler(&state) {
        Ok(status) => Json(status).into_response(),
        Err(e) => e.into_response(),
    }
}

/// Get coverage.
async fn get_coverage(State(state): State<AppState>) -> impl IntoResponse {
    match get_coverage_handler(&state).await {
        Ok(coverage) => Json(coverage).into_response(),
        Err(e) => e.into_response(),
    }
}

/// Fix failing tests.
async fn fix_tests(
    State(state): State<AppState>,
    Json(request): Json<TestFixRequest>,
) -> impl IntoResponse {
    match fix_tests_handler(&state, request).await {
        Ok(result) => Json(result).into_response(),
        Err(e) => e.into_response(),
    }
}

/// Generate tests.
async fn generate_tests(
    State(state): State<AppState>,
    Json(request): Json<GenerateTestsRequest>,
) -> impl IntoResponse {
    match generate_tests_handler(&state, request).await {
        Ok(result) => Json(result).into_response(),
        Err(e) => e.into_response(),
    }
}

/// Create test routes.
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/test/agent", get(get_test_agent))
        .route("/test/run", post(run_tests))
        .route("/test/plan", post(create_test_plan))
        .route("/test/plan-status", get(get_plan_status))
        .route("/test/write", post(write_tests))
        .route("/test/write-status", get(get_write_status))
        .route("/test/coverage", get(get_coverage))
        .route("/test/fix", post(fix_tests))
        .route("/test/generate", post(generate_tests))
}
