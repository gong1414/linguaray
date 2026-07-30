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

pub mod engines;
pub mod a11y;
pub mod clipboard;
pub mod concurrency;
pub mod cursor;
pub mod dict;
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
use std::str::FromStr;
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
    app: tauri::AppHandle,
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
    // §G: resolve the opt-in fallback engine from settings (None by default).
    let fallback = settings::load(&app).fallback_engine.as_deref().and_then(engines::find);
    let t = service::translate_with_fallback(&state.client, &state.keystore, &preset, input, fallback)
        .await
        .map_err(|e| e.to_string())?;
    Ok(TranslateResult {
        text: t.text,
        engine: t.engine, // actual producing engine (primary or fallback)
    })
}

/// Translate using the settings-configured default provider + target language.
///
/// `req.to == ""` is a sentinel: "use `settings.target_language`". This backs the
/// InputPanel window, which intentionally hides the from/to/provider knobs and
/// just hands the typed text to whatever the user configured (spec §input).
#[tauri::command]
async fn translate_default(
    app: tauri::AppHandle,
    state: tauri::State<'_, Arc<Session>>,
    req: TranslateRequest,
) -> Result<TranslateResult, String> {
    let s = settings::load(&app);
    let to = if req.to.is_empty() {
        s.target_language.clone()
    } else {
        req.to
    };
    let preset = providers::presets()
        .into_iter()
        .find(|p| p.id == s.default_provider)
        .ok_or_else(|| format!("default provider '{}' not found", s.default_provider))?;
    // §G: opt-in fallback engine from settings (None by default).
    let fallback = s.fallback_engine.as_deref().and_then(engines::find);
    let input = service::TranslateInput {
        text: &req.text,
        from: &req.from,
        to: &to,
        options: wire::AppOptions::default(),
    };
    let t = service::translate_with_fallback(&state.client, &state.keystore, &preset, input, fallback)
        .await
        .map_err(|e| e.to_string())?;
    Ok(TranslateResult {
        text: t.text,
        engine: t.engine,
    })
}

