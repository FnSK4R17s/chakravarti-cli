//! # Application State
//!
//! Shared state types for transport handlers.
//!
//! ## Overview
//!
//! This module defines the core state structures shared across all request
//! handlers. The state is thread-safe and can be accessed concurrently.
//!
//! ## Key Types
//!
//! - [`AppState`] - Root state passed to all handlers
//! - [`SystemStatus`] - Current orchestration status
//! - [`SystemMode`] - Active mode (idle, planning, running, etc.)
//!
//! ## Thread Safety
//!
//! `AppState` is `Clone` and internally uses `Arc<RwLock>` for mutable
//! state, making it safe to share across async tasks.

use crate::hub::SharedHub;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

#[cfg(feature = "typescript")]
use ts_rs::TS;

// ============================================================
// System Status
// ============================================================

/// Current system status displayed in the UI header.
///
/// This represents the overall state of the Chakravarti orchestration system.
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "typescript", derive(TS))]
#[cfg_attr(feature = "typescript", ts(export))]
pub struct SystemStatus {
    /// Current Git branch name
    pub active_branch: String,

    /// Feature number if on a feature branch (e.g., "001" from "001-feature-name")
    pub feature_number: Option<String>,

    /// Whether the project is initialized (.specs/ exists)
    pub is_ready: bool,

    /// Current system mode
    pub mode: SystemMode,

    /// Project root directory path (for display in Settings)
    pub project_root: String,
}

/// Operating mode of the orchestration system.
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "typescript", derive(TS))]
#[cfg_attr(feature = "typescript", ts(export))]
#[serde(rename_all = "lowercase")]
pub enum SystemMode {
    /// No active operation
    #[default]
    Idle,
    /// Generating implementation plan
    Planning,
    /// Executing tasks
    Running,
    /// Promoting changes to main branch
    Promoting,
}

// ============================================================
// Application State
// ============================================================

/// Shared application state for all handlers.
///
/// This is the single source of truth for runtime state.
/// Handlers receive a reference and should never clone the Arc.
///
/// ## Example
///
/// ```rust,ignore
/// use ckrv_transport::{AppState, SystemStatus, Hub};
/// use std::sync::Arc;
/// use std::path::PathBuf;
/// use tokio::sync::RwLock;
///
/// let state = AppState {
///     status: Arc::new(RwLock::new(SystemStatus::default())),
///     hub: Arc::new(Hub::new()),
///     project_root: PathBuf::from("/path/to/project"),
/// };
/// ```
#[derive(Clone)]
pub struct AppState {
    /// Current system status (git branch, initialization state, etc.)
    pub status: Arc<RwLock<SystemStatus>>,

    /// Event hub for broadcasting real-time updates
    pub hub: SharedHub,

    /// Root directory of the current project
    pub project_root: PathBuf,
}

impl AppState {
    /// Create a new `AppState` with the given project root.
    ///
    /// Initializes with default status and a new event hub.
    #[must_use]
    pub fn new(project_root: PathBuf) -> Self {
        use crate::hub::Hub;

        Self {
            status: Arc::new(RwLock::new(SystemStatus::default())),
            hub: Arc::new(Hub::new()),
            project_root,
        }
    }

    /// Create a new `AppState` with custom components.
    #[must_use]
    pub fn with_components(
        status: Arc<RwLock<SystemStatus>>,
        hub: SharedHub,
        project_root: PathBuf,
    ) -> Self {
        Self {
            status,
            hub,
            project_root,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_system_status_default() {
        let status = SystemStatus::default();
        assert_eq!(status.active_branch, "");
        assert!(!status.is_ready);
        assert!(matches!(status.mode, SystemMode::Idle));
    }

    #[test]
    fn test_app_state_new() {
        let state = AppState::new(PathBuf::from("/tmp/test"));
        assert_eq!(state.project_root, PathBuf::from("/tmp/test"));
    }

    #[test]
    fn test_system_mode_serialization() {
        let mode = SystemMode::Running;
        let json = serde_json::to_string(&mode).expect("serialization failed");
        assert_eq!(json, "\"running\"");
    }
}
