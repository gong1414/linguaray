//! Typed settings over tauri-plugin-store (default provider id + target language).
//! Replaces 2a's hardcoded "openai"/"zh".
use serde::{Deserialize, Serialize};

const STORE_FILE: &str = "settings.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub default_provider: String,
    pub target_language: String,
}

impl Default for Settings {
    fn default() -> Self {
        Self { default_provider: "openai".into(), target_language: "zh".into() }
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
    Settings { default_provider: provider, target_language: target }
}

/// Save settings (writes through to disk).
pub fn save(app: &tauri::AppHandle, s: &Settings) -> Result<(), String> {
    use tauri_plugin_store::StoreExt;
    let store = app.store(STORE_FILE).map_err(|e| e.to_string())?;
    store.set("default_provider", serde_json::json!(s.default_provider));
    store.set("target_language", serde_json::json!(s.target_language));
    store.save().map_err(|e| e.to_string())
}
