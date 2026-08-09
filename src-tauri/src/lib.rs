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
pub mod adapter;
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
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tauri::menu::MenuEvent;
use tauri::Emitter;
use tauri::Manager;
use tauri_plugin_global_shortcut::{Builder as GlobalShortcutBuilder, GlobalShortcutExt, ShortcutState};

use crate::db::migration::{run_migration, FailpointCell, MigrationError};
use crate::adapter::profile_to_preset;
use crate::db::providers::{self as db_providers, ActiveSelection, ProviderPatch, ProviderProfile, ProviderStatus};
use crate::db::readiness::DataReadiness;
use crate::db::Database;
use crate::service::{translate_parallel, translate_with_fallback_ref, TranslationOutcome};

// Re-export so integration tests can reference the error enum as
// `linguaray_lib::Error` (mirrors `service::TranslationOutcome` usage).
pub use crate::error::Error;

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
    client: Option<reqwest::Client>,
    keystore: Option<keystore::Keystore>,
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

/// Startup readiness reducer (S2a P1.4).
///
/// Computes the pre-migration [`DataReadiness`] from the three independent
/// startup outcomes — DB open, settings-path resolution, keystore init — with a
/// load-bearing **priority** rule: a failed DB open (`NeedsDatabaseRecovery`)
/// is locked in and MUST NOT be masked by a later settings or keystore error.
/// The DB is the foundation; settings/keystore failures only become the visible
/// banner when the DB itself opened successfully.
///
/// Priority order (highest first):
/// 1. **DB failure** → `NeedsDatabaseRecovery`. Nothing else can override this:
///    there is no DB to migrate or to gate provider writes on.
/// 2. **Keystore failure** → `NeedsKeystoreRecovery`. A healthy DB + healthy
///    migration are useless without a usable keystore (provider writes need
///    it), so this beats a settings failure.
/// 3. **Settings failure** → `MigrationIncomplete` ("settings_path"). Migration
///    reads + backs up the legacy settings, so an unresolvable path must skip
///    migration entirely.
/// 4. **All healthy** → the [`DataReadiness::default`] pre-migration state
///    (`MigrationIncomplete "startup not complete"`). The reducer intentionally
///    does NOT return `Ready`: running the migration is what promotes to Ready,
///    and the `setup()` migration block unconditionally assigns this result.
///
/// This is a pure function so the priority matrix is unit-testable without
/// spinning up Tauri or a real DB. The `setup()` closure in [`run`] calls this
/// reducer then, when healthy, overlays the migration outcome on top.
pub fn compute_startup_readiness(
    db_open: Result<(), String>,
    settings_error: Option<String>,
    keystore_error: Option<String>,
) -> DataReadiness {
    // 1. DB failure locks the readiness. NEVER override.
    if let Err(reason) = db_open {
        return DataReadiness::NeedsDatabaseRecovery { reason };
    }
    // DB opened. keystore failure beats settings failure (writes need a
    // usable keystore, so a healthy DB+migration is useless without one).
    if let Some(reason) = keystore_error {
        return DataReadiness::NeedsKeystoreRecovery { reason };
    }
    // Settings path didn't resolve: migration can't run safely against a
    // guessed path, so degrade to MigrationIncomplete and skip migration.
    if let Some(reason) = settings_error {
        return DataReadiness::migration_incomplete("settings_path", reason);
    }
    // All healthy: pre-migration default. The migration step in setup() is
    // what flips this to Ready (or a more specific failure state).
    DataReadiness::default()
}

/// Startup migration gate (round-3 P1.3): decide whether migration may run,
/// and return the REAL settings path it must run against.
///
/// Migration's Phase 1 parses + backs up the legacy settings file, so it needs
/// the actual resolved path. A `None` settings path (resolution failed) must
/// REFUSE migration — running it against a guessed path would read/write the
/// WRONG settings file (on Windows the store plugin targets AppData Roaming
/// while `dir` here is AppLocalData Local). A failed keystore init is also a
/// hard stop (migration's Phase 1 keystore backup needs a usable keystore).
///
/// Returns `Ok(path)` ONLY when the caller may run `run_migration` against
/// `path`; `Err(reason)` otherwise. The refusal decision is what the tests pin
/// down: when this returns `Err`, `run_migration` is never reached, so NO
/// backup is produced and NO DB write occurs (the readiness reducer above
/// already reflects the refusal in `DataReadiness`).
pub fn startup_migration_guard<'a>(
    settings_path: Option<&'a Path>,
    keystore_init_error: Option<&str>,
) -> Result<&'a Path, String> {
    if let Some(e) = keystore_init_error {
        return Err(format!("keystore init failed: {e}"));
    }
    settings_path.ok_or_else(|| {
        "settings path could not be resolved; migration refused (no backup, no DB write)".to_string()
    })
}

/// Resolve the optional `client` from the [`Session`] or return a clear error
/// string. Used by the translate commands so a startup build failure surfaces
/// consistently instead of panicking.
fn session_client(session: &Session) -> Result<&reqwest::Client, String> {
    session.client.as_ref().ok_or_else(|| {
        "HTTP client unavailable: startup build failed (recovery required)".to_string()
    })
}

/// Resolve the optional `keystore` from the [`Session`] or return a clear error
/// string. Used by the translate / key commands so a startup init failure
/// (degraded `NeedsKeystoreRecovery`) surfaces consistently instead of
/// panicking.
fn session_keystore(session: &Session) -> Result<&keystore::Keystore, String> {
    session.keystore.as_ref().ok_or_else(|| {
        "keystore unavailable: startup init failed (recovery required)".to_string()
    })
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
    let client = session_client(&state)?;
    let keystore = session_keystore(&state)?;
    let t = service::translate_with_fallback(client, keystore, &preset, input, fallback)
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
    let client = session_client(&state)?;
    let keystore = session_keystore(&state)?;
    let t = service::translate_with_fallback(client, keystore, &preset, input, fallback)
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
    app_state: tauri::State<'_, Arc<AppState>>,
) -> Result<(), String> {
    // latest-wins：先分配 gen（同步，保证 press 顺序）。
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

    let client = match session_client(&state) {
        Ok(c) => c.clone(),
        Err(msg) => {
            if state.gen.is_latest(gen) {
                let _ = popup::error(&app, &msg);
            }
            return Ok(());
        }
    };
    let keystore = match session_keystore(&state) {
        Ok(k) => k,
        Err(msg) => {
            if state.gen.is_latest(gen) {
                let _ = popup::error(&app, &msg);
            }
            return Ok(());
        }
    };

    // gate + require_ready_gated 拿 db（spawn_blocking 内，与 translate_session 一致）。
    let app_arc = app_state.inner().clone();
    let db = match tauri::async_runtime::spawn_blocking(move || -> Result<Arc<Database>, String> {
        let _gate = app_arc.data_gate.read();
        require_ready_gated(&app_arc, &_gate)
    })
    .await
    {
        Ok(Ok(db)) => db,
        Ok(Err(msg)) => {
            if state.gen.is_latest(gen) {
                let _ = popup::error(&app, &msg);
            }
            return Ok(());
        }
        Err(e) => {
            if state.gen.is_latest(gen) {
                let _ = popup::error(&app, &format!("join error: {e}"));
            }
            return Ok(());
        }
    };

    // 走统一核心（从 settings 读 fallback_engine；target_language 来自 settings）。
    let session_result = run_translate_session(
        &db, &client, keystore, &app, &text, "auto", &s.target_language,
    )
    .await;

    // latest-wins：完成后检查 gen 才发事件。
    if !state.gen.is_latest(gen) {
        return Ok(());
    }
    match session_result {
        Ok(r) => match decide_clipboard_popup(&r) {
            ClipboardPopupDecision::SingleSuccess { text, engine } => {
                let _ = popup::result(&app, &text, &engine);
            }
            ClipboardPopupDecision::Multi => {
                let _ = popup::multi_result(&app, &r.outcomes);
            }
            ClipboardPopupDecision::Error(msg) => {
                let _ = popup::error(&app, &msg);
            }
        },
        Err(msg) => {
            let _ = popup::error(&app, &msg);
        }
    }
    Ok(())
}

// ─── R2a: translate_session ──────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize)]
pub struct TranslateSessionRequest {
    pub text: String,
    pub from: String,
    pub to: String,
}

#[derive(Debug, Serialize)]
pub struct TranslateSessionResult {
    /// 每个引擎的结果（成功或分类过的错误）。单引擎路径长度=1，并行=primary+parallel 数。
    pub outcomes: Vec<TranslationOutcome>,
    /// 单引擎成功时的实际 engine id（preset.id=secret_ref）；并行或全失败时 None。
    /// 老前端可只读这个字段保持单结果 UI 工作；新前端读 outcomes 渲染多结果。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub actual_engine: Option<String>,
}

/// translate_clipboard 根据翻译结果决定发哪种 popup 事件的纯函数决策。
/// 抽出来便于测试（translate_clipboard 本身依赖 Tauri runtime 不可单测）。
#[derive(Debug)]
pub enum ClipboardPopupDecision {
    /// 单引擎成功 → 走老 popup-state 事件（向后兼容）。
    SingleSuccess { text: String, engine: String },
    /// 并行（含部分成功）→ 走 popup-multi-result 事件。
    Multi,
    /// 单引擎失败 / 并行全失败 / 核心错误 → 走 popup-state error。
    Error(String),
}

pub fn decide_clipboard_popup(result: &TranslateSessionResult) -> ClipboardPopupDecision {
    if result.outcomes.is_empty() {
        return ClipboardPopupDecision::Error("translation produced no outcomes".into());
    }
    // 单引擎路径：actual_engine=Some 表示成功。
    if let Some(engine) = &result.actual_engine {
        // 长度必为 1（run_translate_session 单引擎路径契约）。
        if let Some(o) = result.outcomes.first() {
            if let Ok(t) = &o.result {
                return ClipboardPopupDecision::SingleSuccess {
                    text: t.text.clone(),
                    engine: engine.clone(),
                };
            }
        }
        // actual_engine=Some 但 outcome 失败（理论不应发生）→ 当错误处理。
        return ClipboardPopupDecision::Error("single engine failed unexpectedly".into());
    }
    // actual_engine=None：并行路径。
    if result.outcomes.len() == 1 {
        // 退化单引擎但失败。
        if let Some(err) = result.outcomes.first().and_then(|o| o.result.as_ref().err()) {
            return ClipboardPopupDecision::Error(err.to_string());
        }
    }
    // 并行全失败？
    let all_failed = result.outcomes.iter().all(|o| o.result.is_err());
    if all_failed {
        return ClipboardPopupDecision::Error("all engines failed".into());
    }
    // 并行（含部分成功）。
    ClipboardPopupDecision::Multi
}

