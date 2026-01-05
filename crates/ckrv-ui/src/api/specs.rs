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
    pub has_plan: bool,
    pub task_count: usize,
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
                        let mut task_count = 0;
                        if has_tasks {
                            if let Ok(content) = std::fs::read_to_string(&tasks_yaml) {
                                task_count = content.lines()
                                    .filter(|l| l.trim().starts_with("- id:"))
                                    .count();
                            }
                        }

                        // Check if plan.yaml exists
                        let plan_yaml = path.join("plan.yaml");
                        let has_plan = plan_yaml.exists();

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
                            has_plan,
                            task_count,
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

/// Detailed spec content
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UserStoryAcceptance {
    pub given: String,
    pub when: String,
    pub then: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UserStory {
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub priority: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub acceptance: Vec<UserStoryAcceptance>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Requirement {
    pub id: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SuccessCriterion {
    pub id: String,
    pub metric: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SpecDetail {
    pub id: String,
    #[serde(default)]
    pub goal: String,
    #[serde(default)]
    pub constraints: Vec<String>,
    #[serde(default)]
    pub acceptance: Vec<String>,
    #[serde(default)]
    pub user_stories: Vec<UserStory>,
    #[serde(default)]
    pub requirements: Vec<Requirement>,
    #[serde(default)]
    pub success_criteria: Vec<SuccessCriterion>,
    #[serde(default)]
    pub assumptions: Vec<String>,
}

#[derive(Serialize)]
pub struct SpecDetailResponse {
    pub success: bool,
    pub spec: Option<SpecDetail>,
    pub raw_yaml: Option<String>,
    pub error: Option<String>,
}

/// Query params for get_spec
#[derive(Deserialize)]
pub struct GetSpecQuery {
    pub name: String,
}

/// GET /api/specs/detail?name=001-make-todo-list - Get full spec content
pub async fn get_spec(
    axum::extract::Query(query): axum::extract::Query<GetSpecQuery>,
) -> impl IntoResponse {
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let spec_path = cwd.join(".specs").join(&query.name).join("spec.yaml");

    if !spec_path.exists() {
        return Json(SpecDetailResponse {
            success: false,
            spec: None,
            raw_yaml: None,
            error: Some(format!("Spec '{}' not found", query.name)),
        });
    }

    match std::fs::read_to_string(&spec_path) {
        Ok(content) => {
            match serde_yaml::from_str::<SpecDetail>(&content) {
                Ok(spec) => Json(SpecDetailResponse {
                    success: true,
                    spec: Some(spec),
                    raw_yaml: Some(content),
                    error: None,
                }),
                Err(e) => Json(SpecDetailResponse {
                    success: false,
                    spec: None,
                    raw_yaml: Some(content),
                    error: Some(format!("Failed to parse spec: {}", e)),
                }),
            }
        }
        Err(e) => Json(SpecDetailResponse {
            success: false,
            spec: None,
            raw_yaml: None,
            error: Some(format!("Failed to read spec: {}", e)),
        }),
    }
}

/// POST /api/specs/save - Save spec content
#[derive(Deserialize)]
pub struct SaveSpecPayload {
    pub name: String,
    pub spec: SpecDetail,
}

#[derive(Serialize)]
pub struct SaveSpecResponse {
    pub success: bool,
    pub message: Option<String>,
}

pub async fn save_spec(
    Json(payload): Json<SaveSpecPayload>,
) -> impl IntoResponse {
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let spec_path = cwd.join(".specs").join(&payload.name).join("spec.yaml");

    match serde_yaml::to_string(&payload.spec) {
        Ok(yaml) => {
            match std::fs::write(&spec_path, yaml) {
                Ok(_) => Json(SaveSpecResponse {
                    success: true,
                    message: Some("Spec saved successfully".to_string()),
                }),
                Err(e) => Json(SaveSpecResponse {
                    success: false,
                    message: Some(format!("Failed to write spec: {}", e)),
                }),
            }
        }
        Err(e) => Json(SaveSpecResponse {
            success: false,
            message: Some(format!("Failed to serialize spec: {}", e)),
        }),
    }
}
