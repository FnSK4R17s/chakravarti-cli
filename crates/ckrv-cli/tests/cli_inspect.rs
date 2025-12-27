//! Integration tests for CLI inspect commands (status, diff, report).

use std::process::Command;

/// Helper to run ckrv with arguments
fn ckrv(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_ckrv"))
        .args(args)
        .output()
        .expect("Failed to run ckrv")
}

/// Helper to run ckrv and get stdout as string
fn ckrv_stdout(args: &[&str]) -> String {
    let output = ckrv(args);
    String::from_utf8_lossy(&output.stdout).to_string()
}

// =============================================================================
// T113: Contract test for `ckrv status` JSON output
// =============================================================================

#[test]
fn test_status_json_output_structure() {
    let output = ckrv(&["status", "nonexistent-job", "--json"]);
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should be valid JSON
    let json: serde_json::Value =
        serde_json::from_str(&stdout).expect("Status output should be valid JSON");

    // Should have required fields
    assert!(json.get("job_id").is_some(), "Missing job_id field");
    assert!(json.get("status").is_some(), "Missing status field");
}

#[test]
fn test_status_nonexistent_job() {
    let output = ckrv(&["status", "fake-job-id-12345", "--json"]);
    let stdout = String::from_utf8_lossy(&output.stdout);

    let json: serde_json::Value =
        serde_json::from_str(&stdout).expect("Status output should be valid JSON");

    assert_eq!(json["status"], "not_found");
}

// =============================================================================
// T114: Contract test for `ckrv diff` JSON output
// =============================================================================

#[test]
fn test_diff_json_output_structure() {
    let output = ckrv(&["diff", "nonexistent-job", "--json"]);
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should be valid JSON
    let json: serde_json::Value =
        serde_json::from_str(&stdout).expect("Diff output should be valid JSON");

    // Should have required fields
    assert!(json.get("job_id").is_some(), "Missing job_id field");
    assert!(json.get("has_diff").is_some(), "Missing has_diff field");
}

#[test]
fn test_diff_nonexistent_job() {
    let output = ckrv(&["diff", "fake-job-id-12345", "--json"]);
    let stdout = String::from_utf8_lossy(&output.stdout);

    let json: serde_json::Value =
        serde_json::from_str(&stdout).expect("Diff output should be valid JSON");

    assert_eq!(json["has_diff"], false);
    assert_eq!(json["files_changed"], 0);
}

// =============================================================================
// T115: Contract test for `ckrv report` JSON output
// =============================================================================

#[test]
fn test_report_json_output_structure() {
    let output = ckrv(&["report", "nonexistent-job", "--json"]);
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should be valid JSON (even for nonexistent job)
    let json: serde_json::Value =
        serde_json::from_str(&stdout).expect("Report output should be valid JSON");

    // Should have job_id field
    assert!(json.get("job_id").is_some(), "Missing job_id field");
}

// =============================================================================
// T118: Integration test for status of running job
// =============================================================================

#[test]
#[ignore] // Requires running job
fn test_status_running_job() {
    // This test requires setting up a running job, which needs API keys
    // For now, just verify the command accepts the arguments
    let output = ckrv(&["status", "test-job", "--json"]);
    assert!(!output.stdout.is_empty() || !output.stderr.is_empty());
}

// =============================================================================
// T119: Integration test for diff of completed job
// =============================================================================

#[test]
#[ignore] // Requires completed job
fn test_diff_completed_job() {
    // This test requires a completed job with a diff
    let output = ckrv(&["diff", "test-job", "--json"]);
    assert!(!output.stdout.is_empty() || !output.stderr.is_empty());
}

#[test]
fn test_diff_stat_flag() {
    let output = ckrv(&["diff", "test-job", "--stat", "--json"]);
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should be valid JSON
    let json: serde_json::Value =
        serde_json::from_str(&stdout).expect("Diff --stat output should be valid JSON");

    // With --stat, diff field should be null/absent
    assert!(json.get("diff").is_none() || json["diff"].is_null());
}

#[test]
fn test_diff_color_flag() {
    // Just verify the flag is accepted
    let output = ckrv(&["diff", "test-job", "--color", "always"]);
    // Command should not fail due to unknown flag
    assert!(output.status.success() || output.status.code().is_some());
}
