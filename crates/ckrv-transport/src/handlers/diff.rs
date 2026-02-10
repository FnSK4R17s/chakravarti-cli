//! # Diff Handler
//!
//! Handlers for git diff operations.

use crate::error::TransportError;
use crate::state::AppState;
use serde::{Deserialize, Serialize};
use std::process::Command;

// ============================================================================
// Request/Response Types
// ============================================================================

/// Diff query parameters.
#[derive(Debug, Deserialize)]
pub struct DiffQuery {
    /// Base branch to compare from
    pub base: Option<String>,
    /// Target branch to compare to
    pub target: Option<String>,
    /// Specific file path to diff
    pub path: Option<String>,
}

/// Diff response.
#[derive(Debug, Serialize)]
pub struct DiffResponse {
    pub base_branch: String,
    pub target_branch: String,
    pub files: Vec<FileDiff>,
    pub stats: DiffStats,
}

/// File diff details.
#[derive(Debug, Serialize)]
pub struct FileDiff {
    pub path: String,
    pub status: String,
    pub additions: u32,
    pub deletions: u32,
    pub diff: String,
}

/// Diff statistics.
#[derive(Debug, Serialize)]
pub struct DiffStats {
    pub files_changed: u32,
    pub insertions: u32,
    pub deletions: u32,
}

/// Branches response.
#[derive(Debug, Serialize)]
pub struct BranchesResponse {
    pub current: String,
    pub branches: Vec<String>,
}

// ============================================================================
// Handlers
// ============================================================================

/// Get available git branches.
pub async fn get_branches_handler(state: &AppState) -> Result<BranchesResponse, TransportError> {
    let cwd = &state.project_root;

    // Get current branch
    let current = Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .current_dir(cwd)
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "HEAD".to_string());

    // Get all branches
    let output = Command::new("git")
        .args(["branch", "-a", "--format=%(refname:short)"])
        .current_dir(cwd)
        .output()
        .map_err(|e| TransportError::Internal(format!("Failed to get branches: {e}")))?;

    let branches: Vec<String> = String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter(|line| !line.is_empty())
        .map(|s| s.trim().to_string())
        .collect();

    Ok(BranchesResponse { current, branches })
}

/// Get the default branch (main or master).
pub async fn get_default_branch_handler(state: &AppState) -> Result<String, TransportError> {
    let cwd = &state.project_root;

    let main_check = Command::new("git")
        .args(["rev-parse", "--verify", "main"])
        .current_dir(cwd)
        .output();

    if main_check.map(|o| o.status.success()).unwrap_or(false) {
        Ok("main".to_string())
    } else {
        Ok("master".to_string())
    }
}

/// Get diff between two branches.
pub async fn get_diff_handler(
    state: &AppState,
    query: DiffQuery,
) -> Result<DiffResponse, TransportError> {
    let cwd = &state.project_root;

    // Determine base branch
    let base = match query.base {
        Some(b) => b,
        None => get_default_branch_handler(state).await?,
    };

    let target = query.target.unwrap_or_else(|| "HEAD".to_string());

    // Get changed files with stats
    let mut diff_args = vec!["diff", "--numstat", &base, &target];
    if let Some(ref path) = query.path {
        diff_args.push("--");
        diff_args.push(path);
    }

    let numstat_output = Command::new("git")
        .args(&diff_args)
        .current_dir(cwd)
        .output()
        .map_err(|e| TransportError::Internal(format!("Failed to get diff: {e}")))?;

    if !numstat_output.status.success() {
        return Ok(DiffResponse {
            base_branch: base,
            target_branch: target,
            files: vec![],
            stats: DiffStats {
                files_changed: 0,
                insertions: 0,
                deletions: 0,
            },
        });
    }

    let mut files = Vec::new();
    let mut total_insertions = 0u32;
    let mut total_deletions = 0u32;

    for line in String::from_utf8_lossy(&numstat_output.stdout).lines() {
        let parts: Vec<&str> = line.split('\t').collect();
        if parts.len() >= 3 {
            let additions: u32 = parts[0].parse().unwrap_or(0);
            let deletions: u32 = parts[1].parse().unwrap_or(0);
            let path = parts[2].to_string();

            total_insertions += additions;
            total_deletions += deletions;

            // Get full diff for this file
            let file_diff = Command::new("git")
                .args(["diff", &base, &target, "--", &path])
                .current_dir(cwd)
                .output()
                .ok()
                .and_then(|o| String::from_utf8(o.stdout).ok())
                .unwrap_or_default();

            // Determine status
            let status = if additions > 0 && deletions == 0 {
                "added"
            } else if additions == 0 && deletions > 0 {
                "deleted"
            } else {
                "modified"
            }
            .to_string();

            files.push(FileDiff {
                path,
                status,
                additions,
                deletions,
                diff: file_diff,
            });
        }
    }

    let files_changed = files.len() as u32;

    Ok(DiffResponse {
        base_branch: base,
        target_branch: target,
        files,
        stats: DiffStats {
            files_changed,
            insertions: total_insertions,
            deletions: total_deletions,
        },
    })
}

/// Apply a diff patch.
pub async fn apply_diff_handler(state: &AppState, patch: String) -> Result<(), TransportError> {
    let cwd = &state.project_root;

    let mut child = Command::new("git")
        .args(["apply", "--3way", "-"])
        .current_dir(cwd)
        .stdin(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| TransportError::Internal(format!("Failed to apply patch: {e}")))?;

    use std::io::Write;
    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(patch.as_bytes())
            .map_err(|e| TransportError::Internal(format!("Failed to write patch: {e}")))?;
    }

    let output = child
        .wait_with_output()
        .map_err(|e| TransportError::Internal(format!("Failed to apply patch: {e}")))?;

    if output.status.success() {
        Ok(())
    } else {
        Err(TransportError::Internal(format!(
            "Patch failed: {}",
            String::from_utf8_lossy(&output.stderr)
        )))
    }
}

/// Revert changes to a file.
pub async fn revert_file_handler(
    state: &AppState,
    file_path: String,
) -> Result<(), TransportError> {
    let cwd = &state.project_root;

    let output = Command::new("git")
        .args(["checkout", "HEAD", "--", &file_path])
        .current_dir(cwd)
        .output()
        .map_err(|e| TransportError::Internal(format!("Failed to revert file: {e}")))?;

    if output.status.success() {
        Ok(())
    } else {
        Err(TransportError::Internal(format!(
            "Revert failed: {}",
            String::from_utf8_lossy(&output.stderr)
        )))
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[tokio::test]
    async fn test_get_default_branch() {
        let state = AppState::new(PathBuf::from("/tmp/test-diff"));
        // This will likely fail in a non-git directory, which is expected
        let _ = get_default_branch_handler(&state).await;
    }
}
