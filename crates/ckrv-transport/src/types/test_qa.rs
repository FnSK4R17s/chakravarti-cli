//! # Test and QA Types
//!
//! Types for test and QA command handlers.

use serde::{Deserialize, Serialize};

#[cfg(feature = "typescript")]
use ts_rs::TS;

// ============================================================
// Test Types
// ============================================================

/// Test run request.
#[derive(Debug, Clone, Deserialize)]
#[cfg_attr(feature = "typescript", derive(TS))]
#[cfg_attr(feature = "typescript", ts(export))]
pub struct TestRunRequest {
    /// Path to test (file, directory, or pattern)
    pub path: Option<String>,

    /// Test framework to use
    pub framework: Option<String>,

    /// Whether to run in watch mode
    pub watch: Option<bool>,

    /// Filter pattern for tests
    pub filter: Option<String>,
}

/// Test plan request.
#[derive(Debug, Clone, Deserialize)]
#[cfg_attr(feature = "typescript", derive(TS))]
#[cfg_attr(feature = "typescript", ts(export))]
pub struct TestPlanRequest {
    /// Spec to generate test plan for
    pub spec: String,

    /// Agent to use for planning
    pub agent: Option<String>,
}

/// Test write request.
#[derive(Debug, Clone, Deserialize)]
#[cfg_attr(feature = "typescript", derive(TS))]
#[cfg_attr(feature = "typescript", ts(export))]
pub struct TestWriteRequest {
    /// Path to generate tests for
    pub path: String,

    /// Agent to use for writing tests
    pub agent: Option<String>,

    /// Test framework to use
    pub framework: Option<String>,
}

/// Test run result.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "typescript", derive(TS))]
#[cfg_attr(feature = "typescript", ts(export))]
pub struct TestResult {
    /// Whether tests passed
    pub passed: bool,

    /// Number of passed tests
    pub passed_count: usize,

    /// Number of failed tests
    pub failed_count: usize,

    /// Number of skipped tests
    pub skipped_count: usize,

    /// Total test duration in milliseconds
    pub duration_ms: u64,

    /// Test output/logs
    pub output: String,

    /// Individual test results
    pub tests: Vec<TestCaseResult>,
}

/// Individual test case result.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "typescript", derive(TS))]
#[cfg_attr(feature = "typescript", ts(export))]
pub struct TestCaseResult {
    /// Test name
    pub name: String,

    /// Test status
    pub status: TestCaseStatus,

    /// Duration in milliseconds
    pub duration_ms: Option<u64>,

    /// Error message if failed
    pub error: Option<String>,
}

/// Test case status.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(feature = "typescript", derive(TS))]
#[cfg_attr(feature = "typescript", ts(export))]
#[serde(rename_all = "snake_case")]
pub enum TestCaseStatus {
    /// Test passed
    Passed,
    /// Test failed
    Failed,
    /// Test skipped
    Skipped,
}

// ============================================================
// QA Types
// ============================================================

/// QA review request.
#[derive(Debug, Clone, Deserialize)]
#[cfg_attr(feature = "typescript", derive(TS))]
#[cfg_attr(feature = "typescript", ts(export))]
pub struct QaReviewRequest {
    /// Path to review
    pub path: Option<String>,

    /// Spec to review
    pub spec: Option<String>,

    /// Agent to use for review
    pub agent: Option<String>,
}

/// QA bugs request.
#[derive(Debug, Clone, Deserialize)]
#[cfg_attr(feature = "typescript", derive(TS))]
#[cfg_attr(feature = "typescript", ts(export))]
pub struct QaBugsRequest {
    /// Path to scan for bugs
    pub path: Option<String>,

    /// Agent to use for bug detection
    pub agent: Option<String>,
}

/// QA report request.
#[derive(Debug, Clone, Deserialize)]
#[cfg_attr(feature = "typescript", derive(TS))]
#[cfg_attr(feature = "typescript", ts(export))]
pub struct QaReportRequest {
    /// Spec to generate report for
    pub spec: String,

    /// Report format (markdown, html, json)
    pub format: Option<String>,
}

/// QA review result.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "typescript", derive(TS))]
#[cfg_attr(feature = "typescript", ts(export))]
pub struct QaReviewResult {
    /// Overall quality score (0-100)
    pub score: u32,

    /// Summary of findings
    pub summary: String,

    /// Detailed issues found
    pub issues: Vec<QaIssue>,

    /// Recommendations
    pub recommendations: Vec<String>,
}

/// QA issue.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "typescript", derive(TS))]
#[cfg_attr(feature = "typescript", ts(export))]
pub struct QaIssue {
    /// Issue severity
    pub severity: IssueSeverity,

    /// Issue category
    pub category: String,

    /// Issue description
    pub description: String,

    /// File path if applicable
    pub file: Option<String>,

    /// Line number if applicable
    pub line: Option<u32>,

    /// Suggested fix
    pub fix: Option<String>,
}

/// Issue severity level.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(feature = "typescript", derive(TS))]
#[cfg_attr(feature = "typescript", ts(export))]
#[serde(rename_all = "snake_case")]
pub enum IssueSeverity {
    /// Critical issue that blocks deployment
    Critical,
    /// High priority issue
    High,
    /// Medium priority issue
    Medium,
    /// Low priority suggestion
    Low,
    /// Informational note
    Info,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_test_case_status_serialization() {
        let status = TestCaseStatus::Passed;
        let json = serde_json::to_string(&status).expect("serialization failed");
        assert_eq!(json, "\"passed\"");
    }

    #[test]
    fn test_issue_severity_serialization() {
        let severity = IssueSeverity::Critical;
        let json = serde_json::to_string(&severity).expect("serialization failed");
        assert_eq!(json, "\"critical\"");
    }
}
