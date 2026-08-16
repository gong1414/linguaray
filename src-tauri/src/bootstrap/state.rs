//! Shared application state + database gate helpers (moved verbatim from
//! `lib.rs` in refactor P3.1).

use std::path::PathBuf;
use std::sync::Arc;

use crate::db::readiness::DataReadiness;
use crate::db::Database;
use crate::{concurrency, keystore, tray_state};

/// Shared application state.
///
/// `gen` is the latest-wins token generator (§concurrency): every hotkey trigger
/// bumps it, and every async transition (popup, translate-result) checks
/// `is_latest` before mutating the popup, so a stale in-flight request can never
/// clobber the result of a newer trigger.
pub struct Session {
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
    /// Tray plugin owns the controller; this is the sync façade so
    /// `TranslationGuard::drop` can `finish_translation` on the calling thread.
    pub tray: Arc<parking_lot::Mutex<tray_state::TrayStateController>>,
    /// R5: single-flight guard for `updater_download_install` — swap-to-acquire,
    /// Drop-to-release (see `commands::updater::InstallGuard`).
    pub update_install_in_flight: std::sync::atomic::AtomicBool,
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

/// Resolve the optional `client` from the [`Session`] or return a clear error
/// string. Used by the translate commands so a startup build failure surfaces
/// consistently instead of panicking.
/// Catalog fail-closed tests still drive this façade. Translate commands lease
/// `linguaray.http` instead.
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn session_client(session: &Session) -> Result<&reqwest::Client, String> {
    session.client.as_ref().ok_or_else(|| {
        "HTTP client unavailable: startup build failed (recovery required)".to_string()
    })
}
