//! AI provider preset catalog — IslandPot's core differentiator (spec §Wire).
//!
//! Each entry is CONFIG DATA (a FULL endpoint URL + dialect + default model + a
//! key-needed flag). The user fills a key and it works. Adding a provider = one
//! struct literal, not code. The HTTP calling lives in `wire.rs`.

use crate::wire::ApiKind;

#[derive(Debug, Clone)]
pub struct ProviderPreset {
    pub id: String,
    pub label: String,
    /// FULL endpoint URL, e.g. "https://api.openai.com/v1/chat/completions".
    /// Stored in full (not base_url + route) because Url::join with a leading-'/'
    /// route would drop /v1 or /v1beta/openai. (spec §Wire)
    pub endpoint: String,
    pub api_kind: ApiKind,
    pub default_model: String,
    pub needs_key: bool,
}

pub fn presets() -> Vec<ProviderPreset> {
    vec![
        ProviderPreset {
            id: "openai".into(), label: "OpenAI".into(),
            endpoint: "https://api.openai.com/v1/chat/completions".into(),
            api_kind: ApiKind::OpenAIChat, default_model: "gpt-4o-mini".into(), needs_key: true,
        },
        ProviderPreset {
            id: "anthropic".into(), label: "Anthropic Claude".into(),
            endpoint: "https://api.anthropic.com/v1/messages".into(),
            api_kind: ApiKind::Anthropic, default_model: "claude-sonnet-4-5".into(), needs_key: true,
        },
        ProviderPreset {
            id: "gemini".into(), label: "Google Gemini".into(),
            // OpenAI-compatible path (spec §Wire): /v1beta/openai/chat/completions
            endpoint: "https://generativelanguage.googleapis.com/v1beta/openai/chat/completions".into(),
            api_kind: ApiKind::OpenAIChat, default_model: "gemini-2.0-flash".into(), needs_key: true,
        },
        ProviderPreset {
            id: "ollama".into(), label: "Ollama (local)".into(),
            endpoint: "http://localhost:11434/v1/chat/completions".into(),
            api_kind: ApiKind::OpenAIChat, default_model: "qwen2.5:7b".into(), needs_key: false,
        },
    ]
}

/// Spec §Privacy: remote endpoints must be HTTPS; HTTP allowed only for loopback
/// (Ollama). Any other scheme is rejected.
pub fn validate_endpoint(url: &str) -> Result<(), String> {
    let parsed = url::Url::parse(url).map_err(|e| format!("bad url: {e}"))?;
    match parsed.scheme() {
        "https" => Ok(()),
        "http" => {
            let h = parsed.host_str().unwrap_or("");
            if h == "localhost" || h == "127.0.0.1" || h == "::1" {
                Ok(())
            } else {
                Err(format!("http only allowed for loopback, got {h}"))
            }
        }
        s => Err(format!("scheme {s} not allowed")),
    }
}
