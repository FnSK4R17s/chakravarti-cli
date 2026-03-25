//! # Commands Handler
//!
//! Handlers for CLI command execution.

use crate::error::TransportError;
use crate::state::AppState;
use serde::{Deserialize, Serialize};
use std::process::Command;

// ============================================================
// Request/Response Types
// ============================================================

/// Command execution response.
#[derive(Debug, Serialize)]
pub struct CommandResponse {
    /// Whether the command succeeded.
    pub success: bool,
    /// Success message.
    pub message: Option<String>,
    /// Standard output from the command.
    pub output: Option<String>,
    /// Error output if the command failed.
    pub error: Option<String>,
}

/// Spec creation request.
#[derive(Debug, Deserialize)]
pub struct SpecNewRequest {
    /// Description of what to build.
    pub description: String,
    /// Optional name for the spec.
    pub name: Option<String>,
}

/// Diff command request.
#[derive(Debug, Deserialize)]
pub struct DiffRequest {
    /// Base branch for comparison.
    pub base: Option<String>,
    /// Show diffstat summary.
    pub stat: Option<bool>,
    /// Show changed file names only.
    pub files: Option<bool>,
    /// Show abbreviated summary.
    pub summary: Option<bool>,
}

/// Verify command request.
#[derive(Debug, Deserialize)]
pub struct VerifyRequest {
    /// Run lint checks.
    pub lint: Option<bool>,
    /// Run type checking.
    pub typecheck: Option<bool>,
    /// Run test suite.
    pub test: Option<bool>,
    /// Auto-fix issues found.
    pub fix: Option<bool>,
}

/// Promote command request.
#[derive(Debug, Deserialize)]
pub struct PromoteRequest {
    /// Base branch to promote to.
    pub base: Option<String>,
    /// Create as a draft PR.
    pub draft: Option<bool>,
    /// Push to remote before creating PR.
    pub push: Option<bool>,
}

/// Plan command request.
#[derive(Debug, Deserialize)]
pub struct PlanRequest {
    /// Spec name to generate plan for (auto-detects from branch if omitted).
    pub spec: Option<String>,
}

/// Fix command request.
#[derive(Debug, Deserialize)]
pub struct FixRequest {
    /// Fix lint issues.
    pub lint: Option<bool>,
    /// Fix type errors.
    pub typecheck: Option<bool>,
    /// Fix failing tests.
    pub test: Option<bool>,
    /// Run all checks after fixing.
    pub check: Option<bool>,
    /// Specific error message to fix.
    pub error: Option<String>,
}

// ============================================================
// Handlers
// ============================================================

/// Run ckrv init command.
pub fn run_init_handler(state: &AppState) -> Result<CommandResponse, TransportError> {
    run_ckrv_command(state, &["init"])
}

/// Run git init command.
pub fn run_git_init_handler(state: &AppState) -> Result<CommandResponse, TransportError> {
    let output = Command::new("git")
        .args(["init"])
        .current_dir(&state.project_root)
        .output()
        .map_err(|e| TransportError::Internal(format!("Failed to run git init: {e}")))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        return Err(TransportError::Internal(if stderr.is_empty() {
            "git init failed".to_string()
        } else {
            stderr
        }));
    }

    Ok(CommandResponse {
        success: true,
        message: Some("Git repository initialized".to_string()),
        output: Some(String::from_utf8_lossy(&output.stdout).to_string()),
        error: None,
    })
}

