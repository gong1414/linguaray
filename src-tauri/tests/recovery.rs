//! S2a P1 recovery tests: the `archive_database_core` close → rename → reopen →
//! migrate pipeline, exercised through every [`ArchiveFailpoint`].
//!
//! Each test builds a REAL temp DB + REAL temp keystore + a REAL `AppState`
//! (no Tauri State), runs `archive_database_core` with a specific failpoint, and
//! asserts the resulting slot / readiness / on-disk file state matches the
//! failpoint's documented recovery contract:
//! - failures BEFORE the rename leave the original DB untouched + usable.
//! - failures AFTER the rename leave the slot `None` (or `Some` fresh DB for
//!   migration failures) + a non-`Ready` readiness.
//! - the production path (afp=None) reaches Ready with a fresh, migrated DB.
//!
//! The `None` (production) + close/rename/reopen failpoints run the REAL
//! migration; the migration failpoints inject synthetic errors at the migration
//! step (the migration itself is covered by tests/migration.rs).

use linguaray_lib::db::recovery::{archive_database_core, ArchiveFailpoint};
use linguaray_lib::db::readiness::DataReadiness;
use linguaray_lib::db::Database;
use std::sync::Arc;
use tempfile::TempDir;

/// The app state used by these tests. Mirrors the production `AppState` shape
/// with temp-dir-backed paths.
struct Harness {
    _dir: TempDir,
    app: Arc<linguaray_lib::AppState>,
    db_path: std::path::PathBuf,
}

impl Harness {
    /// Fresh dir, an OPEN DB installed in the slot, readiness Ready, and a
    /// resolved settings_path (pointing at a non-existent file → fresh-install
    /// migration, which completes with no candidates). This is the "happy path"
    /// starting state: archive_database_core should be able to close/renam/
    /// reopen/migrate it back to Ready.
    fn new_ready() -> Self {
        let dir = tempfile::tempdir().unwrap();
        let db_path = dir.path().join("linguaray.db");
        // Open + install a real DB so the close step has something to close.
        let db = Database::open(&db_path).unwrap();
        let app = Arc::new(linguaray_lib::AppState {
            db: parking_lot::RwLock::new(Some(Arc::new(db))),
            data_gate: parking_lot::RwLock::new(()),
            readiness: parking_lot::RwLock::new(DataReadiness::Ready),
            db_path: db_path.clone(),
            keystore_dir: dir.path().join("keystore"),
            settings_path: Some(dir.path().join("settings.json")),
        });
        Self {
            _dir: dir,
            app,
            db_path,
        }
    }

    fn readiness(&self) -> DataReadiness {
        self.app.readiness.read().clone()
    }

    fn slot_is_some(&self) -> bool {
        self.app.db.read().is_some()
    }

    /// Does the original `linguaray.db` still exist at the canonical path?
    fn original_db_exists(&self) -> bool {
        self.db_path.exists()
    }

    /// Count `*.db.broken-*` archived files in the dir.
    fn broken_archive_count(&self) -> usize {
        std::fs::read_dir(self.db_path.parent().unwrap())
            .map(|entries| {
                entries
                    .filter_map(std::result::Result::ok)
                    .filter(|e| {
                        e.file_name()
                            .to_string_lossy()
                            .contains(".db.broken-")
                    })
                    .count()
            })
            .unwrap_or(0)
    }
}

// ─── Production path (afp=None): full close → rename → reopen → migrate → Ready

#[test]
fn r1_production_path_reaches_ready() {
    // The happy path: a Ready DB is archived; the core closes it, renames it
    // aside, reopens a fresh DB at the same path, migrates (fresh install →
    // completes), and lands back at Ready. The original file is renamed to a
    // .broken-* archive; a new linguaray.db takes its place.
    let h = Harness::new_ready();
    assert!(h.original_db_exists(), "original DB exists before archive");
    assert_eq!(h.readiness(), DataReadiness::Ready);

    let archived_path =
        archive_database_core(&h.app, ArchiveFailpoint::None).expect("production path succeeds");
    assert!(!archived_path.is_empty(), "archived path returned");

    assert_eq!(h.readiness(), DataReadiness::Ready, "back to Ready");
    assert!(h.slot_is_some(), "fresh DB installed in the slot");
    assert!(h.original_db_exists(), "a fresh linguaray.db exists at db_path");
    assert_eq!(h.broken_archive_count(), 1, "original DB renamed aside once");
}

// ─── CloseError: failure BEFORE the rename — original DB untouched + usable

#[test]
fn r2_close_error_leaves_original_usable() {
    let h = Harness::new_ready();

    let err = archive_database_core(&h.app, ArchiveFailpoint::CloseError).unwrap_err();
    assert!(err.contains("close"), "error mentions close: {err}");

    // Contract: failure BEFORE the rename. The original DB is untouched and the
    // slot is restored so the app keeps serving.
    assert!(h.slot_is_some(), "DB restored to the slot after simulated close failure");
    assert!(h.original_db_exists(), "original linguaray.db still at db_path");
    assert_eq!(h.broken_archive_count(), 0, "no rename happened");
    assert_eq!(h.readiness(), DataReadiness::Ready, "readiness unchanged (caller's value)");
}

// ─── RenameError: failure BEFORE the rename commits — original DB still usable

