//! # Execution Handler
//!
//! Handlers for batch execution control.

use crate::error::TransportError;
use crate::state::AppState;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::process::Command;

// ============================================================================
// Request/Response Types
// ============================================================================

/// Execute request.
#[derive(Debug, Deserialize)]
pub struct ExecuteRequest {
    /// Spec name to execute
    pub spec: String,
    /// Optional batch ID to execute (if not provided, executes all pending)
    pub batch_id: Option<String>,
    /// Dry run mode
    #[serde(default)]
    pub dry_run: bool,
}

/// Execution status.
#[derive(Debug, Serialize)]
pub struct ExecutionStatus {
    pub running: bool,
    pub spec_name: Option<String>,
    pub batch_id: Option<String>,
    pub progress: f32,
    pub current_task: Option<String>,
    pub message: Option<String>,
}

/// Execute response.
#[derive(Debug, Serialize)]
pub struct ExecuteResponse {
    pub started: bool,
    pub run_id: Option<String>,
    pub message: Option<String>,
}

/// Stop execution request.
#[derive(Debug, Deserialize)]
pub struct StopRequest {
    pub spec: String,
    pub run_id: Option<String>,
}

// ============================================================================
// Handlers
// ============================================================================

/// Start batch execution.
pub async fn start_execution_handler(
    state: &AppState,
    request: ExecuteRequest,
) -> Result<ExecuteResponse, TransportError> {
    let mut args = vec!["execute"];

    if request.dry_run {
        args.push("--dry-run");
    }

    if let Some(ref batch_id) = request.batch_id {
        args.push("--batch");
        args.push(batch_id);
    }

    // Start execution in background
    let output = Command::new("ckrv")
        .args(&args)
        .current_dir(&state.project_root)
        .spawn();

    match output {
        Ok(_child) => {
            // Generate a run ID
            let run_id = format!("run-{}", chrono::Utc::now().timestamp());

            Ok(ExecuteResponse {
                started: true,
                run_id: Some(run_id),
                message: Some("Execution started".to_string()),
            })
        }
        Err(e) => Err(TransportError::Internal(format!(
            "Failed to start execution: {e}"
        ))),
    }
}

/// Get current execution status.
pub async fn get_execution_status_handler(
    state: &AppState,
) -> Result<ExecutionStatus, TransportError> {
    // Check if there's a running execution by looking for lock files or processes
    // For now, return a static "not running" status
    Ok(ExecutionStatus {
        running: false,
        spec_name: None,
        batch_id: None,
        progress: 0.0,
        current_task: None,
        message: None,
    })
}

/// Stop execution.
pub async fn stop_execution_handler(
    state: &AppState,
    request: StopRequest,
) -> Result<(), TransportError> {
    // Try to abort the current run
    let output = Command::new("ckrv")
        .args(["abort"])
        .current_dir(&state.project_root)
        .output()
        .map_err(|e| TransportError::Internal(format!("Failed to abort execution: {e}")))?;

    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(TransportError::Internal(format!("Abort failed: {stderr}")))
    }
}

/// Pause execution (if supported).
pub async fn pause_execution_handler(_state: &AppState) -> Result<ExecutionStatus, TransportError> {
    // Pausing is not currently supported
    Err(TransportError::BadRequest(
        "Pause not supported".to_string(),
    ))
}

/// Resume execution (if supported).
pub async fn resume_execution_handler(
    _state: &AppState,
) -> Result<ExecutionStatus, TransportError> {
    // Resuming is not currently supported
    Err(TransportError::BadRequest(
        "Resume not supported".to_string(),
    ))
}

// ============================================================================
// Branch Management
// ============================================================================

/// Request to list branches.
#[derive(Debug, Deserialize)]
pub struct ListBranchesRequest {
    pub spec: Option<String>,
}

/// Branch info.
#[derive(Debug, Serialize)]
pub struct BranchInfo {
    pub name: String,
    pub batch_name: String,
    pub ahead_commits: u32,
    pub is_clean: bool,
}

/// Response with branches.
#[derive(Debug, Serialize)]
pub struct ListBranchesResponse {
    pub success: bool,
    pub current_branch: String,
    pub branches: Vec<BranchInfo>,
    pub message: Option<String>,
}

