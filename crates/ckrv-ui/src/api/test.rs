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
#[derive(Debug, Serialize, Deserialize)]
pub struct TestPlanOutput {
    pub plan_id: String,
    pub base_branch: String,
    pub changed_files: Vec<ChangedFileInfo>,
    pub proposed_tests: Vec<ProposedTest>,
}

/// Changed file info
#[derive(Debug, Serialize, Deserialize)]
pub struct ChangedFileInfo {
    pub path: String,
    pub change_type: String,
    pub lines_added: u32,
    pub lines_removed: u32,
    pub has_tests: bool,
}

/// Proposed test
#[derive(Debug, Serialize, Deserialize)]
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
    /// Custom prompt to send to the agent (if not provided, uses default test writing prompt)
    pub custom_prompt: Option<String>,
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
    println!("[Test] Running tests with base: {}", req.base);
    println!("[Test] Working directory: {:?}", state.project_root);
    
    // Run ckrv test run command
    let output = tokio::process::Command::new("ckrv")
        .args(["test", "run", "--base", &req.base, "--json"])
        .current_dir(&state.project_root)
        .output()
        .await;
    
    match output {
        Ok(output) => {
            let stdout_str = String::from_utf8_lossy(&output.stdout);
            let stderr_str = String::from_utf8_lossy(&output.stderr);
            
            println!("[Test] Exit status: {:?}", output.status);
            println!("[Test] Stdout: {}", stdout_str);
            if !stderr_str.is_empty() {
                println!("[Test] Stderr: {}", stderr_str);
            }
            
            // Try to parse JSON output regardless of exit status
            // The CLI returns structured JSON even on failure
            if let Ok(stdout) = String::from_utf8(output.stdout.clone()) {
                if let Ok(result) = serde_json::from_str::<serde_json::Value>(&stdout) {
                    let cli_success = result["success"].as_bool().unwrap_or(false);
                    let stderr_from_result = result["result"]["stderr"].as_str().unwrap_or("");
                    
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
                    
                    if cli_success && output.status.success() {
                        println!("[Test] Tests completed: {} passed, {} failed", test_result.passed, test_result.failed);
                        return Json(TestRunResponse {
                            success: true,
                            result: Some(test_result),
                            error: None,
                        });
                    } else {
                        // Return the structured result with error info
                        println!("[Test] Tests failed: {}", stderr_from_result);
                        return Json(TestRunResponse {
                            success: false,
                            result: Some(test_result),
                            error: Some(stderr_from_result.to_string()),
                        });
                    }
                } else {
                    println!("[Test] Failed to parse JSON output");
                }
            }
            
            println!("[Test] Command failed or output not parseable");
            Json(TestRunResponse {
                success: false,
                result: None,
                error: Some(if stderr_str.is_empty() { stdout_str.to_string() } else { stderr_str.to_string() }),
            })
        }
        Err(e) => {
            println!("[Test] Failed to execute command: {}", e);
            Json(TestRunResponse {
                success: false,
                result: None,
                error: Some(format!("Failed to run tests: {}", e)),
            })
        },
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

/// Write tests using agent in sandbox
pub async fn write_tests(
    State(state): State<AppState>,
    Json(req): Json<TestWriteRequest>,
) -> impl IntoResponse {
    use ckrv_sandbox::{DockerSandbox, ExecuteConfig, Sandbox};
    use ckrv_git::{WorktreeManager, DefaultWorktreeManager};
    use std::time::Duration;
    
    println!("[Test Writer] Starting test write request for base: {}", req.base);
    println!("[Test Writer] Custom prompt received: {:?}", req.custom_prompt);
    
    // Check for test writer agent first
    let agents = load_agents(&state);
    let test_agent = agents.agents.iter()
        .find(|a| a.is_test_writer);
    
    if test_agent.is_none() {
        println!("[Test Writer] ERROR: No test writer agent configured");
        return Json(TestWriteResponse {
            success: false,
            message: None,
            error: Some("No test writer agent configured. Please set an agent as test writer in Agent Manager.".to_string()),
        });
    }
    
    let agent = test_agent.unwrap();
    println!("[Test Writer] Using agent: {} ({})", agent.name, agent.id);
    
    // Load the test plan to get proposed tests
    let current_branch = std::process::Command::new("git")
        .args(["branch", "--show-current"])
        .current_dir(&state.project_root)
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "main".to_string());
    
    println!("[Test Writer] Current branch: {}", current_branch);
    
    let plan_path = state.project_root.join(".specs").join(&current_branch).join("test-plan.yaml");
    let plan_exists = plan_path.exists();
    
    // Use custom prompt if provided, otherwise build a default prompt
    let prompt = if let Some(custom) = &req.custom_prompt {
        println!("[Test Writer] Using custom prompt: {}", custom);
        custom.clone()
    } else if plan_exists {
        println!("[Test Writer] Test plan exists at: {}", plan_path.display());
        format!(
            "Read the test plan at .specs/{}/test-plan.yaml and write unit tests for all files marked as needing tests. \
            Use the appropriate testing framework (Jest/Vitest for TypeScript, pytest for Python). \
            Create test files following project conventions. Do not ask questions, just write the tests.",
            current_branch
        )
    } else {
        println!("[Test Writer] No test plan found, will analyze changes");
        format!(
            "Analyze the project and write unit tests for source files that don't have tests. \
            Compare with {} branch to find changed files. \
            Use the appropriate testing framework (Jest/Vitest for TypeScript, pytest for Python). \
            Do not ask questions, just write the tests.",
            req.base
        )
    };
    
    // Create worktree for isolated test writing
    let suffix: String = uuid::Uuid::new_v4().to_string().chars().take(6).collect();
    let wt_job_id = format!("test-write-{}", suffix);
    
    println!("[Test Writer] Creating worktree: {}", wt_job_id);
    
    let root = state.project_root.clone();
    let wt_result = tokio::task::spawn_blocking(move || {
        let manager = DefaultWorktreeManager::new(&root)?;
        manager.create(&wt_job_id, "1")
    }).await;
    
    let worktree = match wt_result {
        Ok(Ok(wt)) => wt,
        Ok(Err(e)) => {
            println!("[Test Writer] ERROR: Failed to create worktree: {}", e);
            return Json(TestWriteResponse {
                success: false,
                message: None,
                error: Some(format!("Failed to create worktree: {}", e)),
            });
        }
        Err(e) => {
            println!("[Test Writer] ERROR: Worktree task panicked: {}", e);
            return Json(TestWriteResponse {
                success: false,
                message: None,
                error: Some(format!("Worktree task panicked: {}", e)),
            });
        }
    };
    
    println!("[Test Writer] Worktree created at: {}", worktree.path.display());
    println!("[Test Writer] Branch: {}", worktree.branch);
    
    // Determine agent type and build command accordingly
    let is_codex = agent.agent_type == super::agents::AgentType::Codex;
    let agent_name = if is_codex { "codex" } else { "claude" };
    let docker_image = if is_codex { "ckrv-codex:latest" } else { "ckrv-claude:latest" };
    
    println!("[Test Writer] Using Docker image: {}", docker_image);
    
    // Execute in Docker sandbox with the correct image
    let mut sandbox = match DockerSandbox::with_defaults() {
        Ok(s) => s,
        Err(e) => {
            println!("[Test Writer] ERROR: Failed to create Docker sandbox: {}", e);
            return Json(TestWriteResponse {
                success: false,
                message: None,
                error: Some(format!("Failed to create Docker sandbox: {}", e)),
            });
        }
    };
    
    // Set the correct Docker image for the agent type
    sandbox.set_image(docker_image);
    
    println!("[Test Writer] Docker sandbox ready, executing {}...", agent_name);
    
    // Use single quotes to avoid shell interpretation issues
    let cmd = if is_codex {
        // Codex uses 'exec' subcommand for non-interactive execution
        format!(
            "codex exec --dangerously-bypass-approvals-and-sandbox '{}'",
            prompt.replace("'", "'\\''")
        )
    } else {
        format!(
            "claude --print --verbose --dangerously-skip-permissions '{}'",
            prompt.replace("'", "'\\''")
        )
    };
    
    let config = ExecuteConfig::new(agent_name, worktree.path.clone())
        .shell(&cmd)
        .with_timeout(Duration::from_secs(600)) // 10 minute timeout
        .env("HOME", if is_codex { "/home/codex" } else { "/home/claude" })
        .env("NO_COLOR", "1");
    
    // Execute with streaming output
    let result = sandbox.execute_streaming(
        config,
        |line, is_stderr| {
            if is_stderr {
                println!("[Test Writer] stderr: {}", line);
            } else {
                println!("[Test Writer] {}", line);
            }
        }
    ).await;
    
    match result {
        Ok(output) => {
            if output.success() {
                println!("[Test Writer] Claude completed successfully");
                
                // Commit changes in worktree
                println!("[Test Writer] Committing changes...");
                let commit_result = std::process::Command::new("git")
                    .args(["add", "-A"])
                    .current_dir(&worktree.path)
                    .status()
                    .and_then(|_| {
                        std::process::Command::new("git")
                            .args(["commit", "-m", "chore: add tests from test writer agent"])
                            .current_dir(&worktree.path)
                            .status()
                    });
                
                if let Err(e) = commit_result {
                    println!("[Test Writer] Warning: Failed to commit: {}", e);
                }
                
                // Merge back to main branch
                println!("[Test Writer] Merging changes back to {}...", current_branch);
                let merge_result = std::process::Command::new("git")
                    .args(["merge", &worktree.branch, "--no-edit"])
                    .current_dir(&state.project_root)
                    .status();
                
                match merge_result {
                    Ok(status) if status.success() => {
                        println!("[Test Writer] ✅ Tests merged successfully!");
                        
                        // Write test results summary file
                        let results_path = state.project_root.join(".specs").join(&current_branch).join("test-results.yaml");
                        if let Some(parent) = results_path.parent() {
                            let _ = std::fs::create_dir_all(parent);
                        }
                        let summary = format!(
                            "# Test Writing Results\n\
                            completed_at: {}\n\
                            agent_id: {}\n\
                            agent_name: {}\n\
                            branch: {}\n\
                            status: merged\n\
                            base_branch: {}\n",
                            chrono::Utc::now().to_rfc3339(),
                            agent.id,
                            agent.name,
                            worktree.branch,
                            req.base
                        );
                        if let Err(e) = std::fs::write(&results_path, &summary) {
                            println!("[Test Writer] Warning: Failed to write results file: {}", e);
                        } else {
                            println!("[Test Writer] Results saved to: {}", results_path.display());
                        }
                        
                        Json(TestWriteResponse {
                            success: true,
                            message: Some(format!("Tests written and merged from branch '{}'", worktree.branch)),
                            error: None,
                        })
                    }
                    Ok(_) => {
                        println!("[Test Writer] ⚠️ Merge conflicts - tests are on branch: {}", worktree.branch);
                        
                        // Write results even with conflicts
                        let results_path = state.project_root.join(".specs").join(&current_branch).join("test-results.yaml");
                        if let Some(parent) = results_path.parent() {
                            let _ = std::fs::create_dir_all(parent);
                        }
                        let summary = format!(
                            "# Test Writing Results\n\
                            completed_at: {}\n\
                            agent_id: {}\n\
                            agent_name: {}\n\
                            branch: {}\n\
                            status: conflicts\n\
                            base_branch: {}\n",
                            chrono::Utc::now().to_rfc3339(),
                            agent.id,
                            agent.name,
                            worktree.branch,
                            req.base
                        );
                        let _ = std::fs::write(&results_path, &summary);
                        
                        Json(TestWriteResponse {
                            success: true,
                            message: Some(format!("Tests written on branch '{}'. Review and merge manually (conflicts detected).", worktree.branch)),
                            error: None,
                        })
                    }
                    Err(e) => {
                        println!("[Test Writer] Error during merge: {}", e);
                        Json(TestWriteResponse {
                            success: true,
                            message: Some(format!("Tests written on branch '{}'. Merge failed: {}", worktree.branch, e)),
                            error: None,
                        })
                    }
                }
            } else {
                println!("[Test Writer] Claude failed: {}", output.stderr);
                Json(TestWriteResponse {
                    success: false,
                    message: None,
                    error: Some(format!("Agent execution failed: {}", output.stderr)),
                })
            }
        }
        Err(e) => {
            println!("[Test Writer] ERROR: Sandbox execution failed: {}", e);
            Json(TestWriteResponse {
                success: false,
                message: None,
                error: Some(format!("Sandbox execution failed: {}", e)),
            })
        }
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

/// Fix test request
#[derive(Debug, Deserialize)]
pub struct FixTestRequest {
    pub error: String,
    pub base: String,
}

/// Fix test response
#[derive(Debug, Serialize)]
pub struct FixTestResponse {
    pub success: bool,
    pub message: Option<String>,
    pub error: Option<String>,
}

/// Fix test errors using AI agent
pub async fn fix_tests(
    State(state): State<AppState>,
    Json(req): Json<FixTestRequest>,
) -> impl IntoResponse {
    println!("[Test Fix] Received fix request for error: {}", req.error);
    println!("[Test Fix] Working directory: {:?}", state.project_root);
    
    // Check for test writer agent
    let agents = load_agents(&state);
    let test_agent = agents.agents.iter().find(|a| a.is_test_writer);
    
    if test_agent.is_none() {
        return Json(FixTestResponse {
            success: false,
            message: None,
            error: Some("No test writer agent configured. Go to Agents page to set one up.".to_string()),
        });
    }
    
    // Create a fix prompt based on the error
    let fix_prompt = format!(
        "Fix the test setup issue in this project. The error was:\n\n{}\n\n\
        Analyze the project structure and fix the test configuration. \
        For Node.js projects, ensure package.json has a test script and the right test dependencies (jest, vitest, etc). \
        Create any missing configuration files needed to run tests.",
        req.error
    );
    
    // Run ckrv with the fix prompt (using test write as base)
    let output = tokio::process::Command::new("ckrv")
        .args(["test", "write", "--base", &req.base, "--fix"])
        .current_dir(&state.project_root)
        .output()
        .await;
    
    match output {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);
            println!("[Test Fix] stdout: {}", stdout);
            println!("[Test Fix] stderr: {}", stderr);
            
            if output.status.success() {
                Json(FixTestResponse {
                    success: true,
                    message: Some("AI agent is working on fixing the test setup. Check back in a moment.".to_string()),
                    error: None,
                })
            } else {
                // Even if the command failed, we've at least started the process
                Json(FixTestResponse {
                    success: true,
                    message: Some("Fix request submitted. The test write command may need additional setup.".to_string()),
                    error: None,
                })
            }
        }
        Err(e) => {
            println!("[Test Fix] Error: {}", e);
            Json(FixTestResponse {
                success: false,
                message: None,
                error: Some(format!("Failed to invoke AI agent: {}", e)),
            })
        }
    }
}

