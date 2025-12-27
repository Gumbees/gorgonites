//! LLM API client

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Configuration for AI client
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiClientConfig {
    /// API provider
    pub provider: AiProvider,

    /// API endpoint (can be local)
    pub endpoint: String,

    /// API key (from environment)
    #[serde(skip)]
    pub api_key: String,

    /// Model identifier
    pub model: String,

    /// Request timeout in seconds
    pub timeout_secs: u64,

    /// Max retries on failure
    pub max_retries: u32,
}

/// Supported AI providers
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AiProvider {
    /// Anthropic Claude API
    Anthropic,
    /// OpenAI API
    OpenAI,
    /// Local Ollama instance
    Ollama,
    /// Custom/compatible API
    Custom,
}

impl Default for AiClientConfig {
    fn default() -> Self {
        Self {
            provider: AiProvider::Anthropic,
            endpoint: "https://api.anthropic.com/v1/messages".to_string(),
            api_key: String::new(),
            model: "claude-sonnet-4-20250514".to_string(),
            timeout_secs: 30,
            max_retries: 3,
        }
    }
}

/// The AI client for making requests
pub struct AiClient {
    config: AiClientConfig,
    http_client: reqwest::Client,
}

impl AiClient {
    pub fn new(config: AiClientConfig) -> Self {
        let http_client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(config.timeout_secs))
            .build()
            .expect("Failed to create HTTP client");

        Self { config, http_client }
    }

    /// Load API key from environment
    pub fn load_api_key(&mut self) {
        let env_var = match self.config.provider {
            AiProvider::Anthropic => "ANTHROPIC_API_KEY",
            AiProvider::OpenAI => "OPENAI_API_KEY",
            AiProvider::Ollama => return, // No key needed
            AiProvider::Custom => "AI_API_KEY",
        };

        self.config.api_key = std::env::var(env_var).unwrap_or_default();
    }

    /// Send a message and get a response
    pub async fn complete(&self, prompt: &str, system: &str) -> Result<String, AiError> {
        match self.config.provider {
            AiProvider::Anthropic => self.complete_anthropic(prompt, system).await,
            AiProvider::OpenAI => self.complete_openai(prompt, system).await,
            AiProvider::Ollama => self.complete_ollama(prompt, system).await,
            AiProvider::Custom => self.complete_custom(prompt, system).await,
        }
    }

    async fn complete_anthropic(&self, prompt: &str, system: &str) -> Result<String, AiError> {
        #[derive(Serialize)]
        struct AnthropicRequest {
            model: String,
            max_tokens: u32,
            system: String,
            messages: Vec<AnthropicMessage>,
        }

        #[derive(Serialize)]
        struct AnthropicMessage {
            role: String,
            content: String,
        }

        #[derive(Deserialize)]
        struct AnthropicResponse {
            content: Vec<ContentBlock>,
        }

        #[derive(Deserialize)]
        struct ContentBlock {
            text: String,
        }

        let request = AnthropicRequest {
            model: self.config.model.clone(),
            max_tokens: 1024,
            system: system.to_string(),
            messages: vec![AnthropicMessage {
                role: "user".to_string(),
                content: prompt.to_string(),
            }],
        };

        let response = self
            .http_client
            .post(&self.config.endpoint)
            .header("x-api-key", &self.config.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| AiError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(AiError::ApiError(format!(
                "Status {}: {}",
                response.status(),
                response.text().await.unwrap_or_default()
            )));
        }

        let parsed: AnthropicResponse = response
            .json()
            .await
            .map_err(|e| AiError::ParseError(e.to_string()))?;

        parsed
            .content
            .first()
            .map(|c| c.text.clone())
            .ok_or_else(|| AiError::ParseError("Empty response".to_string()))
    }

    async fn complete_openai(&self, _prompt: &str, _system: &str) -> Result<String, AiError> {
        // TODO: Implement OpenAI API
        Err(AiError::NotImplemented("OpenAI provider".to_string()))
    }

    async fn complete_ollama(&self, prompt: &str, system: &str) -> Result<String, AiError> {
        #[derive(Serialize)]
        struct OllamaRequest {
            model: String,
            prompt: String,
            system: String,
            stream: bool,
        }

        #[derive(Deserialize)]
        struct OllamaResponse {
            response: String,
        }

        let request = OllamaRequest {
            model: self.config.model.clone(),
            prompt: prompt.to_string(),
            system: system.to_string(),
            stream: false,
        };

        let response = self
            .http_client
            .post(&self.config.endpoint)
            .header("content-type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| AiError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(AiError::ApiError(format!(
                "Status {}: {}",
                response.status(),
                response.text().await.unwrap_or_default()
            )));
        }

        let parsed: OllamaResponse = response
            .json()
            .await
            .map_err(|e| AiError::ParseError(e.to_string()))?;

        Ok(parsed.response)
    }

    async fn complete_custom(&self, _prompt: &str, _system: &str) -> Result<String, AiError> {
        // TODO: Implement custom API
        Err(AiError::NotImplemented("Custom provider".to_string()))
    }
}

/// Errors from AI operations
#[derive(Debug, Error)]
pub enum AiError {
    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("API error: {0}")]
    ApiError(String),

    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("Rate limited")]
    RateLimited,

    #[error("Not implemented: {0}")]
    NotImplemented(String),

    #[error("No API key configured")]
    NoApiKey,
}
