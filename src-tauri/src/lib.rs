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
pub mod error;
pub mod keystore;
pub mod providers;
pub mod service;
pub mod wire;

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::Manager;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslateRequest {
    pub text: String,
    pub from: String,
    pub to: String,
    #[serde(default)]
    pub options: serde_json::Value,
}

#[derive(Debug, Clone, Serialize)]
pub struct TranslateResult {
    pub text: String,
    pub engine: String,
}

struct AppState {
    client: reqwest::Client,
    keystore: keystore::Keystore,
}

#[tauri::command]
async fn translate(
    state: tauri::State<'_, Arc<AppState>>,
    req: TranslateRequest,
    engine: String,
) -> Result<TranslateResult, String> {
    let preset = providers::presets().into_iter()
        .find(|p| p.id == engine)
        .ok_or_else(|| format!("unknown engine: {engine}"))?;
    let opts = wire::AppOptions::default(); // v1: no app-options UI yet
    let input = service::TranslateInput {
        text: &req.text,
        from: &req.from,
        to: &req.to,
        options: opts,
    };
    let text = service::translate(&state.client, &state.keystore, &preset, input)
        .await
        .map_err(|e| e.to_string())?;
    Ok(TranslateResult { text, engine: preset.id })
}

#[tauri::command]
fn list_engines() -> Vec<EngineInfo> {
    providers::presets()
        .into_iter()
        .map(EngineInfo::from_provider)
        .collect()
}

#[tauri::command]
fn set_key(
    state: tauri::State<'_, Arc<AppState>>,
    provider_id: String,
    key: String,
) -> Result<(), String> {
    let mut keys = state.keystore.load().map_err(|e| e.to_string())?;
    if !keys.is_object() {
        keys = serde_json::json!({});
    }
    keys[&provider_id] = serde_json::json!(key);
    state.keystore.store(&keys).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn delete_key(
    state: tauri::State<'_, Arc<AppState>>,
    provider_id: String,
) -> Result<(), String> {
    let mut keys = state.keystore.load().map_err(|e| e.to_string())?;
    if let Some(obj) = keys.as_object_mut() {
        obj.remove(&provider_id);
    }
    state.keystore.store(&keys).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn key_status(
    state: tauri::State<'_, Arc<AppState>>,
) -> Result<std::collections::HashMap<String, bool>, String> {
    let keys = state.keystore.load().map_err(|e| e.to_string())?;
    let mut map = std::collections::HashMap::new();
    if let Some(obj) = keys.as_object() {
        for (k, _v) in obj {
            map.insert(k.clone(), true);
        }
    }
    Ok(map)
}

#[derive(Debug, Clone, Serialize)]
pub struct EngineInfo {
    pub id: String,
    pub label: String,
    pub kind: String,
    pub needs_key: bool,
}

impl EngineInfo {
    fn from_provider(p: providers::ProviderPreset) -> Self {
        Self {
            id: p.id,
            label: p.label,
            kind: "provider".into(),
            needs_key: p.needs_key,
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let dir = app
                .path()
                .app_local_data_dir()
                .expect("app_local_data_dir");
            let keystore = keystore::Keystore::new(dir).expect("keystore init");
            // Spec §Privacy: no cross-origin redirects.
            let client = reqwest::Client::builder()
                .redirect(reqwest::redirect::Policy::none())
                .build()
                .expect("client");
            app.manage(Arc::new(AppState { client, keystore }));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            translate,
            list_engines,
            set_key,
            delete_key,
            key_status
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
