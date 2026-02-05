//! # Specs Handler
//!
//! Handlers for specification management.

use crate::error::TransportError;
use crate::state::AppState;
use crate::types::{
    CreateSpecRequest, ListSpecsResponse, SpecDetail, SpecStatus, SpecSummary, UpdateSpecRequest,
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::process::Command;

// ============================================================================
// Internal Types
// ============================================================================

/// Implementation summary from YAML file.
#[derive(Deserialize)]
struct ImplementationSummary {
    status: String,
    branch: String,
}

// ============================================================================
// Path Utilities
// ============================================================================

/// Get the specs directory path.
fn get_specs_dir() -> PathBuf {
    std::env::current_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join(".specs")
}

/// Get path to a specific spec.
fn get_spec_path(name: &str) -> PathBuf {
    get_specs_dir().join(name)
}

// ============================================================================
// Handlers
// ============================================================================

/// List all specifications.
pub async fn list_specs_handler(_state: &AppState) -> Result<ListSpecsResponse, TransportError> {
    let specs_dir = get_specs_dir();
    let mut specs = Vec::new();

    if specs_dir.exists() && specs_dir.is_dir() {
        if let Ok(entries) = std::fs::read_dir(&specs_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    let name = path
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("")
                        .to_string();

                    // Check if spec.yaml exists
                    let spec_yaml = path.join("spec.yaml");
                    if spec_yaml.exists() {
                        // Check for various artifacts
                        let tasks_yaml = path.join("tasks.yaml");
                        let has_tasks = tasks_yaml.exists();
                        let mut task_count = 0;
                        if has_tasks {
                            if let Ok(content) = std::fs::read_to_string(&tasks_yaml) {
                                task_count = content
                                    .lines()
                                    .filter(|l| l.trim().starts_with("- id:"))
                                    .count();
                            }
                        }

                        let plan_yaml = path.join("plan.yaml");
                        let has_plan = plan_yaml.exists();

                        // Check implementation status
                        let impl_yaml = path.join("implementation.yaml");
                        let (status, implementation_branch) = if impl_yaml.exists() {
                            if let Ok(content) = std::fs::read_to_string(&impl_yaml) {
                                if let Ok(summary) =
                                    serde_yaml::from_str::<ImplementationSummary>(&content)
                                {
                                    if summary.status == "completed" {
                                        (SpecStatus::Implemented, Some(summary.branch))
                                    } else {
                                        (SpecStatus::InProgress, None)
                                    }
                                } else {
                                    (SpecStatus::Draft, None)
                                }
                            } else {
                                (SpecStatus::Draft, None)
                            }
                        } else if has_tasks {
                            (SpecStatus::Planned, None)
                        } else {
                            (SpecStatus::Draft, None)
                        };

                        // Try to read title from spec.yaml
                        let title = if let Ok(content) = std::fs::read_to_string(&spec_yaml) {
                            extract_title_from_yaml(&content)
                        } else {
                            None
                        };

                        specs.push(SpecSummary {
                            name: name.clone(),
                            title,
                            status,
                            has_plan,
                            has_tasks,
                            task_count,
                            created_at: None,
                            updated_at: None,
                        });
                    }
                }
            }
        }
    }

    // Sort by name
    specs.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(specs)
}

/// Extract title from spec YAML content.
fn extract_title_from_yaml(content: &str) -> Option<String> {
    // Simple extraction - look for overview or goal field
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("overview:") || trimmed.starts_with("goal:") {
            let value = trimmed.split(':').nth(1)?.trim();
            if !value.is_empty() && !value.starts_with('|') && !value.starts_with('>') {
                return Some(value.trim_matches('"').trim_matches('\'').to_string());
            }
        }
    }
    None
}

