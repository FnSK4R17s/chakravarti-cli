//! Spec-Kit integration and parsing for Chakravarti CLI.
//!
//! This crate handles loading, parsing, and validating specification files.

pub mod error;
pub mod loader;
pub mod template;
pub mod validator;

pub use error::SpecError;
pub use loader::SpecLoader;
pub use validator::{ValidationError, ValidationResult};
