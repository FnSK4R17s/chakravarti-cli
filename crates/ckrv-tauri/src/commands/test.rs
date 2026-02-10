//! Test commands for Tauri IPC

use crate::SharedState;
use ckrv_transport::handlers::test::{
    create_test_plan_handler, fix_tests_handler, generate_tests_handler, get_coverage_handler,
    get_plan_status_handler, get_test_writer_agent_handler, get_write_status_handler,
    run_tests_handler, write_tests_handler, AsyncStatusResponse, CoverageInfo,
    GenerateTestsRequest, GenerateTestsResponse, RunTestsRequest, TestFixRequest, TestFixResponse,
    TestPlanRequest, TestRunResponse, TestWriteRequest, TestWriterAgentInfo,
};
use serde::Serialize;
use tauri::State;

/// Response wrapper for test agent.
#[derive(Serialize)]
pub struct TestAgentWrapped {
    agent: Option<TestWriterAgentInfo>,
}

/// Response wrapper for running tests to match frontend expectations.
#[derive(Serialize)]
pub struct TestRunWrapped {
    success: bool,
    result: Option<TestRunResponseData>,
    error: Option<String>,
}

/// Test result data matching frontend expectations.
#[derive(Serialize)]
pub struct TestRunResponseData {
    total: u32,
    passed: u32,
    failed: u32,
    skipped: u32,
    duration_ms: u64,
    failures: Vec<TestFailureData>,
    framework: String,
}

/// Test failure data.
#[derive(Serialize)]
pub struct TestFailureData {
    name: String,
    file: String,
    line: Option<u32>,
    message: String,
    stdout: Option<String>,
    stderr: Option<String>,
}

/// Response wrapper for test plan to match frontend expectations.
#[derive(Serialize)]
pub struct TestPlanWrapped {
    success: bool,
    plan: Option<TestPlanData>,
    error: Option<String>,
}

/// Test plan data matching frontend expectations.
#[derive(Serialize)]
pub struct TestPlanData {
    plan_id: String,
    base_branch: String,
    changed_files: Vec<ChangedFileData>,
    proposed_tests: Vec<ProposedTestData>,
}

#[derive(Serialize)]
pub struct ChangedFileData {
    path: String,
    change_type: String,
    lines_added: u32,
    lines_removed: u32,
    has_tests: bool,
}

#[derive(Serialize)]
pub struct ProposedTestData {
    target_file: String,
    test_file: String,
    description: String,
    priority: String,
}

/// Response wrapper for test write to match frontend expectations.
#[derive(Serialize)]
pub struct TestWriteWrapped {
    success: bool,
    message: Option<String>,
    error: Option<String>,
}

/// Response wrapper for coverage to match frontend expectations.
#[derive(Serialize)]
pub struct CoverageWrapped {
    success: bool,
    coverage: Option<CoverageData>,
    error: Option<String>,
}

#[derive(Serialize)]
pub struct CoverageData {
    total: u32,
    covered: u32,
    uncovered: u32,
    coverage_percent: f32,
}

/// Response wrapper for plan status.
#[derive(Serialize)]
pub struct PlanStatusWrapped {
    exists: bool,
    plan: Option<TestPlanData>,
}

/// Response wrapper for write status.
#[derive(Serialize)]
pub struct WriteStatusWrapped {
    exists: bool,
    completed_at: Option<String>,
    status: Option<String>,
    agent_name: Option<String>,
    worktree_branch: Option<String>,
    base_branch: Option<String>,
}

/// Get test writer agent.
#[tauri::command]
pub async fn get_test_agent(state: State<'_, SharedState>) -> Result<TestAgentWrapped, String> {
    let app_state = state.read().await;
    get_test_writer_agent_handler(&app_state)
        .await
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
    )
    .await
    {
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
    match get_plan_status_handler(&app_state).await {
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
    match get_write_status_handler(&app_state).await {
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
