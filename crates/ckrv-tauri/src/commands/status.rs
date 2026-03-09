//! Status commands for Tauri IPC

use crate::SharedState;
use ckrv_transport::handlers::cloud::CloudStatus;
use ckrv_transport::handlers::{
    cloud::get_cloud_status_handler, docker::check_docker_handler, status::get_status_handler,
};
use ckrv_transport::types::DockerStatus;
use ckrv_transport::SystemStatus;
use tauri::State;

/// Get current system status.
///
/// Returns git branch, initialization state, and other status info.
/// Uses `spawn_blocking` because the handler calls `blocking_read()`
/// and spawns synchronous git subprocesses.
#[tauri::command]
pub async fn get_status(state: State<'_, SharedState>) -> Result<SystemStatus, String> {
    let app_state = state.read().await.clone();
    tokio::task::spawn_blocking(move || get_status_handler(&app_state).map_err(|e| e.to_string()))
        .await
        .map_err(|e| format!("Task panicked: {e}"))?
}

/// Check Docker daemon status.
///
/// Returns whether Docker is available and running.
/// Uses `spawn_blocking` because the handler spawns synchronous subprocesses.
#[tauri::command]
pub async fn check_docker() -> Result<DockerStatus, String> {
    tokio::task::spawn_blocking(|| check_docker_handler().map_err(|e| e.to_string()))
        .await
        .map_err(|e| format!("Task panicked: {e}"))?
}

/// Get cloud service status.
///
/// Returns cloud authentication state.
/// Uses `spawn_blocking` because the handler spawns synchronous subprocesses.
#[tauri::command]
pub async fn get_cloud_status() -> Result<CloudStatus, String> {
    tokio::task::spawn_blocking(|| get_cloud_status_handler().map_err(|e| e.to_string()))
        .await
        .map_err(|e| format!("Task panicked: {e}"))?
}
