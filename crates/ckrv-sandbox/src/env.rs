//! Environment variable injection.

use std::collections::HashMap;

/// Environment configuration for sandbox execution.
#[derive(Debug, Clone, Default)]
pub struct EnvConfig {
    /// Variables to set.
    vars: HashMap<String, String>,
    /// Variables to pass through from host.
    passthrough: Vec<String>,
}

impl EnvConfig {
    /// Create a new env config.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set an environment variable.
    #[must_use]
    pub fn set(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.vars.insert(key.into(), value.into());
        self
    }

    /// Pass through a host environment variable.
    #[must_use]
    pub fn passthrough(mut self, key: impl Into<String>) -> Self {
        self.passthrough.push(key.into());
        self
    }

    /// Build the final environment map.
    #[must_use]
    pub fn build(&self) -> HashMap<String, String> {
        let mut env = self.vars.clone();

        // Add passthrough variables from host
        for key in &self.passthrough {
            if let Ok(value) = std::env::var(key) {
                env.insert(key.clone(), value);
            }
        }

        env
    }

    /// Create default env for Rust projects.
    #[must_use]
    pub fn rust_defaults() -> Self {
        Self::new()
            .set("CARGO_HOME", "/usr/local/cargo")
            .set("RUSTUP_HOME", "/usr/local/rustup")
            .set("PATH", "/usr/local/cargo/bin:/usr/local/bin:/usr/bin:/bin")
            .set("RUST_BACKTRACE", "1")
    }

    /// Create default env for Node.js projects.
    #[must_use]
    pub fn node_defaults() -> Self {
        Self::new()
            .set("NODE_ENV", "test")
            .set("PATH", "/usr/local/bin:/usr/bin:/bin")
    }

    /// Create default env for Python projects.
    #[must_use]
    pub fn python_defaults() -> Self {
        Self::new()
            .set("PYTHONDONTWRITEBYTECODE", "1")
            .set("PYTHONUNBUFFERED", "1")
            .set("PATH", "/usr/local/bin:/usr/bin:/bin")
    }
}

/// Detect appropriate env config from project files.
#[must_use]
pub fn detect_env(project_path: &std::path::Path) -> EnvConfig {
    if project_path.join("Cargo.toml").exists() {
        EnvConfig::rust_defaults()
    } else if project_path.join("package.json").exists() {
        EnvConfig::node_defaults()
    } else if project_path.join("pyproject.toml").exists()
        || project_path.join("requirements.txt").exists()
    {
        EnvConfig::python_defaults()
    } else {
        EnvConfig::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_env_config_set() {
        let config = EnvConfig::new().set("FOO", "bar").set("BAZ", "qux");

        let env = config.build();
        assert_eq!(env.get("FOO"), Some(&"bar".to_string()));
        assert_eq!(env.get("BAZ"), Some(&"qux".to_string()));
    }

    #[test]
    fn test_env_config_passthrough() {
        std::env::set_var("TEST_VAR_123", "test_value");

        let config = EnvConfig::new().passthrough("TEST_VAR_123");

        let env = config.build();
        assert_eq!(env.get("TEST_VAR_123"), Some(&"test_value".to_string()));

        std::env::remove_var("TEST_VAR_123");
    }

    #[test]
    fn test_rust_defaults() {
        let config = EnvConfig::rust_defaults();
        let env = config.build();

        assert!(env.contains_key("CARGO_HOME"));
        assert!(env.contains_key("RUST_BACKTRACE"));
    }

    #[test]
    fn test_detect_env_rust() {
        let dir = TempDir::new().expect("temp dir");
        std::fs::write(dir.path().join("Cargo.toml"), "[package]").ok();

        let config = detect_env(dir.path());
        let env = config.build();

        assert!(env.contains_key("CARGO_HOME"));
    }

    #[test]
    fn test_detect_env_node() {
        let dir = TempDir::new().expect("temp dir");
        std::fs::write(dir.path().join("package.json"), "{}").ok();

        let config = detect_env(dir.path());
        let env = config.build();

        assert!(env.contains_key("NODE_ENV"));
    }
}
