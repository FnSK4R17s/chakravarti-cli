//! Specs API endpoints

use axum::{extract::State, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use crate::state::AppState;

#[derive(Serialize)]
pub struct Spec {
    pub name: String,
    pub path: String,
    pub has_tasks: bool,
    pub has_implementation: bool,
    pub implementation_branch: Option<String>,
}

#[derive(Serialize)]
pub struct SpecsResponse {
    pub specs: Vec<Spec>,
    pub count: usize,
}

/// Implementation summary structure (matches run.rs)
#[derive(Deserialize)]
struct ImplementationSummary {
    status: String,
    branch: String,
}

/// GET /api/specs - List all specifications
pub async fn list_specs(State(_state): State<AppState>) -> impl IntoResponse {
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let specs_dir = cwd.join(".specs");

    let mut specs = Vec::new();

    if specs_dir.exists() && specs_dir.is_dir() {
        if let Ok(entries) = std::fs::read_dir(&specs_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    let name = path.file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("")
                        .to_string();
                    
                    // Check if spec.yaml exists
                    let spec_yaml = path.join("spec.yaml");
                    if spec_yaml.exists() {
                        // Check if tasks.yaml exists
                        let tasks_yaml = path.join("tasks.yaml");
                        let has_tasks = tasks_yaml.exists();

                        // Check if implementation.yaml exists (run completed)
                        let impl_yaml = path.join("implementation.yaml");
                        let (has_implementation, implementation_branch) = if impl_yaml.exists() {
                            // Try to read the branch from implementation.yaml
                            if let Ok(content) = std::fs::read_to_string(&impl_yaml) {
                                if let Ok(summary) = serde_yaml::from_str::<ImplementationSummary>(&content) {
                                    if summary.status == "completed" {
                                        (true, Some(summary.branch))
                                    } else {
                                        (false, None)
                                    }
                                } else {
                                    (false, None)
                                }
                            } else {
                                (false, None)
                            }
                        } else {
                            (false, None)
                        };

                        specs.push(Spec {
                            name,
                            path: path.to_string_lossy().to_string(),
                            has_tasks,
                            has_implementation,
                            implementation_branch,
                        });
                    }
                }
            }
        }
    }

    // Sort by name (which includes the spec number prefix)
    specs.sort_by(|a, b| a.name.cmp(&b.name));

    let count = specs.len();
    Json(SpecsResponse { specs, count })
}

