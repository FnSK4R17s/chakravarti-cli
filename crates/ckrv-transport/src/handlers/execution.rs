//! # Execution Handlers
//!
//! Handlers for batch execution control with in-process orchestration.
//!
//! ## Overview
//!
//! These handlers manage the execution lifecycle: starting runs, tracking
//! status, stopping runs, and managing branches. Execution runs in-process
//! using the orchestration engine with a Hub-connected event handler that
//! bridges `JobEvent` to `OrchestrationEvent` for real-time UI updates.
//!
//! ## Architecture
//!
//! ```text
//! POST /start ──> validate spec ──> register run ──> tokio::spawn
//!                                                        │
//!                                  ┌─────────────────────┘
//!                                  ▼
//!                          orchestrator.run(spec)
//!                                  │
//!                          HubEventHandler bridges
//!                          JobEvent ──> OrchestrationEvent
//!                                  │
//!                          Hub.broadcast() ──> WebSocket
//! ```
//!
//! ## See Also
//!
//! - [`crate::hub`] - Event broadcasting to WebSocket clients
//! - [`crate::state::RunRegistry`] - Run state tracking

// ============================================================
// IMPORTS
// ============================================================

use std::collections::HashSet;
use std::process::Command;
use std::sync::Arc;
use std::time::Instant;

use serde::{Deserialize, Serialize};
use tokio_util::sync::CancellationToken;
use tracing::{error, info, warn};

use ckrv_core::events::JobEvent;
use ckrv_core::EventHandler;

use crate::error::TransportError;
use crate::hub::{OrchestrationEvent, SharedHub};
use crate::state::{AppState, RunEntry, RunStatus};

// ============================================================
// Request/Response Types
// ============================================================

/// Request to start execution of a spec.
#[derive(Debug, Deserialize)]
pub struct ExecuteRequest {
    /// Spec name to execute.
    pub spec: String,
    /// Optional batch ID to execute (if not provided, executes all pending).
    pub batch_id: Option<String>,
    /// Dry run mode.
    #[serde(default)]
    pub dry_run: bool,
}

/// Current execution status.
#[derive(Debug, Serialize)]
pub struct ExecutionStatus {
    /// Whether execution is currently active.
    pub running: bool,
    /// Name of the spec being executed.
    pub spec_name: Option<String>,
    /// Current batch being processed.
    pub batch_id: Option<String>,
    /// Execution progress (0.0 to 1.0).
    pub progress: f32,
    /// Currently executing task name.
    pub current_task: Option<String>,
    /// Descriptive status message.
    pub message: Option<String>,
}

/// Response after starting execution.
#[derive(Debug, Serialize)]
pub struct ExecuteResponse {
    /// Whether execution was successfully started.
    pub started: bool,
    /// Unique identifier for this execution run.
    pub run_id: Option<String>,
    /// Descriptive message about the execution start.
    pub message: Option<String>,
}

/// Request to stop a running execution.
#[derive(Debug, Deserialize)]
pub struct StopRequest {
    /// Spec to stop execution for.
    pub spec: String,
    /// Specific run to stop (if multiple).
    pub run_id: Option<String>,
}

// ============================================================
// Hub Event Handler
// ============================================================

/// Bridges orchestration `JobEvent`s to `OrchestrationEvent`s via the Hub.
///
/// This handler is passed to the orchestrator and maps each `JobEvent` variant
/// into the corresponding `OrchestrationEvent`, broadcasting it to all
/// connected WebSocket clients. Optionally persists events to a JSONL log file.
struct HubEventHandler {
    /// Shared hub for broadcasting events.
    hub: SharedHub,
    /// Optional JSONL log writer for persistence.
    log_writer: std::sync::Mutex<Option<std::io::BufWriter<std::fs::File>>>,
}

impl HubEventHandler {
    /// Create a new hub event handler.
    fn new(hub: SharedHub) -> Self {
        Self {
            hub,
            log_writer: std::sync::Mutex::new(None),
        }
    }

    /// Create a new hub event handler with JSONL log persistence.
    fn with_log_file(hub: SharedHub, log_path: std::path::PathBuf) -> Result<Self, String> {
        if let Some(parent) = log_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create log directory: {e}"))?;
        }
        let file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_path)
            .map_err(|e| format!("Failed to open log file: {e}"))?;
        Ok(Self {
            hub,
            log_writer: std::sync::Mutex::new(Some(std::io::BufWriter::new(file))),
        })
    }

    /// Get the current timestamp as ISO 8601.
    fn now() -> String {
        chrono::Utc::now().to_rfc3339()
    }

    /// Append an event to the JSONL log file (if configured).
    fn persist_event(&self, event: &OrchestrationEvent) {
        use std::io::Write;
        if let Ok(mut guard) = self.log_writer.lock() {
            if let Some(ref mut writer) = *guard {
                if let Ok(json) = serde_json::to_string(event) {
                    let _ = writeln!(writer, "{json}");
                    let _ = writer.flush();
                }
            }
        }
    }
}

