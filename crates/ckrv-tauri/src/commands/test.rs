//! Test commands for Tauri IPC.

// ============================================================
// Imports
// ============================================================

use crate::SharedState;
use ckrv_transport::handlers::test::{
    create_test_plan_handler, fix_tests_handler, generate_tests_handler, get_coverage_handler,
    get_plan_status_handler, get_test_writer_agent_handler, get_write_status_handler,
    run_tests_handler, write_tests_handler, GenerateTestsRequest, GenerateTestsResponse,
    RunTestsRequest, TestFixRequest, TestFixResponse, TestPlanRequest, TestWriteRequest,
    TestWriterAgentInfo,
};
use serde::Serialize;
use tauri::State;

// ============================================================
// Types
// ============================================================

/// Response wrapper for test agent.
#[derive(Serialize)]
pub struct TestAgentWrapped {
    /// Configured test writer agent, if any.
    agent: Option<TestWriterAgentInfo>,
}

/// Response wrapper for running tests to match frontend expectations.
#[derive(Serialize)]
pub struct TestRunWrapped {
    /// Whether all tests passed.
    success: bool,
    /// Test result data if execution completed.
    result: Option<TestRunResponseData>,
    /// Error message if execution failed.
    error: Option<String>,
}

/// Test result data matching frontend expectations.
#[derive(Serialize)]
pub struct TestRunResponseData {
    /// Total number of tests.
    total: u32,
    /// Number of passing tests.
    passed: u32,
    /// Number of failing tests.
    failed: u32,
    /// Number of skipped tests.
    skipped: u32,
    /// Total execution time in milliseconds.
    duration_ms: u64,
    /// Details of each failing test.
    failures: Vec<TestFailureData>,
    /// Test framework used (e.g., "vitest").
    framework: String,
}

/// Test failure data.
#[derive(Serialize)]
pub struct TestFailureData {
    /// Test case name.
    name: String,
    /// File containing the failing test.
    file: String,
    /// Line number of the failure.
    line: Option<u32>,
    /// Failure message.
    message: String,
    /// Captured stdout output.
    stdout: Option<String>,
    /// Captured stderr output.
    stderr: Option<String>,
}

/// Response wrapper for test plan to match frontend expectations.
#[derive(Serialize)]
pub struct TestPlanWrapped {
    /// Whether the plan was created successfully.
    success: bool,
    /// Test plan data if creation succeeded.
    plan: Option<TestPlanData>,
    /// Error message if creation failed.
    error: Option<String>,
}

/// Test plan data matching frontend expectations.
#[derive(Serialize)]
pub struct TestPlanData {
    /// Unique identifier for the test plan.
    plan_id: String,
    /// Base branch used for diff analysis.
    base_branch: String,
    /// Files that changed relative to base.
    changed_files: Vec<ChangedFileData>,
    /// Tests proposed by the AI agent.
    proposed_tests: Vec<ProposedTestData>,
}

/// Information about a file changed relative to the base branch.
#[derive(Serialize)]
pub struct ChangedFileData {
    /// File path relative to project root.
    path: String,
    /// Type of change (added, modified, deleted).
    change_type: String,
    /// Number of lines added.
    lines_added: u32,
    /// Number of lines removed.
    lines_removed: u32,
    /// Whether the file already has tests.
    has_tests: bool,
}

/// A test proposed by the AI agent.
#[derive(Serialize)]
pub struct ProposedTestData {
    /// File being tested.
    target_file: String,
    /// Path for the test file.
    test_file: String,
    /// Description of what the test covers.
    description: String,
    /// Priority level (high, medium, low).
    priority: String,
}

/// Response wrapper for test write to match frontend expectations.
#[derive(Serialize)]
pub struct TestWriteWrapped {
    /// Whether tests were written successfully.
    success: bool,
    /// Success message with count of tests written.
    message: Option<String>,
    /// Error message if writing failed.
    error: Option<String>,
}

