//! LLM translation providers.
//!
//! [`openai_compatible`] carries every vendor that speaks the OpenAI chat
//! completions protocol (OpenAI, xAI, DeepSeek, Qwen, Zhipu, Moonshot,
//! Doubao, Groq, Gemini, plus self-hosted endpoints); only Anthropic and
//! Ollama need dedicated implementations.

pub mod anthropic;
pub mod ollama;
pub mod openai_compatible;

fn configured_default_model(default_model: &str) -> Result<String, String> {
    let default_model = default_model.trim().to_owned();
    if default_model.is_empty() {
        return Err("default_model must be configured".to_owned());
    }
    Ok(default_model)
}

#[cfg(feature = "anthropic")]
pub use anthropic::AnthropicProvider;
pub use anthropic::AnthropicProviderConfig;
#[cfg(feature = "ollama")]
pub use ollama::OllamaProvider;
pub use ollama::OllamaProviderConfig;
pub use openai_compatible::{
    specs, OpenAiCompatibleProvider, OpenAiCompatibleProviderConfig, OpenAiCompatibleSpec,
};

pub type OpenAiProviderConfig = OpenAiCompatibleProviderConfig;
pub type XAiProviderConfig = OpenAiCompatibleProviderConfig;
