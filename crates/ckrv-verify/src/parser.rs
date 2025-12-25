//! Test output parser.
//!
//! Parses output from various test frameworks (cargo test, npm test, etc.)

use crate::{TestResult, TestStatus};

/// Supported test frameworks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TestFramework {
    /// Rust's cargo test.
    Cargo,
    /// Node.js npm/yarn test.
    Npm,
    /// Python pytest/unittest.
    Python,
    /// Go test.
    Go,
    /// Generic (parse exit code only).
    Generic,
}

/// Parser for test output.
pub struct TestOutputParser {
    framework: TestFramework,
}

impl TestOutputParser {
    /// Create a new parser for a specific framework.
    #[must_use]
    pub fn new(framework: TestFramework) -> Self {
        Self { framework }
    }

    /// Detect framework from command string.
    #[must_use]
    pub fn detect_framework(command: &str) -> TestFramework {
        if command.contains("cargo test") {
            TestFramework::Cargo
        } else if command.contains("npm test") || command.contains("yarn test") {
            TestFramework::Npm
        } else if command.contains("pytest") || command.contains("python -m unittest") {
            TestFramework::Python
        } else if command.contains("go test") {
            TestFramework::Go
        } else {
            TestFramework::Generic
        }
    }

    /// Parse test output into structured results.
    #[must_use]
    pub fn parse(&self, output: &str, success: bool, duration_ms: u64) -> Vec<TestResult> {
        match self.framework {
            TestFramework::Cargo => self.parse_cargo(output, duration_ms),
            TestFramework::Npm => self.parse_npm(output, duration_ms),
            TestFramework::Python => self.parse_python(output, duration_ms),
            TestFramework::Go => self.parse_go(output, duration_ms),
            TestFramework::Generic => self.parse_generic(output, success, duration_ms),
        }
    }

    fn parse_cargo(&self, output: &str, duration_ms: u64) -> Vec<TestResult> {
        let mut results = Vec::new();

        for line in output.lines() {
            let line = line.trim();

            // Parse "test module::test_name ... ok" or "test module::test_name ... FAILED"
            if line.starts_with("test ") && (line.ends_with(" ok") || line.ends_with(" FAILED")) {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 3 {
                    let name = parts[1].to_string();
                    let status = if line.ends_with(" ok") {
                        TestStatus::Passed
                    } else {
                        TestStatus::Failed
                    };

                    results.push(TestResult {
                        name,
                        status,
                        duration_ms: 0, // Individual timing not available
                        output: None,
                    });
                }
            }
        }

        // If no individual tests found, create a summary result
        if results.is_empty() {
            let passed = output.contains("test result: ok");
            results.push(TestResult {
                name: "cargo test".to_string(),
                status: if passed {
                    TestStatus::Passed
                } else {
                    TestStatus::Failed
                },
                duration_ms,
                output: Some(output.to_string()),
            });
        }

        results
    }

    fn parse_npm(&self, output: &str, duration_ms: u64) -> Vec<TestResult> {
        let mut results = Vec::new();

        // Look for common npm test patterns
        for line in output.lines() {
            let line = line.trim();

            // Jest format: ✓ test name (10 ms)
            if line.starts_with('✓') || line.starts_with("✓") {
                let name = line.trim_start_matches('✓').trim().to_string();
                results.push(TestResult {
                    name,
                    status: TestStatus::Passed,
                    duration_ms: 0,
                    output: None,
                });
            } else if line.starts_with('✕') || line.starts_with("✕") {
                let name = line.trim_start_matches('✕').trim().to_string();
                results.push(TestResult {
                    name,
                    status: TestStatus::Failed,
                    duration_ms: 0,
                    output: None,
                });
            }
        }

        if results.is_empty() {
            let passed = !output.contains("FAIL") && !output.contains("Error:");
            results.push(TestResult {
                name: "npm test".to_string(),
                status: if passed {
                    TestStatus::Passed
                } else {
                    TestStatus::Failed
                },
                duration_ms,
                output: Some(output.to_string()),
            });
        }

        results
    }

