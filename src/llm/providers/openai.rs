// src/llm/providers/openai.rs

//! OpenAI LLM Provider
//!
//! This module provides an implementation of the LLMProvider trait for
//! OpenAI's API, supporting GPT-3.5, GPT-4, and other OpenAI models.

use crate::llm::providers::ProviderConfig;
use crate::llm::{GenerationOptions, LLMMessage, LLMProvider, LLMResponse, Usage};

use async_trait::async_trait;
use futures::stream::{Stream, StreamExt};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

// OpenAI API Types

#[derive(Debug, Serialize)]
struct OpenAIChatRequest {
    model: String,
    messages: Vec<OpenAIMessage>,
    temperature: f32,
    top_p: f32,
    max_tokens: usize,
    frequency_penalty: f32,
    presence_penalty: f32,
    stream: bool,
    stop: Option<Vec<String>>,
    seed: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize)]
struct OpenAIMessage {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct OpenAIChatResponse {
    id: String,
    object: String,
    created: u64,
    model: String,
    choices: Vec<OpenAIChoice>,
    usage: Option<OpenAIUsage>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct OpenAIChoice {
    index: usize,
    message: OpenAIMessage,
    finish_reason: String,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct OpenAIUsage {
    prompt_tokens: usize,
    completion_tokens: usize,
    total_tokens: usize,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct OpenAIStreamResponse {
    id: String,
    object: String,
    created: u64,
    model: String,
    choices: Vec<OpenAIStreamChoice>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct OpenAIStreamChoice {
    index: usize,
    delta: OpenAIMessageDelta,
    finish_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct OpenAIMessageDelta {
    content: Option<String>,
    role: Option<String>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct OpenAIModelsResponse {
    data: Vec<OpenAIModel>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct OpenAIModel {
    id: String,
    object: String,
    created: u64,
    owned_by: String,
}

// OpenAI Provider

pub struct OpenAIProvider {
    client: Client,
    api_key: String,
    base_url: String,
    model: String,
    config: ProviderConfig,
    available: bool,
}

impl OpenAIProvider {
    pub async fn new(config: &ProviderConfig) -> std::result::Result<Self, String> {
        let api_key = config
            .api_key
            .clone()
            .ok_or_else(|| "OpenAI API key is required".to_string())?;

        let base_url = config
            .base_url
            .clone()
            .unwrap_or_else(|| "https://api.openai.com/v1".to_string());

        // Build HTTP client
        let client = Client::builder()
            .timeout(Duration::from_secs(config.timeout_seconds))
            .build()
            .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

        let provider = Self {
            client,
            api_key,
            base_url,
            model: config.model.clone(),
            config: config.clone(),
            available: true,
        };

        Ok(provider)
    }

    /// Check if provider is available
    pub fn is_available(&self) -> bool {
        self.available
    }
}

// LLMProvider Implementation

#[async_trait]
impl LLMProvider for OpenAIProvider {
    async fn generate(
        &self,
        messages: &[LLMMessage],
        options: &GenerationOptions,
    ) -> std::result::Result<LLMResponse, String> {
        // Check availability
        if !self.available {
            return Err("OpenAI is not available".to_string());
        }

        // Convert messages
        let openai_messages: Vec<OpenAIMessage> = messages
            .iter()
            .map(|msg| OpenAIMessage {
                role: crate::llm::role_to_string(&msg.role),
                content: msg.content.clone(),
            })
            .collect();

        // Build request
        let request = OpenAIChatRequest {
            model: self.model.clone(),
            messages: openai_messages,
            temperature: options.temperature,
            top_p: options.top_p,
            max_tokens: options.max_tokens,
            frequency_penalty: options.frequency_penalty,
            presence_penalty: options.presence_penalty,
            stream: false,
            stop: if options.stop_sequences.is_empty() {
                None
            } else {
                Some(options.stop_sequences.clone())
            },
            seed: options.seed,
        };

        let url = format!("{}/chat/completions", self.base_url);

        // Send request with retries
        let mut last_error = None;
        for attempt in 0..self.config.max_retries {
            match self
                .client
                .post(&url)
                .header("Authorization", format!("Bearer {}", self.api_key))
                .header("Content-Type", "application/json")
                .json(&request)
                .send()
                .await
            {
                Ok(response) => {
                    if response.status().is_success() {
                        // Parse response
                        match response.json::<OpenAIChatResponse>().await {
                            Ok(openai_response) => {
                                let choice = openai_response
                                    .choices
                                    .first()
                                    .ok_or_else(|| "No choices in response".to_string())?;

                                return Ok(LLMResponse {
                                    content: choice.message.content.clone(),
                                    model: openai_response.model,
                                    usage: openai_response.usage.map(|u| Usage {
                                        prompt_tokens: u.prompt_tokens,
                                        completion_tokens: u.completion_tokens,
                                        total_tokens: u.total_tokens,
                                    }),
                                    finish_reason: Some(choice.finish_reason.clone()),
                                });
                            }
                            Err(e) => {
                                return Err(format!("Failed to parse: {}", e));
                            }
                        }
                    } else {
                        let status = response.status();
                        let text = response.text().await.unwrap_or_default();
                        last_error = Some(format!("OpenAI API error {}: {}", status, text));

                        // Don't retry on certain errors
                        if status == 401 || status == 403 {
                            break;
                        }
                    }
                }
                Err(e) => {
                    last_error = Some(format!("Request failed: {}", e));
                    if attempt < self.config.max_retries - 1 {
                        tokio::time::sleep(Duration::from_secs(2u64.pow(attempt))).await;
                        continue;
                    }
                }
            }
        }

        Err(last_error.unwrap_or_else(|| "Max retries exceeded".to_string()))
    }

    async fn generate_stream(
        &self,
        messages: &[LLMMessage],
        options: &GenerationOptions,
    ) -> std::result::Result<
        Box<dyn Stream<Item = std::result::Result<String, String>> + Send>,
        String,
    > {
        // Check availability
        if !self.available {
            return Err("OpenAI is not available".to_string());
        }

        // Convert messages
        let openai_messages: Vec<OpenAIMessage> = messages
            .iter()
            .map(|msg| OpenAIMessage {
                role: crate::llm::role_to_string(&msg.role),
                content: msg.content.clone(),
            })
            .collect();

        // Build request with streaming
        let request = OpenAIChatRequest {
            model: self.model.clone(),
            messages: openai_messages,
            temperature: options.temperature,
            top_p: options.top_p,
            max_tokens: options.max_tokens,
            frequency_penalty: options.frequency_penalty,
            presence_penalty: options.presence_penalty,
            stream: true,
            stop: if options.stop_sequences.is_empty() {
                None
            } else {
                Some(options.stop_sequences.clone())
            },
            seed: options.seed,
        };

        let url = format!("{}/chat/completions", self.base_url);

        // Send streaming request
        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| format!("Stream request failed: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(format!("OpenAI API error {}: {}", status, text));
        }

        // Create stream from response
        let stream = response.bytes_stream().map(move |chunk| {
            match chunk {
                Ok(bytes) => {
                    let text = String::from_utf8_lossy(&bytes);
                    // Parse SSE format
                    for line in text.lines() {
                        if line.starts_with("data: ") {
                            let data = &line[6..];
                            if data == "[DONE]" {
                                continue;
                            }
                            if let Ok(stream_resp) =
                                serde_json::from_str::<OpenAIStreamResponse>(data)
                            {
                                if let Some(choice) = stream_resp.choices.first() {
                                    if let Some(content) = &choice.delta.content {
                                        return Ok(content.clone());
                                    }
                                }
                            }
                        }
                    }
                    Ok(String::new())
                }
                Err(e) => Err(format!("Stream error: {}", e)),
            }
        });

        Ok(Box::new(stream))
    }

    fn model_name(&self) -> &str {
        &self.model
    }

    fn max_context_length(&self) -> usize {
        match self.model.as_str() {
            // GPT-4 models
            "gpt-4" | "gpt-4-32k" => 8192,
            "gpt-4-turbo" | "gpt-4-1106-preview" => 128000,
            "gpt-4-vision-preview" => 128000,
            // GPT-3.5 models
            "gpt-3.5-turbo" => 4096,
            "gpt-3.5-turbo-16k" => 16384,
            "gpt-3.5-turbo-1106" => 16384,
            // Default
            _ => 4096,
        }
    }

    async fn is_available(&self) -> bool {
        self.available
    }
}
