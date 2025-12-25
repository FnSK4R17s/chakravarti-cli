//! Acceptance criteria checking.
//!
//! Compares spec.acceptance against execution results.

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

    /// Check if verification results meet spec acceptance criteria.
    #[must_use]
    pub fn check(&self, spec: &Spec, verdict: &Verdict) -> AcceptanceResult {
        let mut criteria = Vec::new();

        for acceptance in &spec.acceptance {
            let (met, evidence) = self.evaluate_criterion(acceptance, verdict);
            criteria.push(CriterionResult {
                criterion: acceptance.clone(),
                met,
                evidence,
            });
        }

        let passed = criteria.iter().all(|c| c.met);

        AcceptanceResult { passed, criteria }
    }

    fn evaluate_criterion(&self, criterion: &str, verdict: &Verdict) -> (bool, String) {
        let criterion_lower = criterion.to_lowercase();

        // Check common patterns
        if criterion_lower.contains("tests pass") || criterion_lower.contains("all tests") {
            if verdict.passed {
                (true, format!("All {} tests passed", verdict.passed_count()))
            } else {
                (false, format!("{} tests failed", verdict.failed_count()))
            }
        } else if criterion_lower.contains("no errors") || criterion_lower.contains("error-free") {
            let has_errors = verdict
                .logs
                .iter()
                .any(|l| l.to_lowercase().contains("error"));
            if has_errors {
                (false, "Errors found in logs".to_string())
            } else {
                (true, "No errors in logs".to_string())
            }
        } else if criterion_lower.contains("compiles") || criterion_lower.contains("builds") {
            // Assume if tests ran, it compiled
            if !verdict.test_results.is_empty() {
                (true, "Build succeeded (tests executed)".to_string())
            } else {
                (false, "Unable to verify build status".to_string())
            }
        } else {
            // For unrecognized criteria, check if tests passed as a proxy
            if verdict.passed {
                (true, "Assumed met (tests passed)".to_string())
            } else {
                (false, "Cannot verify (tests failed)".to_string())
            }
        }
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
            id: "test".to_string(),
            goal: "Test goal".to_string(),
            constraints: vec![],
            acceptance: vec![
                "All tests pass".to_string(),
                "No errors in output".to_string(),
            ],
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
