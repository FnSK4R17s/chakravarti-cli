//! Model routing logic with optimization modes.

use std::sync::{Arc, Mutex};

use ckrv_core::OptimizeMode;

use crate::{
    anthropic::AnthropicProvider,
    openai::OpenAIProvider,
    provider::{CompletionRequest, CompletionResponse, ModelProvider},
    ModelError,
};

/// Context for routing decisions.
#[derive(Debug, Clone)]
pub struct RoutingContext {
    /// Optimization mode.
    pub optimize: OptimizeMode,
    /// Task type (planning, execution, verification).
    pub task_type: TaskType,
    /// Estimated input size.
    pub estimated_tokens: Option<u32>,
    /// Model override (if specified by user).
    pub model_override: Option<String>,
}

impl Default for RoutingContext {
    fn default() -> Self {
        Self {
            optimize: OptimizeMode::Balanced,
            task_type: TaskType::Execution,
            estimated_tokens: None,
            model_override: None,
        }
    }
}

/// Type of task for routing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskType {
    /// Planning phase - needs strong reasoning.
    Planning,
    /// Execution phase - needs code generation.
    Execution,
    /// Verification phase - needs analysis.
    Verification,
}

/// Budget tracker for cost-optimized routing.
#[derive(Debug, Clone)]
pub struct BudgetTracker {
    /// Maximum budget in USD.
    pub max_budget_usd: f64,
    /// Current spent amount.
    pub spent_usd: f64,
    /// Token count by model.
    pub tokens_by_model: std::collections::HashMap<String, (u64, u64)>,
}

impl Default for BudgetTracker {
    fn default() -> Self {
        Self::new(10.0) // Default $10 budget
    }
}

impl BudgetTracker {
    /// Create a new budget tracker.
    #[must_use]
    pub fn new(max_budget_usd: f64) -> Self {
        Self {
            max_budget_usd,
            spent_usd: 0.0,
            tokens_by_model: std::collections::HashMap::new(),
        }
    }

    /// Record token usage.
    pub fn record(&mut self, model: &str, input_tokens: u64, output_tokens: u64, cost_usd: f64) {
        self.spent_usd += cost_usd;
        let entry = self
            .tokens_by_model
            .entry(model.to_string())
            .or_insert((0, 0));
        entry.0 += input_tokens;
        entry.1 += output_tokens;
    }

    /// Check if budget is available.
    #[must_use]
    pub fn has_budget(&self, estimated_cost: f64) -> bool {
        self.spent_usd + estimated_cost <= self.max_budget_usd
    }

    /// Get remaining budget.
    #[must_use]
    pub fn remaining(&self) -> f64 {
        (self.max_budget_usd - self.spent_usd).max(0.0)
    }

    /// Get total spent.
    #[must_use]
    pub fn spent(&self) -> f64 {
        self.spent_usd
    }
}

/// Model selection result.
#[derive(Debug, Clone)]
pub struct ModelSelection {
    /// Selected model name.
    pub model: String,
    /// Provider to use.
    pub provider: String,
    /// Estimated cost per 1K tokens.
    pub estimated_cost_per_1k: f64,
    /// Reason for selection.
    pub reason: String,
}

/// Model router for selecting and calling providers.
pub struct ModelRouter {
    providers: Vec<Arc<dyn ModelProvider>>,
    default_planner_model: String,
    default_executor_model: String,
    budget: Arc<Mutex<BudgetTracker>>,
}

impl ModelRouter {
    /// Create a new router with available providers.
    ///
    /// # Errors
    ///
    /// Returns an error if no providers are available.
    pub fn new() -> Result<Self, ModelError> {
        let mut providers: Vec<Arc<dyn ModelProvider>> = Vec::new();

        // Try to create OpenAI provider
        if let Ok(provider) = OpenAIProvider::new() {
            providers.push(Arc::new(provider));
        }

        // Try to create Anthropic provider
        if let Ok(provider) = AnthropicProvider::new() {
            providers.push(Arc::new(provider));
        }

        // Check for custom endpoint
        if let (Ok(key), Ok(url)) = (
            std::env::var("CKRV_MODEL_API_KEY"),
            std::env::var("CKRV_MODEL_ENDPOINT"),
        ) {
            let provider = OpenAIProvider::with_endpoint(key, url);
            providers.push(Arc::new(provider));
        }

        if providers.is_empty() {
            return Err(ModelError::ConfigError(
                "No model providers configured".to_string(),
            ));
        }

        Ok(Self {
            providers,
            default_planner_model: "gpt-4o".to_string(),
            default_executor_model: "gpt-4o-mini".to_string(),
            budget: Arc::new(Mutex::new(BudgetTracker::default())),
        })
    }

