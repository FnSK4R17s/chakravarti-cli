//! # ckrv-ui
//!
//! Web dashboard backend for Chakravarti orchestration.
//!
//! ## Overview
//!
//! This crate provides the HTTP server and WebSocket infrastructure for the
//! Chakravarti web UI. It exposes APIs for:
//! - Specification management (CRUD, status)
//! - Execution control (start, stop, resume)
//! - Real-time log streaming via WebSocket
//! - History and metrics viewing
//! - Agent configuration
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────┐
//! │                    Browser (React)                       │
//! └──────────────────────────┬──────────────────────────────┘
//!                            │ HTTP/WebSocket
//! ┌──────────────────────────▼──────────────────────────────┐
//! │                    ckrv-ui Server                        │
//! │  ┌───────────┐  ┌───────────┐  ┌───────────┐            │
//! │  │  api/     │  │ services/ │  │  models/  │            │
//! │  │  routes   │  │  engine   │  │   types   │            │
//! │  └─────┬─────┘  └─────┬─────┘  └───────────┘            │
//! │        │              │                                  │
//! │        └──────────────┼──────────────────────────────────┤
//! │                       ▼                                  │
//! │                 ┌───────────┐                            │
//! │                 │   hub     │  ← Event broadcasting      │
//! │                 └───────────┘                            │
//! └──────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Modules
//!
//! - [`api`] - HTTP route handlers (REST + WebSocket)
//! - [`hub`] - Event broadcasting to connected clients
//! - [`models`] - Data types for logs, history, etc.
//! - [`server`] - Axum server setup and lifecycle
//! - [`services`] - Business logic (execution, commands)
//! - [`state`] - Shared application state
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

pub mod api;
pub mod hub;
pub mod models;
pub mod server;
pub mod services;
pub mod state;

pub use server::start_server;
