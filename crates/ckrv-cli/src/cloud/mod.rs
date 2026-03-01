//! Cloud client library for Chakravarti CLI.
//!
//! This module provides the client infrastructure for communicating
//! with the Chakravarti Cloud API.

/// OAuth2 device flow authentication.
pub mod auth;
/// HTTP client for Cloud API requests.
pub mod client;
/// Cloud configuration management.
pub mod config;
/// Credential storage for authentication tokens.
pub mod credentials;
/// Cloud-specific error types.
pub mod error;
/// Job-related cloud operations.
pub mod jobs;
/// Log streaming and retrieval.
pub mod logs;

/// Re-export the primary cloud client.
pub use client::CloudClient;
/// Re-export the cloud configuration struct.
pub use config::CloudConfig;
/// Re-export the cloud error type.
pub use error::CloudError;
