// src/llm/providers/mod.rs

//! LLM Provider Implementations

pub mod anthropic;
pub mod mock;
pub mod ollama;
pub mod openai;

pub use anthropic::AnthropicProvider;
pub use mock::MockProvider;
pub use ollama::OllamaProvider;
pub use openai::OpenAIProvider;

use crate::llm::{LLMError, LLMProvider, Result};
use std::sync::Arc;

// ============================================================================
// Provider Types
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ProviderType {
    Ollama,
    OpenAI,
    Anthropic,
    Mock,
}

impl ProviderType {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "ollama" => Some(Self::Ollama),
            "openai" => Some(Self::OpenAI),
            "anthropic" => Some(Self::Anthropic),
            "mock" => Some(Self::Mock),
            _ => None,
        }
    }
}

impl std::fmt::Display for ProviderType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProviderType::Ollama => write!(f, "ollama"),
            ProviderType::OpenAI => write!(f, "openai"),
            ProviderType::Anthropic => write!(f, "anthropic"),
            ProviderType::Mock => write!(f, "mock"),
        }
    }
}

// ============================================================================
// Provider Configuration
// ============================================================================

#[derive(Debug, Clone)]
pub struct ProviderConfig {
    pub api_key: Option<String>,
    pub base_url: Option<String>,
    pub model: String,
    pub timeout_seconds: u64,
    pub max_retries: u32,
    pub extra_headers: Vec<(String, String)>,
}

impl ProviderConfig {
    pub fn new(model: impl Into<String>) -> Self {
        Self {
            api_key: None,
            base_url: None,
            model: model.into(),
            timeout_seconds: 30,
            max_retries: 3,
            extra_headers: Vec::new(),
        }
    }

    pub fn with_api_key(mut self, api_key: impl Into<String>) -> Self {
        self.api_key = Some(api_key.into());
        self
    }

    pub fn with_base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = Some(base_url.into());
        self
    }
}

impl Default for ProviderConfig {
    fn default() -> Self {
        Self {
            api_key: None,
            base_url: None,
            model: "phi:2.7b".to_string(),
            timeout_seconds: 30,
            max_retries: 3,
            extra_headers: Vec::new(),
        }
    }
}

// ============================================================================
// Factory Functions
// ============================================================================

pub async fn create_provider(
    provider_type: ProviderType,
    config: &ProviderConfig,
) -> Result<Arc<dyn LLMProvider>> {
    match provider_type {
        ProviderType::Ollama => {
            let provider = OllamaProvider::new(config)
                .await
                .map_err(|e| LLMError::ConfigError(e.to_string()))?;
            Ok(Arc::new(provider))
        }
        ProviderType::OpenAI => {
            let provider = OpenAIProvider::new(config)
                .await
                .map_err(|e| LLMError::ConfigError(e.to_string()))?;
            Ok(Arc::new(provider))
        }
        ProviderType::Anthropic => {
            let provider =
                AnthropicProvider::new(config).map_err(|e| LLMError::ConfigError(e.to_string()))?;
            Ok(Arc::new(provider))
        }
        ProviderType::Mock => Ok(Arc::new(MockProvider::new())),
    }
}

pub async fn create_ollama_phi2() -> Result<Arc<dyn LLMProvider>> {
    let config = ProviderConfig {
        model: "phi:2.7b".to_string(),
        base_url: Some("http://localhost:11434".to_string()),
        ..Default::default()
    };
    create_provider(ProviderType::Ollama, &config).await
}

pub async fn create_provider_from_string(s: &str) -> Result<Arc<dyn LLMProvider>> {
    let mut config = ProviderConfig::default();

    let parts: Vec<&str> = s.split('@').collect();
    let provider_part = parts[0];
    let base_url = parts.get(1).map(|s| s.to_string());

    let provider_parts: Vec<&str> = provider_part.split(':').collect();
    let provider_type = if provider_parts.len() >= 2 {
        let type_str = provider_parts[0];
        let model = provider_parts[1..].join(":");
        config.model = model;
        ProviderType::from_str(type_str)
            .ok_or_else(|| LLMError::ConfigError(format!("Unknown provider: {}", type_str)))?
    } else if provider_parts.len() == 1 {
        config.model = "phi:2.7b".to_string();
        ProviderType::Ollama
    } else {
        return Err(LLMError::ConfigError("Invalid provider string".to_string()));
    };

    if let Some(url) = base_url {
        config.base_url = Some(url);
    }

    if matches!(provider_type, ProviderType::Ollama) && config.base_url.is_none() {
        config.base_url = Some("http://localhost:11434".to_string());
    }

    create_provider(provider_type, &config).await
}
