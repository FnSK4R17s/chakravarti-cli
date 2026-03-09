//! # Execution Engine
//!
//! Core orchestration engine for running AI agent tasks.
//!
//! ## Overview
//!
//! The `ExecutionEngine` is responsible for:
//! - Loading and parsing execution plans (plan.yaml)
//! - Scheduling batches with dependency resolution
//! - Creating isolated Git worktrees for each batch
//! - Executing tasks in Docker sandboxes
//! - Streaming logs to connected WebSocket clients
//! - Persisting execution history
//!
//! ## Architecture
//!
//! ```text
//! ExecutionPlan
//!       │
//!       ▼
//! ┌─────────────┐
//! │   Batch 1   │──────────────────────────────┐
//! │  (no deps)  │                              │
//! └─────────────┘                              │
//!       │                                      │
//!       ▼                                      ▼
//! ┌─────────────┐                       ┌─────────────┐
//! │   Batch 2   │                       │   Batch 3   │
//! │ depends:[1] │                       │ depends:[1] │
//! └─────────────┘                       └─────────────┘
//!       │                                      │
//!       └──────────────┬───────────────────────┘
//!                      ▼
//!                ┌─────────────┐
//!                │   Batch 4   │
//!                │depends:[2,3]│
//!                └─────────────┘
//! ```
//!
//! ## Key Types
//!
//! - [`ExecutionEngine`] - Main orchestrator
//! - [`ExecutionPlan`] - Parsed plan.yaml structure
//! - [`Batch`] - Group of tasks to execute together
//! - [`LogMessage`] - Real-time log event
//!
//! ## Example
//!
//! ```rust,ignore
//! use ckrv_ui::services::engine::ExecutionEngine;
//! use tokio::sync::mpsc;
//!
//! let (tx, rx) = mpsc::channel(100);
//! let engine = ExecutionEngine::new(project_root, tx);
//!
//! engine.run_spec("my-feature".to_string(), false, None, "claude".to_string(), None)
//!     .await?;
//! ```

// ============================================================
// IMPORTS
// ============================================================

use std::collections::{HashMap, HashSet, VecDeque};
use std::path::{Path, PathBuf};
use std::time::Duration;

use anyhow::{anyhow, Context, Result};
use chrono::Utc;
use futures::stream::FuturesUnordered;
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use tokio::process::Command as AsyncCommand;
use tokio::sync::mpsc;

use ckrv_git::{DefaultWorktreeManager, WorktreeManager};
use ckrv_sandbox::{DefaultAllowList, DockerSandbox, ExecuteConfig, Sandbox};

use crate::models::history::{HistoryBatchStatus, Run, RunStatus};
use crate::models::log::{LogEntry, LogLevel};
use crate::services::history::HistoryService;
use crate::services::log_store::LogStore;

// ============================================================
// TYPES
// ============================================================

/// Status of a batch in the execution plan.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum BatchStatus {
    /// Waiting to start.
    #[default]
    Pending,
    /// Currently executing.
    Running,
    /// Successfully completed.
    Completed,
    /// Execution failed.
    Failed,
}

// Custom deserializer that handles empty strings as Pending
impl<'de> serde::Deserialize<'de> for BatchStatus {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.to_lowercase().as_str() {
            "running" => Ok(Self::Running),
            "completed" => Ok(Self::Completed),
            "failed" => Ok(Self::Failed),
            // "" | "pending" and unknown values default to Pending
            _ => Ok(Self::Pending),
        }
    }
}

/// Execution plan structure (plan.yaml).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ExecutionPlan {
    /// Optional spec identifier this plan belongs to.
    #[serde(default)]
    pub spec_id: Option<String>,
    /// Ordered list of batches to execute.
    pub batches: Vec<Batch>,
}

/// A group of tasks to execute together in a single worktree.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Batch {
    /// Unique batch identifier.
    pub id: String,
    /// Human-readable batch name.
    pub name: String,
    /// IDs of tasks included in this batch.
    pub task_ids: Vec<String>,
    /// IDs of batches that must complete before this one.
    #[serde(default)]
    pub depends_on: Vec<String>,
    /// Current batch status.
    #[serde(default)]
    pub status: BatchStatus,
    /// Git branch created for this batch.
    #[serde(default)]
    pub branch: Option<String>,
    /// Reasoning behind the batch grouping.
    pub reasoning: String,
    /// Model assignment configuration for this batch.
    pub model_assignment: ModelAssignment,
}

/// Model assignment configuration for a batch.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModelAssignment {
    /// Default model to use for tasks.
    pub default: Option<String>,
    /// Per-task model overrides (task_id -> model_name).
    #[serde(default)]
    pub overrides: HashMap<String, String>,
}

/// Task file structure (tasks.yaml)
#[derive(Serialize, Deserialize)]
struct TaskFile {
    tasks: Vec<SpecTask>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct SpecTask {
    pub id: String,
    pub title: String,
    pub description: String,
    pub status: String,
    #[serde(default)]
    pub complexity: u8,
}

/// Log message structure for streaming updates.
#[derive(Debug, Clone, Serialize)]
pub struct LogMessage {
    /// Message type (e.g., "info", "success", "error", "batch_status").
    #[serde(rename = "type")]
    pub type_: String,
    /// Human-readable log message content.
    pub message: String,
    /// Output stream origin (`stdout` or `stderr`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<String>,
    /// ISO 8601 timestamp of when the message was generated.
    pub timestamp: String,
    /// Execution or batch status for state-transition messages.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    /// Batch identifier for batch-attributed messages.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub batch_id: Option<String>,
    /// Batch name for batch-attributed messages.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub batch_name: Option<String>,
    /// Branch name associated with the message.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub branch: Option<String>,
    /// Error message if applicable.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl LogMessage {
    /// Create a new log message with the given type and content.
    pub fn new(type_: &str, message: &str) -> Self {
        Self {
            type_: type_.to_string(),
            message: message.to_string(),
            stream: None,
            timestamp: Utc::now().to_rfc3339(),
            status: None,
            batch_id: None,
            batch_name: None,
            branch: None,
            error: None,
        }
    }