impl EventHandler for HubEventHandler {
    fn handle(&self, event: JobEvent) {
        let orch_event = match event {
            JobEvent::StepStarted { ref step_id } => {
                info!(step_id = %step_id, "Step started");
                OrchestrationEvent::StepStart {
                    step_name: step_id.clone(),
                    timestamp: Self::now(),
                }
            }
            JobEvent::StepCompleted {
                ref step_id,
                duration_ms,
            } => {
                info!(step_id = %step_id, duration_ms = duration_ms, "Step completed");
                OrchestrationEvent::StepEnd {
                    step_name: step_id.clone(),
                    timestamp: Self::now(),
                    status: "success".to_string(),
                }
            }
            JobEvent::StepFailed {
                ref step_id,
                ref error,
            } => {
                error!(step_id = %step_id, error = %error, "Step failed");
                // Emit both an error log and a step end event
                let error_log = OrchestrationEvent::Log {
                    message: format!("Step {step_id} failed: {error}"),
                    timestamp: Self::now(),
                    metadata: None,
                };
                self.persist_event(&error_log);
                self.hub.broadcast(error_log);
                OrchestrationEvent::StepEnd {
                    step_name: step_id.clone(),
                    timestamp: Self::now(),
                    status: "error".to_string(),
                }
            }
            JobEvent::StateChanged { ref state } => {
                info!(state = ?state, "Job state changed");
                OrchestrationEvent::Log {
                    message: format!("Execution state: {state:?}"),
                    timestamp: Self::now(),
                    metadata: None,
                }
            }
            JobEvent::AttemptStarted { number } => {
                info!(attempt = number, "Attempt started");
                OrchestrationEvent::Log {
                    message: format!("Attempt {number} started"),
                    timestamp: Self::now(),
                    metadata: None,
                }
            }
            JobEvent::AttemptCompleted { number, ref result } => {
                info!(attempt = number, result = ?result, "Attempt completed");
                OrchestrationEvent::Log {
                    message: format!("Attempt {number} completed: {result:?}"),
                    timestamp: Self::now(),
                    metadata: None,
                }
            }
        };

        self.persist_event(&orch_event);
        self.hub.broadcast(orch_event);
    }
}

// ============================================================
// Handlers
// ============================================================