    fn parse_python(&self, output: &str, duration_ms: u64) -> Vec<TestResult> {
        let mut results = Vec::new();

        for line in output.lines() {
            let line = line.trim();

            // pytest format: test_name PASSED or test_name FAILED
            if line.contains(" PASSED") {
                let name = line
                    .split(" PASSED")
                    .next()
                    .unwrap_or("")
                    .trim()
                    .to_string();
                if !name.is_empty() {
                    results.push(TestResult {
                        name,
                        status: TestStatus::Passed,
                        duration_ms: 0,
                        output: None,
                    });
                }
            } else if line.contains(" FAILED") {
                let name = line
                    .split(" FAILED")
                    .next()
                    .unwrap_or("")
                    .trim()
                    .to_string();
                if !name.is_empty() {
                    results.push(TestResult {
                        name,
                        status: TestStatus::Failed,
                        duration_ms: 0,
                        output: None,
                    });
                }
            }
        }

        if results.is_empty() {
            let passed = output.contains("passed") && !output.contains("failed");
            results.push(TestResult {
                name: "pytest".to_string(),
                status: if passed {
                    TestStatus::Passed
                } else {
                    TestStatus::Failed
                },
                duration_ms,
                output: Some(output.to_string()),
            });
        }

        results
    }

    fn parse_go(&self, output: &str, duration_ms: u64) -> Vec<TestResult> {
        let mut results = Vec::new();

        for line in output.lines() {
            let line = line.trim();

            // Go format: --- PASS: TestName (0.00s) or --- FAIL: TestName (0.00s)
            if line.starts_with("--- PASS:") {
                let name = line
                    .trim_start_matches("--- PASS:")
                    .split('(')
                    .next()
                    .unwrap_or("")
                    .trim()
                    .to_string();
                results.push(TestResult {
                    name,
                    status: TestStatus::Passed,
                    duration_ms: 0,
                    output: None,
                });
            } else if line.starts_with("--- FAIL:") {
                let name = line
                    .trim_start_matches("--- FAIL:")
                    .split('(')
                    .next()
                    .unwrap_or("")
                    .trim()
                    .to_string();
                results.push(TestResult {
                    name,
                    status: TestStatus::Failed,
                    duration_ms: 0,
                    output: None,
                });
            }
        }

        if results.is_empty() {
            let passed = output.contains("PASS") && !output.contains("FAIL");
            results.push(TestResult {
                name: "go test".to_string(),
                status: if passed {
                    TestStatus::Passed
                } else {
                    TestStatus::Failed
                },
                duration_ms,
                output: Some(output.to_string()),
            });
        }

        results
    }

    fn parse_generic(&self, output: &str, success: bool, duration_ms: u64) -> Vec<TestResult> {
        vec![TestResult {
            name: "test".to_string(),
            status: if success {
                TestStatus::Passed
            } else {
                TestStatus::Failed
            },
            duration_ms,
            output: Some(output.to_string()),
        }]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_framework_cargo() {
        assert_eq!(
            TestOutputParser::detect_framework("cargo test --lib"),
            TestFramework::Cargo
        );
    }

    #[test]
    fn test_detect_framework_npm() {
        assert_eq!(
            TestOutputParser::detect_framework("npm test"),
            TestFramework::Npm
        );
    }

    #[test]
    fn test_detect_framework_python() {
        assert_eq!(
            TestOutputParser::detect_framework("pytest tests/"),
            TestFramework::Python
        );
    }

    #[test]
    fn test_detect_framework_go() {
        assert_eq!(
            TestOutputParser::detect_framework("go test ./..."),
            TestFramework::Go
        );
    }

    #[test]
    fn test_parse_cargo_output() {
        let output = r#"
running 3 tests
test tests::test_one ... ok
test tests::test_two ... ok
test tests::test_three ... FAILED

test result: FAILED. 2 passed; 1 failed; 0 ignored
"#;
        let parser = TestOutputParser::new(TestFramework::Cargo);
        let results = parser.parse(output, false, 100);

        assert_eq!(results.len(), 3);
        assert_eq!(results[0].status, TestStatus::Passed);
        assert_eq!(results[2].status, TestStatus::Failed);
    }

    #[test]
    fn test_parse_go_output() {
        let output = r#"
=== RUN   TestAdd
--- PASS: TestAdd (0.00s)
=== RUN   TestSubtract
--- FAIL: TestSubtract (0.00s)
FAIL
"#;
        let parser = TestOutputParser::new(TestFramework::Go);
        let results = parser.parse(output, false, 100);

        assert_eq!(results.len(), 2);
        assert_eq!(results[0].name, "TestAdd");
        assert_eq!(results[0].status, TestStatus::Passed);
        assert_eq!(results[1].status, TestStatus::Failed);
    }

    #[test]
    fn test_parse_generic() {
        let parser = TestOutputParser::new(TestFramework::Generic);
        let results = parser.parse("some output", true, 50);

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].status, TestStatus::Passed);
    }
}
