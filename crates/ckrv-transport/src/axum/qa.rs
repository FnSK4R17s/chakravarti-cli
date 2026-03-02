//! # QA Axum Routes
//!
//! Axum route wrappers for QA handlers.

use crate::handlers::qa::{
    get_qa_agent_handler, run_bugs_handler, run_report_handler, run_review_handler, QABugsRequest,
    QAReportRequest, QAReviewRequest,
};
use crate::state::AppState;
use axum::extract::State;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};

/// Get QA agent.
async fn get_qa_agent(State(state): State<AppState>) -> impl IntoResponse {
    match get_qa_agent_handler(&state) {
        Ok(agent) => Json(serde_json::json!({ "agent": agent })).into_response(),
        Err(e) => e.into_response(),
    }
}

/// Run QA review.
async fn qa_review(
    State(state): State<AppState>,
    Json(request): Json<QAReviewRequest>,
) -> impl IntoResponse {
    match run_review_handler(&state, request).await {
        Ok(review) => Json(serde_json::json!({
            "success": true,
            "review": review,
            "error": null
        }))
        .into_response(),
        Err(e) => Json(serde_json::json!({
            "success": false,
            "review": null,
            "error": e.to_string()
        }))
        .into_response(),
    }
}

/// Detect bugs.
async fn qa_bugs(
    State(state): State<AppState>,
    Json(request): Json<QABugsRequest>,
) -> impl IntoResponse {
    match run_bugs_handler(&state, request).await {
        Ok(issues) => Json(serde_json::json!({
            "success": true,
            "issues": issues,
            "error": null
        }))
        .into_response(),
        Err(e) => Json(serde_json::json!({
            "success": false,
            "issues": null,
            "error": e.to_string()
        }))
        .into_response(),
    }
}

/// Generate QA report.
async fn qa_report(
    State(state): State<AppState>,
    Json(request): Json<QAReportRequest>,
) -> impl IntoResponse {
    match run_report_handler(&state, request).await {
        Ok(report) => Json(serde_json::json!({
            "success": true,
            "review": report.review,
            "report": report.report,
            "error": null
        }))
        .into_response(),
        Err(e) => Json(serde_json::json!({
            "success": false,
            "review": null,
            "report": null,
            "error": e.to_string()
        }))
        .into_response(),
    }
}

/// Create QA routes.
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/qa/agent", get(get_qa_agent))
        .route("/qa/review", post(qa_review))
        .route("/qa/bugs", post(qa_bugs))
        .route("/qa/report", post(qa_report))
}
