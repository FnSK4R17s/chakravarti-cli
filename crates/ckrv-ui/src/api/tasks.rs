//! Tasks API endpoints

use axum::{extract::State, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::process::Command;
use crate::state::AppState;

#[derive(Serialize, Deserialize, Clone)]
pub struct Task {
    pub id: String,
    pub phase: String,
    pub title: String,
    pub description: Option<String>,
    pub file: Option<String>,
    pub status: String,
}

#[derive(Serialize)]
pub struct TasksResponse {
    pub tasks: Vec<Task>,
    pub spec_id: String,
}

#[derive(Deserialize)]
struct TasksYaml {
    tasks: Option<Vec<TaskEntry>>,
}

#[derive(Deserialize)]
struct TaskEntry {
    id: String,
    phase: Option<String>,
    title: String,
    description: Option<String>,
    file: Option<String>,
    status: Option<String>,
}

/// GET /api/tasks - Get tasks for current spec (based on current branch)
pub async fn list_tasks(State(_state): State<AppState>) -> impl IntoResponse {
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    
    // Get current git branch to find the spec
    let branch = get_current_branch(&cwd);
    
    if branch.is_empty() || branch == "main" || branch == "master" {
        return Json(TasksResponse {
            tasks: vec![],
            spec_id: String::new(),
        });
    }

    // Look for spec folder matching the branch name
    let specs_dir = cwd.join(".specs");
    let spec_folder = specs_dir.join(&branch);
    let tasks_path = spec_folder.join("tasks.yaml");

    if !tasks_path.exists() {
        return Json(TasksResponse {
            tasks: vec![],
            spec_id: branch,
        });
    }

    // Parse tasks.yaml
    let tasks = match std::fs::read_to_string(&tasks_path) {
        Ok(content) => {
            match serde_yaml::from_str::<TasksYaml>(&content) {
                Ok(yaml) => {
                    yaml.tasks.unwrap_or_default().into_iter().map(|t| Task {
                        id: t.id,
                        phase: t.phase.unwrap_or_else(|| "Unknown".to_string()),
                        title: t.title,
                        description: t.description,
                        file: t.file,
                        status: t.status.unwrap_or_else(|| "pending".to_string()),
                    }).collect()
                }
                Err(_) => vec![],
            }
        }
        Err(_) => vec![],
    };

    Json(TasksResponse {
        tasks,
        spec_id: branch,
    })
}

fn get_current_branch(cwd: &PathBuf) -> String {
    Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .current_dir(cwd)
        .output()
        .ok()
        .and_then(|o| {
            if o.status.success() {
                String::from_utf8(o.stdout).ok().map(|s| s.trim().to_string())
            } else {
                None
            }
        })
        .unwrap_or_default()
}

