//! Docker status API endpoint

use axum::{response::IntoResponse, Json};
use serde::Serialize;
use std::process::Command;

#[derive(Serialize)]
pub struct DockerStatus {
    pub available: bool,
    pub message: String,
}

/// GET /api/docker - Check if Docker is available
pub async fn get_docker_status() -> impl IntoResponse {
    let status = check_docker();
    Json(status)
}

fn check_docker() -> DockerStatus {
    // Try to run `docker info` to check if Docker is available
    match Command::new("docker").arg("info").output() {
        Ok(output) => {
            if output.status.success() {
                DockerStatus {
                    available: true,
                    message: "Docker is running".to_string(),
                }
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                if stderr.contains("Cannot connect") || stderr.contains("Is the docker daemon running") {
                    DockerStatus {
                        available: false,
                        message: "Docker daemon not running".to_string(),
                    }
                } else {
                    DockerStatus {
                        available: false,
                        message: "Docker not available".to_string(),
                    }
                }
            }
        }
        Err(_) => {
            DockerStatus {
                available: false,
                message: "Docker not installed".to_string(),
            }
        }
    }
}

