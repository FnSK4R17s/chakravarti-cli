//! # API Routes
//!
//! HTTP route handlers for the ckrv-ui web server.
//!
//! ## Modules
//!
//! - [`agents`] - Agent configuration CRUD
//! - [`cloud`] - Cloud connection status
//! - [`commands`] - CLI command execution
//! - [`console`] - Interactive command console
//! - [`diff`] - Git diff viewing
//! - [`docker`] - Docker status checks
//! - [`events`] - Server-Sent Events stream
//! - [`execution`] - Batch execution control
//! - [`history`] - Run history management
//! - [`plans`] - Execution plan management
//! - [`qa`] - QA command handlers
//! - [`session`] - Docker session management
//! - [`specs`] - Specification CRUD
//! - [`status`] - System status endpoint
//! - [`tasks`] - Task management
//! - [`terminal`] - Interactive terminal WebSocket
//! - [`test`] - Test command handlers

pub mod agents;
pub mod cloud;
pub mod commands;
pub mod console;
pub mod diff;
pub mod docker;
pub mod events;
pub mod execution;
pub mod history;
pub mod plans;
pub mod qa;
pub mod session;
pub mod specs;
pub mod status;
pub mod tasks;
pub mod terminal;
pub mod test;