    /// T005: Create an execution status message
    /// Used to signal running/completed/failed state transitions
    pub fn status(status: &str) -> Self {
        Self {
            type_: "status".to_string(),
            message: String::new(),
            stream: None,
            timestamp: Utc::now().to_rfc3339(),
            status: Some(status.to_string()),
            batch_id: None,
            batch_name: None,
            branch: None,
            error: None,
        }
    }

    /// T005: Create a batch status message
    /// Used to signal batch running/completed/failed state transitions
    pub fn batch_status(batch_id: &str, batch_name: &str, status: &str) -> Self {
        Self {
            type_: "batch_status".to_string(),
            message: String::new(),
            stream: None,
            timestamp: Utc::now().to_rfc3339(),
            status: Some(status.to_string()),
            batch_id: Some(batch_id.to_string()),
            batch_name: Some(batch_name.to_string()),
            branch: None,
            error: None,
        }
    }

    /// Set branch name (for completed batches)
    pub fn with_branch(mut self, branch: &str) -> Self {
        self.branch = Some(branch.to_string());
        self
    }

    /// Set error message (for failed batches)
    pub fn with_error(mut self, error: &str) -> Self {
        self.error = Some(error.to_string());
        self
    }

    /// Set batch id and name (for batch-attributed logs)
    pub fn with_batch(mut self, batch_id: &str, batch_name: &str) -> Self {
        self.batch_id = Some(batch_id.to_string());
        self.batch_name = Some(batch_name.to_string());
        self
    }
}

/// Core orchestration engine for running AI agent tasks.
///
/// Loads execution plans, schedules batches with dependency resolution,
/// creates isolated Git worktrees, and streams logs to connected clients.
pub struct ExecutionEngine {
    project_root: PathBuf,
    sender: mpsc::Sender<LogMessage>,
    /// T012: LogStore for persisting logs to disk
    log_store: LogStore,
    /// Current execution ID for log persistence
    current_execution_id: std::sync::Arc<std::sync::Mutex<Option<String>>>,
}

// ============================================================
// IMPLEMENTATION
// ============================================================

impl ExecutionEngine {
    /// Create a new execution engine rooted at the given project path.
    pub fn new(project_root: PathBuf, sender: mpsc::Sender<LogMessage>) -> Self {
        Self {
            log_store: LogStore::new(&project_root),
            project_root,
            sender,
            current_execution_id: std::sync::Arc::new(std::sync::Mutex::new(None)),
        }
    }

    /// Set the current execution ID for log persistence
    fn set_execution_id(&self, execution_id: &str) {
        if let Ok(mut guard) = self.current_execution_id.lock() {
            *guard = Some(execution_id.to_string());
        }
    }

    /// T013: Modified log method that persists logs via LogStore
    async fn log(&self, type_: &str, message: &str) {
        let _ = self.sender.send(LogMessage::new(type_, message)).await;
        // Also print to server stdout for debugging
        println!("[ExecutionEngine] {}: {}", type_, message);

        // T013: Persist to disk if execution ID is set
        let execution_id = self
            .current_execution_id
            .lock()
            .ok()
            .and_then(|g| g.clone());
        if let Some(execution_id) = execution_id {
            let level = match type_ {
                "warning" => LogLevel::Warning,
                "error" => LogLevel::Error,
                "log" => LogLevel::Log,
                "start" => LogLevel::Start,
                "batch_start" => LogLevel::BatchStart,
                "batch_complete" => LogLevel::BatchComplete,
                "batch_error" => LogLevel::BatchError,
                "success" => LogLevel::Success,
                "status" => LogLevel::Status,
                // "info" and anything else
                _ => LogLevel::Info,
            };

            let entry = LogEntry::new(&execution_id, level, message);
            if let Err(e) = self.log_store.append(&execution_id, &entry) {
                eprintln!("Warning: Failed to persist log entry: {}", e);
            }
        }
    }

    /// T013: Log method for batch-attributed logs that persists with batch info
    #[allow(dead_code)]
    async fn log_with_batch(&self, type_: &str, message: &str, batch_id: &str, batch_name: &str) {
        let _ = self
            .sender
            .send(LogMessage::new(type_, message).with_batch(batch_id, batch_name))
            .await;
        // Also print to server stdout for debugging
        println!("[ExecutionEngine] {}/{}: {}", batch_name, type_, message);

        // T013: Persist to disk if execution ID is set
        let execution_id = self
            .current_execution_id
            .lock()
            .ok()
            .and_then(|g| g.clone());
        if let Some(execution_id) = execution_id {
            let level = match type_ {
                "warning" => LogLevel::Warning,
                "error" => LogLevel::Error,
                "log" => LogLevel::Log,
                "start" => LogLevel::Start,
                "batch_start" => LogLevel::BatchStart,
                "batch_complete" => LogLevel::BatchComplete,
                "batch_error" => LogLevel::BatchError,
                "success" => LogLevel::Success,
                "status" => LogLevel::Status,
                // "info" and anything else
                _ => LogLevel::Info,
            };

            let entry = LogEntry::with_batch(&execution_id, level, message, batch_id, batch_name);
            if let Err(e) = self.log_store.append(&execution_id, &entry) {
                eprintln!("Warning: Failed to persist log entry: {}", e);
            }
        }
    }

    #[allow(clippy::unused_self)]
    fn save_plan(&self, plan_path: &Path, plan: &ExecutionPlan) -> Result<()> {
        let content = serde_yaml::to_string(plan)?;
        std::fs::write(plan_path, content)?;
        Ok(())
    }