/// Central sentinel resolver: the frontend passes `to: ""` to mean "use the
/// stored target language". Exposed so the hotkey contract is locked by an
/// integration test.
pub fn resolve_target_language(to: &str, settings_target: &str) -> String {
    if to.is_empty() {
        settings_target.to_string()
    } else {
        to.to_string()
    }
}

/// 翻译会话核心逻辑（纯函数，无 Tauri State 依赖）。
///
/// 被两个入口共享：
/// - `translate_session` 命令（带 AppHandle，从 settings 读 fallback_engine）
/// - `translate_clipboard`（同上）
/// - 测试用 `run_translate_session_no_settings`（fallback=None，避开 settings）
///
/// 流程见 plan Task 4：read_active_selection → list → 过滤 active+enabled →
/// 单引擎 or translate_parallel。
async fn run_translate_session(
    db: &Arc<Database>,
    client: &reqwest::Client,
    keystore: &keystore::Keystore,
    app: &tauri::AppHandle,
    text: &str,
    from: &str,
    to: &str,
) -> Result<TranslateSessionResult, String> {
    // P1-C: resolve the "" sentinel CENTRALLY so on_hotkey, translate_session,
    // translate_selection_ipc, and the tray all agree.
    let settings_target = settings::load(app).target_language;
    let to = resolve_target_language(to, &settings_target);
    // 读 fallback_engine（§G opt-in，默认 None）。
    let fallback_box = settings::load(app).fallback_engine.as_deref().and_then(engines::find);
    let fallback: Option<Arc<dyn engines::TraditionalEngine>> =
        fallback_box.map(Arc::<dyn engines::TraditionalEngine>::from);
    run_translate_session_with_fallback(db, client, keystore, text, from, &to, fallback).await
}

/// 测试入口：不读 settings，fallback 直接传 None（聚焦核心路径）。
pub async fn run_translate_session_no_settings(
    db: &Arc<Database>,
    client: &reqwest::Client,
    keystore: &keystore::Keystore,
    text: &str,
    from: &str,
    to: &str,
) -> Result<TranslateSessionResult, String> {
    run_translate_session_with_fallback(db, client, keystore, text, from, to, None).await
}

async fn run_translate_session_with_fallback(
    db: &Arc<Database>,
    client: &reqwest::Client,
    keystore: &keystore::Keystore,
    text: &str,
    from: &str,
    to: &str,
    fallback: Option<Arc<dyn engines::TraditionalEngine>>,
) -> Result<TranslateSessionResult, String> {
    // 读 active selection + 全量 list（一个 blocking 块内，gate 保护）。
    // 注意：调用方（命令）已持有 data_gate + require_ready_gated；这里为了
    // 让纯函数可测，直接用 db（测试里 db 是健康的）。命令路径会在外层先 gate。
    let (selection, all_profiles): (ActiveSelection, Vec<ProviderProfile>) = {
        let sel = db
            .with_conn(|conn| db_providers::read_active_selection(conn))
            .map_err(|e| e.to_string())?;
        let list = db
            .with_conn(|conn| db_providers::list(conn))
            .map_err(|e| e.to_string())?;
        (sel, list)
    };

    // 过滤出 active+enabled 的 profile，按 selection 顺序（primary 先，parallel 次）。
    // 与 validate_active_selection 的 active+enabled 判定一致。
    let is_callable = |p: &ProviderProfile| {
        p.status == ProviderStatus::Active.as_str() && p.enabled
    };
    let mut profiles: Vec<ProviderProfile> = Vec::new();
    if let Some(primary_uuid) = &selection.primary {
        if let Some(p) = all_profiles.iter().find(|p| &p.uuid == primary_uuid) {
            if is_callable(p) {
                profiles.push(p.clone());
            }
        }
    }
    for uuid in &selection.parallel {
        if let Some(p) = all_profiles.iter().find(|p| &p.uuid == uuid) {
            if is_callable(p) && !profiles.iter().any(|q| q.uuid == p.uuid) {
                profiles.push(p.clone());
            }
        }
    }
    if profiles.is_empty() {
        return Err("no active provider selected".into());
    }

    // 单引擎 vs 并行。
    if selection.parallel.is_empty() {
        // 单引擎：用 primary profile + translate_with_fallback_ref。
        let preset = profile_to_preset(&profiles[0])
            .map_err(|e| format!("adapter error: {e}"))?;
        let input = service::TranslateInput {
            text,
            from,
            to,
            options: wire::AppOptions::default(),
        };
        let fb_ref: Option<&dyn engines::TraditionalEngine> = fallback.as_deref();
        let result = translate_with_fallback_ref(
            client, keystore, &preset, input, fb_ref,
        )
        .await;
        let actual_engine = match &result {
            Ok(t) => Some(t.engine.clone()),
            Err(_) => None,
        };
        Ok(TranslateSessionResult {
            outcomes: vec![TranslationOutcome {
                uuid: profiles[0].uuid.clone(),
                result,
            }],
            actual_engine,
        })
    } else {
        // 并行。
        let outcomes = translate_parallel(
            client,
            keystore,
            profiles,
            text,
            from,
            to,
            wire::AppOptions::default(),
            fallback,
        )
        .await;
        Ok(TranslateSessionResult {
            outcomes,
            actual_engine: None,
        })
    }
}

/// 并行/单引擎翻译命令（R2a）。前端用 `invoke('translate_session', { req })` 调用。
///
/// 从 AppState 读 active selection + providers，从 Session 读 client/keystore，
/// 从 settings 读 fallback_engine。parallel 为空时退化为单引擎（actual_engine=Some）。
#[tauri::command]
async fn translate_session(
    app: tauri::AppHandle,
    state: tauri::State<'_, Arc<Session>>,
    app_state: tauri::State<'_, Arc<AppState>>,
    req: TranslateSessionRequest,
) -> Result<TranslateSessionResult, String> {
    let client = session_client(&state)?.clone();
    let keystore = session_keystore(&state)?;
    let app_arc = app_state.inner().clone();
    // 在 blocking 内 gate + 读 DB 快照（gate 必须在 clone Arc 之前，见 provider_list 注释）。
    // 这里我们让 run_translate_session 自己用 db.with_conn 读；但 gate 要由命令持有。
    // 所以：spawn_blocking 里 gate + require_ready_gated 拿到 db Arc，直接交给核心。
    let db = tauri::async_runtime::spawn_blocking(move || -> Result<Arc<Database>, String> {
        let _gate = app_arc.data_gate.read();
        let db = require_ready_gated(&app_arc, &_gate)?;
        Ok(db)
    })
    .await
    .map_err(|e| e.to_string())??;
    run_translate_session(&db, &client, keystore, &app, &req.text, &req.from, &req.to).await
}

/// A4 (P1-5): translate the live OS selection (fresh capture) OR a
/// caller-supplied SOURCE text (Retry). Distinct from `translate_clipboard`
/// which reads the clipboard — this NEVER reads the clipboard. The tray
/// `translate-selection` action and the popup Retry both route here.
///
/// `text = Some(t)` (Retry): skip capture, use the saved SOURCE text.
/// `text = None` (tray): capture the selection fresh under the selection lock.
///
/// Returns `Result<(), ()>` because the popup state is emitted via events —
/// there is no useful payload to return to the caller. Allocation of the
/// generation token + the capture+translate pipeline run on the async runtime
/// so the IPC call does not block on capture_selection (which simulates Cmd+C).
#[tauri::command]
async fn translate_selection_ipc(
    app: tauri::AppHandle,
    state: tauri::State<'_, Arc<Session>>,
    app_state: tauri::State<'_, Arc<AppState>>,
    text: Option<String>,
) -> Result<(), ()> {
    let state = state.inner().clone();
    let app_state = app_state.inner().clone();
    let gen = state.gen.next();
    tauri::async_runtime::spawn(async move {
        // The cursor read + capture_selection happen together under ONE lock
        // inside capture_and_translate (None coords → helper reads cursor under
        // the lock). For the Retry path (supplied text), the popup is re-shown
        // at the current cursor, so None coords are correct there too.
        capture_and_translate(&app, &state, &app_state, text, None, None, gen).await;
    });
    Ok(())
}

/// A4 (P1-5): show the main (settings) window and emit a `navigate` event so
/// the App mount sets the shell's active page. The popup/input CTAs and the
/// tray `settings` action both route here so the main window surfaces + jumps
/// to the right section in one call. `section = None` defaults to the provider
/// center.
#[tauri::command]
async fn open_settings_window(
    app: tauri::AppHandle,
    section: Option<String>,
) -> Result<(), String> {
    let page = section.unwrap_or_else(|| "provider-center".to_string());
    let app2 = app.clone();
    tauri::async_runtime::spawn(async move {
        if let Some(w) = app2.get_webview_window("main") {
            let _ = w.show();
            let _ = w.set_focus();
            let _ = w.emit("navigate", page);
        }
    })
    .await
    .map_err(|e| format!("join error: {e}"))?;
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
    let keystore = session_keystore(&state)?;
    keystore.set_key(&provider_id, &key).map_err(|e| e.to_string())?;
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
    let keystore = session_keystore(&state)?;
    keystore.delete_key(&provider_id).map_err(|e| e.to_string())?;
    Ok(())
}

