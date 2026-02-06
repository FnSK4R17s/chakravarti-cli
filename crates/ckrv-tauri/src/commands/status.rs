//! Status commands for Tauri IPC

use crate::SharedState;
use ckrv_transport::handlers::{
    cloud::get_cloud_status_handler, docker::check_docker_handler, status::get_status_handler,
};
use ckrv_transport::handlers::cloud::CloudStatus;
use ckrv_transport::types::DockerStatus;
use ckrv_transport::SystemStatus;
use tauri::State;

/// Get current system status.
///
/// Returns git branch, initialization state, and other status info.
#[tauri::command]
pub async fn get_status(state: State<'_, SharedState>) -> Result<SystemStatus, String> {
    let app_state = state.read().await;
    get_status_handler(&app_state)
        .await
        .map_err(|e| e.to_string())
}

/// Check Docker daemon status.
///
/// Returns whether Docker is available and running.
#[tauri::command]
pub async fn check_docker() -> Result<DockerStatus, String> {
    check_docker_handler().await.map_err(|e| e.to_string())
}

/// Get cloud service status.
///
/// Returns cloud authentication state.
#[tauri::command]
pub async fn get_cloud_status() -> Result<CloudStatus, String> {
    get_cloud_status_handler().await.map_err(|e| e.to_string())
}