    /// Execute a specification's plan by scheduling and running all batches.
    ///
    /// # Errors
    ///
    /// Returns an error if the spec or plan files cannot be loaded,
    /// or if a batch execution fails and is not recoverable.
    pub async fn run_spec(
        &self,
        spec_name: String,
        dry_run: bool,
        executor_model: Option<String>,
        agent: String,
        existing_run_id: Option<String>,
    ) -> Result<()> {
        let spec_path = self
            .project_root
            .join(".specs")
            .join(&spec_name)
            .join("spec.yaml");
        let tasks_path = self
            .project_root
            .join(".specs")
            .join(&spec_name)
            .join("tasks.yaml");
        let plan_path = self
            .project_root
            .join(".specs")
            .join(&spec_name)
            .join("plan.yaml");

        if !spec_path.exists() {
            self.log("error", &format!("Spec not found: {}", spec_name))
                .await;
            return Err(anyhow!("Spec not found"));
        }

        if !plan_path.exists() {
            self.log("error", "Plan not found. Run 'ckrv plan' first.")
                .await;
            return Err(anyhow!("Plan not found"));
        }

        self.log(
            "start",
            &format!("Starting execution for spec: {}", spec_name),
        )
        .await;

        // T006: Send explicit status message so frontend knows execution is running
        let _ = self.sender.send(LogMessage::status("running")).await;

        // Load plan
        let plan_content = std::fs::read_to_string(&plan_path)?;
        let mut plan: ExecutionPlan = serde_yaml::from_str(&plan_content)?;

        // Load tasks to map IDs to details
        let tasks_content = std::fs::read_to_string(&tasks_path)?;
        let task_file: TaskFile = serde_yaml::from_str(&tasks_content)?;
        let task_map: HashMap<String, SpecTask> = task_file
            .tasks
            .into_iter()
            .map(|t| (t.id.clone(), t))
            .collect();

        // T016: Initialize history service and create/resume run
        let history_service = HistoryService::new(&self.project_root);

        let run_id = if let Some(id) = existing_run_id {
            // T032: Resume existing run - sync plan with history
            match history_service.get_run(&spec_name, &id) {
                Ok(Some(run)) => {
                    // Sync plan with history: completed stays completed, others reset to pending
                    for batch in &mut plan.batches {
                        if let Some(result) = run.batches.iter().find(|b| b.id == batch.id) {
                            match result.status {
                                HistoryBatchStatus::Completed => {
                                    batch.status = BatchStatus::Completed;
                                }
                                _ => {
                                    // Reset non-completed batches so they run again
                                    batch.status = BatchStatus::Pending;
                                }
                            }
                        }
                    }
                    self.save_plan(&plan_path, &plan)?;

                    // Update history status to Running
                    let _ = history_service.update_run(&spec_name, &id, |r| {
                        r.status = RunStatus::Running;
                        r.ended_at = None;
                        r.error = None;
                    });

                    self.log("info", &format!("Resuming run: {}", id)).await;
                    id
                }
                Ok(None) => {
                    self.log("warning", &format!("Run {} not found, starting fresh", id))
                        .await;
                    Run::generate_id()
                }
                Err(e) => {
                    self.log(
                        "warning",
                        &format!("Failed to load run {}, starting fresh: {}", id, e),
                    )
                    .await;
                    // Fall back to creating new run
                    Run::generate_id()
                }
            }
        } else {
            let id = Run::generate_id();
            let batch_info: Vec<(String, String)> = plan
                .batches
                .iter()
                .map(|b| (b.id.clone(), b.name.clone()))
                .collect();

            // Create run entry (best-effort - don't fail execution if history fails)
            match history_service.create_run(&spec_name, &id, batch_info, dry_run) {
                Ok(run) => {
                    self.log("info", &format!("Created run history entry: {}", run.id))
                        .await;
                }
                Err(e) => {
                    self.log("warning", &format!("Failed to create history entry: {}", e))
                        .await;
                }
            }
            id
        };

        // T012: Set execution ID for log persistence
        self.set_execution_id(&run_id);

        // Initialize Worktree Manager
        let _manager = DefaultWorktreeManager::new(&self.project_root)
            .context("Failed to init worktree manager")?;
        let exe = std::env::current_exe()?; // Self-reference for spawning tasks?
                                            // Wait, self-referencing implies `ckrv task` is available.
                                            // If we are running in `ckrv-ui` binary, we can't assume it supports `task` subcommand.
                                            // We must check if we are running `ckrv` CLI or `ckrv-ui`.
                                            // If `ckrv-ui` does not support `task` subcommand, we need to find `ckrv` binary.

        // Try to find ckrv in the same directory as the current executable, or use PATH
        let ckrv_exe = exe.parent().map_or_else(
            || PathBuf::from("ckrv"),
            |parent| {
                let candidate = parent.join("ckrv");
                if candidate.exists() {
                    candidate
                } else {
                    PathBuf::from("ckrv")
                }
            },
        );

        let mut completed_batches = HashSet::new();
        let mut batch_task_map = HashMap::new();

        // Populate initial state from plan (if resuming)
        for batch in &plan.batches {
            batch_task_map.insert(batch.id.clone(), batch.task_ids.clone());
            if batch.status == BatchStatus::Completed {
                completed_batches.insert(batch.id.clone());
            }
        }

        let count = completed_batches.len();
        if count > 0 {
            self.log(
                "info",
                &format!("Resuming: {} batches already completed", count),
            )
            .await;
        }

        let mut pending_batches: VecDeque<_> = plan
            .batches
            .iter()
            .filter(|b| b.status != BatchStatus::Completed)
            .cloned()
            .collect();

        let mut running_futures = FuturesUnordered::new();

        // Loop until all done
        while !pending_batches.is_empty() || !running_futures.is_empty() {
            // 1. Spawn unblocked
            let mut still_pending = VecDeque::new();

            while let Some(batch) = pending_batches.pop_front() {
                let unblocked = batch
                    .depends_on
                    .iter()
                    .all(|d| completed_batches.contains(d));

                if unblocked {
                    self.log("batch_start", &format!("Spawning batch: {}", batch.name))
                        .await;

                    // T011: Send explicit batch status so frontend updates batch card
                    let _ = self
                        .sender
                        .send(LogMessage::batch_status(&batch.id, &batch.name, "running"))
                        .await;

                    // Update status to running in plan file
                    self.update_batch_status(&plan_path, &batch.id, BatchStatus::Running, None)?;

                    let batch_clone = batch.clone();
                    let task_map_clone = task_map.clone();
                    let exe_path = ckrv_exe.clone();
                    let project_root = self.project_root.clone();
                    let executor_model = executor_model.clone();
                    let agent = agent.clone();
                    let sender = self.sender.clone();
                    let execution_id = self
                        .current_execution_id
                        .lock()
                        .ok()
                        .and_then(|g| g.clone());
                    let batch_id_for_error = batch.id.clone(); // Capture for error handling

                    // Spawn the batch execution - wrap result to always include batch_id
                    running_futures.push(tokio::spawn(async move {
                        let result = Self::execute_batch(
                            project_root,
                            exe_path,
                            batch_clone,
                            task_map_clone,
                            dry_run,
                            true, // use_sandbox: always use Docker
                            executor_model,
                            agent,
                            sender,
                            execution_id,
                        )
                        .await;
                        // Wrap error to include batch_id
                        result.map_err(|e| (batch_id_for_error, e))
                    }));
                } else {
                    still_pending.push_back(batch);
                }
            }
            pending_batches = still_pending;

            // 2. Wait for completion
            if let Some(result) = running_futures.next().await {
                // internal join handle result
                match result {
                    Ok(batch_result) => {
                        match batch_result {
                            Ok((batch_id, branch_name)) => {
                                // Batch succeeded
                                self.log(
                                    "batch_complete",
                                    &format!(
                                        "Batch {} completed on branch {}",
                                        batch_id, branch_name
                                    ),
                                )
                                .await;

                                // T012: Send explicit batch status so frontend updates counter
                                let _ = self
                                    .sender
                                    .send(
                                        LogMessage::batch_status(&batch_id, &batch_id, "completed")
                                            .with_branch(&branch_name),
                                    )
                                    .await;

                                // Update state
                                completed_batches.insert(batch_id.clone());

                                if !dry_run {
                                    // Merge Logic Here
                                    self.merge_batch(&branch_name, &spec_path).await?;

                                    // Mark tasks complete
                                    if let Some(tids) = batch_task_map.get(&batch_id) {
                                        self.mark_tasks_complete(&tasks_path, tids)?;
                                    }

                                    self.update_batch_status(
                                        &plan_path,
                                        &batch_id,
                                        BatchStatus::Completed,
                                        Some(&branch_name),
                                    )?;
                                }

                                // T017: Update history with batch completion
                                {
                                    let run_id = &run_id;
                                    let _ = history_service.update_batch_status(
                                        &spec_name,
                                        run_id,
                                        &batch_id,
                                        HistoryBatchStatus::Completed,
                                        Some(&branch_name),
                                        None,
                                    );
                                }
                            }
                            Err((failed_batch_id, e)) => {
                                self.log(
                                    "batch_error",
                                    &format!("Batch {} failed: {}", failed_batch_id, e),
                                )
                                .await;

                                // Update plan.yaml with failed status
                                let _ = self.update_batch_status(
                                    &plan_path,
                                    &failed_batch_id,
                                    BatchStatus::Failed,
                                    None,
                                );

                                // Send batch_status message so frontend updates the batch pill
                                let _ = self
                                    .sender
                                    .send(
                                        LogMessage::batch_status(
                                            &failed_batch_id,
                                            &failed_batch_id,
                                            "failed",
                                        )
                                        .with_error(&e.to_string()),
                                    )
                                    .await;

                                // T015: Send status failed so frontend stops timer and shows error
                                let _ = self.sender.send(LogMessage::status("failed")).await;

                                // T018: Update history with run failure
                                {
                                    let run_id = &run_id;
                                    let _ = history_service.fail_run(
                                        &spec_name,
                                        run_id,
                                        &e.to_string(),
                                    );
                                }

                                return Err(e);
                            }
                        }
                    }
                    Err(e) => {
                        // T015: Send status failed for task panics
                        let _ = self.sender.send(LogMessage::status("failed")).await;

                        // T018: Update history with run failure
                        {
                            let run_id = &run_id;
                            let _ = history_service.fail_run(
                                &spec_name,
                                run_id,
                                &format!("Task panic: {}", e),
                            );
                        }

                        return Err(anyhow!("Task panic: {}", e));
                    }
                }
            } else if !pending_batches.is_empty() {
                // T015: Send status failed for deadlocks
                let _ = self.sender.send(LogMessage::status("failed")).await;

                // T018: Update history with run failure
                {
                    let run_id = &run_id;
                    let _ = history_service.fail_run(
                        &spec_name,
                        run_id,
                        &format!("Deadlock: {} batches pending", pending_batches.len()),
                    );
                }

                return Err(anyhow!(
                    "Deadlock: {} batches pending but none can run.",
                    pending_batches.len()
                ));
            }
        }

        // T014: Send explicit status completed so frontend knows execution is done
        let _ = self.sender.send(LogMessage::status("completed")).await;

        // T018: Update history with run completion
        {
            let run_id = &run_id;
            let _ = history_service.complete_run(&spec_name, run_id);
            self.log("info", &format!("Run history entry completed: {}", run_id))
                .await;
        }

        // Create implementation.yaml to mark the spec as implemented
        // This enables the View Diff, Verify, and Create PR buttons in the UI
        let impl_yaml_path = self
            .project_root
            .join(".specs")
            .join(&spec_name)
            .join("implementation.yaml");
        let impl_content = format!(
            "status: completed\nbranch: {}\ncompleted_at: {}\nrun_id: {}\n",
            spec_name, // The spec name is used as the implementation branch
            chrono::Utc::now().to_rfc3339(),
            run_id
        );
        if let Err(e) = std::fs::write(&impl_yaml_path, impl_content) {
            self.log(
                "warning",
                &format!("Failed to create implementation.yaml: {}", e),
            )
            .await;
        } else {
            self.log(
                "info",
                &format!("Created implementation.yaml for spec {}", spec_name),
            )
            .await;
        }

        self.log("success", "All batches completed successfully.")
            .await;
        Ok(())
    }