/// P1 orphan-key guard shared by the legacy `set_key` / `delete_key` commands.
///
/// Asserts that `provider_id` equals the `secret_ref` of some non-deleted
/// provider row in the DB. Returns `Err` (refusing the keystore write) when:
/// - the DB isn't `Ready` (S2a P1: refuse key writes during
///   `MigrationIncomplete` / `NeedsDatabaseRecovery` / `NeedsKeystoreRecovery` —
///   the row set may be mid-migration or absent, so we can't validate ownership
///   safely and a write could create an orphan the migration's Phase-5
///   verification would later reject),
/// - the DB handle is unavailable (can't validate — refuse rather than risk an
///   orphan), or
/// - no non-deleted row owns that `secret_ref` (the write would create / touch
///   an orphan key the migration's Phase-5 verification would later reject).
///
/// MUST be called while holding `data_gate.read()` (the legacy commands acquire
/// it before calling) so the row set can't change under us.
fn assert_secret_ref_owned(app: &Arc<AppState>, provider_id: &str) -> Result<(), String> {
    // Readiness gate FIRST: refuse key writes unless the DB is Ready. A
    // MigrationIncomplete / Needs*Recovery state means the row set is in flux
    // (or absent), so the COUNT below could race the migration or read a
    // half-built schema. The keystore is independent, but writing a key whose
    // owner we can't verify would create an orphan.
    let readiness = app.readiness.read();
    if !readiness.is_ready() {
        return Err(format!(
            "cannot set/delete key: database not ready ({:?})",
            *readiness
        ));
    }
    drop(readiness);
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
    let refs = match state.keystore.as_ref() {
        Some(ks) => match ks.list_provider_key_refs() {
            Ok(r) => r,
            Err(_) => return std::collections::HashMap::new(),
        },
        // No keystore (startup init failure): return empty so onMount doesn't
        // abort. The recovery banner reads `keystore_health` for the reason.
        None => return std::collections::HashMap::new(),
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
    // When the keystore couldn't be initialized at startup (Session.keystore is
    // None), surface that reason so the recovery banner shows it.
    match state.keystore.as_ref() {
        Some(ks) => match ks.load() {
            Ok(_) => String::new(),
            Err(e) => format!("{e}"),
        },
        None => "keystore unavailable: startup init failed".to_string(),
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

/// Build the translate HTTP client (S2a Task 5b).
///
/// Spec §Privacy: no cross-origin redirects, a 30s total request timeout so a
/// hung connection can't freeze translate + the popup, and a 10s connect
/// timeout so `wire::call` can classify a real `Timeout`. This MUST NOT panic
/// and MUST NOT silently downgrade the privacy policy: the hardened builder
/// (with `redirect(Policy::none())`) is the ONLY client we ever return. On a
/// builder failure (pathological TLS-init environment) we return `Err` so the
/// caller can log it and enter a degraded startup state — a default client
/// would drop `redirect(Policy::none())`, re-opening the cross-origin-redirect
/// leak the policy exists to close.
fn build_http_client() -> Result<reqwest::Client, String> {
    reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .timeout(std::time::Duration::from_secs(30))
        .connect_timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| format!("hardened HTTP client build failed: {e}"))
}

/// Init a last-resort keystore in a PID-uniquified temp subdir (S2a Task 5b).
///
/// Called only when both the canonical-dir keystore AND the shared temp-dir
/// fallback failed to init. A PID-suffixed subdir sidesteps any flock/perm
/// state the earlier attempts left behind, and a second concurrent instance
/// gets its own dir. This MUST NOT panic: it keeps trying uniquified dirs and,
/// if all of them fail (genuinely unreachable — `Keystore::new` only does
/// `create_dir_all` + `set_dir_perms`), returns `Err` so the caller can record
/// the failure and degrade to `NeedsKeystoreRecovery`. The OS temp allocator is
/// effectively always writable, so the `Err` path is defence-in-depth.
fn init_last_resort_keystore() -> Result<keystore::Keystore, String> {
    let pid = std::process::id();
    // Try up to 17 PID-suffixed temp subdirs. Each attempt logs on failure; the
    // loop returns Ok on the first success. If all 17 fail (genuinely
    // unreachable — the OS temp allocator is effectively always writable), make
    // one final attempt with a random name and surface the error.
    let mut last_err = String::new();
    for suffix in 0..=16 {
        let candidate = std::env::temp_dir()
            .join(format!("linguaray-keystore-lastresort-{pid}-{suffix}"));
        match keystore::Keystore::new(candidate) {
            Ok(ks) => return Ok(ks),
            Err(e) => {
                log::warn!("last-resort keystore dir {suffix} failed: {e}");
                last_err = e.to_string();
            }
        }
    }
    // Defence-in-depth: all PID-suffixed dirs failed. Construct one final
    // attempt with a random-ish name, then surface the error (no panic).
    let nano = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let final_dir = std::env::temp_dir().join(format!("linguaray-keystore-{pid}-{nano}"));
    log::error!(
        "all last-resort keystore dirs failed; final attempt: {}",
        final_dir.display()
    );
    match keystore::Keystore::new(final_dir) {
        Ok(ks) => Ok(ks),
        Err(e) => Err(format!(
            "cannot init any keystore (last error: {e}; prior loop: {last_err})"
        )),
    }
}

/// Validate every shipped preset endpoint (S2a Task 5b, spec §Privacy).
///
/// Thin wrapper over [`validate_preset_endpoints`] reading the hardcoded
/// catalog. Kept separate so the core validation is injectable in tests (a test
/// catalog containing a deliberately-invalid endpoint can prove the fail-closed
/// chain end-to-end instead of only exercising `validate_endpoint` in
/// isolation).
fn validate_all_preset_endpoints() -> Vec<String> {
    validate_preset_endpoints(&providers::presets())
}

/// Validate a PRESET LIST's endpoints (spec §Privacy): every endpoint must be
/// HTTPS (loopback HTTP allowed for local engines like Ollama). Returns the ids
/// whose endpoints FAILED scheme validation (empty = all valid).
///
/// Each failure is logged + the offending preset is effectively disabled (its
/// engine won't be usable until the catalog is fixed), but a single bad preset
/// does not crash startup — this is a far better failure mode than refusing to
/// launch. The caller decides the fail-closed response via
/// [`preset_gate_allows_client`].
fn validate_preset_endpoints(list: &[providers::ProviderPreset]) -> Vec<String> {
    let mut invalid = Vec::new();
    for p in list {
        if let Err(e) = providers::validate_endpoint(&p.endpoint) {
            log::error!(
                "preset '{}' endpoint '{}' failed scheme validation ({e}); engine disabled",
                p.id,
                p.endpoint
            );
            invalid.push(p.id.clone());
        }
    }
    invalid
}

/// Fail-closed gate for the HTTP client (round-3 P1.1).
///
/// Given the preset ids whose endpoints failed validation, decide whether ANY
/// outbound request may be shipped. A single invalid preset disables the client
/// ENTIRELY (the setup path builds `None` and every translate entry-point then
/// refuses via `session_client`) — a leaked/broken catalog endpoint must never
/// fire a request just because the user happens to select a different engine.
///
/// This is the deterministic, testable decision behind the setup inline branch:
/// `Ok(c) if preset_gate_allows_client(&invalid) => Some(c)`, else `None`.
fn preset_gate_allows_client(invalid_presets: &[String]) -> bool {
    invalid_presets.is_empty()
}

/// Returns the live [`DataReadiness`] so the frontend can drive the recovery
/// banner. Always available (no readiness gate) — it's how the UI discovers the
/// gate is closed in the first place.
///
/// Returns the typed `DataReadiness` directly (Tauri auto-serializes it via the
/// `#[derive(Serialize)]` + `#[serde(tag="state", rename_all="snake_case")]` on
/// the enum).
///
/// WIRE CONTRACT: this is a breaking change from the pre-S2a `String` return —
/// the old command returned a JSON-ENCODED STRING (the frontend had to parse
/// the string's contents as JSON). It now ships a real JSON OBJECT:
/// - `Ready` → `{"state":"ready"}`
/// - `NeedsKeystoreRecovery` → `{"state":"needs_keystore_recovery","reason":"…"}`
/// - `NeedsDatabaseRecovery` → `{"state":"needs_database_recovery","reason":"…"}`
/// - `MigrationIncomplete` → `{"state":"migration_incomplete","checkpoint":…,"reason":"…"}`
/// (internally-tagged enum: the variant determines which fields exist).
/// Frontend callers must read a JSON object, not a string.
#[tauri::command]
fn get_data_readiness(state: tauri::State<'_, Arc<AppState>>) -> DataReadiness {
    state.readiness.read().clone()
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
    app_handle: tauri::AppHandle,
    template_id: String,
    name: String,
    endpoint: String,
    model: Option<String>,
) -> Result<ProviderProfile, String> {
    let app_state = state.inner().clone();
    let result = tauri::async_runtime::spawn_blocking(move || {
        // Acquire the gate FIRST (see provider_list).
        let _gate = app_state.data_gate.read();
        let db = require_ready_gated(&app_state, &_gate)?;
        db.with_conn(|conn| {
            db_providers::create(conn, &template_id, &name, &endpoint, model.as_deref())
        })
        .map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())??;
    // rev-8-8: refresh the tray AFTER the write commits. Best-effort.
    refresh_tray_if_available(&app_handle);
    Ok(result)
}

/// Apply a partial patch to a provider. An endpoint change is validated and may
/// invalidate the parallel consent (see `db_providers::update`).
#[tauri::command]
async fn provider_update(
    state: tauri::State<'_, Arc<AppState>>,
    app_handle: tauri::AppHandle,
    uuid: String,
    patch: ProviderPatch,
) -> Result<ProviderProfile, String> {
    let app_state = state.inner().clone();
    let result = tauri::async_runtime::spawn_blocking(move || {
        // Acquire the gate FIRST (see provider_list).
        let _gate = app_state.data_gate.read();
        let db = require_ready_gated(&app_state, &_gate)?;
        db.with_conn(|conn| db_providers::update(conn, &uuid, &patch)).map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())??;
    refresh_tray_if_available(&app_handle);
    Ok(result)
}

/// Duplicate a provider. New UUID, new `secret_ref`, keyless (the original key
/// is never copied).
#[tauri::command]
async fn provider_duplicate(
    state: tauri::State<'_, Arc<AppState>>,
    app_handle: tauri::AppHandle,
    uuid: String,
) -> Result<ProviderProfile, String> {
    let app_state = state.inner().clone();
    let result = tauri::async_runtime::spawn_blocking(move || {
        // Acquire the gate FIRST (see provider_list).
        let _gate = app_state.data_gate.read();
        let db = require_ready_gated(&app_state, &_gate)?;
        db.with_conn(|conn| db_providers::duplicate(conn, &uuid)).map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())??;
    refresh_tray_if_available(&app_handle);
    Ok(result)
}

/// Begin the 3-step delete (mark `deleting`, evict from slots), purge the key
/// from the keystore, then finalize the tombstone. Each step is committed before
/// the next; the lock-order rule (DB Mutex and keystore flock never nested) is
/// preserved by releasing the DB guard between steps. All three steps run on one
/// blocking thread so the `data_gate` write guard spans the whole operation.
#[tauri::command]
async fn provider_delete(
    state: tauri::State<'_, Arc<AppState>>,
    app_handle: tauri::AppHandle,
    uuid: String,
) -> Result<(), String> {
    let app_state = state.inner().clone();
    let keystore_dir = app_state.keystore_dir.clone();
    tauri::async_runtime::spawn_blocking(move || -> Result<(), String> {
        // Write guard: a delete mutates selection slots + status; no reader/other
        // writer may interleave. Held for all 3 steps (the DB Mutex + keystore
        // flock are still released between steps inside their own calls).
        // Acquire the gate FIRST (see provider_list) so the readiness check +
        // Arc clone are atomic w.r.t. the DB swap.
        let _gate = app_state.data_gate.write();
        let db = require_ready_gated_write(&app_state, &_gate)?;

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
    .map_err(|e| e.to_string())??;
    refresh_tray_if_available(&app_handle);
    Ok(())
}

/// Re-assign `sort_order` to the given UUID order. The list MUST be exactly the
/// set of active UUIDs.
#[tauri::command]
async fn provider_reorder(
    state: tauri::State<'_, Arc<AppState>>,
    app_handle: tauri::AppHandle,
    uuids: Vec<String>,
) -> Result<(), String> {
    let app_state = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        // Acquire the gate FIRST (see provider_list).
        let _gate = app_state.data_gate.write();
        let db = require_ready_gated_write(&app_state, &_gate)?;
        db.with_conn(|conn| db_providers::reorder(conn, &uuids)).map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())??;
    refresh_tray_if_available(&app_handle);
    Ok(())
}

/// Flip `enabled`. Disabling also evicts the row from selection slots and
/// invalidates parallel consent (mirrors `begin_delete`).
#[tauri::command]
async fn provider_toggle(
    state: tauri::State<'_, Arc<AppState>>,
    app_handle: tauri::AppHandle,
    uuid: String,
    enabled: bool,
) -> Result<(), String> {
    let app_state = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        // Acquire the gate FIRST (see provider_list).
        let _gate = app_state.data_gate.write();
        let db = require_ready_gated_write(&app_state, &_gate)?;
        db.with_conn(|conn| db_providers::toggle(conn, &uuid, enabled)).map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())??;
    refresh_tray_if_available(&app_handle);
    Ok(())
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
    app_handle: tauri::AppHandle,
    primary: String,
    parallel: Vec<String>,
    fallback: Option<String>,
) -> Result<SetActiveResult, String> {
    let app_state = state.inner().clone();
    let outcome = tauri::async_runtime::spawn_blocking(move || -> Result<SetActiveResult, String> {
        // Acquire the gate FIRST (see provider_list).
        let _gate = app_state.data_gate.write();
        let db = require_ready_gated_write(&app_state, &_gate)?;
        // The `with_conn` closure must return Result<_, DbError> (Database's
        // contract). We carry the consent-required signal out via a SetActiveOutcome
        // so the outer closure can map it to the frontend-facing SetActiveResult
        // without smuggling a ConsentError through the DbError boundary.
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
        // P1.1: map the internal outcome to the serialized tagged union. The
        // consent-required path is now an Ok(SetActiveResult::NeedsConsent) so
        // the frontend gets a structured payload, not a parsed error string.
        Ok(match outcome {
            SetActiveOutcome::Written => SetActiveResult::Written,
            SetActiveOutcome::NeedsConsent { actual_scope } => SetActiveResult::NeedsConsent {
                actual_scope,
            },
        })
    })
    .await
    .map_err(|e| e.to_string())??;
    // rev-7-8: refresh so the status item + submenu reflect the new primary.
    refresh_tray_if_available(&app_handle);
    Ok(outcome)
}
/// popup/popup CTAs (Surface 02/03) so they can show a friendly engine label.
/// Returns the default (all-empty) selection if the DB is not ready yet.
#[tauri::command]
fn provider_get_active_selection(
    app_state: tauri::State<'_, Arc<AppState>>,
) -> Result<ActiveSelection, String> {
    let app = app_state.inner().clone();
    let _gate = app.data_gate.read();
    let db = require_ready_gated(&app, &_gate)?;
    db.with_conn(|conn| db_providers::read_active_selection(conn))
        .map_err(|e| e.to_string())
}

/// A4 + rev-5-4: the sync core of `provider_set_active`, callable from the
/// tray (which cannot resolve `tauri::State`). Sets `uuid` as the sole primary,
/// no parallel, no fallback. This is the BODY the tray handler runs inside a
/// `spawn_blocking` — do NOT wrap it in another `block_on(spawn_blocking(...))`.
/// Uses the real write helper `set_active_slots`; because `parallel` is empty,
/// the consent gate is never entered and the NeedsConsent branch is unreachable
/// (kept in the match for exhaustiveness; if it ever fires it maps through).
fn set_active_primary_core(
    app_state: Arc<AppState>,
    uuid: String,
) -> Result<SetActiveResult, String> {
    let app = app_state.clone();
    let outcome = db_set_active_primary(&app, &uuid)?;
    Ok(match outcome {
        SetActiveOutcome::Written => SetActiveResult::Written,
        SetActiveOutcome::NeedsConsent { actual_scope } => {
            SetActiveResult::NeedsConsent { actual_scope }
        }
    })
}

/// rev-5-4: the gate + transaction that `set_active_primary_core` and the tray
/// share. Acquires the write gate, runs `validate_active_selection` + the
/// `set_active_slots` write inside ONE transaction. Returns the internal
/// `SetActiveOutcome` so the caller can map it to the serialized result.
fn db_set_active_primary(
    app: &Arc<AppState>,
    uuid: &str,
) -> Result<SetActiveOutcome, String> {
    let _gate = app.data_gate.write();
    let db = require_ready_gated_write(app, &_gate)?;
    let outcome = db
        .with_conn(|conn| -> Result<SetActiveOutcome, DbErr> {
            let tx = conn.transaction()?;
            let active = db_providers::list(&tx)?;
            db_providers::validate_active_selection(uuid, &[], None, &active)?;
            // parallel is empty → set_active_slots (clears prior consent).
            set_active_slots(&tx, uuid, &[], None)?;
            tx.commit()?;
            Ok(SetActiveOutcome::Written)
        })
        .map_err(|e| e.to_string())?;
    Ok(outcome)
}

/// A4 minimal `handle_switch_provider` (A5 Step 10 will enhance this with the
/// tray-state controller wiring — begin_switch/finish_switch + the red-dot on
/// failure). This A4 version: set the provider as sole primary (SYNC, inside
/// the caller's `spawn_blocking`), refresh the tray, and on failure set a
/// `"Switch failed: <msg>"` tooltip AFTER the refresh (rev-19-5) so the
/// refresh's own tooltip is not clobbered. Returns Ok/Err so the caller can
/// decide logging.
fn handle_switch_provider(
    app: &tauri::AppHandle,
    app_state: &Arc<AppState>,
    uuid: &str,
) -> Result<(), String> {
    match set_active_primary_core(app_state.clone(), uuid.to_string()) {
        Ok(_) => {
            // Success: refresh so the status item + submenu reflect the new primary.
            refresh_tray_if_available(app);
            Ok(())
        }
        Err(msg) => {
            // rev-19-5: refresh FIRST (restores the pre-switch tooltip), THEN
            // override with the failure tooltip (rev-21-2: prefixed).
            refresh_tray_if_available(app);
            if let Some(tray) = app.tray_by_id("main-tray") {
                let _ = tray.set_tooltip(Some(&format!("Switch failed: {msg}")));
            }
            Err(msg)
        }
    }
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
    app_handle: tauri::AppHandle,
    primary: String,
    parallel: Vec<String>,
    fallback: Option<String>,
    expected_scope: String,
) -> Result<i64, ProviderCommandError> {
    let app_state = state.inner().clone();
    let version = tauri::async_runtime::spawn_blocking(move || -> Result<i64, ProviderCommandError> {
        // Acquire the gate FIRST (see provider_list).
        let _gate = app_state.data_gate.write();
        let db = require_ready_gated_write(&app_state, &_gate).map_err(ProviderCommandError::from)?;
        let outcome = db.with_conn(|conn| -> Result<ConfirmActiveOutcome, DbErr> {
            let tx = conn.transaction()?;
            let active = db_providers::list(&tx)?;
            db_providers::validate_active_selection(
                &primary,
                &parallel,
                fallback.as_deref(),
                &active,
            )?;
            let actual_scope = db_providers::compute_scope(&primary, &parallel, &active)
                .map_err(consent_to_db)?;
            if expected_scope != actual_scope {
                // Stale frontend: the scope it asserts doesn't match what the
                // backend recomputes (it raced a provider change). Carried out
                // as a typed variant — no sentinel string to parse.
                return Ok(ConfirmActiveOutcome::StaleScope { actual_scope });
            }
            let new_version = write_consented_selection(
                &tx,
                &primary,
                &parallel,
                fallback.as_deref(),
                &actual_scope,
            )?;
            tx.commit()?;
            Ok(ConfirmActiveOutcome::Written { version: new_version })
        });
        // Map the typed outcome: StaleScope → ProviderCommandError::StaleScope
        // (structured wire error), Written → the consent version. Everything
        // else (real DB errors) stays an error.
        outcome
            .map(|o| match o {
                ConfirmActiveOutcome::Written { version } => Ok(version),
                ConfirmActiveOutcome::StaleScope { actual_scope } => {
                    Err(ProviderCommandError::StaleScope { actual_scope })
                }
            })
            .map_err(ProviderCommandError::from)?
    })
    .await
    .map_err(|e| ProviderCommandError::Db {
        message: format!("{e:?}"),
    })??;
    // rev-8-8: refresh so the status item + submenu reflect the new primary.
    refresh_tray_if_available(&app_handle);
    Ok(version)
}

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
    let client = match session.client.as_ref() {
        Some(c) => c,
        None => {
            return Ok(ConnectionResult {
                ok: false,
                message: "HTTP client unavailable: startup build failed".into(),
            })
        }
    };
    let req = client.get(&profile.endpoint).send().await;
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
/// Thin wrapper around [`crate::db::recovery::archive_database_core`] (the
/// shared close/rename/reopen/migrate pipeline) with the production failpoint
/// ([`crate::db::recovery::ArchiveFailpoint::None`]). The core owns the whole
/// state machine so the production path and the recovery failpoint tests
/// exercise the SAME logic.
///
/// Pipeline (see the core for the full contract):
/// 1. PREFLIGHT `settings_path` BEFORE any destructive op (S2a P1). If the
///    path is `None`, refuse immediately — the DB is untouched and usable. This
///    avoids closing/renaming a working DB only to discover we can't migrate
///    because the settings path couldn't be resolved at startup.
/// 2. Acquire `data_gate.write()` (blocks every provider command).
/// 3. `Arc::try_unwrap` + `Database::close` — release the SQLite file handle.
/// 4. `fs::rename(db_path, broken_path)`.
/// 5. Open a fresh DB + run migration + `resume_deletions`.
/// 6. Install the new handle + `Ready`.
///
/// Any failure AFTER the rename leaves the slot `None` (or, for migration
/// failures, `Some` fresh DB) and a non-`Ready` readiness. Any failure BEFORE
/// the rename leaves the original DB untouched and usable.
#[tauri::command]
async fn archive_database(
    state: tauri::State<'_, Arc<AppState>>,
) -> Result<String, String> {
    let app = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || -> Result<String, String> {
        // Thin wrapper: delegate to the shared core with the production failpoint
        // (None). The core owns the close → rename → reopen → migrate → resume
        // pipeline + the settings-path preflight + the readiness transitions, so
        // the production path and the recovery failpoint tests exercise the SAME
        // logic (no drift).
        db::recovery::archive_database_core(&app, db::recovery::ArchiveFailpoint::None)
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

/// Result of [`provider_set_active`] (P1.1). A serializable tagged union so the
/// frontend distinguishes "written" from "needs consent" via a structured
/// payload instead of parsing an error string.
///
/// Wire shapes:
/// - `Written` → `{"outcome":"written"}`
/// - `NeedsConsent { actual_scope }` → `{"outcome":"needs_consent","actual_scope":"..."}`
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
#[serde(tag = "outcome", rename_all = "snake_case")]
pub enum SetActiveResult {
    /// The selection was written (no consent needed, or scope already matched).
    Written,
    /// A non-empty parallel selection needs explicit consent; carries the
    /// canonical scope the frontend must echo back via
    /// `provider_confirm_and_set_active`.
    NeedsConsent { actual_scope: String },
}

/// Structured error type for provider IPC commands (P1.1 fix).
/// Replaces free-form `String` errors so the frontend can pattern-match
/// instead of parsing string prefixes.
/// Wire shape for StaleScope: `{"error":"stale_scope","actual_scope":"..."}`
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
#[serde(tag = "error", rename_all = "snake_case")]
pub enum ProviderCommandError {
    StaleScope { actual_scope: String },
    /// Generic database or validation error.
    Db { message: String },
    /// Provider not found, invalid selection, etc.
    Validation { message: String },
}

impl std::fmt::Display for ProviderCommandError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::StaleScope { actual_scope } => {
                write!(f, "stale scope: {actual_scope}")
            }
            Self::Db { message } => write!(f, "{message}"),
            Self::Validation { message } => write!(f, "{message}"),
        }
    }
}

impl From<crate::db::DbError> for ProviderCommandError {
    fn from(e: crate::db::DbError) -> Self {
        ProviderCommandError::Db {
            message: e.to_string(),
        }
    }
}

impl From<String> for ProviderCommandError {
    fn from(message: String) -> Self {
        ProviderCommandError::Validation { message }
    }
}

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

/// Outcome of a `provider_confirm_and_set_active` DB transaction (round-3
/// cleanup #1). Replaces the old `__stale_scope__:` string sentinel smuggled
/// through `DbError::Integrity`: the stale-scope signal now rides out of the
/// `with_conn` closure as a first-class variant, so the outer mapping is a
/// plain `match` with no string-prefix parsing to get wrong.
enum ConfirmActiveOutcome {
    /// Consent written; carries the new `parallel_consent_version`.
    Written { version: i64 },
    /// The frontend's `expected_scope` didn't match the backend-recomputed
    /// canonical scope (it raced a provider change). Carries the actual scope
    /// the frontend must re-echo.
    StaleScope { actual_scope: String },
}

/// Map a [`ConsentError`] (other than `ConsentRequired`, which is handled by
/// the caller via `SetActiveOutcome`) into a [`DbError`] so it can cross the
/// `with_conn` boundary. `ConsentRequired` is only ever surfaced by
/// `provider_set_active` (as `SetActiveOutcome::NeedsConsent`), never by
/// `provider_confirm_and_set_active` (whose stale-scope path is now a typed
/// `ConfirmActiveOutcome` variant, not an error string).
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

/// Shared selection-capture + translate-session pipeline. Used by on_hotkey,
/// translate_selection_ipc (tray + Retry). Emits the popup state per outcome.
///
/// - `supplied_text = Some(t)` (Retry): skip capture, use the saved SOURCE text.
/// - `supplied_text = None` (hotkey/tray): run the selection_lock +
///   capture_selection block on_hotkey used.
/// - `x`, `y`: `Option<f64>` — `Some` = caller-supplied PHYSICAL cursor coords
///   (Retry, tray: the caller captured coordinates itself); `None` = the helper
///   reads `cursor::position()` itself, INSIDE the single `selection_lock()`
///   guard that ALSO runs `capture_selection`, so the cursor read and the
///   clipboard-touching capture are atomic — a second rapid press cannot
///   interleave clipboard save/restore between them (P1-1). The helper resolves
///   the cursor's monitor via `monitor_from_point` and converts to logical via
///   THAT monitor's scale_factor (rev-7-1).
/// - `gen`: the generation token. Checked at every await boundary so a stale
///   run never overwrites a fresher popup (P1-1).
///
/// `to` is passed as `""` so run_translate_session's central resolver handles it.
#[allow(clippy::too_many_arguments)]
async fn capture_and_translate(
    app: &tauri::AppHandle,
    state: &Arc<Session>,
    app_state: &Arc<AppState>,
    supplied_text: Option<String>,
    x: Option<f64>,
    y: Option<f64>,
    gen: u64,
) {
    // 1. Acquire text (capture or supplied).
    let (text, anchor) = match supplied_text {
        Some(t) if !t.is_empty() => {
            let cx = x.unwrap_or(0.0);
            let cy = y.unwrap_or(0.0);
            let anchor = match build_popup_anchor(app, cx, cy) {
                Some(a) => a,
                None => return,
            };
            (t, anchor)
        }
        _ => {
            // The SAME selection_lock + capture_selection(800, owner) block
            // on_hotkey uses.
            let captured: Result<(String, f64, f64), ()> = {
                let _g = state.gen.selection_lock();
                // Read the cursor under the SAME guard as capture_selection so two
                // rapid presses cannot interleave clipboard save/restore between the
                // cursor read and the capture (the capture touches the clipboard).
                let (cx, cy) = match (x, y) {
                    (Some(cx), Some(cy)) => (cx, cy),
                    // hotkey/tray path: no pre-captured coords; read live now.
                    _ => {
                        let pos = cursor::position();
                        (pos.0 as f64, pos.1 as f64)
                    }
                };
                #[cfg(target_os = "windows")]
                let owner = match app
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
                match selection::capture_selection(800, owner) {
                    Ok(selection_engine::Capture::Selected(t)) => Ok((t, cx, cy)),
                    Ok(selection_engine::Capture::NoSelection) => {
                        let anchor = match build_popup_anchor(app, cx, cy) {
                            Some(a) => a,
                            None => return,
                        };
                        let (px, py, pw, ph) =
                            popup::compute_popup_geometry_logical(popup::PopupMode::Error, &anchor);
                        let _ = popup::show_at_sized(app, px, py, pw, ph);
                        let _ = popup::error(
                            app,
                            if !a11y::enabled() {
                                "No selection captured. Grant Accessibility in System Settings → Privacy → Accessibility."
                            } else {
                                "No text selected."
                            },
                        );
                        Err(())
                    }
                    Err(e) => {
                        let anchor = match build_popup_anchor(app, cx, cy) {
                            Some(a) => a,
                            None => return,
                        };
                        let (px, py, pw, ph) =
                            popup::compute_popup_geometry_logical(popup::PopupMode::Error, &anchor);
                        let _ = popup::show_at_sized(app, px, py, pw, ph);
                        let _ = popup::error(app, &e);
                        Err(())
                    }
                }
            };
            if !state.gen.is_latest(gen) {
                return;
            }
            let (text, cx, cy) = match captured {
                Ok(v) => v,
                Err(_) => return,
            };
            let anchor = match build_popup_anchor(app, cx, cy) {
                Some(a) => a,
                None => return,
            };
            (text, anchor)
        }
    };

    // 2. Show loading popup sized + clamped, carrying the source (P1-3).
    if !state.gen.is_latest(gen) {
        return;
    }
    let _ = popup::loading_with_source(app, &anchor, Some(&text));

    // 3. client/keystore guards acquired from Session FIRST.
    let client = match state.client.as_ref() {
        Some(c) => c.clone(),
        None => {
            if state.gen.is_latest(gen) {
                let _ = popup::set_popup_mode(app, popup::PopupMode::Error, &anchor);
                let _ = popup::error_with_source(
                    app,
                    "HTTP client unavailable: startup build failed (recovery required)",
                    &text,
                );
            }
            return;
        }
    };
    let keystore = match state.keystore.as_ref() {
        Some(k) => k,
        None => {
            if state.gen.is_latest(gen) {
                let _ = popup::set_popup_mode(app, popup::PopupMode::Error, &anchor);
                let _ = popup::error_with_source(
                    app,
                    "keystore unavailable: startup init failed (recovery required)",
                    &text,
                );
            }
            return;
        }
    };

    // rev-9-1: acquire the db Arc via spawn_blocking (gate guard INSIDE the closure).
    let app_arc = app_state.clone();
    let db = match tauri::async_runtime::spawn_blocking(move || -> Result<Arc<Database>, String> {
        let _gate = app_arc.data_gate.read();
        require_ready_gated(&app_arc, &_gate)
    })
    .await
    {
        Ok(Ok(db)) => db,
        Ok(Err(msg)) => {
            if state.gen.is_latest(gen) {
                let _ = popup::set_popup_mode(app, popup::PopupMode::Error, &anchor);
                let _ = popup::error_with_source(app, &msg, &text);
            }
            return;
        }
        Err(e) => {
            if state.gen.is_latest(gen) {
                let _ = popup::set_popup_mode(app, popup::PopupMode::Error, &anchor);
                let _ = popup::error_with_source(app, &format!("join error: {e}"), &text);
            }
            return;
        }
    };

    if !state.gen.is_latest(gen) {
        return;
    }

    // 4. run_translate_session — to:"" is resolved centrally inside it.
    let session_result = match run_translate_session(
        &db, &client, keystore, app, &text, "auto", "",
    )
    .await
    {
        Ok(r) => r,
        Err(msg) => {
            if state.gen.is_latest(gen) {
                let _ = popup::set_popup_mode(app, popup::PopupMode::Error, &anchor);
                let _ = popup::error_with_source(app, &msg, &text);
            }
            return;
        }
    };
    if !state.gen.is_latest(gen) {
        return;
    }

    // 5. Route per decision + size per state.
    match decide_clipboard_popup(&session_result) {
        ClipboardPopupDecision::SingleSuccess { text: t, engine } => {
            let _ = popup::set_popup_mode(app, popup::PopupMode::Single, &anchor);
            let _ = popup::result_with_source(app, &t, &engine, &text);
        }
        ClipboardPopupDecision::Multi => {
            let _ = popup::set_popup_mode(app, popup::PopupMode::Multi, &anchor);
            let _ = popup::multi_result_with_source(app, &session_result.outcomes, &text);
        }
        ClipboardPopupDecision::Error(msg) => {
            let _ = popup::set_popup_mode(app, popup::PopupMode::Error, &anchor);
            let _ = popup::error_with_source(app, &msg, &text);
        }
    }
}

/// Build a PopupAnchor from the physical cursor coords. The scale factor used
/// to convert the work area AND the cursor is the TARGET MONITOR's
/// `scale_factor()` — NOT the popup window's.
fn build_popup_anchor(app: &tauri::AppHandle, x_phys: f64, y_phys: f64) -> Option<popup::PopupAnchor> {
    let win = app.get_webview_window("popup")?;

    let monitor = app
        .monitor_from_point(x_phys, y_phys)
        .ok()
        .flatten()
        .or_else(|| app.primary_monitor().ok().flatten());

    let mut sf = match &monitor {
        Some(m) => m.scale_factor(),
        None => win.scale_factor().unwrap_or(1.0),
    };
    if !(sf > 0.0 && sf.is_finite()) {
        sf = 1.0;
    }
    let cursor_logical = (x_phys / sf, y_phys / sf);

    let work_area_logical = if let Some(m) = &monitor {
        let wa = m.work_area();
        let pos = &wa.position;
        let sz = &wa.size;
        let left = pos.x as f64 / sf;
        let top = pos.y as f64 / sf;
        let right = left + sz.width as f64 / sf;
        let bottom = top + sz.height as f64 / sf;
        popup::LogicalWorkArea { left, top, right, bottom }
    } else {
        let (cx, cy) = cursor_logical;
        popup::LogicalWorkArea { left: cx, top: cy, right: cx + 1.0, bottom: cy + 1.0 }
    };

    Some(popup::PopupAnchor {
        cursor_logical,
        work_area: work_area_logical,
        scale_factor: sf,
    })
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
    let state = app.state::<Arc<Session>>().inner().clone();
    let gen = state.gen.next();

    let app2 = app.clone();
    tauri::async_runtime::spawn(async move {
        let state = app2.state::<Arc<Session>>().inner().clone();
        let app_state = app2.state::<Arc<AppState>>().inner().clone();

        // The cursor read + capture_selection happen together under ONE lock
        // inside capture_and_translate (so two rapid presses cannot interleave
        // clipboard save/restore between them). on_hotkey no longer takes the
        // lock itself.
        capture_and_translate(&app2, &state, &app_state, None, None, None, gen).await;
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

// ─── R2b Surface 04: system tray menu ──────────────────────────────────────

/// rev-5-4: build the tray for the FIRST time (registers `"main-tray"`).
/// Subsequent updates go through `refresh_tray` → `build_tray_menu` +
/// `tray.set_menu(...)` so we never register a duplicate tray id.
/// Called once from `setup()`. Built last so a tray-init failure does not
/// block DB/keystore/window setup; the caller logs and continues on `Err`.
fn build_tray(app: &tauri::AppHandle) -> tauri::Result<()> {
    use tauri::tray::{MouseButton, TrayIconBuilder, TrayIconEvent};
    let menu = build_tray_menu(app)?;
    TrayIconBuilder::with_id("main-tray")
        .icon(app.default_window_icon().cloned().expect("default window icon"))
        .menu(&menu)
        .tooltip(read_primary_status(app))
        .show_menu_on_left_click(false)
        .on_menu_event(handle_tray_menu_event)
        .on_tray_icon_event(|tray, event| {
            // Double-click on the icon surfaces the main window for
            // discoverability (macOS left-click opens the menu by default;
            // DoubleClick is documented as Windows-only but harmless to match).
            if let TrayIconEvent::DoubleClick {
                button: MouseButton::Left,
                ..
            } = event
            {
                let app = tray.app_handle();
                if let Some(w) = app.get_webview_window("main") {
                    let _ = w.show();
                    let _ = w.set_focus();
                }
            }
        })
        .build(app)?;
    Ok(())
}

/// rev-5-4: build ONLY the menu (reusable by build_tray + refresh_tray). Returns
/// the full menu with the fresh provider list + status item text.
fn build_tray_menu(app: &tauri::AppHandle) -> tauri::Result<tauri::menu::Menu<tauri::Wry>> {
    use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};

    // Quick actions group.
    let sel = MenuItem::with_id(app, "tray.translate-selection", "Translate Selection", true, None::<&str>)?;
    let clip = MenuItem::with_id(app, "tray.translate-clipboard", "Translate Clipboard", true, None::<&str>)?;
    let sep1 = PredefinedMenuItem::separator(app)?;

    // Switch Provider submenu: built from the db at menu-build time;
    // refresh_tray() rebuilds it after provider mutations.
    let switch_sub = build_switch_provider_submenu(app)?;
    let provider_status = MenuItem::with_id(app, "tray.provider-status", read_primary_status(app), false, None::<&str>)?;
    let sep2 = PredefinedMenuItem::separator(app)?;

    // Disabled "Coming later" items (P1-D).
    let ocr = MenuItem::with_id(app, "tray.ocr-capture", "OCR Translate (Coming later)", false, None::<&str>)?;
    let history = MenuItem::with_id(app, "tray.history", "History (Coming later)", false, None::<&str>)?;
    let sep3 = PredefinedMenuItem::separator(app)?;

    // Navigation + system group.
    let settings = MenuItem::with_id(app, "tray.settings", "Settings", true, None::<&str>)?;
    let sep4 = PredefinedMenuItem::separator(app)?;
    let quit = MenuItem::with_id(app, "tray.quit", "Quit", true, None::<&str>)?;

    let menu = Menu::with_items(app, &[
        &sel, &clip, &sep1,
        &switch_sub, &provider_status, &sep2,
        &ocr, &history, &sep3,
        &settings, &sep4,
        &quit,
    ])?;
    Ok(menu)
}

/// Build the Switch Provider submenu from the enabled providers in the db. Each
/// item id encodes the uuid: `tray.switch-<uuid>`. Returns a Submenu.
fn build_switch_provider_submenu(app: &tauri::AppHandle) -> tauri::Result<tauri::menu::Submenu<tauri::Wry>> {
    use tauri::menu::{MenuItem, SubmenuBuilder};
    let mut sub = SubmenuBuilder::new(app, "Switch Provider");
    // Read enabled providers from the db (best-effort; empty submenu on error).
    let enabled: Vec<(String, String)> = read_enabled_providers(app).unwrap_or_default();
    for (uuid, name) in &enabled {
        let item = MenuItem::with_id(app, format!("tray.switch-{uuid}"), name, true, None::<&str>)?;
        sub = sub.item(&item);
    }
    sub.build()
}

/// Read (uuid, name) for enabled providers. Best-effort: returns empty on db error.
fn read_enabled_providers(app: &tauri::AppHandle) -> Result<Vec<(String, String)>, String> {
    use tauri::Manager;
    let app_state = app.state::<Arc<AppState>>().inner().clone();
    let result = tauri::async_runtime::block_on(tauri::async_runtime::spawn_blocking(move || {
        let _gate = app_state.data_gate.read();
        let db = require_ready_gated(&app_state, &_gate)?;
        db.with_conn(|conn| {
            let list = db_providers::list(conn)?;
            Ok(list.into_iter().filter(|p| p.enabled).map(|p| (p.uuid, p.name)).collect::<Vec<_>>())
        })
        .map_err(|e: DbErr| e.to_string())
    }));
    match result {
        Ok(Ok(v)) => Ok(v),
        Ok(Err(_)) => Ok(Vec::new()),
        Err(_) => Ok(Vec::new()),
    }
}

/// Read the primary provider name for the status item. Falls back to "No provider".
fn read_primary_status(app: &tauri::AppHandle) -> String {
    use tauri::Manager;
    let app_state = match app.try_state::<Arc<AppState>>() {
        Some(s) => s.inner().clone(),
        None => return "No provider".into(),
    };
    let result = tauri::async_runtime::block_on(tauri::async_runtime::spawn_blocking(move || {
        let _gate = app_state.data_gate.read();
        let db = match require_ready_gated(&app_state, &_gate) {
            Ok(d) => d,
            Err(_) => return "No provider".to_string(),
        };
        let selection = db.with_conn(|conn| db_providers::read_active_selection(conn));
        match selection {
            Ok(sel) => match sel.primary {
                Some(uuid) => {
                    let name = db.with_conn(|conn| db_providers::get(conn, &uuid)).ok().map(|p| p.name);
                    name.unwrap_or_else(|| "Unknown provider".into())
                }
                None => "No provider".into(),
            },
            Err(_) => "No provider".into(),
        }
    }));
    result.unwrap_or_else(|_| "No provider".into())
}

/// Refresh the tray menu + status after a provider mutation. Called from the
/// eight provider mutation command handlers (P1-5) via `refresh_tray_if_available`.
///
/// rev-5-4: refresh the EXISTING `"main-tray"` in place — rebuild the menu +
/// re-set the status tooltip via `app.tray_by_id("main-tray")`. Rebuilding from
/// scratch via `build_tray` would register a DUPLICATE tray icon (Tauri panics
/// on duplicate id). Instead, fetch the existing tray and update its menu +
/// tooltip. If the tray does not exist yet (first build), fall back to
/// `build_tray`. Errors are PROPAGATED so the wrapper can log them.
pub fn refresh_tray(app: &tauri::AppHandle) -> tauri::Result<()> {
    if let Some(tray) = app.tray_by_id("main-tray") {
        let menu = build_tray_menu(app)?;
        tray.set_menu(Some(menu))?;
        tray.set_tooltip(Some(&read_primary_status(app)))?;
        Ok(())
    } else {
        build_tray(app)
    }
}

/// rev-9-3: best-effort tray refresh after a provider mutation. Wraps
/// `refresh_tray` (which returns `tauri::Result<()>`) so a tray rebuild failure
/// (e.g. tray not yet built during startup) NEVER turns a successful provider
/// write into an error.
pub fn refresh_tray_if_available(app: &tauri::AppHandle) {
    if let Err(e) = refresh_tray(app) {
        log::warn!("tray refresh failed: {e}");
    }
}

/// Menu item handler. Each arm matches a `with_id` string from [`build_tray_menu`].
///
/// The translation entry points emit a `tray-action` event that the main window
/// forwards (its listener invokes the matching backend command). The
/// `tray.switch-<uuid>` arm runs the SYNC `handle_switch_provider` wrapper inside
/// a `spawn_blocking` (rev-18-1/rev-20-4: offload the SYNC SQLite I/O). Settings
/// shows the main window + emits a real `SettingsSection` value (rev-6-3).
fn handle_tray_menu_event(app: &tauri::AppHandle, event: MenuEvent) {
    let id = event.id().as_ref();
    if let Some(uuid) = id.strip_prefix("tray.switch-") {
        // P1-5 + rev-5-4: set this provider as the sole primary, then refresh
        // the tray. On failure the write tx rolled back (old primary preserved);
        // handle_switch_provider surfaces the error in the tray tooltip.
        let app_state = app.state::<Arc<AppState>>().inner().clone();
        let app_clone = app.clone();
        let uuid_owned = uuid.to_string();
        tauri::async_runtime::spawn_blocking(move || {
            let _ = handle_switch_provider(&app_clone, &app_state, &uuid_owned);
        });
        return;
    }
    match id {
        "tray.translate-selection" => {
            let _ = app.emit("tray-action", "translate-selection");
        }
        "tray.translate-clipboard" => {
            let _ = app.emit("tray-action", "translate-clipboard");
        }
        "tray.ocr-capture" => {
            let _ = app.emit("tray-action", "ocr-capture");
        }
        "tray.settings" => {
            if let Some(w) = app.get_webview_window("main") {
                let _ = w.show();
                let _ = w.set_focus();
                // rev-6-3: navigate value is a real SettingsSection union member,
                // NOT the generic "settings" string the type rejects.
                let _ = w.emit("navigate", "provider-center");
            }
        }
        "tray.quit" => {
            app.exit(0);
        }
        _ => {}
    }
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
        .plugin(tauri_plugin_clipboard_manager::init())
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
            // build the Session WITHOUT a keystore (translate will surface a
            // clear error) and record the failure so the DB readiness block
            // below degrades to NeedsKeystoreRecovery.
            let (keystore, keystore_init_error) = match keystore::Keystore::new(dir.clone()) {
                Ok(ks) => (Some(ks), None),
                Err(e) => {
                    log::error!(
                        "keystore init in {} failed: {e}; falling back to temp dir",
                        dir.display()
                    );
                    let fallback_dir = std::env::temp_dir().join("linguaray-keystore");
                    // Try the shared temp fallback, then the PID-uniquified
                    // last-resort (which itself returns Result — no panic).
                    let (ks, lr_err) =
                        match keystore::Keystore::new(fallback_dir) {
                            Ok(ks) => (Some(ks), None),
                            Err(e2) => {
                                log::error!("temp keystore fallback also failed: {e2}");
                                match init_last_resort_keystore() {
                                    Ok(ks) => (Some(ks), None),
                                    // Total failure (OS temp dir unwritable —
                                    // unreachable in practice): Session.keystore
                                    // is None; every keystore-touching command
                                    // surfaces a clear error, and readiness
                                    // degrades to NeedsKeystoreRecovery.
                                    Err(lr) => {
                                        log::error!(
                                            "all last-resort keystore dirs failed: {lr}"
                                        );
                                        (None, Some(lr))
                                    }
                                }
                            }
                        };
                    let reason = match lr_err {
                        Some(lr) => format!(
                            "keystore init in {} failed: {e}; last-resort also failed: {lr}",
                            dir.display()
                        ),
                        None => format!("keystore init in {}: {e}", dir.display()),
                    };
                    (ks, Some(reason))
                }
            };
            // Spec §Privacy: every preset endpoint must be HTTPS (loopback HTTP
            // allowed for local engines like Ollama). Reject at config-load so an
            // invalid/leaked preset never ships a request. A bad preset is logged
            // (not fatal) so a single broken catalog entry can't crash startup —
            // see `validate_all_preset_endpoints`. The invalid list is recorded
            // here; every shipped preset currently validates, so this is the
            // fail-closed seam that surfaces a future bad catalog entry.
            let invalid_presets = validate_all_preset_endpoints();
            let preset_validation_ok = preset_gate_allows_client(&invalid_presets);
            if !preset_validation_ok {
                log::error!(
                    "preset endpoint validation failed for {} preset(s): {:?}; \
                     ALL translation requests are disabled until the catalog is fixed",
                    invalid_presets.len(),
                    invalid_presets
                );
            }
            // Spec §Privacy: no cross-origin redirects. Review P1 #7: a 30s total
            // request timeout so a hung connection can't freeze translate + the
            // popup indefinitely; lets wire::call classify a real Timeout.
            // `build_http_client` returns the ONLY client we ever use (hardened
            // builder with redirect=none). On a builder failure (pathological
            // TLS-init env) we log + degrade: Session.client is None, so every
            // translate path returns a clear "client unavailable" error. We do
            // NOT fall back to a default client — that would drop
            // redirect(Policy::none()), re-opening the cross-origin-redirect
            // leak the policy exists to close.
            let client = match build_http_client() {
                Ok(c) if preset_validation_ok => Some(c),
                Ok(_) => {
                    // Preset validation failed — disable ALL outbound requests
                    // (fail-closed, see `preset_gate_allows_client`). A bad preset
                    // catalog must not ship any request.
                    log::error!("preset validation failed; client disabled (fail-closed)");
                    None
                }
                Err(e) => {
                    log::error!(
                        "{e}; translate is unavailable until the app is restarted in a healthy TLS environment"
                    );
                    None
                }
            };
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

            // 1. Open the DB. Err → db=None (app keeps running; readiness
            //    computed below degrades to NeedsDatabaseRecovery).
            let (db_handle, db_open_result) = match Database::open(&db_path) {
                Ok(db) => (Some(Arc::new(db)), Ok(())),
                Err(e) => (None, Err(format!("open linguaray.db: {e}"))),
            };

            // Compute the pre-migration readiness from the three independent
            // startup outcomes via the priority reducer. P1.4: a failed DB open
            // is LOCKED IN — subsequent settings/keystore errors must NOT mask
            // NeedsDatabaseRecovery (the DB is the foundation; there is nothing
            // for a keystore error to gate if no DB exists). Keystore failure
            // beats settings failure (writes need a usable keystore). See
            // `compute_startup_readiness` + tests/startup_readiness.rs.
            let mut readiness = compute_startup_readiness(
                db_open_result,
                settings_resolution_error.clone(),
                keystore_init_error.clone(),
            );

            // 2-4. Only run migration + resume + preflight when the DB opened
            // AND the keystore initialized in its canonical dir AND the settings
            // path resolved. `startup_migration_guard` is the single source of
            // truth for the refusal decision (round-3 P1.3): a None settings
            // path (resolution failed) must refuse migration entirely — no
            // backup, no DB write — rather than run against a guessed path (on
            // Windows the store plugin targets AppData Roaming while `dir` here
            // is AppLocalData Local, so a guessed path would touch a DIFFERENT
            // settings file). The refusal itself is already reflected in
            // `readiness` by the reducer above (NeedsKeystoreRecovery /
            // MigrationIncomplete "settings_path"), so on Err we just skip
            // migration and keep the rest of setup running.
            if let Some(db) = db_handle.clone() {
                let fp = FailpointCell::none();
                match startup_migration_guard(
                    settings_path.as_deref(),
                    keystore_init_error.as_deref(),
                ) {
                    Ok(settings_path_ref) => {
                        readiness = match run_migration(&db, &keystore_dir, settings_path_ref, &fp)
                        {
                            Ok(()) => {
                                // Resume any in-flight deletes (3-step sweep). A
                                // failure here does NOT exit setup — log + mark
                                // incomplete so the next startup retries.
                                match crate::db::delete::provider_resume_deletions(
                                    &db,
                                    &keystore_dir,
                                ) {
                                    Ok(_) => {
                                        // Final keystore preflight: a Corrupt
                                        // keystore (detected after migration) →
                                        // recovery.
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
                    Err(reason) => {
                        // Refused: keystore init failed or settings path could
                        // not be resolved. Migration is skipped entirely — NO
                        // backup, NO DB write. readiness already carries the
                        // correct degraded state from the reducer above.
                        log::debug!("startup migration refused: {reason}");
                    }
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

            // Surface 04: system tray (R2b). Built LAST so a tray init failure
            // does not block DB/keystore/window/shortcut setup. Log-only on
            // error — the app stays usable without a tray.
            if let Err(e) = build_tray(app.handle()) {
                log::error!("tray init failed: {e}");
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            translate,
            translate_default,
            translate_clipboard,
            translate_session,
            translate_selection_ipc,
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
            provider_get_active_selection,
            // P1 #3: multi-engine consent.
            provider_confirm_and_set_active,
            // P1 #8: provider diagnostics + DB recovery.
            provider_get_models,
            provider_test_connection,
            archive_database,
            open_settings_window
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Task 5a: `get_data_readiness` returns a `DataReadiness` (not a hand-rolled
    /// JSON `String`) so the frontend gets a properly serialized tagged union via
    /// Tauri's auto-serialization. This IS a wire-contract change from the
    /// pre-S2a `String` return: the old command returned a JSON-ENCODED STRING,
    /// the new one ships a JSON object via `#[serde(tag="state", rename_all="snake_case")]`
    /// on `DataReadiness` itself.
    #[test]
    fn read_data_readiness_from_state_returns_typed_object() {
        let dir = tempfile::tempdir().unwrap();
        let state = AppState {
            db: parking_lot::RwLock::new(None),
            data_gate: parking_lot::RwLock::new(()),
            readiness: parking_lot::RwLock::new(DataReadiness::Ready),
            db_path: dir.path().join("linguaray.db"),
            keystore_dir: dir.path().join("keystore"),
            settings_path: Some(dir.path().join("settings.json")),
        };
        let got = state.readiness.read().clone();
        assert_eq!(got, DataReadiness::Ready);

        // Verify the serialized shape is the SAME tagged-union JSON the frontend
        // already consumes (`{"state":"ready"}`), so the signature change does
        // not alter the wire format.
        let json = serde_json::to_string(&got).unwrap();
        assert_eq!(json, "{\"state\":\"ready\"}");
    }

    /// A non-Ready readiness must round-trip with its `reason` payload intact
    /// (this is the case that matters for the recovery banner).
    #[test]
    fn read_data_readiness_preserves_reason_payload() {
        let dir = tempfile::tempdir().unwrap();
        let state = AppState {
            db: parking_lot::RwLock::new(None),
            data_gate: parking_lot::RwLock::new(()),
            readiness: parking_lot::RwLock::new(DataReadiness::NeedsKeystoreRecovery {
                reason: "corrupt envelope".into(),
            }),
            db_path: dir.path().join("linguaray.db"),
            keystore_dir: dir.path().join("keystore"),
            settings_path: Some(dir.path().join("settings.json")),
        };
        let got = state.readiness.read().clone();
        let json = serde_json::to_string(&got).unwrap();
        assert!(json.contains("\"state\":\"needs_keystore_recovery\""), "{json}");
        assert!(json.contains("\"reason\":\"corrupt envelope\""), "{json}");
    }

    /// Task 5b: building the HTTP client must NOT `.expect()`/panic. The hardened
    /// builder (redirect=none + timeouts) is the only client we ever return — on
    /// a builder error we surface `Err` rather than silently degrading to a
    /// privacy-losing default client. This test is network-free: it only checks
    /// the builder succeeds and returns a usable `reqwest::Client`.
    #[test]
    fn build_http_client_returns_usable_client() {
        let c = build_http_client()
            .expect("hardened HTTP client builder must succeed in a normal env");
        // No network: a freshly built client is still a real reqwest::Client. We
        // confirm it's usable by constructing a request (build, not send).
        let _req = c.get("https://invalid.invalid/");
    }

    /// Task 5b: `build_http_client` returns `Result` and must NOT silently fall
    /// back to a default client (which would drop `redirect(Policy::none())`). A
    /// builder error must propagate as `Err`, not be swallowed. In a normal
    /// environment the hardened builder succeeds, so this asserts the Ok shape.
    #[test]
    fn build_http_client_returns_result_not_client() {
        // Signature contract: the function returns Result<Client, String>, not a
        // bare Client. This compiles only if the signature is the Result form,
        // locking in the no-panic / no-silent-fallback contract at the type level.
        let result: Result<reqwest::Client, String> = build_http_client();
        assert!(result.is_ok(), "normal env must build the hardened client");
    }

    /// Task 5b: preset-endpoint validation must NOT `.expect()`/panic. A bad
    /// preset is logged + skipped, not fatal — every shipped preset validates,
    /// so this exercises the happy path AND the skip path via the helper directly.
    #[test]
    fn validate_preset_endpoints_does_not_panic() {
        // All shipped presets are HTTPS/loopback-valid → Ok (empty error list).
        let invalid = validate_all_preset_endpoints();
        assert!(invalid.is_empty(), "shipped presets must all validate: {invalid:?}");

        // A single bad endpoint validates to Err (the per-endpoint check the
        // loop calls), proving the loop would skip rather than panic.
        assert!(
            providers::validate_endpoint("ftp://evil.example/x").is_err(),
            "ftp must be rejected"
        );
    }

    /// Task 5b: `init_last_resort_keystore` must return `Result<Keystore, String>`
    /// and NEVER panic. In a normal environment the OS temp dir is writable, so
    /// this asserts the Ok shape — locking in the no-panic contract at the type
    /// level (the function signature is `Result`, so a panic in the unreachable
    /// final arm would now be a compile error).
    #[test]
    fn init_last_resort_keystore_returns_result_no_panic() {
        // Signature contract: returns Result, not a bare Keystore. Compiles only
        // if the signature is the Result form.
        let result: Result<keystore::Keystore, String> = init_last_resort_keystore();
        assert!(
            result.is_ok(),
            "normal OS temp dir must be writable for a last-resort keystore: {:?}",
            result.err()
        );
    }

    // ─── Round-3 P1.1: preset fail-closed chain, deterministically ──────────
    //
    // The review requirement: prove that when an invalid preset EXISTS, no
    // network request can be produced — validating `validate_endpoint("ftp://…")`
    // in isolation is not enough. These tests pin the full chain:
    //   1. a catalog containing an invalid endpoint surfaces that id,
    //   2. that id flips `preset_gate_allows_client` to false (client disabled),
    //   3. a client-less Session makes `session_client` return Err — the first
    //      barrier every translate entry-point (`translate`, `translate_default`,
    //      `translate_clipboard`, `on_hotkey`) hits before it can build a
    //      request. No client handle ⇒ no request can ever be shipped.
    #[test]
    fn invalid_preset_in_catalog_blocks_client_gate() {
        let bad = providers::ProviderPreset {
            id: "evil".into(),
            label: "Evil".into(),
            endpoint: "ftp://evil.example/x".into(),
            api_kind: crate::wire::ApiKind::OpenAIChat,
            default_model: "x".into(),
            needs_key: true,
        };
        let good = providers::presets()
            .into_iter()
            .next()
            .expect("shipped catalog is non-empty");
        let invalid = validate_preset_endpoints(&[good, bad]);
        assert_eq!(
            invalid,
            vec!["evil".to_string()],
            "the invalid endpoint must surface in the invalid list"
        );
        assert!(
            !preset_gate_allows_client(&invalid),
            "a single invalid preset must disable the client entirely (fail-closed)"
        );
    }

    #[test]
    fn all_valid_presets_keep_client_gate_open() {
        // Positive control: a clean catalog keeps the gate open.
        let invalid = validate_preset_endpoints(&providers::presets());
        assert!(invalid.is_empty(), "shipped catalog must validate: {invalid:?}");
        assert!(preset_gate_allows_client(&invalid));
    }

    #[test]
    fn session_client_refuses_when_client_disabled() {
        // A Session whose client is None (the fail-closed setup outcome) must
        // make `session_client` return Err — the deterministic barrier every
        // translate entry-point uses before building a request. No network, no
        // reqwest involvement: a None handle cannot ship anything.
        let session = Session {
            client: None,
            keystore: None,
            gen: concurrency::GenerationToken::new(),
        };
        let err = session_client(&session).unwrap_err();
        assert!(err.contains("unavailable"), "{err}");
    }

    #[test]
    fn session_client_returns_client_when_present() {
        // Positive control: a healthy Session yields the client, so the barrier
        // only trips on the disabled path (not universally).
        let c = build_http_client().expect("hardened builder succeeds in a normal env");
        let session = Session {
            client: Some(c),
            keystore: None,
            gen: concurrency::GenerationToken::new(),
        };
        assert!(session_client(&session).is_ok());
    }

    // ─── R2a Task 6: translate_clipboard 分支决策 ──────────────────────────────

    #[test]
    fn clipboard_decision_single_success_uses_legacy_event() {
        let result = TranslateSessionResult {
            outcomes: vec![TranslationOutcome {
                uuid: "u1".into(),
                result: Ok(service::Translation { text: "你好".into(), engine: "provider/u1".into() }),
            }],
            actual_engine: Some("provider/u1".into()),
        };
        let d = decide_clipboard_popup(&result);
        assert!(matches!(d, ClipboardPopupDecision::SingleSuccess { .. }));
        if let ClipboardPopupDecision::SingleSuccess { text, engine } = d {
            assert_eq!(text, "你好");
            assert_eq!(engine, "provider/u1");
        }
    }

    #[test]
    fn clipboard_decision_parallel_uses_multi_event() {
        let result = TranslateSessionResult {
            outcomes: vec![
                TranslationOutcome {
                    uuid: "u1".into(),
                    result: Ok(service::Translation { text: "a".into(), engine: "p/u1".into() }),
                },
                TranslationOutcome {
                    uuid: "u2".into(),
                    result: Err(crate::error::Error::LocalNoFallback),
                },
            ],
            actual_engine: None,
        };
        let d = decide_clipboard_popup(&result);
        assert!(matches!(d, ClipboardPopupDecision::Multi));
    }

    #[test]
    fn clipboard_decision_single_failure_is_error() {
        let result = TranslateSessionResult {
            outcomes: vec![TranslationOutcome {
                uuid: "u1".into(),
                result: Err(crate::error::Error::LocalNoFallback),
            }],
            actual_engine: None,
        };
        let d = decide_clipboard_popup(&result);
        match d {
            ClipboardPopupDecision::Error(msg) => assert!(msg.contains("no fallback"), "{msg}"),
            other => panic!("expected Error, got {other:?}"),
        }
    }

    #[test]
    fn clipboard_decision_all_parallel_failed_is_error() {
        let result = TranslateSessionResult {
            outcomes: vec![
                TranslationOutcome { uuid: "u1".into(), result: Err(crate::error::Error::LocalNoFallback) },
                TranslationOutcome { uuid: "u2".into(), result: Err(crate::error::Error::LocalNoFallback) },
            ],
            actual_engine: None,
        };
        let d = decide_clipboard_popup(&result);
        assert!(matches!(d, ClipboardPopupDecision::Error(_)));
    }
}
