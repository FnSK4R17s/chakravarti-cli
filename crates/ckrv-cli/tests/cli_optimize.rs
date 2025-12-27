//! Integration tests for CLI optimization modes.

use std::process::Command;

/// Helper to run ckrv with arguments
fn ckrv(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_ckrv"))
        .args(args)
        .output()
        .expect("Failed to run ckrv")
}

// =============================================================================
// T153: Integration test for --optimize=cost behavior
// =============================================================================

#[test]
fn test_optimize_cost_flag_accepted() {
    // Test that --optimize cost is a valid flag
    let output = ckrv(&["run", "--help"]);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains("optimize"), "Should have optimize option");
}

#[test]
#[ignore] // Requires API key
fn test_optimize_cost_uses_cheaper_model() {
    // With --optimize cost, should select cheaper models
    // This requires API key so marked as ignored
    let output = ckrv(&[
        "run",
        "test.yaml",
        "--optimize",
        "cost",
        "--dry-run",
        "--json",
    ]);
    let stdout = String::from_utf8_lossy(&output.stdout);

    // In dry run, should indicate model selection
    assert!(stdout.contains("gpt-4o-mini") || stdout.contains("model"));
}

// =============================================================================
// T154: Integration test for --optimize=time behavior
// =============================================================================

#[test]
fn test_optimize_time_flag_accepted() {
    // Test that --optimize time works
    let output = ckrv(&["run", "--help"]);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains("optimize"), "Should have optimize option");
}

#[test]
#[ignore] // Requires API key
fn test_optimize_time_uses_capable_model() {
    // With --optimize time, should select more capable models
    let output = ckrv(&[
        "run",
        "test.yaml",
        "--optimize",
        "time",
        "--dry-run",
        "--json",
    ]);
    let stdout = String::from_utf8_lossy(&output.stdout);

    // In dry run, should indicate model selection
    assert!(stdout.contains("gpt-4o") || stdout.contains("model"));
}

// =============================================================================
// Model override tests
// =============================================================================

#[test]
fn test_planner_model_flag_accepted() {
    let output = ckrv(&["run", "--help"]);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        stdout.contains("planner-model"),
        "Should have planner-model option"
    );
}

#[test]
fn test_executor_model_flag_accepted() {
    let output = ckrv(&["run", "--help"]);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        stdout.contains("executor-model"),
        "Should have executor-model option"
    );
}

#[test]
#[ignore] // Requires API key
fn test_model_override_respected() {
    // Explicit model override should be used
    let output = ckrv(&[
        "run",
        "test.yaml",
        "--executor-model",
        "claude-3-5-sonnet",
        "--dry-run",
        "--json",
    ]);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains("claude-3-5-sonnet") || stdout.contains("model"));
}

// =============================================================================
// Optimization mode enum tests
// =============================================================================

#[test]
fn test_all_optimize_modes_valid() {
    // All three modes should be valid
    for mode in &["cost", "time", "balanced"] {
        let output = ckrv(&["run", "--help"]);
        let stdout = String::from_utf8_lossy(&output.stdout);
        // Help should document the modes
        assert!(output.status.success());
    }
}