    // Separate function to execute a single batch
    async fn execute_batch(
        root: PathBuf,
        exe: PathBuf,
        batch: Batch,
        task_map: HashMap<String, SpecTask>,
        dry_run: bool,
        use_sandbox: bool, // NEW: Use Docker sandbox for execution
        executor_model: Option<String>,
        agent: String, // Agent to use: "claude" or "codex"
        sender: mpsc::Sender<LogMessage>,
        execution_id: Option<String>, // For log persistence
    ) -> Result<(String, String)> {
        // Returns (batch_id, branch_name)

        // Construct description
        let mut description = format!(
            "MISSION: {}\nREASONING: {}\n\nTASKS:\n",
            batch.name, batch.reasoning
        );
        for id in &batch.task_ids {
            if let Some(t) = task_map.get(id) {
                use std::fmt::Write;
                let _ = writeln!(description, "- [{}]: {} ({})", t.id, t.title, t.description);
            }
        }

        if dry_run {
            // Simulate delay
            tokio::time::sleep(Duration::from_millis(500)).await;
            return Ok((batch.id, "dry-run-branch".to_string()));
        }

        // Create Worktree using spawn_blocking to avoid blocking the async runtime
        // (git2 is synchronous and would otherwise block all async tasks)
        let root_clone = root.clone();
        let suffix: String = uuid::Uuid::new_v4().to_string().chars().take(6).collect();
        let wt_job_id = format!("batch-{}-{}", batch.id, suffix);
        let wt_job_id_clone = wt_job_id.clone();

        let worktree = tokio::task::spawn_blocking(move || {
            let manager =
                DefaultWorktreeManager::new(&root_clone).context("Failed to init wt manager")?;
            manager
                .create(&wt_job_id_clone, "1")
                .context("Failed to create worktree")
        })
        .await
        .context("Worktree task panicked")??;

        let branch_name = worktree.branch.clone();

        // Execute 'ckrv task' in that worktree
        let batch_run_id = format!("{}-run", batch.id);

        // Build the command arguments
        let mut task_args = vec![
            "task".to_string(),
            description.clone(),
            "--use-worktree".to_string(),
            worktree.path.to_string_lossy().to_string(),
            "--continue-task".to_string(),
            batch_run_id.clone(),
        ];

        // Determine the model to use
        let model = executor_model.or_else(|| batch.model_assignment.default.clone());

        if let Some(ref m) = model {
            task_args.push("--agent".to_string());
            task_args.push(m.clone());
        }

        if use_sandbox {
            // Determine agent type from the passed agent parameter
            let is_codex = agent == "codex";

            // Determine if this is an OpenRouter model or native Claude
            let is_openrouter = model
                .as_ref()
                .map(|m| {
                    m.contains('/')
                        && !m.starts_with("claude")
                        && !m.starts_with("glm")
                        && !is_codex
                })
                .unwrap_or(false);

            // Determine if this is a GLM Coding Plan model
            let is_glm = model
                .as_ref()
                .map(|m| m.starts_with("glm-") || m.starts_with("glm_"))
                .unwrap_or(false);

            let agent_name = if is_codex {
                "OpenAI Codex"
            } else if is_glm {
                "GLM Coding Plan"
            } else {
                "Claude Code"
            };
            let _ = sender
                .send(
                    LogMessage::new(
                        "info",
                        &format!("Executing in Docker sandbox with {}...", agent_name),
                    )
                    .with_batch(&batch.id, &batch.name),
                )
                .await;

            // Try to create Docker sandbox, fall back to local if unavailable
            match DockerSandbox::with_defaults() {
                Ok(sandbox) => {
                    // Build the agent command
                    // Use --print and appropriate flags for non-interactive execution
                    let agent_prompt = format!(
                        "You are implementing code changes in a project. Follow these instructions exactly:\n\n{}\n\nMake all changes to the files in /workspace. Do not ask questions - implement the code directly.",
                        description
                    );

                    let escaped_prompt = agent_prompt.replace('"', "\\\"");

                    // Base command based on agent type
                    let (cmd, agent_binary) = if is_codex {
                        // Codex CLI command
                        let cmd =
                            format!("codex --print --full-auto --quiet \"{}\"", escaped_prompt);
                        (cmd, "codex")
                    } else {
                        // Claude Code CLI command with streaming JSON output
                        let cmd = format!(
                            "claude --print --verbose --output-format stream-json --dangerously-skip-permissions \"{}\"",
                            escaped_prompt
                        );
                        (cmd, "claude")
                    };

                    let mut cfg = ExecuteConfig::new(agent_binary, worktree.path.clone())
                        .shell(&cmd)
                        .with_timeout(Duration::from_secs(900)); // 15 minute timeout

                    // Configure execution based on agent type
                    if is_codex {
                        // Codex path: Use mounted credentials from ~/.codex
                        let _ = sender
                            .send(
                                LogMessage::new(
                                    "info",
                                    "Using OpenAI Codex with mounted credentials",
                                )
                                .with_batch(&batch.id, &batch.name),
                            )
                            .await;

                        // Set model if specified
                        if let Some(ref model_name) = model {
                            cfg = cfg.env("OPENAI_MODEL", model_name);
                        }
                    } else if is_glm {
                        // GLM Coding Plan path: Use Claude Code CLI with Z.AI env vars
                        // Per https://docs.z.ai/devpack/tool/claude#manual-configuration
                        let model_name = model
                            .as_ref()
                            .ok_or_else(|| anyhow::anyhow!("GLM agent requires a model name"))?;
                        let _ = sender
                            .send(
                                LogMessage::new(
                                    "info",
                                    &format!("Using GLM Coding Plan model: {}", model_name),
                                )
                                .with_batch(&batch.id, &batch.name),
                            )
                            .await;

                        // Get GLM API key from environment or agent config
                        let api_key = std::env::var("ZAI_API_KEY")
                            .or_else(|_| std::env::var("GLM_API_KEY"))
                            .ok()
                            .or_else(|| Self::find_glm_key(model_name));

                        if let Some(key) = api_key {
                            // Required env vars for Z.AI GLM Coding Plan
                            cfg = cfg.env("ANTHROPIC_BASE_URL", "https://api.z.ai/api/anthropic");
                            cfg = cfg.env("ANTHROPIC_AUTH_TOKEN", &key);
                            cfg = cfg.env("ANTHROPIC_API_KEY", ""); // Must be explicitly empty!
                            cfg = cfg.env("API_TIMEOUT_MS", "3000000"); // Extended timeout for GLM

                            // Set model for all tiers
                            cfg = cfg.env("ANTHROPIC_DEFAULT_SONNET_MODEL", model_name);
                            cfg = cfg.env("ANTHROPIC_DEFAULT_OPUS_MODEL", model_name);
                            cfg = cfg.env("ANTHROPIC_DEFAULT_HAIKU_MODEL", model_name);
                        } else {
                            let _ = sender
                                .send(
                                    LogMessage::new(
                                        "warning",
                                        "No ZAI_API_KEY or GLM_API_KEY found, execution may fail",
                                    )
                                    .with_batch(&batch.id, &batch.name),
                                )
                                .await;
                        }
                    } else if is_openrouter {
                        // OpenRouter path: Use Claude Code CLI with OpenRouter env vars
                        // Per https://openrouter.ai/docs/guides/guides/claude-code-integration
                        let model_name = model.as_ref().ok_or_else(|| {
                            anyhow::anyhow!("OpenRouter agent requires a model name")
                        })?;
                        let _ = sender
                            .send(
                                LogMessage::new(
                                    "info",
                                    &format!("Using OpenRouter model: {}", model_name),
                                )
                                .with_batch(&batch.id, &batch.name),
                            )
                            .await;

                        // Get OpenRouter API key from environment or agent config
                        let api_key = std::env::var("OPENROUTER_API_KEY").ok().or_else(|| {
                            let agents_dir = root.join(".agents");
                            Self::find_openrouter_key(&agents_dir, model_name)
                        });

                        if let Some(key) = api_key {
                            // Required env vars for OpenRouter (same as runner.rs)
                            cfg = cfg.env("ANTHROPIC_BASE_URL", "https://openrouter.ai/api");
                            cfg = cfg.env("ANTHROPIC_AUTH_TOKEN", &key);
                            cfg = cfg.env("ANTHROPIC_API_KEY", ""); // Must be explicitly empty!

                            // Set model for all tiers
                            cfg = cfg.env("ANTHROPIC_DEFAULT_SONNET_MODEL", model_name);
                            cfg = cfg.env("ANTHROPIC_DEFAULT_OPUS_MODEL", model_name);
                            cfg = cfg.env("ANTHROPIC_DEFAULT_HAIKU_MODEL", model_name);
                        } else {
                            let _ = sender
                                .send(
                                    LogMessage::new(
                                        "warning",
                                        "No OPENROUTER_API_KEY found, execution may fail",
                                    )
                                    .with_batch(&batch.id, &batch.name),
                                )
                                .await;
                        }
                    } else {
                        // Claude subscription path: Use native Claude Code auth via ~/.claude
                        let _ = sender
                            .send(
                                LogMessage::new("info", "Using Claude subscription")
                                    .with_batch(&batch.id, &batch.name),
                            )
                            .await;
                    }

                    // Set HOME for Claude Code config
                    let config = cfg.env("HOME", "/home/claude");
                    let config = config.env("NO_COLOR", "1");

                    // Execute in sandbox with real-time streaming
                    // Create a channel to forward log messages from the callback
                    let (log_tx, mut log_rx) = tokio::sync::mpsc::channel::<(String, bool)>(100);

                    // Clone sender for forwarding task
                    let sender_clone = sender.clone();
                    let batch_id_clone = batch.id.clone();
                    let batch_name_clone = batch.name.clone();

                    // Clone log_store path for persistence in spawned task
                    let log_store_path = root.clone();
                    let execution_id_clone = execution_id.clone();

                    // Spawn a task to forward logs to the WebSocket sender AND persist to disk
                    let forward_handle = tokio::spawn(async move {
                        let log_store = LogStore::new(&log_store_path);

                        while let Some((line, is_stderr)) = log_rx.recv().await {
                            // Parse streaming JSON if possible, otherwise forward as-is
                            let log_type = if is_stderr { "error" } else { "log" };

                            // Determine the message to send and log level
                            let (msg, level_str): (String, &str) = serde_json::from_str::<serde_json::Value>(&line)
                                .map_or_else(
                                    |_| (line.clone(), log_type),
                                    |json| {
                                        // Extract text content from Claude's JSON output
                                        json.get("result")
                                            .and_then(|content| content.as_str())
                                            .map_or_else(
                                                || {
                                                    json.pointer("/type")
                                                        .and_then(|v| v.as_str())
                                                        .and_then(|tool_name| {
                                                            if tool_name == "tool_use" {
                                                                json.pointer("/content/0/name")
                                                                    .and_then(|v| v.as_str())
                                                                    .map(|name| (format!("\u{1f527} Using tool: {}", name), "info"))
                                                            } else {
                                                                None
                                                            }
                                                        })
                                                        .unwrap_or_else(|| (line.clone(), log_type))
                                                },
                                                |text| (text.to_string(), "log"),
                                            )
                                    },
                                );

                            // Send to WebSocket
                            let _ = sender_clone
                                .send(
                                    LogMessage::new(level_str, &msg)
                                        .with_batch(&batch_id_clone, &batch_name_clone),
                                )
                                .await;

                            // Persist to disk
                            if let Some(ref exec_id) = execution_id_clone {
                                let level = match level_str {
                                    "info" => LogLevel::Info,
                                    "error" => LogLevel::Error,
                                    // "log" and anything else
                                    _ => LogLevel::Log,
                                };
                                let entry = LogEntry::with_batch(
                                    exec_id,
                                    level,
                                    &msg,
                                    &batch_id_clone,
                                    &batch_name_clone,
                                );
                                let _ = log_store.append(exec_id, &entry);
                            }
                        }
                    });

                    // Execute with streaming callback
                    let execute_result = sandbox
                        .execute_streaming(config, |line, is_stderr| {
                            // Send log line to the forwarder (non-blocking)
                            let _ = log_tx.try_send((line.to_string(), is_stderr));
                        })
                        .await;

                    // Drop the sender to signal the forwarder to stop
                    drop(log_tx);

                    // Wait for forwarder to finish
                    let _ = forward_handle.await;

                    match execute_result {
                        Ok(result) => {
                            if !result.success() {
                                return Err(anyhow!(
                                    "Claude Code execution failed with exit code {}",
                                    result.exit_code
                                ));
                            }
                        }
                        Err(e) => {
                            let _ = sender
                                .send(
                                    LogMessage::new(
                                        "error",
                                        &format!("Sandbox execution error: {}", e),
                                    )
                                    .with_batch(&batch.id, &batch.name),
                                )
                                .await;
                            return Err(anyhow!("Sandbox execution failed: {}", e));
                        }
                    }
                }
                Err(e) => {
                    // Fall back to local execution if Docker is not available
                    let _ = sender
                        .send(
                            LogMessage::new(
                                "warning",
                                &format!(
                                    "Docker unavailable ({}), falling back to local execution",
                                    e
                                ),
                            )
                            .with_batch(&batch.id, &batch.name),
                        )
                        .await;
                    Self::execute_local(&exe, &task_args, &batch.id, &batch.name, &sender).await?;
                }
            }
        } else {
            // Local execution (no sandbox) - uses ckrv task
            Self::execute_local(&exe, &task_args, &batch.id, &batch.name, &sender).await?;
        }

        // Commit changes inside the worktree
        AsyncCommand::new("git")
            .arg("add")
            .arg(".")
            .current_dir(&worktree.path)
            .status()
            .await?;

        let commit_msg = format!("feat(batch): {} - {}", batch.name, batch.id);
        AsyncCommand::new("git")
            .args(["commit", "-m", &commit_msg])
            .current_dir(&worktree.path)
            .status()
            .await?;

        Ok((batch.id, branch_name))
    }

