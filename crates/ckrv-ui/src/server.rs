use axum::{
    routing::get,
    Router,
    response::{IntoResponse, Response},
    extract::State,
    http::{StatusCode, header, Uri},
    body::Body,
};
use std::net::SocketAddr;
use rust_embed::RustEmbed;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::state::{AppState, SystemStatus};
use crate::hub::Hub;

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
            ).into_response();
        }
    }

    // 2. Fallback to index.html (for root / or SPA routing)
    if let Some(content) = FrontendAssets::get("index.html") {
        return (
            [(header::CONTENT_TYPE, "text/html")],
            Body::from(content.data),
        ).into_response();
    }

    (StatusCode::NOT_FOUND, "404 Not Found").into_response()
}

pub async fn start_server(port: u16) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let project_root = std::env::current_dir().unwrap_or_default();
    
    let state = AppState {
        status: Arc::new(RwLock::new(SystemStatus::default())),
        hub: Arc::new(Hub::new()),
        project_root,
    };

    let app = Router::new()
        .route("/health", get(|| async { "OK" }))
        .route("/api/status", get(crate::api::status::get_status))
        .route("/api/docker", get(crate::api::docker::get_docker_status))
        .route("/api/events", get(crate::api::events::sse_handler))
        .route("/api/specs", get(crate::api::specs::list_specs))
        .route("/api/tasks", get(crate::api::tasks::list_tasks))
        .route("/api/command/init", axum::routing::post(crate::api::commands::run_init))
        .route("/api/command/git-init", axum::routing::post(crate::api::commands::run_git_init))
        .route("/api/command/spec-new", axum::routing::post(crate::api::commands::run_spec_new))
        .route("/api/command/spec-tasks", axum::routing::post(crate::api::commands::run_spec_tasks))
        .route("/api/command/run", axum::routing::post(crate::api::commands::run_run))
        .route("/api/command/diff", axum::routing::post(crate::api::commands::run_diff))
        .route("/api/command/verify", axum::routing::post(crate::api::commands::run_verify))
        .route("/api/command/promote", axum::routing::post(crate::api::commands::run_promote))
        .route("/api/command/fix", axum::routing::post(crate::api::commands::run_fix))
        // Agent management routes
        .route("/api/agents", get(crate::api::agents::list_agents))
        .route("/api/agents/models", get(crate::api::agents::get_openrouter_models))
        .route("/api/agents/upsert", axum::routing::post(crate::api::agents::upsert_agent))
        .route("/api/agents/delete", axum::routing::post(crate::api::agents::delete_agent))
        .route("/api/agents/set-default", axum::routing::post(crate::api::agents::set_default_agent))
        .route("/api/agents/test", axum::routing::post(crate::api::agents::test_agent))
        // Fallback for SPA
        .fallback(static_handler)
        .with_state(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    println!("UI Server listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