/// Start batch execution.
///
/// Validates the spec exists, checks no other execution is running,
/// registers the run in the registry, and spawns an in-process
/// orchestration task with Hub event bridging.
///
/// # Errors
///
/// Returns an error if:
/// - Another execution is already running
/// - The spec does not exist
/// - The orchestrator fails to start
pub async fn start_execution_handler(
    state: &AppState,
    request: ExecuteRequest,
) -> Result<ExecuteResponse, TransportError> {
    // Check for concurrent execution
    {
        let registry = state.run_registry.read().await;
        if let Some(active) = registry.active_run() {
            return Err(TransportError::BadRequest(format!(
                "Execution already in progress: {} (run {})",
                active.spec_name, active.run_id
            )));
        }
    }

    // Validate spec exists
    let spec_dir = state.project_root.join(".specs").join(&request.spec);
    if !spec_dir.exists() {
        return Err(TransportError::NotFound(format!(
            "Spec not found: {}",
            request.spec
        )));
    }

    // Generate run ID and create registry entry
    let run_id = format!("run-{}", chrono::Utc::now().timestamp_millis());
    let cancel_token = CancellationToken::new();

    let entry = RunEntry {
        run_id: run_id.clone(),
        spec_name: request.spec.clone(),
        started_at: Instant::now(),
        status: RunStatus::Running,
        cancel_token: cancel_token.clone(),
        error_message: None,
    };

    {
        let mut registry = state.run_registry.write().await;
        registry.insert(entry);
    }

    info!(
        run_id = %run_id,
        spec = %request.spec,
        dry_run = request.dry_run,
        "Starting execution"
    );

    // Broadcast start event
    state.hub.broadcast(OrchestrationEvent::Log {
        message: format!("Starting execution for spec: {}", request.spec),
        timestamp: chrono::Utc::now().to_rfc3339(),
        metadata: None,
    });

    // Spawn execution task
    let hub = state.hub.clone();
    let registry = state.run_registry.clone();
    let run_id_clone = run_id.clone();
    let spec_name = request.spec.clone();
    let project_root = state.project_root.clone();

    tokio::spawn(async move {
        let hub_clone = hub.clone();
        let project_root_clone = project_root.clone();
        let spec_name_for_orch = spec_name.clone();
        let spec_name_for_cleanup = spec_name.clone();
        let run_id_for_orch = run_id_clone.clone();
        let dry_run = request.dry_run;

        let result = tokio::select! {
            result = tokio::task::spawn_blocking(move || {
                run_orchestration(
                    &project_root_clone,
                    &spec_name_for_orch,
                    &run_id_for_orch,
                    hub_clone,
                    dry_run,
                )
            }) => result.unwrap_or_else(|e| Err(format!("Execution task panicked: {e}"))),
            () = cancel_token.cancelled() => {
                warn!(run_id = %run_id_clone, "Execution cancelled");
                cleanup_docker_containers(&spec_name_for_cleanup);
                Err("Execution cancelled by user".to_string())
            }
        };

        // Update registry with result
        match result {
            Ok(()) => {
                info!(run_id = %run_id_clone, "Execution completed successfully");
                registry.write().await.set_status(&run_id_clone, RunStatus::Done, None);
                hub.broadcast(OrchestrationEvent::Success {
                    message: format!("Execution of {spec_name} completed successfully"),
                    timestamp: chrono::Utc::now().to_rfc3339(),
                });
            }
            Err(err) => {
                error!(run_id = %run_id_clone, error = %err, "Execution failed");
                registry.write().await.set_status(
                    &run_id_clone,
                    RunStatus::Error,
                    Some(err.clone()),
                );
                hub.broadcast(OrchestrationEvent::Error {
                    message: err,
                    timestamp: chrono::Utc::now().to_rfc3339(),
                });
            }
        }
    });

    Ok(ExecuteResponse {
        started: true,
        run_id: Some(run_id),
        message: Some("Execution started".to_string()),
    })
}

