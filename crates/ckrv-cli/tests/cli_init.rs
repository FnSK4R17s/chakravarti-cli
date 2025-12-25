//! Integration tests for `ckrv init` command.
//!
//! Tests the init command per the CLI contract:
//! - Creates `.specs/` and `.chakravarti/` directories
//! - Creates default configuration
//! - Handles already-initialized repos
//! - Rejects non-git directories

use std::process::Command;

use tempfile::TempDir;

/// Helper to run the ckrv binary with arguments.
fn ckrv(args: &[&str], cwd: &std::path::Path) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_ckrv"))
        .args(args)
        .current_dir(cwd)
        .output()
        .expect("Failed to execute ckrv")
}

/// Helper to create a git repository in a temp directory.
fn create_git_repo() -> TempDir {
    let dir = TempDir::new().expect("Failed to create temp dir");
    Command::new("git")
        .args(["init"])
        .current_dir(dir.path())
        .output()
        .expect("Failed to init git repo");
    dir
}

// =============================================================================
// T029: Contract test for `ckrv init` JSON output
// =============================================================================

#[test]
fn test_init_json_output_has_required_fields() {
    let repo = create_git_repo();

    let output = ckrv(&["init", "--json"], repo.path());

    assert!(output.status.success(), "init should succeed");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value =
        serde_json::from_str(&stdout).expect("Output should be valid JSON");

    // Contract: JSON output must have these fields
    assert!(
        json.get("success").is_some(),
        "JSON must have 'success' field"
    );
    assert!(
        json.get("specs_dir").is_some(),
        "JSON must have 'specs_dir' field"
    );
    assert!(
        json.get("chakravarti_dir").is_some(),
        "JSON must have 'chakravarti_dir' field"
    );
}

#[test]
fn test_init_json_exit_code_zero_on_success() {
    let repo = create_git_repo();

    let output = ckrv(&["init", "--json"], repo.path());

    assert!(output.status.success(), "Exit code should be 0 on success");
    assert_eq!(output.status.code(), Some(0));
}

// =============================================================================
// T030: Integration test for init in fresh git repo
// =============================================================================

#[test]
fn test_init_creates_specs_directory() {
    let repo = create_git_repo();

    let output = ckrv(&["init"], repo.path());

    assert!(output.status.success(), "init should succeed");
    assert!(
        repo.path().join(".specs").exists(),
        ".specs directory should exist"
    );
}

#[test]
fn test_init_creates_chakravarti_directory() {
    let repo = create_git_repo();

    let output = ckrv(&["init"], repo.path());

    assert!(output.status.success(), "init should succeed");
    assert!(
        repo.path().join(".chakravarti").exists(),
        ".chakravarti directory should exist"
    );
}

#[test]
fn test_init_creates_config_file() {
    let repo = create_git_repo();

    let output = ckrv(&["init"], repo.path());

    assert!(output.status.success(), "init should succeed");
    assert!(
        repo.path()
            .join(".chakravarti")
            .join("config.json")
            .exists(),
        "config.json should exist"
    );
}

// =============================================================================
// T031: Integration test for init in already-initialized repo
// =============================================================================

#[test]
fn test_init_already_initialized_warns() {
    let repo = create_git_repo();

    // First init
    let output1 = ckrv(&["init"], repo.path());
    assert!(output1.status.success(), "First init should succeed");

    // Second init without --force
    let output2 = ckrv(&["init"], repo.path());

    // Should succeed but indicate already initialized
    assert!(output2.status.success(), "Second init should not fail");
    let stderr = String::from_utf8_lossy(&output2.stderr);
    let stdout = String::from_utf8_lossy(&output2.stdout);
    let combined = format!("{stdout}{stderr}");
    assert!(
        combined.contains("already") || combined.contains("Already"),
        "Should mention already initialized"
    );
}

#[test]
fn test_init_force_reinitializes() {
    let repo = create_git_repo();

    // First init
    ckrv(&["init"], repo.path());

    // Second init with --force
    let output = ckrv(&["init", "--force"], repo.path());

    assert!(output.status.success(), "Force init should succeed");
}

// =============================================================================
// T032: Integration test for init in non-git directory
// =============================================================================

#[test]
fn test_init_non_git_directory_fails() {
    let dir = TempDir::new().expect("Failed to create temp dir");
    // Don't init git

    let output = ckrv(&["init"], dir.path());

    assert!(
        !output.status.success(),
        "init should fail in non-git directory"
    );
}

#[test]
fn test_init_non_git_directory_exit_code_nonzero() {
    let dir = TempDir::new().expect("Failed to create temp dir");

    let output = ckrv(&["init"], dir.path());

    assert_ne!(
        output.status.code(),
        Some(0),
        "Exit code should be non-zero"
    );
}

#[test]
fn test_init_non_git_directory_shows_error_message() {
    let dir = TempDir::new().expect("Failed to create temp dir");

    let output = ckrv(&["init"], dir.path());

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("git") || stderr.contains("repository"),
        "Error should mention git repository requirement"
    );
}
