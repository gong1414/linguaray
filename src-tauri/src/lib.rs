//! LinguaRay — translation core.
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
pub mod db;
pub mod fs_acl;
pub mod uuid_util;

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use tauri::Manager;
use tauri_plugin_global_shortcut::{Builder as GlobalShortcutBuilder, GlobalShortcutExt, ShortcutState};

use crate::db::migration::{run_migration, FailpointCell, MigrationError};
use crate::db::providers::{self as db_providers, ProviderPatch, ProviderProfile};
use crate::db::readiness::DataReadiness;
use crate::db::Database;

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

/// S2a application state: the SQLite database + data-readiness gate.
///
/// Managed alongside [`Session`] as `Arc<AppState>` (existing translate/key
/// commands keep their `State<'_, Arc<Session>>` signature unchanged — least
/// disruptive). The provider commands added in step 6 take
/// `State<'_, Arc<AppState>>` and gate on [`DataReadiness`] via
/// [`require_ready_gated`] / [`require_ready_gated_write`].
///
/// ## Field semantics
///
/// - `db` — `None` when the DB file couldn't be opened (`NeedsDatabaseRecovery`).
///   Once opened it stays `Some` for the process lifetime; recovery is a
///   separate flow (archive + reset), not a re-open.
/// - `data_gate` — coarse rwlock serializing archive/reset (write) against
///   provider reads (read). Held only briefly; the DB Mutex is the real
///   per-query serializer.
/// - `readiness` — the single source of truth for "can provider commands run?"
///   Computed once at startup; mutate only from recovery commands.
/// - `db_path` / `keystore_dir` / `settings_path` — cached so recovery commands
///   (and diagnostics) don't re-resolve them. `settings_path` is `Option`:
///   `None` when `resolve_store_path` failed at startup (we don't know where
///   settings.json lives, so migration — which reads + backs up the legacy
///   settings — must be refused rather than run against a guessed path).
pub struct AppState {
    pub db: parking_lot::RwLock<Option<Arc<Database>>>,
    pub data_gate: parking_lot::RwLock<()>,
    pub readiness: parking_lot::RwLock<DataReadiness>,
    pub db_path: PathBuf,
    pub keystore_dir: PathBuf,
    pub settings_path: Option<PathBuf>,
}

/// Gating check for provider commands that ALREADY hold the `data_gate` guard.
///
/// The `_gate_guard` parameter is proof (by reference) that the caller holds the
/// gate — it is read once and discarded. Holding the gate guarantees no
/// archive/reset/recovery (which take the WRITE guard) can mutate the DB handle
/// or the readiness while this reads them, so the readiness check + `Arc` clone
/// are atomic w.r.t. those mutators.
///
/// Use this INSIDE `spawn_blocking`, after acquiring `data_gate.read()`. The
/// gate-first ordering is load-bearing: cloning the `Arc` before acquiring the
/// gate races a concurrent archive/reset/recovery that holds the write guard
/// and swaps the DB handle, handing the command a stale DB.
fn require_ready_gated(
    state: &AppState,
    _gate_guard: &parking_lot::RwLockReadGuard<'_, ()>,
) -> Result<Arc<Database>, String> {
    let readiness = state.readiness.read();
    if !readiness.is_ready() {
        return Err(format!("Database not ready: {:?}", *readiness));
    }
    drop(readiness);
    state
        .db
        .read()
        .clone()
        .ok_or_else(|| "Database not available".to_string())
}

/// Same as [`require_ready_gated`] but the proof is a WRITE guard (for commands
/// that need exclusive access: delete/reorder/toggle/set_active). Holding the
/// write guard excludes every other gate holder, so the readiness + Arc clone
/// are atomic w.r.t. the DB mutators just the same.
fn require_ready_gated_write(
    state: &AppState,
    _gate_guard: &parking_lot::RwLockWriteGuard<'_, ()>,
) -> Result<Arc<Database>, String> {
    let readiness = state.readiness.read();
    if !readiness.is_ready() {
        return Err(format!("Database not ready: {:?}", *readiness));
    }
    drop(readiness);
    state
        .db
        .read()
        .clone()
        .ok_or_else(|| "Database not available".to_string())
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
    // mid-selection-capture (which would send `__linguaray_sel_*__` to a remote
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
    let mut out: Vec<EngineInfo> = providers::presets()
        .into_iter()
        .map(EngineInfo::from_provider)
        .collect();
    // Also include built-in traditional engines (for the fallback selector).
    out.extend(engines::registry().iter().map(|e| EngineInfo::from_traditional(e.as_ref())));
    out
}