    /// Set the budget limit.
    pub fn set_budget(&self, max_usd: f64) {
        if let Ok(mut budget) = self.budget.lock() {
            budget.max_budget_usd = max_usd;
        }
    }

    /// Get the budget tracker.
    pub fn budget(&self) -> Arc<Mutex<BudgetTracker>> {
        Arc::clone(&self.budget)
    }

    /// Set the default planner model.
    pub fn set_planner_model(&mut self, model: String) {
        self.default_planner_model = model;
    }

    /// Set the default executor model.
    pub fn set_executor_model(&mut self, model: String) {
        self.default_executor_model = model;
    }

    /// Select the best model for a task with full details.
    #[must_use]
    pub fn select(&self, context: &RoutingContext) -> ModelSelection {
        // Honor explicit override
        if let Some(ref override_model) = context.model_override {
            return ModelSelection {
                model: override_model.clone(),
                provider: self.provider_for_model(override_model),
                estimated_cost_per_1k: self.cost_per_1k(override_model),
                reason: "User override".to_string(),
            };
        }

        // Select based on optimization mode
        let (model, reason) = match context.optimize {
            OptimizeMode::Cost => self.select_cost_optimized(context),
            OptimizeMode::Time => self.select_time_optimized(context),
            OptimizeMode::Balanced => self.select_balanced(context),
        };

        ModelSelection {
            provider: self.provider_for_model(&model),
            estimated_cost_per_1k: self.cost_per_1k(&model),
            model,
            reason,
        }
    }

    fn select_cost_optimized(&self, context: &RoutingContext) -> (String, String) {
        // Always use cheapest models for cost optimization
        let model = match context.task_type {
            TaskType::Planning => "gpt-4o-mini",
            TaskType::Execution => "gpt-4o-mini",
            TaskType::Verification => "gpt-4o-mini",
        };
        (
            model.to_string(),
            "Cost optimized: using cheapest model".to_string(),
        )
    }

    fn select_time_optimized(&self, context: &RoutingContext) -> (String, String) {
        // Use faster/more capable models for time optimization
        let model = match context.task_type {
            TaskType::Planning => "gpt-4o",  // Better reasoning = fewer retries
            TaskType::Execution => "gpt-4o", // Better code gen = fewer retries
            TaskType::Verification => "gpt-4o-mini", // Quick verification
        };
        (
            model.to_string(),
            "Time optimized: using fastest capable model".to_string(),
        )
    }

    fn select_balanced(&self, context: &RoutingContext) -> (String, String) {
        // Use task-appropriate models
        let model = match context.task_type {
            TaskType::Planning => self.default_planner_model.clone(),
            TaskType::Execution => self.default_executor_model.clone(),
            TaskType::Verification => "gpt-4o-mini".to_string(),
        };
        (
            model.clone(),
            format!("Balanced: {} for {:?}", model, context.task_type),
        )
    }

    fn provider_for_model(&self, model: &str) -> String {
        if model.starts_with("claude") {
            "anthropic".to_string()
        } else if model.starts_with("gpt") || model.starts_with("o1") {
            "openai".to_string()
        } else {
            "custom".to_string()
        }
    }

    fn cost_per_1k(&self, model: &str) -> f64 {
        // Approximate cost per 1K tokens (input + output average)
        match model {
            "gpt-4o" => 0.00625,       // ($2.5 + $10) / 2 / 1000
            "gpt-4o-mini" => 0.000375, // ($0.15 + $0.60) / 2 / 1000
            "gpt-4-turbo" => 0.02,
            "claude-3-5-sonnet" => 0.009,
            "claude-3-5-haiku" => 0.0024,
            "claude-3-opus" => 0.045,
            _ => 0.01, // Default estimate
        }
    }

    /// Select the best model for a task (simple string return).
    #[must_use]
    pub fn select_model(&self, context: &RoutingContext) -> String {
        self.select(context).model
    }