/// Run the orchestration engine in-process.
///
/// This function simulates orchestration for now (the real orchestrator
/// has simulated step execution). In the future, this will wire up
/// `DefaultOrchestrator` with real Docker sandbox execution.
fn run_orchestration(
    project_root: &std::path::Path,
    spec_name: &str,
    run_id: &str,
    hub: SharedHub,
    dry_run: bool,
) -> Result<(), String> {
    let log_path = project_root
        .join(".specs")
        .join(spec_name)
        .join("runs")
        .join(run_id)
        .join("logs.jsonl");
    let handler = Arc::new(HubEventHandler::with_log_file(hub.clone(), log_path)
        .unwrap_or_else(|e| {
            warn!("Failed to create log file, running without persistence: {e}");
            HubEventHandler::new(hub.clone())
        }));

    // Load the plan to get steps
    let plan_path = project_root
        .join(".specs")
        .join(spec_name)
        .join("plan.yaml");

    if !plan_path.exists() {
        return Err(format!("No plan found for spec: {spec_name}"));
    }

    let plan_content = std::fs::read_to_string(&plan_path)
        .map_err(|e| format!("Failed to read plan: {e}"))?;

    // Parse batches from the plan YAML
    let plan_yaml: serde_yaml::Value = serde_yaml::from_str(&plan_content)
        .map_err(|e| format!("Failed to parse plan YAML: {e}"))?;

    let batches = plan_yaml
        .get("batches")
        .and_then(|b| b.as_sequence())
        .ok_or_else(|| "Plan has no batches".to_string())?;

    if dry_run {
        handler.handle(JobEvent::StateChanged {
            state: ckrv_core::RunState::Planning,
        });
        info!(spec = %spec_name, batch_count = batches.len(), "Dry run - skipping execution");
        let event = OrchestrationEvent::Log {
            message: format!("Dry run: found {} batches, skipping execution", batches.len()),
            timestamp: chrono::Utc::now().to_rfc3339(),
            metadata: None,
        };
        handler.persist_event(&event);
        hub.broadcast(event);
        return Ok(());
    }

    handler.handle(JobEvent::StateChanged {
        state: ckrv_core::RunState::Executing {
            attempt: 1,
            step: "start".to_string(),
        },
    });

    // Execute each batch as a step
    for batch in batches {
        let batch_name = batch
            .get("name")
            .and_then(|n| n.as_str())
            .unwrap_or("unnamed");
        let batch_id = batch
            .get("id")
            .and_then(|n| n.as_str())
            .unwrap_or(batch_name);

        handler.handle(JobEvent::StepStarted {
            step_id: batch_id.to_string(),
        });

        // Execute the batch via ckrv execute command
        let mut args = vec!["execute", "--batch", batch_id];
        let spec_path = format!(".specs/{spec_name}");
        args.push(&spec_path);

        info!(batch_id = %batch_id, "Executing batch");

        let start = Instant::now();
        let output = Command::new("ckrv")
            .args(&args)
            .current_dir(project_root)
            .output();

        match output {
            Ok(out) if out.status.success() => {
                let duration_ms = start.elapsed().as_millis() as u64;

                // Forward stdout as log events
                let stdout = String::from_utf8_lossy(&out.stdout);
                for line in stdout.lines().filter(|l| !l.is_empty()) {
                    let event = OrchestrationEvent::Log {
                        message: line.to_string(),
                        timestamp: chrono::Utc::now().to_rfc3339(),
                        metadata: None,
                    };
                    handler.persist_event(&event);
                    hub.broadcast(event);
                }

                // Forward stderr as log events (warnings, diagnostics)
                let stderr = String::from_utf8_lossy(&out.stderr);
                for line in stderr.lines().filter(|l| !l.is_empty()) {
                    let event = OrchestrationEvent::Log {
                        message: line.to_string(),
                        timestamp: chrono::Utc::now().to_rfc3339(),
                        metadata: Some(serde_json::json!({ "level": "error" })),
                    };
                    handler.persist_event(&event);
                    hub.broadcast(event);
                }

                handler.handle(JobEvent::StepCompleted {
                    step_id: batch_id.to_string(),
                    duration_ms,
                });
            }
            Ok(out) => {
                // Forward stderr lines as individual log events before failing
                let stderr = String::from_utf8_lossy(&out.stderr);
                for line in stderr.lines().filter(|l| !l.is_empty()) {
                    let event = OrchestrationEvent::Log {
                        message: line.to_string(),
                        timestamp: chrono::Utc::now().to_rfc3339(),
                        metadata: Some(serde_json::json!({ "level": "error" })),
                    };
                    handler.persist_event(&event);
                    hub.broadcast(event);
                }
                handler.handle(JobEvent::StepFailed {
                    step_id: batch_id.to_string(),
                    error: stderr.to_string(),
                });
                return Err(format!("Batch {batch_id} failed: {stderr}"));
            }
            Err(e) => {
                handler.handle(JobEvent::StepFailed {
                    step_id: batch_id.to_string(),
                    error: e.to_string(),
                });
                return Err(format!("Failed to execute batch {batch_id}: {e}"));
            }
        }
    }

    handler.handle(JobEvent::StateChanged {
        state: ckrv_core::RunState::Succeeded {
            attempt: 1,
            diff_path: std::path::PathBuf::new(),
        },
    });

    Ok(())
}

/// Get current execution status.
///
/// Reads from the run registry to report the actual state of
/// any active or recent execution.
///
/// # Errors
///
/// Returns an error if the registry lock cannot be acquired.
pub async fn get_execution_status_handler(
    state: &AppState,
) -> Result<ExecutionStatus, TransportError> {
    let registry = state.run_registry.read().await;

    Ok(registry.active_run().map_or(
        ExecutionStatus {
            running: false,
            spec_name: None,
            batch_id: None,
            progress: 0.0,
            current_task: None,
            message: None,
        },
        |active| {
            let elapsed = active.started_at.elapsed();
            ExecutionStatus {
                running: true,
                spec_name: Some(active.spec_name.clone()),
                batch_id: None,
                progress: 0.0,
                current_task: None,
                message: Some(format!("Running for {}s", elapsed.as_secs())),
            }
        },
    ))
}

/// Stop a running execution.
///
/// Finds the run by spec name or run_id and triggers its cancellation token.
/// The spawned task handles cleanup.
///
/// # Errors
///
/// Returns an error if no matching run is found or the run is not active.
pub async fn stop_execution_handler(
    state: &AppState,
    request: StopRequest,
) -> Result<(), TransportError> {
    let registry = state.run_registry.read().await;

    // Find the run entry
    let entry = if let Some(ref run_id) = request.run_id {
        registry.runs.get(run_id)
    } else {
        registry.find_by_spec(&request.spec)
    };

    match entry {
        Some(entry) if entry.status == RunStatus::Running || entry.status == RunStatus::Pending => {
            info!(
                run_id = %entry.run_id,
                spec = %entry.spec_name,
                "Stopping execution"
            );
            entry.cancel_token.cancel();
            Ok(())
        }
        Some(_) => Err(TransportError::BadRequest(
            "Run is not active".to_string(),
        )),
        None => Err(TransportError::NotFound(
            "No matching run found".to_string(),
        )),
    }
}

