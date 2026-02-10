//! Project commands for Tauri IPC
//!
//! Provides commands for managing project selection in Tauri.
//! The project root is stored in a config file and persists across app restarts.

use crate::SharedState;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tauri::{AppHandle, State};
use tauri_plugin_dialog::DialogExt;

/// Configuration file path for Tauri settings.
fn config_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("/"))
        .join(".ckrv")
        .join("tauri-config.json")
}

/// Tauri configuration structure.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TauriConfig {
    /// Last opened project path
    pub project_root: Option<PathBuf>,
    /// Recent project paths (max 10)
    pub recent_projects: Vec<PathBuf>,
}

impl TauriConfig {
    /// Load config from disk.
    pub fn load() -> Self {
        let path = config_path();
        if path.exists() {
            match std::fs::read_to_string(&path) {
                Ok(contents) => serde_json::from_str(&contents).unwrap_or_default(),
                Err(_) => Self::default(),
            }
        } else {
            Self::default()
        }
    }

    /// Save config to disk.
    pub fn save(&self) -> Result<(), String> {
        let path = config_path();

        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }

        let contents = serde_json::to_string_pretty(self).map_err(|e| e.to_string())?;
        std::fs::write(&path, contents).map_err(|e| e.to_string())
    }

    /// Add a project to recent list (max 10, no duplicates).
    pub fn add_recent(&mut self, path: PathBuf) {
        // Remove if already exists
        self.recent_projects.retain(|p| p != &path);

        // Add to front
        self.recent_projects.insert(0, path);

        // Trim to max 10
        self.recent_projects.truncate(10);
    }
}

/// Project info for display.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectInfo {
    pub path: PathBuf,
    pub name: String,
    pub exists: bool,
}

impl From<PathBuf> for ProjectInfo {
    fn from(path: PathBuf) -> Self {
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("Unknown")
            .to_string();
        let exists = path.exists();
        Self { path, name, exists }
    }
}

/// Get the current project root from saved config.
///
/// Returns None if no project has been explicitly selected (app is using cwd fallback).
#[tauri::command]
pub async fn get_project_root(_state: State<'_, SharedState>) -> Result<Option<String>, String> {
    // Check if we have a saved config with an explicit project root
    let config = TauriConfig::load();

    // Return the saved project root if it exists and the path still exists
    match config.project_root {
        Some(path) if path.exists() => Ok(Some(path.display().to_string())),
        Some(_) => Ok(None), // Path was saved but no longer exists
        None => Ok(None),    // No project explicitly selected
    }
}

/// Set the project root (requires app restart to take effect).
///
/// Saves the project root to config and adds to recent projects.
#[tauri::command]
pub async fn set_project_root(path: String) -> Result<(), String> {
    let path = PathBuf::from(&path);

    if !path.exists() {
        return Err(format!("Path does not exist: {}", path.display()));
    }

    if !path.is_dir() {
        return Err(format!("Path is not a directory: {}", path.display()));
    }

    let mut config = TauriConfig::load();
    config.project_root = Some(path.clone());
    config.add_recent(path);
    config.save()?;

    Ok(())
}

/// Get list of recent projects.
#[tauri::command]
pub async fn get_recent_projects() -> Result<Vec<ProjectInfo>, String> {
    let config = TauriConfig::load();
    let projects: Vec<ProjectInfo> = config
        .recent_projects
        .into_iter()
        .map(ProjectInfo::from)
        .collect();
    Ok(projects)
}

/// Open a native folder picker dialog.
///
/// Returns the selected path or None if cancelled.
#[tauri::command]
pub async fn open_project_dialog(app: AppHandle) -> Result<Option<String>, String> {
    // Use blocking spawn since the dialog is blocking
    let result = tokio::task::spawn_blocking(move || {
        let file_path = app.dialog().file().blocking_pick_folder();
        file_path.map(|fp| fp.to_string())
    })
    .await
    .map_err(|e| e.to_string())?;

    Ok(result)
}
