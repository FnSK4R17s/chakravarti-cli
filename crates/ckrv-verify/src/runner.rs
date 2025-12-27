//! Test runner.

use std::path::Path;
use std::process::Command;
use std::time::Instant;

use ckrv_core::Spec;

use crate::{TestResult, TestStatus, Verdict, VerifyError};

/// Configuration for verification.
#[derive(Debug, Clone)]
pub struct VerifyConfig {
    /// Worktree path.
    pub worktree_path: String,
    /// Spec to verify against.
    pub spec: Spec,
    /// Test commands to run.
    pub test_commands: Vec<String>,
    /// Timeout in seconds.
    pub timeout_secs: u64,
}

impl VerifyConfig {
    /// Create a new verify config.
    #[must_use]
    pub fn new(worktree_path: impl Into<String>, spec: Spec) -> Self {
        Self {
            worktree_path: worktree_path.into(),
            spec,
            test_commands: vec!["cargo test".to_string()],
            timeout_secs: 300,
        }
    }

    /// Add a test command.
    #[must_use]
    pub fn with_command(mut self, cmd: impl Into<String>) -> Self {
        self.test_commands.push(cmd.into());
        self
    }

    /// Set timeout.
    #[must_use]
    pub fn with_timeout(mut self, secs: u64) -> Self {
        self.timeout_secs = secs;
        self
    }
}

/// Trait for verification.
pub trait Verifier: Send + Sync {
    /// Run verification on a worktree.
    ///
    /// # Errors
    ///
    /// Returns an error if verification fails to run.
    fn verify(&self, config: &VerifyConfig) -> Result<Verdict, VerifyError>;
}

/// Default verifier that runs tests via shell commands.
pub struct DefaultVerifier;

impl DefaultVerifier {
    /// Create a new verifier.
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    /// Run a single command and parse results.
    fn run_command(&self, cmd: &str, cwd: &Path) -> Result<(bool, String, u64), VerifyError> {
        let start = Instant::now();

        let output = Command::new("sh")
            .args(["-c", cmd])
            .current_dir(cwd)
            .output()
            .map_err(|e| VerifyError::ExecutionFailed(e.to_string()))?;

        let duration = start.elapsed().as_millis() as u64;
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let combined = format!("{stdout}\n{stderr}");

        Ok((output.status.success(), combined, duration))
    }
}

impl Default for DefaultVerifier {
    fn default() -> Self {
        Self::new()
    }
}

impl Verifier for DefaultVerifier {
    fn verify(&self, config: &VerifyConfig) -> Result<Verdict, VerifyError> {
        let cwd = Path::new(&config.worktree_path);
        if !cwd.exists() {
            return Err(VerifyError::ExecutionFailed(format!(
                "Worktree path does not exist: {}",
                config.worktree_path
            )));
        }

        let start = Instant::now();
        let mut test_results = Vec::new();
        let mut logs = Vec::new();
        let mut all_passed = true;

        for cmd in &config.test_commands {
            let (success, output, duration) = self.run_command(cmd, cwd)?;

            let result = if success {
                TestResult {
                    name: cmd.clone(),
                    status: TestStatus::Passed,
                    duration_ms: duration,
                    output: Some(output.clone()),
                }
            } else {
                all_passed = false;
                logs.push(format!("Command failed: {cmd}\n{output}"));
                TestResult {
                    name: cmd.clone(),
                    status: TestStatus::Failed,
                    duration_ms: duration,
                    output: Some(output),
                }
            };

            test_results.push(result);
        }

        let total_duration = start.elapsed().as_millis() as u64;

        Ok(Verdict {
            passed: all_passed,
            test_results,
            logs,
            artifacts: Vec::new(),
            duration_ms: total_duration,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_spec() -> Spec {
        Spec {
            id: "test".to_string(),
            goal: "Test goal".to_string(),
            constraints: vec![],
            acceptance: vec!["Tests pass".to_string()],
            verify: None,
            source_path: None,
        }
    }

    #[test]
    fn test_verify_config_new() {
        let spec = create_test_spec();
        let config = VerifyConfig::new("/tmp/test", spec);

        assert_eq!(config.worktree_path, "/tmp/test");
        assert_eq!(config.timeout_secs, 300);
    }

    #[test]
    fn test_verify_config_with_command() {
        let spec = create_test_spec();
        let config = VerifyConfig::new("/tmp", spec)
            .with_command("npm test")
            .with_timeout(60);

        assert!(config.test_commands.contains(&"npm test".to_string()));
        assert_eq!(config.timeout_secs, 60);
    }

    #[test]
    fn test_default_verifier_with_simple_command() {
        let dir = TempDir::new().expect("temp dir");
        let spec = create_test_spec();
        let config = VerifyConfig {
            worktree_path: dir.path().to_string_lossy().to_string(),
            spec,
            test_commands: vec!["echo hello".to_string()],
            timeout_secs: 10,
        };

        let verifier = DefaultVerifier::new();
        let verdict = verifier.verify(&config).expect("verify");

        assert!(verdict.passed);
        assert_eq!(verdict.test_results.len(), 1);
    }

    #[test]
    fn test_default_verifier_with_failing_command() {
        let dir = TempDir::new().expect("temp dir");
        let spec = create_test_spec();
        let config = VerifyConfig {
            worktree_path: dir.path().to_string_lossy().to_string(),
            spec,
            test_commands: vec!["exit 1".to_string()],
            timeout_secs: 10,
        };

        let verifier = DefaultVerifier::new();
        let verdict = verifier.verify(&config).expect("verify");

        assert!(!verdict.passed);
        assert_eq!(verdict.failed_count(), 1);
    }

    #[test]
    fn test_verifier_nonexistent_path() {
        let spec = create_test_spec();
        let config = VerifyConfig::new("/nonexistent/path", spec);

        let verifier = DefaultVerifier::new();
        let result = verifier.verify(&config);

        assert!(result.is_err());
    }
}
