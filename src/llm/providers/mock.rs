// src/llm/providers/mock.rs

//! Mock LLM Provider for Testing
//!
//! This module provides a mock implementation of the LLMProvider trait
//! for use in tests and development without requiring an actual LLM.

use crate::llm::{GenerationOptions, LLMMessage, LLMProvider, LLMResponse, Usage};
use async_trait::async_trait;
use futures::stream::{self, Stream, StreamExt};
use std::collections::HashMap;
use std::sync::Mutex;

// Mock Provider

/// A mock LLM provider for testing
pub struct MockProvider {
    responses: Mutex<HashMap<String, String>>,
    default_response: String,
    delay_ms: u64,
    should_fail: bool,
}

impl MockProvider {
    pub fn new() -> Self {
        Self {
            responses: Mutex::new(HashMap::new()),
            default_response: "This is a mock response from the LLM.".to_string(),
            delay_ms: 0,
            should_fail: false,
        }
    }

    /// Set a custom response for a specific prompt
    pub fn with_response(self, prompt: &str, response: &str) -> Self {
        self.responses
            .lock()
            .unwrap()
            .insert(prompt.to_string(), response.to_string());
        self
    }

    /// Set the default response for any prompt not matched
    pub fn with_default_response(mut self, response: &str) -> Self {
        self.default_response = response.to_string();
        self
    }

    /// Add a delay to simulate LLM latency
    pub fn with_delay(mut self, delay_ms: u64) -> Self {
        self.delay_ms = delay_ms;
        self
    }

    /// Make the provider fail on requests
    pub fn with_failure(mut self, should_fail: bool) -> Self {
        self.should_fail = should_fail;
        self
    }

    /// Find the best matching response for a prompt
    fn get_response(&self, prompt: &str) -> String {
        // This lock should never fail in single-threaded tests
        let responses = self
            .responses
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());

        // Try exact match first
        if let Some(response) = responses.get(prompt) {
            return response.clone();
        }

        // Try partial match
        for (key, response) in responses.iter() {
            if prompt.contains(key) || key.contains(prompt) {
                return response.clone();
            }
        }

        // Return default
        self.default_response.clone()
    }

    /// Generate a mock response for duplicate analysis
    pub fn mock_duplicate_response(is_duplicate: bool, confidence: f64) -> String {
        format!(
            r#"{{
    "is_duplicate": {},
    "confidence": {},
    "similarity_score": {},
    "reasoning": "These functions {} duplicates because they have similar structure and purpose.",
    "key_differences": ["Different naming", "Slightly different implementation"],
    "refactoring_suggestion": "Extract common logic into a shared function",
    "refactoring_effort": "medium"
}}"#,
            is_duplicate,
            confidence,
            confidence * 0.9,
            if is_duplicate { "are" } else { "are not" }
        )
    }

    /// Generate a mock response for function summarization
    pub fn mock_summary_response(function_name: &str) -> String {
        format!(
            "This function `{}` performs a specific operation and returns a result.",
            function_name
        )
    }

    /// Generate a mock response for bug analysis
    pub fn mock_bug_response() -> String {
        r#"{
    "issues": [
        {
            "severity": "medium",
            "category": "error_handling",
            "description": "No error handling for edge cases",
            "suggestion": "Add proper error handling with Result type",
            "line": "unknown"
        }
    ]
}"#
        .to_string()
    }

    /// Generate a mock response for improvement suggestions
    pub fn mock_improvement_response() -> String {
        r#"{
    "suggestions": [
        {
            "type": "performance",
            "description": "Use iterative approach instead of recursion",
            "reason": "Avoids stack overflow for large inputs",
            "example": "fn iterative_process(data: &[T]) -> Result<T>"
        }
    ]
}"#
        .to_string()
    }
}

impl Default for MockProvider {
    fn default() -> Self {
        Self::new()
    }
}

// LLMProvider Implementation

#[async_trait]
impl LLMProvider for MockProvider {
    async fn generate(
        &self,
        messages: &[LLMMessage],
        _options: &GenerationOptions,
    ) -> crate::error::Result<LLMResponse> {
        // Simulate failure if configured
        if self.should_fail {
            return Err(anyhow::anyhow!("Mock provider failed as configured"));
        }

        // Simulate delay
        if self.delay_ms > 0 {
            tokio::time::sleep(std::time::Duration::from_millis(self.delay_ms)).await;
        }

        // Extract the prompt from messages
        let prompt: String = messages
            .iter()
            .map(|msg| msg.content.clone())
            .collect::<Vec<_>>()
            .join("\n");

        // Get response
        let content = self.get_response(&prompt);

        // Simulate token usage
        let prompt_tokens = prompt.len() / 4;
        let completion_tokens = content.len() / 4;

        Ok(LLMResponse {
            content,
            model: "mock".to_string(),
            usage: Some(Usage {
                prompt_tokens,
                completion_tokens,
                total_tokens: prompt_tokens + completion_tokens,
            }),
            finish_reason: Some("stop".to_string()),
        })
    }

