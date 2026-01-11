use axum::{
    extract::{Query, State},
    response::{IntoResponse, Json},
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use crate::state::AppState;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ModelAssignment {
    pub default: String,
    #[serde(default)]
    pub overrides: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Batch {
    pub id: String,
    pub name: String,
    pub task_ids: Vec<String>,
    #[serde(default)]
    pub depends_on: Vec<String>,
    pub model_assignment: ModelAssignment,
    #[serde(default)]
    pub execution_strategy: String,
    #[serde(default)]
    pub estimated_cost: f64,
    #[serde(default)]
    pub estimated_time: String,
    #[serde(default)]
    pub reasoning: String,
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub branch: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Plan {
    pub batches: Vec<Batch>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanResponse {
    pub success: bool,
    pub batches: Vec<Batch>,
    pub raw_yaml: Option<String>,
    pub error: Option<String>,
}

#[derive(Deserialize)]
pub struct GetPlanQuery {
    pub spec: String,
}

#[derive(Deserialize)]
pub struct SavePlanPayload {
    pub spec: String,
    pub batches: Vec<Batch>,
}

#[derive(Serialize)]
pub struct SavePlanResponse {
    pub success: bool,
    pub message: Option<String>,
}

// Helpers
fn get_spec_dir(branch: &str) -> PathBuf {
    // This logic mimics tasks.rs. 
    // Ideally this should be centralized in a safe utility.
    // Assuming run from root or finding .specs dir
    let mut path = std::env::current_dir().unwrap_or_default();
    
    // Naive workspace traversal - similar to tasks.rs (assumed)
    // We expect the user to run this where .specs exists or in a known workspace structure
     if path.join(".specs").exists() {
        return path.join(".specs").join(branch);
    }
    
    // Check known apps paths (as per user env)
    let apps = vec![
        "/apps/chakra-test",
        "/apps/chakravarti-cli",
        // Add others if needed or rely on a more robust find
    ];
    
    for app in apps {
        let p = PathBuf::from(app).join(".specs").join(branch);
        if p.exists() {
            return p;
        }
    }

    path.join(".specs").join(branch)
}

pub async fn get_plan(
    Query(query): Query<GetPlanQuery>,
) -> impl IntoResponse {
    let spec_dir = get_spec_dir(&query.spec);
    let plan_path = spec_dir.join("plan.yaml");

    println!("Loading plan from: {:?}", plan_path);

    if !plan_path.exists() {
        return Json(PlanResponse {
            success: false,
            batches: vec![],
            raw_yaml: None,
            error: Some(format!("Plan file not found at {:?}", plan_path)),
        });
    }

    match fs::read_to_string(&plan_path) {
        Ok(content) => {
            match serde_yaml::from_str::<Plan>(&content) {
                Ok(plan) => Json(PlanResponse {
                    success: true,
                    batches: plan.batches,
                    raw_yaml: Some(content),
                    error: None,
                }),
                Err(e) => Json(PlanResponse {
                    success: false,
                    batches: vec![],
                    raw_yaml: Some(content),
                    error: Some(e.to_string()),
                }),
            }
        },
        Err(e) => Json(PlanResponse {
            success: false,
            batches: vec![],
            raw_yaml: None,
            error: Some(e.to_string()),
        }),
    }
}

pub async fn save_plan(
    Json(payload): Json<SavePlanPayload>,
) -> impl IntoResponse {
    let spec_dir = get_spec_dir(&payload.spec);
    let plan_path = spec_dir.join("plan.yaml");
    
    let plan = Plan {
        batches: payload.batches,
    };

    match serde_yaml::to_string(&plan) {
        Ok(yaml) => {
            match fs::write(&plan_path, yaml) {
                Ok(_) => Json(SavePlanResponse {
                    success: true,
                    message: None,
                }),
                Err(e) => Json(SavePlanResponse {
                    success: false,
                    message: Some(format!("Failed to write file: {}", e)),
                }),
            }
        },
        Err(e) => Json(SavePlanResponse {
            success: false,
            message: Some(format!("Failed to serialize plan: {}", e)),
        }),
    }
}

// OpenRouter Models & Pricing
#[derive(Debug, Serialize, Deserialize)]
struct OpenRouterModelPricing {
    prompt: String,
    completion: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct OpenRouterModel {
    id: String,
    name: String,
    pricing: Option<OpenRouterModelPricing>,
    context_length: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize)]
struct OpenRouterResponse {
    data: Vec<OpenRouterModel>,
}

#[derive(Debug, Serialize)]
pub struct ModelInfo {
    pub id: String,
    pub name: String,
    pub cost_per_1k_prompt: f64,
    pub cost_per_1k_completion: f64,
    pub context_length: u64,
}

#[derive(Debug, Serialize)]
pub struct ModelsResponse {
    pub success: bool,
    pub models: Vec<ModelInfo>,
}

pub async fn get_openrouter_models() -> impl IntoResponse {
    let client = reqwest::Client::new();
    match client.get("https://openrouter.ai/api/v1/models").send().await {
        Ok(resp) => {
            if let Ok(data) = resp.json::<OpenRouterResponse>().await {
                let models: Vec<ModelInfo> = data.data.into_iter().map(|m| {
                    let prompt_cost = m.pricing.as_ref()
                        .and_then(|p| p.prompt.parse::<f64>().ok())
                        .unwrap_or(0.0) * 1000.0;
                    let completion_cost = m.pricing.as_ref()
                        .and_then(|p| p.completion.parse::<f64>().ok())
                        .unwrap_or(0.0) * 1000.0;
                    
                    ModelInfo {
                        id: m.id,
                        name: m.name,
                        cost_per_1k_prompt: prompt_cost,
                        cost_per_1k_completion: completion_cost,
                        context_length: m.context_length.unwrap_or(0),
                    }
                }).collect();

                Json(ModelsResponse {
                    success: true,
                    models,
                })
            } else {
                Json(ModelsResponse {
                    success: false,
                    models: vec![],
                })
            }
        },
        Err(_) => Json(ModelsResponse {
            success: false,
            models: vec![],
        }),
    }
}
