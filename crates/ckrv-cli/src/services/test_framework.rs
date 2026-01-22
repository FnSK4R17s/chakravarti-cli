//! Test framework detection and execution.

use std::path::Path;
use std::process::Command;
use std::time::Instant;

use serde::Serialize;

/// Supported test frameworks
#[derive(Debug, Clone, PartialEq)]
pub enum TestFramework {
    Cargo,
    Npm,
    Pytest,
    GoTest,
    Make,
    Unknown,
}

impl TestFramework {
    pub fn name(&self) -> &'static str {
        match self {
            TestFramework::Cargo => "Cargo (Rust)",
            TestFramework::Npm => "npm (Node.js)",
            TestFramework::Pytest => "pytest (Python)",
            TestFramework::GoTest => "go test (Go)",
            TestFramework::Make => "make test",
            TestFramework::Unknown => "Unknown",
        }
    }
}

/// Test execution result
#[derive(Debug, Clone, Serialize)]
pub struct TestResult {
    pub success: bool,
    pub framework: String,
    pub total: u32,
    pub passed: u32,
    pub failed: u32,
    pub skipped: u32,
    pub duration_ms: u64,
    pub stdout: String,
    pub stderr: String,
    pub failures: Vec<TestFailure>,
}

/// Individual test failure
#[derive(Debug, Clone, Serialize)]
pub struct TestFailure {
    pub name: String,
    pub file: Option<String>,
    pub line: Option<u32>,
    pub message: String,
}

/// Detect the test framework for a project
pub fn detect_framework(cwd: &Path) -> TestFramework {
    // Check for Rust
    if cwd.join("Cargo.toml").exists() {
        return TestFramework::Cargo;
    }
    
    // Check for Go
    if cwd.join("go.mod").exists() {
        return TestFramework::GoTest;
    }
    
    // Check for Python
    if cwd.join("pyproject.toml").exists() 
        || cwd.join("pytest.ini").exists()
        || cwd.join("setup.py").exists() 
    {
        return TestFramework::Pytest;
    }
    
    // Check for Node.js
    if cwd.join("package.json").exists() {
        return TestFramework::Npm;
    }
    
    // Check for Makefile as fallback
    if cwd.join("Makefile").exists() || cwd.join("makefile").exists() {
        // Check if Makefile has a test target
        if let Ok(content) = std::fs::read_to_string(cwd.join("Makefile")) {
            if content.contains("test:") || content.contains("test :") {
                return TestFramework::Make;
            }
        }
        if let Ok(content) = std::fs::read_to_string(cwd.join("makefile")) {
            if content.contains("test:") || content.contains("test :") {
                return TestFramework::Make;
            }
        }
    }
    
    TestFramework::Unknown
}

/// Get the command and arguments to run tests
pub fn get_test_command(framework: &TestFramework) -> (String, Vec<String>) {
    match framework {
        TestFramework::Cargo => ("cargo".to_string(), vec!["test".to_string()]),
        TestFramework::Npm => ("npm".to_string(), vec!["test".to_string(), "--".to_string(), "--passWithNoTests".to_string()]),
        TestFramework::Pytest => ("pytest".to_string(), vec!["-v".to_string()]),
        TestFramework::GoTest => ("go".to_string(), vec!["test".to_string(), "./...".to_string()]),
        TestFramework::Make => ("make".to_string(), vec!["test".to_string()]),
        TestFramework::Unknown => ("echo".to_string(), vec!["No test framework detected".to_string()]),
    }
}

