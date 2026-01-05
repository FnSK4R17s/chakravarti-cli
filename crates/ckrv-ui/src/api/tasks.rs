//! Tasks API endpoints

use axum::{extract::State, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::process::Command;
use crate::state::AppState;

/// Full task structure matching tasks.yaml
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Task {
    pub id: String,
    #[serde(default)]
    pub phase: String,
    pub title: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub file: String,
    #[serde(default)]
    pub user_story: Option<String>,
    #[serde(default)]
    pub parallel: bool,
    #[serde(default = "default_complexity")]
    pub complexity: i32,
    #[serde(default = "default_model_tier")]
    pub model_tier: String,
    #[serde(default)]
    pub estimated_tokens: i32,
    #[serde(default = "default_risk")]
    pub risk: String,
    #[serde(default)]
    pub context_required: Vec<String>,
    #[serde(default = "default_status")]
    pub status: String,
}

fn default_complexity() -> i32 { 1 }
fn default_model_tier() -> String { "light".to_string() }
fn default_risk() -> String { "low".to_string() }
fn default_status() -> String { "pending".to_string() }

#[derive(Serialize)]
pub struct TasksResponse {
    pub tasks: Vec<Task>,
    pub spec_id: String,
    pub count: usize,
}

#[derive(Deserialize)]
struct TasksYaml {
    tasks: Option<Vec<Task>>,
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
            count: 0,
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
            count: 0,
        });
    }

    // Parse tasks.yaml
    let tasks = parse_tasks_file(&tasks_path);
    let count = tasks.len();

    Json(TasksResponse {
        tasks,
        spec_id: branch,
        count,
    })
}

/// Query params for getting tasks by spec name
#[derive(Deserialize)]
pub struct GetTasksQuery {
    pub spec: String,
}

#[derive(Serialize)]
pub struct TasksDetailResponse {
    pub success: bool,
    pub tasks: Vec<Task>,
    pub raw_yaml: Option<String>,
    pub count: usize,
    pub error: Option<String>,
}

/// GET /api/tasks/detail?spec=001-make-todo-list - Get tasks for a specific spec
pub async fn get_tasks(
    axum::extract::Query(query): axum::extract::Query<GetTasksQuery>,
) -> impl IntoResponse {
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let tasks_path = cwd.join(".specs").join(&query.spec).join("tasks.yaml");

    if !tasks_path.exists() {
        return Json(TasksDetailResponse {
            success: false,
            tasks: vec![],
            raw_yaml: None,
            count: 0,
            error: Some(format!("Tasks file not found for spec '{}'", query.spec)),
        });
    }

    match std::fs::read_to_string(&tasks_path) {
        Ok(content) => {
            let tasks = parse_tasks_yaml(&content);
            let count = tasks.len();
            Json(TasksDetailResponse {
                success: true,
                tasks,
                raw_yaml: Some(content),
                count,
                error: None,
            })
        }
        Err(e) => Json(TasksDetailResponse {
            success: false,
            tasks: vec![],
            raw_yaml: None,
            count: 0,
            error: Some(format!("Failed to read tasks: {}", e)),
        }),
    }
}

/// Request to save tasks
#[derive(Deserialize)]
pub struct SaveTasksPayload {
    pub spec: String,
    pub tasks: Vec<Task>,
}

#[derive(Serialize)]
pub struct SaveTasksResponse {
    pub success: bool,
    pub message: Option<String>,
}

/// POST /api/tasks/save - Save tasks to a specific spec
pub async fn save_tasks(
    Json(payload): Json<SaveTasksPayload>,
) -> impl IntoResponse {
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let tasks_path = cwd.join(".specs").join(&payload.spec).join("tasks.yaml");

    #[derive(Serialize)]
    struct TasksYamlOutput {
        tasks: Vec<Task>,
    }

    let output = TasksYamlOutput { tasks: payload.tasks };

    match serde_yaml::to_string(&output) {
        Ok(yaml) => {
            match std::fs::write(&tasks_path, yaml) {
                Ok(_) => Json(SaveTasksResponse {
                    success: true,
                    message: Some("Tasks saved successfully".to_string()),
                }),
                Err(e) => Json(SaveTasksResponse {
                    success: false,
                    message: Some(format!("Failed to write tasks: {}", e)),
                }),
            }
        }
        Err(e) => Json(SaveTasksResponse {
            success: false,
            message: Some(format!("Failed to serialize tasks: {}", e)),
        }),
    }
}

/// Request to update a single task's status
#[derive(Deserialize)]
pub struct UpdateTaskStatusPayload {
    pub spec: String,
    pub task_id: String,
    pub status: String,
}

/// POST /api/tasks/status - Update a single task's status
pub async fn update_task_status(
    Json(payload): Json<UpdateTaskStatusPayload>,
) -> impl IntoResponse {
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let tasks_path = cwd.join(".specs").join(&payload.spec).join("tasks.yaml");

    if !tasks_path.exists() {
        return Json(SaveTasksResponse {
            success: false,
            message: Some("Tasks file not found".to_string()),
        });
    }

    let content = match std::fs::read_to_string(&tasks_path) {
        Ok(c) => c,
        Err(e) => return Json(SaveTasksResponse {
            success: false,
            message: Some(format!("Failed to read tasks: {}", e)),
        }),
    };

    let mut tasks = parse_tasks_yaml(&content);
    
    // Find and update the task
    let mut found = false;
    for task in &mut tasks {
        if task.id == payload.task_id {
            task.status = payload.status.clone();
            found = true;
            break;
        }
    }

    if !found {
        return Json(SaveTasksResponse {
            success: false,
            message: Some(format!("Task '{}' not found", payload.task_id)),
        });
    }

    // Write back
    #[derive(Serialize)]
    struct TasksYamlOutput {
        tasks: Vec<Task>,
    }

    let output = TasksYamlOutput { tasks };
    match serde_yaml::to_string(&output) {
        Ok(yaml) => {
            match std::fs::write(&tasks_path, yaml) {
                Ok(_) => Json(SaveTasksResponse {
                    success: true,
                    message: Some("Task status updated".to_string()),
                }),
                Err(e) => Json(SaveTasksResponse {
                    success: false,
                    message: Some(format!("Failed to write: {}", e)),
                }),
            }
        }
        Err(e) => Json(SaveTasksResponse {
            success: false,
            message: Some(format!("Failed to serialize: {}", e)),
        }),
    }
}

fn parse_tasks_file(path: &PathBuf) -> Vec<Task> {
    std::fs::read_to_string(path)
        .ok()
        .map(|c| parse_tasks_yaml(&c))
        .unwrap_or_default()
}

fn parse_tasks_yaml(content: &str) -> Vec<Task> {
    serde_yaml::from_str::<TasksYaml>(content)
        .ok()
        .and_then(|y| y.tasks)
        .unwrap_or_default()
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
