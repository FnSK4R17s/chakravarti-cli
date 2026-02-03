//! # Services
//!
//! Business logic layer for the ckrv-ui web server.
//!
//! ## Modules
//!
//! - [`command`] - CLI command execution wrappers
//! - [`engine`] - Batch execution orchestrator
//! - [`history`] - Run history persistence
//! - [`log_store`] - Execution log storage

pub mod command;
pub mod engine;
pub mod history;
pub mod log_store;
