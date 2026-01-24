//! QA command API endpoints for UI

use axum::{extract::State, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};

use crate::state::AppState;
use super::agents::load_agents;

/// QA review request
#[derive(Debug, Deserialize)]
pub struct QAReviewRequest {
    pub base: String,
}

/// QA review response
#[derive(Debug, Serialize)]
pub struct QAReviewResponse {
    pub success: bool,
    pub review: Option<QAReviewOutput>,
    pub error: Option<String>,
}

/// QA review output
#[derive(Debug, Serialize)]
pub struct QAReviewOutput {
    pub report_id: String,
    pub base_branch: String,
    pub issues: Vec<QAIssue>,
    pub summary: QASummary,
    pub agent_id: Option<String>,
}

/// QA issue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QAIssue {
    pub id: String,
    pub file: String,
    pub line: Option<u32>,
    pub severity: String,
    pub category: String,
    pub message: String,
    pub suggestion: Option<String>,
}

/// QA summary
#[derive(Debug, Serialize)]
pub struct QASummary {
    pub total_issues: u32,
    pub critical: u32,
    pub major: u32,
    pub minor: u32,
    pub info: u32,
    pub files_reviewed: u32,
    pub verdict: String,
}

/// QA bugs request
#[derive(Debug, Deserialize)]
pub struct QABugsRequest {
    pub base: String,
}

/// QA bugs response
#[derive(Debug, Serialize)]
pub struct QABugsResponse {
    pub success: bool,
    pub issues: Option<Vec<QAIssue>>,
    pub error: Option<String>,
}

/// QA report request
#[derive(Debug, Deserialize)]
pub struct QAReportRequest {
    pub base: String,
    pub full: bool,
}

/// QA report response
#[derive(Debug, Serialize)]
pub struct QAReportResponse {
    pub success: bool,
    pub review: Option<QAReviewOutput>,
    pub report: Option<String>,
    pub error: Option<String>,
}

/// Agent response for QA
#[derive(Debug, Serialize)]
pub struct QAAgentResponse {
    pub agent: Option<AgentInfo>,
}

/// Minimal agent info for UI
#[derive(Debug, Serialize)]
pub struct AgentInfo {
    pub id: String,
    pub name: String,
    pub model: String,
}

/// Get the QA agent
pub async fn get_qa_agent(
    State(state): State<AppState>,
) -> impl IntoResponse {
    let agents = load_agents(&state);
    
    // Find agent with is_qa_agent = true
    let qa_agent = agents.agents.iter()
        .find(|a| a.is_qa_agent)
        .map(|a| AgentInfo {
            id: a.id.clone(),
            name: a.name.clone(),
            model: a.openrouter.as_ref().map(|o| o.model.clone()).unwrap_or_else(|| "claude".to_string()),
        });
    
    Json(QAAgentResponse { agent: qa_agent })
}

/// Run QA review
pub async fn run_review(
    State(state): State<AppState>,
    Json(req): Json<QAReviewRequest>,
) -> impl IntoResponse {
    // Check for QA agent first
    let agents = load_agents(&state);
    let qa_agent = agents.agents.iter()
        .find(|a| a.is_qa_agent);
    
    if qa_agent.is_none() {
        return Json(QAReviewResponse {
            success: false,
            review: None,
            error: Some("No QA agent configured. Please set an agent as QA agent in Agent Manager.".to_string()),
        });
    }
    
    let agent_id = qa_agent.map(|a| a.id.clone());
    
    // Run ckrv qa review command
    let output = tokio::process::Command::new("ckrv")
        .args(["qa", "review", "--base", &req.base, "--json"])
        .current_dir(&state.project_root)
        .output()
        .await;
    
    match output {
        Ok(output) => {
            if output.status.success() {
                if let Ok(stdout) = String::from_utf8(output.stdout) {
                    if let Ok(result) = serde_json::from_str::<serde_json::Value>(&stdout) {
                        let issues: Vec<QAIssue> = result["issues"]
                            .as_array()
                            .map(|arr| arr.iter().filter_map(|i| {
                                Some(QAIssue {
                                    id: i["id"].as_str()?.to_string(),
                                    file: i["file"].as_str()?.to_string(),
                                    line: i["line"].as_u64().map(|n| n as u32),
                                    severity: i["severity"].as_str().unwrap_or("info").to_string(),
                                    category: i["category"].as_str().unwrap_or("code_quality").to_string(),
                                    message: i["message"].as_str()?.to_string(),
                                    suggestion: i["suggestion"].as_str().map(|s| s.to_string()),
                                })
                            }).collect())
                            .unwrap_or_default();
                        
                        let summary_val = &result["summary"];
                        let summary = QASummary {
                            total_issues: summary_val["total_issues"].as_u64().unwrap_or(issues.len() as u64) as u32,
                            critical: summary_val["critical"].as_u64().unwrap_or(0) as u32,
                            major: summary_val["major"].as_u64().unwrap_or(0) as u32,
                            minor: summary_val["minor"].as_u64().unwrap_or(0) as u32,
                            info: summary_val["info"].as_u64().unwrap_or(0) as u32,
                            files_reviewed: summary_val["files_reviewed"].as_u64().unwrap_or(0) as u32,
                            verdict: summary_val["verdict"].as_str().unwrap_or("review").to_string(),
                        };
                        
                        return Json(QAReviewResponse {
                            success: true,
                            review: Some(QAReviewOutput {
                                report_id: result["report_id"].as_str().unwrap_or("qa-0").to_string(),
                                base_branch: result["base_branch"].as_str().unwrap_or(&req.base).to_string(),
                                issues,
                                summary,
                                agent_id,
                            }),
                            error: None,
                        });
                    }
                }
            }
            
            let stderr = String::from_utf8_lossy(&output.stderr);
            Json(QAReviewResponse {
                success: false,
                review: None,
                error: Some(stderr.to_string()),
            })
        }
        Err(e) => Json(QAReviewResponse {
            success: false,
            review: None,
            error: Some(format!("Failed to run review: {}", e)),
        }),
    }
}

