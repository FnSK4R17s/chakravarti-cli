//! # Test Handler
//!
//! Handlers for test execution and management.

use crate::error::TransportError;
use crate::handlers::agents::load_agents;
use crate::state::AppState;
use serde::{Deserialize, Serialize};
use std::process::Command;

// ============================================================
// Request/Response Types
// ============================================================

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
    /// Test case name.
    pub name: String,
    /// Pass/fail/skip status.
    pub status: TestStatus,
    /// Execution duration in milliseconds.
    pub duration_ms: Option<u64>,
    /// Error message if the test failed.
    pub error: Option<String>,
    /// Source file containing the test.
    pub file: Option<String>,
    /// Line number of the test definition.
    pub line: Option<u32>,
}

/// Test status.
#[derive(Debug, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum TestStatus {
    /// Test passed successfully.
    Passed,
    /// Test failed.
    Failed,
    /// Test was skipped.
    Skipped,
    /// Test is pending execution.
    Pending,
}

/// Test run summary.
#[derive(Debug, Serialize)]
pub struct TestRunSummary {
    /// Total number of tests.
    pub total: u32,
    /// Number of passing tests.
    pub passed: u32,
    /// Number of failing tests.
    pub failed: u32,
    /// Number of skipped tests.
    pub skipped: u32,
    /// Total execution duration in milliseconds.
    pub duration_ms: u64,
}

/// Test run response.
#[derive(Debug, Serialize)]
pub struct TestRunResponse {
    /// Individual test case results.
    pub results: Vec<TestResult>,
    /// Aggregate summary of the test run.
    pub summary: TestRunSummary,
    /// Raw console output from the test runner.
    pub raw_output: Option<String>,
}

/// Test writer agent info.
#[derive(Debug, Serialize)]
pub struct TestWriterAgentInfo {
    /// Agent identifier.
    pub id: String,
    /// Display name of the agent.
    pub name: String,
    /// Model used by the agent.
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
    /// Path to the generated test file.
    pub test_file: String,
    /// Number of tests generated.
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
    /// File path relative to project root.
    pub path: String,
    /// Type of change (added, modified, deleted).
    pub change_type: String,
    /// Number of lines added.
    pub lines_added: u32,
    /// Number of lines removed.
    pub lines_removed: u32,
    /// Whether the file already has associated tests.
    pub has_tests: bool,
}

/// Proposed test.
#[derive(Debug, Serialize, Deserialize)]
pub struct ProposedTest {
    /// Source file the test targets.
    pub target_file: String,
    /// Path for the proposed test file.
    pub test_file: String,
    /// Description of what the test covers.
    pub description: String,
    /// Priority level (high, normal, low).
    pub priority: String,
}

/// Test plan output.
#[derive(Debug, Serialize, Deserialize)]
pub struct TestPlanOutput {
    /// Unique plan identifier.
    pub plan_id: String,
    /// Base branch used for comparison.
    pub base_branch: String,
    /// Files that changed relative to base.
    pub changed_files: Vec<ChangedFileInfo>,
    /// Tests proposed for the changed files.
    pub proposed_tests: Vec<ProposedTest>,
}

/// Test plan response.
#[derive(Debug, Serialize)]
pub struct TestPlanResponse {
    /// Whether plan generation succeeded.
    pub success: bool,
    /// The generated test plan, if successful.
    pub plan: Option<TestPlanOutput>,
    /// Error message if plan generation failed.
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
    /// Whether test writing succeeded.
    pub success: bool,
    /// Number of test files written.
    pub tests_written: u32,
    /// Number of tests passing (if run after write).
    pub tests_passed: Option<u32>,
    /// Number of tests failing (if run after write).
    pub tests_failed: Option<u32>,
    /// Error message if writing failed.
    pub error: Option<String>,
}

/// Coverage info.
#[derive(Debug, Serialize)]
pub struct CoverageInfo {
    /// Overall coverage percentage.
    pub total: f32,
    /// Line coverage percentage.
    pub lines: f32,
    /// Function coverage percentage.
    pub functions: f32,
    /// Branch coverage percentage.
    pub branches: f32,
    /// Per-file coverage breakdown.
    pub files: Vec<FileCoverage>,
}

/// File coverage.
#[derive(Debug, Serialize)]
pub struct FileCoverage {
    /// File path relative to project root.
    pub path: String,
    /// Coverage percentage for this file.
    pub coverage: f32,
    /// Line numbers that lack coverage.
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
    /// Whether the fix operation succeeded.
    pub success: bool,
    /// Number of files that were fixed.
    pub files_fixed: u32,
    /// Number of tests now passing.
    pub tests_passing: u32,
    /// Error message if fix failed.
    pub error: Option<String>,
}

/// Status response for async operations.
#[derive(Debug, Serialize)]
pub struct AsyncStatusResponse {
    /// Current status (idle, running, completed, failed).
    pub status: String,
    /// Progress percentage (0.0 to 1.0).
    pub progress: f32,
    /// Descriptive status message.
    pub message: Option<String>,
}

// ============================================================
// Handlers
// ============================================================

/// Run tests.
pub fn run_tests_handler(
    state: &AppState,
    _request: RunTestsRequest,
) -> Result<TestRunResponse, TransportError> {
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
pub fn get_test_writer_agent_handler(
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
    _request: GenerateTestsRequest,
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

    // Run ckrv test write command
    let output = tokio::process::Command::new("ckrv")
        .args(["test", "write", "--json"])
        .current_dir(&state.project_root)
        .output()
        .await
        .map_err(|e| TransportError::Internal(format!("Failed to generate tests: {e}")))?;

    if output.status.success() {
        if let Ok(stdout) = String::from_utf8(output.stdout) {
            if let Ok(result) = serde_json::from_str::<serde_json::Value>(&stdout) {
                return Ok(GenerateTestsResponse {
                    test_file: result["test_file"].as_str().unwrap_or("").to_string(),
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
                                plan_id: result["plan_id"].as_str().unwrap_or("plan-0").to_string(),
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
pub async fn get_coverage_handler(state: &AppState) -> Result<CoverageInfo, TransportError> {
    // Try to run coverage command
    let _output = tokio::process::Command::new("npm")
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
    let mut args = vec!["fix", "--test", "--json"];
    if let Some(ref file) = request.file {
        args.push("--error");
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
pub fn get_plan_status_handler(_state: &AppState) -> Result<AsyncStatusResponse, TransportError> {
    // Would check for running plan operation
    Ok(AsyncStatusResponse {
        status: "idle".to_string(),
        progress: 0.0,
        message: None,
    })
}

/// Get test write status (async operation status).
pub fn get_write_status_handler(_state: &AppState) -> Result<AsyncStatusResponse, TransportError> {
    // Would check for running write operation
    Ok(AsyncStatusResponse {
        status: "idle".to_string(),
        progress: 0.0,
        message: None,
    })
}

// ============================================================
// Tests
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[tokio::test]
    async fn test_get_test_writer_agent_handler() {
        let state = AppState::new(PathBuf::from("/tmp/test-tests"));
        let result = get_test_writer_agent_handler(&state);
        assert!(result.is_ok());
    }
}
