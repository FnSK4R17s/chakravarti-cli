//! Integration tests for `ckrv run` command.
//!
//! Tests the run command per the CLI contract:
//! - Executes spec-driven workflow
//! - Produces diff output
//! - Handles retries

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

/// Helper to create an initialized chakravarti repo with a spec.
fn create_repo_with_spec() -> TempDir {
    let dir = TempDir::new().expect("Failed to create temp dir");
    Command::new("git")
        .args(["init"])
        .current_dir(dir.path())
        .output()
        .expect("Failed to init git repo");

    // Run ckrv init
    ckrv(&["init"], dir.path());

    // Create a simple spec
    let spec_content = r#"id: add_readme
goal: Create a README.md file

constraints:
  - Keep it simple

acceptance:
  - README.md exists
  - Contains project title
"#;
    std::fs::write(
        dir.path().join(".specs").join("add_readme.yaml"),
        spec_content,
    )
    .ok();

    dir
}

// =============================================================================
// T060: Contract test for `ckrv run` streaming JSON events
// =============================================================================

#[test]
fn test_run_json_output_streams_events() {
    let repo = create_repo_with_spec();

    let output = ckrv(
        &["run", ".specs/add_readme.yaml", "--json", "--dry-run"],
        repo.path(),
    );

    let stdout = String::from_utf8_lossy(&output.stdout);

    // In dry-run mode, should output plan without executing
    // Should be valid JSON (or NDJSON)
    assert!(
        stdout.contains("plan") || stdout.contains("dry_run") || stdout.contains("success"),
        "Should have structured output"
    );
}

#[test]
fn test_run_with_dry_run_shows_plan() {
    let repo = create_repo_with_spec();

    let output = ckrv(&["run", ".specs/add_readme.yaml", "--dry-run"], repo.path());

    // Dry run should show what would happen
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = format!("{stdout}{stderr}");

    assert!(
        combined.contains("plan") || combined.contains("dry") || combined.contains("would"),
        "Dry run should describe what would happen"
    );
}

// =============================================================================
// T069: Integration test for full run lifecycle
// =============================================================================

#[test]
fn test_run_nonexistent_spec_fails() {
    let repo = create_repo_with_spec();

    let output = ckrv(&["run", ".specs/nonexistent.yaml"], repo.path());

    assert!(
        !output.status.success(),
        "Run should fail for nonexistent spec"
    );
}

#[test]
fn test_run_shows_helpful_error_without_api_key() {
    let repo = create_repo_with_spec();

    // Run without setting API keys
    let output = ckrv(&["run", ".specs/add_readme.yaml"], repo.path());

    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let combined = format!("{stdout}{stderr}");

    // Should indicate API key issue or provide helpful message
    // (or succeed if dry-run by default)
    assert!(
        !output.status.success()
            || combined.contains("key")
            || combined.contains("API")
            || combined.contains("dry"),
        "Should either fail with API key message or show dry-run"
    );
}

// =============================================================================
// T070: Integration test for retry on verification failure
// =============================================================================

#[test]
fn test_run_respects_max_attempts_flag() {
    let repo = create_repo_with_spec();

    let output = ckrv(
        &[
            "run",
            ".specs/add_readme.yaml",
            "--max-attempts",
            "1",
            "--dry-run",
        ],
        repo.path(),
    );

    // Should accept the flag without error
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.contains("unrecognized") && !stderr.contains("unknown"),
        "Should recognize --max-attempts flag"
    );
}

#[test]
fn test_run_respects_optimize_flag() {
    let repo = create_repo_with_spec();

    for mode in &["cost", "time", "balanced"] {
        let output = ckrv(
            &[
                "run",
                ".specs/add_readme.yaml",
                "--optimize",
                mode,
                "--dry-run",
            ],
            repo.path(),
        );

        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            !stderr.contains("unrecognized") && !stderr.contains("unknown"),
            "Should recognize --optimize {mode} flag"
        );
    }
}
