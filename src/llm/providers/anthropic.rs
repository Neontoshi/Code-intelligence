use crate::llm::providers::ProviderConfig;
use crate::llm::{GenerationOptions, LLMMessage, LLMProvider, LLMResponse, MessageRole, Usage};
use async_trait::async_trait;
use futures::Stream;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Serialize)]
struct AnthropicRequest {
    model: String,
    messages: Vec<AnthropicMessage>,
    max_tokens: usize,
    temperature: f32,
    stream: bool,
    system: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct AnthropicMessage {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct AnthropicResponse {
    _id: String,
    model: String,
    content: Vec<AnthropicContent>,
    usage: Option<AnthropicUsage>,
    stop_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
struct AnthropicContent {
    text: String,
}

#[derive(Debug, Deserialize)]
struct AnthropicUsage {
    input_tokens: usize,
    output_tokens: usize,
}

pub struct AnthropicProvider {
    client: Client,
    api_key: String,
    base_url: String,
    model: String,
    _timeout_seconds: u64,
}

impl AnthropicProvider {
    pub fn new(config: &ProviderConfig) -> crate::error::Result<Self> {
        let api_key = config
            .api_key
            .clone()
            .ok_or_else(|| anyhow::anyhow!("Anthropic API key is required"))?;

        let base_url = config
            .base_url
            .clone()
            .unwrap_or_else(|| "https://api.anthropic.com/v1".to_string());

        let client = Client::builder()
            .timeout(Duration::from_secs(config.timeout_seconds))
            .build()
            .map_err(|e| anyhow::anyhow!("Failed to create HTTP client: {}", e))?;

        Ok(Self {
            client,
            api_key,
            base_url,
            model: config.model.clone(),
            _timeout_seconds: config.timeout_seconds,
        })
    }
}

#[async_trait]
impl LLMProvider for AnthropicProvider {
    async fn generate(
        &self,
        messages: &[LLMMessage],
        options: &GenerationOptions,
    ) -> crate::error::Result<LLMResponse> {
        // Extract system message (Anthropic has a separate system field)
        let mut system = None;
        let mut anthropic_messages = Vec::new();

        for msg in messages {
            match msg.role {
                MessageRole::System => {
                    system = Some(msg.content.clone());
                }
                _ => {
                    anthropic_messages.push(AnthropicMessage {
                        role: crate::llm::role_to_string(&msg.role),
                        content: msg.content.clone(),
                    });
                }
            }
        }

        let request = AnthropicRequest {
            model: self.model.clone(),
            messages: anthropic_messages,
            max_tokens: options.max_tokens,
            temperature: options.temperature,
            stream: false,
            system,
        };

        let url = format!("{}/messages", self.base_url);

        let response = self
            .client
            .post(&url)
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("Request failed: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!("Anthropic API error {}: {}", status, text));
        }

        let anthropic_response: AnthropicResponse = response
            .json()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to parse response: {}", e))?;

        let content = anthropic_response
            .content
            .first()
            .map(|c| c.text.clone())
            .unwrap_or_default();

        Ok(LLMResponse {
            content,
            model: anthropic_response.model,
            usage: anthropic_response.usage.map(|u| Usage {
                prompt_tokens: u.input_tokens,
                completion_tokens: u.output_tokens,
                total_tokens: u.input_tokens + u.output_tokens,
            }),
            finish_reason: anthropic_response.stop_reason,
        })
    }

    async fn generate_stream(
        &self,
        _messages: &[LLMMessage],
        _options: &GenerationOptions,
    ) -> crate::error::Result<Box<dyn Stream<Item = crate::error::Result<String>> + Send>> {
        Err(anyhow::anyhow!(
            "Streaming not yet implemented for Anthropic provider"
        ))
    }

    fn model_name(&self) -> &str {
        &self.model
    }

    fn max_context_length(&self) -> usize {
        match self.model.as_str() {
            "claude-3-opus-20240229" => 200000,
            "claude-3-sonnet-20240229" => 200000,
            "claude-3-haiku-20240307" => 200000,
            "claude-2.1" | "claude-2.0" => 100000,
            _ => 100000,
        }
    }

    async fn is_available(&self) -> bool {
        // Check if API key is valid by making a lightweight request
        let test_messages = vec![LLMMessage::user("Hello")];

        match self
            .generate(&test_messages, &GenerationOptions::default())
            .await
        {
            Ok(_) => true,
            Err(e) => {
                eprintln!("⚠️ Anthropic API check failed: {}", e);
                false
            }
        }
    }
}
