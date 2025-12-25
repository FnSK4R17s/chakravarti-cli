//! Verification verdict types.

use serde::{Deserialize, Serialize};

/// Result of verification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Verdict {
    /// Whether all tests passed.
    pub passed: bool,
    /// Individual test results.
    pub test_results: Vec<TestResult>,
    /// Captured logs.
    pub logs: Vec<String>,
    /// Artifact paths.
    pub artifacts: Vec<String>,
    /// Total duration in milliseconds.
    pub duration_ms: u64,
}

impl Verdict {
    /// Create a passing verdict.
    #[must_use]
    pub fn pass(test_results: Vec<TestResult>, duration_ms: u64) -> Self {
        Self {
            passed: true,
            test_results,
            logs: Vec::new(),
            artifacts: Vec::new(),
            duration_ms,
        }
    }

    /// Create a failing verdict.
    #[must_use]
    pub fn fail(test_results: Vec<TestResult>, logs: Vec<String>, duration_ms: u64) -> Self {
        Self {
            passed: false,
            test_results,
            logs,
            artifacts: Vec::new(),
            duration_ms,
        }
    }

    /// Count passed tests.
    #[must_use]
    pub fn passed_count(&self) -> usize {
        self.test_results
            .iter()
            .filter(|t| t.status == TestStatus::Passed)
            .count()
    }

    /// Count failed tests.
    #[must_use]
    pub fn failed_count(&self) -> usize {
        self.test_results
            .iter()
            .filter(|t| t.status == TestStatus::Failed)
            .count()
    }

    /// Get summary string.
    #[must_use]
    pub fn summary(&self) -> String {
        format!(
            "{} passed, {} failed, {} total in {}ms",
            self.passed_count(),
            self.failed_count(),
            self.test_results.len(),
            self.duration_ms
        )
    }
}

/// Result of a single test.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestResult {
    /// Test name.
    pub name: String,
    /// Test status.
    pub status: TestStatus,
    /// Duration in milliseconds.
    pub duration_ms: u64,
    /// Test output.
    pub output: Option<String>,
}

impl TestResult {
    /// Create a passed test result.
    #[must_use]
    pub fn passed(name: impl Into<String>, duration_ms: u64) -> Self {
        Self {
            name: name.into(),
            status: TestStatus::Passed,
            duration_ms,
            output: None,
        }
    }

    /// Create a failed test result.
    #[must_use]
    pub fn failed(name: impl Into<String>, duration_ms: u64, output: Option<String>) -> Self {
        Self {
            name: name.into(),
            status: TestStatus::Failed,
            duration_ms,
            output,
        }
    }
}

/// Status of a test.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TestStatus {
    /// Test passed.
    Passed,
    /// Test failed.
    Failed,
    /// Test was skipped.
    Skipped,
    /// Test errored.
    Error,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verdict_pass() {
        let verdict = Verdict::pass(vec![TestResult::passed("test1", 100)], 150);
        assert!(verdict.passed);
        assert_eq!(verdict.passed_count(), 1);
        assert_eq!(verdict.failed_count(), 0);
    }

    #[test]
    fn test_verdict_fail() {
        let verdict = Verdict::fail(
            vec![
                TestResult::passed("test1", 50),
                TestResult::failed("test2", 100, Some("assertion failed".to_string())),
            ],
            Vec::new(),
            200,
        );
        assert!(!verdict.passed);
        assert_eq!(verdict.passed_count(), 1);
        assert_eq!(verdict.failed_count(), 1);
    }

    #[test]
    fn test_verdict_summary() {
        let verdict = Verdict::pass(
            vec![TestResult::passed("t1", 10), TestResult::passed("t2", 20)],
            30,
        );
        let summary = verdict.summary();
        assert!(summary.contains("2 passed"));
        assert!(summary.contains("0 failed"));
    }

    #[test]
    fn test_test_result_passed() {
        let result = TestResult::passed("my_test", 50);
        assert_eq!(result.name, "my_test");
        assert_eq!(result.status, TestStatus::Passed);
        assert!(result.output.is_none());
    }

    #[test]
    fn test_test_result_failed() {
        let result = TestResult::failed("bad_test", 100, Some("oops".to_string()));
        assert_eq!(result.status, TestStatus::Failed);
        assert_eq!(result.output, Some("oops".to_string()));
    }
}
