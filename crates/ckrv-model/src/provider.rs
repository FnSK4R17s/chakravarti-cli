//! Model provider abstraction.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::{ModelError, TokenUsage};

/// A message in a completion request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    /// Role: system, user, or assistant.
    pub role: String,
    /// Message content.
    pub content: String,
}

/// Request for a model completion.
#[derive(Debug, Clone)]
pub struct CompletionRequest {
    /// Model to use.
    pub model: String,
    /// Messages for the completion.
    pub messages: Vec<Message>,
    /// Maximum tokens to generate.
    pub max_tokens: Option<u32>,
    /// Temperature for sampling.
    pub temperature: Option<f32>,
}

/// Response from a model completion.
#[derive(Debug, Clone)]
pub struct CompletionResponse {
    /// Generated content.
    pub content: String,
    /// Token usage.
    pub usage: TokenUsage,
    /// Model used.
    pub model: String,
    /// Reason for finishing.
    pub finish_reason: String,
}

/// Trait for model providers.
#[async_trait]
pub trait ModelProvider: Send + Sync {
    /// Get provider name.
    fn name(&self) -> &str;

    /// Generate a completion.
    ///
    /// # Errors
    ///
    /// Returns an error if the API call fails.
    async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse, ModelError>;
}