/// Get a single specification.
pub async fn get_spec_handler(
    _state: &AppState,
    name: String,
) -> Result<SpecDetail, TransportError> {
    let spec_path = get_spec_path(&name).join("spec.yaml");

    if !spec_path.exists() {
        return Err(TransportError::NotFound(format!("Spec not found: {name}")));
    }

    let content = std::fs::read_to_string(&spec_path)
        .map_err(|e| TransportError::Internal(format!("Failed to read spec: {e}")))?;

    let spec: serde_yaml::Value = serde_yaml::from_str(&content)
        .map_err(|e| TransportError::Internal(format!("Failed to parse spec: {e}")))?;

    // Extract fields from YAML
    let id = spec
        .get("id")
        .and_then(|v| v.as_str())
        .unwrap_or(&name)
        .to_string();

    let overview = spec
        .get("overview")
        .or_else(|| spec.get("goal"))
        .and_then(|v| v.as_str())
        .map(String::from);

    let status = spec
        .get("status")
        .and_then(|v| v.as_str())
        .map(String::from);

    let branch = spec
        .get("branch")
        .and_then(|v| v.as_str())
        .map(String::from);

    let created = spec
        .get("created")
        .and_then(|v| v.as_str())
        .map(String::from);

    // Extract user stories
    let user_stories: Vec<serde_json::Value> = spec
        .get("user_stories")
        .and_then(|v| serde_json::to_value(v).ok())
        .and_then(|v| v.as_array().cloned())
        .unwrap_or_default();

    // Extract requirements
    let requirements: Vec<serde_json::Value> = spec
        .get("requirements")
        .and_then(|v| {
            if let Some(arr) = v.as_sequence() {
                serde_json::to_value(arr).ok()
            } else if let Some(map) = v.as_mapping() {
                map.get(&serde_yaml::Value::String("functional".to_string()))
                    .and_then(|f| serde_json::to_value(f).ok())
            } else {
                None
            }
        })
        .and_then(|v| v.as_array().cloned())
        .unwrap_or_default();

    // Extract success criteria
    let success_criteria: Vec<serde_json::Value> = spec
        .get("success_criteria")
        .and_then(|v| serde_json::to_value(v).ok())
        .and_then(|v| v.as_array().cloned())
        .unwrap_or_default();

    // Extract assumptions
    let assumptions: Vec<String> = spec
        .get("assumptions")
        .and_then(|v| v.as_sequence())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();

    // Extract edge cases
    let edge_cases: Vec<String> = spec
        .get("edge_cases")
        .and_then(|v| v.as_sequence())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();

    Ok(SpecDetail {
        id,
        overview,
        status,
        branch,
        created,
        user_stories,
        requirements,
        success_criteria,
        assumptions,
        edge_cases,
        raw_yaml: Some(content),
    })
}

/// Create a new specification.
pub async fn create_spec_handler(
    _state: &AppState,
    request: CreateSpecRequest,
) -> Result<SpecSummary, TransportError> {
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

    let mut cmd = Command::new("ckrv");
    cmd.arg("spec").arg("new").arg(&request.description);
    if let Some(ref name) = request.name {
        cmd.arg("--name").arg(name);
    }
    cmd.arg("--json").current_dir(&cwd);

    let output = cmd
        .output()
        .map_err(|e| TransportError::Internal(format!("Failed to run ckrv: {e}")))?;

    if output.status.success() {
        if let Ok(json_str) = String::from_utf8(output.stdout) {
            if let Ok(result) = serde_json::from_str::<serde_json::Value>(&json_str) {
                let spec_id = result
                    .get("id")
                    .and_then(|v| v.as_str())
                    .map(String::from)
                    .unwrap_or_else(|| "unknown".to_string());

                return Ok(SpecSummary {
                    name: spec_id,
                    title: Some(request.description),
                    status: SpecStatus::Draft,
                    has_plan: false,
                    has_tasks: false,
                    task_count: 0,
                    created_at: Some(chrono::Utc::now().to_rfc3339()),
                    updated_at: None,
                });
            }
        }
    }

    let stderr = String::from_utf8_lossy(&output.stderr);
    Err(TransportError::Internal(format!(
        "Failed to create spec: {stderr}"
    )))
}

/// Update a specification.
pub async fn update_spec_handler(
    _state: &AppState,
    name: String,
    request: UpdateSpecRequest,
) -> Result<SpecDetail, TransportError> {
    let spec_path = get_spec_path(&name).join("spec.yaml");

    if !spec_path.exists() {
        return Err(TransportError::NotFound(format!("Spec not found: {name}")));
    }

    // If raw YAML provided, write it directly
    if let Some(yaml) = request.raw_yaml {
        std::fs::write(&spec_path, &yaml)
            .map_err(|e| TransportError::Internal(format!("Failed to write spec: {e}")))?;
    }

    // Return updated spec
    get_spec_handler(_state, name).await
}

/// Delete a specification.
pub async fn delete_spec_handler(_state: &AppState, name: String) -> Result<(), TransportError> {
    let spec_path = get_spec_path(&name);

    if !spec_path.exists() {
        return Err(TransportError::NotFound(format!("Spec not found: {name}")));
    }

    std::fs::remove_dir_all(&spec_path)
        .map_err(|e| TransportError::Internal(format!("Failed to delete spec: {e}")))?;

    Ok(())
}

// ============================================================================
// Additional Spec Operations
// ============================================================================

/// Validate a spec response.
#[derive(Debug, Serialize)]
pub struct ValidateSpecResponse {
    pub valid: bool,
    pub errors: Vec<ValidationError>,
    pub warnings: Vec<String>,
}

