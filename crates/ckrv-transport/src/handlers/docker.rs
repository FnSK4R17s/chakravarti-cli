//! # Docker Handler
//!
//! Handler for Docker status checking.

use crate::error::TransportError;
use crate::types::DockerStatus;
use std::process::Command;

/// Check Docker daemon status.
///
/// Returns whether Docker is available and running.
pub fn check_docker_handler() -> Result<DockerStatus, TransportError> {
    Ok(check_docker())
}

/// Internal function to check Docker status using CLI.
fn check_docker() -> DockerStatus {
    // Try to run `docker info` to check if Docker is available
    match Command::new("docker").arg("info").output() {
        Ok(output) => {
            if output.status.success() {
                // Get version for additional info
                let version = get_docker_version();
                DockerStatus {
                    available: true,
                    version,
                    error: None,
                }
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                if stderr.contains("Cannot connect")
                    || stderr.contains("Is the docker daemon running")
                {
                    DockerStatus {
                        available: false,
                        version: None,
                        error: Some("Docker daemon not running".to_string()),
                    }
                } else {
                    DockerStatus {
                        available: false,
                        version: None,
                        error: Some("Docker not available".to_string()),
                    }
                }
            }
        }
        Err(_) => DockerStatus {
            available: false,
            version: None,
            error: Some("Docker not installed".to_string()),
        },
    }
}

/// Get Docker version string.
fn get_docker_version() -> Option<String> {
    let output = Command::new("docker")
        .args(["version", "--format", "{{.Server.Version}}"])
        .output()
        .ok()?;

    if output.status.success() {
        let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !version.is_empty() {
            return Some(version);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_check_docker_handler() {
        // This test just verifies the handler doesn't panic
        // Actual Docker availability depends on the environment
        let result = check_docker_handler();
        assert!(result.is_ok());
    }

    #[test]
    fn test_check_docker() {
        // Verify the sync function doesn't panic
        let _status = check_docker();
    }
}