#[tauri::command]
fn set_key(
    state: tauri::State<'_, Arc<Session>>,
    app: tauri::State<'_, Arc<AppState>>,
    provider_id: String,
    key: String,
) -> Result<(), String> {
    // Acquire data_gate.read() for the duration of the keystore write so this
    // legacy command can't race archive/reset/recovery (which hold the write
    // guard). Key writes are allowed even when the DB isn't Ready — the
    // keystore is independent — but they MUST serialize against the data-gate
    // writers.
    //
    // S2a P0: typed accessor — converges the payload to v2 (a load()+store() or
    // a flat-map write would create a mixed v1/v2 structure post-migration).
    let _gate = app.data_gate.read();
    // P1 orphan-key guard: a legacy `set_key(provider_id, …)` accepts ANY
    // provider_id and writes a key under it. Post-2a the keystore is keyed by
    // `secret_ref` (a bare preset id like "openai" for legacy rows, or
    // "provider/<uuid>" for v2 rows), and every keystore key MUST be owned by a
    // non-deleted provider row (verified at migration Phase 5 + on every
    // translate). Writing a key whose `provider_id` is no row's `secret_ref`
    // creates an orphan that Phase-5 verification would later reject as
    // "keystore key has no matching active provider row", surfacing a bogus
    // recovery banner. So validate ownership BEFORE the write: the `provider_id`
    // must equal some non-deleted row's `secret_ref`. If the DB isn't available
    // (NeedsDatabaseRecovery / not yet open) we can't validate, so refuse.
    assert_secret_ref_owned(&app, &provider_id)?;
    state.keystore.set_key(&provider_id, &key).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn delete_key(
    state: tauri::State<'_, Arc<Session>>,
    app: tauri::State<'_, Arc<AppState>>,
    provider_id: String,
) -> Result<(), String> {
    // See set_key: serialize against archive/reset via the data_gate read guard.
    // S2a P0: typed accessor (idempotent — removing an absent key succeeds).
    let _gate = app.data_gate.read();
    // P1 orphan-key guard (mirrors set_key): only delete a key whose
    // `provider_id` is owned by a non-deleted provider row. This stops a stale
    // frontend from deleting a key under an arbitrary id (harmless to the DB but
    // inconsistent with the ownership invariant the keystore now maintains).
    // Unlike set_key, a delete of a key whose owner was just tombstoned is
    // legitimate (finalize_delete already purged it), so we still require the
    // row to exist — but a 'deleted' row no longer owns its secret_ref, hence
    // the `status != 'deleted'` clause. If the DB isn't available, refuse.
    assert_secret_ref_owned(&app, &provider_id)?;
    state.keystore.delete_key(&provider_id).map_err(|e| e.to_string())?;
    Ok(())
}

/// P1 orphan-key guard shared by the legacy `set_key` / `delete_key` commands.
///
/// Asserts that `provider_id` equals the `secret_ref` of some non-deleted
/// provider row in the DB. Returns `Err` (refusing the keystore write) when:
/// - the DB handle is unavailable (can't validate — refuse rather than risk an
///   orphan), or
/// - no non-deleted row owns that `secret_ref` (the write would create / touch
///   an orphan key the migration's Phase-5 verification would later reject).
///
/// MUST be called while holding `data_gate.read()` (the legacy commands acquire
/// it before calling) so the row set can't change under us.
fn assert_secret_ref_owned(app: &Arc<AppState>, provider_id: &str) -> Result<(), String> {
    let db = app
        .db
        .read()
        .clone()
        .ok_or_else(|| "cannot set/delete key: database unavailable".to_string())?;
    let owned: i64 = db
        .with_conn(|conn| -> Result<i64, crate::db::DbError> {
            let n: i64 = conn.query_row(
                "SELECT COUNT(*) FROM providers WHERE secret_ref=?1 AND status != 'deleted'",
                rusqlite::params![provider_id],
                |r| r.get(0),
            )?;
            Ok(n)
        })
        .map_err(|e| format!("cannot set/delete key: db lookup failed: {e}"))?;
    if owned == 0 {
        return Err(format!(
            "cannot set/delete key: no active provider owns secret_ref '{provider_id}'"
        ));
    }
    Ok(())
}

/// User-initiated keystore recovery (§A fail-closed): archive the unreadable file
/// to keystore.json.broken-<secs>-<nanos> so the user can re-enter keys.
///
/// Review P1 #2: recovery MUST coordinate with `AppState`, not just `Session.keystore`.
/// The command acquires the `data_gate` write lock (blocking all provider commands),
/// runs the keystore archive, then performs the DB cleanup transaction
/// (disable needs-key providers, clear active selection + consent, mark
/// migration complete) and updates `DataReadiness` based on the old state +
/// whether the DB is still usable.
#[tauri::command]
async fn archive_keystore(
    state: tauri::State<'_, Arc<AppState>>,
) -> Result<String, String> {
    let app = state.inner().clone();
    let ks_dir = app.keystore_dir.clone();
    tauri::async_runtime::spawn_blocking(move || -> Result<String, String> {
        // 1. data_gate write guard: blocks all provider reads/writes for the
        //    duration of the recovery so no command observes a half-archived
        //    keystore + un-updated DB.
        let _gate = app.data_gate.write();
        // 2. Keystore archive (existing logic, now under the gate). Construct a
        //    fresh Keystore for the canonical dir rather than reusing the
        //    Session's (which may be pointing at a fallback dir after a startup
        //    init failure).
        let ks = keystore::Keystore::new(ks_dir.clone()).map_err(|e| e.to_string())?;
        let dst = ks.archive().map_err(|e| e.to_string())?;
        let dst_str = dst.to_string_lossy().into_owned();
        // 3. DB cleanup transaction + 4. readiness update. A cleanup failure
        //    propagates: the keystore archive already happened, but the DB is
        //    now in an inconsistent state (keys gone, needs-key providers still
        //    enabled) and the user must see the error + recovery banner.
        apply_keystore_recovery_db_cleanup(&app)?;
        Ok(dst_str)
    })
    .await
    .map_err(|e| e.to_string())?
}

/// User-initiated: reset the keystore to a fresh state (archive then clear tmp).
///
/// Review P1 #2: like `archive_keystore`, this now coordinates with `AppState`
/// via the `data_gate` write lock, runs the DB cleanup transaction, and updates
/// `DataReadiness` based on the old state + DB availability.
#[tauri::command]
async fn reset_keystore(
    state: tauri::State<'_, Arc<AppState>>,
) -> Result<Option<String>, String> {
    let app = state.inner().clone();
    let ks_dir = app.keystore_dir.clone();
    tauri::async_runtime::spawn_blocking(move || -> Result<Option<String>, String> {
        let _gate = app.data_gate.write();
        let ks = keystore::Keystore::new(ks_dir.clone()).map_err(|e| e.to_string())?;
        let archived = ks
            .reset()
            .map_err(|e| e.to_string())?
            .map(|p| p.to_string_lossy().into_owned());
        // See archive_keystore: a cleanup failure propagates (keystore already
        // reset, but DB is now inconsistent).
        apply_keystore_recovery_db_cleanup(&app)?;
        Ok(archived)
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
fn key_status(
    state: tauri::State<'_, Arc<Session>>,
) -> std::collections::HashMap<String, bool> {
    // Review P1 #6: swallow the error (return empty) so frontend onMount never
    // aborts. The recovery banner reads `keystore_health` for the reason.
    //
    // S2a P0: enumerate via the typed accessor so the map is keyed by
    // `secret_ref` from the nested v2 `provider_keys` (the old raw-object walk
    // iterated the flat map and missed migrated keys).
    let refs = match state.keystore.list_provider_key_refs() {
        Ok(r) => r,
        Err(_) => return std::collections::HashMap::new(),
    };
    refs.into_iter().map(|r| (r, true)).collect()
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
/// definition is found, returns None. The dictionary product UI (select-word →
/// definition popup) is deferred to v1.x; the backend groundwork is kept here.
#[allow(dead_code)] // v1.x: dictionary UI not yet wired (removed from invoke_handler + AppManifest)
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

// ─── S2a data-readiness + provider commands ──────────────────────────────
//
// All provider commands follow the same shape:
//   1. `spawn_blocking` — rusqlite is blocking; don't hold the async runtime.
//   2. Acquire `data_gate` (read or write) INSIDE the blocking closure. The
//      parking_lot guards are `!Send`, so they must never cross an `.await`;
//      keeping them on the blocking thread for the closure's duration is the
//      one safe pattern.
//   3. `require_ready_gated` / `require_ready_gated_write` — gate on
//      DataReadiness + clone the Arc<Database>, passing the guard from step 2
//      as proof the gate is held. Acquiring the gate BEFORE the clone is
//      load-bearing: a concurrent archive/reset/recovery holds the write guard
//      while it swaps the DB handle, so cloning first would race the swap and
//      could hand the command a stale DB.
//   4. `db.with_conn(|conn| db_providers::<op>(conn, ...))`.
//
// Multi-step cross-store commands (`provider_delete`, `provider_set_key`) run
// all steps inside ONE `spawn_blocking` so the `data_gate` guard spans the whole
// operation on a single thread. The DB Mutex ↔ keystore flock lock-order rule is
// still respected: the DB guard (`with_conn` closure) is released before each
// keystore step (the keystore takes only its own flock).

/// Returns the serialized [`DataReadiness`] so the frontend can drive the
/// recovery banner. Always available (no readiness gate) — it's how the UI
/// discovers the gate is closed in the first place.
#[tauri::command]
fn get_data_readiness(state: tauri::State<'_, Arc<AppState>>) -> String {
    let r = state.readiness.read();
    serde_json::to_string(&*r).unwrap_or_else(|_| "{\"state\":\"migration_incomplete\"}".into())
}

/// List active provider profiles (`status='active'`), ordered by `sort_order`.
#[tauri::command]
async fn provider_list(
    state: tauri::State<'_, Arc<AppState>>,
) -> Result<Vec<ProviderProfile>, String> {
    let app = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        // Acquire the gate FIRST so the readiness check + Arc clone are atomic
        // w.r.t. archive/reset/recovery (which take the write guard + swap the
        // DB handle). Cloning the Arc before the gate (the old shape) raced the
        // swap and could hand the command a stale DB.
        let _gate = app.data_gate.read();
        let db = require_ready_gated(&app, &_gate)?;
        db.with_conn(|conn| db_providers::list(conn)).map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}

/// Create a new provider from a template (preset id). The preset catalog
/// derives protocol/endpoint/default-model/needs_key; caller values override
/// endpoint/model when non-empty.
#[tauri::command]
async fn provider_create(
    state: tauri::State<'_, Arc<AppState>>,
    template_id: String,
    name: String,
    endpoint: String,
    model: Option<String>,
) -> Result<ProviderProfile, String> {
    let app = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        // Acquire the gate FIRST (see provider_list).
        let _gate = app.data_gate.read();
        let db = require_ready_gated(&app, &_gate)?;
        db.with_conn(|conn| {
            db_providers::create(conn, &template_id, &name, &endpoint, model.as_deref())
        })
        .map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}

/// Apply a partial patch to a provider. An endpoint change is validated and may
/// invalidate the parallel consent (see `db_providers::update`).
#[tauri::command]
async fn provider_update(
    state: tauri::State<'_, Arc<AppState>>,
    uuid: String,
    patch: ProviderPatch,
) -> Result<ProviderProfile, String> {
    let app = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        // Acquire the gate FIRST (see provider_list).
        let _gate = app.data_gate.read();
        let db = require_ready_gated(&app, &_gate)?;
        db.with_conn(|conn| db_providers::update(conn, &uuid, &patch)).map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}

/// Duplicate a provider. New UUID, new `secret_ref`, keyless (the original key
/// is never copied).
#[tauri::command]
async fn provider_duplicate(
    state: tauri::State<'_, Arc<AppState>>,
    uuid: String,
) -> Result<ProviderProfile, String> {
    let app = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        // Acquire the gate FIRST (see provider_list).
        let _gate = app.data_gate.read();
        let db = require_ready_gated(&app, &_gate)?;
        db.with_conn(|conn| db_providers::duplicate(conn, &uuid)).map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}

/// Begin the 3-step delete (mark `deleting`, evict from slots), purge the key
/// from the keystore, then finalize the tombstone. Each step is committed before
/// the next; the lock-order rule (DB Mutex and keystore flock never nested) is
/// preserved by releasing the DB guard between steps. All three steps run on one
/// blocking thread so the `data_gate` write guard spans the whole operation.
#[tauri::command]
async fn provider_delete(
    state: tauri::State<'_, Arc<AppState>>,
    uuid: String,
) -> Result<(), String> {
    let app = state.inner().clone();
    let keystore_dir = app.keystore_dir.clone();
    tauri::async_runtime::spawn_blocking(move || -> Result<(), String> {
        // Write guard: a delete mutates selection slots + status; no reader/other
        // writer may interleave. Held for all 3 steps (the DB Mutex + keystore
        // flock are still released between steps inside their own calls).
        // Acquire the gate FIRST (see provider_list) so the readiness check +
        // Arc clone are atomic w.r.t. the DB swap.
        let _gate = app.data_gate.write();
        let db = require_ready_gated_write(&app, &_gate)?;

        // Step 1: begin_delete under the DB Mutex → returns the secret_ref. The
        // DB guard (with_conn closure) is released before the keystore step.
        let secret_ref = db
            .with_conn(|conn| db_providers::begin_delete(conn, &uuid))
            .map_err(|e| e.to_string())?;

        // Step 2: purge the key (keystore flock only, DB NOT locked). Uses the
        // typed `delete_key` RMW so the payload converges to v2. Idempotent — a
        // missing key is a successful no-op.
        let ks = keystore::Keystore::new(keystore_dir).map_err(|e| e.to_string())?;
        ks.delete_key(&secret_ref).map_err(|e| e.to_string())?;

        // Step 3: finalize the tombstone (DB Mutex only, keystore NOT locked).
        db.with_conn(|conn| db_providers::finalize_delete(conn, &uuid))
            .map_err(|e| e.to_string())?;
        Ok(())
    })
    .await
    .map_err(|e| e.to_string())?
}

/// Re-assign `sort_order` to the given UUID order. The list MUST be exactly the
/// set of active UUIDs.
#[tauri::command]
async fn provider_reorder(
    state: tauri::State<'_, Arc<AppState>>,
    uuids: Vec<String>,
) -> Result<(), String> {
    let app = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        // Acquire the gate FIRST (see provider_list).
        let _gate = app.data_gate.write();
        let db = require_ready_gated_write(&app, &_gate)?;
        db.with_conn(|conn| db_providers::reorder(conn, &uuids)).map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}

/// Flip `enabled`. Disabling also evicts the row from selection slots and
/// invalidates parallel consent (mirrors `begin_delete`).
#[tauri::command]
async fn provider_toggle(
    state: tauri::State<'_, Arc<AppState>>,
    uuid: String,
    enabled: bool,
) -> Result<(), String> {
    let app = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        // Acquire the gate FIRST (see provider_list).
        let _gate = app.data_gate.write();
        let db = require_ready_gated_write(&app, &_gate)?;
        db.with_conn(|conn| db_providers::toggle(conn, &uuid, enabled)).map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
    .map(|_| ())
}

/// Set/clear a provider's API key in the keystore. The provider row's
/// `secret_ref` names the key. Cross-store (DB read → keystore write) but the
/// two locks are never held at once: the DB read releases before the keystore
/// RMW begins (lock-order rule). Both steps run on one blocking thread.
#[tauri::command]
async fn provider_set_key(
    state: tauri::State<'_, Arc<AppState>>,
    uuid: String,
    key: String,
) -> Result<(), String> {
    let app = state.inner().clone();
    let keystore_dir = app.keystore_dir.clone();
    tauri::async_runtime::spawn_blocking(move || -> Result<(), String> {
        // Acquire the gate FIRST (see provider_list) so the readiness check +
        // Arc clone are atomic w.r.t. the DB swap.
        let _gate = app.data_gate.read();
        let db = require_ready_gated(&app, &_gate)?;

        // 1. Read the secret_ref + status under the DB Mutex, then release.
        //    Reject deleting/deleted profiles: writing a key for a row that's
        //    mid-deletion would resurrect a secret whose owner is being torn
        //    down, and the next finalize_delete would orphan it silently.
        let secret_ref = db
            .with_conn(|conn| {
                let p = db_providers::get(conn, &uuid)?;
                if p.status != "active" {
                    return Err(crate::db::DbError::Integrity(format!(
                        "provider {} status is '{}'; cannot set key on a non-active profile",
                        uuid, p.status
                    )));
                }
                Ok(p.secret_ref)
            })
            .map_err(|e| e.to_string())?;

        // 2. Keystore RMW (flock only, DB NOT locked). Typed accessor converges
        //    the payload to v2 and handles both v1 flat-map and v2 shapes.
        let ks = keystore::Keystore::new(keystore_dir).map_err(|e| e.to_string())?;
        ks.set_key(&secret_ref, &key).map_err(|e| e.to_string())?;
        Ok(())
    })
    .await
    .map_err(|e| e.to_string())?
}

/// Set the active selection (primary, parallel, fallback). Empty primary and
/// empty parallel list mean "no selection". Validates the selection against the
/// active provider set before writing.
///
/// Review P1 #3 (multi-engine consent): when `parallel` is non-empty, the
/// backend recomputes the canonical consent scope and compares it against the
/// stored scope. A mismatch (no prior consent, or a different parallel set)
/// returns [`db_providers::ConsentError::ConsentRequired`] carrying the
/// `actual_scope`. The frontend shows the consent dialog, then calls
/// [`provider_confirm_and_set_active`] with `expected_scope = actual_scope` to
/// record the approval. A matching scope (re-affirming the same selection) is
/// written immediately.
#[tauri::command]
async fn provider_set_active(
    state: tauri::State<'_, Arc<AppState>>,
    primary: String,
    parallel: Vec<String>,
    fallback: Option<String>,
) -> Result<(), String> {
    let app = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || -> Result<(), String> {
        // Acquire the gate FIRST (see provider_list).
        let _gate = app.data_gate.write();
        let db = require_ready_gated_write(&app, &_gate)?;
        // The `with_conn` closure must return Result<_, DbError> (Database's
        // contract). We carry the consent-required signal out via a SetActiveOutcome
        // so the outer closure can map it to the frontend-facing string without
        // smuggling a ConsentError through the DbError boundary.
        //
        // P1 #4: ALL reads (list, compute_scope, read_consent_scope) + the
        // write run inside ONE transaction so a concurrent writer can't change
        // the active set between validation and the slot write.
        let outcome = db
            .with_conn(|conn| -> Result<SetActiveOutcome, DbErr> {
                let tx = conn.transaction()?;
                // Validate against the active set BEFORE writing.
                let active = db_providers::list(&tx)?;
                db_providers::validate_active_selection(
                    &primary,
                    &parallel,
                    fallback.as_deref(),
                    &active,
                )?;
                // P1 #3: parallel consent gate. A non-empty parallel selection
                // requires explicit user consent; if the stored scope doesn't
                // match the recomputed scope, return ConsentRequired so the
                // frontend can prompt. A matching scope (re-affirming the same
                // set) is allowed through without re-prompting.
                if !parallel.is_empty() {
                    let actual = db_providers::compute_scope(&primary, &parallel, &active)
                        .map_err(consent_to_db)?;
                    let stored = db_providers::read_consent_scope(&tx)?;
                    if stored.as_deref() != Some(actual.as_str()) {
                        // No write — drop the tx (rolls back, which is a no-op
                        // since nothing was written) and surface NeedsConsent.
                        return Ok(SetActiveOutcome::NeedsConsent { actual_scope: actual });
                    }
                }
                // Scope matches (or parallel is empty → no consent needed):
                // write the three slots. Clear prior consent only when there's
                // no parallel set (membership went to a non-consented shape); a
                // matching-scope write keeps the consent as-is.
                if parallel.is_empty() {
                    set_active_slots(&tx, &primary, &parallel, fallback.as_deref())?;
                } else {
                    set_active_slots_keep_consent(
                        &tx,
                        &primary,
                        &parallel,
                        fallback.as_deref(),
                    )?;
                }
                tx.commit()?;
                Ok(SetActiveOutcome::Written)
            })
            .map_err(|e| e.to_string())?;
        match outcome {
            SetActiveOutcome::Written => Ok(()),
            SetActiveOutcome::NeedsConsent { actual_scope } => Err(format!(
                "consent_required:{actual_scope}"
            )),
        }
    })
    .await
    .map_err(|e| e.to_string())?
}

/// Confirm the user's explicit consent for a parallel selection and write it
/// atomically (P1 #3).
///
/// Single DB transaction that:
/// 1. Re-reads ALL active providers (inside the tx — no TOCTOU between the
///    `provider_set_active` probe and this confirm).
/// 2. Validates the candidate selection (`validate_active_selection`).
/// 3. Backend recomputes canonical scope via `compute_scope`.
/// 4. Asserts the frontend's `expected_scope` matches the backend's
///    `actual_scope` (rejects a stale frontend that raced a provider change).
/// 5. Writes the selection + consent scope + bumped version in the SAME tx.
#[tauri::command]
async fn provider_confirm_and_set_active(
    state: tauri::State<'_, Arc<AppState>>,
    primary: String,
    parallel: Vec<String>,
    fallback: Option<String>,
    expected_scope: String,
) -> Result<i64, String> {
    let app = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || -> Result<i64, String> {
        // Acquire the gate FIRST (see provider_list).
        let _gate = app.data_gate.write();
        let db = require_ready_gated_write(&app, &_gate)?;
        db.with_conn(|conn| -> Result<i64, DbErr> {
            // P1 #4: ALL reads + validation + scope computation + writes run in
            // ONE transaction so no concurrent writer can change the active set
            // between the probe and the consented write.
            let tx = conn.transaction()?;
            // 1. Re-read inside the tx (no TOCTOU).
            let active = db_providers::list(&tx)?;
            // 2. Validate.
            db_providers::validate_active_selection(
                &primary,
                &parallel,
                fallback.as_deref(),
                &active,
            )?;
            // 3. Recompute scope.
            let actual_scope = db_providers::compute_scope(&primary, &parallel, &active)
                .map_err(consent_to_db)?;
            // 4. Assert frontend's expectation matches backend reality. A
            //    mismatch is a stale-frontend guard; surface as Integrity so it
            //    propagates as a string error (the frontend re-prompts).
            if expected_scope != actual_scope {
                return Err(DbErr::Integrity(format!(
                    "consent_required:{actual_scope}"
                )));
            }
            // 5. Bump version + write selection + record scope atomically (same tx).
            let new_version = write_consented_selection(
                &tx,
                &primary,
                &parallel,
                fallback.as_deref(),
                &actual_scope,
            )?;
            tx.commit()?;
            Ok(new_version)
        })
        .map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}

// ─── P1 #8: missing commands (provider diagnostics + DB recovery) ─────────

/// One selectable model for a provider. The full HTTP model-list fetch is S3
/// scope; for now [`provider_get_models`] returns a preset-derived list so the
/// UI has something to render.
#[derive(Debug, Clone, Serialize)]
pub struct ModelInfo {
    pub id: String,
    pub label: String,
}

/// Result of a connection probe (P1 #8). `ok` + a human-readable message; the
/// full connection-test HTTP flow is S3 scope, so the current implementation is
/// a best-effort "reachable" check.
#[derive(Debug, Clone, Serialize)]
pub struct ConnectionResult {
    pub ok: bool,
    pub message: String,
}

/// List the models a provider can use (P1 #8).
///
/// Reads the provider profile snapshot in `spawn_blocking` (so the async
/// runtime isn't held by rusqlite), then returns a preset-derived model list.
/// The preset catalog is the source of the default model; the profile's own
/// `model` (if set) is surfaced first as the "current" choice. The full HTTP
/// `/models` fetch is S3 scope.
#[tauri::command]
async fn provider_get_models(
    state: tauri::State<'_, Arc<AppState>>,
    uuid: String,
) -> Result<Vec<ModelInfo>, String> {
    let app = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        // Acquire the gate FIRST (see provider_list).
        let _gate = app.data_gate.read();
        let db = require_ready_gated(&app, &_gate)?;
        let profile = db
            .with_conn(|conn| db_providers::get(conn, &uuid))
            .map_err(|e| e.to_string())?;
        let mut out: Vec<ModelInfo> = Vec::new();
        // The profile's configured model is the "current" entry, surfaced first.
        if let Some(m) = &profile.model {
            if !m.is_empty() {
                out.push(ModelInfo {
                    id: m.clone(),
                    label: m.clone(),
                });
            }
        }
        // Append the preset default model as a secondary option when it differs
        // from the configured one (so the UI can offer "reset to default").
        if let Some(p) = providers::presets().into_iter().find(|p| p.id == profile.template_id) {
            if profile.model.as_deref() != Some(p.default_model.as_str()) {
                out.push(ModelInfo {
                    id: p.default_model.clone(),
                    label: format!("{} (default)", p.default_model),
                });
            }
        }
        Ok(out)
    })
    .await
    .map_err(|e| e.to_string())?
}

/// Probe whether a provider is reachable (P1 #8).
///
/// Reads the profile snapshot in `spawn_blocking`, then runs an async HEAD-ish
/// request against the endpoint. Full connection testing (auth-balanced probe,
/// latency buckets, quota introspection) is S3 scope; for now this is a simple
/// "could we establish a TCP/TLS connection" check that classifies the outcome.
#[tauri::command]
async fn provider_test_connection(
    state: tauri::State<'_, Arc<AppState>>,
    session: tauri::State<'_, Arc<Session>>,
    uuid: String,
) -> Result<ConnectionResult, String> {
    let app = state.inner().clone();
    // Read the profile on a blocking thread, then hand the endpoint back to the
    // async caller for the HTTP probe.
    let profile = tauri::async_runtime::spawn_blocking(move || -> Result<db_providers::ProviderProfile, String> {
        // Acquire the gate FIRST (see provider_list).
        let _gate = app.data_gate.read();
        let db = require_ready_gated(&app, &_gate)?;
        db.with_conn(|conn| db_providers::get(conn, &uuid))
            .map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())??;

    if profile.endpoint.is_empty() {
        return Ok(ConnectionResult {
            ok: false,
            message: "endpoint not configured".into(),
        });
    }
    // Validate the endpoint shape before sending any bytes.
    if let Err(e) = providers::validate_endpoint(&profile.endpoint) {
        return Ok(ConnectionResult {
            ok: false,
            message: format!("invalid endpoint: {e}"),
        });
    }
    // Best-effort reachability probe. We don't care about the response body —
    // any HTTP response (even a 401/404) means the endpoint is reachable; only
    // a transport-level failure (connect/timeout/TLS) counts as "not ok".
    let req = session.client.get(&profile.endpoint).send().await;
    match req {
        Ok(resp) => Ok(ConnectionResult {
            ok: true,
            message: format!("reachable (HTTP {})", resp.status().as_u16()),
        }),
        Err(e) => Ok(ConnectionResult {
            ok: false,
            message: format!("connection failed: {e}"),
        }),
    }
}

/// User-initiated database recovery (P1 #8).
///
/// Implements the frozen close/rename/reopen state machine so the DB file
/// handle is released BEFORE the rename (otherwise Windows refuses to rename a
/// file with an open SQLite handle, and a cloned `Arc<Database>` could keep
/// serving queries against the wrong file). Steps:
///
/// 1. Acquire `data_gate.write()` FIRST (blocks every provider command — no
///    new `with_conn` call can start while we're tearing down).
/// 2. Take the `Arc<Database>` out of the slot (`db.write().take()`).
/// 3. `Arc::try_unwrap` — the write gate guarantees no in-flight command holds
///    a clone; if one still does (a programming bug), restore the slot and
///    bail instead of leaving split-brain handles.
/// 4. `Database::close(self)` — release the SQLite file handle. On failure,
///    reopen the slot from the original path (the file still exists there) and
///    bail.
/// 5. `fs::rename(db_path, broken_path)` — the file handle is gone, so this
///    succeeds even on Windows. If it fails, the file is still at the original
///    path: reopen it, restore the slot, and bail.
/// 6. Open a fresh DB at the original path + run migration.
/// 7. `resume_deletions` against the fresh DB. A failure here is a real
///    problem (not just a logged best-effort one) → MigrationIncomplete, NOT
///    Ready.
/// 8. On success install the new handle + Ready.
///
/// Any failure AFTER the rename leaves the slot `None` and readiness
/// `NeedsDatabaseRecovery` — the file is gone, so the user must retry the
/// recovery (which will recreate it). Any failure BEFORE the rename leaves the
/// original DB untouched and usable.
#[tauri::command]
async fn archive_database(
    state: tauri::State<'_, Arc<AppState>>,
) -> Result<String, String> {
    let app = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || -> Result<String, String> {
        // 1. data_gate write guard for the whole operation. Acquired FIRST so
        //    require_ready (which clones the Arc) cannot race us: once we hold
        //    the write guard no provider command can start a new with_conn.
        let _gate = app.data_gate.write();

        let db_path = app.db_path.clone();

        // 2-4. Close the existing connection (if any) so the file handle is
        //      released before the rename. The slot is left None across the
        //      rename so a concurrent reader observes "no DB" rather than a
        //      handle pointing at a renamed file.
        let closed = match app.db.write().take() {
            Some(arc) => match Arc::try_unwrap(arc) {
                Ok(owned) => match owned.close() {
                    Ok(()) => true,
                    Err((recovered_db, e)) => {
                        // Close failed — the file handle may still be open but
                        // the DB object is still usable. Restore the slot so the
                        // app keeps running against the original file and bail.
                        let reason = e.to_string();
                        *app.db.write() = Some(Arc::new(recovered_db));
                        return Err(format!("close linguaray.db before archive: {reason}"));
                    }
                },
                Err(arc) => {
                    // Someone still holds a clone. The write gate should make
                    // this impossible; restore + bail rather than risk a
                    // split-brain handle racing the rename.
                    *app.db.write() = Some(arc);
                    return Err(
                        "archive_database: DB handle still in use (data_gate violated)".into(),
                    );
                }
            },
            None => false, // No handle to close (e.g. NeedsDatabaseRecovery).
        };

        // 5. Rename the DB file aside (recoverable). If the file doesn't exist,
        //    there's nothing to archive — proceed straight to opening a fresh
        //    one. On rename failure the original file is still at db_path, so
        //    we can restore the slot by reopening it.
        let archived_path = if db_path.exists() {
            // Nanosecond-precision suffix so two archives taken within the same
            // second don't collide (a second-only suffix would let a rapid
            // second archive silently overwrite the first via the rename).
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default();
            let dst = db_path.with_extension(format!("db.broken-{}-{}", now.as_secs(), now.subsec_nanos()));
            match std::fs::rename(&db_path, &dst) {
                Ok(()) => dst.to_string_lossy().into_owned(),
                Err(e) => {
                    // Rename failed: the file is still at the original path.
                    // Restore the slot by reopening it (only if we previously
                    // held a handle, i.e. the app was using this DB) so the
                    // user isn't left with a None slot over a usable file.
                    if closed {
                        if let Ok(db) = Database::open(&db_path) {
                            *app.db.write() = Some(Arc::new(db));
                        } else {
                            *app.readiness.write() = DataReadiness::NeedsDatabaseRecovery {
                                reason: format!(
                                    "rename {} failed ({e}) and reopen also failed",
                                    db_path.display()
                                ),
                            };
                        }
                    }
                    return Err(format!("rename linguaray.db aside: {e}"));
                }
            }
        } else {
            String::new()
        };

        // 6-8. Open a fresh DB at the original path + migrate + resume.
        // `settings_path = None` means the canonical settings path couldn't be
        // resolved at startup. Migration reads + backs up the legacy settings
        // file, so running it against a guessed path would touch the wrong file
        // (on Windows the store plugin targets AppData (Roaming) while `dir` is
        // AppLocalData (Local)). Refuse: leave the DB installed but incomplete
        // so the next startup (which re-resolves the path) retries.
        let settings_path = match app.settings_path.as_ref() {
            Some(p) => p.clone(),
            None => {
                let reason = "settings path unresolved; cannot migrate".to_string();
                *app.readiness.write() = DataReadiness::migration_incomplete(
                    "archive_database",
                    reason.clone(),
                );
                // Still install the handle so a later retry can proceed.
                if let Ok(db) = Database::open(&db_path) {
                    *app.db.write() = Some(Arc::new(db));
                }
                return Err(reason);
            }
        };
        match Database::open(&db_path) {
            Ok(db) => {
                let db = Arc::new(db);
                let fp = FailpointCell::none();
                match run_migration(&db, &app.keystore_dir, &settings_path, &fp) {
                    Ok(()) => {}
                    Err(MigrationError::NeedsKeystoreRecovery(reason)) => {
                        // A keystore-needs-recovery outcome must surface as
                        // NeedsKeystoreRecovery (NOT MigrationIncomplete) so the
                        // recovery banner routes the user to the keystore flow
                        // rather than the migration flow.
                        let msg = reason.clone();
                        *app.readiness.write() = DataReadiness::NeedsKeystoreRecovery {
                            reason: msg.clone(),
                        };
                        // Still install the handle so a later retry can proceed.
                        *app.db.write() = Some(db);
                        return Err(msg);
                    }
                    Err(e) => {
                        let reason = format!("migration after archive: {e}");
                        *app.readiness.write() = DataReadiness::migration_incomplete(
                            "archive_database",
                            reason.clone(),
                        );
                        // Still install the handle so a later recovery/migration
                        // retry can proceed (NeedsDatabaseRecovery would hide the
                        // partially-migrated DB).
                        *app.db.write() = Some(db);
                        return Err(reason);
                    }
                }
                // Resume in-flight deletes against the fresh DB. A failure here
                // is a real consistency problem — surface MigrationIncomplete
                // (NOT Ready) so the user sees the recovery banner.
                if let Err(e) =
                    crate::db::delete::provider_resume_deletions(&db, &app.keystore_dir)
                {
                    let reason = format!("resume_deletions after archive: {e}");
                    *app.readiness.write() = DataReadiness::migration_incomplete(
                        "archive_database",
                        reason.clone(),
                    );
                    *app.db.write() = Some(db);
                    return Err(reason);
                }
                *app.db.write() = Some(db);
                *app.readiness.write() = DataReadiness::Ready;
                Ok(archived_path)
            }
            Err(e) => {
                // The file was renamed away (or never existed); the reopen
                // failed too. There's nothing to serve — NeedsDatabaseRecovery.
                let reason = format!("reopen linguaray.db: {e}");
                *app.readiness.write() = DataReadiness::NeedsDatabaseRecovery {
                    reason: reason.clone(),
                };
                Err(reason)
            }
        }
    })
    .await
    .map_err(|e| e.to_string())?
}

// ─── provider command helpers ──────────────────────────────────────────────

/// Type alias so the closures above can name the error without importing the
/// full path each time.
type DbErr = crate::db::DbError;

/// Type alias for the consent-computation error (P1 #3).
type ConsentErr = db_providers::ConsentError;

/// Outcome of a `provider_set_active` DB transaction (P1 #3). Carries the
/// consent-required signal out of the `with_conn` closure (whose error type is
/// fixed to `DbError`) so the command can surface it to the frontend.
enum SetActiveOutcome {
    /// Selection written (no consent needed, or scope already matched).
    Written,
    /// A non-empty parallel selection needs explicit consent; carries the
    /// canonical scope the frontend must echo back via
    /// `provider_confirm_and_set_active`.
    NeedsConsent { actual_scope: String },
}

/// Map a [`ConsentError`] (other than `ConsentRequired`, which is handled by
/// the caller via `SetActiveOutcome`) into a [`DbError`] so it can cross the
/// `with_conn` boundary. The consent-required arm is mapped to an Integrity
/// error carrying the scope (the only place this fires is the
/// `provider_confirm_and_set_active` stale-scope guard).
fn consent_to_db(e: ConsentErr) -> DbErr {
    match e {
        ConsentErr::Db(d) => d,
        other => DbErr::Integrity(other.to_string()),
    }
}

/// Write the primary/parallel/fallback slots in `preferences` + null consent.
/// Runs against the caller's transaction (P1 #4: reads + writes in ONE tx) —
/// no inner transaction is opened.
fn set_active_slots(
    tx: &rusqlite::Transaction<'_>,
    primary: &str,
    parallel: &[String],
    fallback: Option<&str>,
) -> Result<(), DbErr> {
    let primary_val = if primary.is_empty() {
        None
    } else {
        Some(primary)
    };
    let parallel_json = serde_json::to_string(parallel).unwrap_or_else(|_| "[]".into());
    let fallback_val = fallback.filter(|s| !s.is_empty());
    tx.execute(
        "UPDATE preferences SET primary_uuid=?1, parallel_uuids=?2, fallback_uuid=?3, \
         parallel_consent_version=NULL, parallel_consent_scope=NULL WHERE id=1",
        rusqlite::params![
            primary_val,
            parallel_json,
            fallback_val,
        ],
    )?;
    Ok(())
}

/// Shared DB cleanup + readiness update for keystore recovery
/// (`archive_keystore` / `reset_keystore`). Review P1 #2.
///
/// Must be called while the caller holds the `data_gate` write guard so no
/// provider command races the cleanup. Performs (inside one DB transaction):
/// 1. `UPDATE providers SET enabled=0 WHERE needs_key=1` — a provider that needs
///    a key can't be used until the user re-enters one (the archived keystore
///    just lost them all). Keyless providers (Ollama, traditional engines) keep
///    their enabled state.
/// 2. Clear `primary_uuid`, `parallel_uuids`, `fallback_uuid` — the prior
///    selection referenced providers whose keys may be gone, so a stale
///    selection can't drive a translate.
/// 3. Clear `parallel_consent_version` / `parallel_consent_scope` — consent was
///    given for the now-archived key set.
/// 4. `UPDATE _schema_migrations SET migration_complete=1` — a recovery completes
///    migration (the DB is now in a known-good state, just without keys).
///    **Guaranteed only when the OLD readiness was `Ready` or
///    `NeedsKeystoreRecovery`.** When the OLD readiness was
///    `MigrationIncomplete`, this UPDATE is SKIPPED: the prior migration did
///    not reach `Complete` for a reason this archive does not address (a
///    half-applied schema, a corrupt settings file, a resume-deletions fault),
///    so writing `migration_complete=1` would persist a half-migrated DB as
///    complete and permanently lock it (the next startup sees `Complete` and
///    skips migration).
///
/// Returns `Err` if the cleanup transaction fails (so the caller can surface
/// the failure — previously it was only logged and readiness was bumped to
/// Ready, hiding a real consistency break).
///
/// Readiness transition rules:
/// - DB cleanup tx FAILED → `MigrationIncomplete` (the DB is in an unknown
///   state; the user must see the recovery banner). The OLD readiness is NOT
///   promoted.
/// - Old `Ready` or `NeedsKeystoreRecovery` + cleanup OK + DB exists → `Ready`
///   (the keystore problem is fixed and the DB is consistent).
/// - Old `NeedsDatabaseRecovery` → kept as-is (a keystore archive does NOT fix
///   a corrupt/unopenable DB). The cleanup tx isn't attempted (no DB handle).
/// - Old `MigrationIncomplete` → kept as-is even when the cleanup succeeds. The
///   prior migration was incomplete for a reason this archive doesn't address
///   (a half-applied schema, a corrupt settings file, a resume-deletions
///   fault); promoting to Ready would hide that. The user must retry the
///   recovery that targets the migration itself.
fn apply_keystore_recovery_db_cleanup(app: &Arc<AppState>) -> Result<(), String> {
    // Capture the OLD readiness BEFORE any mutation so the post-state logic can
    // branch on it, and so the cleanup tx knows whether it may mark migration
    // complete (only Ready / NeedsKeystoreRecovery were fully migrated before
    // the archive; MigrationIncomplete must NOT be promoted on disk).
    let old_readiness = app.readiness.read().clone();
    let may_mark_complete = matches!(
        &old_readiness,
        DataReadiness::Ready | DataReadiness::NeedsKeystoreRecovery { .. }
    );

    // Try the DB cleanup only when a DB handle exists. A NeedsDatabaseRecovery
    // state means there's no handle; the cleanup is a no-op and we keep that
    // state (a keystore archive doesn't fix a corrupt DB).
    //
    // `cleanup_result` is Ok(true) when a handle existed and the tx succeeded;
    // Ok(false) when there was no handle to clean up (NeedsDatabaseRecovery);
    // Err(_) when a handle existed but the tx failed (the DB is now in an
    // unknown state).
    let cleanup_result: Result<bool, String> = match app.db.read().clone() {
        Some(db) => db
            .with_conn(|conn| {
                let tx = conn.transaction()?;
                // 1. Disable needs-key providers (their keys are gone after archive).
                tx.execute(
                    "UPDATE providers SET enabled=0 WHERE needs_key=1",
                    [],
                )?;
                // 2-3. Clear active selection + consent.
                tx.execute(
                    "UPDATE preferences SET primary_uuid=NULL, parallel_uuids='[]', \
                     fallback_uuid=NULL, parallel_consent_version=NULL, \
                     parallel_consent_scope=NULL WHERE id=1",
                    [],
                )?;
                // 4. Mark migration complete — ONLY when the OLD readiness was
                //    Ready / NeedsKeystoreRecovery. When it was
                //    MigrationIncomplete, the prior migration did not reach
                //    Complete and this archive doesn't fix that; writing
                //    complete=1 would persist the half-migrated DB as final and
                //    the next startup would skip migration entirely.
                if may_mark_complete {
                    tx.execute(
                        "UPDATE _schema_migrations SET migration_complete=1 WHERE id=1",
                        [],
                    )?;
                }
                tx.commit()?;
                Ok(())
            })
            .map(|_| true)
            .map_err(|e| {
                // Surface the failure: the cleanup tx is part of the recovery's
                // atomic contract; logging + bumping to Ready would hide a real
                // consistency break.
                log::error!("keystore recovery DB cleanup failed: {e}");
                e.to_string()
            }),
        None => Ok(false), // No DB handle (NeedsDatabaseRecovery): nothing to clean up.
    };

    // Compute the new readiness. The cleanup tx FAILED → MigrationIncomplete
    // (the DB is in an unknown state). On success, the transition depends on
    // the OLD readiness (see the doc comment above).
    let new_readiness = match cleanup_result {
        Err(_) => DataReadiness::migration_incomplete(
            "keystore_recovery_cleanup",
            "DB cleanup transaction failed after keystore archive",
        ),
        // No DB handle (NeedsDatabaseRecovery): keep that state regardless of
        // whether the archive succeeded.
        Ok(false) => match &old_readiness {
            DataReadiness::NeedsDatabaseRecovery { reason } => {
                DataReadiness::NeedsDatabaseRecovery {
                    reason: reason.clone(),
                }
            }
            // Defensive: cleanup is a no-op but there's no DB handle, so do NOT
            // claim Ready (Ready implies an open DB). Keep the old state.
            other => other.clone(),
        },
        // Cleanup succeeded against a real DB. Only Ready / NeedsKeystoreRecovery
        // promote to Ready; MigrationIncomplete is kept (the prior migration was
        // incomplete for a reason this archive doesn't fix); NeedsDatabaseRecovery
        // can't reach this arm (no handle → Ok(false)).
        Ok(true) => match &old_readiness {
            DataReadiness::Ready | DataReadiness::NeedsKeystoreRecovery { .. } => {
                DataReadiness::Ready
            }
            DataReadiness::MigrationIncomplete { checkpoint, reason } => {
                DataReadiness::MigrationIncomplete {
                    checkpoint: checkpoint.clone(),
                    reason: reason.clone(),
                }
            }
            DataReadiness::NeedsDatabaseRecovery { reason } => {
                DataReadiness::NeedsDatabaseRecovery {
                    reason: reason.clone(),
                }
            }
        },
    };
    *app.readiness.write() = new_readiness;
    // Propagate a cleanup-tx failure so the caller (archive_keystore /
    // reset_keystore) surfaces it to the frontend; the readiness has already
    // been set to MigrationIncomplete above.
    cleanup_result.map(|_| ())
}

/// Like [`set_active_slots`] but PRESERVES the prior parallel consent
/// (version + scope). Used by `provider_set_active` when the recomputed scope
/// matches the stored scope (re-affirming the same selection): we update the
/// slot pointers without invalidating consent. Runs against the caller's
/// transaction (P1 #4) — no inner transaction is opened.
fn set_active_slots_keep_consent(
    tx: &rusqlite::Transaction<'_>,
    primary: &str,
    parallel: &[String],
    fallback: Option<&str>,
) -> Result<(), DbErr> {
    let primary_val = if primary.is_empty() {
        None
    } else {
        Some(primary)
    };
    let parallel_json = serde_json::to_string(parallel).unwrap_or_else(|_| "[]".into());
    let fallback_val = fallback.filter(|s| !s.is_empty());
    tx.execute(
        "UPDATE preferences SET primary_uuid=?1, parallel_uuids=?2, fallback_uuid=?3 WHERE id=1",
        rusqlite::params![primary_val, parallel_json, fallback_val],
    )?;
    Ok(())
}

/// Write the active selection AND record the consent (scope + bumped version)
/// against the caller's transaction (P1 #3 + P1 #4: ALL reads + writes in ONE
/// tx). Returns the new consent version. The caller owns the transaction and
/// commits it.
fn write_consented_selection(
    tx: &rusqlite::Transaction<'_>,
    primary: &str,
    parallel: &[String],
    fallback: Option<&str>,
    scope: &str,
) -> Result<i64, DbErr> {
    let primary_val = if primary.is_empty() { None } else { Some(primary) };
    let parallel_json = serde_json::to_string(parallel).unwrap_or_else(|_| "[]".into());
    let fallback_val = fallback.filter(|s| !s.is_empty());
    // Bump the version: COALESCE(NULL, 0) + 1 so the first consent is version 1.
    tx.execute(
        "UPDATE preferences SET primary_uuid=?1, parallel_uuids=?2, fallback_uuid=?3, \
         parallel_consent_version=COALESCE(parallel_consent_version, 0) + 1, \
         parallel_consent_scope=?4 WHERE id=1",
        rusqlite::params![primary_val, parallel_json, fallback_val, scope],
    )?;
    let new_version: i64 = tx.query_row(
        "SELECT parallel_consent_version FROM preferences WHERE id=1",
        [],
        |r| r.get(0),
    )?;
    Ok(new_version)
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
    fn from_traditional(e: &dyn engines::TraditionalEngine) -> Self {
        Self {
            id: e.id().into(),
            label: e.label().into(),
            kind: "traditional".into(),
            needs_key: e.needs_key(),
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
            // Windows: owner HWND from the main webview window (the event-loop thread that
            // pumps messages + receives WM_DESTROYCLIPBOARD). `WebviewWindow::hwnd()` is
            // #[cfg(windows)] and returns the windows-crate HWND (newtype HWND(*mut c_void));
            // `.0` is the raw *mut c_void == windows-sys HWND. Non-Windows: pass ().
            // The async block returns (), so resolve the HWND via a `match` (not `?`) and
            // log+return on failure (best-effort: no valid owner → no compound restore).
            #[cfg(target_os = "windows")]
            let owner = match app2
                .get_webview_window("main")
                .ok_or_else(|| "main window unavailable".to_string())
                .and_then(|w| w.hwnd().map(|h| h.0).map_err(|e| e.to_string()))
            {
                Ok(h) => h,
                Err(e) => {
                    log::warn!("clipboard restore skipped: no owner HWND ({e})");
                    return;
                }
            };
            #[cfg(not(target_os = "windows"))]
            let owner = ();
            let cap = selection::capture_selection(800, owner);
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
                // vanish into a popup that's never shown). show_at reveals the window
                // (popup::error alone only emits to a hidden window — review catch).
                let _ = popup::show_at(&app2, x, y);
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
                let _ = popup::show_at(&app2, x, y);
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
    // Round-2 review P1 #2: the REAL registration failure point is the plugin's
    // `setup`, which calls `manager.register(shortcut)?` for every `with_shortcut`
    // and propagates a conflict error to `.run().expect()` → startup crash.
    // Parse-time tolerance (round-1) was insufficient. Fix: register the plugin
    // with NO shortcuts (Builder builds a plugin, but registers nothing at setup),
    // then in the app `setup()` call the runtime `on_shortcut` PER shortcut and
    // catch each Result — a conflict logs + skips THAT shortcut only, the app and
    // the other shortcut keep running.
    let shortcut_plugin = GlobalShortcutBuilder::new().build();

    tauri::Builder::default()
        // single-instance MUST be first: defense-in-depth on top of the real
        // per-dir fs2 flock in keystore.rs (the flock is what serializes a second
        // instance/external writer on the same dir; single-instance just avoids
        // spawning a second process). This plugin focuses the existing instance
        // instead of launching a second.
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
            // Resolve the app-local data dir. Review P1 #2: this MUST NOT crash
            // setup — if the platform path is unavailable, fall back to a temp
            // dir so the app still launches (keys simply won't persist; the
            // recovery banner surfaces the problem). `dir` feeds both the
            // keystore and the DB, so a fallback here keeps every downstream
            // `.expect()`-free path alive.
            let dir = app.path().app_local_data_dir().unwrap_or_else(|e| {
                log::error!("app_local_data_dir unavailable, falling back to temp dir: {e}");
                std::env::temp_dir().join("linguaray-data")
            });
            // Review P1 #2: keystore init must NOT crash either. On failure,
            // build the Session against a temp-dir keystore (translate will find
            // no keys but won't panic) and record the failure so the DB
            // readiness block below degrades to NeedsKeystoreRecovery.
            let (keystore, keystore_init_error) = match keystore::Keystore::new(dir.clone()) {
                Ok(ks) => (ks, None),
                Err(e) => {
                    log::error!(
                        "keystore init in {} failed: {e}; falling back to temp dir",
                        dir.display()
                    );
                    let fallback_dir = std::env::temp_dir().join("linguaray-keystore");
                    let ks = keystore::Keystore::new(fallback_dir).unwrap_or_else(|e2| {
                        log::error!("temp keystore fallback also failed: {e2}");
                        // Last resort: an empty in-memory-shaped dir that won't
                        // be written until a key is set. Keystore::new on a fresh
                        // temp subdir must succeed; this branch is effectively
                        // unreachable but keeps setup panic-free.
                        let last = std::env::temp_dir().join("linguaray-keystore-lastresort");
                        keystore::Keystore::new(last)
                            .expect("temp keystore last-resort must be creatable")
                    });
                    (ks, Some(format!("keystore init in {}: {e}", dir.display())))
                }
            };
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

            // ── S2a data-readiness startup (DB open → migrate → resume → gate) ──
            //
            // NO `.expect()` on DB/migration — the app always launches. Every
            // failure mode degrades `DataReadiness`; provider commands then fail
            // closed via `require_ready`, while the always-available commands
            // (keystore_health / archive_keystore / reset_keystore /
            // get_data_readiness) keep working so the user can recover.
            let db_path = dir.join("linguaray.db");
            let keystore_dir = dir.clone();
            // Resolve the canonical settings path via the store plugin. On
            // failure we MUST NOT guess a fallback path (a wrong-dir guess would
            // read/write the wrong settings file — on Windows the store plugin
            // targets AppData (Roaming) while `dir` here is AppLocalData (Local),
            // so `dir.join("settings.json")` is a different file). Instead record
            // the failure by storing `settings_path = None` and degrade to
            // MigrationIncomplete below; migration is skipped entirely (it needs
            // the real settings path). `archive_database` also treats `None` as a
            // hard stop: it refuses to re-run migration so the user must retry
            // from a state where the path resolves.
            let (settings_path, settings_resolution_error) =
                match tauri_plugin_store::resolve_store_path(app.handle(), "settings.json") {
                    Ok(p) => (Some(p), None),
                    Err(e) => {
                        let reason = format!("settings path resolution failed: {e}");
                        log::error!("{reason}");
                        // None: a non-existent sentinel that the readiness gate
                        // keeps from ever being read. The startup block below
                        // degrades to MigrationIncomplete.
                        (None, Some(reason))
                    }
                };

            // 1. Open the DB. Err → db=None, NeedsDatabaseRecovery (app keeps running).
            let (db_handle, mut readiness) = match Database::open(&db_path) {
                Ok(db) => (Some(Arc::new(db)), DataReadiness::default()),
                Err(e) => (
                    None,
                    DataReadiness::NeedsDatabaseRecovery {
                        reason: format!("open linguaray.db: {e}"),
                    },
                ),
            };

            // Settings path resolution failure takes precedence over migration:
            // we don't know where settings.json lives, so we can't safely migrate
            // (the migration reads + backs up the legacy settings). Degrade to
            // MigrationIncomplete so the recovery banner surfaces it; a retry
            // (next startup) re-resolves.
            if let Some(reason) = &settings_resolution_error {
                readiness = DataReadiness::migration_incomplete("settings_path", reason.clone());
            }

            // Review P1 #2: if the keystore couldn't be initialized in the
            // canonical dir (we're now running on a temp fallback), the app is
            // in keystore-recovery territory regardless of DB state — provider
            // commands that touch the keystore must stay gated off and the
            // recovery banner must show. This takes precedence: a healthy DB +
            // healthy migration are useless without a usable keystore.
            if let Some(reason) = &keystore_init_error {
                readiness = DataReadiness::NeedsKeystoreRecovery {
                    reason: reason.clone(),
                };
            }

            // 2-4. Only run migration + resume + preflight when the DB opened
            // AND the keystore initialized in its canonical dir AND the settings
            // path resolved (otherwise we skip migration to avoid touching a
            // keystore dir we can't lock or reading a guessed settings path).
            if keystore_init_error.is_none() && settings_resolution_error.is_none() {
                if let Some(db) = db_handle.clone() {
                    let fp = FailpointCell::none();
                    // settings_resolution_error.is_none() ⇒ settings_path is Some.
                    // Unwrap once here (the only consumer of the resolved path
                    // outside archive_database) and pass a &PathBuf into run_migration.
                    let settings_path_ref = settings_path
                        .as_ref()
                        .expect("settings path is Some when resolution succeeded");
                    readiness = match run_migration(&db, &keystore_dir, settings_path_ref, &fp) {
                        Ok(()) => {
                            // Resume any in-flight deletes (3-step sweep). A failure
                            // here does NOT exit setup — log + mark incomplete so the
                            // next startup retries.
                            match crate::db::delete::provider_resume_deletions(&db, &keystore_dir) {
                                Ok(_) => {
                                    // Final keystore preflight: a Corrupt keystore
                                    // (detected after migration) → recovery.
                                    match keystore::load_state(&keystore_dir) {
                                        keystore::KeystoreLoadState::Corrupt(e) => {
                                            DataReadiness::NeedsKeystoreRecovery {
                                                reason: format!("keystore corrupt: {e}"),
                                            }
                                        }
                                        _ => DataReadiness::Ready,
                                    }
                                }
                                Err(e) => {
                                    log::error!("resume_deletions failed: {e}");
                                    DataReadiness::migration_incomplete(
                                        "resume_deletions",
                                        format!("resume deletions: {e}"),
                                    )
                                }
                            }
                        }
                        Err(MigrationError::NeedsKeystoreRecovery(reason)) => {
                            DataReadiness::NeedsKeystoreRecovery { reason }
                        }
                        Err(MigrationError::SettingsCorrupt(reason)) => {
                            DataReadiness::migration_incomplete("settings", reason)
                        }
                        Err(other) => DataReadiness::migration_incomplete(
                            "migration",
                            other.to_string(),
                        ),
                    };
                }
            }

            app.manage(Arc::new(AppState {
                db: parking_lot::RwLock::new(db_handle),
                data_gate: parking_lot::RwLock::new(()),
                readiness: parking_lot::RwLock::new(readiness),
                db_path,
                keystore_dir,
                settings_path,
            }));
            // Round-2 review P1 #2: register hotkeys at RUNTIME (per-shortcut,
            // catching each Result) so a conflict skips just that shortcut, not the
            // whole app. on_shortcut registers + attaches the handler in one call.
            let handle: tauri::AppHandle = app.handle().clone();
            let gs = handle.global_shortcut();
            let handle_for_alt = handle.clone();
            if let Err(e) = gs.on_shortcut("Alt+Space", move |_a, _s, ev| {
                on_hotkey(&handle_for_alt, _s, ev);
            }) {
                log::warn!("Alt+Space registration failed (conflict?): {e} — selection hotkey disabled");
            }
            let handle_for_ctrl = handle.clone();
            if let Err(e) = gs.on_shortcut("Ctrl+Space", move |_a, _s, ev| {
                on_input_hotkey(&handle_for_ctrl, _s, ev);
            }) {
                log::warn!("Ctrl+Space registration failed (conflict?): {e} — input hotkey disabled");
            }
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
            a11y_status,
            keystore_health,
            archive_keystore,
            reset_keystore,
            // S2a data-readiness + provider CRUD.
            get_data_readiness,
            provider_list,
            provider_create,
            provider_update,
            provider_duplicate,
            provider_delete,
            provider_reorder,
            provider_toggle,
            provider_set_key,
            provider_set_active,
            // P1 #3: multi-engine consent.
            provider_confirm_and_set_active,
            // P1 #8: provider diagnostics + DB recovery.
            provider_get_models,
            provider_test_connection,
            archive_database
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
