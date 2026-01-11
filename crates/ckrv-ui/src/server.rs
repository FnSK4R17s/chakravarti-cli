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
    // T006: Support CKRV_PROJECT_ROOT env var for test isolation (TR-007)
    // E2E tests set this to a temporary directory to prevent modifying working code
    let project_root = std::env::var("CKRV_PROJECT_ROOT")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| std::env::current_dir().unwrap_or_default());
    
    if std::env::var("CKRV_PROJECT_ROOT").is_ok() {
        println!("Using custom project root from CKRV_PROJECT_ROOT: {:?}", project_root);
    }
    
    let state = AppState {
        status: Arc::new(RwLock::new(SystemStatus::default())),
        hub: Arc::new(Hub::new()),
        project_root,
    };

    let app = Router::new()
        .route("/health", get(|| async { "OK" }))
        .route("/api/status", get(crate::api::status::get_status))
        .route("/api/docker", get(crate::api::docker::get_docker_status))
        .route("/api/cloud", get(crate::api::cloud::get_cloud_status))
        .route("/api/events", get(crate::api::events::sse_handler))
        .route("/api/specs", get(crate::api::specs::list_specs))
        .route("/api/specs/detail", get(crate::api::specs::get_spec))
        .route("/api/specs/save", axum::routing::post(crate::api::specs::save_spec))
        .route("/api/tasks", get(crate::api::tasks::list_tasks))
        .route("/api/tasks/detail", get(crate::api::tasks::get_tasks))
        .route("/api/tasks/save", axum::routing::post(crate::api::tasks::save_tasks))
        .route("/api/tasks/status", axum::routing::post(crate::api::tasks::update_task_status))
        // Plan management routes
        .route("/api/plans/detail", get(crate::api::plans::get_plan))
        .route("/api/plans/save", axum::routing::post(crate::api::plans::save_plan))
        .route("/api/plans/models", get(crate::api::plans::get_openrouter_models))
        .route("/api/command/init", axum::routing::post(crate::api::commands::run_init))
        .route("/api/command/git-init", axum::routing::post(crate::api::commands::run_git_init))
        .route("/api/command/spec-new", axum::routing::post(crate::api::commands::run_spec_new))
        .route("/api/command/spec-tasks", axum::routing::post(crate::api::commands::run_spec_tasks))
        .route("/api/command/plan", axum::routing::post(crate::api::commands::run_plan))
        .route("/api/command/run", axum::routing::post(crate::api::commands::run_run))
        .route("/api/command/execute", axum::routing::post(crate::api::commands::run_execute))
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
        .route("/api/agents/cli", axum::routing::post(crate::api::console::execute_command))
        // Session management routes
        .route("/api/session/start", axum::routing::post(crate::api::session::start_session))
        .route("/api/session/exec", axum::routing::post(crate::api::session::exec_in_session))
        .route("/api/session/stop", axum::routing::post(crate::api::session::stop_session))
        // Interactive terminal routes (WebSocket)
        .route("/api/terminal/start", axum::routing::post(crate::api::terminal::start_terminal_session))
        .route("/api/terminal/ws", axum::routing::get(crate::api::terminal::terminal_ws))
        .route("/api/terminal/stop", axum::routing::post(crate::api::terminal::stop_terminal_session))
        // Execution streaming routes (WebSocket)
        .route("/api/execution/start", axum::routing::post(crate::api::execution::start_execution))
        .route("/api/execution/ws", axum::routing::get(crate::api::execution::execution_ws))
        .route("/api/execution/stop", axum::routing::post(crate::api::execution::stop_execution))
        .route("/api/execution/branches", axum::routing::post(crate::api::execution::list_unmerged_branches))
        .route("/api/execution/merge", axum::routing::post(crate::api::execution::merge_branch))
        .route("/api/execution/merge-all", axum::routing::post(crate::api::execution::merge_all_branches))
        // Run history routes
        .route("/api/history/{spec}", axum::routing::get(crate::api::history::list_runs))
        .route("/api/history/{spec}", axum::routing::post(crate::api::history::create_run))
        .route("/api/history/{spec}/{run_id}", axum::routing::get(crate::api::history::get_run))
        .route("/api/history/{spec}/{run_id}", axum::routing::patch(crate::api::history::update_run))
        .route("/api/history/{spec}/{run_id}", axum::routing::delete(crate::api::history::delete_run))
        // Git diff routes
        .route("/api/diff/branches", axum::routing::get(crate::api::diff::get_branches))
        .route("/api/diff", axum::routing::get(crate::api::diff::get_diff))
        // Fallback for SPA
        .fallback(static_handler)
        .with_state(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    println!("UI Server listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
