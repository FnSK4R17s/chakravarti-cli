//! Model gateway and routing for Chakravarti CLI.
//!
//! This crate provides model provider abstraction, routing logic,
//! and token/cost accounting.

pub mod accounting;
pub mod anthropic;
pub mod error;
pub mod openai;
pub mod pricing;
pub mod provider;
pub mod router;

pub use accounting::{TokenUsage, UsageAccumulator};
pub use anthropic::AnthropicProvider;
pub use error::ModelError;
pub use openai::OpenAIProvider;
pub use pricing::{ModelPricing, PricingCatalog};
pub use provider::{CompletionRequest, CompletionResponse, Message, ModelProvider};
pub use router::{BudgetTracker, ModelRouter, ModelSelection, RoutingContext, TaskType};
