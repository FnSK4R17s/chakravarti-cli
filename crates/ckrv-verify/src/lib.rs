//! Verification pipeline for Chakravarti CLI.
//!
//! This crate handles test execution, result parsing, and acceptance criteria checking.

pub mod acceptance;
pub mod error;
pub mod parser;
pub mod runner;
pub mod verdict;

pub use acceptance::{AcceptanceChecker, AcceptanceResult, CriterionResult};
pub use error::VerifyError;
pub use parser::{TestFramework, TestOutputParser};
pub use runner::{DefaultVerifier, Verifier, VerifyConfig};
pub use verdict::{TestResult, TestStatus, Verdict};