/// Translate the clipboard contents ONCE, on user demand.
///
/// Reads the clipboard exactly once and surfaces the result in the popup window at
/// the current cursor position. Deliberately NOT a background listener — there is
/// no clipboard-changed subscription anywhere in the app (spec §Scope: user-initiated).
#[tauri::command]
async fn translate_clipboard(
    app: tauri::AppHandle,
    state: tauri::State<'_, Arc<Session>>,
) -> Result<(), String> {
    // Participate in latest-wins + the selection mutex so we can't read a sentinel
    // mid-selection-capture (which would send `__islandpot_sel_*__` to a remote
    // provider), and so two entry points can't clobber one popup.
    let gen = state.gen.next();
    let text = {
        let _g = state.gen.selection_lock();
        clipboard::get_text()?
    };
    if text.trim().is_empty() {
        return Err("clipboard empty".into());
    }
    let (x, y) = cursor::position();
    let _ = popup::show_at(&app, x, y);
    let s = settings::load(&app);
    let preset = providers::presets()
        .into_iter()
        .find(|p| p.id == s.default_provider)
        .ok_or_else(|| format!("default provider '{}' not found", s.default_provider))?;
    // §G: opt-in fallback engine from settings (None by default).
    let fallback = s.fallback_engine.as_deref().and_then(engines::find);
    let input = service::TranslateInput {
        text: &text,
        from: "auto",
        to: &s.target_language,
        options: wire::AppOptions::default(),
    };
    match service::translate_with_fallback(&state.client, &state.keystore, &preset, input, fallback).await {
        Ok(out) => {
            if state.gen.is_latest(gen) {
                let _ = popup::result(&app, &out.text, &out.engine);
            }
        }
        Err(e) => {
            if state.gen.is_latest(gen) {
                let _ = popup::error(&app, &e.to_string());
            }
        }
    }
    Ok(())
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
    // Atomic read-modify-write under the lock — load()+store() would interleave.
    state.keystore.update_keys(|keys| {
        keys[&provider_id] = serde_json::json!(key);
    }).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn delete_key(
    state: tauri::State<'_, Arc<Session>>,
    provider_id: String,
) -> Result<(), String> {
    state.keystore.update_keys(|keys| {
        if let Some(obj) = keys.as_object_mut() {
            obj.remove(&provider_id);
        }
    }).map_err(|e| e.to_string())?;
    Ok(())
}

/// User-initiated keystore recovery (§A fail-closed): archive the unreadable file
/// to keystore.json.broken-<ts> so the user can re-enter keys.
#[tauri::command]
fn archive_keystore(state: tauri::State<'_, Arc<Session>>) -> Result<String, String> {
    let dst = state.keystore.archive().map_err(|e| e.to_string())?;
    Ok(dst.to_string_lossy().into_owned())
}

/// User-initiated: delete the keystore entirely (fresh start).
#[tauri::command]
fn reset_keystore(state: tauri::State<'_, Arc<Session>>) -> Result<(), String> {
    state.keystore.reset().map_err(|e| e.to_string())
}

#[tauri::command]
fn key_status(
    state: tauri::State<'_, Arc<Session>>,
) -> std::collections::HashMap<String, bool> {
    // Review P1 #6: swallow the error (return empty) so frontend onMount never
    // aborts. The recovery banner reads `keystore_health` for the reason.
    let keys = match state.keystore.load() {
        Ok(k) => k,
        Err(_) => return std::collections::HashMap::new(),
    };
    let mut map = std::collections::HashMap::new();
    if let Some(obj) = keys.as_object() {
        for (k, _v) in obj {
            map.insert(k.clone(), true);
        }
    }
    map
}

#[tauri::command]
fn get_settings(app: tauri::AppHandle) -> settings::Settings {
    settings::load(&app)
}

#[tauri::command]
fn set_setting(app: tauri::AppHandle, key: String, value: String) -> Result<(), String> {
    let mut s = settings::load(&app);
    match key.as_str() {
        "default_provider" => s.default_provider = value,
        "target_language" => s.target_language = value,
        // §G: fallback_engine is opt-in. Empty string clears it (None = no
        // fallback); a non-empty value must be a known traditional-engine id,
        // validated against the registry so a typo can't silently disable fallback.
        "fallback_engine" => {
            if value.is_empty() {
                s.fallback_engine = None;
            } else if engines::find(&value).is_some() {
                s.fallback_engine = Some(value);
            } else {
                return Err(format!("unknown fallback engine: {value}"));
            }
        }
        _ => return Err(format!("unknown setting: {key}")),
    }
    settings::save(&app, &s)
}

/// Look up a word in the macOS system dictionary (spec §E: word definitions
/// where LLMs are weak). Returns plain text or None. On non-macOS / when no
/// definition is found, returns None. This command just exposes the capability;
/// wiring a UI affordance (lookup button / settings toggle) is a later UX task.
#[tauri::command]
fn lookup_dictionary(word: String) -> Option<String> {
    dict::lookup(&word)
}

/// Is Accessibility (macOS) granted? Selection capture needs it for both the AX
/// direct-read and the simulated Cmd+C. Non-macOS: always true.
#[tauri::command]
fn a11y_status() -> bool {
    a11y::enabled()
}

/// Keystore health: Ok / the fail-closed reason (corrupt / auth / unknown).
/// key_status swallows the error (returns empty) so onMount never aborts; this
/// command surfaces the reason for the recovery banner. Review P1 #6.
#[tauri::command]
fn keystore_health(state: tauri::State<'_, Arc<Session>>) -> String {
    // "" = healthy (or absent = first run). Non-empty = the fail-closed reason.
    match state.keystore.load() {
        Ok(_) => String::new(),
        Err(e) => format!("{e}"),
    }
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

    // (1) latest-wins token — allocate SYNCHRONOUSLY in the handler, BEFORE spawn.
    // Doing this inside spawn let two presses' futures start out of order so the
    // older press could grab the newer token (rev-3 race). Allocating here, in the
    // handler thread (which is serialized per-press), guarantees strict press order.
    let state = app.state::<Arc<Session>>().inner().clone();
    let gen = state.gen.next();

    let app2 = app.clone();
    tauri::async_runtime::spawn(async move {
        let state = app2.state::<Arc<Session>>().inner().clone();

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
                // Surface it visibly (review P1 #5: errors must reach the user, not
                // vanish into a popup that's never shown). Show at cursor.
                let _ = popup::error(
                    &app2,
                    if !a11y::enabled() {
                        "No selection captured. Grant Accessibility in System Settings → Privacy → Accessibility."
                    } else {
                        "No text selected."
                    },
                );
                return;
            }
            Err(e) => {
                let _ = popup::error(&app2, &e);
                return;
            }
        };

        // (4) show loading popup at the cursor.
        let _ = popup::show_at(&app2, x, y);

        // (5) translate via the §G fallback-aware service. Default provider and
        //     target language come from settings (Phase 2b); fall back to a
        //     provider-not-found error popup instead of panicking. The opt-in
        //     `fallback_engine` (Phase 3 Task 3) is resolved here — None by
        //     default, so behavior is unchanged unless the user opts in.
        let s = settings::load(&app2);
        let preset = match providers::presets().into_iter().find(|p| p.id == s.default_provider) {
            Some(p) => p,
            None => {
                let _ = popup::error(
                    &app2,
                    &format!("default provider '{}' not found", s.default_provider),
                );
                return;
            }
        };
        let fallback = s.fallback_engine.as_deref().and_then(engines::find);
        let input = service::TranslateInput {
            text: &text,
            from: "auto",
            to: &s.target_language,
            options: wire::AppOptions::default(),
        };
        match service::translate_with_fallback(&state.client, &state.keystore, &preset, input, fallback).await {
            Ok(out) => {
                if state.gen.is_latest(gen) {
                    let _ = popup::result(&app2, &out.text, &out.engine);
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

/// Show the input-translate window (bound to `Ctrl+Space`).
///
/// Unlike `on_hotkey` (Alt+Space), this is a pure UI toggle — no selection capture,
/// no translate call, no popup, no generation token. It just surfaces the
/// pre-declared `input` webview window so the user can type text into InputPanel.
fn on_input_hotkey(
    app: &tauri::AppHandle,
    _shortcut: &tauri_plugin_global_shortcut::Shortcut,
    event: tauri_plugin_global_shortcut::ShortcutEvent,
) {
    // Only act on key-down; ignore release.
    if event.state != ShortcutState::Pressed {
        return;
    }
    let app2 = app.clone();
    tauri::async_runtime::spawn(async move {
        if let Some(win) = app2.get_webview_window("input") {
            let _ = win.show();
            let _ = win.set_focus();
        }
    });
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // The global-shortcut `Builder` holds ONE shared handler (its `with_handler`
    // is a `replace`, and `build()` dispatches every registered shortcut to it by
    // `ShortcutEvent.id`). So we register both hotkeys on a single Builder and
    // route by the shortcut's string form in the handler below.
    //
    // Review P1 #10: registration must be FAULT-TOLERANT. A shortcut already owned
    // by the OS or another app must NOT bring down startup (the old code .expect-
    // ed and crashed via .run().expect()). We register each on a best-effort basis:
    // if one fails to register, we log + skip it (that hotkey just won't fire) and
    // the app still starts. (Full rebindability is a later UX feature.)
    let alt_space = tauri_plugin_global_shortcut::Shortcut::from_str("Alt+Space");
    let ctrl_space = tauri_plugin_global_shortcut::Shortcut::from_str("Ctrl+Space");
    // Capture the ids (for handler routing) BEFORE moving the shortcuts into the builder.
    let alt_space_id = alt_space.as_ref().ok().map(|s| s.id());
    let ctrl_space_id = ctrl_space.as_ref().ok().map(|s| s.id());
    let mut builder = GlobalShortcutBuilder::new();
    // Register only shortcuts that parsed (from_str is the failure point). A parse
    // failure skips that hotkey but does NOT crash startup. with_shortcut on an
    // already-parsed Shortcut won't fail, so .expect is sound here.
    if let Ok(s) = alt_space {
        builder = builder.with_shortcut(s).expect("register parsed Alt+Space");
    } else {
        log::warn!("Alt+Space parse failed — selection hotkey disabled");
    }
    if let Ok(s) = ctrl_space {
        builder = builder.with_shortcut(s).expect("register parsed Ctrl+Space");
    } else {
        log::warn!("Ctrl+Space parse failed — input hotkey disabled");
    }
    let shortcut_plugin = builder
        .with_handler(move |app, shortcut, event| {
            if Some(shortcut.id()) == ctrl_space_id {
                on_input_hotkey(app, shortcut, event);
            } else if Some(shortcut.id()) == alt_space_id {
                on_hotkey(app, shortcut, event);
            }
        })
        .build();

    tauri::Builder::default()
        // single-instance MUST be first: the keystore lock model assumes one process
        // (no cross-process lock is wired); a second instance would race on the same
        // keystore file. This plugin focuses the existing instance instead of launching
        // a second one.
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            if let Some(w) = app.get_webview_window("main") {
                let _ = w.show();
                let _ = w.set_focus();
            }
        }))
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
            // Spec §Privacy: no cross-origin redirects. Review P1 #7: a 30s total
            // request timeout so a hung connection can't freeze translate + the
            // popup indefinitely; lets wire::call classify a real Timeout.
            let client = reqwest::Client::builder()
                .redirect(reqwest::redirect::Policy::none())
                .timeout(std::time::Duration::from_secs(30))
                .connect_timeout(std::time::Duration::from_secs(10))
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
            translate_default,
            translate_clipboard,
            list_engines,
            set_key,
            delete_key,
            key_status,
            get_settings,
            set_setting,
            lookup_dictionary,
            a11y_status,
            keystore_health,
            archive_keystore,
            reset_keystore
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
