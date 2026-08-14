//! LinguaRay — translation core.
//!
//! Thin host: Tauri commands live in `commands/`. Vendor rows live in
//! `linguaray-catalog`. Official features are in-tree Capability/Driver plugins
//! (kernel crate exists; production Fiber hookup waits on K0 Go).
//! Traditional engines are compiled-in fallbacks (`engines/`). Google GTX is
//! isolated as `engines/google_legacy` until the clean-room rewrite.

pub mod a11y;
pub mod adapter;
pub mod clipboard;
pub mod commands;
pub mod concurrency;
pub mod cursor;
pub mod db;
pub mod dict;
pub mod engines;
pub mod error;
pub mod fs_acl;
pub mod history;
pub mod keystore;
pub mod plugins;
pub mod popup;
pub mod providers;
pub mod selection;
pub mod selection_engine;
pub mod service;
pub mod settings;
pub mod shortcuts;
pub mod tray_state;
pub mod uuid_util;
pub mod wire;

use std::path::{Path, PathBuf};
use std::sync::Arc;
use tauri::menu::MenuEvent;
use tauri::Emitter;
use tauri::Manager;
use tauri_plugin_global_shortcut::{Builder as GlobalShortcutBuilder, ShortcutState};

use crate::commands::{
    a11y_status, archive_database, archive_keystore, get_data_readiness, get_settings,
    history_clear_all, history_privacy_status, history_search, history_set_enabled,
    history_set_retention, key_status, keystore_health, open_settings_window,
    provider_confirm_and_set_active, provider_create, provider_delete, provider_duplicate,
    provider_get_active_selection, provider_get_models, provider_list, provider_list_presets,
    provider_reorder, provider_set_active, provider_set_key, provider_test_connection,
    provider_toggle, provider_update, reset_keystore, set_setting, shortcut_check_conflict,
    shortcut_list, shortcut_recording_begin, shortcut_recording_end, shortcut_reset_defaults,
    shortcut_save, translate, translate_clipboard, translate_default, translate_selection_ipc,
    translate_session,
};
use crate::db::migration::{run_migration, FailpointCell, MigrationError};
use crate::db::providers::{self as db_providers};
use crate::db::readiness::DataReadiness;
use crate::db::Database;

type DbErr = crate::db::DbError;
use crate::shortcuts::ShortcutController;

// Re-export so integration tests can reference the error enum as
// `linguaray_lib::Error` (mirrors `service::TranslationOutcome` usage).
pub use crate::error::Error;

pub use crate::commands::providers::{
    db_set_active_primary, handle_switch_provider, handle_switch_provider_core, measure_latency_ms,
    set_key_blocking, ConnectionResult, ModelInfo, ProviderCommandError, SetActiveOutcome,
    SetActiveResult,
};
pub use crate::commands::translate::{
    decide_clipboard_popup, resolve_target_language, run_translate_session_no_settings,
    ClipboardPopupDecision, TranslateRequest, TranslateResult, TranslateSessionRequest,
    TranslateSessionResult,
};

/// Shared application state.
///
/// `gen` is the latest-wins token generator (§concurrency): every hotkey trigger
/// bumps it, and every async transition (popup, translate-result) checks
/// `is_latest` before mutating the popup, so a stale in-flight request can never
/// clobber the result of a newer trigger.
pub(crate) struct Session {
    pub(crate) client: Option<reqwest::Client>,
    pub(crate) keystore: Option<Arc<keystore::Keystore>>,
    pub(crate) gen: concurrency::GenerationToken,
}

