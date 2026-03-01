//! Shared services for CLI commands.
//!
//! These modules provide reusable functionality used across
//! multiple CLI command implementations.

/// Agent configuration lookup and management.
pub mod agent_lookup;
/// Git diff analysis utilities.
pub mod diff_analyzer;
/// Markdown report generation for test and QA results.
pub mod report_generator;
/// Test framework detection and execution.
pub mod test_framework;
