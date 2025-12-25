//! Integration tests for `ckrv spec` commands.
//!
//! Tests the spec commands per the CLI contract:
//! - `ckrv spec new <name>` creates a spec file
//! - `ckrv spec validate <path>` validates a spec file

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

/// Helper to create an initialized chakravarti repo.
fn create_initialized_repo() -> TempDir {
    let dir = TempDir::new().expect("Failed to create temp dir");
    Command::new("git")
        .args(["init"])
        .current_dir(dir.path())
        .output()
        .expect("Failed to init git repo");

    // Run ckrv init
    let output = ckrv(&["init"], dir.path());
    assert!(output.status.success(), "init should succeed");

    dir
}

// =============================================================================
// T043: Contract test for `ckrv spec new` JSON output
// =============================================================================

#[test]
fn test_spec_new_json_output_has_required_fields() {
    let repo = create_initialized_repo();

    let output = ckrv(&["spec", "new", "test_feature", "--json"], repo.path());

    assert!(output.status.success(), "spec new should succeed");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("Should be valid JSON");

    assert!(
        json.get("success").is_some(),
        "JSON must have 'success' field"
    );
    assert!(
        json.get("spec_path").is_some(),
        "JSON must have 'spec_path' field"
    );
}

#[test]
fn test_spec_new_creates_file() {
    let repo = create_initialized_repo();

    let output = ckrv(&["spec", "new", "my_feature"], repo.path());

    assert!(output.status.success(), "spec new should succeed");
    assert!(
        repo.path().join(".specs").join("my_feature.yaml").exists(),
        "Spec file should exist"
    );
}

// =============================================================================
// T044: Contract test for `ckrv spec validate` JSON output
// =============================================================================

#[test]
fn test_spec_validate_json_output_has_required_fields() {
    let repo = create_initialized_repo();

    // Create a spec first
    ckrv(&["spec", "new", "test_spec"], repo.path());

    let output = ckrv(
        &["spec", "validate", ".specs/test_spec.yaml", "--json"],
        repo.path(),
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("Should be valid JSON");

    assert!(json.get("valid").is_some(), "JSON must have 'valid' field");
}

// =============================================================================
// T047: Integration test for creating new spec
// =============================================================================

#[test]
fn test_spec_new_with_goal_flag() {
    let repo = create_initialized_repo();

    let output = ckrv(
        &["spec", "new", "rate_limiter", "--goal", "Add rate limiting"],
        repo.path(),
    );

    assert!(output.status.success(), "spec new with goal should succeed");

    let spec_path = repo.path().join(".specs").join("rate_limiter.yaml");
    let content = std::fs::read_to_string(&spec_path).expect("read spec");
    assert!(
        content.contains("Add rate limiting"),
        "Spec should contain the goal"
    );
}

#[test]
fn test_spec_new_fails_without_init() {
    let dir = TempDir::new().expect("temp dir");
    Command::new("git")
        .args(["init"])
        .current_dir(dir.path())
        .output()
        .ok();
    // Don't run ckrv init

    let output = ckrv(&["spec", "new", "test"], dir.path());

    assert!(
        !output.status.success(),
        "spec new should fail without init"
    );
}

// =============================================================================
// T048: Integration test for validating valid spec
// =============================================================================

#[test]
fn test_spec_validate_valid_spec_succeeds() {
    let repo = create_initialized_repo();

    // Create a valid spec
    let spec_content = r#"id: valid_spec
goal: A valid goal statement

constraints:
  - Must not break tests

acceptance:
  - Tests pass
"#;
    let spec_path = repo.path().join(".specs").join("valid_spec.yaml");
    std::fs::write(&spec_path, spec_content).expect("write spec");

    let output = ckrv(&["spec", "validate", ".specs/valid_spec.yaml"], repo.path());

    assert!(
        output.status.success(),
        "validate should succeed for valid spec"
    );
}

// =============================================================================
// T049: Integration test for validating invalid spec
// =============================================================================

#[test]
fn test_spec_validate_missing_goal_fails() {
    let repo = create_initialized_repo();

    // Create an invalid spec (missing goal)
    let spec_content = r#"id: invalid_spec

acceptance:
  - Tests pass
"#;
    let spec_path = repo.path().join(".specs").join("invalid_spec.yaml");
    std::fs::write(&spec_path, spec_content).expect("write spec");

    let output = ckrv(
        &["spec", "validate", ".specs/invalid_spec.yaml"],
        repo.path(),
    );

    assert!(
        !output.status.success(),
        "validate should fail for invalid spec"
    );
}

#[test]
fn test_spec_validate_missing_acceptance_fails() {
    let repo = create_initialized_repo();

    // Create an invalid spec (missing acceptance)
    let spec_content = r#"id: invalid_spec
goal: Some goal
"#;
    let spec_path = repo.path().join(".specs").join("invalid_spec.yaml");
    std::fs::write(&spec_path, spec_content).expect("write spec");

    let output = ckrv(
        &["spec", "validate", ".specs/invalid_spec.yaml"],
        repo.path(),
    );

    assert!(
        !output.status.success(),
        "validate should fail for spec without acceptance"
    );
}

#[test]
fn test_spec_validate_invalid_id_fails() {
    let repo = create_initialized_repo();

    // Create an invalid spec (bad id format)
    let spec_content = r#"id: invalid-id-with-dashes
goal: Some goal

acceptance:
  - Tests pass
"#;
    let spec_path = repo.path().join(".specs").join("bad_id.yaml");
    std::fs::write(&spec_path, spec_content).expect("write spec");

    let output = ckrv(&["spec", "validate", ".specs/bad_id.yaml"], repo.path());

    assert!(
        !output.status.success(),
        "validate should fail for invalid id format"
    );
}

#[test]
fn test_spec_validate_nonexistent_file_fails() {
    let repo = create_initialized_repo();

    let output = ckrv(
        &["spec", "validate", ".specs/does_not_exist.yaml"],
        repo.path(),
    );

    assert!(
        !output.status.success(),
        "validate should fail for nonexistent file"
    );
}
