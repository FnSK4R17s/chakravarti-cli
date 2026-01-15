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
    pub has_design: bool,
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

                        // Check if design.md exists
                        let design_md = path.join("design.md");
                        let has_design = design_md.exists();

                        specs.push(Spec {
                            name,
                            path: path.to_string_lossy().to_string(),
                            has_tasks,
                            has_plan,
                            has_design,
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

/// Detailed spec content - supports both old and new formats

// Acceptance scenario for new format (Given/When/Then)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AcceptanceScenario {
    #[serde(default)]
    pub given: String,
    #[serde(default)]
    pub when: String,
    #[serde(default)]
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
    // Old format: acceptance array of Given/When/Then
    #[serde(default)]
    pub acceptance: Vec<AcceptanceScenario>,
    // New format: acceptance_scenarios
    #[serde(default)]
    pub acceptance_scenarios: Vec<AcceptanceScenario>,
    // New format fields
    #[serde(default)]
    pub why_priority: Option<String>,
    #[serde(default)]
    pub independent_test: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FunctionalRequirement {
    pub id: String,
    pub description: String,
    #[serde(default)]
    pub category: Option<String>,
}

// Requirements can be either a Vec (old) or a map with 'functional' key (new)
#[derive(Debug, Clone, Serialize, Default)]
pub struct Requirements {
    pub functional: Vec<FunctionalRequirement>,
}

// Custom deserializer for Requirements to handle both formats
impl<'de> Deserialize<'de> for Requirements {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::{self, MapAccess, SeqAccess, Visitor};
        use std::fmt;

        struct RequirementsVisitor;

        impl<'de> Visitor<'de> for RequirementsVisitor {
            type Value = Requirements;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a sequence or map for requirements")
            }

            // Old format: requirements is an array
            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let mut reqs = Vec::new();
                while let Some(item) = seq.next_element::<FunctionalRequirement>()? {
                    reqs.push(item);
                }
                Ok(Requirements { functional: reqs })
            }

            // New format: requirements is a map with 'functional' key
            fn visit_map<M>(self, mut map: M) -> Result<Self::Value, M::Error>
            where
                M: MapAccess<'de>,
            {
                let mut functional = Vec::new();
                while let Some(key) = map.next_key::<String>()? {
                    if key == "functional" {
                        functional = map.next_value()?;
                    } else {
                        // Skip unknown keys
                        let _: serde::de::IgnoredAny = map.next_value()?;
                    }
                }
                Ok(Requirements { functional })
            }
        }

        deserializer.deserialize_any(RequirementsVisitor)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SuccessCriterion {
    pub id: String,
    pub metric: String,
    #[serde(default)]
    pub measurement: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ClarificationOption {
    #[serde(default)]
    pub label: String,
    #[serde(default)]
    pub answer: String,
    #[serde(default)]
    pub implications: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SpecClarification {
    #[serde(default)]
    pub topic: String,
    #[serde(default)]
    pub question: String,
    #[serde(default)]
    pub options: Vec<ClarificationOption>,
    #[serde(default)]
    pub resolved: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SpecDetail {
    pub id: String,
    // Old format field
    #[serde(default)]
    pub goal: Option<String>,
    // New format field (replaces goal)
    #[serde(default)]
    pub overview: Option<String>,
    // New format metadata
    #[serde(default)]
    pub branch: Option<String>,
    #[serde(default)]
    pub created: Option<String>,
    #[serde(default)]
    pub status: Option<String>,
    // Old format fields
    #[serde(default)]
    pub constraints: Vec<String>,
    #[serde(default)]
    pub acceptance: Vec<String>,
    // Shared fields
    #[serde(default)]
    pub user_stories: Vec<UserStory>,
    #[serde(default)]
    pub requirements: Requirements,
    #[serde(default)]
    pub success_criteria: Vec<SuccessCriterion>,
    #[serde(default)]
    pub assumptions: Vec<String>,
    // New format fields
    #[serde(default)]
    pub edge_cases: Vec<String>,
    #[serde(default)]
    pub clarifications: Vec<SpecClarification>,
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

// ============================================================================
// New API endpoints for spec workflow
// ============================================================================

/// POST /api/specs/create - Create a new spec from description
#[derive(Deserialize)]
pub struct CreateSpecPayload {
    pub description: String,
    pub name: Option<String>,
}

#[derive(Serialize)]
pub struct CreateSpecResponse {
    pub success: bool,
    pub spec_id: Option<String>,
    pub spec_path: Option<String>,
    pub message: Option<String>,
    pub error: Option<String>,
}

pub async fn create_spec(
    Json(payload): Json<CreateSpecPayload>,
) -> impl IntoResponse {
    use std::process::Command;
    
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    
    // Build command
    let mut cmd = Command::new("ckrv");
    cmd.arg("spec").arg("new").arg(&payload.description);
    if let Some(ref name) = payload.name {
        cmd.arg("--name").arg(name);
    }
    cmd.arg("--json");
    cmd.current_dir(&cwd);
    
    match cmd.output() {
        Ok(output) => {
            if output.status.success() {
                // Parse JSON output from ckrv spec new --json
                if let Ok(json_str) = String::from_utf8(output.stdout) {
                    if let Ok(result) = serde_json::from_str::<serde_json::Value>(&json_str) {
                        return Json(CreateSpecResponse {
                            success: true,
                            spec_id: result.get("id").and_then(|v| v.as_str()).map(String::from),
                            spec_path: result.get("spec_folder").and_then(|v| v.as_str()).map(String::from),
                            message: Some("Spec created successfully".to_string()),
                            error: None,
                        });
                    }
                }
                Json(CreateSpecResponse {
                    success: true,
                    spec_id: None,
                    spec_path: None,
                    message: Some("Spec created".to_string()),
                    error: None,
                })
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr).to_string();
                Json(CreateSpecResponse {
                    success: false,
                    spec_id: None,
                    spec_path: None,
                    message: None,
                    error: Some(stderr),
                })
            }
        }
        Err(e) => Json(CreateSpecResponse {
            success: false,
            spec_id: None,
            spec_path: None,
            message: None,
            error: Some(format!("Failed to run ckrv: {}", e)),
        }),
    }
}

/// GET /api/specs/{name}/validate - Validate a spec
#[derive(Serialize)]
pub struct ValidateSpecResponse {
    pub success: bool,
    pub valid: bool,
    pub errors: Vec<ValidationError>,
    pub warnings: Vec<String>,
}

#[derive(Serialize)]
pub struct ValidationError {
    pub field: String,
    pub message: String,
}

pub async fn validate_spec(
    axum::extract::Path(name): axum::extract::Path<String>,
) -> impl IntoResponse {
    use std::process::Command;
    
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let spec_path = cwd.join(".specs").join(&name).join("spec.yaml");
    
    let output = Command::new("ckrv")
        .args(["spec", "validate", "--json"])
        .arg(&spec_path)
        .current_dir(&cwd)
        .output();
    
    match output {
        Ok(out) => {
            if let Ok(json_str) = String::from_utf8(out.stdout) {
                if let Ok(result) = serde_json::from_str::<serde_json::Value>(&json_str) {
                    let valid = result.get("valid").and_then(|v| v.as_bool()).unwrap_or(false);
                    let errors: Vec<ValidationError> = result.get("errors")
                        .and_then(|v| v.as_array())
                        .map(|arr| arr.iter().filter_map(|e| {
                            Some(ValidationError {
                                field: e.get("field")?.as_str()?.to_string(),
                                message: e.get("message")?.as_str()?.to_string(),
                            })
                        }).collect())
                        .unwrap_or_default();
                    let warnings: Vec<String> = result.get("warnings")
                        .and_then(|v| v.as_array())
                        .map(|arr| arr.iter().filter_map(|w| w.as_str().map(String::from)).collect())
                        .unwrap_or_default();
                    
                    return Json(ValidateSpecResponse {
                        success: true,
                        valid,
                        errors,
                        warnings,
                    });
                }
            }
            Json(ValidateSpecResponse {
                success: false,
                valid: false,
                errors: vec![],
                warnings: vec![],
            })
        }
        Err(e) => Json(ValidateSpecResponse {
            success: false,
            valid: false,
            errors: vec![ValidationError {
                field: "system".to_string(),
                message: format!("Failed to run validation: {}", e),
            }],
            warnings: vec![],
        }),
    }
}

/// POST /api/specs/{name}/design - Generate design document
#[derive(Serialize)]
pub struct DesignResponse {
    pub success: bool,
    pub design_path: Option<String>,
    pub research_path: Option<String>,
    pub message: Option<String>,
    pub error: Option<String>,
}

pub async fn generate_design(
    axum::extract::Path(name): axum::extract::Path<String>,
) -> impl IntoResponse {
    use std::process::Command;
    
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let spec_path = cwd.join(".specs").join(&name).join("spec.yaml");
    
    let output = Command::new("ckrv")
        .args(["spec", "design", "--json"])
        .arg(&spec_path)
        .current_dir(&cwd)
        .output();
    
    match output {
        Ok(out) => {
            if out.status.success() {
                if let Ok(json_str) = String::from_utf8(out.stdout) {
                    if let Ok(result) = serde_json::from_str::<serde_json::Value>(&json_str) {
                        return Json(DesignResponse {
                            success: true,
                            design_path: result.get("design_path").and_then(|v| v.as_str()).map(String::from),
                            research_path: result.get("research_path").and_then(|v| v.as_str()).map(String::from),
                            message: Some("Design generated successfully".to_string()),
                            error: None,
                        });
                    }
                }
            }
            let stderr = String::from_utf8_lossy(&out.stderr).to_string();
            Json(DesignResponse {
                success: false,
                design_path: None,
                research_path: None,
                message: None,
                error: Some(stderr),
            })
        }
        Err(e) => Json(DesignResponse {
            success: false,
            design_path: None,
            research_path: None,
            message: None,
            error: Some(format!("Failed to run ckrv: {}", e)),
        }),
    }
}

/// POST /api/specs/{name}/tasks - Generate implementation tasks
#[derive(Serialize)]
pub struct TasksResponse {
    pub success: bool,
    pub tasks_path: Option<String>,
    pub task_count: Option<usize>,
    pub message: Option<String>,
    pub error: Option<String>,
}

pub async fn generate_tasks(
    axum::extract::Path(name): axum::extract::Path<String>,
) -> impl IntoResponse {
    use std::process::Command;
    
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let spec_path = cwd.join(".specs").join(&name).join("spec.yaml");
    
    let output = Command::new("ckrv")
        .args(["spec", "tasks", "--json"])
        .arg(&spec_path)
        .current_dir(&cwd)
        .output();
    
    match output {
        Ok(out) => {
            if out.status.success() {
                if let Ok(json_str) = String::from_utf8(out.stdout) {
                    if let Ok(result) = serde_json::from_str::<serde_json::Value>(&json_str) {
                        return Json(TasksResponse {
                            success: true,
                            tasks_path: result.get("tasks_path").and_then(|v| v.as_str()).map(String::from),
                            task_count: result.get("task_count").and_then(|v| v.as_u64()).map(|v| v as usize),
                            message: Some("Tasks generated successfully".to_string()),
                            error: None,
                        });
                    }
                }
            }
            let stderr = String::from_utf8_lossy(&out.stderr).to_string();
            Json(TasksResponse {
                success: false,
                tasks_path: None,
                task_count: None,
                message: None,
                error: Some(stderr),
            })
        }
        Err(e) => Json(TasksResponse {
            success: false,
            tasks_path: None,
            task_count: None,
            message: None,
            error: Some(format!("Failed to run ckrv: {}", e)),
        }),
    }
}

/// GET /api/specs/{name}/clarifications - Get clarifications for a spec
// Use SpecClarification as Clarification alias
pub type Clarification = SpecClarification;

#[derive(Serialize)]
pub struct ClarificationsResponse {
    pub success: bool,
    pub clarifications: Vec<Clarification>,
    pub unresolved_count: usize,
}

pub async fn get_clarifications(
    axum::extract::Path(name): axum::extract::Path<String>,
) -> impl IntoResponse {
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let spec_path = cwd.join(".specs").join(&name).join("spec.yaml");
    
    if !spec_path.exists() {
        return Json(ClarificationsResponse {
            success: false,
            clarifications: vec![],
            unresolved_count: 0,
        });
    }
    
    // Read spec and extract clarifications
    if let Ok(content) = std::fs::read_to_string(&spec_path) {
        // Try to parse as YAML and extract clarifications
        #[derive(Deserialize)]
        struct SpecWithClarifications {
            #[serde(default)]
            clarifications: Vec<ClarificationParsed>,
        }
        
        #[derive(Deserialize)]
        struct ClarificationParsed {
            topic: String,
            question: String,
            #[serde(default)]
            options: Vec<ClarificationOptionParsed>,
            resolved: Option<String>,
        }
        
        #[derive(Deserialize)]
        struct ClarificationOptionParsed {
            #[serde(default)]
            label: String,
            answer: String,
            implications: Option<String>,
        }
        
        if let Ok(spec) = serde_yaml::from_str::<SpecWithClarifications>(&content) {
            let clarifications: Vec<Clarification> = spec.clarifications.into_iter().map(|c| {
                Clarification {
                    topic: c.topic,
                    question: c.question,
                    options: c.options.into_iter().map(|o| ClarificationOption {
                        label: o.label,
                        answer: o.answer,
                        implications: o.implications,
                    }).collect(),
                    resolved: c.resolved,
                }
            }).collect();
            
            let unresolved_count = clarifications.iter().filter(|c| c.resolved.is_none()).count();
            
            return Json(ClarificationsResponse {
                success: true,
                clarifications,
                unresolved_count,
            });
        }
    }
    
    Json(ClarificationsResponse {
        success: false,
        clarifications: vec![],
        unresolved_count: 0,
    })
}

/// POST /api/specs/{name}/clarify - Submit clarification answers
#[derive(Deserialize)]
pub struct SubmitClarificationPayload {
    pub answers: Vec<ClarificationAnswer>,
}

#[derive(Deserialize)]
pub struct ClarificationAnswer {
    pub topic: String,
    pub answer: String,
}

#[derive(Serialize)]
pub struct SubmitClarificationResponse {
    pub success: bool,
    pub resolved_count: usize,
    pub remaining: usize,
    pub message: Option<String>,
}

pub async fn submit_clarifications(
    axum::extract::Path(name): axum::extract::Path<String>,
    Json(payload): Json<SubmitClarificationPayload>,
) -> impl IntoResponse {
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let spec_path = cwd.join(".specs").join(&name).join("spec.yaml");
    
    if !spec_path.exists() {
        return Json(SubmitClarificationResponse {
            success: false,
            resolved_count: 0,
            remaining: 0,
            message: Some("Spec not found".to_string()),
        });
    }
    
    // Read and update spec
    if let Ok(content) = std::fs::read_to_string(&spec_path) {
        if let Ok(mut spec_value) = serde_yaml::from_str::<serde_yaml::Value>(&content) {
            let mut resolved_count = 0;
            
            if let Some(clarifications) = spec_value.get_mut("clarifications").and_then(|c| c.as_sequence_mut()) {
                for answer in &payload.answers {
                    for clarif in clarifications.iter_mut() {
                        if let Some(topic) = clarif.get("topic").and_then(|t| t.as_str()) {
                            if topic == answer.topic {
                                clarif["resolved"] = serde_yaml::Value::String(answer.answer.clone());
                                resolved_count += 1;
                            }
                        }
                    }
                }
                
                // Count remaining
                let remaining = clarifications.iter()
                    .filter(|c| c.get("resolved").is_none() || c.get("resolved").and_then(|r| r.as_str()).is_none())
                    .count();
                
                // Write back
                if resolved_count > 0 {
                    if let Ok(yaml) = serde_yaml::to_string(&spec_value) {
                        let _ = std::fs::write(&spec_path, yaml);
                    }
                }
                
                return Json(SubmitClarificationResponse {
                    success: true,
                    resolved_count,
                    remaining,
                    message: Some(format!("Resolved {} clarification(s)", resolved_count)),
                });
            }
        }
    }
    
    Json(SubmitClarificationResponse {
        success: false,
        resolved_count: 0,
        remaining: 0,
        message: Some("Failed to update spec".to_string()),
    })
}
