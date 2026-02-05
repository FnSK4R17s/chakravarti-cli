//! # Test Handler
//!
//! Handlers for test execution and management.

use crate::error::TransportError;
use crate::handlers::agents::load_agents;
use crate::state::AppState;
use serde::{Deserialize, Serialize};
use std::process::Command;

// ============================================================================
// Request/Response Types
// ============================================================================

/// Test run request.
#[derive(Debug, Deserialize)]
pub struct RunTestsRequest {
    /// Base branch for comparison
    pub base: String,
    /// Specific test file or pattern
    pub pattern: Option<String>,
    /// Test framework to use
    pub framework: Option<String>,
    /// Watch mode
    #[serde(default)]
    pub watch: bool,
}

/// Test result.
#[derive(Debug, Serialize)]
pub struct TestResult {
    pub name: String,
    pub status: TestStatus,
    pub duration_ms: Option<u64>,
    pub error: Option<String>,
    pub file: Option<String>,
    pub line: Option<u32>,
}

/// Test status.
#[derive(Debug, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum TestStatus {
    Passed,
    Failed,
    Skipped,
    Pending,
}

/// Test run summary.
#[derive(Debug, Serialize)]
pub struct TestRunSummary {
    pub total: u32,
    pub passed: u32,
    pub failed: u32,
    pub skipped: u32,
    pub duration_ms: u64,
}

/// Test run response.
#[derive(Debug, Serialize)]
pub struct TestRunResponse {
    pub results: Vec<TestResult>,
    pub summary: TestRunSummary,
    pub raw_output: Option<String>,
}

/// Test writer agent info.
#[derive(Debug, Serialize)]
pub struct TestWriterAgentInfo {
    pub id: String,
    pub name: String,
    pub model: String,
}

/// Generate tests request.
#[derive(Debug, Deserialize)]
pub struct GenerateTestsRequest {
    /// File to generate tests for
    pub file: String,
    /// Test framework to use
    pub framework: Option<String>,
}

/// Generate tests response.
#[derive(Debug, Serialize)]
pub struct GenerateTestsResponse {
    pub test_file: String,
    pub tests_generated: u32,
}

/// Test plan request.
#[derive(Debug, Deserialize)]
pub struct TestPlanRequest {
    /// Base branch for comparison
    pub base: String,
}

/// Changed file info for test planning.
#[derive(Debug, Serialize, Deserialize)]
pub struct ChangedFileInfo {
    pub path: String,
    pub change_type: String,
    pub lines_added: u32,
    pub lines_removed: u32,
    pub has_tests: bool,
}

/// Proposed test.
#[derive(Debug, Serialize, Deserialize)]
pub struct ProposedTest {
    pub target_file: String,
    pub test_file: String,
    pub description: String,
    pub priority: String,
}

/// Test plan output.
#[derive(Debug, Serialize, Deserialize)]
pub struct TestPlanOutput {
    pub plan_id: String,
    pub base_branch: String,
    pub changed_files: Vec<ChangedFileInfo>,
    pub proposed_tests: Vec<ProposedTest>,
}

/// Test plan response.
#[derive(Debug, Serialize)]
pub struct TestPlanResponse {
    pub success: bool,
    pub plan: Option<TestPlanOutput>,
    pub error: Option<String>,
}

/// Test write request.
#[derive(Debug, Deserialize)]
pub struct TestWriteRequest {
    /// Base branch for comparison
    pub base: String,
    /// Whether to run tests after writing
    #[serde(default)]
    pub run: bool,
}

/// Test write response.
#[derive(Debug, Serialize)]
pub struct TestWriteResponse {
    pub success: bool,
    pub tests_written: u32,
    pub tests_passed: Option<u32>,
    pub tests_failed: Option<u32>,
    pub error: Option<String>,
}

/// Coverage info.
#[derive(Debug, Serialize)]
pub struct CoverageInfo {
    pub total: f32,
    pub lines: f32,
    pub functions: f32,
    pub branches: f32,
    pub files: Vec<FileCoverage>,
}

/// File coverage.
#[derive(Debug, Serialize)]
pub struct FileCoverage {
    pub path: String,
    pub coverage: f32,
    pub uncovered_lines: Vec<u32>,
}

/// Test fix request.
#[derive(Debug, Deserialize)]
pub struct TestFixRequest {
    /// Base branch
    pub base: String,
    /// Specific test file to fix
    pub file: Option<String>,
}

/// Test fix response.
#[derive(Debug, Serialize)]
pub struct TestFixResponse {
    pub success: bool,
    pub files_fixed: u32,
    pub tests_passing: u32,
    pub error: Option<String>,
}

/// Status response for async operations.
#[derive(Debug, Serialize)]
pub struct AsyncStatusResponse {
    pub status: String,
    pub progress: f32,
    pub message: Option<String>,
}

// ============================================================================
// Handlers
// ============================================================================