/// S2a application state: the SQLite database + data-readiness gate.
///
/// Managed alongside [`Session`] as `Arc<AppState>` (existing translate/key
/// commands keep their `State<'_, Arc<Session>>` signature unchanged — least
/// disruptive). The provider commands added in step 6 take
/// `State<'_, Arc<AppState>>` and gate on the database handle via
/// [`require_database`] / [`require_database_write`]. `DataReadiness` is the
/// banner projection only.
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
    /// rev-13/rev-14 (Task A5): the tray visual-state controller. SYNC
    /// `parking_lot::Mutex` (NOT `tokio::sync::Mutex`) so `TranslationGuard::drop`
    /// runs `finish_translation` synchronously on the calling thread.
    pub tray: Arc<parking_lot::Mutex<tray_state::TrayStateController>>,
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
///
/// §5.7.0: this is a **database** gate, not `DataReadiness == Ready`.
/// `NeedsKeystoreRecovery` still drives the Settings banner and must not
/// block keyless translate or DB-only provider commands.
pub(crate) fn require_database(
    state: &AppState,
    _gate_guard: &parking_lot::RwLockReadGuard<'_, ()>,
) -> Result<Arc<Database>, String> {
    state
        .db
        .read()
        .clone()
        .ok_or_else(|| "Database not available".to_string())
}

/// Same as [`require_database`] but the proof is a WRITE guard (delete /
/// reorder / toggle / set_active). Holding the write guard excludes every
/// other gate holder, so the Arc clone is atomic w.r.t. archive/reset.
pub(crate) fn require_database_write(
    state: &AppState,
    _gate_guard: &parking_lot::RwLockWriteGuard<'_, ()>,
) -> Result<Arc<Database>, String> {
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
        "settings path could not be resolved; migration refused (no backup, no DB write)"
            .to_string()
    })
}

/// Resolve the optional `client` from the [`Session`] or return a clear error
/// string. Used by the translate commands so a startup build failure surfaces
/// consistently instead of panicking.
pub(crate) fn session_client(session: &Session) -> Result<&reqwest::Client, String> {
    session.client.as_ref().ok_or_else(|| {
        "HTTP client unavailable: startup build failed (recovery required)".to_string()
    })
}

/// Resolve the optional `keystore` from the [`Session`] or return a clear error
/// string. Used by the translate / key commands so a startup init failure
/// (degraded `NeedsKeystoreRecovery`) surfaces consistently instead of
/// panicking.
pub(crate) fn session_keystore(session: &Session) -> Result<&keystore::Keystore, String> {
    session
        .keystore
        .as_deref()
        .ok_or_else(|| "keystore unavailable: startup init failed (recovery required)".to_string())
}

// ─── S2a data-readiness + provider commands ──────────────────────────────
//
// All provider commands follow the same shape:
//   1. `spawn_blocking` — rusqlite is blocking; don't hold the async runtime.
//   2. Acquire `data_gate` (read or write) INSIDE the blocking closure. The
//      parking_lot guards are `!Send`, so they must never cross an `.await`;
//      keeping them on the blocking thread for the closure's duration is the
//      one safe pattern.
//   3. `require_database` / `require_database_write` — gate on
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
        let candidate =
            std::env::temp_dir().join(format!("linguaray-keystore-lastresort-{pid}-{suffix}"));
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
        // setup_required rows (Azure / Custom) ship with an empty endpoint.
        // They are uncallable until the user pastes a URL; they must not fail
        // the whole client gate.
        if p.endpoint.is_empty() {
            continue;
        }
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