/// Pause execution (if supported).
///
/// # Errors
///
/// Always returns `BadRequest` as pause is not currently supported.
pub fn pause_execution_handler(_state: &AppState) -> Result<ExecutionStatus, TransportError> {
    Err(TransportError::BadRequest(
        "Pause not supported".to_string(),
    ))
}

/// Resume execution (if supported).
///
/// # Errors
///
/// Always returns `BadRequest` as resume is not currently supported.
pub fn resume_execution_handler(_state: &AppState) -> Result<ExecutionStatus, TransportError> {
    Err(TransportError::BadRequest(
        "Resume not supported".to_string(),
    ))
}

// ============================================================
// Branch Management
// ============================================================

/// Request to list branches.
#[derive(Debug, Deserialize)]
pub struct ListBranchesRequest {
    /// Optional spec name to filter branches.
    pub spec: Option<String>,
}

/// Branch info.
#[derive(Debug, Serialize)]
pub struct BranchInfo {
    /// Full branch name.
    pub name: String,
    /// Batch name extracted from the branch.
    pub batch_name: String,
    /// Number of commits ahead of HEAD.
    pub ahead_commits: u32,
    /// Whether the worktree has no uncommitted changes.
    pub is_clean: bool,
}

/// Response with branches.
#[derive(Debug, Serialize)]
pub struct ListBranchesResponse {
    /// Whether the operation succeeded.
    pub success: bool,
    /// Current HEAD branch name.
    pub current_branch: String,
    /// Unmerged worktree branches.
    pub branches: Vec<BranchInfo>,
    /// Descriptive message.
    pub message: Option<String>,
}

/// List unmerged worktree branches.
///
/// # Errors
///
/// Returns an error if git commands fail.
pub fn list_branches_handler(
    state: &AppState,
    request: ListBranchesRequest,
) -> Result<ListBranchesResponse, TransportError> {
    let cwd = &state.project_root;

    // Get current branch
    let current_output = Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .current_dir(cwd)
        .output();

    let current_branch = current_output
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "HEAD".to_string());

    // Get list of actual worktrees
    let worktree_output = Command::new("git")
        .args(["worktree", "list", "--porcelain"])
        .current_dir(cwd)
        .output();

    // Build set of branches that have actual worktrees
    let mut worktree_branches: HashSet<String> = HashSet::new();
    if let Ok(output) = worktree_output {
        let text = String::from_utf8_lossy(&output.stdout);
        for line in text.lines() {
            if line.starts_with("branch refs/heads/") {
                let branch = line
                    .strip_prefix("branch refs/heads/")
                    .unwrap_or("")
                    .to_string();
                if branch.contains("worktree/") {
                    worktree_branches.insert(branch);
                }
            }
        }
    }

    // If no worktrees exist, return empty list
    if worktree_branches.is_empty() {
        return Ok(ListBranchesResponse {
            success: true,
            current_branch,
            branches: vec![],
            message: None,
        });
    }

    // Filter by spec if provided
    let filter_pattern = request.spec.as_ref().map_or_else(
        || "worktree/".to_string(),
        |spec| format!("worktree/{spec}/"),
    );

    let mut branches = Vec::new();

    for branch_name in worktree_branches {
        // Check if matches filter pattern
        if !branch_name.starts_with(&filter_pattern) && request.spec.is_some() {
            continue;
        }

        // Check if branch is already merged into HEAD
        let is_merged = Command::new("git")
            .args(["merge-base", "--is-ancestor", &branch_name, "HEAD"])
            .current_dir(cwd)
            .status()
            .map(|s| s.success())
            .unwrap_or(false);

        if is_merged {
            continue;
        }

        // Get ahead commit count
        let ahead_output = Command::new("git")
            .args(["rev-list", "--count", &format!("HEAD..{branch_name}")])
            .current_dir(cwd)
            .output();

        let ahead_commits: u32 = ahead_output
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .and_then(|s| s.trim().parse().ok())
            .unwrap_or(0);

        // Extract batch name from branch name
        let batch_name = branch_name
            .split('/')
            .next_back()
            .unwrap_or(&branch_name)
            .replace("ckrv-batch-", "");

        branches.push(BranchInfo {
            name: branch_name,
            batch_name,
            ahead_commits,
            is_clean: true,
        });
    }

    Ok(ListBranchesResponse {
        success: true,
        current_branch,
        branches,
        message: None,
    })
}

/// Request to merge all branches.
#[derive(Debug, Deserialize)]
pub struct MergeAllRequest {
    /// Optional spec name to filter branches.
    pub spec: Option<String>,
}

