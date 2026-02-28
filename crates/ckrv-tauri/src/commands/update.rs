//! Auto-update commands for the Tauri desktop app.
//!
//! Provides IPC commands for checking and installing updates from GitHub Releases.
//! The updater plugin also runs a background check on app startup (see main.rs).

use serde::Serialize;
use tauri::AppHandle;
use tauri_plugin_process::ProcessExt;
use tauri_plugin_updater::UpdaterExt;

/// Information about an available update returned to the frontend.
#[derive(Serialize)]
pub struct UpdateInfo {
    pub version: String,
    pub body: Option<String>,
    pub date: Option<String>,
}

/// Check if a newer version is available on GitHub Releases.
///
/// Returns `Some(UpdateInfo)` if an update exists, `None` if already up-to-date.
#[tauri::command]
pub async fn check_for_updates(app: AppHandle) -> Result<Option<UpdateInfo>, String> {
    let updater = app.updater().map_err(|e| e.to_string())?;
    match updater.check().await {
        Ok(Some(update)) => Ok(Some(UpdateInfo {
            version: update.version.clone(),
            body: update.body.clone(),
            date: update.date.map(|d| d.to_string()),
        })),
        Ok(None) => Ok(None),
        Err(e) => Err(e.to_string()),
    }
}

/// Download and install the latest update, then restart the app.
///
/// This will download the update artifact, verify its signature, apply it,
/// and restart the application. The frontend should confirm with the user
/// before calling this command.
#[tauri::command]
pub async fn install_update(app: AppHandle) -> Result<(), String> {
    let updater = app.updater().map_err(|e| e.to_string())?;
    match updater.check().await {
        Ok(Some(update)) => {
            update
                .download_and_install(|_chunk_len, _content_len| {}, || {})
                .await
                .map_err(|e| e.to_string())?;
            // Restart the app to apply the update
            app.restart();
            Ok(())
        }
        Ok(None) => Err("No update available".to_string()),
        Err(e) => Err(e.to_string()),
    }
}
