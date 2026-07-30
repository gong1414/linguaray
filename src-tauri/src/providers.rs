//! AI provider preset catalog — IslandPot's core differentiator.
//!
//! Model: cc-switch. Each entry is CONFIG DATA (base_url + model + auth kind +
//! a slot for the user's key). The user fills a key and it works — they do NOT
//! hand-edit a generic OpenAI endpoint form. Adding a provider = adding one
//! `ProviderPreset`, a few lines.
//!
//! The `translate` caller below is a unified OpenAI/Anthropic-compatible HTTP
//! client driven by the preset's `api_kind`. No WASM, no plugins — plain Rust.

use crate::{TranslateError, TranslateRequest};

/// A pre-configured AI provider (or API 中转站). Pure data.
#[derive(Debug, Clone)]
pub struct ProviderPreset {
    pub id: String,
    pub label: String,
    /// Where requests are sent.
    pub base_url: String,
    /// Which API dialect the provider speaks.
    pub api_kind: ApiKind,
    /// Default model id to use if the user doesn't override.
    pub default_model: String,
    /// `true` if a key must be supplied (false for local ollama).
    pub needs_key: bool,
}

/// The wire dialect. Most 中转站 speak OpenAI's chat/completions format even
/// when serving Claude/Gemini models behind a proxy.
#[derive(Debug, Clone, Copy)]
pub enum ApiKind {
    OpenAIChat,
    Anthropic,
}

impl ProviderPreset {
    /// Perform a translation through this provider.
    ///
    /// NOTE: this is the v1 stub. Real impl will:
    /// 1. read the user's key from settings (TBD: tauri-plugin-store)
    /// 2. build a translation-tuned prompt (auto-detect, terminology, segmentation)
    /// 3. POST to `{base_url}/{chat_completions_or_messages}` per `api_kind`
    /// 4. return the assistant text
    pub fn translate(&self, _req: &TranslateRequest) -> Result<String, TranslateError> {
        anyhow::bail!(
            "provider '{}' translate() not yet implemented — v1 stub",
            self.id
        )
    }
}

/// The built-in preset catalog. This list grows by appending entries — the whole
/// point of the cc-switch model. Start small; the long tail is community-driven.
pub fn presets() -> Vec<ProviderPreset> {
    vec![
        ProviderPreset {
            id: "openai".into(),
            label: "OpenAI".into(),
            base_url: "https://api.openai.com/v1".into(),
            api_kind: ApiKind::OpenAIChat,
            default_model: "gpt-4o-mini".into(),
            needs_key: true,
        },
        ProviderPreset {
            id: "anthropic".into(),
            label: "Anthropic Claude".into(),
            base_url: "https://api.anthropic.com/v1".into(),
            api_kind: ApiKind::Anthropic,
            default_model: "claude-sonnet-4-5".into(),
            needs_key: true,
        },
        ProviderPreset {
            id: "gemini".into(),
            label: "Google Gemini".into(),
            base_url: "https://generativelanguage.googleapis.com/v1beta".into(),
            api_kind: ApiKind::OpenAIChat, // Gemini has an OpenAI-compatible path
            default_model: "gemini-2.0-flash".into(),
            needs_key: true,
        },
        ProviderPreset {
            id: "ollama".into(),
            label: "Ollama (local)".into(),
            base_url: "http://localhost:11434/v1".into(),
            api_kind: ApiKind::OpenAIChat,
            default_model: "qwen2.5:7b".into(),
            needs_key: false, // local-first: no key, no telemetry, works offline
        },
    ]
}
