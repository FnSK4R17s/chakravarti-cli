//! OpenAI model provider implementation.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::{
    provider::{CompletionRequest, CompletionResponse, Message, ModelProvider},
    ModelError, TokenUsage,
};

/// OpenAI API provider.
pub struct OpenAIProvider {
    client: reqwest::Client,
    api_key: String,
    base_url: String,
}

impl OpenAIProvider {
    /// Create a new OpenAI provider.
    ///
    /// # Errors
    ///
    /// Returns an error if the API key is not set.
    pub fn new() -> Result<Self, ModelError> {
        let api_key = std::env::var("OPENAI_API_KEY")
            .map_err(|_| ModelError::ConfigError("OPENAI_API_KEY not set".to_string()))?;

        Ok(Self {
            client: reqwest::Client::new(),
            api_key,
            base_url: "https://api.openai.com/v1".to_string(),
        })
    }

    /// Create with custom endpoint (for OpenRouter/compatible APIs).
    #[must_use]
    pub fn with_endpoint(api_key: String, base_url: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            api_key,
            base_url,
        }
    }
}

#[derive(Serialize)]
struct OpenAIRequest {
    model: String,
    messages: Vec<OpenAIMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
}

#[derive(Serialize)]
struct OpenAIMessage {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct OpenAIResponse {
    choices: Vec<OpenAIChoice>,
    usage: OpenAIUsage,
    model: String,
}

#[derive(Deserialize)]
struct OpenAIChoice {
    message: OpenAIMessageResponse,
    finish_reason: String,
}

#[derive(Deserialize)]
struct OpenAIMessageResponse {
    content: String,
}

#[derive(Deserialize)]
struct OpenAIUsage {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32,
}

#[derive(Deserialize)]
struct OpenAIError {
    error: OpenAIErrorDetail,
}

#[derive(Deserialize)]
struct OpenAIErrorDetail {
    message: String,
}

#[async_trait]
impl ModelProvider for OpenAIProvider {
    fn name(&self) -> &str {
        "openai"
    }

    async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse, ModelError> {
        let messages: Vec<OpenAIMessage> = request
            .messages
            .into_iter()
            .map(|m| OpenAIMessage {
                role: m.role,
                content: m.content,
            })
            .collect();

        let api_request = OpenAIRequest {
            model: request.model,
            messages,
            max_tokens: request.max_tokens,
            temperature: request.temperature,
        };

        let response = self
            .client
            .post(format!("{}/chat/completions", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&api_request)
            .send()
            .await
            .map_err(|e| ModelError::NetworkError(e.to_string()))?;

        let status = response.status();
        let body = response
            .text()
            .await
            .map_err(|e| ModelError::NetworkError(e.to_string()))?;

        if !status.is_success() {
            if let Ok(error) = serde_json::from_str::<OpenAIError>(&body) {
                return Err(ModelError::ApiError {
                    status: status.as_u16(),
                    message: error.error.message,
                });
            }
            return Err(ModelError::ApiError {
                status: status.as_u16(),
                message: body,
            });
        }

        let api_response: OpenAIResponse =
            serde_json::from_str(&body).map_err(|e| ModelError::ParseError(e.to_string()))?;

        let choice = api_response
            .choices
            .into_iter()
            .next()
            .ok_or_else(|| ModelError::ParseError("No choices in response".to_string()))?;

        Ok(CompletionResponse {
            content: choice.message.content,
            usage: TokenUsage {
                prompt_tokens: api_response.usage.prompt_tokens,
                completion_tokens: api_response.usage.completion_tokens,
                total_tokens: api_response.usage.total_tokens,
            },
            model: api_response.model,
            finish_reason: choice.finish_reason,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_openai_provider_requires_api_key() {
        // Remove key if set
        std::env::remove_var("OPENAI_API_KEY");

        let result = OpenAIProvider::new();
        assert!(result.is_err());
    }

    #[test]
    fn test_openai_provider_with_endpoint() {
        let provider = OpenAIProvider::with_endpoint(
            "test-key".to_string(),
            "https://custom.api.com".to_string(),
        );

        assert_eq!(provider.name(), "openai");
        assert_eq!(provider.base_url, "https://custom.api.com");
    }
}
