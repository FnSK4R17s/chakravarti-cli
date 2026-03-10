//! # Specs Handler
//!
//! Handlers for specification management.

use crate::error::TransportError;
use crate::state::AppState;
use crate::types::{
    CreateSpecRequest, ListSpecsResponse, SpecDetail, SpecStatus, SpecSummary, UpdateSpecRequest,
};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::process::Command;

// ============================================================
// Internal Types
// ============================================================

/// Implementation summary from YAML file.
#[derive(Deserialize)]
struct ImplementationSummary {
    status: String,
    branch: String,
}

// ============================================================
// Path Utilities
// ============================================================

/// Get the specs directory path for a given project root.
fn get_specs_dir(project_root: &Path) -> PathBuf {
    project_root.join(".specs")
}

/// Get path to a specific spec within a project.
fn get_spec_path(project_root: &Path, name: &str) -> PathBuf {
    get_specs_dir(project_root).join(name)
}

// ============================================================
// Handlers
// ============================================================

/// List all specifications.
pub fn list_specs_handler(state: &AppState) -> Result<ListSpecsResponse, TransportError> {
    let specs_dir = get_specs_dir(&state.project_root);
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

                        // Check for design artifact (CLI creates design.md)
                        let has_design = path.join("design.md").exists() || path.join("design.yaml").exists();

                        // Check implementation status
                        let impl_yaml = path.join("implementation.yaml");
                        let has_implementation = impl_yaml.exists();
                        let (status, implementation_branch) = if has_implementation {
                            std::fs::read_to_string(&impl_yaml).ok().map_or(
                                (SpecStatus::Draft, None),
                                |content| {
                                    serde_yaml::from_str::<ImplementationSummary>(&content)
                                        .ok()
                                        .map_or((SpecStatus::Draft, None), |summary| {
                                            if summary.status == "completed" {
                                                (SpecStatus::Implemented, Some(summary.branch))
                                            } else {
                                                (SpecStatus::InProgress, Some(summary.branch))
                                            }
                                        })
                                },
                            )
                        } else if has_tasks {
                            (SpecStatus::Planned, None)
                        } else {
                            (SpecStatus::Draft, None)
                        };

                        // Try to read title from spec.yaml
                        let title = std::fs::read_to_string(&spec_yaml)
                            .ok()
                            .and_then(|content| extract_title_from_yaml(&content));

                        let spec_path = format!(".specs/{name}");
                        specs.push(SpecSummary {
                            name: name.clone(),
                            path: spec_path,
                            title,
                            status,
                            has_plan,
                            has_tasks,
                            has_design,
                            has_implementation,
                            implementation_branch,
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
    let count = specs.len();
    Ok(ListSpecsResponse { specs, count })
}

/// Extract title from spec YAML content.
///
/// Handles both inline values (`overview: Some text`) and block scalars
/// (`overview: |\n  First line of text`).
fn extract_title_from_yaml(content: &str) -> Option<String> {
    let lines: Vec<&str> = content.lines().collect();
    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with("overview:") || trimmed.starts_with("goal:") {
            let value = trimmed.split(':').nth(1)?.trim();
            if value.starts_with('|') || value.starts_with('>') {
                // Block scalar — grab the first non-empty indented line
                for next_line in &lines[i + 1..] {
                    let next_trimmed = next_line.trim();
                    if next_trimmed.is_empty() {
                        continue;
                    }
                    // Stop if we hit a non-indented line (new YAML key)
                    if !next_line.starts_with(' ') && !next_line.starts_with('\t') {
                        break;
                    }
                    // Truncate long overviews for the title
                    let title = if next_trimmed.len() > 120 {
                        format!("{}…", &next_trimmed[..120])
                    } else {
                        next_trimmed.to_string()
                    };
                    return Some(title);
                }
            } else if !value.is_empty() {
                return Some(value.trim_matches('"').trim_matches('\'').to_string());
            }
        }
    }
    None
}

/// Get a single specification.
pub fn get_spec_handler(state: &AppState, name: String) -> Result<SpecDetail, TransportError> {
    let spec_path = get_spec_path(&state.project_root, &name).join("spec.yaml");

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
            v.as_sequence().map_or_else(
                || {
                    v.as_mapping().and_then(|map| {
                        map.get(serde_yaml::Value::String("functional".to_string()))
                            .and_then(|f| serde_json::to_value(f).ok())
                    })
                },
                |arr| serde_json::to_value(arr).ok(),
            )
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
pub fn create_spec_handler(
    state: &AppState,
    request: CreateSpecRequest,
) -> Result<SpecSummary, TransportError> {
    let project_root = &state.project_root;

    let mut cmd = Command::new("ckrv");
    cmd.arg("spec").arg("new").arg(&request.description);
    if let Some(ref name) = request.name {
        cmd.arg("--name").arg(name);
    }
    cmd.arg("--json").current_dir(project_root);

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

                let spec_path = format!(".specs/{spec_id}");
                return Ok(SpecSummary {
                    name: spec_id,
                    path: spec_path,
                    title: Some(request.description),
                    status: SpecStatus::Draft,
                    has_plan: false,
                    has_tasks: false,
                    has_design: false,
                    has_implementation: false,
                    implementation_branch: None,
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
pub fn update_spec_handler(
    state: &AppState,
    name: String,
    request: UpdateSpecRequest,
) -> Result<SpecDetail, TransportError> {
    let spec_path = get_spec_path(&state.project_root, &name).join("spec.yaml");

    if !spec_path.exists() {
        return Err(TransportError::NotFound(format!("Spec not found: {name}")));
    }

    // If raw YAML provided, write it directly
    if let Some(yaml) = request.raw_yaml {
        std::fs::write(&spec_path, &yaml)
            .map_err(|e| TransportError::Internal(format!("Failed to write spec: {e}")))?;
    }

    // Return updated spec
    get_spec_handler(state, name)
}

/// Delete a specification.
pub fn delete_spec_handler(state: &AppState, name: String) -> Result<(), TransportError> {
    let spec_path = get_spec_path(&state.project_root, &name);

    if !spec_path.exists() {
        return Err(TransportError::NotFound(format!("Spec not found: {name}")));
    }

    std::fs::remove_dir_all(&spec_path)
        .map_err(|e| TransportError::Internal(format!("Failed to delete spec: {e}")))?;

    Ok(())
}

// ============================================================
// Additional Spec Operations
// ============================================================

/// Validate a spec response.
#[derive(Debug, Serialize)]
pub struct ValidateSpecResponse {
    /// Whether the spec is valid.
    pub valid: bool,
    /// Validation errors found.
    pub errors: Vec<ValidationError>,
    /// Non-blocking warnings.
    pub warnings: Vec<String>,
}

/// Validation error.
#[derive(Debug, Serialize)]
pub struct ValidationError {
    /// YAML field path where the error occurred.
    pub field: String,
    /// Description of the validation failure.
    pub message: String,
}

/// Validate a specification.
pub fn validate_spec_handler(
    state: &AppState,
    name: String,
) -> Result<ValidateSpecResponse, TransportError> {
    let project_root = &state.project_root;
    let spec_path = project_root.join(".specs").join(&name).join("spec.yaml");

    let output = Command::new("ckrv")
        .args(["spec", "validate", "--json"])
        .arg(&spec_path)
        .current_dir(project_root)
        .output()
        .map_err(|e| TransportError::Internal(format!("Failed to run validation: {e}")))?;

    if let Ok(json_str) = String::from_utf8(output.stdout) {
        if let Ok(result) = serde_json::from_str::<serde_json::Value>(&json_str) {
            let valid = result
                .get("valid")
                .and_then(serde_json::Value::as_bool)
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
    /// Path to the generated design document.
    pub design_path: Option<String>,
    /// Path to the generated research document.
    pub research_path: Option<String>,
}

/// Extract the last JSON object from mixed stdout output.
///
/// The Docker sandbox streams container stdout (AI agent response text) to the
/// parent process stdout, so the captured stdout may contain non-JSON text before
/// the CLI's final JSON output. This finds and parses the last `{...}` block.
fn extract_json_from_output(raw: &str) -> Option<serde_json::Value> {
    // First try the entire string as JSON (fast path)
    if let Ok(val) = serde_json::from_str::<serde_json::Value>(raw) {
        return Some(val);
    }

    // Find the last top-level JSON object by scanning backwards for '}'
    // then matching its opening '{'
    let bytes = raw.as_bytes();
    let mut end = bytes.len();
    while end > 0 {
        // Find the last '}' before current end
        if let Some(pos) = raw[..end].rfind('}') {
            let candidate_end = pos + 1;
            // Now walk backwards from pos to find the matching '{'
            let mut depth = 0i32;
            let mut start = None;
            for i in (0..candidate_end).rev() {
                match bytes[i] {
                    b'}' => depth += 1,
                    b'{' => {
                        depth -= 1;
                        if depth == 0 {
                            start = Some(i);
                            break;
                        }
                    }
                    _ => {}
                }
            }
            if let Some(start) = start {
                let slice = &raw[start..candidate_end];
                if let Ok(val) = serde_json::from_str::<serde_json::Value>(slice) {
                    return Some(val);
                }
            }
            end = pos; // Try further back
        } else {
            break;
        }
    }
    None
}

/// Generate design for a spec.
pub fn generate_design_handler(
    state: &AppState,
    name: String,
) -> Result<DesignResponse, TransportError> {
    let project_root = &state.project_root;
    let spec_path = project_root.join(".specs").join(&name).join("spec.yaml");

    // Use --force if design.md exists but is empty (previous failed generation)
    let design_file = project_root.join(".specs").join(&name).join("design.md");
    let needs_force = design_file.exists()
        && std::fs::metadata(&design_file)
            .map(|m| m.len() == 0)
            .unwrap_or(false);

    let mut args = vec!["spec", "design", "--json"];
    if needs_force {
        args.push("--force");
    }

    let output = Command::new("ckrv")
        .args(&args)
        .arg(&spec_path)
        .current_dir(project_root)
        .output()
        .map_err(|e| TransportError::Internal(format!("Failed to run design: {e}")))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    if let Some(result) = extract_json_from_output(&stdout) {
        // Check for logical failure (CLI exits 0 but returns success: false)
        if result.get("success").and_then(|v| v.as_bool()) == Some(false) {
            let error_msg = result
                .get("error")
                .and_then(|v| v.as_str())
                .unwrap_or("Design generation failed");
            return Err(TransportError::Internal(error_msg.to_string()));
        }

        let design_path = result
            .get("design_path")
            .and_then(|v| v.as_str())
            .map(String::from);

        // Verify the design file has actual content
        if let Some(ref path) = design_path {
            let file_path = std::path::Path::new(path);
            if file_path.exists() {
                let metadata = std::fs::metadata(file_path)
                    .map_err(|e| TransportError::Internal(format!("Cannot read design file: {e}")))?;
                if metadata.len() == 0 {
                    return Err(TransportError::Internal(
                        "Design generation produced an empty file. The AI agent may have failed to generate content. Try running again.".to_string()
                    ));
                }
            }
        }

        return Ok(DesignResponse {
            design_path,
            research_path: result
                .get("research_path")
                .and_then(|v| v.as_str())
                .map(String::from),
        });
    }

    let stderr = String::from_utf8_lossy(&output.stderr);
    Err(TransportError::Internal(format!(
        "Failed to generate design: {stderr}"
    )))
}

/// Generate tasks response.
#[derive(Debug, Serialize)]
pub struct GenerateTasksResponse {
    /// Path to the generated tasks YAML file.
    pub tasks_path: Option<String>,
    /// Number of tasks generated.
    pub task_count: usize,
}

/// Generate tasks for a spec.
pub fn generate_tasks_handler(
    state: &AppState,
    name: String,
) -> Result<GenerateTasksResponse, TransportError> {
    let project_root = &state.project_root;
    let spec_path = project_root.join(".specs").join(&name).join("spec.yaml");

    // Use --force if tasks.yaml exists but is empty (previous failed generation)
    let tasks_file = project_root.join(".specs").join(&name).join("tasks.yaml");
    let needs_force = tasks_file.exists()
        && std::fs::metadata(&tasks_file)
            .map(|m| m.len() == 0)
            .unwrap_or(false);

    let mut args = vec!["spec", "tasks", "--json"];
    if needs_force {
        args.push("--force");
    }

    let output = Command::new("ckrv")
        .args(&args)
        .arg(&spec_path)
        .current_dir(project_root)
        .output()
        .map_err(|e| TransportError::Internal(format!("Failed to run tasks: {e}")))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    if let Some(result) = extract_json_from_output(&stdout) {
        // Check for logical failure (CLI exits 0 but returns success: false)
        if result.get("success").and_then(|v| v.as_bool()) == Some(false) {
            let error_msg = result
                .get("error")
                .and_then(|v| v.as_str())
                .unwrap_or("Task generation failed");
            return Err(TransportError::Internal(error_msg.to_string()));
        }

        let tasks_path = result
            .get("tasks_path")
            .and_then(|v| v.as_str())
            .map(String::from);

        // Verify the tasks file has actual content
        if let Some(ref path) = tasks_path {
            let file_path = std::path::Path::new(path);
            if file_path.exists() {
                let metadata = std::fs::metadata(file_path)
                    .map_err(|e| TransportError::Internal(format!("Cannot read tasks file: {e}")))?;
                if metadata.len() == 0 {
                    return Err(TransportError::Internal(
                        "Task generation produced an empty file. The AI agent may have failed to generate content. Try running again.".to_string()
                    ));
                }
            }
        }

        return Ok(GenerateTasksResponse {
            tasks_path,
            task_count: result
                .get("task_count")
                .and_then(serde_json::Value::as_u64)
                .map(|v| v as usize)
                .unwrap_or(0),
        });
    }

    let stderr = String::from_utf8_lossy(&output.stderr);
    Err(TransportError::Internal(format!(
        "Failed to generate tasks: {stderr}"
    )))
}

// ============================================================
// Tests
// ============================================================

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

    #[test]
    fn test_extract_title_block_scalar() {
        let yaml = r#"id: "001-webapp"
overview: |
  A full-stack web application with a compiled backend
  serving a modern frontend.
status: draft
"#;
        let title = extract_title_from_yaml(yaml);
        assert_eq!(
            title,
            Some("A full-stack web application with a compiled backend".to_string())
        );
    }

    #[test]
    fn test_extract_title_folded_scalar() {
        let yaml = r#"id: "002-api"
overview: >
  A REST API with authentication
status: draft
"#;
        let title = extract_title_from_yaml(yaml);
        assert_eq!(title, Some("A REST API with authentication".to_string()));
    }

    #[test]
    fn test_extract_json_clean() {
        let input = r#"{"success": true, "message": "done"}"#;
        let result = extract_json_from_output(input).unwrap();
        assert_eq!(result["success"], true);
    }

    #[test]
    fn test_extract_json_with_leading_text() {
        let input = "Generating tasks with AI...\nSome agent output here\n{\"success\": true, \"tasks_path\": \"/tmp/tasks.yaml\"}";
        let result = extract_json_from_output(input).unwrap();
        assert_eq!(result["success"], true);
        assert_eq!(result["tasks_path"], "/tmp/tasks.yaml");
    }

    #[test]
    fn test_extract_json_with_lots_of_text() {
        let input = "# Design Document\n\nThis is a long AI response with {inline braces} and more text.\n\nFinal answer:\n{\"success\": true, \"design_path\": \"/tmp/design.md\", \"research_path\": \"/tmp/research.md\", \"message\": \"Design generated successfully\"}";
        let result = extract_json_from_output(input).unwrap();
        assert_eq!(result["success"], true);
        assert_eq!(result["design_path"], "/tmp/design.md");
    }

    #[test]
    fn test_extract_json_no_json() {
        let input = "Just some plain text with no JSON";
        assert!(extract_json_from_output(input).is_none());
    }

    #[test]
    fn test_extract_json_pretty_printed() {
        let input = "Agent output...\n{\n  \"success\": true,\n  \"message\": \"done\"\n}";
        let result = extract_json_from_output(input).unwrap();
        assert_eq!(result["success"], true);
    }
}
