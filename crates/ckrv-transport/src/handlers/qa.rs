//! # QA Handler
//!
//! Handlers for QA (code review) operations.

use crate::error::TransportError;
use crate::handlers::agents::load_agents;
use crate::state::AppState;
use serde::{Deserialize, Serialize};

// ============================================================================
// Request/Response Types
// ============================================================================

/// QA review request.
#[derive(Debug, Deserialize)]
pub struct QAReviewRequest {
    /// Base branch for comparison.
    pub base: String,
}

/// QA issue.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QAIssue {
    /// Unique issue identifier.
    pub id: String,
    /// File where the issue was found.
    pub file: String,
    /// Line number of the issue.
    pub line: Option<u32>,
    /// Severity level (critical, major, minor, info).
    pub severity: String,
    /// Issue category (code_quality, security, performance).
    pub category: String,
    /// Description of the issue.
    pub message: String,
    /// Suggested fix for the issue.
    pub suggestion: Option<String>,
}

/// QA summary.
#[derive(Debug, Serialize)]
pub struct QASummary {
    /// Total number of issues found.
    pub total_issues: u32,
    /// Number of critical severity issues.
    pub critical: u32,
    /// Number of major severity issues.
    pub major: u32,
    /// Number of minor severity issues.
    pub minor: u32,
    /// Number of informational findings.
    pub info: u32,
    /// Number of files reviewed.
    pub files_reviewed: u32,
    /// Overall verdict (approve, review, reject).
    pub verdict: String,
}

/// QA review output.
#[derive(Debug, Serialize)]
pub struct QAReviewOutput {
    /// Unique report identifier.
    pub report_id: String,
    /// Base branch used for comparison.
    pub base_branch: String,
    /// Issues found during review.
    pub issues: Vec<QAIssue>,
    /// Aggregate summary of findings.
    pub summary: QASummary,
    /// Agent that performed the review.
    pub agent_id: Option<String>,
}

/// QA bugs request.
#[derive(Debug, Deserialize)]
pub struct QABugsRequest {
    /// Base branch for comparison.
    pub base: String,
}

/// QA report request.
#[derive(Debug, Deserialize)]
pub struct QAReportRequest {
    /// Base branch for comparison.
    pub base: String,
    /// Whether to generate a full detailed report.
    pub full: bool,
}

/// QA report response.
#[derive(Debug, Serialize)]
pub struct QAReportResponse {
    /// Review results with issues and summary.
    pub review: QAReviewOutput,
    /// Full report text (if requested).
    pub report: Option<String>,
}

/// Agent info for QA.
#[derive(Debug, Serialize)]
pub struct QAAgentInfo {
    /// Agent identifier.
    pub id: String,
    /// Display name of the agent.
    pub name: String,
    /// Model used by the agent.
    pub model: String,
}

// ============================================================================
// Handlers
// ============================================================================

/// Get the configured QA agent.
pub fn get_qa_agent_handler(state: &AppState) -> Result<Option<QAAgentInfo>, TransportError> {
    let agents = load_agents(state);

    let qa_agent = agents
        .agents
        .iter()
        .find(|a| a.is_qa_agent)
        .map(|a| QAAgentInfo {
            id: a.id.clone(),
            name: a.name.clone(),
            model: a
                .openrouter
                .as_ref()
                .map(|o| o.model.clone())
                .unwrap_or_else(|| "claude".to_string()),
        });

    Ok(qa_agent)
}

