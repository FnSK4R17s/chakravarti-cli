//! # Commands Handler
//!
//! Handlers for CLI command execution.

use crate::error::TransportError;
use crate::state::AppState;
use serde::{Deserialize, Serialize};
use std::process::Command;

// ============================================================================
// Request/Response Types
// ============================================================================

/// Command execution response.
#[derive(Debug, Serialize)]
pub struct CommandResponse {
    pub success: bool,
    pub message: Option<String>,
    pub output: Option<String>,
    pub error: Option<String>,
}

/// Spec creation request.
#[derive(Debug, Deserialize)]
pub struct SpecNewRequest {
    pub description: String,
    pub name: Option<String>,
}

/// Diff command request.
#[derive(Debug, Deserialize)]
pub struct DiffRequest {
    pub base: Option<String>,
    pub stat: Option<bool>,
    pub files: Option<bool>,
    pub summary: Option<bool>,
}

/// Verify command request.
#[derive(Debug, Deserialize)]
pub struct VerifyRequest {
    pub lint: Option<bool>,
    pub typecheck: Option<bool>,
    pub test: Option<bool>,
    pub fix: Option<bool>,
}

/// Promote command request.
#[derive(Debug, Deserialize)]
pub struct PromoteRequest {
    pub base: Option<String>,
    pub draft: Option<bool>,
    pub push: Option<bool>,
}

/// Fix command request.
#[derive(Debug, Deserialize)]
pub struct FixRequest {
    pub lint: Option<bool>,
    pub typecheck: Option<bool>,
    pub test: Option<bool>,
    pub check: Option<bool>,
    pub error: Option<String>,
}

// ============================================================================
// Handlers
// ============================================================================

/// Run ckrv init command.
pub async fn run_init_handler(state: &AppState) -> Result<CommandResponse, TransportError> {
    run_ckrv_command(state, &["init"])
}

/// Run git init command.
pub async fn run_git_init_handler(state: &AppState) -> Result<CommandResponse, TransportError> {
    let output = Command::new("git")
        .args(["init"])
        .current_dir(&state.project_root)
        .output()
        .map_err(|e| TransportError::Internal(format!("Failed to run git init: {e}")))?;

    Ok(CommandResponse {
        success: output.status.success(),
        message: Some("Git repository initialized".to_string()),
        output: Some(String::from_utf8_lossy(&output.stdout).to_string()),
        error: if output.status.success() {
            None
        } else {
            Some(String::from_utf8_lossy(&output.stderr).to_string())
        },
    })
}

/// Run ckrv spec new command.
pub async fn run_spec_new_handler(
    state: &AppState,
    request: SpecNewRequest,
) -> Result<CommandResponse, TransportError> {
    let mut args = vec!["spec", "new", &request.description];
    let name_str;
    if let Some(ref name) = request.name {
        args.push("--name");
        name_str = name.clone();
        args.push(&name_str);
    }

    run_ckrv_command(state, &args)
}

/// Run ckrv spec tasks command.
pub async fn run_spec_tasks_handler(state: &AppState) -> Result<CommandResponse, TransportError> {
    run_ckrv_command(state, &["spec", "tasks"])
}

/// Run ckrv plan command.
pub async fn run_plan_handler(state: &AppState) -> Result<CommandResponse, TransportError> {
    run_ckrv_command(state, &["plan"])
}

/// Run ckrv execute command.
pub async fn run_execute_handler(state: &AppState) -> Result<CommandResponse, TransportError> {
    run_ckrv_command(state, &["execute"])
}

/// Run ckrv diff command.
pub async fn run_diff_handler(
    state: &AppState,
    request: DiffRequest,
) -> Result<CommandResponse, TransportError> {
    let mut args = vec!["diff"];
    let base_str;

    if let Some(ref base) = request.base {
        args.push("--base");
        base_str = base.clone();
        args.push(&base_str);
    }
    if request.stat.unwrap_or(false) {
        args.push("--stat");
    }
    if request.files.unwrap_or(false) {
        args.push("--files");
    }
    if request.summary.unwrap_or(false) {
        args.push("--summary");
    }

    run_ckrv_command(state, &args)
}

/// Run ckrv verify command.
pub async fn run_verify_handler(
    state: &AppState,
    request: VerifyRequest,
) -> Result<CommandResponse, TransportError> {
    let mut args = vec!["verify"];

    if request.lint.unwrap_or(false) {
        args.push("--lint");
    }
    if request.typecheck.unwrap_or(false) {
        args.push("--typecheck");
    }
    if request.test.unwrap_or(false) {
        args.push("--test");
    }
    if request.fix.unwrap_or(false) {
        args.push("--fix");
    }

    run_ckrv_command(state, &args)
}

/// Run ckrv promote command.
pub async fn run_promote_handler(
    state: &AppState,
    request: PromoteRequest,
) -> Result<CommandResponse, TransportError> {
    let mut args = vec!["promote"];
    let base_str;

    if let Some(ref base) = request.base {
        args.push("--base");
        base_str = base.clone();
        args.push(&base_str);
    }
    if request.draft.unwrap_or(false) {
        args.push("--draft");
    }
    if !request.push.unwrap_or(true) {
        args.push("--no-push");
    }

    run_ckrv_command(state, &args)
}

/// Run ckrv fix command.
pub async fn run_fix_handler(
    state: &AppState,
    request: FixRequest,
) -> Result<CommandResponse, TransportError> {
    let mut args = vec!["fix"];

    if request.lint.unwrap_or(false) {
        args.push("--lint");
    }
    if request.typecheck.unwrap_or(false) {
        args.push("--typecheck");
    }
    if request.test.unwrap_or(false) {
        args.push("--test");
    }
    if request.check.unwrap_or(false) {
        args.push("--check");
    }
    // Note: --error flag would need escaping, skipped for now

    run_ckrv_command(state, &args)
}

// ============================================================================
// Helpers
// ============================================================================

/// Run a ckrv command and return the result.
fn run_ckrv_command(state: &AppState, args: &[&str]) -> Result<CommandResponse, TransportError> {
    let output = Command::new("ckrv")
        .args(args)
        .current_dir(&state.project_root)
        .output()
        .map_err(|e| TransportError::Internal(format!("Failed to run ckrv: {e}")))?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    Ok(CommandResponse {
        success: output.status.success(),
        message: if output.status.success() {
            Some("Command completed successfully".to_string())
        } else {
            None
        },
        output: Some(stdout),
        error: if stderr.is_empty() { None } else { Some(stderr) },
    })
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_command_response_default() {
        let response = CommandResponse {
            success: true,
            message: Some("test".to_string()),
            output: None,
            error: None,
        };
        assert!(response.success);
    }
}
