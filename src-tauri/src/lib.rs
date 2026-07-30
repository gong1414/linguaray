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
pub mod clipboard;
pub mod concurrency;
pub mod cursor;
pub mod error;
pub mod keystore;
pub mod popup;
pub mod providers;
pub mod selection;
pub mod selection_engine;
pub mod service;
pub mod settings;
pub mod wire;

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::Manager;
use tauri_plugin_global_shortcut::{Builder as GlobalShortcutBuilder, ShortcutState};

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

/// Shared application state.
///
/// `gen` is the latest-wins token generator (§concurrency): every hotkey trigger
/// bumps it, and every async transition (popup, translate-result) checks
/// `is_latest` before mutating the popup, so a stale in-flight request can never
/// clobber the result of a newer trigger.
struct Session {
    client: reqwest::Client,
    keystore: keystore::Keystore,
    gen: concurrency::GenerationToken,
}

#[tauri::command]
async fn translate(
    state: tauri::State<'_, Arc<Session>>,
    req: TranslateRequest,
    engine: String,
) -> Result<TranslateResult, String> {
    let preset = providers::presets()
        .into_iter()
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
    Ok(TranslateResult {
        text,
        engine: preset.id,
    })
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
    state: tauri::State<'_, Arc<Session>>,
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
    state: tauri::State<'_, Arc<Session>>,
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
    state: tauri::State<'_, Arc<Session>>,
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

/// Selection-translate loop bound to the `Alt+Space` global shortcut (§B/§C/§D).
///
/// The handler runs on the global-shortcut event thread, so all real work is
/// moved onto the async runtime via `tauri::async_runtime::spawn`. Ordering,
/// per spec §concurrency:
///   1. `gen.next()` FIRST — any concurrent trigger now supersedes us.
///   2. capture cursor + selection under the selection mutex (selection touches
///      the clipboard; serializing it prevents two triggers from corrupting the
///      saved-clipboard restore). Cursor position is read before the popup can
///      steal focus.
///   3. `is_latest` check after capture — bail if superseded.
///   4. `popup::show_at` loading at the captured cursor.
///   5. translate, then `is_latest` again before showing the result, so a stale
///      result never overwrites a fresher popup.
fn on_hotkey(app: &tauri::AppHandle, _shortcut: &tauri_plugin_global_shortcut::Shortcut, event: tauri_plugin_global_shortcut::ShortcutEvent) {
    // Only act on key-down; ignore release.
    if event.state != ShortcutState::Pressed {
        return;
    }

    let app2 = app.clone();
    tauri::async_runtime::spawn(async move {
        let state = app2.state::<Arc<Session>>().inner().clone();

        // (1) latest-wins token — allocate FIRST, before any work.
        let gen = state.gen.next();

        // (2) capture cursor position + selection under ONE selection-mutex hold,
        // BEFORE the popup steals focus. The lock must span BOTH reads in a single
        // guard — splitting it into two lock acquisitions lets a second hotkey
        // trigger run its own capture_selection in between, whose clipboard writes
        // (sentinel/copy) would interleave with this run's restore window and
        // reopen the clipboard-corruption race the mutex exists to close (spec §concurrency).
        let (x, y, captured) = {
            let _g = state.gen.selection_lock();
            let pos = cursor::position();
            let cap = selection::capture_selection(800);
            (pos.0, pos.1, cap)
        };

        // (3) superseded by a newer trigger — drop this run silently.
        if !state.gen.is_latest(gen) {
            return;
        }

        let text = match captured {
            Ok(selection_engine::Capture::Selected(t)) => t,
            Ok(selection_engine::Capture::NoSelection) => {
                let _ = popup::hide(&app2);
                return;
            }
            Err(e) => {
                let _ = popup::error(&app2, &e);
                return;
            }
        };

        // (4) show loading popup at the cursor.
        let _ = popup::show_at(&app2, x, y);

        // (5) translate via the Phase-1 service. Default provider is "openai"
        //     for now; the real default-choice UX is Phase 2b.
        let preset = providers::presets()
            .into_iter()
            .find(|p| p.id == "openai")
            .expect("default preset \"openai\" must exist in providers::presets");
        let input = service::TranslateInput {
            text: &text,
            from: "auto",
            to: "zh",
            options: wire::AppOptions::default(),
        };
        match service::translate(&state.client, &state.keystore, &preset, input).await {
            Ok(out) => {
                if state.gen.is_latest(gen) {
                    let _ = popup::result(&app2, &out, &preset.id);
                }
            }
            Err(e) => {
                if state.gen.is_latest(gen) {
                    let _ = popup::error(&app2, &e.to_string());
                }
            }
        }
    });
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let shortcut_plugin = GlobalShortcutBuilder::new()
        .with_shortcut("Alt+Space")
        .expect("parse Alt+Space shortcut")
        .with_handler(on_hotkey)
        .build();

    tauri::Builder::default()
        .plugin(shortcut_plugin)
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_store::Builder::new().build())
        .setup(|app| {
            let dir = app
                .path()
                .app_local_data_dir()
                .expect("app_local_data_dir");
            let keystore = keystore::Keystore::new(dir).expect("keystore init");
            // Spec §Privacy: every preset endpoint must be HTTPS (loopback HTTP
            // allowed for local engines like Ollama). Reject at config-load so an
            // invalid/leaked preset never ships a request.
            for p in providers::presets() {
                providers::validate_endpoint(&p.endpoint)
                    .expect("preset endpoint failed scheme validation");
            }
            // Spec §Privacy: no cross-origin redirects.
            let client = reqwest::Client::builder()
                .redirect(reqwest::redirect::Policy::none())
                .build()
                .expect("client");
            app.manage(Arc::new(Session {
                client,
                keystore,
                gen: concurrency::GenerationToken::new(),
            }));
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
