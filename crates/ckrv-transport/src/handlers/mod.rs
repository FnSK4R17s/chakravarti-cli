//! # Transport Handlers
//!
//! Transport-agnostic handler implementations.
//!
//! ## Overview
//!
//! This module contains all the business logic for API endpoints.
//! Handlers are transport-agnostic and return `Result<T, TransportError>`.
//!
//! ## Handler Pattern
//!
//! All handlers follow this signature:
//!
//! ```rust,ignore
//! pub async fn handler_name(
//!     state: &AppState,
//!     request: RequestType,  // Optional
//! ) -> Result<ResponseType, TransportError>
//! ```
//!
//! ## Adding New Handlers
//!
//! 1. Create handler function in appropriate module
//! 2. Add Axum wrapper in `axum/` module (if axum feature enabled)
//! 3. Add Tauri command in `tauri/` module (if tauri feature enabled)
//!
//! See `docs/adding-endpoints.md` for detailed instructions.

#![allow(clippy::needless_pass_by_value)]

pub mod agents;
pub mod cloud;
pub mod commands;
pub mod console;
pub mod diff;
pub mod docker;
pub mod events;
pub mod example;
pub mod execution;
pub mod history;
pub mod plans;
pub mod qa;
pub mod session;
pub mod specs;
pub mod status;
pub mod tasks;
pub mod test;

// Terminal handler uses WebSocket types from axum, so it's feature-gated
#[cfg(feature = "axum")]
pub mod terminal;

// Re-export handlers for convenience
pub use agents::*;
pub use cloud::*;
pub use commands::*;
pub use console::*;
pub use diff::*;
pub use docker::*;
pub use events::*;
pub use execution::*;
pub use history::*;
pub use plans::*;
pub use qa::*;
pub use session::*;
pub use specs::*;
pub use status::*;
pub use tasks::*;
pub use test::*;

#[cfg(feature = "axum")]
pub use terminal::*;