#[test]
fn r3_rename_error_leaves_original_usable() {
    let h = Harness::new_ready();

    let err = archive_database_core(&h.app, ArchiveFailpoint::RenameError).unwrap_err();
    assert!(err.contains("rename"), "error mentions rename: {err}");

    // Contract: failure BEFORE the rename commits. The original file is still at
    // db_path; the slot is restored by reopening it.
    assert!(h.slot_is_some(), "DB restored to the slot after simulated rename failure");
    assert!(h.original_db_exists(), "original linguaray.db still at db_path (not renamed)");
    assert_eq!(h.broken_archive_count(), 0, "no archive created");
    assert_eq!(h.readiness(), DataReadiness::Ready, "readiness unchanged");
}

// ─── ReopenError: failure AFTER the rename — slot None, NeedsDatabaseRecovery

#[test]
fn r4_reopen_error_leaves_slot_none_recovery() {
    let h = Harness::new_ready();

    let err = archive_database_core(&h.app, ArchiveFailpoint::ReopenError).unwrap_err();
    assert!(err.contains("reopen"), "error mentions reopen: {err}");

    // Contract: failure AFTER the rename. The file was renamed away and the
    // reopen failed too — there's nothing to serve.
    assert!(!h.slot_is_some(), "slot is None (nothing to serve)");
    assert!(
        !h.original_db_exists(),
        "no linguaray.db at db_path (renamed away, reopen failed)"
    );
    assert_eq!(h.broken_archive_count(), 1, "original DB was renamed aside");
    assert!(
        matches!(h.readiness(), DataReadiness::NeedsDatabaseRecovery { .. }),
        "NeedsDatabaseRecovery, got {:?}",
        h.readiness()
    );
}

// ─── MigrationKeystoreCorrupt: NeedsKeystoreRecovery, fresh DB installed

#[test]
fn r5_migration_keystore_corrupt_needs_keystore_recovery() {
    let h = Harness::new_ready();

    let err =
        archive_database_core(&h.app, ArchiveFailpoint::MigrationKeystoreCorrupt).unwrap_err();
    assert!(
        err.contains("simulated keystore corruption"),
        "error carries the synthetic reason: {err}"
    );

    // Contract: failure AFTER the rename. The fresh DB is installed so a later
    // retry can proceed; readiness routes the user to the keystore flow.
    assert!(h.slot_is_some(), "fresh DB installed for retry");
    assert!(h.original_db_exists(), "fresh linguaray.db at db_path");
    assert_eq!(h.broken_archive_count(), 1, "original DB renamed aside");
    assert!(
        matches!(h.readiness(), DataReadiness::NeedsKeystoreRecovery { .. }),
        "NeedsKeystoreRecovery, got {:?}",
        h.readiness()
    );
}

// ─── MigrationOther: MigrationIncomplete, fresh DB installed

#[test]
fn r6_migration_other_is_migration_incomplete() {
    let h = Harness::new_ready();

    let err = archive_database_core(&h.app, ArchiveFailpoint::MigrationOther).unwrap_err();
    assert!(
        err.contains("simulated migration failure"),
        "error carries the synthetic reason: {err}"
    );

    // Contract: failure AFTER the rename. The fresh (partially-migrated) DB is
    // installed so a later recovery/migration retry can proceed; readiness is
    // MigrationIncomplete so the recovery banner shows.
    assert!(h.slot_is_some(), "fresh DB installed for retry");
    assert!(h.original_db_exists(), "fresh linguaray.db at db_path");
    assert_eq!(h.broken_archive_count(), 1, "original DB renamed aside");
    assert!(
        matches!(h.readiness(), DataReadiness::MigrationIncomplete { .. }),
        "MigrationIncomplete, got {:?}",
        h.readiness()
    );
}

// ─── Preflight (Fix 2): settings_path = None refuses BEFORE any destructive op

#[test]
fn r7_unresolved_settings_path_refuses_before_destructive_ops() {
    // settings_path = None must refuse IMMEDIATELY — before close/rename. The
    // original DB stays installed and usable; no archive is created.
    let dir = tempfile::tempdir().unwrap();
    let db_path = dir.path().join("linguaray.db");
    let db = Database::open(&db_path).unwrap();
    let app = Arc::new(linguaray_lib::AppState {
        db: parking_lot::RwLock::new(Some(Arc::new(db))),
        data_gate: parking_lot::RwLock::new(()),
        readiness: parking_lot::RwLock::new(DataReadiness::Ready),
        db_path: db_path.clone(),
        keystore_dir: dir.path().join("keystore"),
        settings_path: None, // unresolved — preflight must refuse.
    });

    let err = archive_database_core(&app, ArchiveFailpoint::None).unwrap_err();
    assert!(
        err.contains("settings path unresolved"),
        "preflight error: {err}"
    );

    // Nothing destructive happened.
    assert!(app.db.read().is_some(), "DB still in the slot (not closed)");
    assert!(db_path.exists(), "original linguaray.db untouched");
    let broken = std::fs::read_dir(dir.path())
        .map(|entries| {
            entries
                .filter_map(std::result::Result::ok)
                .filter(|e| e.file_name().to_string_lossy().contains(".db.broken-"))
                .count()
        })
        .unwrap_or(0);
    assert_eq!(broken, 0, "no rename happened");
}
