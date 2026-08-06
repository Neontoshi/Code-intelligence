// src/llm/mod.rs

//! LLM (Large Language Model) Integration Module

pub mod code_understanding;
pub mod prompts;
pub mod providers;

// Re-exports
pub use code_understanding::{
    AnalysisScore, ArchitectureAnalysis, ArchitectureLayer, ArchitecturePattern, CodeIssue,
    CodeSuggestion, CodeUnderstandingEngine, DuplicateAnalysisResult,
};
pub use prompts::PromptBuilder;
pub use providers::{
    create_ollama_phi2, create_provider, create_provider_from_string, MockProvider, OllamaProvider,
    OpenAIProvider, ProviderConfig, ProviderType,
};

use async_trait::async_trait;
use futures::Stream;
use serde::{Deserialize, Serialize};
use std::fmt;

// ============================================================================
// Core Types
// ============================================================================

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LLMMessage {
    pub role: MessageRole,
    pub content: String,
}

impl LLMMessage {
    pub fn new(role: MessageRole, content: impl Into<String>) -> Self {
        Self {
            role,
            content: content.into(),
        }
    }

    pub fn system(content: impl Into<String>) -> Self {
        Self::new(MessageRole::System, content)
    }

    pub fn user(content: impl Into<String>) -> Self {
        Self::new(MessageRole::User, content)
    }

    pub fn assistant(content: impl Into<String>) -> Self {
        Self::new(MessageRole::Assistant, content)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MessageRole {
    System,
    User,
    Assistant,
}

impl fmt::Display for MessageRole {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MessageRole::System => write!(f, "system"),
            MessageRole::User => write!(f, "user"),
            MessageRole::Assistant => write!(f, "assistant"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMResponse {
    pub content: String,
    pub model: String,
    pub usage: Option<Usage>,
    pub finish_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Usage {
    pub prompt_tokens: usize,
    pub completion_tokens: usize,
    pub total_tokens: usize,
}

#[derive(Debug, Clone)]
pub struct GenerationOptions {
    pub temperature: f32,
    pub max_tokens: usize,
    pub top_p: f32,
    pub frequency_penalty: f32,
    pub presence_penalty: f32,
    pub stop_sequences: Vec<String>,
    pub seed: Option<u64>,
}

impl Default for GenerationOptions {
    fn default() -> Self {
        Self {
            temperature: 0.7,
            max_tokens: 1000,
            top_p: 0.95,
            frequency_penalty: 0.0,
            presence_penalty: 0.0,
            stop_sequences: Vec::new(),
            seed: None,
        }
    }
}

impl GenerationOptions {
    pub fn with_temperature(mut self, temperature: f32) -> Self {
        self.temperature = temperature;
        self
    }

    pub fn with_max_tokens(mut self, max_tokens: usize) -> Self {
        self.max_tokens = max_tokens;
        self
    }

    pub fn with_top_p(mut self, top_p: f32) -> Self {
        self.top_p = top_p;
        self
    }
}

// ============================================================================
// Provider Trait
// ============================================================================

#[async_trait]
pub trait LLMProvider: Send + Sync {
    async fn generate(
        &self,
        messages: &[LLMMessage],
        options: &GenerationOptions,
    ) -> std::result::Result<LLMResponse, String>;

    async fn generate_stream(
        &self,
        messages: &[LLMMessage],
        options: &GenerationOptions,
    ) -> std::result::Result<
        Box<dyn Stream<Item = std::result::Result<String, String>> + Send>,
        String,
    >;

    fn model_name(&self) -> &str;
    fn max_context_length(&self) -> usize;
    async fn is_available(&self) -> bool {
        true
    }
}

// ============================================================================
// Error Types
// ============================================================================

#[derive(Debug, thiserror::Error)]
pub enum LLMError {
    #[error("Provider not available: {0}")]
    ProviderUnavailable(String),

    #[error("API error: {0}")]
    ApiError(String),

    #[error("Rate limit exceeded")]
    RateLimitExceeded,

    #[error("Context length exceeded (max: {max}, requested: {requested})")]
    ContextLengthExceeded { max: usize, requested: usize },

    #[error("Invalid response: {0}")]
    InvalidResponse(String),

    #[error("Timeout")]
    Timeout,

    #[error("Configuration error: {0}")]
    ConfigError(String),
}

pub type Result<T> = std::result::Result<T, LLMError>;

impl From<String> for LLMError {
    fn from(s: String) -> Self {
        LLMError::ApiError(s)
    }
}

impl From<LLMError> for String {
    fn from(e: LLMError) -> Self {
        e.to_string()
    }
}

// ============================================================================
// Utility Functions
// ============================================================================

pub fn extract_json_from_response(response: &str) -> Result<serde_json::Value> {
    use regex::Regex;

    let patterns = [
        r"```json\s*([\s\S]*?)\s*```",
        r"```\s*([\s\S]*?)\s*```",
        r"\{[\s\S]*\}",
    ];

    if let Ok(re) = Regex::new(patterns[0]) {
        if let Some(caps) = re.captures(response) {
            if let Some(json_str) = caps.get(1) {
                if let Ok(json) = serde_json::from_str(json_str.as_str()) {
                    return Ok(json);
                }
            }
        }
    }

    if let Ok(re) = Regex::new(patterns[1]) {
        if let Some(caps) = re.captures(response) {
            if let Some(json_str) = caps.get(1) {
                if let Ok(json) = serde_json::from_str(json_str.as_str()) {
                    return Ok(json);
                }
            }
        }
    }

    if let Ok(re) = Regex::new(patterns[2]) {
        if let Some(matched) = re.find(response) {
            if let Ok(json) = serde_json::from_str(matched.as_str()) {
                return Ok(json);
            }
        }
    }

    if let Ok(json) = serde_json::from_str(response) {
        return Ok(json);
    }

    Err(LLMError::InvalidResponse(
        "No valid JSON found in response".to_string(),
    ))
}

pub fn estimate_tokens(text: &str) -> usize {
    let code_chars = text
        .chars()
        .filter(|c| c.is_ascii_punctuation() || c.is_ascii_digit())
        .count();
    let text_chars = text
        .chars()
        .filter(|c| c.is_alphabetic() || c.is_whitespace())
        .count();
    (code_chars / 3) + (text_chars / 4)
}

pub fn truncate_to_token_limit(text: &str, max_tokens: usize) -> String {
    let current_tokens = estimate_tokens(text);
    if current_tokens <= max_tokens {
        return text.to_string();
    }

    let chars_per_token = 4;
    let max_chars = max_tokens * chars_per_token;
    if text.len() <= max_chars {
        return text.to_string();
    }

    let truncated = &text[..max_chars];
    if let Some(last_newline) = truncated.rfind('\n') {
        format!("{}\n... (truncated)", &truncated[..last_newline])
    } else {
        format!("{}... (truncated)", truncated)
    }
}

/// Convert MessageRole to provider-agnostic string
pub fn role_to_string(role: &MessageRole) -> String {
    match role {
        MessageRole::System => "system".to_string(),
        MessageRole::User => "user".to_string(),
        MessageRole::Assistant => "assistant".to_string(),
    }
}
