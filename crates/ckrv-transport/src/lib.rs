//! # ckrv-transport
//!
//! Transport abstraction layer for Chakravarti CLI.
//!
//! ## Overview
//!
//! This crate provides a unified handler layer that can be used by both
//! web (Axum) and desktop (Tauri) backends. The core handlers are
//! transport-agnostic, with feature-gated wrappers for each transport.
//!
//! ## Features
//!
//! - `axum` - Enable Axum HTTP/WebSocket transport wrappers
//! - `tauri` - Enable Tauri IPC command wrappers  
//! - `typescript` - Enable TypeScript type generation via ts-rs
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────┐
//! │                   ckrv-transport                         │
//! │  ┌───────────────────────────────────────────────────┐  │
//! │  │              handlers/ (core logic)               │  │
//! │  │  status.rs, agents.rs, specs.rs, execution.rs...  │  │
//! │  └───────────────────────────────────────────────────┘  │
//! │                          │                              │
//! │           ┌──────────────┼──────────────┐               │
//! │           ▼              ▼              ▼               │
//! │  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐     │
//! │  │ axum/       │  │ tauri/      │  │ types/      │     │
//! │  │ (feature)   │  │ (feature)   │  │ (always)    │     │
//! │  └─────────────┘  └─────────────┘  └─────────────┘     │
//! └─────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Usage
//!
//! ```rust,ignore
//! // In ckrv-ui (web server)
//! use ckrv_transport::axum::create_router;
//! let router = create_router(state);
//!
//! // In ckrv-tauri (desktop app)
//! use ckrv_transport::tauri::get_invoke_handlers;
//! let handlers = get_invoke_handlers();
//! ```
//!
//! ## Module Overview
//!
//! - [`error`] - Transport error types with transport-specific conversions
//! - [`state`] - Shared application state
//! - [`types`] - Request/response type definitions
//! - [`handlers`] - Transport-agnostic handler implementations
//! - [`hub`] - Event broadcasting infrastructure
//! - [`axum`] - Axum-specific route wrappers (feature-gated)
//! - [`tauri`] - Tauri-specific command wrappers (feature-gated)

// ============================================================
// Core Modules (always compiled)
// ============================================================

pub mod error;
pub mod handlers;
pub mod hub;
pub mod state;
pub mod types;

// ============================================================
// Transport-Specific Modules (feature-gated)
// ============================================================

/// Axum HTTP/WebSocket transport wrappers.
///
/// Provides `create_router()` function that returns an Axum `Router`
/// with all API routes configured.
#[cfg(feature = "axum")]
pub mod axum;

/// Tauri IPC command wrappers.
///
/// Provides `get_invoke_handlers()` function that returns a list of
/// Tauri command handlers for IPC registration.
#[cfg(feature = "tauri")]
pub mod tauri;

// ============================================================
// Re-exports
// ============================================================

pub use error::TransportError;
pub use hub::{Hub, OrchestrationEvent, SharedHub};
pub use state::{
    AppState, RunEntry, RunRegistry, RunStatus, SharedRunRegistry, SystemMode, SystemStatus,
};

// Re-export types for convenience
pub use types::*;