/// Shared DB cleanup + readiness update for keystore recovery
/// (`archive_keystore` / `reset_keystore`). Review P1 #2.
///
/// Must be called while the caller holds the `data_gate` write guard so no
/// provider command races the cleanup. Performs (inside one DB transaction):
/// 1. `UPDATE providers SET enabled=0 WHERE needs_key=1` — a provider that needs
///    a key can't be used until the user re-enters one (the archived keystore
///    just lost them all). Keyless providers (Ollama, traditional engines) keep
///    their enabled state.
/// 2. Disable encrypted history and delete history/vocabulary ciphertext that
///    can no longer be decrypted after the key archive/reset.
/// 3. Clear `primary_uuid`, `parallel_uuids`, `fallback_uuid` — the prior
///    selection referenced providers whose keys may be gone, so a stale
///    selection can't drive a translate.
/// 4. Clear `parallel_consent_version` / `parallel_consent_scope` — consent was
///    given for the now-archived key set.
/// 5. `UPDATE _schema_migrations SET migration_complete=1` — a recovery completes
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
pub(crate) fn apply_keystore_recovery_db_cleanup(app: &Arc<AppState>) -> Result<(), String> {
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
                tx.execute("UPDATE providers SET enabled=0 WHERE needs_key=1", [])?;
                // 2. The history/vocabulary key was just archived. Keeping
                // undecryptable ciphertext would strand private data while the
                // UI claims recovery succeeded, so clear it atomically and
                // revoke future history consent.
                tx.execute("DELETE FROM history_sessions", [])?;
                tx.execute("DELETE FROM vocabulary", [])?;
                tx.execute("UPDATE preferences SET history_enabled=0 WHERE id=1", [])?;
                // 3-4. Clear active selection + consent.
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
pub(crate) fn on_hotkey(
    app: &tauri::AppHandle,
    _shortcut: &tauri_plugin_global_shortcut::Shortcut,
    event: tauri_plugin_global_shortcut::ShortcutEvent,
) {
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
        crate::commands::translate::capture_and_translate(
            &app2, &state, &app_state, None, None, None, gen,
        )
        .await;
    });
}

/// Show the input-translate window (bound to `Ctrl+Space`).
///
/// Unlike `on_hotkey` (Alt+Space), this is a pure UI toggle — no selection capture,
/// no translate call, no popup, no generation token. It just surfaces the
/// pre-declared `input` webview window so the user can type text into InputPanel.
pub(crate) fn on_input_hotkey(
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
    // P1-2: the tray data readers + menu builder are `async`. `build_tray` runs
    // exactly once from `setup()` (sync, on the main thread, before the runtime
    // serves commands), so a SINGLE `block_on` driving both awaits is safe here
    // — it cannot nest inside an async worker thread the way the tray refresh
    // path can. This is the ONLY legitimate `block_on` in the tray path.
    let (menu, status) = tauri::async_runtime::block_on(async {
        let menu = build_tray_menu(app).await?;
        let status = read_primary_status(app).await;
        Ok::<_, tauri::Error>((menu, status))
    })?;
    TrayIconBuilder::with_id("main-tray")
        .icon(
            app.default_window_icon()
                .cloned()
                .expect("default window icon"),
        )
        .menu(&menu)
        .tooltip(status)
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
///
/// P1-2: `async fn` — awaits the async DB readers instead of nesting `block_on`.
async fn build_tray_menu(app: &tauri::AppHandle) -> tauri::Result<tauri::menu::Menu<tauri::Wry>> {
    use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};

    // Quick actions group.
    let sel = MenuItem::with_id(
        app,
        "tray.translate-selection",
        "Translate Selection",
        true,
        None::<&str>,
    )?;
    let clip = MenuItem::with_id(
        app,
        "tray.translate-clipboard",
        "Translate Clipboard",
        true,
        None::<&str>,
    )?;
    let sep1 = PredefinedMenuItem::separator(app)?;

    // Switch Provider submenu: built from the db at menu-build time;
    // refresh_tray() rebuilds it after provider mutations.
    let enabled = read_enabled_providers(app).await;
    let switch_sub = build_switch_provider_submenu(app, &enabled)?;
    let provider_status = MenuItem::with_id(
        app,
        "tray.provider-status",
        read_primary_status(app).await,
        false,
        None::<&str>,
    )?;
    let sep2 = PredefinedMenuItem::separator(app)?;

    // Disabled "Coming later" items (P1-D).
    let ocr = MenuItem::with_id(
        app,
        "tray.ocr-capture",
        "OCR Translate (Coming later)",
        false,
        None::<&str>,
    )?;
    let history = MenuItem::with_id(
        app,
        "tray.history",
        "History (Coming later)",
        false,
        None::<&str>,
    )?;
    let sep3 = PredefinedMenuItem::separator(app)?;

    // Navigation + system group.
    let settings = MenuItem::with_id(app, "tray.settings", "Settings", true, None::<&str>)?;
    let sep4 = PredefinedMenuItem::separator(app)?;
    let quit = MenuItem::with_id(app, "tray.quit", "Quit", true, None::<&str>)?;

    let menu = Menu::with_items(
        app,
        &[
            &sel,
            &clip,
            &sep1,
            &switch_sub,
            &provider_status,
            &sep2,
            &ocr,
            &history,
            &sep3,
            &settings,
            &sep4,
            &quit,
        ],
    )?;
    Ok(menu)
}

