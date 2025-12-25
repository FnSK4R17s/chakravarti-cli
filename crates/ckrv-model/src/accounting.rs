//! Token usage accounting.

use serde::{Deserialize, Serialize};

/// Token usage for a model request.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TokenUsage {
    /// Input/prompt tokens consumed.
    pub prompt_tokens: u32,
    /// Output/completion tokens generated.
    pub completion_tokens: u32,
    /// Total tokens used.
    pub total_tokens: u32,
}

impl TokenUsage {
    /// Create new token usage.
    #[must_use]
    pub fn new(prompt: u32, completion: u32) -> Self {
        Self {
            prompt_tokens: prompt,
            completion_tokens: completion,
            total_tokens: prompt + completion,
        }
    }

    /// Merge with another usage.
    pub fn add(&mut self, other: &TokenUsage) {
        self.prompt_tokens += other.prompt_tokens;
        self.completion_tokens += other.completion_tokens;
        self.total_tokens += other.total_tokens;
    }
}

/// Accumulated usage across multiple requests.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UsageAccumulator {
    /// Total usage across all requests.
    pub total: TokenUsage,
    /// Number of requests.
    pub request_count: u32,
    /// Requests by model.
    pub by_model: std::collections::HashMap<String, TokenUsage>,
}

impl UsageAccumulator {
    /// Create a new accumulator.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Record usage from a request.
    pub fn record(&mut self, model: &str, usage: &TokenUsage) {
        self.total.add(usage);
        self.request_count += 1;

        self.by_model
            .entry(model.to_string())
            .or_default()
            .add(usage);
    }

    /// Estimate cost in USD (rough estimates).
    #[must_use]
    pub fn estimate_cost(&self) -> f64 {
        // Very rough cost estimates per 1M tokens
        // GPT-4o: ~$5/1M input, ~$15/1M output
        // GPT-4o-mini: ~$0.15/1M input, ~$0.60/1M output
        let input_cost_per_m = 2.5; // Average
        let output_cost_per_m = 7.5; // Average

        let input_cost = (f64::from(self.total.prompt_tokens) / 1_000_000.0) * input_cost_per_m;
        let output_cost =
            (f64::from(self.total.completion_tokens) / 1_000_000.0) * output_cost_per_m;

        input_cost + output_cost
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_usage_new() {
        let usage = TokenUsage::new(100, 50);
        assert_eq!(usage.prompt_tokens, 100);
        assert_eq!(usage.completion_tokens, 50);
        assert_eq!(usage.total_tokens, 150);
    }

    #[test]
    fn test_token_usage_add() {
        let mut usage1 = TokenUsage::new(100, 50);
        let usage2 = TokenUsage::new(200, 100);
        usage1.add(&usage2);

        assert_eq!(usage1.prompt_tokens, 300);
        assert_eq!(usage1.completion_tokens, 150);
        assert_eq!(usage1.total_tokens, 450);
    }

    #[test]
    fn test_accumulator_record() {
        let mut acc = UsageAccumulator::new();
        acc.record("gpt-4o", &TokenUsage::new(100, 50));
        acc.record("gpt-4o", &TokenUsage::new(200, 100));

        assert_eq!(acc.request_count, 2);
        assert_eq!(acc.total.total_tokens, 450);
        assert_eq!(acc.by_model.get("gpt-4o").unwrap().total_tokens, 450);
    }

    #[test]
    fn test_estimate_cost() {
        let mut acc = UsageAccumulator::new();
        acc.record("gpt-4o", &TokenUsage::new(1_000_000, 500_000));

        let cost = acc.estimate_cost();
        assert!(cost > 0.0);
    }
}
