// src/llm/providers/ollama.rs

//! Ollama LLM Provider
//!
//! This module provides an implementation of the LLMProvider trait for
//! Ollama, with specific optimizations for phi-2 model.

use crate::llm::providers::ProviderConfig;
use crate::llm::{GenerationOptions, LLMMessage, LLMProvider, LLMResponse, Result, Usage};

use async_trait::async_trait;
use futures::stream::{Stream, StreamExt};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

// Ollama API Types

#[derive(Debug, Serialize)]
struct OllamaChatRequest {
    model: String,
    messages: Vec<OllamaMessage>,
    stream: bool,
    options: Option<OllamaOptions>,
    format: Option<String>, // For structured output
}

#[derive(Debug, Serialize, Deserialize)]
struct OllamaMessage {
    role: String,
    content: String,
}

#[derive(Debug, Serialize)]
struct OllamaOptions {
    temperature: f32,
    top_p: f32,
    num_predict: usize,
    frequency_penalty: f32,
    presence_penalty: f32,
    stop: Option<Vec<String>>,
    seed: Option<u64>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct OllamaChatResponse {
    message: OllamaMessage,
    done: bool,
    total_duration: Option<u64>,
    load_duration: Option<u64>,
    prompt_eval_count: Option<usize>,
    eval_count: Option<usize>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct OllamaStreamResponse {
    message: OllamaMessage,
    done: bool,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct OllamaModelsResponse {
    models: Vec<OllamaModel>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct OllamaModel {
    name: String,
    modified_at: String,
    size: u64,
}

// Ollama Provider

pub struct OllamaProvider {
    client: Client,
    base_url: String,
    model: String,
    config: ProviderConfig,
    available: bool,
    // Cache model info
    model_info: Option<OllamaModel>,
}

impl OllamaProvider {
    pub async fn new(config: &ProviderConfig) -> std::result::Result<Self, String> {
        let base_url = config
            .base_url
            .clone()
            .unwrap_or_else(|| "http://localhost:11434".to_string());

        // Build HTTP client with timeout
        let client = Client::builder()
            .timeout(Duration::from_secs(config.timeout_seconds))
            .build()
            .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

        let mut provider = Self {
            client,
            base_url,
            model: config.model.clone(),
            config: config.clone(),
            available: false,
            model_info: None,
        };

        // Check availability and model
        provider.check_availability().await;

        Ok(provider)
    }

    /// Check if Ollama is available and the model exists
    async fn check_availability(&mut self) {
        let url = format!("{}/api/tags", self.base_url);
        let response = self
            .client
            .get(&url)
            .timeout(Duration::from_secs(5))
            .send()
            .await
            .ok();

        match response {
            Some(resp) if resp.status().is_success() => {
                // Parse models
                if let Ok(models) = resp.json::<OllamaModelsResponse>().await {
                    // Check if our model exists
                    let model_exists = models
                        .models
                        .iter()
                        .any(|m| m.name == self.model || m.name.starts_with(&self.model));

                    if model_exists {
                        self.available = true;
                        self.model_info = models
                            .models
                            .into_iter()
                            .find(|m| m.name == self.model || m.name.starts_with(&self.model));
                        eprintln!("✅ Ollama connected. Model '{}' available.", self.model);
                    } else {
                        eprintln!("⚠️ Model '{}' not found in Ollama.", self.model);
                        eprintln!("   Available models:");
                        for model in &models.models {
                            eprintln!("   - {}", model.name);
                        }
                        eprintln!("   To pull: `ollama pull {}`", self.model);
                        self.available = false;
                    }
                }
            }
            Some(resp) => {
                eprintln!("⚠️ Ollama error: {}", resp.status());
                self.available = false;
            }
            None => {
                eprintln!("⚠️ Ollama not available at {}", self.base_url);
                eprintln!("   Please start Ollama: `ollama serve`");
                eprintln!("   Or pull the model: `ollama pull {}`", self.model);
                self.available = false;
            }
        }
    }

    /// Build the request options
    fn build_options(options: &GenerationOptions) -> OllamaOptions {
        OllamaOptions {
            temperature: options.temperature,
            top_p: options.top_p,
            num_predict: options.max_tokens,
            frequency_penalty: options.frequency_penalty,
            presence_penalty: options.presence_penalty,
            stop: if options.stop_sequences.is_empty() {
                None
            } else {
                Some(options.stop_sequences.clone())
            },
            seed: options.seed,
        }
    }

    /// Check if provider is available and model is loaded
    pub fn is_available(&self) -> bool {
        self.available
    }

    /// Get model information
    pub fn model_info(&self) -> Option<&OllamaModel> {
        self.model_info.as_ref()
    }

    /// Estimate tokens for a prompt (Ollama doesn't provide this, so we estimate)
    pub fn estimate_tokens(&self, text: &str) -> usize {
        // Rough estimate: ~4 chars per token for English, ~3 for code
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
}

// LLMProvider Implementation

#[async_trait]
impl LLMProvider for OllamaProvider {
    async fn generate(
        &self,
        messages: &[LLMMessage],
        options: &GenerationOptions,
    ) -> std::result::Result<LLMResponse, String> {
        // Check availability
        if !self.available {
            return Err("Ollama is not available".to_string());
        }

        // Convert messages
        let ollama_messages: Vec<OllamaMessage> = messages
            .iter()
            .map(|msg| OllamaMessage {
                role: crate::llm::role_to_string(&msg.role),
                content: msg.content.clone(),
            })
            .collect();

        // Build request
        let request = OllamaChatRequest {
            model: self.model.clone(),
            messages: ollama_messages,
            stream: false,
            options: Some(Self::build_options(options)),
            format: None,
        };

        let url = format!("{}/api/chat", self.base_url);

        // Send request with retries
        let mut last_error = None;
        for attempt in 0..self.config.max_retries {
            match self.client.post(&url).json(&request).send().await {
                Ok(response) => {
                    if response.status().is_success() {
                        // Parse response
                        match response.json::<OllamaChatResponse>().await {
                            Ok(ollama_response) => {
                                return Ok(LLMResponse {
                                    content: ollama_response.message.content,
                                    model: self.model.clone(),
                                    usage: Some(Usage {
                                        prompt_tokens: ollama_response
                                            .prompt_eval_count
                                            .unwrap_or(0),
                                        completion_tokens: ollama_response.eval_count.unwrap_or(0),
                                        total_tokens: ollama_response
                                            .prompt_eval_count
                                            .unwrap_or(0)
                                            + ollama_response.eval_count.unwrap_or(0),
                                    }),
                                    finish_reason: if ollama_response.done {
                                        Some("stop".to_string())
                                    } else {
                                        None
                                    },
                                });
                            }
                            Err(e) => {
                                return Err(format!("Failed to parse Ollama response: {}", e));
                            }
                        }
                    } else {
                        let status = response.status();
                        let text = response.text().await.unwrap_or_default();
                        last_error = Some(format!("Ollama API error {}: {}", status, text));
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
            return Err("Ollama is not available".to_string());
        }

        // Convert messages
        let ollama_messages: Vec<OllamaMessage> = messages
            .iter()
            .map(|msg| OllamaMessage {
                role: crate::llm::role_to_string(&msg.role),
                content: msg.content.clone(),
            })
            .collect();

        // Build request with streaming
        let request = OllamaChatRequest {
            model: self.model.clone(),
            messages: ollama_messages,
            stream: true,
            options: Some(Self::build_options(options)),
            format: None,
        };

        let url = format!("{}/api/chat", self.base_url);

        // Send streaming request
        let response = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .await
            .map_err(|e| format!("Stream request failed: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(format!("Ollama API error {}: {}", status, text));
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
                                serde_json::from_str::<OllamaStreamResponse>(data)
                            {
                                return Ok(stream_resp.message.content);
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
            // Phi-2 has 4096 context
            "phi:2.7b" | "phi-2" => 4096,
            // Other models
            _ => 4096,
        }
    }

    async fn is_available(&self) -> bool {
        self.available
    }
}

// Phi-2 Specific Optimizations

impl OllamaProvider {
    /// Create a provider specifically configured for phi-2
    pub async fn phi2() -> Self {
        let config = ProviderConfig {
            model: "phi:2.7b".to_string(),
            base_url: Some("http://localhost:11434".to_string()),
            timeout_seconds: 60,
            max_retries: 3,
            extra_headers: Vec::new(),
            api_key: None,
        };
        Self::new(&config)
            .await
            .expect("Failed to create Ollama provider for phi-2. Make sure Ollama is running.")
    }

    /// Optimized generation for phi-2 with lower temperature for structured output
    pub async fn generate_structured(
        &self,
        messages: &[LLMMessage],
        schema: serde_json::Value,
    ) -> Result<serde_json::Value> {
        let options = GenerationOptions {
            temperature: 0.1, // Low temperature for deterministic output
            max_tokens: 2000,
            top_p: 0.9,
            ..Default::default()
        };

        // Add schema to system prompt
        let mut enhanced_messages = Vec::new();
        let schema_str = serde_json::to_string(&schema).unwrap_or_default();

        // Add system message about output format
        enhanced_messages.push(LLMMessage::system(format!(
            "Respond in the following JSON format:\n{}\nUse valid JSON only.",
            schema_str
        )));

        // Add user messages
        for msg in messages {
            enhanced_messages.push(msg.clone());
        }

        // Generate response
        let response = self
            .generate(&enhanced_messages, &options)
            .await
            .map_err(|e| format!("Generation failed: {}", e))?;

        // Parse JSON from response
        let json = crate::llm::extract_json_from_response(&response.content)
            .map_err(|_| "Failed to parse JSON from response".to_string())?;

        Ok(json)
    }
}