/// Build the Switch Provider submenu from the given `(uuid, name)` pairs. Each
/// item id encodes the uuid: `tray.switch-<uuid>`. Returns a Submenu.
///
/// P1-2: the DB read is no longer performed here — the caller (an async fn)
/// reads the providers via [`read_enabled_providers`] and passes the slice in,
/// so this builder stays sync and `block_on`-free.
fn build_switch_provider_submenu(
    app: &tauri::AppHandle,
    enabled: &[(String, String)],
) -> tauri::Result<tauri::menu::Submenu<tauri::Wry>> {
    use tauri::menu::{MenuItem, SubmenuBuilder};
    let mut sub = SubmenuBuilder::new(app, "Switch Provider");
    for (uuid, name) in enabled {
        let item = MenuItem::with_id(app, format!("tray.switch-{uuid}"), name, true, None::<&str>)?;
        sub = sub.item(&item);
    }
    sub.build()
}

/// Read (uuid, name) for enabled providers. Best-effort: returns empty on db
/// error.
///
/// P1-2: this is an `async fn` that drives the blocking DB read via
/// `spawn_blocking().await`. It MUST NOT use `block_on(spawn_blocking(...))`
/// because it is awaited from async command handlers — nesting `block_on`
/// inside the async runtime risks a runtime panic ("Cannot start a runtime from
/// within a runtime"). The single legitimate `block_on` caller is `build_tray`,
/// which runs once in `setup()` (sync, before the runtime serves commands).
async fn read_enabled_providers(app: &tauri::AppHandle) -> Vec<(String, String)> {
    use tauri::Manager;
    let app_state = app.state::<Arc<AppState>>().inner().clone();
    match tauri::async_runtime::spawn_blocking(move || {
        let _gate = app_state.data_gate.read();
        let db = require_database(&app_state, &_gate)?;
        db.with_conn(|conn| {
            let list = db_providers::list(conn)?;
            Ok(list
                .into_iter()
                .filter(|p| p.enabled)
                .map(|p| (p.uuid, p.name))
                .collect::<Vec<_>>())
        })
        .map_err(|e: DbErr| e.to_string())
    })
    .await
    {
        Ok(Ok(v)) => v,
        _ => Vec::new(),
    }
}

/// Read the primary provider name for the status item. Falls back to
/// "No provider".
///
/// P1-2: `async fn` driving the blocking DB read via `spawn_blocking().await`
/// (see [`read_enabled_providers`]). No `block_on`.
async fn read_primary_status(app: &tauri::AppHandle) -> String {
    use tauri::Manager;
    let app_state = match app.try_state::<Arc<AppState>>() {
        Some(s) => s.inner().clone(),
        None => return "No provider".into(),
    };
    match tauri::async_runtime::spawn_blocking(move || {
        let _gate = app_state.data_gate.read();
        let db = match require_database(&app_state, &_gate) {
            Ok(d) => d,
            Err(_) => return "No provider".to_string(),
        };
        let selection = db.with_conn(|conn| db_providers::read_active_selection(conn));
        match selection {
            Ok(sel) => match sel.primary {
                Some(uuid) => {
                    let name = db
                        .with_conn(|conn| db_providers::get(conn, &uuid))
                        .ok()
                        .map(|p| p.name);
                    name.unwrap_or_else(|| "Unknown provider".into())
                }
                None => "No provider".into(),
            },
            Err(_) => "No provider".into(),
        }
    })
    .await
    {
        Ok(s) => s,
        Err(_) => "No provider".into(),
    }
}