/// Run QA review.
pub async fn run_review_handler(
    state: &AppState,
    request: QAReviewRequest,
) -> Result<QAReviewOutput, TransportError> {
    // Check for QA agent first
    let agents = load_agents(state);
    let qa_agent = agents.agents.iter().find(|a| a.is_qa_agent);

    if qa_agent.is_none() {
        return Err(TransportError::BadRequest(
            "No QA agent configured. Please set an agent as QA agent in Agent Manager.".to_string(),
        ));
    }

    let agent_id = qa_agent.map(|a| a.id.clone());

    // Run ckrv qa review command
    let output = tokio::process::Command::new("ckrv")
        .args(["qa", "review", "--base", &request.base, "--json"])
        .current_dir(&state.project_root)
        .output()
        .await
        .map_err(|e| TransportError::Internal(format!("Failed to run review: {e}")))?;

    if output.status.success() {
        if let Ok(stdout) = String::from_utf8(output.stdout) {
            if let Ok(result) = serde_json::from_str::<serde_json::Value>(&stdout) {
                let issues: Vec<QAIssue> = result["issues"]
                    .as_array()
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|i| {
                                Some(QAIssue {
                                    id: i["id"].as_str()?.to_string(),
                                    file: i["file"].as_str()?.to_string(),
                                    line: i["line"].as_u64().map(|n| n as u32),
                                    severity: i["severity"].as_str().unwrap_or("info").to_string(),
                                    category: i["category"]
                                        .as_str()
                                        .unwrap_or("code_quality")
                                        .to_string(),
                                    message: i["message"].as_str()?.to_string(),
                                    suggestion: i["suggestion"].as_str().map(str::to_string),
                                })
                            })
                            .collect()
                    })
                    .unwrap_or_default();

                let summary_val = &result["summary"];
                let summary = QASummary {
                    total_issues: summary_val["total_issues"]
                        .as_u64()
                        .unwrap_or(issues.len() as u64) as u32,
                    critical: summary_val["critical"].as_u64().unwrap_or(0) as u32,
                    major: summary_val["major"].as_u64().unwrap_or(0) as u32,
                    minor: summary_val["minor"].as_u64().unwrap_or(0) as u32,
                    info: summary_val["info"].as_u64().unwrap_or(0) as u32,
                    files_reviewed: summary_val["files_reviewed"].as_u64().unwrap_or(0) as u32,
                    verdict: summary_val["verdict"]
                        .as_str()
                        .unwrap_or("review")
                        .to_string(),
                };

                return Ok(QAReviewOutput {
                    report_id: result["report_id"].as_str().unwrap_or("qa-0").to_string(),
                    base_branch: result["base_branch"]
                        .as_str()
                        .unwrap_or(&request.base)
                        .to_string(),
                    issues,
                    summary,
                    agent_id,
                });
            }
        }
    }

    let stderr = String::from_utf8_lossy(&output.stderr);
    Err(TransportError::Internal(format!("Review failed: {stderr}")))
}

/// Run bugs analysis.
pub async fn run_bugs_handler(
    state: &AppState,
    request: QABugsRequest,
) -> Result<Vec<QAIssue>, TransportError> {
    // Check for QA agent first
    let agents = load_agents(state);
    let has_agent = agents.agents.iter().any(|a| a.is_qa_agent);

    if !has_agent {
        return Err(TransportError::BadRequest(
            "No QA agent configured. Please set an agent as QA agent in Agent Manager.".to_string(),
        ));
    }

    // Run ckrv qa bugs command
    let output = tokio::process::Command::new("ckrv")
        .args(["qa", "bugs", "--base", &request.base, "--json"])
        .current_dir(&state.project_root)
        .output()
        .await
        .map_err(|e| TransportError::Internal(format!("Failed to run bugs analysis: {e}")))?;

    if output.status.success() {
        if let Ok(stdout) = String::from_utf8(output.stdout) {
            if let Ok(issues) = serde_json::from_str::<Vec<QAIssue>>(&stdout) {
                return Ok(issues);
            }
        }
    }

    let stderr = String::from_utf8_lossy(&output.stderr);
    Err(TransportError::Internal(format!(
        "Bugs analysis failed: {stderr}"
    )))
}

/// Generate full QA report.
pub async fn run_report_handler(
    state: &AppState,
    request: QAReportRequest,
) -> Result<QAReportResponse, TransportError> {
    // Check for QA agent first
    let agents = load_agents(state);
    let qa_agent = agents.agents.iter().find(|a| a.is_qa_agent);

    if qa_agent.is_none() {
        return Err(TransportError::BadRequest(
            "No QA agent configured. Please set an agent as QA agent in Agent Manager.".to_string(),
        ));
    }

    let agent_id = qa_agent.map(|a| a.id.clone());

    // Run ckrv qa report command
    let mut args = vec!["qa", "report", "--base", &request.base];
    if request.full {
        args.push("--full");
    }

    let output = tokio::process::Command::new("ckrv")
        .args(&args)
        .current_dir(&state.project_root)
        .output()
        .await
        .map_err(|e| TransportError::Internal(format!("Failed to generate report: {e}")))?;

    let stdout = String::from_utf8_lossy(&output.stdout);

    if output.status.success() {
        let summary = QASummary {
            total_issues: 0,
            critical: 0,
            major: 0,
            minor: 0,
            info: 0,
            files_reviewed: 0,
            verdict: "review".to_string(),
        };

        return Ok(QAReportResponse {
            review: QAReviewOutput {
                report_id: format!("qa-report-{}", chrono::Utc::now().timestamp()),
                base_branch: request.base.clone(),
                issues: vec![],
                summary,
                agent_id,
            },
            report: Some(stdout.to_string()),
        });
    }

    let stderr = String::from_utf8_lossy(&output.stderr);
    Err(TransportError::Internal(format!(
        "Report generation failed: {stderr}"
    )))
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[tokio::test]
    async fn test_get_qa_agent_handler() {
        let state = AppState::new(PathBuf::from("/tmp/test-qa"));
        let result = get_qa_agent_handler(&state);
        assert!(result.is_ok());
    }
}