    /// Execute command locally (no sandbox)
    async fn execute_local(
        exe: &Path,
        args: &[String],
        batch_id: &str,
        batch_name: &str,
        sender: &mpsc::Sender<LogMessage>,
    ) -> Result<()> {
        use std::process::Stdio;

        let mut cmd = AsyncCommand::new(exe);
        for arg in args {
            cmd.arg(arg);
        }
        cmd.stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .env("NO_COLOR", "1");

        let mut child = cmd.spawn()?;

        let stdout = child.stdout.take();
        let stderr = child.stderr.take();

        let sender_out = sender.clone();
        let sender_err = sender.clone();
        let batch_id_out = batch_id.to_string();
        let batch_name_out = batch_name.to_string();
        let batch_id_err = batch_id.to_string();
        let batch_name_err = batch_name.to_string();

        // Spawn stdout reader
        let stdout_handle = stdout.map(|out| {
            tokio::spawn(async move {
                use tokio::io::{AsyncBufReadExt, BufReader};
                let mut reader = BufReader::new(out).lines();
                while let Ok(Some(line)) = reader.next_line().await {
                    let _ = sender_out
                        .send(
                            LogMessage::new("log", &line)
                                .with_batch(&batch_id_out, &batch_name_out),
                        )
                        .await;
                }
            })
        });

        // Spawn stderr reader
        let stderr_handle = stderr.map(|err| {
            tokio::spawn(async move {
                use tokio::io::{AsyncBufReadExt, BufReader};
                let mut reader = BufReader::new(err).lines();
                while let Ok(Some(line)) = reader.next_line().await {
                    let _ = sender_err
                        .send(
                            LogMessage::new("error", &line)
                                .with_batch(&batch_id_err, &batch_name_err),
                        )
                        .await;
                }
            })
        });

        let status = child.wait().await?;

        // Wait for I/O to finish
        if let Some(h) = stdout_handle {
            let _ = h.await;
        }
        if let Some(h) = stderr_handle {
            let _ = h.await;
        }

        if !status.success() {
            return Err(anyhow!("Task failed with exit code {:?}", status.code()));
        }

        Ok(())
    }