/// Refresh the tray menu + status after a provider mutation. Called from the
/// eight provider mutation command handlers (P1-5) via `refresh_tray_if_available`.
///
/// rev-5-4: refresh the EXISTING `"main-tray"` in place — rebuild the menu +
/// re-set the status tooltip via `app.tray_by_id("main-tray")`. Rebuilding from
/// scratch via the setup-time builder would register a DUPLICATE tray icon
/// (Tauri panics on duplicate id). Instead, fetch the existing tray and update
/// its menu + tooltip. Errors are PROPAGATED so the wrapper can log them.
///
/// P1-2 (R2-A): if the tray does not exist yet, this is a NO-OP. The
/// setup-time first-build helper nests a single legitimate blocking drive of the
/// runtime, but that is safe ONLY in `setup()` (sync, on the main thread, before
/// the runtime serves commands). Calling it from here — an `async fn` awaited on
/// a runtime worker thread — would nest that blocking drive inside the async
/// runtime and risk a panic ("Cannot start a runtime from within a runtime").
/// The tray is built exactly once in `setup()`; a refresh finding no tray has
/// nothing to update, so it returns `Ok(())`. The tray will be present on the
/// next launch / setup run.
pub async fn refresh_tray(app: &tauri::AppHandle) -> tauri::Result<()> {
    if let Some(tray) = app.tray_by_id("main-tray") {
        let menu = build_tray_menu(app).await?;
        tray.set_menu(Some(menu))?;
        tray.set_tooltip(Some(&read_primary_status(app).await))?;
        Ok(())
    } else {
        // P1-2: Do NOT reach for the setup-time tray builder here — it nests a
        // runtime blocking drive that is unsafe inside this async context. The
        // tray is built once in setup(); if it is absent, there is nothing to
        // refresh yet.
        log::debug!("refresh_tray: main-tray not found, skipping refresh");
        Ok(())
    }
}

