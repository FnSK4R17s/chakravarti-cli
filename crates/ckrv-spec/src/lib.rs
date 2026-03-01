//! Spec-Kit integration and parsing for Chakravarti CLI.
//!
//! This crate handles loading, parsing, and validating specification files.

// ============================================================
// MODULES & RE-EXPORTS
// ============================================================

/// Spec loading and parsing errors.
pub mod error;
/// Spec file loading from disk.
pub mod loader;
/// Spec template generation.
pub mod template;
/// Spec validation rules.
pub mod validator;

pub use error::SpecError;
pub use loader::SpecLoader;
pub use validator::{ValidationError, ValidationResult};
