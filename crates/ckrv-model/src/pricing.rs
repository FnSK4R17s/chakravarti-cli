//! Model pricing configuration.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// Pricing information for a model.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelPricing {
    /// Model identifier.
    pub model: String,
    /// Provider name.
    pub provider: String,
    /// Cost per 1M input tokens in USD.
    pub input_cost_per_million: f64,
    /// Cost per 1M output tokens in USD.
    pub output_cost_per_million: f64,
    /// Context window size in tokens.
    pub context_window: u32,
    /// Max output tokens.
    pub max_output: u32,
}

impl ModelPricing {
    /// Calculate cost for given token counts.
    #[must_use]
    pub fn calculate_cost(&self, input_tokens: u64, output_tokens: u64) -> f64 {
        let input_cost = (input_tokens as f64 / 1_000_000.0) * self.input_cost_per_million;
        let output_cost = (output_tokens as f64 / 1_000_000.0) * self.output_cost_per_million;
        input_cost + output_cost
    }

    /// Get average cost per 1K tokens (for estimation).
    #[must_use]
    pub fn avg_cost_per_1k(&self) -> f64 {
        (self.input_cost_per_million + self.output_cost_per_million) / 2.0 / 1000.0
    }
}

/// Pricing catalog with known model prices.
pub struct PricingCatalog {
    models: HashMap<String, ModelPricing>,
}

impl Default for PricingCatalog {
    fn default() -> Self {
        Self::new()
    }
}

impl PricingCatalog {
    /// Create a new pricing catalog with default prices.
    #[must_use]
    pub fn new() -> Self {
        let mut models = HashMap::new();

        // OpenAI models (as of Dec 2024)
        models.insert(
            "gpt-4o".to_string(),
            ModelPricing {
                model: "gpt-4o".to_string(),
                provider: "openai".to_string(),
                input_cost_per_million: 2.50,
                output_cost_per_million: 10.00,
                context_window: 128_000,
                max_output: 16_384,
            },
        );

        models.insert(
            "gpt-4o-mini".to_string(),
            ModelPricing {
                model: "gpt-4o-mini".to_string(),
                provider: "openai".to_string(),
                input_cost_per_million: 0.15,
                output_cost_per_million: 0.60,
                context_window: 128_000,
                max_output: 16_384,
            },
        );

        models.insert(
            "gpt-4-turbo".to_string(),
            ModelPricing {
                model: "gpt-4-turbo".to_string(),
                provider: "openai".to_string(),
                input_cost_per_million: 10.00,
                output_cost_per_million: 30.00,
                context_window: 128_000,
                max_output: 4_096,
            },
        );

        models.insert(
            "gpt-3.5-turbo".to_string(),
            ModelPricing {
                model: "gpt-3.5-turbo".to_string(),
                provider: "openai".to_string(),
                input_cost_per_million: 0.50,
                output_cost_per_million: 1.50,
                context_window: 16_385,
                max_output: 4_096,
            },
        );

        models.insert(
            "o1".to_string(),
            ModelPricing {
                model: "o1".to_string(),
                provider: "openai".to_string(),
                input_cost_per_million: 15.00,
                output_cost_per_million: 60.00,
                context_window: 200_000,
                max_output: 100_000,
            },
        );

        models.insert(
            "o1-mini".to_string(),
            ModelPricing {
                model: "o1-mini".to_string(),
                provider: "openai".to_string(),
                input_cost_per_million: 3.00,
                output_cost_per_million: 12.00,
                context_window: 128_000,
                max_output: 65_536,
            },
        );

        // Anthropic models (as of Dec 2024)
        models.insert(
            "claude-3-5-sonnet".to_string(),
            ModelPricing {
                model: "claude-3-5-sonnet".to_string(),
                provider: "anthropic".to_string(),
                input_cost_per_million: 3.00,
                output_cost_per_million: 15.00,
                context_window: 200_000,
                max_output: 8_192,
            },
        );

        models.insert(
            "claude-3-5-haiku".to_string(),
            ModelPricing {
                model: "claude-3-5-haiku".to_string(),
                provider: "anthropic".to_string(),
                input_cost_per_million: 0.80,
                output_cost_per_million: 4.00,
                context_window: 200_000,
                max_output: 8_192,
            },
        );

        models.insert(
            "claude-3-opus".to_string(),
            ModelPricing {
                model: "claude-3-opus".to_string(),
                provider: "anthropic".to_string(),
                input_cost_per_million: 15.00,
                output_cost_per_million: 75.00,
                context_window: 200_000,
                max_output: 4_096,
            },
        );

        Self { models }
    }

