//! Git diff API for viewing code changes between branches.

use axum::{
    extract::{Query, State},
    response::{IntoResponse, Json},
};
use serde::{Deserialize, Serialize};
use std::process::Command;

use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct DiffQuery {
    /// Base branch to compare from (default: main or master)
    pub base: Option<String>,
    /// Target branch to compare to (default: HEAD)
    pub target: Option<String>,
    /// Specific file path to diff (optional)
    pub path: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct DiffResponse {
    pub success: bool,
    pub base_branch: String,
    pub target_branch: String,
    pub files: Vec<FileDiff>,
    pub stats: DiffStats,
    pub error: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct FileDiff {
    pub path: String,
    pub status: String, // "added", "modified", "deleted", "renamed"
    pub additions: u32,
    pub deletions: u32,
    pub diff: String,
}

#[derive(Debug, Serialize)]
pub struct DiffStats {
    pub files_changed: u32,
    pub insertions: u32,
    pub deletions: u32,
}

#[derive(Debug, Serialize)]
pub struct BranchesResponse {
    pub success: bool,
    pub current: String,
    pub branches: Vec<String>,
    pub error: Option<String>,
}

/// Get available git branches
pub async fn get_branches(
    State(state): State<AppState>,
) -> impl IntoResponse {
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
    
    // Get all branches (local and remote)
    let output = Command::new("git")
        .args(["branch", "-a", "--format=%(refname:short)"])
        .current_dir(cwd)
        .output();
        
    match output {
        Ok(out) if out.status.success() => {
            let branches: Vec<String> = String::from_utf8_lossy(&out.stdout)
                .lines()
                .filter(|line| !line.is_empty())
                .map(|s| s.trim().to_string())
                .collect();
            
            Json(BranchesResponse {
                success: true,
                current,
                branches,
                error: None,
            })
        }
        Ok(out) => {
            Json(BranchesResponse {
                success: false,
                current,
                branches: vec![],
                error: Some(String::from_utf8_lossy(&out.stderr).to_string()),
            })
        }
        Err(e) => {
            Json(BranchesResponse {
                success: false,
                current,
                branches: vec![],
                error: Some(e.to_string()),
            })
        }
    }
}

/// Get diff between two branches
pub async fn get_diff(
    State(state): State<AppState>,
    Query(query): Query<DiffQuery>,
) -> impl IntoResponse {
    let cwd = &state.project_root;
    
    // Determine base branch (try main, then master, then use provided)
    let base = query.base.unwrap_or_else(|| {
        // Check if main exists
        let main_check = Command::new("git")
            .args(["rev-parse", "--verify", "main"])
            .current_dir(cwd)
            .output();
        
        if main_check.map(|o| o.status.success()).unwrap_or(false) {
            "main".to_string()
        } else {
            "master".to_string()
        }
    });
    
    let target = query.target.unwrap_or_else(|| "HEAD".to_string());
    
    // Get list of changed files with stats
    let mut diff_args = vec!["diff", "--numstat", &base, &target];
    let path_str;
    if let Some(ref path) = query.path {
        path_str = format!("-- {}", path);
        diff_args.push("--");
        diff_args.push(path);
    }
    
    let numstat_output = Command::new("git")
        .args(&diff_args[..])
        .current_dir(cwd)
        .output();
    
    let (files, stats) = match numstat_output {
        Ok(out) if out.status.success() => {
            let mut files = Vec::new();
            let mut total_insertions = 0u32;
            let mut total_deletions = 0u32;
            
            for line in String::from_utf8_lossy(&out.stdout).lines() {
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
                    }.to_string();
                    
                    files.push(FileDiff {
                        path,
                        status,
                        additions,
                        deletions,
                        diff: file_diff,
                    });
                }
            }
            
            let stats = DiffStats {
                files_changed: files.len() as u32,
                insertions: total_insertions,
                deletions: total_deletions,
            };
            
            (files, stats)
        }
        _ => (vec![], DiffStats {
            files_changed: 0,
            insertions: 0,
            deletions: 0,
        })
    };
    
    Json(DiffResponse {
        success: true,
        base_branch: base,
        target_branch: target,
        files,
        stats,
        error: None,
    })
}
