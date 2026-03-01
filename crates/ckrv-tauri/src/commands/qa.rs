//! QA commands for Tauri IPC.

// ============================================================
// Imports
// ============================================================

use crate::SharedState;
use ckrv_transport::handlers::qa::{
    get_qa_agent_handler, run_bugs_handler, run_report_handler, run_review_handler, QAAgentInfo,
    QABugsRequest, QAIssue, QAReportRequest, QAReportResponse, QAReviewOutput, QAReviewRequest,
};
use serde::Serialize;
use tauri::State;

// ============================================================
// Types
// ============================================================

/// Response wrapper for QA agent to match frontend expectations.
#[derive(Serialize)]
pub struct QAAgentWrapped {
    /// Configured QA agent, if any.
    agent: Option<QAAgentInfo>,
}

/// Response wrapper for QA review to match frontend expectations.
#[derive(Serialize)]
pub struct QAReviewWrapped {
    /// Whether the review completed successfully.
    success: bool,
    /// Review output if successful.
    review: Option<QAReviewOutput>,
    /// Error message if the review failed.
    error: Option<String>,
}

/// Response wrapper for QA bugs to match frontend expectations.
#[derive(Serialize)]
pub struct QABugsWrapped {
    /// Whether bug analysis completed successfully.
    success: bool,
    /// List of issues found.
    issues: Option<Vec<QAIssue>>,
    /// Error message if analysis failed.
    error: Option<String>,
}

/// Response wrapper for QA report to match frontend expectations.
#[derive(Serialize)]
pub struct QAReportWrapped {
    /// Whether the report was generated successfully.
    success: bool,
    /// Full review output.
    review: Option<QAReviewOutput>,
    /// Generated report content.
    report: Option<String>,
    /// Error message if generation failed.
    error: Option<String>,
}

// ============================================================
// Handlers
// ============================================================

/// Get the configured QA agent.
#[tauri::command]
pub async fn get_qa_agent(state: State<'_, SharedState>) -> Result<QAAgentWrapped, String> {
    let app_state = state.read().await;
    get_qa_agent_handler(&app_state)
        .await
        .map(|agent| QAAgentWrapped { agent })
        .map_err(|e| e.to_string())
}

/// Run QA review.
#[tauri::command]
pub async fn run_review(
    state: State<'_, SharedState>,
    base: String,
) -> Result<QAReviewWrapped, String> {
    let app_state = state.read().await;
    match run_review_handler(&app_state, QAReviewRequest { base }).await {
        Ok(review) => Ok(QAReviewWrapped {
            success: true,
            review: Some(review),
            error: None,
        }),
        Err(e) => Ok(QAReviewWrapped {
            success: false,
            review: None,
            error: Some(e.to_string()),
        }),
    }
}

/// Run bugs analysis.
#[tauri::command]
pub async fn run_bugs(
    state: State<'_, SharedState>,
    base: String,
) -> Result<QABugsWrapped, String> {
    let app_state = state.read().await;
    match run_bugs_handler(&app_state, QABugsRequest { base }).await {
        Ok(issues) => Ok(QABugsWrapped {
            success: true,
            issues: Some(issues),
            error: None,
        }),
        Err(e) => Ok(QABugsWrapped {
            success: false,
            issues: None,
            error: Some(e.to_string()),
        }),
    }
}

/// Generate full QA report.
#[tauri::command]
pub async fn run_report(
    state: State<'_, SharedState>,
    base: String,
    full: Option<bool>,
) -> Result<QAReportWrapped, String> {
    let app_state = state.read().await;
    match run_report_handler(
        &app_state,
        QAReportRequest {
            base,
            full: full.unwrap_or(false),
        },
    )
    .await
    {
        Ok(response) => Ok(QAReportWrapped {
            success: true,
            review: Some(response.review),
            report: response.report,
            error: None,
        }),
        Err(e) => Ok(QAReportWrapped {
            success: false,
            review: None,
            report: None,
            error: Some(e.to_string()),
        }),
    }
}
