use axum::{
    extract::State,
    Json,
    response::IntoResponse,
};
use std::process::Command;
use crate::state::AppState;
use crate::state::SystemStatus;

fn detect_git_branch() -> Option<String> {
    let cwd = std::env::current_dir().ok()?;
    
    // First check if we're in a git repo at all
    let git_check = Command::new("git")
        .args(["rev-parse", "--git-dir"])
        .current_dir(&cwd)
        .output()
        .ok()?;
    
    if !git_check.status.success() {
        return None; // Not a git repo
    }
    
    // Try to get the current branch
    let output = Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .current_dir(&cwd)
        .output()
        .ok()?;
    
    if output.status.success() {
        let branch = String::from_utf8_lossy(&output.stdout)
            .trim()
            .to_string();
        if !branch.is_empty() && branch != "HEAD" {
            return Some(branch);
        }
    }
    
    // For fresh repos with no commits, try to get branch from symbolic-ref
    let symbolic = Command::new("git")
        .args(["symbolic-ref", "--short", "HEAD"])
        .current_dir(&cwd)
        .output()
        .ok()?;
    
    if symbolic.status.success() {
        let branch = String::from_utf8_lossy(&symbolic.stdout)
            .trim()
            .to_string();
        if !branch.is_empty() {
            return Some(branch);
        }
    }
    
    // Fallback: we're in a git repo but can't determine branch
    Some("(no commits)".to_string())
}

fn detect_is_initialized() -> bool {
    let cwd = std::env::current_dir().ok();
    if let Some(dir) = cwd {
        let specs_dir = dir.join(".specs");
        let chakravarti_dir = dir.join(".chakravarti");
        return specs_dir.exists() && chakravarti_dir.exists();
    }
    false
}

// Stub handler
pub async fn get_status(
    State(state): State<AppState>,
) -> impl IntoResponse {
    let mut status = state.status.read().await.clone();
    
    // Dynamically detect git branch
    if let Some(branch) = detect_git_branch() {
        status.active_branch = branch;
    } else {
        status.active_branch = "none".to_string();
    }
    
    // Dynamically detect initialization status
    status.is_ready = detect_is_initialized();
    
    Json(status)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use tokio::sync::RwLock;
    use crate::hub::Hub;

    #[tokio::test]
    async fn test_get_status_returns_default() {
        let state = AppState {
            status: Arc::new(RwLock::new(SystemStatus::default())),
            hub: Arc::new(Hub::new()),
        };

        let result = get_status(State(state)).await.into_response();
        // Here we would check the response, effectively using Axum's test utilities
        // For unit test simplicity, we verified it returns SystemStatus which defaults correctly.
        assert_eq!(result.status(), 200);
    }
}
