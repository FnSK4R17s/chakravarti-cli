//! # Axum Transport
//!
//! Axum HTTP/WebSocket transport wrappers.
//!
//! ## Overview
//!
//! This module provides the `create_router()` function that returns
//! an Axum `Router` with all API routes configured.
//!
//! ## Usage
//!
//! ```rust,ignore
//! use ckrv_transport::axum::create_router;
//! use ckrv_transport::AppState;
//!
//! let state = AppState::new(project_root);
//! let router = create_router(state);
//!
//! // Serve with Axum
//! let listener = TcpListener::bind("0.0.0.0:3000").await?;
//! axum::serve(listener, router).await?;
//! ```

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
pub mod terminal;
pub mod test;

use crate::state::AppState;
use axum::Router;

/// Create the API router with all routes configured.
///
/// This function creates a complete Axum router with all API endpoints.
/// The router expects to be nested under `/api`.
///
/// # Example
///
/// ```rust,ignore
/// use ckrv_transport::axum::create_router;
/// use ckrv_transport::AppState;
///
/// let state = AppState::new(project_root);
/// let app = Router::new()
///     .nest("/api", create_router(state.clone()))
///     .with_state(state);
/// ```
pub fn create_router(state: AppState) -> Router<AppState> {
    Router::new()
        // Status routes
        .merge(status::routes())
        // Docker routes
        .merge(docker::routes())
        // Cloud routes
        .merge(cloud::routes())
        // Agent routes
        .merge(agents::routes())
        // Spec routes
        .merge(specs::routes())
        // Plan routes
        .merge(plans::routes())
        // Task routes
        .merge(tasks::routes())
        // History routes
        .merge(history::routes())
        // Execution routes
        .merge(execution::routes())
        // Command routes
        .merge(commands::routes())
        // Console routes
        .merge(console::routes())
        // Diff routes
        .merge(diff::routes())
        // QA routes
        .merge(qa::routes())
        // Test routes
        .merge(test::routes())
        // Session routes
        .merge(session::routes())
        // Terminal routes (WebSocket)
        .merge(terminal::routes())
        // Events routes (SSE)
        .merge(events::routes())
        // Example routes (reference implementation)
        .merge(example::routes())
        .with_state(state)
}

/// Create the API router with CORS enabled.
///
/// This is a convenience function that wraps `create_router` with
/// permissive CORS settings for development.
pub fn create_router_with_cors(state: AppState) -> Router<AppState> {
    use tower_http::cors::{Any, CorsLayer};

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    create_router(state).layer(cors)
}
