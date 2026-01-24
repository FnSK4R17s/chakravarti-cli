//! Test command API endpoints for UI

use axum::{extract::State, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};

use crate::state::AppState;
use super::agents::load_agents;

/// Test run request
#[derive(Debug, Deserialize)]
pub struct TestRunRequest {
    pub base: String,
}

/// Test run response
#[derive(Debug, Serialize)]
pub struct TestRunResponse {
    pub success: bool,
    pub result: Option<TestResult>,
    pub error: Option<String>,
}

/// Test result structure
#[derive(Debug, Serialize)]
pub struct TestResult {
    pub total: u32,
    pub passed: u32,
    pub failed: u32,
    pub skipped: u32,
    pub duration_ms: u64,
    pub failures: Vec<TestFailure>,
    pub framework: String,
}

/// Test failure details
#[derive(Debug, Serialize)]
pub struct TestFailure {
    pub name: String,
    pub file: String,
    pub line: Option<u32>,
    pub message: String,
    pub stdout: Option<String>,
    pub stderr: Option<String>,
}

/// Test plan response
#[derive(Debug, Serialize)]
pub struct TestPlanResponse {
    pub success: bool,
    pub plan: Option<TestPlanOutput>,
    pub error: Option<String>,
}

/// Test plan output
#[derive(Debug, Serialize)]
pub struct TestPlanOutput {
    pub plan_id: String,
    pub base_branch: String,
    pub changed_files: Vec<ChangedFileInfo>,
    pub proposed_tests: Vec<ProposedTest>,
}

/// Changed file info
#[derive(Debug, Serialize)]
pub struct ChangedFileInfo {
    pub path: String,
    pub change_type: String,
    pub lines_added: u32,
    pub lines_removed: u32,
    pub has_tests: bool,
}

/// Proposed test
#[derive(Debug, Serialize)]
pub struct ProposedTest {
    pub target_file: String,
    pub test_file: String,
    pub description: String,
    pub priority: String,
}

/// Test write request
#[derive(Debug, Deserialize)]
pub struct TestWriteRequest {
    pub base: String,
    pub run: bool,
}

/// Test write response
#[derive(Debug, Serialize)]
pub struct TestWriteResponse {
    pub success: bool,
    pub message: Option<String>,
    pub error: Option<String>,
}

/// Coverage request
#[derive(Debug, Deserialize)]
pub struct CoverageRequest {
    pub base: String,
}

/// Coverage response
#[derive(Debug, Serialize)]
pub struct CoverageResponse {
    pub success: bool,
    pub coverage: Option<CoverageResult>,
    pub error: Option<String>,
}

/// Coverage result
#[derive(Debug, Serialize)]
pub struct CoverageResult {
    pub total: u32,
    pub covered: u32,
    pub uncovered: u32,
    pub coverage_percent: f64,
}

/// Agent response for test writer
#[derive(Debug, Serialize)]
pub struct TestAgentResponse {
    pub agent: Option<AgentInfo>,
}

/// Minimal agent info for UI
#[derive(Debug, Serialize)]
pub struct AgentInfo {
    pub id: String,
    pub name: String,
    pub model: String,
}

/// Get the test writer agent
pub async fn get_test_agent(
    State(state): State<AppState>,
) -> impl IntoResponse {
    let agents = load_agents(&state);
    
    // Find agent with is_test_writer = true
    let test_writer = agents.agents.iter()
        .find(|a| a.is_test_writer)
        .map(|a| AgentInfo {
            id: a.id.clone(),
            name: a.name.clone(),
            model: a.openrouter.as_ref().map(|o| o.model.clone()).unwrap_or_else(|| "claude".to_string()),
        });
    
    Json(TestAgentResponse { agent: test_writer })
}

/// Run tests
pub async fn run_tests(
    State(state): State<AppState>,
    Json(req): Json<TestRunRequest>,
) -> impl IntoResponse {
    // Run ckrv test run command
    let output = tokio::process::Command::new("ckrv")
        .args(["test", "run", "--base", &req.base, "--json"])
        .current_dir(&state.project_root)
        .output()
        .await;
    
    match output {
        Ok(output) => {
            if output.status.success() {
                // Parse JSON output
                if let Ok(stdout) = String::from_utf8(output.stdout) {
                    if let Ok(result) = serde_json::from_str::<serde_json::Value>(&stdout) {
                        // Convert to our TestResult format
                        let test_result = TestResult {
                            total: result["result"]["total"].as_u64().unwrap_or(0) as u32,
                            passed: result["result"]["passed"].as_u64().unwrap_or(0) as u32,
                            failed: result["result"]["failed"].as_u64().unwrap_or(0) as u32,
                            skipped: result["result"]["skipped"].as_u64().unwrap_or(0) as u32,
                            duration_ms: result["result"]["duration_ms"].as_u64().unwrap_or(0),
                            failures: vec![], // Parse failures if needed
                            framework: result["result"]["framework"].as_str().unwrap_or("unknown").to_string(),
                        };
                        return Json(TestRunResponse {
                            success: true,
                            result: Some(test_result),
                            error: None,
                        });
                    }
                }
            }
            
            let stderr = String::from_utf8_lossy(&output.stderr);
            Json(TestRunResponse {
                success: false,
                result: None,
                error: Some(stderr.to_string()),
            })
        }
        Err(e) => Json(TestRunResponse {
            success: false,
            result: None,
            error: Some(format!("Failed to run tests: {}", e)),
        }),
    }
}