    async fn generate_stream(
        &self,
        messages: &[LLMMessage],
        _options: &GenerationOptions,
    ) -> crate::error::Result<Box<dyn Stream<Item = crate::error::Result<String>> + Send>> {
        // Simulate failure if configured
        if self.should_fail {
            return Err(anyhow::anyhow!("Mock provider failed as configured"));
        }

        // Simulate delay
        if self.delay_ms > 0 {
            tokio::time::sleep(std::time::Duration::from_millis(self.delay_ms)).await;
        }

        // Extract the prompt
        let prompt: String = messages
            .iter()
            .map(|msg| msg.content.clone())
            .collect::<Vec<_>>()
            .join("\n");

        let response = self.get_response(&prompt);

        // Split into tokens (simulate streaming)
        let tokens: Vec<String> = response
            .split_whitespace()
            .map(|s| format!("{} ", s))
            .collect();

        let stream = stream::iter(tokens).map(Ok);
        Ok(Box::new(stream))
    }

    async fn is_available(&self) -> bool {
        !self.should_fail
    }
}

// Pre-configured Mock Providers

impl MockProvider {
    /// Create a mock provider with predefined responses for code analysis
    pub fn code_analysis_mock() -> Self {
        let mut provider = MockProvider::new();

        // Add code analysis responses
        provider = provider
            .with_response(
                "Summarize this function",
                "This function performs a specific operation and returns the result.",
            )
            .with_response(
                "Explain this function in detail",
                "This function implements the core logic of the module. It takes input parameters, processes them through a series of steps, and returns the output.",
            )
            .with_response(
                "Analyze this function for potential bugs",
                &MockProvider::mock_bug_response(),
            )
            .with_response(
                "Suggest improvements for this function",
                &MockProvider::mock_improvement_response(),
            )
            .with_response(
                "Analyze if these two functions are duplicates",
                &MockProvider::mock_duplicate_response(true, 0.95),
            );

        provider
    }

    /// Create a mock provider for duplicate analysis
    pub fn duplicate_analysis_mock(is_duplicate: bool, confidence: f64) -> Self {
        let mut provider = MockProvider::new();
        provider = provider.with_response(
            "Analyze if these two functions are duplicates",
            &MockProvider::mock_duplicate_response(is_duplicate, confidence),
        );
        provider
    }
}

// Tests

#[cfg(test)]
mod tests {
    use super::*;
    use crate::llm::GenerationOptions;

    #[tokio::test]
    async fn test_mock_provider_generate() {
        let provider = MockProvider::new();

        let messages = vec![
            LLMMessage::system("You are a helpful assistant."),
            LLMMessage::user("What is 2+2?"),
        ];

        let response = provider
            .generate(&messages, &GenerationOptions::default())
            .await;
        assert!(response.is_ok());

        let resp = response.expect("Mock response should be available");
        assert!(!resp.content.is_empty());
        assert_eq!(resp.model, "mock");
        assert!(resp.usage.is_some());
    }

    #[tokio::test]
    async fn test_mock_provider_with_custom_response() {
        let provider = MockProvider::new().with_response("test prompt", "custom response");

        let messages = vec![LLMMessage::user("test prompt")];

        let response = provider
            .generate(&messages, &GenerationOptions::default())
            .await;
        assert!(response.is_ok());
        assert_eq!(response.unwrap().content, "custom response");
    }

    #[tokio::test]
    async fn test_mock_provider_with_delay() {
        let provider = MockProvider::new().with_delay(10);

        let messages = vec![LLMMessage::user("test")];

        let start = std::time::Instant::now();
        let _response = provider
            .generate(&messages, &GenerationOptions::default())
            .await;
        let duration = start.elapsed();

        assert!(duration.as_millis() >= 10);
    }

    #[tokio::test]
    async fn test_mock_provider_failure() {
        let provider = MockProvider::new().with_failure(true);

        let messages = vec![LLMMessage::user("test")];

        let response = provider
            .generate(&messages, &GenerationOptions::default())
            .await;
        assert!(response.is_err());
    }

    #[tokio::test]
    async fn test_mock_duplicate_analysis() {
        let provider = MockProvider::duplicate_analysis_mock(true, 0.95);

        let messages = vec![LLMMessage::user(
            "Analyze if these two functions are duplicates",
        )];

        let response = provider
            .generate(&messages, &GenerationOptions::default())
            .await;
        assert!(response.is_ok());

        let content = response.unwrap().content;
        // Check if it contains JSON-like structure
        assert!(content.contains("is_duplicate"));
        assert!(content.contains("true"));
    }

    #[tokio::test]
    async fn test_mock_code_analysis() {
        let provider = MockProvider::code_analysis_mock();

        // Test summarization
        let messages = vec![LLMMessage::user("Summarize this function")];

        let response = provider
            .generate(&messages, &GenerationOptions::default())
            .await;
        assert!(response.is_ok());
        assert!(response.unwrap().content.contains("function"));
    }
}
