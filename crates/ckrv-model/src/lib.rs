//! Model gateway and routing for Chakravarti CLI.
//!
//! This crate provides model provider abstraction, routing logic,
//! and token/cost accounting. It supports OpenAI and Anthropic backends
//! with budget-aware model selection.

/// Token usage accounting and accumulation.
pub mod accounting;
/// Anthropic Claude model provider implementation.
pub mod anthropic;
/// Model error types.
pub mod error;
/// OpenAI model provider implementation.
pub mod openai;
/// Model pricing catalog and cost calculation.
pub mod pricing;
/// Model provider trait and completion types.
pub mod provider;
/// Model routing logic with optimization modes.
pub mod router;

pub use accounting::{TokenUsage, UsageAccumulator};
pub use anthropic::AnthropicProvider;
pub use error::ModelError;
pub use openai::OpenAIProvider;
pub use pricing::{ModelPricing, PricingCatalog};
pub use provider::{CompletionRequest, CompletionResponse, Message, ModelProvider};
pub use router::{BudgetTracker, ModelRouter, ModelSelection, RoutingContext, TaskType};
