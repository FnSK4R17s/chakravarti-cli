//! # Web Server
//!
//! Axum-based HTTP server for the Chakravarti web dashboard.
//!
//! ## Overview
//!
//! This module configures and starts the web server that serves:
//! - Static frontend assets (embedded via `rust-embed`)
//! - REST API endpoints via `ckrv-transport` crate
//! - WebSocket endpoints for real-time streaming
//! - Server-Sent Events for orchestration updates
//!
//! ## Entry Point
//!
//! Use [`start_server`] to launch the server on a given port:
//!
//! ```rust,ignore
//! ckrv_ui::start_server(3000).await?;
//! ```
//!
//! ## Route Organization
//!
//! All API routes are provided by the `ckrv-transport` crate:
//! - `/api/specs/*` - Specification management
//! - `/api/tasks/*` - Task management
//! - `/api/execution/*` - Execution control
//! - `/api/agents/*` - Agent configuration
//! - `/api/test/*` - Test commands
//! - `/api/qa/*` - QA commands
//! - `/api/history/*` - Run history
//!
//! ## Environment Variables
//!
//! - `CKRV_PROJECT_ROOT` - Override project root (used in tests)

// ============================================================
// Imports
// ============================================================

use axum::{
    body::Body,
    http::{header, StatusCode, Uri},
    response::IntoResponse,
    routing::get,
    Router,
};
use ckrv_transport::Hub;
use rust_embed::RustEmbed;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::RwLock;

// Re-export AppState from transport for backward compatibility
pub use ckrv_transport::{AppState, SystemMode, SystemStatus};

// ============================================================
// Static Assets
// ============================================================

#[derive(RustEmbed)]
#[folder = "frontend/dist"]
struct FrontendAssets;

async fn static_handler(uri: Uri) -> impl IntoResponse {
    let path = uri.path().trim_start_matches('/');
    println!("Request path: '{}'", path);

    // 1. Try to find the exact file requested
    if !path.is_empty() {
        if let Some(content) = FrontendAssets::get(path) {
            let mime = mime_guess::from_path(path).first_or_octet_stream();
            return (
                [(header::CONTENT_TYPE, mime.as_ref())],
                Body::from(content.data),
            )
                .into_response();
        }
    }

    // 2. Fallback to index.html (for root / or SPA routing)
    if let Some(content) = FrontendAssets::get("index.html") {
        return (
            [(header::CONTENT_TYPE, "text/html")],
            Body::from(content.data),
        )
            .into_response();
    }

    (StatusCode::NOT_FOUND, "404 Not Found").into_response()
}

// ============================================================
// Server
// ============================================================

/// Start the web dashboard server on the given port.
///
/// Configures the Axum router with API routes from `ckrv-transport`,
/// a health check endpoint, and a static file fallback for the SPA frontend.
///
/// # Arguments
///
/// * `port` - TCP port to bind the HTTP listener to.
///
/// # Errors
///
/// Returns an error if the TCP listener fails to bind or the server exits unexpectedly.
pub async fn start_server(port: u16) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // T006: Support CKRV_PROJECT_ROOT env var for test isolation (TR-007)
    // E2E tests set this to a temporary directory to prevent modifying working code
    let project_root = std::env::var("CKRV_PROJECT_ROOT")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| std::env::current_dir().unwrap_or_default());

    if std::env::var("CKRV_PROJECT_ROOT").is_ok() {
        println!(
            "Using custom project root from CKRV_PROJECT_ROOT: {}",
            project_root.display()
        );
    }

    // Create AppState using the transport crate's type
    let state = AppState {
        status: Arc::new(RwLock::new(SystemStatus::default())),
        hub: Arc::new(Hub::new()),
        project_root,
    };

    // Create the app with transport router and static file handler
    let app = Router::new()
        .route("/health", get(|| async { "OK" }))
        // Nest all API routes from ckrv-transport
        .nest("/api", ckrv_transport::axum::create_router(state.clone()))
        // Fallback for SPA (serves static files)
        .fallback(static_handler)
        .with_state(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    println!("UI Server listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