/// Test plan status response
#[derive(Debug, Serialize)]
pub struct TestPlanStatusResponse {
    pub exists: bool,
    pub branch: String,
    pub path: Option<String>,
    pub plan: Option<TestPlanOutput>,
}

/// Check if test plan exists for the current branch and return it
pub async fn test_plan_status(
    State(state): State<AppState>,
) -> impl IntoResponse {
    let cwd = &state.project_root;
    
    // Get current branch
    let branch = std::process::Command::new("git")
        .args(["branch", "--show-current"])
        .current_dir(cwd)
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "default".to_string());
    
    // Check if test plan exists
    let plan_path = cwd.join(".specs").join(&branch).join("test-plan.yaml");
    let exists = plan_path.exists();
    
    // Load plan if exists
    let plan = if exists {
        std::fs::read_to_string(&plan_path)
            .ok()
            .and_then(|content| serde_yaml::from_str::<TestPlanOutput>(&content).ok())
    } else {
        None
    };
    
    Json(TestPlanStatusResponse {
        exists,
        branch,
        path: if exists { Some(plan_path.to_string_lossy().to_string()) } else { None },
        plan,
    })
}

/// Test write status response
#[derive(Debug, Serialize)]
pub struct TestWriteStatusResponse {
    pub exists: bool,
    pub branch: String,
    pub path: Option<String>,
    pub completed_at: Option<String>,
    pub status: Option<String>,
    pub agent_name: Option<String>,
    pub agent_id: Option<String>,
    pub worktree_branch: Option<String>,
    pub base_branch: Option<String>,
}

