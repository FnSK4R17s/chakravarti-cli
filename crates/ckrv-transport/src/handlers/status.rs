//! # Status Handler
//!
//! Handler for system status endpoint.

// ============================================================
// IMPORTS
// ============================================================

use crate::error::TransportError;
use crate::state::{AppState, SystemStatus};
use std::path::Path;
use std::process::Command;

// ============================================================
// IMPLEMENTATION
// ============================================================

/// Detect the current git branch for a given project root.
fn detect_git_branch(project_root: &Path) -> Option<String> {
    // First check if we're in a git repo at all
    let git_check = Command::new("git")
        .args(["rev-parse", "--git-dir"])
        .current_dir(project_root)
        .output()
        .ok()?;

    if !git_check.status.success() {
        return None; // Not a git repo
    }

    // Try to get the current branch
    let output = Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .current_dir(project_root)
        .output()
        .ok()?;

    if output.status.success() {
        let branch = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !branch.is_empty() && branch != "HEAD" {
            return Some(branch);
        }
    }

    // For fresh repos with no commits, try to get branch from symbolic-ref
    let symbolic = Command::new("git")
        .args(["symbolic-ref", "--short", "HEAD"])
        .current_dir(project_root)
        .output()
        .ok()?;

    if symbolic.status.success() {
        let branch = String::from_utf8_lossy(&symbolic.stdout).trim().to_string();
        if !branch.is_empty() {
            return Some(branch);
        }
    }

    // Fallback: we're in a git repo but can't determine branch
    Some("(no commits)".to_string())
}

/// Detect if the project is initialized (has .specs and .chakravarti directories).
fn detect_is_initialized(project_root: &Path) -> bool {
    let specs_dir = project_root.join(".specs");
    let chakravarti_dir = project_root.join(".chakravarti");
    specs_dir.exists() && chakravarti_dir.exists()
}

/// Get current system status.
///
/// Returns the current system status including git branch and initialization state.
/// Uses `blocking_read()` on the status lock since this runs inside
/// `spawn_blocking`.
///
/// # Example
///
/// ```rust,ignore
/// let status = get_status_handler(&state)?;
/// println!("Branch: {}", status.active_branch);
/// ```
pub fn get_status_handler(state: &AppState) -> Result<SystemStatus, TransportError> {
    let mut status = state.status.blocking_read().clone();

    // Dynamically detect git branch using project root
    if let Some(branch) = detect_git_branch(&state.project_root) {
        status.active_branch = branch;
    } else {
        status.active_branch = "none".to_string();
    }

    // Dynamically detect initialization status using project root
    status.is_ready = detect_is_initialized(&state.project_root);

    // Set project root from state
    status.project_root = state.project_root.display().to_string();

    Ok(status)
}

// ============================================================
// TESTS
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_get_status_handler() {
        let state = AppState::new(PathBuf::from("/tmp/test"));

        let result = get_status_handler(&state);
        assert!(result.is_ok());

        let status = result.expect("should have status");
        // Branch detection depends on environment
        assert!(!status.active_branch.is_empty());
    }

    #[test]
    fn test_detect_git_branch() {
        // This test verifies the function doesn't panic
        let _branch = detect_git_branch(Path::new("/tmp"));
    }

    #[test]
    fn test_detect_is_initialized() {
        // This test verifies the function doesn't panic
        let _initialized = detect_is_initialized(Path::new("/tmp"));
    }
}