/// Response wrapper for coverage to match frontend expectations.
#[derive(Serialize)]
pub struct CoverageWrapped {
    /// Whether coverage data was collected successfully.
    success: bool,
    /// Coverage data if collection succeeded.
    coverage: Option<CoverageData>,
    /// Error message if collection failed.
    error: Option<String>,
}

/// Coverage statistics data.
#[derive(Serialize)]
pub struct CoverageData {
    /// Total number of files analyzed.
    total: u32,
    /// Number of files with test coverage.
    covered: u32,
    /// Number of files without test coverage.
    uncovered: u32,
    /// Overall coverage percentage.
    coverage_percent: f32,
}

/// Response wrapper for plan status.
#[derive(Serialize)]
pub struct PlanStatusWrapped {
    /// Whether a test plan exists.
    exists: bool,
    /// Test plan data if it exists.
    plan: Option<TestPlanData>,
}

/// Response wrapper for write status.
#[derive(Serialize)]
pub struct WriteStatusWrapped {
    /// Whether a test write session exists.
    exists: bool,
    /// When the write completed.
    completed_at: Option<String>,
    /// Current write status.
    status: Option<String>,
    /// Agent that performed the write.
    agent_name: Option<String>,
    /// Git branch used for the worktree.
    worktree_branch: Option<String>,
    /// Base branch for comparison.
    base_branch: Option<String>,
}

// ============================================================
// Handlers
// ============================================================

/// Get test writer agent.
#[tauri::command]
pub async fn get_test_agent(state: State<'_, SharedState>) -> Result<TestAgentWrapped, String> {
    let app_state = state.read().await;
    get_test_writer_agent_handler(&app_state)
        .map(|agent| TestAgentWrapped { agent })
        .map_err(|e| e.to_string())
}

/// Run tests.
#[tauri::command]
pub async fn run_tests(
    state: State<'_, SharedState>,
    base: String,
    pattern: Option<String>,
    framework: Option<String>,
    watch: Option<bool>,
) -> Result<TestRunWrapped, String> {
    let app_state = state.read().await;
    match run_tests_handler(
        &app_state,
        RunTestsRequest {
            base,
            pattern,
            framework,
            watch: watch.unwrap_or(false),
        },
    ) {
        Ok(response) => {
            let result = TestRunResponseData {
                total: response.summary.total,
                passed: response.summary.passed,
                failed: response.summary.failed,
                skipped: response.summary.skipped,
                duration_ms: response.summary.duration_ms,
                failures: response
                    .results
                    .iter()
                    .filter(|r| {
                        matches!(r.status, ckrv_transport::handlers::test::TestStatus::Failed)
                    })
                    .map(|r| TestFailureData {
                        name: r.name.clone(),
                        file: r.file.clone().unwrap_or_default(),
                        line: r.line,
                        message: r.error.clone().unwrap_or_default(),
                        stdout: None,
                        stderr: None,
                    })
                    .collect(),
                framework: "vitest".to_string(),
            };
            Ok(TestRunWrapped {
                success: response.summary.failed == 0,
                result: Some(result),
                error: None,
            })
        }
        Err(e) => Ok(TestRunWrapped {
            success: false,
            result: None,
            error: Some(e.to_string()),
        }),
    }
}

/// Generate tests for a file.
#[tauri::command]
pub async fn generate_tests(
    state: State<'_, SharedState>,
    file: String,
    framework: Option<String>,
) -> Result<GenerateTestsResponse, String> {
    let app_state = state.read().await;
    generate_tests_handler(&app_state, GenerateTestsRequest { file, framework })
        .await
        .map_err(|e| e.to_string())
}