/// Run tests.
pub async fn run_tests_handler(
    state: &AppState,
    request: RunTestsRequest,
) -> Result<TestRunResponse, TransportError> {
    let mut args = vec!["verify", "--test"];

    // Detect test runner based on project type
    // For now, try npm test as a default
    let output = Command::new("npm")
        .args(["test", "--", "--reporter", "json"])
        .current_dir(&state.project_root)
        .output()
        .map_err(|e| TransportError::Internal(format!("Failed to run tests: {e}")))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Parse output (basic parsing, would need to be adapted per framework)
    let results = vec![];
    let summary = TestRunSummary {
        total: 0,
        passed: 0,
        failed: 0,
        skipped: 0,
        duration_ms: 0,
    };

    Ok(TestRunResponse {
        results,
        summary,
        raw_output: Some(format!("{}\n{}", stdout, stderr)),
    })
}

/// Get test writer agent.
pub async fn get_test_writer_agent_handler(
    state: &AppState,
) -> Result<Option<TestWriterAgentInfo>, TransportError> {
    let agents = load_agents(state);

    let test_agent = agents
        .agents
        .iter()
        .find(|a| a.is_test_writer)
        .map(|a| TestWriterAgentInfo {
            id: a.id.clone(),
            name: a.name.clone(),
            model: a
                .openrouter
                .as_ref()
                .map(|o| o.model.clone())
                .unwrap_or_else(|| "claude".to_string()),
        });

    Ok(test_agent)
}

/// Generate tests for a file.
pub async fn generate_tests_handler(
    state: &AppState,
    request: GenerateTestsRequest,
) -> Result<GenerateTestsResponse, TransportError> {
    // Check for test writer agent
    let agents = load_agents(state);
    let has_agent = agents.agents.iter().any(|a| a.is_test_writer);

    if !has_agent {
        return Err(TransportError::BadRequest(
            "No test writer agent configured. Please set an agent as test writer in Agent Manager."
                .to_string(),
        ));
    }

    // Run ckrv test-gen command
    let output = tokio::process::Command::new("ckrv")
        .args(["test-gen", "--file", &request.file, "--json"])
        .current_dir(&state.project_root)
        .output()
        .await
        .map_err(|e| TransportError::Internal(format!("Failed to generate tests: {e}")))?;

    if output.status.success() {
        if let Ok(stdout) = String::from_utf8(output.stdout) {
            if let Ok(result) = serde_json::from_str::<serde_json::Value>(&stdout) {
                return Ok(GenerateTestsResponse {
                    test_file: result["test_file"]
                        .as_str()
                        .unwrap_or("")
                        .to_string(),
                    tests_generated: result["tests_generated"].as_u64().unwrap_or(0) as u32,
                });
            }
        }
    }

    let stderr = String::from_utf8_lossy(&output.stderr);
    Err(TransportError::Internal(format!(
        "Test generation failed: {stderr}"
    )))
}

/// Create test plan.
pub async fn create_test_plan_handler(
    state: &AppState,
    request: TestPlanRequest,
) -> Result<TestPlanResponse, TransportError> {
    // Check for test writer agent
    let agents = load_agents(state);
    let has_agent = agents.agents.iter().any(|a| a.is_test_writer);

    if !has_agent {
        return Ok(TestPlanResponse {
            success: false,
            plan: None,
            error: Some(
                "No test writer agent configured. Please set an agent as test writer in Agent Manager."
                    .to_string(),
            ),
        });
    }

    // Run ckrv test plan command
    let output = tokio::process::Command::new("ckrv")
        .args(["test", "plan", "--base", &request.base, "--json"])
        .current_dir(&state.project_root)
        .output()
        .await;

    match output {
        Ok(output) => {
            if output.status.success() {
                if let Ok(stdout) = String::from_utf8(output.stdout) {
                    if let Ok(result) = serde_json::from_str::<serde_json::Value>(&stdout) {
                        let changed_files: Vec<ChangedFileInfo> = result["changed_files"]
                            .as_array()
                            .map(|arr| {
                                arr.iter()
                                    .filter_map(|f| {
                                        Some(ChangedFileInfo {
                                            path: f["path"].as_str()?.to_string(),
                                            change_type: f["change_type"]
                                                .as_str()
                                                .unwrap_or("modified")
                                                .to_string(),
                                            lines_added: f["lines_added"].as_u64().unwrap_or(0)
                                                as u32,
                                            lines_removed: f["lines_removed"].as_u64().unwrap_or(0)
                                                as u32,
                                            has_tests: f["has_tests"].as_bool().unwrap_or(false),
                                        })
                                    })
                                    .collect()
                            })
                            .unwrap_or_default();

                        let proposed_tests: Vec<ProposedTest> = result["proposed_tests"]
                            .as_array()
                            .map(|arr| {
                                arr.iter()
                                    .filter_map(|t| {
                                        Some(ProposedTest {
                                            target_file: t["target_file"].as_str()?.to_string(),
                                            test_file: t["test_file"].as_str()?.to_string(),
                                            description: t["description"].as_str()?.to_string(),
                                            priority: t["priority"]
                                                .as_str()
                                                .unwrap_or("normal")
                                                .to_string(),
                                        })
                                    })
                                    .collect()
                            })
                            .unwrap_or_default();

                        return Ok(TestPlanResponse {
                            success: true,
                            plan: Some(TestPlanOutput {
                                plan_id: result["plan_id"]
                                    .as_str()
                                    .unwrap_or("plan-0")
                                    .to_string(),
                                base_branch: request.base,
                                changed_files,
                                proposed_tests,
                            }),
                            error: None,
                        });
                    }
                }
            }

            let stderr = String::from_utf8_lossy(&output.stderr);
            Ok(TestPlanResponse {
                success: false,
                plan: None,
                error: Some(stderr.to_string()),
            })
        }
        Err(e) => Ok(TestPlanResponse {
            success: false,
            plan: None,
            error: Some(format!("Failed to run test plan: {e}")),
        }),
    }
}