    async fn merge_batch(&self, branch: &str, spec_path: &Path) -> Result<()> {
        self.log("info", &format!("Merging branch {}", branch))
            .await;

        let status = AsyncCommand::new("git")
            .args(["merge", "--no-ff", "--no-edit", branch])
            .current_dir(&self.project_root)
            .status()
            .await?;

        if !status.success() {
            if self.has_merge_conflicts().await {
                self.log(
                    "info",
                    "Merge conflicts detected. Attempting AI resolution...",
                )
                .await;
                self.resolve_conflicts(branch, spec_path).await?;
            } else {
                return Err(anyhow!("Merge failed"));
            }
        }
        Ok(())
    }

    async fn has_merge_conflicts(&self) -> bool {
        let output = AsyncCommand::new("git")
            .args(["diff", "--name-only", "--diff-filter=U"])
            .current_dir(&self.project_root)
            .output()
            .await;

        match output {
            Ok(o) => !o.stdout.is_empty(),
            Err(_) => false,
        }
    }

    async fn resolve_conflicts(&self, _branch: &str, spec_path: &Path) -> Result<()> {
        // Gather conflicts
        let output = AsyncCommand::new("git")
            .args(["diff", "--name-only", "--diff-filter=U"])
            .current_dir(&self.project_root)
            .output()
            .await?;

        let files = String::from_utf8_lossy(&output.stdout);
        let file_list: Vec<&str> = files.lines().collect();

        if file_list.is_empty() {
            return Ok(());
        }

        // Prompt construction
        let spec_content = std::fs::read_to_string(spec_path).unwrap_or_default();
        let prompt = format!(
            "You are resolving Git merge conflicts for files: {:?}.\nSpec: {}\n\nResolve markers <<<<<<< ======= >>>>>>> and stage files.",
            file_list, spec_content
        );

        // Run Claude Code in Sandbox
        // We use ckrv-sandbox here
        let sandbox =
            DockerSandbox::new(DefaultAllowList::default()).context("Failed to create sandbox")?;

        let escaped_prompt = shell_escape::escape(prompt.into());
        let command = format!(
            "echo {} | claude -p - --dangerously-skip-permissions",
            escaped_prompt
        );

        let config = ExecuteConfig::new("", self.project_root.clone())
            .shell(&command)
            .with_timeout(Duration::from_secs(300));

        let result = sandbox
            .execute(config)
            .await
            .context("Sandbox execution failed")?;

        if !result.success() {
            self.log(
                "error",
                &format!("AI conflict resolution failed: {}", result.stderr),
            )
            .await;
        }

        if self.has_merge_conflicts().await {
            return Err(anyhow!("AI could not resolve all conflicts"));
        }

        // Commit
        AsyncCommand::new("git")
            .args(["commit", "--no-edit"])
            .current_dir(&self.project_root)
            .status()
            .await?;

        Ok(())
    }

