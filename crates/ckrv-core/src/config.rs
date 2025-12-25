//! Configuration types for Chakravarti CLI.

use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::CoreError;

/// Default configuration for a Chakravarti project.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Configuration version for compatibility.
    pub version: String,

    /// Default optimization mode.
    #[serde(default)]
    pub default_optimize: String,

    /// Maximum retry attempts.
    #[serde(default = "default_max_attempts")]
    pub max_attempts: u32,

    /// Planner model override.
    #[serde(default)]
    pub planner_model: Option<String>,

    /// Executor model override.
    #[serde(default)]
    pub executor_model: Option<String>,
}

fn default_max_attempts() -> u32 {
    3
}

impl Default for Config {
    fn default() -> Self {
        Self {
            version: "1.0".to_string(),
            default_optimize: "balanced".to_string(),
            max_attempts: 3,
            planner_model: None,
            executor_model: None,
        }
    }
}

impl Config {
    /// Load configuration from a file.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be read or parsed.
    pub fn load(path: &Path) -> Result<Self, CoreError> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| CoreError::InvalidSpec(format!("Failed to read config: {e}")))?;
        serde_json::from_str(&content)
            .map_err(|e| CoreError::InvalidSpec(format!("Failed to parse config: {e}")))
    }

    /// Save configuration to a file.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be written.
    pub fn save(&self, path: &Path) -> Result<(), CoreError> {
        let content = serde_json::to_string_pretty(self)
            .map_err(|e| CoreError::InvalidSpec(format!("Failed to serialize config: {e}")))?;
        std::fs::write(path, content)
            .map_err(|e| CoreError::InvalidSpec(format!("Failed to write config: {e}")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.version, "1.0");
        assert_eq!(config.default_optimize, "balanced");
        assert_eq!(config.max_attempts, 3);
    }

    #[test]
    fn test_config_roundtrip() {
        let dir = TempDir::new().expect("temp dir");
        let path = dir.path().join("config.json");

        let config = Config::default();
        config.save(&path).expect("save");

        let loaded = Config::load(&path).expect("load");
        assert_eq!(config.version, loaded.version);
        assert_eq!(config.max_attempts, loaded.max_attempts);
    }

    #[test]
    fn test_config_serialization() {
        let config = Config::default();
        let json = serde_json::to_string(&config).expect("serialize");
        assert!(json.contains("version"));
        assert!(json.contains("balanced"));
    }
}