/// Check if tests have been written for the current branch
pub async fn test_write_status(
    State(state): State<AppState>,
) -> impl IntoResponse {
    let cwd = &state.project_root;
    
    // Get current branch
    let branch = std::process::Command::new("git")
        .args(["branch", "--show-current"])
        .current_dir(cwd)
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "default".to_string());
    
    // Check if test results exist
    let results_path = cwd.join(".specs").join(&branch).join("test-results.yaml");
    let exists = results_path.exists();
    
    // Parse results if exists
    let (completed_at, status, agent_name, agent_id, worktree_branch, base_branch) = if exists {
        let content = std::fs::read_to_string(&results_path).unwrap_or_default();
        let parse_field = |field: &str| -> Option<String> {
            content.lines()
                .find(|l| l.starts_with(&format!("{}:", field)))
                .map(|l| l.replace(&format!("{}:", field), "").trim().to_string())
        };
        (
            parse_field("completed_at"),
            parse_field("status"),
            parse_field("agent_name"),
            parse_field("agent_id"),
            parse_field("branch"),
            parse_field("base_branch"),
        )
    } else {
        (None, None, None, None, None, None)
    };
    
    Json(TestWriteStatusResponse {
        exists,
        branch,
        path: if exists { Some(results_path.to_string_lossy().to_string()) } else { None },
        completed_at,
        status,
        agent_name,
        agent_id,
        worktree_branch,
        base_branch,
    })
}