/// Response from merge all.
#[derive(Debug, Serialize)]
pub struct MergeAllResponse {
    /// Whether all merges succeeded.
    pub success: bool,
    /// Branch names that were successfully merged.
    pub merged: Vec<String>,
    /// Branch names that failed to merge.
    pub failed: Vec<String>,
    /// Summary message.
    pub message: String,
}

/// Merge all worktree branches.
///
/// # Errors
///
/// Returns an error if git operations fail.
pub fn merge_all_branches_handler(
    state: &AppState,
    _request: MergeAllRequest,
) -> Result<MergeAllResponse, TransportError> {
    let project_root = &state.project_root;

    let worktree_output = Command::new("git")
        .args(["worktree", "list", "--porcelain"])
        .current_dir(project_root)
        .output();

    let worktree_info: Vec<(String, String)> = match worktree_output {
        Ok(output) => {
            let text = String::from_utf8_lossy(&output.stdout);
            let mut worktrees = Vec::new();
            let mut current_path = String::new();

            for line in text.lines() {
                if line.starts_with("worktree ") {
                    current_path = line.strip_prefix("worktree ").unwrap_or("").to_string();
                } else if line.starts_with("branch refs/heads/") {
                    let current_branch = line
                        .strip_prefix("branch refs/heads/")
                        .unwrap_or("")
                        .to_string();
                    if current_branch.contains("worktree/") && !current_path.is_empty() {
                        worktrees.push((current_path.clone(), current_branch));
                    }
                }
            }
            worktrees
        }
        Err(_) => Vec::new(),
    };

    if worktree_info.is_empty() {
        return Ok(MergeAllResponse {
            success: true,
            merged: vec![],
            failed: vec![],
            message: "No worktree branches to merge".to_string(),
        });
    }

    let mut merged = Vec::new();
    let mut failed = Vec::new();

    for (wt_path, branch) in worktree_info {
        let is_merged = Command::new("git")
            .args(["merge-base", "--is-ancestor", &branch, "HEAD"])
            .current_dir(project_root)
            .status()
            .map(|s| s.success())
            .unwrap_or(false);

        if is_merged {
            let _ = Command::new("git")
                .args(["worktree", "remove", "--force", &wt_path])
                .current_dir(project_root)
                .status();
            merged.push(branch);
            continue;
        }

        let merge_result = Command::new("git")
            .args(["merge", "--no-ff", "--no-edit", &branch])
            .current_dir(project_root)
            .status();

        if merge_result.map(|s| s.success()).unwrap_or(false) {
            let _ = Command::new("git")
                .args(["worktree", "remove", "--force", &wt_path])
                .current_dir(project_root)
                .status();
            merged.push(branch);
        } else {
            let _ = Command::new("git")
                .args(["merge", "--abort"])
                .current_dir(project_root)
                .status();
            failed.push(branch);
        }
    }

    let success = failed.is_empty();
    let message = if success {
        format!("Successfully merged {} branches", merged.len())
    } else {
        format!("Merged {} branches, {} failed", merged.len(), failed.len())
    };

    Ok(MergeAllResponse {
        success,
        merged,
        failed,
        message,
    })
}

/// Request to merge a single branch.
#[derive(Debug, Deserialize)]
pub struct MergeBranchRequest {
    /// Branch name to merge into HEAD.
    pub branch: String,
}

/// Response from merge branch.
#[derive(Debug, Serialize)]
pub struct MergeBranchResponse {
    /// Whether the merge succeeded.
    pub success: bool,
    /// Branch that was merged.
    pub branch: String,
    /// Descriptive result message.
    pub message: String,
}

/// Merge a single branch.
///
/// # Errors
///
/// Returns an error if the merge fails due to conflicts.
pub fn merge_branch_handler(
    state: &AppState,
    request: MergeBranchRequest,
) -> Result<MergeBranchResponse, TransportError> {
    let project_root = &state.project_root;

    let merge_result = Command::new("git")
        .args(["merge", "--no-ff", "--no-edit", &request.branch])
        .current_dir(project_root)
        .status();

    if merge_result.map(|s| s.success()).unwrap_or(false) {
        Ok(MergeBranchResponse {
            success: true,
            branch: request.branch,
            message: "Branch merged successfully".to_string(),
        })
    } else {
        let _ = Command::new("git")
            .args(["merge", "--abort"])
            .current_dir(project_root)
            .status();

        Ok(MergeBranchResponse {
            success: false,
            branch: request.branch,
            message: "Merge failed - conflicts detected".to_string(),
        })
    }
}