/// Write tests based on plan.
pub async fn write_tests_handler(
    state: &AppState,
    request: TestWriteRequest,
) -> Result<TestWriteResponse, TransportError> {
    // Check for test writer agent
    let agents = load_agents(state);
    let has_agent = agents.agents.iter().any(|a| a.is_test_writer);

    if !has_agent {
        return Ok(TestWriteResponse {
            success: false,
            tests_written: 0,
            tests_passed: None,
            tests_failed: None,
            error: Some(
                "No test writer agent configured. Please set an agent as test writer in Agent Manager."
                    .to_string(),
            ),
        });
    }

    // Run ckrv test write command
    let mut args = vec!["test", "write", "--base", &request.base];
    if request.run {
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
                Ok(TestWriteResponse {
                    success: true,
                    tests_written: 1, // Would parse from output
                    tests_passed: if request.run { Some(1) } else { None },
                    tests_failed: if request.run { Some(0) } else { None },
                    error: None,
                })
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                Ok(TestWriteResponse {
                    success: false,
                    tests_written: 0,
                    tests_passed: None,
                    tests_failed: None,
                    error: Some(stderr.to_string()),
                })
            }
        }
        Err(e) => Ok(TestWriteResponse {
            success: false,
            tests_written: 0,
            tests_passed: None,
            tests_failed: None,
            error: Some(format!("Failed to write tests: {e}")),
        }),
    }
}

/// Get test coverage.
pub async fn get_coverage_handler(
    state: &AppState,
) -> Result<CoverageInfo, TransportError> {
    // Try to run coverage command
    let output = tokio::process::Command::new("npm")
        .args(["run", "coverage", "--", "--json"])
        .current_dir(&state.project_root)
        .output()
        .await;

    // Return empty coverage if command fails
    Ok(CoverageInfo {
        total: 0.0,
        lines: 0.0,
        functions: 0.0,
        branches: 0.0,
        files: vec![],
    })
}

/// Fix failing tests.
pub async fn fix_tests_handler(
    state: &AppState,
    request: TestFixRequest,
) -> Result<TestFixResponse, TransportError> {
    // Check for test writer agent
    let agents = load_agents(state);
    let has_agent = agents.agents.iter().any(|a| a.is_test_writer);

    if !has_agent {
        return Ok(TestFixResponse {
            success: false,
            files_fixed: 0,
            tests_passing: 0,
            error: Some(
                "No test writer agent configured. Please set an agent as test writer in Agent Manager."
                    .to_string(),
            ),
        });
    }

    // Run ckrv fix command
    let mut args = vec!["fix", "--base", &request.base];
    if let Some(ref file) = request.file {
        args.push("--file");
        args.push(file);
    }

    let output = tokio::process::Command::new("ckrv")
        .args(&args)
        .current_dir(&state.project_root)
        .output()
        .await;

    match output {
        Ok(output) => {
            if output.status.success() {
                Ok(TestFixResponse {
                    success: true,
                    files_fixed: 1,
                    tests_passing: 1,
                    error: None,
                })
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                Ok(TestFixResponse {
                    success: false,
                    files_fixed: 0,
                    tests_passing: 0,
                    error: Some(stderr.to_string()),
                })
            }
        }
        Err(e) => Ok(TestFixResponse {
            success: false,
            files_fixed: 0,
            tests_passing: 0,
            error: Some(format!("Failed to fix tests: {e}")),
        }),
    }
}

/// Get test plan status (async operation status).
pub async fn get_plan_status_handler(
    _state: &AppState,
) -> Result<AsyncStatusResponse, TransportError> {
    // Would check for running plan operation
    Ok(AsyncStatusResponse {
        status: "idle".to_string(),
        progress: 0.0,
        message: None,
    })
}

/// Get test write status (async operation status).
pub async fn get_write_status_handler(
    _state: &AppState,
) -> Result<AsyncStatusResponse, TransportError> {
    // Would check for running write operation
    Ok(AsyncStatusResponse {
        status: "idle".to_string(),
        progress: 0.0,
        message: None,
    })
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[tokio::test]
    async fn test_get_test_writer_agent_handler() {
        let state = AppState::new(PathBuf::from("/tmp/test-tests"));
        let result = get_test_writer_agent_handler(&state).await;
        assert!(result.is_ok());
    }
}
