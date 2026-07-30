//! IslandPot — translation core.
//!
//! Architecture decided in the grill-me session (2026-07-30):
//! - Unified translate contract: `translate(text, from, to, options) -> text`.
//! - Two layers of engines, both sharing that contract:
//!     * `providers` — AI preset catalog (cc-switch-style "fill key, instant use").
//!       These are OpenAI/Anthropic-compatible HTTP callers, driven by CONFIG DATA.
//!     * `engines`   — built-in traditional MT engines (DeepL/Google/百度/有道/...),
//!       ported from pot's `.potext` JS source. Role: AI-failure fallback +
//!       system-dictionary integration. Built-in Rust modules, NOT plugins.
//! - No WASM, no plugin system in v1 (deferred to post-v1).

mod engines;
mod providers;

use serde::{Deserialize, Serialize};

/// A single translation request — the universal contract every engine implements.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslateRequest {
    pub text: String,
    /// Source language code; `auto` lets the engine detect.
    pub from: String,
    /// Target language code.
    pub to: String,
    /// Engine-specific options (domain, formality, system prompt hints, ...).
    /// Engines MUST ignore options they don't understand.
    #[serde(default)]
    pub options: serde_json::Value,
}

/// A single translation result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslateResult {
    pub text: String,
    /// Which engine produced this (for UI fallback chains & attribution).
    pub engine: String,
}

/// Tauri command: translate via the named engine.
///
/// For v1 the engine registry is static (built-in). Post-v1 this is where a
/// plugin/WASM loader would resolve the engine name.
#[tauri::command]
fn translate(req: TranslateRequest, engine: String) -> Result<TranslateResult, String> {
    // AI providers are resolved by id first, then built-in traditional engines.
    if let Some(provider) = providers::presets().into_iter().find(|p| p.id == engine) {
        let text = provider.translate(&req).map_err(|e| e.to_string())?;
        return Ok(TranslateResult { text, engine: provider.id });
    }

    let registry = engines::registry();
    let selected = registry
        .iter()
        .find(|e| e.id() == engine)
        .ok_or_else(|| format!("unknown engine: {engine}"))?;
    let text = selected.translate(&req).map_err(|e| e.to_string())?;
    Ok(TranslateResult {
        text,
        engine: selected.id().to_string(),
    })
}

/// List the available engines + their metadata (id, label, kind, needs-key).
/// The frontend uses this to render the cc-switch-style picker.
#[tauri::command]
fn list_engines() -> Vec<EngineInfo> {
    let mut info: Vec<EngineInfo> = providers::presets()
        .into_iter()
        .map(EngineInfo::from_provider)
        .collect();
    info.extend(
        engines::registry()
            .iter()
            .map(|e| EngineInfo::from_engine(e.as_ref())),
    );
    info
}

#[derive(Debug, Clone, Serialize)]
pub struct EngineInfo {
    pub id: String,
    pub label: String,
    pub kind: EngineKind,
    /// `true` if the user must supply an API key before this engine works.
    pub needs_key: bool,
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EngineKind {
    /// AI provider preset (OpenAI/Anthropic/Gemini/ollama/中转站).
    Provider,
    /// Built-in traditional MT engine (DeepL/Google/百度/...).
    Traditional,
}

/// Trait implemented by built-in traditional MT engines.
pub trait Engine: Sync {
    /// Stable id, e.g. "google", "deepl".
    fn id(&self) -> &str;
    /// Human label for the picker.
    fn label(&self) -> &str;
    /// Whether the user must supply a key/credential.
    fn needs_key(&self) -> bool {
        false
    }
    /// Perform the translation.
    fn translate(&self, req: &TranslateRequest) -> Result<String, TranslateError>;
}

/// Error type for all engines (traditional + providers).
pub type TranslateError = anyhow::Error;

impl EngineInfo {
    fn from_provider(p: providers::ProviderPreset) -> Self {
        Self {
            id: p.id,
            label: p.label,
            kind: EngineKind::Provider,
            needs_key: p.needs_key,
        }
    }

    fn from_engine(e: &dyn Engine) -> Self {
        Self {
            id: e.id().to_string(),
            label: e.label().to_string(),
            kind: EngineKind::Traditional,
            needs_key: e.needs_key(),
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![translate, list_engines])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