/// List unmerged worktree branches.
pub async fn list_branches_handler(
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
    let filter_pattern = if let Some(ref spec) = request.spec {
        format!("worktree/{}/", spec)
    } else {
        "worktree/".to_string()
    };

    let mut branches = Vec::new();

    for branch_name in worktree_branches {
        // Check if matches filter pattern
        if !branch_name.starts_with(&filter_pattern) {
            if request.spec.is_some() {
                continue;
            }
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
            .args(["rev-list", "--count", &format!("HEAD..{}", branch_name)])
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
            .last()
            .unwrap_or(&branch_name)
            .replace("ckrv-batch-", "")
            .to_string();

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
    pub spec: Option<String>,
}

/// Response from merge all.
#[derive(Debug, Serialize)]
pub struct MergeAllResponse {
    pub success: bool,
    pub merged: Vec<String>,
    pub failed: Vec<String>,
    pub message: String,
}

/// Merge all worktree branches.
pub async fn merge_all_branches_handler(
    state: &AppState,
    _request: MergeAllRequest,
) -> Result<MergeAllResponse, TransportError> {
    let project_root = &state.project_root;

    // Get list of worktree branches to merge
    let worktree_output = Command::new("git")
        .args(["worktree", "list", "--porcelain"])
        .current_dir(project_root)
        .output();

    let worktree_info: Vec<(String, String)> = match worktree_output {
        Ok(output) => {
            let text = String::from_utf8_lossy(&output.stdout);
            let mut worktrees = Vec::new();
            let mut current_path = String::new();
            let mut current_branch = String::new();

            for line in text.lines() {
                if line.starts_with("worktree ") {
                    current_path = line.strip_prefix("worktree ").unwrap_or("").to_string();
                } else if line.starts_with("branch refs/heads/") {
                    current_branch = line
                        .strip_prefix("branch refs/heads/")
                        .unwrap_or("")
                        .to_string();
                    // Only include worktree branches
                    if current_branch.contains("worktree/") && !current_path.is_empty() {
                        worktrees.push((current_path.clone(), current_branch.clone()));
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
        // Check if already merged
        let is_merged = Command::new("git")
            .args(["merge-base", "--is-ancestor", &branch, "HEAD"])
            .current_dir(project_root)
            .status()
            .map(|s| s.success())
            .unwrap_or(false);

        if is_merged {
            // Already merged, just clean up worktree
            let _ = Command::new("git")
                .args(["worktree", "remove", "--force", &wt_path])
                .current_dir(project_root)
                .status();
            merged.push(branch);
            continue;
        }

        // Try to merge
        let merge_result = Command::new("git")
            .args(["merge", "--no-ff", "--no-edit", &branch])
            .current_dir(project_root)
            .status();

        if merge_result.map(|s| s.success()).unwrap_or(false) {
            // Merge successful, clean up worktree
            let _ = Command::new("git")
                .args(["worktree", "remove", "--force", &wt_path])
                .current_dir(project_root)
                .status();
            merged.push(branch);
        } else {
            // Merge failed - abort if in progress
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
    pub branch: String,
}

/// Response from merge branch.
#[derive(Debug, Serialize)]
pub struct MergeBranchResponse {
    pub success: bool,
    pub branch: String,
    pub message: String,
}

/// Merge a single branch.
pub async fn merge_branch_handler(
    state: &AppState,
    request: MergeBranchRequest,
) -> Result<MergeBranchResponse, TransportError> {
    let project_root = &state.project_root;

    // Try to merge
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
        // Abort merge if in progress
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

// ============================================================================
// Log Handlers
// ============================================================================

/// Log entry for execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub timestamp: String,
    pub level: String,
    pub message: String,
}

/// Log history request params.
#[derive(Debug, Deserialize)]
pub struct LogHistoryParams {
    pub offset: Option<usize>,
    pub limit: Option<usize>,
    pub since: Option<String>,
}

/// Log history response.
#[derive(Debug, Serialize)]
pub struct LogHistoryResponse {
    pub execution_id: String,
    pub logs: Vec<LogEntry>,
    pub total_count: usize,
    pub offset: usize,
    pub has_more: bool,
}

/// Log tail params.
#[derive(Debug, Deserialize)]
pub struct LogTailParams {
    pub count: Option<usize>,
}

/// Log tail response.
#[derive(Debug, Serialize)]
pub struct LogTailResponse {
    pub execution_id: String,
    pub logs: Vec<LogEntry>,
    pub total_count: usize,
}

/// Get execution logs.
pub async fn get_logs_handler(
    _state: &AppState,
    execution_id: String,
    _params: LogHistoryParams,
) -> Result<LogHistoryResponse, TransportError> {
    // For now, return empty logs - actual log storage would be implemented here
    Ok(LogHistoryResponse {
        execution_id,
        logs: vec![],
        total_count: 0,
        offset: 0,
        has_more: false,
    })
}

/// Tail execution logs.
pub async fn tail_logs_handler(
    _state: &AppState,
    execution_id: String,
    _params: LogTailParams,
) -> Result<LogTailResponse, TransportError> {
    // For now, return empty logs
    Ok(LogTailResponse {
        execution_id,
        logs: vec![],
        total_count: 0,
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
    async fn test_get_execution_status_handler() {
        let state = AppState::new(PathBuf::from("/tmp/test-execution"));
        let result = get_execution_status_handler(&state).await;
        assert!(result.is_ok());
        assert!(!result.unwrap().running);
    }
}
