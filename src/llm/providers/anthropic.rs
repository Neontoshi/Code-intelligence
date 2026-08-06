use crate::llm::providers::ProviderConfig;
use crate::llm::{GenerationOptions, LLMMessage, LLMProvider, LLMResponse};
use async_trait::async_trait;
use futures::Stream;

pub struct AnthropicProvider {
    _config: ProviderConfig,
}

impl AnthropicProvider {
    pub fn new(config: &ProviderConfig) -> Result<Self, String> {
        Ok(Self {
            _config: config.clone(),
        })
    }
}

#[async_trait]
impl LLMProvider for AnthropicProvider {
    async fn generate(
        &self,
        _messages: &[LLMMessage],
        _options: &GenerationOptions,
    ) -> Result<LLMResponse, String> {
        Err("Anthropic provider not fully implemented".to_string())
    }

    async fn generate_stream(
        &self,
        _messages: &[LLMMessage],
        _options: &GenerationOptions,
    ) -> Result<Box<dyn Stream<Item = Result<String, String>> + Send>, String> {
        Err("Anthropic provider not fully implemented".to_string())
    }

    fn model_name(&self) -> &str {
        "claude-3-haiku"
    }

    fn max_context_length(&self) -> usize {
        4096
    }
}