/// Validation error.
#[derive(Debug, Serialize)]
pub struct ValidationError {
    pub field: String,
    pub message: String,
}

/// Validate a specification.
pub async fn validate_spec_handler(name: String) -> Result<ValidateSpecResponse, TransportError> {
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let spec_path = cwd.join(".specs").join(&name).join("spec.yaml");

    let output = Command::new("ckrv")
        .args(["spec", "validate", "--json"])
        .arg(&spec_path)
        .current_dir(&cwd)
        .output()
        .map_err(|e| TransportError::Internal(format!("Failed to run validation: {e}")))?;

    if let Ok(json_str) = String::from_utf8(output.stdout) {
        if let Ok(result) = serde_json::from_str::<serde_json::Value>(&json_str) {
            let valid = result
                .get("valid")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);

            let errors: Vec<ValidationError> = result
                .get("errors")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|e| {
                            Some(ValidationError {
                                field: e.get("field")?.as_str()?.to_string(),
                                message: e.get("message")?.as_str()?.to_string(),
                            })
                        })
                        .collect()
                })
                .unwrap_or_default();

            let warnings: Vec<String> = result
                .get("warnings")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|w| w.as_str().map(String::from))
                        .collect()
                })
                .unwrap_or_default();

            return Ok(ValidateSpecResponse {
                valid,
                errors,
                warnings,
            });
        }
    }

    Ok(ValidateSpecResponse {
        valid: false,
        errors: vec![],
        warnings: vec![],
    })
}

/// Generate design response.
#[derive(Debug, Serialize)]
pub struct DesignResponse {
    pub design_path: Option<String>,
    pub research_path: Option<String>,
}

/// Generate design for a spec.
pub async fn generate_design_handler(name: String) -> Result<DesignResponse, TransportError> {
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let spec_path = cwd.join(".specs").join(&name).join("spec.yaml");

    let output = Command::new("ckrv")
        .args(["spec", "design", "--json"])
        .arg(&spec_path)
        .current_dir(&cwd)
        .output()
        .map_err(|e| TransportError::Internal(format!("Failed to run design: {e}")))?;

    if output.status.success() {
        if let Ok(json_str) = String::from_utf8(output.stdout) {
            if let Ok(result) = serde_json::from_str::<serde_json::Value>(&json_str) {
                return Ok(DesignResponse {
                    design_path: result
                        .get("design_path")
                        .and_then(|v| v.as_str())
                        .map(String::from),
                    research_path: result
                        .get("research_path")
                        .and_then(|v| v.as_str())
                        .map(String::from),
                });
            }
        }
    }

    let stderr = String::from_utf8_lossy(&output.stderr);
    Err(TransportError::Internal(format!(
        "Failed to generate design: {stderr}"
    )))
}

/// Generate tasks response.
#[derive(Debug, Serialize)]
pub struct GenerateTasksResponse {
    pub tasks_path: Option<String>,
    pub task_count: usize,
}

/// Generate tasks for a spec.
pub async fn generate_tasks_handler(name: String) -> Result<GenerateTasksResponse, TransportError> {
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let spec_path = cwd.join(".specs").join(&name).join("spec.yaml");

    let output = Command::new("ckrv")
        .args(["spec", "tasks", "--json"])
        .arg(&spec_path)
        .current_dir(&cwd)
        .output()
        .map_err(|e| TransportError::Internal(format!("Failed to run tasks: {e}")))?;

    if output.status.success() {
        if let Ok(json_str) = String::from_utf8(output.stdout) {
            if let Ok(result) = serde_json::from_str::<serde_json::Value>(&json_str) {
                return Ok(GenerateTasksResponse {
                    tasks_path: result
                        .get("tasks_path")
                        .and_then(|v| v.as_str())
                        .map(String::from),
                    task_count: result
                        .get("task_count")
                        .and_then(|v| v.as_u64())
                        .map(|v| v as usize)
                        .unwrap_or(0),
                });
            }
        }
    }

    let stderr = String::from_utf8_lossy(&output.stderr);
    Err(TransportError::Internal(format!(
        "Failed to generate tasks: {stderr}"
    )))
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_title_from_yaml() {
        let yaml = r#"
id: test-spec
overview: This is a test spec
status: draft
"#;
        let title = extract_title_from_yaml(yaml);
        assert_eq!(title, Some("This is a test spec".to_string()));
    }

    #[test]
    fn test_extract_title_with_goal() {
        let yaml = r#"
id: old-spec
goal: Old format goal
"#;
        let title = extract_title_from_yaml(yaml);
        assert_eq!(title, Some("Old format goal".to_string()));
    }
}
