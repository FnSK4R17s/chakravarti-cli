//! GitLab/GitHub integrations for Chakravarti CLI.
//!
//! This crate provides optional integrations with GitLab and GitHub
//! for creating merge/pull requests.

// ============================================================
// MODULES & RE-EXPORTS
// ============================================================

/// GitLab merge request integration (feature-gated).
#[cfg(feature = "gitlab")]
pub mod gitlab;

/// GitHub pull request integration (feature-gated).
#[cfg(feature = "github")]
pub mod github;

/// Integration error types.
pub mod error;

pub use error::IntegrationError;
