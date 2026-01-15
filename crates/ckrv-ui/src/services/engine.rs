use std::path::{Path, PathBuf};
use std::collections::{HashMap, HashSet, VecDeque};
use std::time::Duration;

use anyhow::{Context, Result, anyhow};
use futures::stream::FuturesUnordered;
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use tokio::process::Command as AsyncCommand;
use chrono::Utc;

use ckrv_git::{WorktreeManager, DefaultWorktreeManager};
use ckrv_sandbox::{DockerSandbox, ExecuteConfig, Sandbox, DefaultAllowList};

use crate::services::history::HistoryService;
use crate::models::history::{Run, RunStatus, HistoryBatchStatus};

/// Status of a batch in the execution plan
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum BatchStatus {
    Pending,
    Running,
    Completed,
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
            "" | "pending" => Ok(BatchStatus::Pending),
            "running" => Ok(BatchStatus::Running),
            "completed" => Ok(BatchStatus::Completed),
            "failed" => Ok(BatchStatus::Failed),
            _ => Ok(BatchStatus::Pending), // Default to pending for unknown values
        }
    }
}

/// Execution plan structure (plan.yaml)
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ExecutionPlan {
    #[serde(default)]
    pub spec_id: Option<String>,
    pub batches: Vec<Batch>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Batch {
    pub id: String,
    pub name: String,
    pub task_ids: Vec<String>,
    #[serde(default)]
    pub depends_on: Vec<String>,
    pub status: BatchStatus,
    #[serde(default)]
    pub branch: Option<String>,
    pub reasoning: String,
    pub model_assignment: ModelAssignment,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModelAssignment {
    pub default: Option<String>,
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

/// Log message structure for streaming updates
#[derive(Debug, Clone, Serialize)]
pub struct LogMessage {
    #[serde(rename = "type")]
    pub type_: String, // "info", "success", "error", "start", "batch_start", "status", "batch_status", etc.
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<String>, // stdout/stderr
    pub timestamp: String,
    // T004: New fields for explicit status messages
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>, // "running", "completed", "failed", "aborted"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub batch_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub batch_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub branch: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl LogMessage {
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
}

pub struct ExecutionEngine {
    project_root: PathBuf,
    sender: mpsc::Sender<LogMessage>,
}

impl ExecutionEngine {
    pub fn new(project_root: PathBuf, sender: mpsc::Sender<LogMessage>) -> Self {
        Self {
            project_root,
            sender,
        }
    }

    async fn log(&self, type_: &str, message: &str) {
        let _ = self.sender.send(LogMessage::new(type_, message)).await;
        // Also print to server stdout for debugging
        println!("[ExecutionEngine] {}: {}", type_, message);
    }

    fn save_plan(&self, plan_path: &Path, plan: &ExecutionPlan) -> Result<()> {
        let content = serde_yaml::to_string(plan)?;
        std::fs::write(plan_path, content)?;
        Ok(())
    }

    pub async fn run_spec(
        &self,
        spec_name: String,
        dry_run: bool,
        executor_model: Option<String>,
        existing_run_id: Option<String>, // T032: Resume existing run
    ) -> Result<()> {
        let spec_path = self.project_root.join(".specs").join(&spec_name).join("spec.yaml");
        let tasks_path = self.project_root.join(".specs").join(&spec_name).join("tasks.yaml");
        let plan_path = self.project_root.join(".specs").join(&spec_name).join("plan.yaml");
        
        if !spec_path.exists() {
            self.log("error", &format!("Spec not found: {}", spec_name)).await;
            return Err(anyhow!("Spec not found"));
        }

        if !plan_path.exists() {
            self.log("error", "Plan not found. Run 'ckrv plan' first.").await;
            return Err(anyhow!("Plan not found"));
        }

        self.log("start", &format!("Starting execution for spec: {}", spec_name)).await;
        
        // T006: Send explicit status message so frontend knows execution is running
        let _ = self.sender.send(LogMessage::status("running")).await;

        // Load plan
        let plan_content = std::fs::read_to_string(&plan_path)?;
        let mut plan: ExecutionPlan = serde_yaml::from_str(&plan_content)?;
        
        // Load tasks to map IDs to details
        let tasks_content = std::fs::read_to_string(&tasks_path)?;
        let task_file: TaskFile = serde_yaml::from_str(&tasks_content)?;
        let task_map: HashMap<String, SpecTask> = task_file.tasks
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
                    self.log("warning", &format!("Run {} not found, starting fresh", id)).await;
                    Run::generate_id()
                }
                Err(e) => {
                    self.log("warning", &format!("Failed to load run {}, starting fresh: {}", id, e)).await;
                    // Fall back to creating new run
                    Run::generate_id()
                }
            }
        } else {
            let id = Run::generate_id();
            let batch_info: Vec<(String, String)> = plan.batches
                .iter()
                .map(|b| (b.id.clone(), b.name.clone()))
                .collect();
            
            // Create run entry (best-effort - don't fail execution if history fails)
            match history_service.create_run(&spec_name, &id, batch_info, dry_run) {
                Ok(run) => {
                    self.log("info", &format!("Created run history entry: {}", run.id)).await;
                }
                Err(e) => {
                    self.log("warning", &format!("Failed to create history entry: {}", e)).await;
                }
            };
            id
        };

        // Initialize Worktree Manager
        let mut manager = DefaultWorktreeManager::new(&self.project_root).context("Failed to init worktree manager")?;
        let exe = std::env::current_exe()?; // Self-reference for spawning tasks?
        // Wait, self-referencing implies `ckrv task` is available.
        // If we are running in `ckrv-ui` binary, we can't assume it supports `task` subcommand.
        // We must check if we are running `ckrv` CLI or `ckrv-ui`.
        // If `ckrv-ui` does not support `task` subcommand, we need to find `ckrv` binary.
        
        // Try to find ckrv in the same directory as the current executable, or use PATH
        let ckrv_exe = if let Some(parent) = exe.parent() {
            let candidate = parent.join("ckrv");
            if candidate.exists() {
                candidate
            } else {
                PathBuf::from("ckrv")
            }
        } else {
             PathBuf::from("ckrv")
        };

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
            self.log("info", &format!("Resuming: {} batches already completed", count)).await;
        }

        let mut pending_batches: VecDeque<_> = plan.batches
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
                 let unblocked = batch.depends_on.iter().all(|d| completed_batches.contains(d));
                 
                 if unblocked {
                     self.log("batch_start", &format!("Spawning batch: {}", batch.name)).await;
                     
                     // T011: Send explicit batch status so frontend updates batch card
                     let _ = self.sender.send(LogMessage::batch_status(&batch.id, &batch.name, "running")).await;
                     
                     // Update status to running in plan file
                     self.update_batch_status(&plan_path, &batch.id, BatchStatus::Running, None)?;

                     let batch_clone = batch.clone();
                     let task_map_clone = task_map.clone();
                     let exe_path = ckrv_exe.clone();
                     let project_root = self.project_root.clone();
                     let executor_model = executor_model.clone();
                     let sender = self.sender.clone();
                     
                     // Spawn the batch execution
                     running_futures.push(tokio::spawn(async move {
                         Self::execute_batch(
                             project_root,
                             exe_path,
                             batch_clone,
                             task_map_clone,
                             dry_run,
                             true, // use_sandbox: always use Docker
                             executor_model,
                             sender
                         ).await
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
                                 self.log("batch_complete", &format!("Batch {} completed on branch {}", batch_id, branch_name)).await;
                                 
                                 // T012: Send explicit batch status so frontend updates counter
                                 let _ = self.sender.send(
                                     LogMessage::batch_status(&batch_id, &batch_id, "completed")
                                         .with_branch(&branch_name)
                                 ).await;
                                 
                                 // Update state
                                 completed_batches.insert(batch_id.clone());
                                 
                                 if !dry_run {
                                     // Merge Logic Here
                                     self.merge_batch(&branch_name, &spec_path).await?;
                                     
                                     // Mark tasks complete
                                     if let Some(tids) = batch_task_map.get(&batch_id) {
                                         self.mark_tasks_complete(&tasks_path, tids)?;
                                     }
                                     
                                     self.update_batch_status(&plan_path, &batch_id, BatchStatus::Completed, Some(&branch_name))?;
                                 }
                                 
                                 // T017: Update history with batch completion
                                 { let run_id = &run_id;
                                     let _ = history_service.update_batch_status(
                                         &spec_name,
                                         run_id,
                                         &batch_id,
                                         HistoryBatchStatus::Completed,
                                         Some(&branch_name),
                                         None,
                                     );
                                 }
                             },
                             Err(e) => {
                                 self.log("batch_error", &format!("Batch failed: {}", e)).await;
                                 // T015: Send status failed so frontend stops timer and shows error
                                 let _ = self.sender.send(LogMessage::status("failed")).await;
                                 
                                 // T018: Update history with run failure
                                 { let run_id = &run_id;
                                     let _ = history_service.fail_run(&spec_name, run_id, &e.to_string());
                                 }
                                 
                                 return Err(e);
                             }
                         }
                     },
                     Err(e) => {
                         // T015: Send status failed for task panics
                         let _ = self.sender.send(LogMessage::status("failed")).await;
                         
                         // T018: Update history with run failure
                         { let run_id = &run_id;
                             let _ = history_service.fail_run(&spec_name, run_id, &format!("Task panic: {}", e));
                         }
                         
                         return Err(anyhow!("Task panic: {}", e));
                     }
                 }
             } else if !pending_batches.is_empty() {
                 // T015: Send status failed for deadlocks
                 let _ = self.sender.send(LogMessage::status("failed")).await;
                 
                 // T018: Update history with run failure
                 { let run_id = &run_id;
                     let _ = history_service.fail_run(&spec_name, run_id, &format!("Deadlock: {} batches pending", pending_batches.len()));
                 }
                 
                 return Err(anyhow!("Deadlock: {} batches pending but none can run.", pending_batches.len()));
             }
        }
        
        // T014: Send explicit status completed so frontend knows execution is done
        let _ = self.sender.send(LogMessage::status("completed")).await;
        
        // T018: Update history with run completion
        { let run_id = &run_id;
            let _ = history_service.complete_run(&spec_name, run_id);
            self.log("info", &format!("Run history entry completed: {}", run_id)).await;
        }
        
        self.log("success", "All batches completed successfully.").await;
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
        sender: mpsc::Sender<LogMessage>,
    ) -> Result<(String, String)> { // Returns (batch_id, branch_name)
        
        // Construct description
        let mut description = format!("MISSION: {}\nREASONING: {}\n\nTASKS:\n", batch.name, batch.reasoning);
        for id in &batch.task_ids {
            if let Some(t) = task_map.get(id) {
                description.push_str(&format!("- [{}]: {} ({})\n", t.id, t.title, t.description));
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
            let manager = DefaultWorktreeManager::new(&root_clone)
                .context("Failed to init wt manager")?;
            manager.create(&wt_job_id_clone, "1")
                .context("Failed to create worktree")
        }).await.context("Worktree task panicked")??;
        
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
        let model = executor_model.or(batch.model_assignment.default.clone());
        
        if let Some(ref m) = model {
            task_args.push("--agent".to_string());
            task_args.push(m.clone());
        }

        if use_sandbox {
            // Docker sandbox execution using Claude Code CLI
            let _ = sender.send(LogMessage::new("info", "Executing in Docker sandbox with Claude Code...")).await;
            
            // Determine if this is an OpenRouter model or native Claude
            let is_openrouter = model.as_ref().map(|m| {
                m.contains('/') && !m.starts_with("claude")
            }).unwrap_or(false);
            
            // Try to create Docker sandbox, fall back to local if unavailable
            match DockerSandbox::with_defaults() {
                Ok(sandbox) => {
                    // Build the Claude Code command
                    // Use --print and --dangerously-skip-permissions for non-interactive execution
                    let claude_prompt = format!(
                        "You are implementing code changes in a project. Follow these instructions exactly:\n\n{}\n\nMake all changes to the files in /workspace. Do not ask questions - implement the code directly.",
                        description
                    );
                    
                    let escaped_prompt = claude_prompt.replace("\"", "\\\"");
                    
                    // Base command is the same for both paths
                    let cmd = format!(
                        "claude --print --dangerously-skip-permissions \"{}\"",
                        escaped_prompt
                    );
                    
                    let mut cfg = ExecuteConfig::new(
                        "claude",
                        worktree.path.clone()
                    ).shell(&cmd)
                     .with_timeout(Duration::from_secs(900)); // 15 minute timeout
                    
                    // Configure execution with claude CLI
                    let config = if is_openrouter {
                        // OpenRouter path: Use Claude Code CLI with OpenRouter env vars
                        // Per https://openrouter.ai/docs/guides/guides/claude-code-integration
                        let model_name = model.as_ref().unwrap();
                        let _ = sender.send(LogMessage::new("info", &format!("Using OpenRouter model: {}", model_name))).await;
                        
                        // Get OpenRouter API key from environment or agent config
                        let api_key = if let Ok(key) = std::env::var("OPENROUTER_API_KEY") {
                            Some(key)
                        } else {
                            let agents_dir = root.join(".agents");
                            Self::find_openrouter_key(&agents_dir, model_name)
                        };
                        
                        if let Some(key) = api_key {
                            // Required env vars for OpenRouter (same as runner.rs)
                            cfg = cfg.env("ANTHROPIC_BASE_URL", "https://openrouter.ai/api");
                            cfg = cfg.env("ANTHROPIC_AUTH_TOKEN", key);
                            cfg = cfg.env("ANTHROPIC_API_KEY", ""); // Must be explicitly empty!
                            
                            // Set model for all tiers
                            cfg = cfg.env("ANTHROPIC_DEFAULT_SONNET_MODEL", model_name);
                            cfg = cfg.env("ANTHROPIC_DEFAULT_OPUS_MODEL", model_name);
                            cfg = cfg.env("ANTHROPIC_DEFAULT_HAIKU_MODEL", model_name);
                        } else {
                            let _ = sender.send(LogMessage::new("warning", "No OPENROUTER_API_KEY found, execution may fail")).await;
                        }
                        
                        cfg
                    } else {
                        // Claude subscription path: Use native Claude Code auth via ~/.claude
                        let _ = sender.send(LogMessage::new("info", "Using Claude subscription")).await;
                        cfg
                    };
                    
                    // Set HOME for Claude Code config
                    let config = config.env("HOME", "/home/claude");
                    let config = config.env("NO_COLOR", "1");
                    
                    // Execute in sandbox
                    match sandbox.execute(config).await {
                        Ok(result) => {
                            // Log stdout
                            for line in result.stdout.lines() {
                                let _ = sender.send(LogMessage::new("log", line)).await;
                            }
                            // Log stderr
                            for line in result.stderr.lines() {
                                let _ = sender.send(LogMessage::new("error", line)).await;
                            }
                            
                            if !result.success() {
                                return Err(anyhow!("Claude Code execution failed with exit code {}", result.exit_code));
                            }
                        }
                        Err(e) => {
                            let _ = sender.send(LogMessage::new("error", &format!("Sandbox execution error: {}", e))).await;
                            return Err(anyhow!("Sandbox execution failed: {}", e));
                        }
                    }
                }
                Err(e) => {
                    // Fall back to local execution if Docker is not available
                    let _ = sender.send(LogMessage::new("warning", &format!("Docker unavailable ({}), falling back to local execution", e))).await;
                    Self::execute_local(&exe, &task_args, &sender).await?;
                }
            }
        } else {
            // Local execution (no sandbox) - uses ckrv task
            Self::execute_local(&exe, &task_args, &sender).await?;
        }
        
        // Commit changes inside the worktree
        AsyncCommand::new("git")
            .arg("add").arg(".")
            .current_dir(&worktree.path)
            .status().await?;
            
        let commit_msg = format!("feat(batch): {} - {}", batch.name, batch.id);
        AsyncCommand::new("git")
            .args(["commit", "-m", &commit_msg])
            .current_dir(&worktree.path)
            .status().await?;

        Ok((batch.id, branch_name))
    }
    
    /// Execute command locally (no sandbox)
    async fn execute_local(
        exe: &Path,
        args: &[String],
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
        
        // Spawn stdout reader
        let stdout_handle = if let Some(out) = stdout {
            Some(tokio::spawn(async move {
                use tokio::io::{AsyncBufReadExt, BufReader};
                let mut reader = BufReader::new(out).lines();
                while let Ok(Some(line)) = reader.next_line().await {
                    let _ = sender_out.send(LogMessage::new("log", &line)).await;
                }
            }))
        } else {
            None
        };

        // Spawn stderr reader
        let stderr_handle = if let Some(err) = stderr {
            Some(tokio::spawn(async move {
                use tokio::io::{AsyncBufReadExt, BufReader};
                let mut reader = BufReader::new(err).lines();
                while let Ok(Some(line)) = reader.next_line().await {
                    let _ = sender_err.send(LogMessage::new("error", &line)).await;
                }
            }))
        } else {
            None
        };
        
        let status = child.wait().await?;
        
        // Wait for I/O to finish
        if let Some(h) = stdout_handle { let _ = h.await; }
        if let Some(h) = stderr_handle { let _ = h.await; }
        
        if !status.success() {
            return Err(anyhow!("Task failed with exit code {:?}", status.code()));
        }
        
        Ok(())
    }
    
    async fn merge_batch(&self, branch: &str, spec_path: &Path) -> Result<()> {
        self.log("info", &format!("Merging branch {}", branch)).await;
        
        let status = AsyncCommand::new("git")
            .args(["merge", "--no-ff", "--no-edit", branch])
            .current_dir(&self.project_root)
            .status()
            .await?;
            
        if !status.success() {
            if self.has_merge_conflicts().await {
                self.log("info", "Merge conflicts detected. Attempting AI resolution...").await;
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
            .output().await;
            
        match output {
            Ok(o) => !o.stdout.is_empty(),
            Err(_) => false,
        }
    }
    
    async fn resolve_conflicts(&self, branch: &str, spec_path: &Path) -> Result<()> {
         // Gather conflicts
         let output = AsyncCommand::new("git")
            .args(["diff", "--name-only", "--diff-filter=U"])
            .current_dir(&self.project_root)
            .output().await?;
        
        let files = String::from_utf8_lossy(&output.stdout);
        let file_list: Vec<&str> = files.lines().collect();
        
        if file_list.is_empty() { return Ok(()); }
        
        // Prompt construction
        let spec_content = std::fs::read_to_string(spec_path).unwrap_or_default();
        let prompt = format!(
            "You are resolving Git merge conflicts for files: {:?}.\nSpec: {}\n\nResolve markers <<<<<<< ======= >>>>>>> and stage files.",
            file_list, spec_content
        );

        // Run Claude Code in Sandbox
        // We use ckrv-sandbox here
        let sandbox = DockerSandbox::new(DefaultAllowList::default())
             .context("Failed to create sandbox")?;
             
        let escaped_prompt = shell_escape::escape(prompt.into());
        let command = format!(
            "echo {} | claude -p - --dangerously-skip-permissions", 
            escaped_prompt
        );
        
        let config = ExecuteConfig::new("", self.project_root.clone())
            .shell(&command)
            .with_timeout(Duration::from_secs(300));
            
        let result = sandbox.execute(config).await
            .context("Sandbox execution failed")?;
            
        if !result.success() {
             self.log("error", &format!("AI conflict resolution failed: {}", result.stderr)).await;
        }
        
        if self.has_merge_conflicts().await {
            return Err(anyhow!("AI could not resolve all conflicts"));
        }
        
        // Commit
        AsyncCommand::new("git")
             .args(["commit", "--no-edit"])
             .current_dir(&self.project_root)
             .status().await?;
             
        Ok(())
    }

    fn update_batch_status(&self, plan_path: &Path, batch_id: &str, status: BatchStatus, branch: Option<&str>) -> Result<()> {
        let content = std::fs::read_to_string(plan_path)?;
        let mut plan: ExecutionPlan = serde_yaml::from_str(&content)?;
        
        for batch in &mut plan.batches {
            if batch.id == batch_id {
                batch.status = status.clone();
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
}
