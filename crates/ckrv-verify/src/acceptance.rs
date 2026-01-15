//! Acceptance criteria checking.
//!
//! For the new spec format, acceptance is handled via user_stories with acceptance_scenarios.
//! This module provides a simple pass/fail check based on test verification results.

use ckrv_core::Spec;

use crate::Verdict;

/// Result of acceptance checking.
#[derive(Debug, Clone)]
pub struct AcceptanceResult {
    /// Whether all criteria are met.
    pub passed: bool,
    /// Individual criteria results.
    pub criteria: Vec<CriterionResult>,
}

/// Result for a single criterion.
#[derive(Debug, Clone)]
pub struct CriterionResult {
    /// The criterion text.
    pub criterion: String,
    /// Whether it was met.
    pub met: bool,
    /// Evidence or reason.
    pub evidence: String,
}

/// Checker for acceptance criteria.
pub struct AcceptanceChecker;

impl AcceptanceChecker {
    /// Create a new checker.
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    /// Check if verification results meet acceptance criteria.
    /// For new format specs, this checks if all tests pass.
    #[must_use]
    pub fn check(&self, spec: &Spec, verdict: &Verdict) -> AcceptanceResult {
        let mut criteria = Vec::new();

        // Basic criterion: all tests should pass
        let tests_criterion = CriterionResult {
            criterion: "All tests pass".to_string(),
            met: verdict.passed,
            evidence: if verdict.passed {
                format!("All {} tests passed", verdict.passed_count())
            } else {
                format!("{} tests failed", verdict.failed_count())
            },
        };
        criteria.push(tests_criterion);

        // Check for errors in logs
        let has_errors = verdict
            .logs
            .iter()
            .any(|l| l.to_lowercase().contains("error"));
        
        let errors_criterion = CriterionResult {
            criterion: "No errors in output".to_string(),
            met: !has_errors,
            evidence: if has_errors {
                "Errors found in logs".to_string()
            } else {
                "No errors in logs".to_string()
            },
        };
        criteria.push(errors_criterion);

        // Include spec overview for context (not as a criterion)
        let _overview = spec.description();

        let passed = criteria.iter().all(|c| c.met);

        AcceptanceResult { passed, criteria }
    }
}

impl Default for AcceptanceChecker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{TestResult, TestStatus};

    fn create_test_spec() -> Spec {
        Spec {
            id: "001-test-feature".to_string(),
            branch: None,
            created: None,
            status: None,
            overview: Some("Test feature overview".to_string()),
            constraints: vec![],
            verify: None,
            source_path: None,
        }
    }

    fn create_passing_verdict() -> Verdict {
        Verdict {
            passed: true,
            test_results: vec![TestResult {
                name: "test1".to_string(),
                status: TestStatus::Passed,
                duration_ms: 10,
                output: None,
            }],
            logs: vec![],
            artifacts: vec![],
            duration_ms: 100,
        }
    }

    fn create_failing_verdict() -> Verdict {
        Verdict {
            passed: false,
            test_results: vec![TestResult {
                name: "test1".to_string(),
                status: TestStatus::Failed,
                duration_ms: 10,
                output: Some("assertion failed".to_string()),
            }],
            logs: vec!["Error: test failed".to_string()],
            artifacts: vec![],
            duration_ms: 100,
        }
    }

    #[test]
    fn test_acceptance_check_passing() {
        let spec = create_test_spec();
        let verdict = create_passing_verdict();
        let checker = AcceptanceChecker::new();

        let result = checker.check(&spec, &verdict);

        assert!(result.passed);
        assert_eq!(result.criteria.len(), 2);
        assert!(result.criteria[0].met);
    }

    #[test]
    fn test_acceptance_check_failing() {
        let spec = create_test_spec();
        let verdict = create_failing_verdict();
        let checker = AcceptanceChecker::new();

        let result = checker.check(&spec, &verdict);

        assert!(!result.passed);
        assert!(!result.criteria[0].met); // tests didn't pass
        assert!(!result.criteria[1].met); // has errors in logs
    }

    #[test]
    fn test_criterion_result_structure() {
        let result = CriterionResult {
            criterion: "Tests pass".to_string(),
            met: true,
            evidence: "All 5 tests passed".to_string(),
        };

        assert!(result.met);
        assert!(result.evidence.contains("5"));
    }
}
