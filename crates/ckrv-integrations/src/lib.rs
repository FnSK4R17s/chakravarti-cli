//! GitLab/GitHub integrations for Chakravarti CLI.
//!
//! This crate provides optional integrations with GitLab and GitHub
//! for creating merge/pull requests.

#[cfg(feature = "gitlab")]
pub mod gitlab;

#[cfg(feature = "github")]
pub mod github;

pub mod error;

pub use error::IntegrationError;
