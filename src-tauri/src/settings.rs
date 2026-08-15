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
    /// R5: startup update check. ON by default; the README privacy section
    /// documents it as the app's only unsolicited network request (GitHub
    /// Releases). Users who want zero outbound traffic can turn it off.
    pub check_updates_on_startup: bool,
}

impl Default for Settings {
    fn default() -> Self {
        // §G: cross-remote fallback is OPT-IN. Default None.
        Self {
            default_provider: "openai".into(),
            target_language: "zh".into(),
            fallback_engine: None,
            check_updates_on_startup: true,
        }
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
    // R5: absent key = default ON (see field doc). Tolerate legacy stores.
    let check_updates_on_startup = store
        .get("check_updates_on_startup")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);
    Settings {
        default_provider: provider,
        target_language: target,
        fallback_engine,
        check_updates_on_startup,
    }
}

/// Save settings (writes through to disk).
pub fn save(app: &tauri::AppHandle, s: &Settings) -> Result<(), String> {
    use tauri_plugin_store::StoreExt;
    let store = app.store(STORE_FILE).map_err(|e| e.to_string())?;
    store.set("default_provider", serde_json::json!(s.default_provider));
    store.set("target_language", serde_json::json!(s.target_language));
    // §G: write the opt-in fallback engine (None serializes as null).
    store.set("fallback_engine", serde_json::json!(s.fallback_engine));
    store.set("check_updates_on_startup", serde_json::json!(s.check_updates_on_startup));
    store.save().map_err(|e| e.to_string())
}

/// R5: strict bool parser for `set_setting` string values. Only the exact
/// lowercase literals are accepted — "1"/"yes"/"TRUE" silently mapping to a
/// different value than the UI showed would be a settings-integrity bug.
pub fn parse_bool_setting(value: &str) -> Result<bool, String> {
    match value {
        "true" => Ok(true),
        "false" => Ok(false),
        _ => Err(format!("invalid boolean value: {value:?} (expected \"true\"|\"false\")")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bool_setting_accepts_only_exact_literals() {
        assert_eq!(parse_bool_setting("true"), Ok(true));
        assert_eq!(parse_bool_setting("false"), Ok(false));
        for bad in ["True", "TRUE", "1", "yes", "", " true"] {
            assert!(parse_bool_setting(bad).is_err(), "{bad:?} should be rejected");
        }
    }
}
