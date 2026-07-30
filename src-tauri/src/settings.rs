//! Typed settings over tauri-plugin-store (default provider id + target language).
//! Replaces 2a's hardcoded "openai"/"zh".
use serde::{Deserialize, Serialize};

const STORE_FILE: &str = "settings.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub default_provider: String,
    pub target_language: String,
    /// §G: cross-remote fallback is OPT-IN. `None` (default) = no fallback; the
    /// user must explicitly choose a traditional engine to consent to their text
    /// being sent to a second remote endpoint. Set to a `TraditionalEngine` id
    /// (e.g. "google") to enable the single AI→trad fallback attempt.
    pub fallback_engine: Option<String>,
}

impl Default for Settings {
    fn default() -> Self {
        // §G: cross-remote fallback is OPT-IN. Default None.
        Self { default_provider: "openai".into(), target_language: "zh".into(), fallback_engine: None }
    }
}

/// Load settings, falling back to defaults for missing keys.
pub fn load(app: &tauri::AppHandle) -> Settings {
    use tauri_plugin_store::StoreExt;
    let store = match app.store(STORE_FILE) {
        Ok(s) => s,
        Err(_) => return Settings::default(),
    };
    let provider = store
        .get("default_provider")
        .and_then(|v| v.as_str().map(String::from))
        .unwrap_or_else(|| Settings::default().default_provider);
    let target = store
        .get("target_language")
        .and_then(|v| v.as_str().map(String::from))
        .unwrap_or_else(|| Settings::default().target_language);
    // §G: fallback_engine is opt-in — a missing/absent key means None.
    let fallback_engine = store
        .get("fallback_engine")
        .and_then(|v| v.as_str().map(String::from));
    Settings { default_provider: provider, target_language: target, fallback_engine }
}

/// Save settings (writes through to disk).
pub fn save(app: &tauri::AppHandle, s: &Settings) -> Result<(), String> {
    use tauri_plugin_store::StoreExt;
    let store = app.store(STORE_FILE).map_err(|e| e.to_string())?;
    store.set("default_provider", serde_json::json!(s.default_provider));
    store.set("target_language", serde_json::json!(s.target_language));
    // §G: write the opt-in fallback engine (None serializes as null).
    store.set("fallback_engine", serde_json::json!(s.fallback_engine));
    store.save().map_err(|e| e.to_string())
}
