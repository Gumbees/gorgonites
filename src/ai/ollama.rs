//! Ollama LLM client
//!
//! Dedicated client for local/remote Ollama instances.

use serde::{Deserialize, Serialize};
use std::time::Duration;
use crate::config::OllamaConfig;

/// Errors from Ollama operations
#[derive(Debug, thiserror::Error)]
pub enum OllamaError {
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),

    #[error("Request failed: {0}")]
    RequestFailed(String),

    #[error("Invalid response: {0}")]
    InvalidResponse(String),

    #[error("Ollama is disabled")]
    Disabled,

    #[error("Model not found: {0}")]
    ModelNotFound(String),
}

/// Message role for chat API
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    System,
    User,
    Assistant,
}

/// A chat message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: Role,
    pub content: String,
}

impl ChatMessage {
    pub fn system(content: impl Into<String>) -> Self {
        Self {
            role: Role::System,
            content: content.into(),
        }
    }

    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: Role::User,
            content: content.into(),
        }
    }

    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            role: Role::Assistant,
            content: content.into(),
        }
    }
}

/// Ollama client for LLM interactions
pub struct OllamaClient {
    config: OllamaConfig,
    http_client: reqwest::Client,
    base_url: String,
}

impl OllamaClient {
    /// Create a new Ollama client from config
    pub fn new(config: &OllamaConfig) -> Result<Self, OllamaError> {
        let http_client = reqwest::Client::builder()
            .timeout(Duration::from_secs(config.timeout as u64))
            .build()
            .map_err(|e| OllamaError::ConnectionFailed(e.to_string()))?;

        let base_url = format!("http://{}:{}", config.host, config.port);

        Ok(Self {
            config: config.clone(),
            http_client,
            base_url,
        })
    }

    /// Check if the Ollama service is available
    pub async fn health_check(&self) -> Result<bool, OllamaError> {
        if !self.config.enabled {
            return Err(OllamaError::Disabled);
        }

        let url = format!("{}/api/tags", self.base_url);

        match self.http_client.get(&url).send().await {
            Ok(response) => Ok(response.status().is_success()),
            Err(e) => {
                tracing::warn!("Ollama health check failed: {}", e);
                Ok(false)
            }
        }
    }

    /// List available models
    pub async fn list_models(&self) -> Result<Vec<String>, OllamaError> {
        if !self.config.enabled {
            return Err(OllamaError::Disabled);
        }

        #[derive(Deserialize)]
        struct TagsResponse {
            models: Vec<ModelInfo>,
        }

        #[derive(Deserialize)]
        struct ModelInfo {
            name: String,
        }

        let url = format!("{}/api/tags", self.base_url);

        let response = self
            .http_client
            .get(&url)
            .send()
            .await
            .map_err(|e| OllamaError::ConnectionFailed(e.to_string()))?;

        if !response.status().is_success() {
            return Err(OllamaError::RequestFailed(format!(
                "Status: {}",
                response.status()
            )));
        }

        let tags: TagsResponse = response
            .json()
            .await
            .map_err(|e| OllamaError::InvalidResponse(e.to_string()))?;

        Ok(tags.models.into_iter().map(|m| m.name).collect())
    }

    /// Generate a completion using the generate API
    pub async fn generate(&self, prompt: &str, system: Option<&str>) -> Result<String, OllamaError> {
        if !self.config.enabled {
            return Err(OllamaError::Disabled);
        }

        #[derive(Serialize)]
        struct GenerateRequest {
            model: String,
            prompt: String,
            #[serde(skip_serializing_if = "Option::is_none")]
            system: Option<String>,
            stream: bool,
        }

        #[derive(Deserialize)]
        struct GenerateResponse {
            response: String,
        }

        let url = format!("{}/api/generate", self.base_url);

        let request = GenerateRequest {
            model: self.config.model.clone(),
            prompt: prompt.to_string(),
            system: system.map(|s| s.to_string()),
            stream: false,
        };

        let mut req = self.http_client.post(&url).json(&request);

        // Add API key header if configured
        if let Some(ref api_key) = self.config.api_key {
            req = req.header("Authorization", format!("Bearer {}", api_key));
        }

        let response = req
            .send()
            .await
            .map_err(|e| OllamaError::ConnectionFailed(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(OllamaError::RequestFailed(format!(
                "Status {}: {}",
                status, body
            )));
        }

        let parsed: GenerateResponse = response
            .json()
            .await
            .map_err(|e| OllamaError::InvalidResponse(e.to_string()))?;

        Ok(parsed.response)
    }

    /// Chat with the model using the chat API
    pub async fn chat(&self, messages: &[ChatMessage]) -> Result<String, OllamaError> {
        if !self.config.enabled {
            return Err(OllamaError::Disabled);
        }

        #[derive(Serialize)]
        struct ChatRequest {
            model: String,
            messages: Vec<ChatMessage>,
            stream: bool,
        }

        #[derive(Deserialize)]
        struct ChatResponse {
            message: ChatMessage,
        }

        let url = format!("{}/api/chat", self.base_url);

        let request = ChatRequest {
            model: self.config.model.clone(),
            messages: messages.to_vec(),
            stream: false,
        };

        let mut req = self.http_client.post(&url).json(&request);

        // Add API key header if configured
        if let Some(ref api_key) = self.config.api_key {
            req = req.header("Authorization", format!("Bearer {}", api_key));
        }

        let response = req
            .send()
            .await
            .map_err(|e| OllamaError::ConnectionFailed(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(OllamaError::RequestFailed(format!(
                "Status {}: {}",
                status, body
            )));
        }

        let parsed: ChatResponse = response
            .json()
            .await
            .map_err(|e| OllamaError::InvalidResponse(e.to_string()))?;

        Ok(parsed.message.content)
    }

    /// Get the configured model name
    pub fn model(&self) -> &str {
        &self.config.model
    }

    /// Check if the client is enabled
    pub fn is_enabled(&self) -> bool {
        self.config.enabled
    }

    /// Get the base URL
    pub fn base_url(&self) -> &str {
        &self.base_url
    }
}
