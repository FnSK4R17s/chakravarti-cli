//! Verification pipeline for Chakravarti CLI.
//!
//! This crate handles test execution, result parsing, and acceptance criteria checking.

// ============================================================
// MODULES & RE-EXPORTS
// ============================================================

/// Acceptance criteria checking against spec requirements.
pub mod acceptance;
/// Verification error types.
pub mod error;
/// Test output parsing for multiple frameworks.
pub mod parser;
/// Test command execution and result collection.
pub mod runner;
/// Verdict and test result types.
pub mod verdict;

pub use acceptance::{AcceptanceChecker, AcceptanceResult, CriterionResult};
pub use error::VerifyError;
pub use parser::{TestFramework, TestOutputParser};
pub use runner::{DefaultVerifier, Verifier, VerifyConfig};
pub use verdict::{TestResult, TestStatus, Verdict};