/// Run ckrv spec new command.
pub fn run_spec_new_handler(
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
pub fn run_spec_tasks_handler(state: &AppState) -> Result<CommandResponse, TransportError> {
    run_ckrv_command(state, &["spec", "tasks"])
}

/// Run ckrv plan command.
pub fn run_plan_handler(
    state: &AppState,
    request: PlanRequest,
) -> Result<CommandResponse, TransportError> {
    let mut args = vec!["plan", "--json"];
    let spec_path;
    if let Some(ref spec) = request.spec {
        spec_path = format!(".specs/{}", spec);
        args.push(&spec_path);
    }
    run_ckrv_command(state, &args)
}

/// Run ckrv code run command.
pub fn run_execute_handler(state: &AppState) -> Result<CommandResponse, TransportError> {
    run_ckrv_command(state, &["code", "run"])
}

/// Run ckrv diff command.
pub fn run_diff_handler(
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
pub fn run_verify_handler(
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
pub fn run_promote_handler(
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
pub fn run_fix_handler(
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

// ============================================================
// Helpers
// ============================================================

/// Run a ckrv command and return the result.
///
/// Returns `TransportError::Internal` with stderr/stdout detail when the
/// command exits with a non-zero status, so callers (and ultimately the HTTP
/// layer) surface a proper error response instead of HTTP 200 with
/// `success: false`.
fn run_ckrv_command(state: &AppState, args: &[&str]) -> Result<CommandResponse, TransportError> {
    let output = Command::new("ckrv")
        .args(args)
        .current_dir(&state.project_root)
        .output()
        .map_err(|e| TransportError::Internal(format!("Failed to run ckrv: {e}")))?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    if !output.status.success() {
        let error_detail = if !stderr.is_empty() {
            stderr
        } else if !stdout.is_empty() {
            stdout
        } else {
            format!(
                "Command 'ckrv {}' failed with exit code {}",
                args.join(" "),
                output.status.code().unwrap_or(-1)
            )
        };
        return Err(TransportError::Internal(error_detail));
    }

    Ok(CommandResponse {
        success: true,
        message: Some("Command completed successfully".to_string()),
        output: Some(stdout),
        error: if stderr.is_empty() {
            None
        } else {
            Some(stderr)
        },
    })
}

// ============================================================
// Tests
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;

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

    #[test]
    fn test_git_init_nonexistent_dir_returns_error() {
        let state = AppState::new(std::path::PathBuf::from(
            "/nonexistent/path/that/does/not/exist",
        ));
        let result = run_git_init_handler(&state);
        assert!(
            result.is_err(),
            "git init in nonexistent dir should return Err"
        );
        match result {
            Err(TransportError::Internal(msg)) => {
                assert!(!msg.is_empty(), "Error message should not be empty");
            }
            _ => panic!("Expected TransportError::Internal"),
        }
    }

    #[test]
    fn test_git_init_valid_tempdir_succeeds() {
        let dir = tempfile::TempDir::new().expect("create temp dir");
        let state = AppState::new(dir.path().to_path_buf());
        let result = run_git_init_handler(&state);
        assert!(result.is_ok(), "git init in valid dir should succeed");
        let response = result.unwrap();
        assert!(response.success);
        assert!(response.message.unwrap().contains("initialized"));
    }

    #[test]
    fn test_ckrv_command_nonexistent_binary_returns_internal_error() {
        // When ckrv binary is not in PATH, the command should fail with Internal error
        let dir = tempfile::TempDir::new().expect("create temp dir");
        let state = AppState::new(dir.path().to_path_buf());

        // Clear PATH so ckrv binary can't be found
        let result = Command::new("ckrv")
            .args(["--version"])
            .env("PATH", "")
            .current_dir(dir.path())
            .output();

        // If ckrv isn't available at all, test the handler error path
        if result.is_err() || !result.unwrap().status.success() {
            let result = run_init_handler(&state);
            // Should be Err because ckrv either isn't installed or project isn't initialized
            assert!(result.is_err(), "Should return error when ckrv fails");
            match result {
                Err(TransportError::Internal(msg)) => {
                    assert!(!msg.is_empty(), "Error message should contain details");
                }
                _ => panic!("Expected TransportError::Internal"),
            }
        }
    }

    #[test]
    fn test_spec_new_handler_builds_correct_args() {
        // Test that the handler constructs args correctly (will fail at execution
        // since ckrv isn't running, but tests the error return path)
        let dir = tempfile::TempDir::new().expect("create temp dir");
        let state = AppState::new(dir.path().to_path_buf());
        let request = SpecNewRequest {
            description: "test feature".to_string(),
            name: Some("my-spec".to_string()),
        };

        let result = run_spec_new_handler(&state, request);
        // Should fail (ckrv not initialized) but return a proper error, not Ok
        assert!(result.is_err(), "Should return Err when ckrv command fails");
    }

    #[test]
    fn test_ckrv_command_nonexistent_dir_returns_error() {
        // Use a nonexistent directory so the command spawn fails
        let state = AppState::new(std::path::PathBuf::from(
            "/nonexistent/ckrv/test/project/path",
        ));
        let request = VerifyRequest {
            lint: Some(true),
            typecheck: None,
            test: None,
            fix: None,
        };

        let result = run_verify_handler(&state, request);
        assert!(result.is_err(), "Should return Err for nonexistent dir");
        match result {
            Err(TransportError::Internal(msg)) => {
                assert!(!msg.is_empty(), "Error should contain details");
            }
            _ => panic!("Expected TransportError::Internal"),
        }
    }
}
