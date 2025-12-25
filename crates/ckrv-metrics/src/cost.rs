//! Cost estimation with model pricing.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// Cost estimate for a job.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CostEstimate {
    /// Total estimated cost in USD.
    pub total_usd: f64,
    /// Cost breakdown by model.
    pub by_model: HashMap<String, f64>,
}

impl CostEstimate {
    /// Create a new empty cost estimate.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Add cost for a model.
    pub fn add(&mut self, model: impl Into<String>, cost: f64) {
        let model = model.into();
        *self.by_model.entry(model).or_insert(0.0) += cost;
        self.total_usd += cost;
    }

    /// Calculate cost from token usage.
    #[must_use]
    pub fn from_tokens(model: &str, input_tokens: u64, output_tokens: u64) -> f64 {
        let pricing = ModelPricing::default();
        pricing.calculate(model, input_tokens, output_tokens)
    }
}

/// Model pricing information (per 1M tokens).
#[derive(Debug, Clone)]
pub struct ModelPricing {
    /// Input price per 1M tokens by model.
    input_prices: HashMap<String, f64>,
    /// Output price per 1M tokens by model.
    output_prices: HashMap<String, f64>,
}

impl Default for ModelPricing {
    fn default() -> Self {
        let mut input_prices = HashMap::new();
        let mut output_prices = HashMap::new();

        // OpenAI pricing (as of Dec 2024)
        input_prices.insert("gpt-4o".to_string(), 2.50);
        output_prices.insert("gpt-4o".to_string(), 10.00);

        input_prices.insert("gpt-4o-mini".to_string(), 0.15);
        output_prices.insert("gpt-4o-mini".to_string(), 0.60);

        input_prices.insert("gpt-4-turbo".to_string(), 10.00);
        output_prices.insert("gpt-4-turbo".to_string(), 30.00);

        input_prices.insert("gpt-3.5-turbo".to_string(), 0.50);
        output_prices.insert("gpt-3.5-turbo".to_string(), 1.50);

        // Anthropic pricing (as of Dec 2024)
        input_prices.insert("claude-3-5-sonnet".to_string(), 3.00);
        output_prices.insert("claude-3-5-sonnet".to_string(), 15.00);

        input_prices.insert("claude-3-5-haiku".to_string(), 0.80);
        output_prices.insert("claude-3-5-haiku".to_string(), 4.00);

        input_prices.insert("claude-3-opus".to_string(), 15.00);
        output_prices.insert("claude-3-opus".to_string(), 75.00);

        Self {
            input_prices,
            output_prices,
        }
    }
}

impl ModelPricing {
    /// Calculate cost for a model and token count.
    #[must_use]
    pub fn calculate(&self, model: &str, input_tokens: u64, output_tokens: u64) -> f64 {
        // Try to find exact match or prefix match
        let (input_price, output_price) = self.get_prices(model);

        let input_cost = (input_tokens as f64 / 1_000_000.0) * input_price;
        let output_cost = (output_tokens as f64 / 1_000_000.0) * output_price;

        input_cost + output_cost
    }

    fn get_prices(&self, model: &str) -> (f64, f64) {
        // Exact match
        if let (Some(&input), Some(&output)) =
            (self.input_prices.get(model), self.output_prices.get(model))
        {
            return (input, output);
        }

        // Prefix match (e.g., "gpt-4o-mini-2024-07-18" matches "gpt-4o-mini")
        for (key, &input_price) in &self.input_prices {
            if model.starts_with(key) {
                if let Some(&output_price) = self.output_prices.get(key) {
                    return (input_price, output_price);
                }
            }
        }

        // Default fallback pricing
        (1.0, 2.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cost_estimate_add() {
        let mut cost = CostEstimate::new();
        cost.add("gpt-4o", 0.05);
        cost.add("gpt-4o-mini", 0.01);

        assert!((cost.total_usd - 0.06).abs() < 0.0001);
        assert_eq!(cost.by_model.len(), 2);
    }

    #[test]
    fn test_model_pricing_gpt4o_mini() {
        let pricing = ModelPricing::default();

        // 1000 input, 500 output tokens for gpt-4o-mini
        let cost = pricing.calculate("gpt-4o-mini", 1000, 500);

        // $0.15/1M input + $0.60/1M output
        // = (1000/1M * 0.15) + (500/1M * 0.60)
        // = 0.00015 + 0.0003
        // = 0.00045
        assert!((cost - 0.00045).abs() < 0.0001);
    }

    #[test]
    fn test_model_pricing_prefix_match() {
        let pricing = ModelPricing::default();

        // Should match "gpt-4o-mini" prefix
        let cost = pricing.calculate("gpt-4o-mini-2024-07-18", 1000, 500);
        assert!(cost > 0.0);
    }

    #[test]
    fn test_model_pricing_fallback() {
        let pricing = ModelPricing::default();

        // Unknown model should use fallback
        let cost = pricing.calculate("unknown-model", 1000, 500);
        assert!(cost > 0.0);
    }

    #[test]
    fn test_from_tokens() {
        let cost = CostEstimate::from_tokens("gpt-4o", 10000, 5000);
        assert!(cost > 0.0);
    }
}
