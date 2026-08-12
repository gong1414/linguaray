//! R11: backend defense in `provider_set_key`.
//!
//! `provider_set_key` must refuse to write a key when:
//! 1. The key is empty/whitespace (validated at the command boundary before any
//!    DB or keystore work, and again in the blocking core as defense-in-depth).
//! 2. The provider is `needs_key=false` (a keyless provider — e.g. a local
//!    Ollama preset — must never accept a key; writing one would leave a
//!    dangling secret the provider will never read and the UI now hides the key
//!    input for).
//!
//! The checks live in the synchronous, testable core `set_key_blocking`; the
//! `#[tauri::command]` wrapper delegates to it inside `spawn_blocking` and
//! fast-fails on empty keys before spawning a thread.

use linguaray_lib::db::providers;
use linguaray_lib::db::readiness::DataReadiness;
use linguaray_lib::db::schema;
use linguaray_lib::db::Database;
use linguaray_lib::set_key_blocking;
use linguaray_lib::tray_state::{Locale, RecordingRenderer, TrayStateController};
use std::sync::Arc;
use tempfile::TempDir;

/// Build a `TrayStateController` for the test AppState (the `tray` field is
/// required by the struct). Mirrors the recovery-test helper.
fn test_tray() -> TrayStateController {
    TrayStateController::with_renderer(Arc::new(RecordingRenderer::default()), Locale::En)
}

struct Harness {
    _dir: TempDir,
    app: Arc<linguaray_lib::AppState>,
    keystore_dir: std::path::PathBuf,
}

impl Harness {
    /// Fresh temp dir, an OPEN + seeded DB installed in the slot, readiness
    /// Ready, and a keystore dir under the temp dir. Mirrors recovery.rs.
    fn new_ready() -> Self {
        let dir = tempfile::tempdir().unwrap();
        let db_path = dir.path().join("linguaray.db");
        let keystore_dir = dir.path().join("keystore");
        let db = Database::open(&db_path).unwrap();
        db.with_conn(|conn| {
            let tx = conn.transaction()?;
            schema::create_all_tables(&tx)?;
            schema::seed_singletons(&tx)?;
            tx.commit()?;
            Ok(())
        })
        .unwrap();
        let app = Arc::new(linguaray_lib::AppState {
            db: parking_lot::RwLock::new(Some(Arc::new(db))),
            data_gate: parking_lot::RwLock::new(()),
            readiness: parking_lot::RwLock::new(DataReadiness::Ready),
            db_path,
            keystore_dir: keystore_dir.clone(),
            settings_path: None,
            tray: Arc::new(parking_lot::Mutex::new(test_tray())),
        });
        Self {
            _dir: dir,
            app,
            keystore_dir,
        }
    }

    /// Create a provider in the harness DB, returning its profile.
    fn create(
        &self,
        template_id: &str,
        name: &str,
        endpoint: &str,
    ) -> linguaray_lib::db::providers::ProviderProfile {
        self.app
            .db
            .read()
            .as_ref()
            .unwrap()
            .with_conn(|conn| providers::create(conn, template_id, name, endpoint, None))
            .unwrap()
    }

    /// Count files under the keystore dir (a successful set_key writes at least
    /// one; a rejected set_key writes none).
    fn keystore_file_count(&self) -> usize {
        std::fs::read_dir(&self.keystore_dir)
            .map(|entries| entries.filter_map(std::result::Result::ok).count())
            .unwrap_or(0)
    }
}

// ─── R11: needs_key=false rejection (test 6) ──────────────────────────────

/// A keyless provider (Ollama preset → needs_key=false) must NOT accept a key.
/// `set_key_blocking` returns an Err naming the rule, and the keystore is left
/// untouched (no key file written).
#[test]
fn r11_set_key_rejects_keyless_provider() {
    let h = Harness::new_ready();
    let p = h.create("ollama", "Ollama", "http://localhost:11434");
    assert!(!p.needs_key, "Ollama preset must be needs_key=false");

    let err = set_key_blocking(&h.app, &p.uuid, "sk-not-allowed").unwrap_err();
    assert!(
        err.contains("does not require a key"),
        "expected needs_key error, got: {err}"
    );
    // Defense-in-depth: the keystore was NOT written.
    assert_eq!(
        h.keystore_file_count(),
        0,
        "no keystore file should exist after a keyless rejection"
    );
}

// ─── R11: empty key rejection (test 7) ─────────────────────────────────────

/// An empty/whitespace key is rejected before any DB or keystore work. The error
/// names the rule, and the keystore is untouched.
#[test]
fn r11_set_key_rejects_empty_key() {
    let h = Harness::new_ready();
    let p = h.create("openai", "OpenAI", "https://api.openai.com/v1/chat/completions");
    assert!(p.needs_key, "OpenAI preset must be needs_key=true");

    let err = set_key_blocking(&h.app, &p.uuid, "   ").unwrap_err();
    assert!(
        err.contains("must not be empty"),
        "expected empty-key error, got: {err}"
    );
    assert_eq!(h.keystore_file_count(), 0);
}

// ─── R11: happy path (sanity — validation must not be over-eager) ──────────

/// A needs_key=true provider with a non-empty key DOES write the key. Guards
/// against the new validation rejecting legitimate writes.
#[test]
fn r11_set_key_writes_for_needs_key_provider() {
    let h = Harness::new_ready();
    let p = h.create("openai", "OpenAI", "https://api.openai.com/v1/chat/completions");
    set_key_blocking(&h.app, &p.uuid, "sk-real-key")
        .expect("a needs_key provider accepts a non-empty key");
    assert!(
        h.keystore_file_count() >= 1,
        "a key file should exist after a successful set_key"
    );
}

// ─── R11: non-active provider still rejected (pre-existing behavior) ───────

/// A deleting/deleted provider still cannot accept a key (pre-existing status
/// check, unchanged by R11). Asserts the status check still fires after the new
/// needs_key branch is added above it.
#[test]
fn r11_set_key_rejects_non_active_provider() {
    let h = Harness::new_ready();
    let p = h.create("openai", "OpenAI", "https://api.openai.com/v1/chat/completions");
    // Mark the row deleting (a status the keyless branch must not reach first).
    h.app
        .db
        .read()
        .as_ref()
        .unwrap()
        .with_conn(|conn| {
            conn.execute(
                "UPDATE providers SET status='deleting' WHERE uuid=?1",
                rusqlite::params![&p.uuid],
            )?;
            Ok(())
        })
        .unwrap();
    let err = set_key_blocking(&h.app, &p.uuid, "sk-real-key").unwrap_err();
    assert!(
        err.contains("non-active"),
        "expected non-active status error, got: {err}"
    );
    assert_eq!(h.keystore_file_count(), 0);
}