/// rev-9-3: best-effort tray refresh after a provider mutation. Wraps
/// `refresh_tray` (which returns `tauri::Result<()>`) so a tray rebuild failure
/// (e.g. tray not yet built during startup) NEVER turns a successful provider
/// write into an error.
///
/// P1-2: `async fn` — awaits [`refresh_tray`]. Callers in async command
/// handlers `.await` this directly; the SYNC `handle_switch_provider` (runs in
/// `spawn_blocking`) detaches it via `tauri::async_runtime::spawn`.
pub async fn refresh_tray_if_available(app: &tauri::AppHandle) {
    if let Err(e) = refresh_tray(app).await {
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
        // R2-B (P1-3 residual): allocate the switch revision in the SYNC menu
        // callback BEFORE spawn_blocking, so revision order = click order
        // regardless of OS thread scheduling. The pre-allocated `rev` is passed
        // into the spawned closure (the core no longer calls begin_switch itself).
        let rev = app_state.tray.lock().begin_switch();
        let app_clone = app.clone();
        let uuid_owned = uuid.to_string();
        tauri::async_runtime::spawn_blocking(move || {
            let _ = handle_switch_provider(&app_clone, &app_state, &uuid_owned, rev);
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

// ─── R3b Surface 07: revisioned global shortcuts ─────────────────────────

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
            let keystore = keystore.map(Arc::new);
            app.manage(Arc::new(Session {
                client: client.clone(),
                keystore: keystore.clone(),
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

            let startup_ready = readiness.is_ready();
            app.manage(Arc::new(AppState {
                db: parking_lot::RwLock::new(db_handle),
                data_gate: parking_lot::RwLock::new(()),
                readiness: parking_lot::RwLock::new(readiness),
                db_path,
                keystore_dir,
                settings_path,
                tray: Arc::new(parking_lot::Mutex::new(
                    tray_state::TrayStateController::new(app.handle().clone()),
                )),
            }));

            // S2b retention is enforced at startup, independently of whether
            // history is currently enabled. Disabling history intentionally
            // preserves existing encrypted rows, but rows older than the
            // consented retention window must still be removed. Favorites are
            // excluded by `cleanup_expired_now`. Failure is fail-soft because
            // cleanup must never prevent translation or recovery UI startup.
            if startup_ready {
                if let Some(history_db) = app.state::<Arc<AppState>>().db.read().clone() {
                    match history_db.with_conn(crate::history::cleanup_expired_now) {
                        Ok(removed) if removed > 0 => {
                            log::info!("expired encrypted history removed: {removed}");
                        }
                        Ok(_) => {}
                        Err(error) => {
                            log::warn!("encrypted history retention cleanup failed: {error}");
                        }
                    }
                }
            }

            // R3b Surface 07: seed/load the persisted revisioned shortcut map,
            // then atomically bind every available action through one registrar.
            // Startup OS conflicts are fail-soft and appear in shortcut_list;
            // DB failures leave the Settings page in its retryable load-error
            // state without preventing the rest of the application from starting.
            let shortcut_db = app
                .state::<Arc<AppState>>()
                .db
                .read()
                .clone();
            let mut shortcuts_plugin = None;
            if let Some(shortcut_db) = shortcut_db {
                let registrar = Arc::new(crate::plugins::shortcuts::TauriShortcutRegistrar::new(
                    app.handle().clone(),
                ));
                match ShortcutController::load(shortcut_db, registrar.clone()) {
                    Ok(controller) => {
                        let controller = Arc::new(controller);
                        app.manage(controller.clone());
                        shortcuts_plugin = Some(Arc::new(
                            crate::plugins::shortcuts::ShortcutsPlugin::new(registrar, controller),
                        ));
                    }
                    Err(error) => {
                        log::error!("shortcut controller startup failed: {error}");
                    }
                }
            }

            let database_plugin = Arc::new(crate::plugins::database::DatabasePlugin::new(
                app.state::<Arc<AppState>>().db.read().clone(),
            ));
            let secrets_plugin =
                Arc::new(crate::plugins::secrets::SecretsPlugin::new(keystore.clone()));
            let http_plugin = Arc::new(crate::plugins::http::HttpPlugin::new(client));
            match linguaray_kernel::Supervisor::compose(crate::plugins::builtin_plugins(
                database_plugin,
                secrets_plugin,
                http_plugin,
                shortcuts_plugin,
            )) {
                Ok(supervisor) => {
                    tauri::async_runtime::block_on(supervisor.enable_all());
                    app.manage(supervisor);
                }
                Err(error) => {
                    log::error!("kernel compose failed: {error}");
                }
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
            key_status,
            get_settings,
            set_setting,
            a11y_status,
            keystore_health,
            archive_keystore,
            reset_keystore,
            // S2a data-readiness + provider CRUD.
            get_data_readiness,
            provider_list_presets,
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
            open_settings_window,
            shortcut_list,
            shortcut_check_conflict,
            shortcut_save,
            shortcut_reset_defaults,
            shortcut_recording_begin,
            shortcut_recording_end,
            history_privacy_status,
            history_set_enabled,
            history_set_retention,
            history_clear_all,
            history_search
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|app, event| {
            if let tauri::RunEvent::Exit = event {
                if let Some(supervisor) = app.try_state::<linguaray_kernel::Supervisor>() {
                    let supervisor = supervisor.inner().clone();
                    tauri::async_runtime::spawn(async move {
                        supervisor.shutdown().await;
                    });
                }
            }
        });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::service::TranslationOutcome;

    /// Build a `TrayStateController` backed by a `RecordingRenderer` for unit
    /// tests that construct an `AppState` (the `tray` field is required by the
    /// struct but these tests only inspect `readiness`).
    fn test_tray_controller() -> tray_state::TrayStateController {
        tray_state::TrayStateController::with_renderer(
            Arc::new(tray_state::RecordingRenderer::default()),
            tray_state::Locale::En,
        )
    }

    #[test]
    fn keystore_recovery_disables_and_clears_irrecoverable_encrypted_content() {
        let dir = tempfile::tempdir().unwrap();
        let db_path = dir.path().join("recovery-history.db");
        let db = Arc::new(Database::open(&db_path).unwrap());
        db.with_conn(|conn| {
            let tx = conn.transaction()?;
            crate::db::schema::create_all_tables(&tx)?;
            crate::db::schema::seed_singletons(&tx)?;
            tx.execute("UPDATE preferences SET history_enabled=1 WHERE id=1", [])?;
            tx.execute(
                "INSERT INTO history_sessions
                 (session_uuid,timestamp,trigger_source,target_language,is_favorite,
                  source_text_encrypted,source_text_nonce,crypto_version)
                 VALUES ('s1',1,'input','zh',0,X'AA',X'000102030405060708090A0B',1)",
                [],
            )?;
            tx.execute(
                "INSERT INTO vocabulary
                 (item_uuid,timestamp,source_language,target_language,word_encrypted,
                  word_nonce,definition_encrypted,definition_nonce,crypto_version)
                 VALUES ('v1',1,'en','zh',X'AA',X'000102030405060708090A0B',
                         X'BB',X'000102030405060708090A0B',1)",
                [],
            )?;
            tx.commit()?;
            Ok(())
        })
        .unwrap();
        let app = Arc::new(AppState {
            db: parking_lot::RwLock::new(Some(db.clone())),
            data_gate: parking_lot::RwLock::new(()),
            readiness: parking_lot::RwLock::new(DataReadiness::Ready),
            db_path,
            keystore_dir: dir.path().join("keystore"),
            settings_path: Some(dir.path().join("settings.json")),
            tray: Arc::new(parking_lot::Mutex::new(test_tray_controller())),
        });

        apply_keystore_recovery_db_cleanup(&app).unwrap();
        let state = db
            .with_conn(|conn| {
                Ok((
                    conn.query_row(
                        "SELECT history_enabled FROM preferences WHERE id=1",
                        [],
                        |row| row.get::<_, i64>(0),
                    )?,
                    conn.query_row("SELECT COUNT(*) FROM history_sessions", [], |row| {
                        row.get::<_, i64>(0)
                    })?,
                    conn.query_row("SELECT COUNT(*) FROM vocabulary", [], |row| {
                        row.get::<_, i64>(0)
                    })?,
                ))
            })
            .unwrap();
        assert_eq!(state, (0, 0, 0));
    }

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
            tray: Arc::new(parking_lot::Mutex::new(test_tray_controller())),
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
            tray: Arc::new(parking_lot::Mutex::new(test_tray_controller())),
        };
        let got = state.readiness.read().clone();
        let json = serde_json::to_string(&got).unwrap();
        assert!(
            json.contains("\"state\":\"needs_keystore_recovery\""),
            "{json}"
        );
        assert!(json.contains("\"reason\":\"corrupt envelope\""), "{json}");
    }

    /// Task 5b: building the HTTP client must NOT `.expect()`/panic. The hardened
    /// builder (redirect=none + timeouts) is the only client we ever return — on
    /// a builder error we surface `Err` rather than silently degrading to a
    /// privacy-losing default client. This test is network-free: it only checks
    /// the builder succeeds and returns a usable `reqwest::Client`.
    #[test]
    fn build_http_client_returns_usable_client() {
        let c =
            build_http_client().expect("hardened HTTP client builder must succeed in a normal env");
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
        assert!(
            invalid.is_empty(),
            "shipped presets must all validate: {invalid:?}"
        );

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
            protocol: linguaray_contracts::ProtocolKind::OpenaiChat,
            default_model: "x".into(),
            needs_key: true,
            auth: linguaray_contracts::AuthKind::Bearer,
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
        assert!(
            invalid.is_empty(),
            "shipped catalog must validate: {invalid:?}"
        );
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

    #[test]
    fn database_gate_allows_keystore_recovery_banner() {
        let dir = tempfile::tempdir().unwrap();
        let db_path = dir.path().join("gate.db");
        let db = Arc::new(Database::open(&db_path).unwrap());
        let state = AppState {
            db: parking_lot::RwLock::new(Some(db)),
            data_gate: parking_lot::RwLock::new(()),
            readiness: parking_lot::RwLock::new(DataReadiness::NeedsKeystoreRecovery {
                reason: "corrupt".into(),
            }),
            db_path,
            keystore_dir: dir.path().join("keystore"),
            settings_path: Some(dir.path().join("settings.json")),
            tray: Arc::new(parking_lot::Mutex::new(test_tray_controller())),
        };
        let gate = state.data_gate.read();
        assert!(
            require_database(&state, &gate).is_ok(),
            "NeedsKeystoreRecovery must not block the database gate"
        );
    }

    #[test]
    fn database_gate_fails_without_handle() {
        let dir = tempfile::tempdir().unwrap();
        let state = AppState {
            db: parking_lot::RwLock::new(None),
            data_gate: parking_lot::RwLock::new(()),
            readiness: parking_lot::RwLock::new(DataReadiness::Ready),
            db_path: dir.path().join("missing.db"),
            keystore_dir: dir.path().join("keystore"),
            settings_path: Some(dir.path().join("settings.json")),
            tray: Arc::new(parking_lot::Mutex::new(test_tray_controller())),
        };
        let gate = state.data_gate.read();
        assert!(require_database(&state, &gate).is_err());
    }

    // ─── R2a Task 6: translate_clipboard 分支决策 ──────────────────────────────

    #[test]
    fn clipboard_decision_single_success_uses_legacy_event() {
        let result = TranslateSessionResult {
            outcomes: vec![TranslationOutcome {
                uuid: "u1".into(),
                result: Ok(service::Translation {
                    text: "你好".into(),
                    engine: "provider/u1".into(),
                }),
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
                    result: Ok(service::Translation {
                        text: "a".into(),
                        engine: "p/u1".into(),
                    }),
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
                TranslationOutcome {
                    uuid: "u1".into(),
                    result: Err(crate::error::Error::LocalNoFallback),
                },
                TranslationOutcome {
                    uuid: "u2".into(),
                    result: Err(crate::error::Error::LocalNoFallback),
                },
            ],
            actual_engine: None,
        };
        let d = decide_clipboard_popup(&result);
        assert!(matches!(d, ClipboardPopupDecision::Error(_)));
    }

    #[test]
    fn parse_model_ids_openai_data_array() {
        let body = serde_json::json!({
            "data": [
                {"id": "gpt-4o-mini", "object": "model"},
                {"id": "gpt-4o", "object": "model"},
                {"object": "model"}
            ]
        });
        assert_eq!(
            crate::commands::providers::parse_model_ids(&body),
            vec!["gpt-4o-mini".to_string(), "gpt-4o".to_string()]
        );
    }

    #[test]
    fn parse_model_ids_anthropic_top_level_array() {
        let body = serde_json::json!([
            {"id": "claude-sonnet-4-5", "type": "model"},
            {"id": "claude-haiku-4-5"}
        ]);
        assert_eq!(
            crate::commands::providers::parse_model_ids(&body),
            vec![
                "claude-sonnet-4-5".to_string(),
                "claude-haiku-4-5".to_string()
            ]
        );
    }

    #[test]
    fn parse_model_ids_unknown_shape_is_empty() {
        let body = serde_json::json!({"models": [{"name": "x"}]});
        assert!(crate::commands::providers::parse_model_ids(&body).is_empty());
    }
}
