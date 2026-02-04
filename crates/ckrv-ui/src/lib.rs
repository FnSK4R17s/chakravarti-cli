//! # ckrv-ui
//!
//! Web dashboard backend for Chakravarti orchestration.
//!
//! ## Overview
//!
//! This crate provides the HTTP server and WebSocket infrastructure for the
//! Chakravarti web UI. It leverages the `ckrv-transport` crate for all API
//! handlers and exposes a unified web server that serves:
//! - Specification management (CRUD, status)
//! - Execution control (start, stop, resume)
//! - Real-time log streaming via WebSocket
//! - History and metrics viewing
//! - Agent configuration
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────┐
//! │                    Browser (React)                               │
//! └──────────────────────────┬──────────────────────────────────────┘
//!                            │ HTTP/WebSocket
//! ┌──────────────────────────▼──────────────────────────────────────┐
//! │                    ckrv-ui Server                                │
//! │  ┌─────────────────────────────────────────────────────────┐    │
//! │  │                 ckrv-transport                           │    │
//! │  │   handlers/ → axum/ → Router                            │    │
//! │  └─────────────────────────────────────────────────────────┘    │
//! │                              │                                   │
//! │  ┌───────────┐  ┌───────────┐  ┌───────────┐                    │
//! │  │  server   │  │ services  │  │   hub     │ ← Event broadcast  │
//! │  └───────────┘  └───────────┘  └───────────┘                    │
//! └──────────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Modules
//!
//! - [`hub`] - Event broadcasting to connected clients
//! - [`models`] - Data types for logs, history, etc.
//! - [`server`] - Axum server setup and lifecycle
//! - [`services`] - Business logic (execution, commands)
//!
//! ## Re-exports
//!
//! Types from `ckrv-transport` are re-exported for convenience:
//! - [`AppState`] - Shared application state
//! - [`SystemStatus`] - Current orchestration status
//! - [`SystemMode`] - Active mode (idle, planning, running, etc.)
//!
//! ## Example
//!
//! ```rust,ignore
//! use ckrv_ui::start_server;
//!
//! #[tokio::main]
//! async fn main() {
//!     let port = 3000;
//!     start_server(port).await.expect("Server failed");
//! }
//! ```

pub mod models;
pub mod server;
pub mod services;

// Re-export types from ckrv-transport for backward compatibility
pub use ckrv_transport::{AppState, Hub, OrchestrationEvent, SharedHub, SystemMode, SystemStatus};

// Re-export the main entry point
pub use server::start_server;