/// Run tests locally (not in sandbox)
pub async fn run_tests_local(cwd: &Path) -> TestResult {
    let start = Instant::now();
    let framework = detect_framework(cwd);
    let (cmd, args) = get_test_command(&framework);
    
    let output = Command::new(&cmd)
        .args(&args)
        .current_dir(cwd)
        .output();
    
    let duration_ms = start.elapsed().as_millis() as u64;
    
    match output {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout).to_string();
            let stderr = String::from_utf8_lossy(&out.stderr).to_string();
            let success = out.status.success();
            
            // Parse test counts from output (simplified)
            let (total, passed, failed) = parse_test_counts(&stdout, &stderr, &framework);
            let failures = if !success {
                parse_failures(&stdout, &stderr, &framework)
            } else {
                vec![]
            };
            
            TestResult {
                success,
                framework: framework.name().to_string(),
                total,
                passed,
                failed,
                skipped: 0,
                duration_ms,
                stdout,
                stderr,
                failures,
            }
        }
        Err(e) => TestResult {
            success: false,
            framework: framework.name().to_string(),
            total: 0,
            passed: 0,
            failed: 1,
            skipped: 0,
            duration_ms,
            stdout: String::new(),
            stderr: format!("Failed to run {}: {}", cmd, e),
            failures: vec![TestFailure {
                name: "test_runner".to_string(),
                file: None,
                line: None,
                message: format!("Failed to run tests: {}", e),
            }],
        },
    }
}

/// Parse test counts from output (simplified heuristics)
fn parse_test_counts(stdout: &str, stderr: &str, framework: &TestFramework) -> (u32, u32, u32) {
    let combined = format!("{}\n{}", stdout, stderr);
    
    match framework {
        TestFramework::Cargo => {
            // Look for "test result: ok. X passed; Y failed"
            for line in combined.lines() {
                if line.contains("test result:") {
                    let passed = extract_number(line, "passed");
                    let failed = extract_number(line, "failed");
                    return (passed + failed, passed, failed);
                }
            }
        }
        TestFramework::Pytest => {
            // Look for "X passed, Y failed"
            for line in combined.lines() {
                if line.contains("passed") || line.contains("failed") {
                    let passed = extract_number(line, "passed");
                    let failed = extract_number(line, "failed");
                    if passed > 0 || failed > 0 {
                        return (passed + failed, passed, failed);
                    }
                }
            }
        }
        _ => {}
    }
    
    // Default: if no parse, assume 1 test
    (1, if combined.contains("FAIL") || combined.contains("FAILED") { 0 } else { 1 }, 
     if combined.contains("FAIL") || combined.contains("FAILED") { 1 } else { 0 })
}

/// Extract a number before a keyword (e.g., "5 passed")
fn extract_number(text: &str, keyword: &str) -> u32 {
    let parts: Vec<&str> = text.split_whitespace().collect();
    for (i, part) in parts.iter().enumerate() {
        if part.contains(keyword) && i > 0 {
            if let Ok(n) = parts[i - 1].trim_matches(|c: char| !c.is_numeric()).parse() {
                return n;
            }
        }
    }
    0
}

/// Parse failure details from output (simplified)
fn parse_failures(_stdout: &str, stderr: &str, _framework: &TestFramework) -> Vec<TestFailure> {
    // Simple heuristic: look for lines containing "FAILED" or "Error"
    let mut failures = vec![];
    
    for line in stderr.lines() {
        if line.contains("FAILED") || line.contains("error[") || line.contains("Error:") {
            failures.push(TestFailure {
                name: "test".to_string(),
                file: None,
                line: None,
                message: line.trim().to_string(),
            });
        }
    }
    
    if failures.is_empty() && stderr.contains("FAIL") {
        failures.push(TestFailure {
            name: "test".to_string(),
            file: None,
            line: None,
            message: "Test failed - see output for details".to_string(),
        });
    }
    
    failures
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    
    #[test]
    fn test_detect_framework_rust() {
        // This crate has Cargo.toml
        let cwd = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        assert_eq!(detect_framework(&cwd), TestFramework::Cargo);
    }
    
    #[test]
    fn test_get_test_command_cargo() {
        let (cmd, args) = get_test_command(&TestFramework::Cargo);
        assert_eq!(cmd, "cargo");
        assert_eq!(args, vec!["test"]);
    }
    
    #[test]
    fn test_get_test_command_make() {
        let (cmd, args) = get_test_command(&TestFramework::Make);
        assert_eq!(cmd, "make");
        assert_eq!(args, vec!["test"]);
    }
    
    #[test]
    fn test_extract_number() {
        assert_eq!(extract_number("5 passed", "passed"), 5);
        assert_eq!(extract_number("test result: ok. 10 passed; 2 failed", "passed"), 10);
        assert_eq!(extract_number("test result: ok. 10 passed; 2 failed", "failed"), 2);
    }
}