    #[allow(clippy::unused_self)]
    fn update_batch_status(
        &self,
        plan_path: &Path,
        batch_id: &str,
        status: BatchStatus,
        branch: Option<&str>,
    ) -> Result<()> {
        let content = std::fs::read_to_string(plan_path)?;
        let mut plan: ExecutionPlan = serde_yaml::from_str(&content)?;

        for batch in &mut plan.batches {
            if batch.id == batch_id {
                batch.status = status;
                if let Some(b) = branch {
                    batch.branch = Some(b.to_string());
                }
            }
        }

        // This helper fn serialization is needed?
        // Let's just use serde_yaml directly for now
        let yaml = serde_yaml::to_string(&plan)?;
        std::fs::write(plan_path, yaml)?;
        Ok(())
    }

    #[allow(clippy::unused_self)]
    fn mark_tasks_complete(&self, tasks_path: &Path, ids: &[String]) -> Result<()> {
        let content = std::fs::read_to_string(tasks_path)?;
        let mut file: TaskFile = serde_yaml::from_str(&content)?;
        let mut updated = false;

        for task in &mut file.tasks {
            if ids.contains(&task.id) {
                task.status = "completed".to_string();
                updated = true;
            }
        }

        if updated {
            std::fs::write(tasks_path, serde_yaml::to_string(&file)?)?;
        }
        Ok(())
    }

