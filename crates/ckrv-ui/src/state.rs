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

use std::sync::Arc;
use std::path::PathBuf;

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