    /// Get pricing for a model.
    #[must_use]
    pub fn get(&self, model: &str) -> Option<&ModelPricing> {
        self.models.get(model).or_else(|| {
            // Try prefix match
            for (key, pricing) in &self.models {
                if model.starts_with(key) {
                    return Some(pricing);
                }
            }
            None
        })
    }

    /// Get all available models.
    #[must_use]
    pub fn models(&self) -> Vec<&str> {
        self.models.keys().map(String::as_str).collect()
    }

    /// Get models by provider.
    #[must_use]
    pub fn by_provider(&self, provider: &str) -> Vec<&ModelPricing> {
        self.models
            .values()
            .filter(|p| p.provider == provider)
            .collect()
    }

    /// Get the cheapest model for a provider.
    #[must_use]
    pub fn cheapest(&self, provider: Option<&str>) -> Option<&ModelPricing> {
        self.models
            .values()
            .filter(|p| provider.map_or(true, |prov| p.provider == prov))
            .min_by(|a, b| {
                a.avg_cost_per_1k()
                    .partial_cmp(&b.avg_cost_per_1k())
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
    }

    /// Get the most capable model for a provider (by context window).
    #[must_use]
    pub fn most_capable(&self, provider: Option<&str>) -> Option<&ModelPricing> {
        self.models
            .values()
            .filter(|p| provider.map_or(true, |prov| p.provider == prov))
            .max_by_key(|p| p.context_window)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_pricing_calculate_cost() {
        let pricing = ModelPricing {
            model: "test".to_string(),
            provider: "test".to_string(),
            input_cost_per_million: 1.0,
            output_cost_per_million: 2.0,
            context_window: 8000,
            max_output: 4000,
        };

        // 1M input + 1M output = $1 + $2 = $3
        let cost = pricing.calculate_cost(1_000_000, 1_000_000);
        assert!((cost - 3.0).abs() < 0.001);
    }

    #[test]
    fn test_catalog_get() {
        let catalog = PricingCatalog::new();

        let gpt4o = catalog.get("gpt-4o").expect("should find gpt-4o");
        assert_eq!(gpt4o.provider, "openai");
        assert!(gpt4o.input_cost_per_million > 0.0);
    }

    #[test]
    fn test_catalog_prefix_match() {
        let catalog = PricingCatalog::new();

        // Should match gpt-4o-mini for versioned model
        let matched = catalog.get("gpt-4o-mini-2024-07-18");
        assert!(matched.is_some());
    }

    #[test]
    fn test_catalog_cheapest() {
        let catalog = PricingCatalog::new();

        let cheapest = catalog
            .cheapest(Some("openai"))
            .expect("should find cheapest");
        assert_eq!(cheapest.model, "gpt-4o-mini");
    }

    #[test]
    fn test_catalog_by_provider() {
        let catalog = PricingCatalog::new();

        let anthropic_models = catalog.by_provider("anthropic");
        assert!(!anthropic_models.is_empty());
        assert!(anthropic_models.iter().all(|p| p.provider == "anthropic"));
    }

    #[test]
    fn test_avg_cost_per_1k() {
        let pricing = ModelPricing {
            model: "test".to_string(),
            provider: "test".to_string(),
            input_cost_per_million: 2.0,
            output_cost_per_million: 4.0,
            context_window: 8000,
            max_output: 4000,
        };

        // (2 + 4) / 2 / 1000 = 0.003
        assert!((pricing.avg_cost_per_1k() - 0.003).abs() < 0.0001);
    }
}