/// Create test plan.
#[tauri::command]
pub async fn plan_tests(
    state: State<'_, SharedState>,
    base: String,
) -> Result<TestPlanWrapped, String> {
    let app_state = state.read().await;
    match create_test_plan_handler(&app_state, TestPlanRequest { base }).await {
        Ok(response) => {
            if response.success {
                if let Some(plan) = response.plan {
                    return Ok(TestPlanWrapped {
                        success: true,
                        plan: Some(TestPlanData {
                            plan_id: plan.plan_id,
                            base_branch: plan.base_branch,
                            changed_files: plan
                                .changed_files
                                .into_iter()
                                .map(|f| ChangedFileData {
                                    path: f.path,
                                    change_type: f.change_type,
                                    lines_added: f.lines_added,
                                    lines_removed: f.lines_removed,
                                    has_tests: f.has_tests,
                                })
                                .collect(),
                            proposed_tests: plan
                                .proposed_tests
                                .into_iter()
                                .map(|t| ProposedTestData {
                                    target_file: t.target_file,
                                    test_file: t.test_file,
                                    description: t.description,
                                    priority: t.priority,
                                })
                                .collect(),
                        }),
                        error: None,
                    });
                }
            }
            Ok(TestPlanWrapped {
                success: false,
                plan: None,
                error: response.error,
            })
        }
        Err(e) => Ok(TestPlanWrapped {
            success: false,
            plan: None,
            error: Some(e.to_string()),
        }),
    }
}

/// Write tests based on plan.
#[tauri::command]
pub async fn write_tests(
    state: State<'_, SharedState>,
    base: String,
    run: Option<bool>,
) -> Result<TestWriteWrapped, String> {
    let app_state = state.read().await;
    match write_tests_handler(
        &app_state,
        TestWriteRequest {
            base,
            run: run.unwrap_or(false),
        },
    )
    .await
    {
        Ok(response) => Ok(TestWriteWrapped {
            success: response.success,
            message: if response.success {
                Some(format!("Written {} tests", response.tests_written))
            } else {
                None
            },
            error: response.error,
        }),
        Err(e) => Ok(TestWriteWrapped {
            success: false,
            message: None,
            error: Some(e.to_string()),
        }),
    }
}

/// Get test coverage.
#[tauri::command]
pub async fn get_coverage(state: State<'_, SharedState>) -> Result<CoverageWrapped, String> {
    let app_state = state.read().await;
    match get_coverage_handler(&app_state).await {
        Ok(info) => {
            let total_files = info.files.len() as u32;
            let covered = info.files.iter().filter(|f| f.coverage > 0.0).count() as u32;
            Ok(CoverageWrapped {
                success: true,
                coverage: Some(CoverageData {
                    total: total_files,
                    covered,
                    uncovered: total_files.saturating_sub(covered),
                    coverage_percent: info.total,
                }),
                error: None,
            })
        }
        Err(e) => Ok(CoverageWrapped {
            success: false,
            coverage: None,
            error: Some(e.to_string()),
        }),
    }
}

/// Fix failing tests.
#[tauri::command]
pub async fn fix_tests(
    state: State<'_, SharedState>,
    base: String,
    file: Option<String>,
) -> Result<TestFixResponse, String> {
    let app_state = state.read().await;
    fix_tests_handler(&app_state, TestFixRequest { base, file })
        .await
        .map_err(|e| e.to_string())
}

/// Get test plan status.
#[tauri::command]
pub async fn get_plan_status(state: State<'_, SharedState>) -> Result<PlanStatusWrapped, String> {
    let app_state = state.read().await;
    match get_plan_status_handler(&app_state) {
        Ok(status) => {
            // Check if status indicates a plan exists
            let exists = status.status != "idle";
            Ok(PlanStatusWrapped { exists, plan: None })
        }
        Err(_) => Ok(PlanStatusWrapped {
            exists: false,
            plan: None,
        }),
    }
}

/// Get test write status.
#[tauri::command]
pub async fn get_write_status(state: State<'_, SharedState>) -> Result<WriteStatusWrapped, String> {
    let app_state = state.read().await;
    match get_write_status_handler(&app_state) {
        Ok(status) => {
            let exists = status.status != "idle";
            Ok(WriteStatusWrapped {
                exists,
                completed_at: None,
                status: Some(status.status),
                agent_name: None,
                worktree_branch: None,
                base_branch: None,
            })
        }
        Err(_) => Ok(WriteStatusWrapped {
            exists: false,
            completed_at: None,
            status: None,
            agent_name: None,
            worktree_branch: None,
            base_branch: None,
        }),
    }
}