// ============================================================
// Log Handlers
// ============================================================

/// Log entry for execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    /// ISO 8601 timestamp.
    pub timestamp: String,
    /// Log level (info, warn, error).
    pub level: String,
    /// Log message content.
    pub message: String,
}

/// Log history request params.
#[derive(Debug, Deserialize)]
pub struct LogHistoryParams {
    /// Starting offset for pagination.
    pub offset: Option<usize>,
    /// Maximum number of entries to return.
    pub limit: Option<usize>,
    /// Only return logs after this ISO 8601 timestamp.
    pub since: Option<String>,
}

/// Log history response.
#[derive(Debug, Serialize)]
pub struct LogHistoryResponse {
    /// Execution run identifier.
    pub execution_id: String,
    /// Log entries for this page.
    pub logs: Vec<LogEntry>,
    /// Total number of log entries.
    pub total_count: usize,
    /// Current page offset.
    pub offset: usize,
    /// Whether more entries exist beyond this page.
    pub has_more: bool,
}

/// Log tail params.
#[derive(Debug, Deserialize)]
pub struct LogTailParams {
    /// Number of most recent log entries to return.
    pub count: Option<usize>,
}

/// Log tail response.
#[derive(Debug, Serialize)]
pub struct LogTailResponse {
    /// Execution run identifier.
    pub execution_id: String,
    /// Most recent log entries.
    pub logs: Vec<LogEntry>,
    /// Total number of log entries available.
    pub total_count: usize,
}

/// Get execution logs from JSONL persistence.
///
/// Reads log entries from `.specs/{spec}/runs/{run_id}/logs.jsonl`.
///
/// # Errors
///
/// Returns an error if the log file cannot be read.
pub fn get_logs_handler(
    state: &AppState,
    execution_id: String,
    params: LogHistoryParams,
) -> Result<LogHistoryResponse, TransportError> {
    let log_path = find_log_file(&state.project_root, &execution_id);

    let entries = match log_path {
        Some(path) => read_jsonl_logs(&path)
            .map_err(|e| TransportError::Internal(format!("Failed to read logs: {e}")))?,
        None => vec![],
    };

    let total_count = entries.len();
    let offset = params.offset.unwrap_or(0);
    let limit = params.limit.unwrap_or(100);

    let page: Vec<LogEntry> = entries
        .into_iter()
        .skip(offset)
        .take(limit)
        .collect();

    let has_more = offset + page.len() < total_count;

    Ok(LogHistoryResponse {
        execution_id,
        logs: page,
        total_count,
        offset,
        has_more,
    })
}

/// Tail execution logs from JSONL persistence.
///
/// Returns the most recent N log entries.
///
/// # Errors
///
/// Returns an error if the log file cannot be read.
pub fn tail_logs_handler(
    state: &AppState,
    execution_id: String,
    params: LogTailParams,
) -> Result<LogTailResponse, TransportError> {
    let log_path = find_log_file(&state.project_root, &execution_id);

    let entries = match log_path {
        Some(path) => read_jsonl_logs(&path)
            .map_err(|e| TransportError::Internal(format!("Failed to read logs: {e}")))?,
        None => vec![],
    };

    let total_count = entries.len();
    let count = params.count.unwrap_or(50);
    let start = total_count.saturating_sub(count);

    let tail: Vec<LogEntry> = entries.into_iter().skip(start).collect();

    Ok(LogTailResponse {
        execution_id,
        logs: tail,
        total_count,
    })
}

// ============================================================
// Docker Cleanup
// ============================================================

/// Kill any Docker containers associated with a spec.
///
/// This is best-effort — if Docker isn't running or no containers match,
/// it silently does nothing.
fn cleanup_docker_containers(spec_name: &str) {
    // Find containers with the ckrv label for this spec
    let containers = Command::new("docker")
        .args([
            "ps", "-q",
            "--filter", &format!("label=ckrv.spec={spec_name}"),
        ])
        .output();

    match containers {
        Ok(output) if output.status.success() => {
            let ids: Vec<&str> = std::str::from_utf8(&output.stdout)
                .unwrap_or("")
                .lines()
                .filter(|l| !l.is_empty())
                .collect();

            if ids.is_empty() {
                return;
            }

            info!(spec = %spec_name, count = ids.len(), "Killing Docker containers");
            let mut kill_args = vec!["kill"];
            kill_args.extend(ids);
            let _ = Command::new("docker").args(&kill_args).output();
        }
        _ => {
            // Docker not available or command failed — nothing to clean up
        }
    }
}

// ============================================================
// Log Persistence Helpers
// ============================================================