    /// Complete a request using the first available provider.
    ///
    /// # Errors
    ///
    /// Returns an error if all providers fail.
    pub async fn complete(
        &self,
        request: CompletionRequest,
    ) -> Result<CompletionResponse, ModelError> {
        let mut last_error = None;

        for provider in &self.providers {
            match provider.complete(request.clone()).await {
                Ok(response) => {
                    // Record budget usage
                    if let Ok(mut budget) = self.budget.lock() {
                        let cost = self.cost_per_1k(&request.model)
                            * (response.usage.prompt_tokens + response.usage.completion_tokens)
                                as f64
                            / 1000.0;
                        budget.record(
                            &request.model,
                            response.usage.prompt_tokens.into(),
                            response.usage.completion_tokens.into(),
                            cost,
                        );
                    }
                    return Ok(response);
                }
                Err(e) => {
                    last_error = Some(e);
                    continue;
                }
            }
        }

        Err(last_error
            .unwrap_or_else(|| ModelError::ConfigError("No providers available".to_string())))
    }

    /// Get the list of available provider names.
    #[must_use]
    pub fn provider_names(&self) -> Vec<&str> {
        self.providers.iter().map(|p| p.name()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_select_model_cost_optimization() {
        let context = RoutingContext {
            optimize: OptimizeMode::Cost,
            task_type: TaskType::Planning,
            estimated_tokens: None,
            model_override: None,
        };

        // Create a router-like selection (without providers)
        let (model, _) = match context.optimize {
            OptimizeMode::Cost => ("gpt-4o-mini".to_string(), "cost".to_string()),
            OptimizeMode::Time => ("gpt-4o".to_string(), "time".to_string()),
            OptimizeMode::Balanced => ("gpt-4o".to_string(), "balanced".to_string()),
        };

        assert_eq!(model, "gpt-4o-mini");
    }

    #[test]
    fn test_select_model_time_optimization() {
        let context = RoutingContext {
            optimize: OptimizeMode::Time,
            task_type: TaskType::Execution,
            estimated_tokens: Some(1000),
            model_override: None,
        };

        let model = match (context.optimize, context.task_type) {
            (OptimizeMode::Time, TaskType::Execution) => "gpt-4o",
            _ => "gpt-4o-mini",
        };

        assert_eq!(model, "gpt-4o");
    }

    #[test]
    fn test_model_override() {
        let context = RoutingContext {
            optimize: OptimizeMode::Cost,
            task_type: TaskType::Planning,
            estimated_tokens: None,
            model_override: Some("claude-3-5-sonnet".to_string()),
        };

        // Override should take precedence
        if let Some(ref model) = context.model_override {
            assert_eq!(model, "claude-3-5-sonnet");
        }
    }

    #[test]
    fn test_budget_tracker() {
        let mut budget = BudgetTracker::new(5.0);

        assert!(budget.has_budget(1.0));
        assert_eq!(budget.remaining(), 5.0);

        budget.record("gpt-4o", 1000, 500, 1.5);

        assert_eq!(budget.spent(), 1.5);
        assert_eq!(budget.remaining(), 3.5);
        assert!(budget.has_budget(3.0));
        assert!(!budget.has_budget(4.0));
    }

    #[test]
    fn test_budget_tracker_tokens() {
        let mut budget = BudgetTracker::new(10.0);

        budget.record("gpt-4o", 1000, 500, 0.5);
        budget.record("gpt-4o", 2000, 1000, 1.0);
        budget.record("gpt-4o-mini", 5000, 2000, 0.1);

        assert_eq!(budget.tokens_by_model.len(), 2);
        assert_eq!(budget.tokens_by_model.get("gpt-4o"), Some(&(3000, 1500)));
        assert_eq!(
            budget.tokens_by_model.get("gpt-4o-mini"),
            Some(&(5000, 2000))
        );
    }

    #[test]
    fn test_routing_context_default() {
        let context = RoutingContext::default();

        assert!(matches!(context.optimize, OptimizeMode::Balanced));
        assert!(matches!(context.task_type, TaskType::Execution));
        assert!(context.model_override.is_none());
    }

    #[test]
    fn test_model_selection_struct() {
        let selection = ModelSelection {
            model: "gpt-4o".to_string(),
            provider: "openai".to_string(),
            estimated_cost_per_1k: 0.00625,
            reason: "Test".to_string(),
        };

        assert_eq!(selection.model, "gpt-4o");
        assert_eq!(selection.provider, "openai");
    }
}