/// Run bugs analysis
pub async fn run_bugs(
    State(state): State<AppState>,
    Json(req): Json<QABugsRequest>,
) -> impl IntoResponse {
    // Check for QA agent first
    let agents = load_agents(&state);
    let has_agent = agents.agents.iter()
        .any(|a| a.is_qa_agent);
    
    if !has_agent {
        return Json(QABugsResponse {
            success: false,
            issues: None,
            error: Some("No QA agent configured. Please set an agent as QA agent in Agent Manager.".to_string()),
        });
    }
    
    // Run ckrv qa bugs command
    let output = tokio::process::Command::new("ckrv")
        .args(["qa", "bugs", "--base", &req.base, "--json"])
        .current_dir(&state.project_root)
        .output()
        .await;
    
    match output {
        Ok(output) => {
            if output.status.success() {
                if let Ok(stdout) = String::from_utf8(output.stdout) {
                    if let Ok(issues) = serde_json::from_str::<Vec<QAIssue>>(&stdout) {
                        return Json(QABugsResponse {
                            success: true,
                            issues: Some(issues),
                            error: None,
                        });
                    }
                }
            }
            
            let stderr = String::from_utf8_lossy(&output.stderr);
            Json(QABugsResponse {
                success: false,
                issues: None,
                error: Some(stderr.to_string()),
            })
        }
        Err(e) => Json(QABugsResponse {
            success: false,
            issues: None,
            error: Some(format!("Failed to run bugs analysis: {}", e)),
        }),
    }
}

/// Generate full QA report
pub async fn run_report(
    State(state): State<AppState>,
    Json(req): Json<QAReportRequest>,
) -> impl IntoResponse {
    // Check for QA agent first
    let agents = load_agents(&state);
    let qa_agent = agents.agents.iter()
        .find(|a| a.is_qa_agent);
    
    if qa_agent.is_none() {
        return Json(QAReportResponse {
            success: false,
            review: None,
            report: None,
            error: Some("No QA agent configured. Please set an agent as QA agent in Agent Manager.".to_string()),
        });
    }
    
    let agent_id = qa_agent.map(|a| a.id.clone());
    
    // Run ckrv qa report command
    let mut args = vec!["qa", "report", "--base", &req.base];
    if req.full {
        args.push("--full");
    }
    
    let output = tokio::process::Command::new("ckrv")
        .args(&args)
        .current_dir(&state.project_root)
        .output()
        .await;
    
    match output {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            
            if output.status.success() {
                // The report command outputs markdown, not JSON
                // Just return the markdown as the report
                
                // Create a minimal review summary
                let summary = QASummary {
                    total_issues: 0,
                    critical: 0,
                    major: 0,
                    minor: 0,
                    info: 0,
                    files_reviewed: 0,
                    verdict: "review".to_string(),
                };
                
                Json(QAReportResponse {
                    success: true,
                    review: Some(QAReviewOutput {
                        report_id: format!("qa-report-{}", chrono::Utc::now().timestamp()),
                        base_branch: req.base.clone(),
                        issues: vec![],
                        summary,
                        agent_id,
                    }),
                    report: Some(stdout.to_string()),
                    error: None,
                })
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                Json(QAReportResponse {
                    success: false,
                    review: None,
                    report: None,
                    error: Some(stderr.to_string()),
                })
            }
        }
        Err(e) => Json(QAReportResponse {
            success: false,
            review: None,
            report: None,
            error: Some(format!("Failed to generate report: {}", e)),
        }),
    }
}
