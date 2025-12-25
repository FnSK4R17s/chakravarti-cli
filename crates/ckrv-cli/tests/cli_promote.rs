//! Integration tests for CLI promote command.

use std::process::Command;

/// Helper to run ckrv with arguments
fn ckrv(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_ckrv"))
        .args(args)
        .output()
        .expect("Failed to run ckrv")
}

// =============================================================================
// T135: Contract test for `ckrv promote` JSON output
// =============================================================================

#[test]
fn test_promote_json_output_structure() {
    let output = ckrv(&["promote", "test-job", "--branch", "test-branch", "--json"]);
    let stdout = String::from_utf8_lossy(&output.stdout);
    
    // Should be valid JSON
    let json: serde_json::Value = serde_json::from_str(&stdout)
        .expect("Promote output should be valid JSON");
    
    // Should have required fields
    assert!(json.get("job_id").is_some(), "Missing job_id field");
    assert!(json.get("branch").is_some(), "Missing branch field");
    assert!(json.get("promoted").is_some(), "Missing promoted field");
}

#[test]
fn test_promote_nonexistent_job() {
    let output = ckrv(&["promote", "fake-job-id-12345", "--branch", "test-branch", "--json"]);
    let stdout = String::from_utf8_lossy(&output.stdout);
    
    let json: serde_json::Value = serde_json::from_str(&stdout)
        .expect("Promote output should be valid JSON");
    
    assert_eq!(json["promoted"], false);
    assert!(json.get("error").is_some());
}

// =============================================================================
// T137: Integration test for promoting successful job
// =============================================================================

#[test]
#[ignore] // Requires successful job
fn test_promote_successful_job() {
    // This test requires a successful job, which needs API keys and git setup
    let output = ckrv(&["promote", "test-job", "--branch", "feature/test", "--json"]);
    let stdout = String::from_utf8_lossy(&output.stdout);
    
    let json: serde_json::Value = serde_json::from_str(&stdout)
        .expect("Promote output should be valid JSON");
    
    // If job exists and succeeded, promoted should be true
    if json["error"].is_null() {
        assert_eq!(json["promoted"], true);
    }
}

// =============================================================================
// T138: Integration test for refusing to promote failed job
// =============================================================================

#[test]
#[ignore] // Requires failed job
fn test_promote_refuses_failed_job() {
    // This test requires a failed job
    let output = ckrv(&["promote", "failed-job", "--branch", "feature/test", "--json"]);
    let stdout = String::from_utf8_lossy(&output.stdout);
    
    let json: serde_json::Value = serde_json::from_str(&stdout)
        .expect("Promote output should be valid JSON");
    
    assert_eq!(json["promoted"], false);
    // Should have error about job not succeeding
    assert!(json["error"].as_str().map_or(false, |e| e.contains("succeed") || e.contains("failed")));
}

// =============================================================================
// T139: Integration test for --force overwrite
// =============================================================================

#[test]
fn test_promote_force_flag_accepted() {
    let output = ckrv(&["promote", "test-job", "--branch", "test-branch", "--force", "--json"]);
    let stdout = String::from_utf8_lossy(&output.stdout);
    
    // Should be valid JSON (command should accept --force flag)
    let json: serde_json::Value = serde_json::from_str(&stdout)
        .expect("Promote output should be valid JSON");
    
    assert!(json.get("job_id").is_some());
}

#[test]
fn test_promote_push_flag_accepted() {
    let output = ckrv(&["promote", "test-job", "--branch", "test-branch", "--push", "--json"]);
    let stdout = String::from_utf8_lossy(&output.stdout);
    
    // Should be valid JSON (command should accept --push flag)
    let json: serde_json::Value = serde_json::from_str(&stdout)
        .expect("Promote output should be valid JSON");
    
    assert!(json.get("job_id").is_some());
    assert!(json.get("pushed").is_some());
}

#[test]
fn test_promote_remote_flag_accepted() {
    let output = ckrv(&["promote", "test-job", "--branch", "test-branch", "--push", "--remote", "upstream", "--json"]);
    let stdout = String::from_utf8_lossy(&output.stdout);
    
    // Should be valid JSON
    let json: serde_json::Value = serde_json::from_str(&stdout)
        .expect("Promote output should be valid JSON");
    
    assert!(json.get("job_id").is_some());
}