    /// Find OpenRouter API key from agent config files
    /// Checks global config at ~/.config/chakravarti/agents.yaml
    fn find_openrouter_key(_agents_dir: &Path, model: &str) -> Option<String> {
        // Agent config structures matching the actual format
        #[derive(serde::Deserialize)]
        struct AgentsFile {
            agents: Vec<AgentEntry>,
        }

        #[derive(serde::Deserialize)]
        struct AgentEntry {
            #[allow(dead_code)]
            id: String,
            #[allow(dead_code)]
            agent_type: String,
            openrouter: Option<OpenRouterConfig>,
        }

        #[derive(serde::Deserialize)]
        struct OpenRouterConfig {
            api_key: Option<String>,
            model: Option<String>,
        }

        // Check global config path first (same as task.rs)
        let agents_path = dirs::config_dir()
            .map(|d| d.join("chakravarti").join("agents.yaml"))
            .filter(|p| p.exists());

        if let Some(path) = agents_path {
            if let Ok(content) = std::fs::read_to_string(&path) {
                if let Ok(file) = serde_yaml::from_str::<AgentsFile>(&content) {
                    // First try to find the specific agent matching the model
                    for agent in &file.agents {
                        if let Some(ref or) = agent.openrouter {
                            if or.model.as_ref().map(|m| m == model).unwrap_or(false) {
                                if let Some(ref key) = or.api_key {
                                    return Some(key.clone());
                                }
                            }
                        }
                    }

                    // If no specific match, return any OpenRouter key we find
                    for agent in &file.agents {
                        if let Some(ref or) = agent.openrouter {
                            if let Some(ref key) = or.api_key {
                                return Some(key.clone());
                            }
                        }
                    }
                }
            }
        }

        None
    }

    /// Find GLM API key from agent config files
    /// Checks global config at ~/.config/chakravarti/agents.yaml
    fn find_glm_key(model: &str) -> Option<String> {
        // Agent config structures matching the actual format
        #[derive(serde::Deserialize)]
        struct AgentsFile {
            agents: Vec<AgentEntry>,
        }

        #[derive(serde::Deserialize)]
        struct AgentEntry {
            #[allow(dead_code)]
            id: String,
            #[allow(dead_code)]
            agent_type: String,
            glm: Option<GLMConfig>,
        }

        #[derive(serde::Deserialize)]
        struct GLMConfig {
            api_key: Option<String>,
            model: Option<String>,
        }

        // Check global config path
        let agents_path = dirs::config_dir()
            .map(|d| d.join("chakravarti").join("agents.yaml"))
            .filter(|p| p.exists());

        if let Some(path) = agents_path {
            if let Ok(content) = std::fs::read_to_string(&path) {
                if let Ok(file) = serde_yaml::from_str::<AgentsFile>(&content) {
                    // First try to find the specific agent matching the model
                    for agent in &file.agents {
                        if let Some(ref glm) = agent.glm {
                            if glm.model.as_ref().map(|m| m == model).unwrap_or(false) {
                                if let Some(ref key) = glm.api_key {
                                    return Some(key.clone());
                                }
                            }
                        }
                    }

                    // If no specific match, return any GLM key we find
                    for agent in &file.agents {
                        if let Some(ref glm) = agent.glm {
                            if let Some(ref key) = glm.api_key {
                                return Some(key.clone());
                            }
                        }
                    }
                }
            }
        }

        None
    }
}