/// Generate test plan
pub async fn plan_tests(
    State(state): State<AppState>,
    Json(req): Json<TestRunRequest>,
) -> impl IntoResponse {
    // Run ckrv test plan command
    let output = tokio::process::Command::new("ckrv")
        .args(["test", "plan", "--base", &req.base, "--json"])
        .current_dir(&state.project_root)
        .output()
        .await;
    
    match output {
        Ok(output) => {
            if output.status.success() {
                if let Ok(stdout) = String::from_utf8(output.stdout) {
                    if let Ok(result) = serde_json::from_str::<serde_json::Value>(&stdout) {
                        let plan = TestPlanOutput {
                            plan_id: result["plan_id"].as_str().unwrap_or("plan-0").to_string(),
                            base_branch: result["base_branch"].as_str().unwrap_or(&req.base).to_string(),
                            changed_files: result["changed_files"]
                                .as_array()
                                .map(|arr| arr.iter().map(|f| ChangedFileInfo {
                                    path: f["path"].as_str().unwrap_or("").to_string(),
                                    change_type: f["change_type"].as_str().unwrap_or("modified").to_string(),
                                    lines_added: f["lines_added"].as_u64().unwrap_or(0) as u32,
                                    lines_removed: f["lines_removed"].as_u64().unwrap_or(0) as u32,
                                    has_tests: f["has_tests"].as_bool().unwrap_or(false),
                                }).collect())
                                .unwrap_or_default(),
                            proposed_tests: result["proposed_tests"]
                                .as_array()
                                .map(|arr| arr.iter().map(|t| ProposedTest {
                                    target_file: t["target_file"].as_str().unwrap_or("").to_string(),
                                    test_file: t["test_file"].as_str().unwrap_or("").to_string(),
                                    description: t["description"].as_str().unwrap_or("").to_string(),
                                    priority: t["priority"].as_str().unwrap_or("medium").to_string(),
                                }).collect())
                                .unwrap_or_default(),
                        };
                        return Json(TestPlanResponse {
                            success: true,
                            plan: Some(plan),
                            error: None,
                        });
                    }
                }
            }
            
            let stderr = String::from_utf8_lossy(&output.stderr);
            Json(TestPlanResponse {
                success: false,
                plan: None,
                error: Some(stderr.to_string()),
            })
        }
        Err(e) => Json(TestPlanResponse {
            success: false,
            plan: None,
            error: Some(format!("Failed to plan tests: {}", e)),
        }),
    }
}

/// Write tests using agent
pub async fn write_tests(
    State(state): State<AppState>,
    Json(req): Json<TestWriteRequest>,
) -> impl IntoResponse {
    // Check for test writer agent first
    let agents = load_agents(&state);
    let has_agent = agents.agents.iter()
        .any(|a| a.is_test_writer);
    
    if !has_agent {
        return Json(TestWriteResponse {
            success: false,
            message: None,
            error: Some("No test writer agent configured. Please set an agent as test writer in Agent Manager.".to_string()),
        });
    }
    
    // Run ckrv test write command
    let mut args = vec!["test", "write", "--base", &req.base];
    if req.run {
        args.push("--run");
    }
    
    let output = tokio::process::Command::new("ckrv")
        .args(&args)
        .current_dir(&state.project_root)
        .output()
        .await;
    
    match output {
        Ok(output) => {
            if output.status.success() {
                Json(TestWriteResponse {
                    success: true,
                    message: Some("Test writing initiated. Agent is analyzing changes.".to_string()),
                    error: None,
                })
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                Json(TestWriteResponse {
                    success: false,
                    message: None,
                    error: Some(stderr.to_string()),
                })
            }
        }
        Err(e) => Json(TestWriteResponse {
            success: false,
            message: None,
            error: Some(format!("Failed to write tests: {}", e)),
        }),
    }
}

/// Check coverage
pub async fn check_coverage(
    State(state): State<AppState>,
    Json(req): Json<CoverageRequest>,
) -> impl IntoResponse {
    // Run ckrv test coverage command
    let output = tokio::process::Command::new("ckrv")
        .args(["test", "coverage", "--base", &req.base, "--json"])
        .current_dir(&state.project_root)
        .output()
        .await;
    
    match output {
        Ok(output) => {
            if output.status.success() {
                if let Ok(stdout) = String::from_utf8(output.stdout) {
                    if let Ok(result) = serde_json::from_str::<serde_json::Value>(&stdout) {
                        let coverage = CoverageResult {
                            total: result["total"].as_u64().unwrap_or(0) as u32,
                            covered: result["covered"].as_u64().unwrap_or(0) as u32,
                            uncovered: result["uncovered"].as_u64().unwrap_or(0) as u32,
                            coverage_percent: result["coverage_percent"].as_f64().unwrap_or(0.0),
                        };
                        return Json(CoverageResponse {
                            success: true,
                            coverage: Some(coverage),
                            error: None,
                        });
                    }
                }
            }
            
            let stderr = String::from_utf8_lossy(&output.stderr);
            Json(CoverageResponse {
                success: false,
                coverage: None,
                error: Some(stderr.to_string()),
            })
        }
        Err(e) => Json(CoverageResponse {
            success: false,
            coverage: None,
            error: Some(format!("Failed to check coverage: {}", e)),
        }),
    }
}