/// Find the JSONL log file for an execution run.
///
/// Searches for `.specs/*/runs/{execution_id}/logs.jsonl`.
fn find_log_file(
    project_root: &std::path::Path,
    execution_id: &str,
) -> Option<std::path::PathBuf> {
    let specs_dir = project_root.join(".specs");
    if !specs_dir.exists() {
        return None;
    }

    // Search all specs for the run
    if let Ok(entries) = std::fs::read_dir(&specs_dir) {
        for entry in entries.flatten() {
            let log_path = entry
                .path()
                .join("runs")
                .join(execution_id)
                .join("logs.jsonl");
            if log_path.exists() {
                return Some(log_path);
            }
        }
    }

    None
}

/// Read log entries from a JSONL file.
fn read_jsonl_logs(path: &std::path::Path) -> Result<Vec<LogEntry>, std::io::Error> {
    let content = std::fs::read_to_string(path)?;
    let entries: Vec<LogEntry> = content
        .lines()
        .filter(|line| !line.is_empty())
        .filter_map(|line| serde_json::from_str(line).ok())
        .collect();
    Ok(entries)
}

// ============================================================
// Tests
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[tokio::test]
    async fn test_get_execution_status_handler_no_active_run() {
        let state = AppState::new(PathBuf::from("/tmp/test-execution"));
        let result = get_execution_status_handler(&state).await;
        assert!(result.is_ok());
        assert!(!result.unwrap().running);
    }

    #[tokio::test]
    async fn test_start_blocks_concurrent_execution() {
        let state = AppState::new(PathBuf::from("/tmp/test-execution"));

        // Insert a running entry
        {
            let mut registry = state.run_registry.write().await;
            registry.insert(RunEntry {
                run_id: "run-test".to_string(),
                spec_name: "test-spec".to_string(),
                started_at: Instant::now(),
                status: RunStatus::Running,
                cancel_token: CancellationToken::new(),
                error_message: None,
            });
        }

        let request = ExecuteRequest {
            spec: "another-spec".to_string(),
            batch_id: None,
            dry_run: false,
        };

        let result = start_execution_handler(&state, request).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_stop_nonexistent_run() {
        let state = AppState::new(PathBuf::from("/tmp/test-execution"));
        let request = StopRequest {
            spec: "nonexistent".to_string(),
            run_id: None,
        };
        let result = stop_execution_handler(&state, request).await;
        assert!(result.is_err());
    }

    #[test]
    fn test_hub_event_handler_maps_step_started() {
        let hub = Arc::new(crate::hub::Hub::new());
        let mut rx = hub.subscribe();
        let handler = HubEventHandler::new(hub);

        handler.handle(JobEvent::StepStarted {
            step_id: "batch-1".to_string(),
        });

        let event = rx.try_recv().expect("should receive event");
        if let OrchestrationEvent::StepStart { step_name, .. } = event {
            assert_eq!(step_name, "batch-1");
        } else {
            panic!("expected StepStart event");
        }
    }

    #[test]
    fn test_hub_event_handler_maps_step_failed() {
        let hub = Arc::new(crate::hub::Hub::new());
        let mut rx = hub.subscribe();
        let handler = HubEventHandler::new(hub);

        handler.handle(JobEvent::StepFailed {
            step_id: "batch-1".to_string(),
            error: "test error".to_string(),
        });

        // Should receive a Log event first (error detail)
        let event1 = rx.try_recv().expect("should receive log event");
        assert!(matches!(event1, OrchestrationEvent::Log { .. }));

        // Then a StepEnd with error status
        let event2 = rx.try_recv().expect("should receive step end event");
        if let OrchestrationEvent::StepEnd { status, .. } = event2 {
            assert_eq!(status, "error");
        } else {
            panic!("expected StepEnd event");
        }
    }

    #[test]
    fn test_read_jsonl_logs_empty() {
        let dir = tempfile::TempDir::new().unwrap();
        let path = dir.path().join("logs.jsonl");
        std::fs::write(&path, "").unwrap();

        let result = read_jsonl_logs(&path);
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[test]
    fn test_read_jsonl_logs_with_entries() {
        let dir = tempfile::TempDir::new().unwrap();
        let path = dir.path().join("logs.jsonl");
        let content = r#"{"timestamp":"2026-03-20T00:00:00Z","level":"info","message":"hello"}
{"timestamp":"2026-03-20T00:00:01Z","level":"error","message":"oops"}
"#;
        std::fs::write(&path, content).unwrap();

        let result = read_jsonl_logs(&path).unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].message, "hello");
        assert_eq!(result[1].level, "error");
    }
}
