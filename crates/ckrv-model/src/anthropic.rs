//! Anthropic Claude model provider implementation.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::{
    provider::{CompletionRequest, CompletionResponse, ModelProvider},
    ModelError, TokenUsage,
};

/// Anthropic API provider.
pub struct AnthropicProvider {
    client: reqwest::Client,
    api_key: String,
    base_url: String,
}

impl AnthropicProvider {
    /// Create a new Anthropic provider.
    ///
    /// # Errors
    ///
    /// Returns an error if the API key is not set.
    pub fn new() -> Result<Self, ModelError> {
        let api_key = std::env::var("ANTHROPIC_API_KEY")
            .map_err(|_| ModelError::ConfigError("ANTHROPIC_API_KEY not set".to_string()))?;

        Ok(Self {
            client: reqwest::Client::new(),
            api_key,
            base_url: "https://api.anthropic.com/v1".to_string(),
        })
    }

    /// Create with custom endpoint.
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
struct AnthropicRequest {
    model: String,
    messages: Vec<AnthropicMessage>,
    max_tokens: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    system: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
}

#[derive(Serialize)]
struct AnthropicMessage {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct AnthropicResponse {
    content: Vec<AnthropicContent>,
    usage: AnthropicUsage,
    model: String,
    stop_reason: String,
}

#[derive(Deserialize)]
struct AnthropicContent {
    #[serde(rename = "type")]
    content_type: String,
    text: String,
}

#[derive(Deserialize)]
struct AnthropicUsage {
    input_tokens: u32,
    output_tokens: u32,
}

#[derive(Deserialize)]
struct AnthropicError {
    error: AnthropicErrorDetail,
}

#[derive(Deserialize)]
struct AnthropicErrorDetail {
    message: String,
}

#[async_trait]
impl ModelProvider for AnthropicProvider {
    fn name(&self) -> &str {
        "anthropic"
    }

    async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse, ModelError> {
        // Extract system message if present
        let mut system_message: Option<String> = None;
        let messages: Vec<AnthropicMessage> = request
            .messages
            .into_iter()
            .filter_map(|m| {
                if m.role == "system" {
                    system_message = Some(m.content);
                    None
                } else {
                    Some(AnthropicMessage {
                        role: m.role,
                        content: m.content,
                    })
                }
            })
            .collect();

        let api_request = AnthropicRequest {
            model: request.model,
            messages,
            max_tokens: request.max_tokens.unwrap_or(4096),
            system: system_message,
            temperature: request.temperature,
        };

        let response = self
            .client
            .post(format!("{}/messages", self.base_url))
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2024-01-01")
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
            if let Ok(error) = serde_json::from_str::<AnthropicError>(&body) {
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

        let api_response: AnthropicResponse =
            serde_json::from_str(&body).map_err(|e| ModelError::ParseError(e.to_string()))?;

        let content = api_response
            .content
            .into_iter()
            .filter(|c| c.content_type == "text")
            .map(|c| c.text)
            .collect::<Vec<_>>()
            .join("");

        let total_tokens = api_response.usage.input_tokens + api_response.usage.output_tokens;

        Ok(CompletionResponse {
            content,
            usage: TokenUsage {
                prompt_tokens: api_response.usage.input_tokens,
                completion_tokens: api_response.usage.output_tokens,
                total_tokens,
            },
            model: api_response.model,
            finish_reason: api_response.stop_reason,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_anthropic_provider_requires_api_key() {
        std::env::remove_var("ANTHROPIC_API_KEY");

        let result = AnthropicProvider::new();
        assert!(result.is_err());
    }

    #[test]
    fn test_anthropic_provider_with_endpoint() {
        let provider = AnthropicProvider::with_endpoint(
            "test-key".to_string(),
            "https://custom.api.com".to_string(),
        );

        assert_eq!(provider.name(), "anthropic");
    }
}
