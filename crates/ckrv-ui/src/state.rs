//! # Application State
//!
//! Shared state types for the ckrv-ui web server.
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

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SystemStatus {
    pub active_branch: String,
    pub feature_number: Option<String>,
    pub is_ready: bool,
    pub mode: SystemMode,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SystemMode {
    Idle,
    Planning,
    Running,
    Promoting,
}

use std::path::PathBuf;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub status: Arc<tokio::sync::RwLock<SystemStatus>>,
    pub hub: crate::hub::SharedHub,
    pub project_root: PathBuf,
}

impl Default for SystemStatus {
    fn default() -> Self {
        Self {
            active_branch: "main".to_string(),
            feature_number: None,
            is_ready: false,
            mode: SystemMode::Idle,
        }
    }
}
