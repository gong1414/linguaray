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

use linguaray_lib::db::readiness::DataReadiness;
use linguaray_lib::db::recovery::{archive_database_core, ArchiveFailpoint};
use linguaray_lib::db::Database;
use linguaray_lib::tray_state::{Locale, RecordingRenderer, TrayStateController};
use std::sync::Arc;

/// Build a `TrayStateController` backed by a `RecordingRenderer` for the test
/// `AppState` construction sites (the `tray` field is required by the struct).
fn test_tray() -> TrayStateController {
    TrayStateController::with_renderer(
        Arc::new(RecordingRenderer::default()),
        Locale::En,
    )
}
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
            tray: Arc::new(parking_lot::Mutex::new(test_tray())),
            update_install_in_flight: std::sync::atomic::AtomicBool::new(false),
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
                    .filter(|e| e.file_name().to_string_lossy().contains(".db.broken-"))
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
    assert!(
        h.original_db_exists(),
        "a fresh linguaray.db exists at db_path"
    );
    assert_eq!(
        h.broken_archive_count(),
        1,
        "original DB renamed aside once"
    );
}

// ─── CloseError: failure BEFORE the rename — original DB untouched + usable

#[test]
fn r2_close_error_leaves_original_usable() {
    let h = Harness::new_ready();

    let err = archive_database_core(&h.app, ArchiveFailpoint::CloseError).unwrap_err();
    assert!(err.contains("close"), "error mentions close: {err}");

    // Contract: failure BEFORE the rename. The original DB is untouched and the
    // slot is restored so the app keeps serving.
    assert!(
        h.slot_is_some(),
        "DB restored to the slot after simulated close failure"
    );
    assert!(
        h.original_db_exists(),
        "original linguaray.db still at db_path"
    );
    assert_eq!(h.broken_archive_count(), 0, "no rename happened");
    assert_eq!(
        h.readiness(),
        DataReadiness::Ready,
        "readiness unchanged (caller's value)"
    );
}

// ─── RenameError: failure BEFORE the rename commits — original DB still usable

#[test]
fn r3_rename_error_leaves_original_usable() {
    let h = Harness::new_ready();

    let err = archive_database_core(&h.app, ArchiveFailpoint::RenameError).unwrap_err();
    assert!(err.contains("rename"), "error mentions rename: {err}");

    // Contract: failure BEFORE the rename commits. The original file is still at
    // db_path; the slot is restored by reopening it.
    assert!(
        h.slot_is_some(),
        "DB restored to the slot after simulated rename failure"
    );
    assert!(
        h.original_db_exists(),
        "original linguaray.db still at db_path (not renamed)"
    );
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
//     Zero-change contract: the DB slot, the file, AND the readiness are all
//     untouched. No `.broken-*` archive is created. The DB handle in the slot is
//     still openable (usable) afterwards.

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
        tray: Arc::new(parking_lot::Mutex::new(test_tray())),
        update_install_in_flight: std::sync::atomic::AtomicBool::new(false),
    });

    let err = archive_database_core(&app, ArchiveFailpoint::None).unwrap_err();
    assert!(
        err.contains("settings path unresolved"),
        "preflight error: {err}"
    );

    // Zero-change contract: nothing destructive happened.
    // 1. DB slot unchanged — still Some AND the handle is usable (a with_conn
    //    round-trip succeeds, proving the connection wasn't closed underneath
    //    us).
    let slot = app.db.read().clone();
    assert!(slot.is_some(), "DB still in the slot (not closed/taken)");
    let db = slot.unwrap();
    db.with_conn(ping)
        .expect("slot DB is still open + usable after preflight refusal");

    // 2. DB file still exists at the original path.
    assert!(db_path.exists(), "original linguaray.db untouched");

    // 3. Readiness unchanged from the caller's value.
    assert_eq!(
        *app.readiness.read(),
        DataReadiness::Ready,
        "readiness untouched by preflight refusal"
    );

    // 4. No `.broken-*` archive created.
    assert_eq!(count_broken(dir.path()), 0, "no rename happened");
}

/// Helper: count `*.db.broken-*` archived files in `dir`.
fn count_broken(dir: &std::path::Path) -> usize {
    std::fs::read_dir(dir)
        .map(|entries| {
            entries
                .filter_map(std::result::Result::ok)
                .filter(|e| e.file_name().to_string_lossy().contains(".db.broken-"))
                .count()
        })
        .unwrap_or(0)
}

/// Helper: a `with_conn` closure that proves a DB connection is live + usable.
/// Runs a trivial round-trip and coerces the rusqlite error into DbError (the
/// `From<rusqlite::Error>` impl) so it satisfies `with_conn`'s signature.
fn ping(conn: &mut rusqlite::Connection) -> Result<(), linguaray_lib::db::DbError> {
    conn.execute_batch("SELECT 1")
        .map_err(linguaray_lib::db::DbError::from)
}

// ─── ResumeDeletionsError (Commit C): MigrationIncomplete, fresh DB installed
//     Migration succeeded but the in-flight delete sweep failed. The fresh DB is
//     installed so a retry can proceed; readiness is MigrationIncomplete (NOT
//     Ready) so the recovery banner stays visible.

#[test]
fn r8_resume_deletions_error_is_migration_incomplete() {
    let h = Harness::new_ready();

    let err = archive_database_core(&h.app, ArchiveFailpoint::ResumeDeletionsError).unwrap_err();
    assert!(
        err.contains("resume_deletions"),
        "error mentions resume_deletions: {err}"
    );
    assert!(
        err.contains("simulated"),
        "error carries the injected reason: {err}"
    );

    // Contract: failure AFTER the archive + migration. The fresh DB is installed
    // so a later retry can proceed; readiness is MigrationIncomplete (NOT Ready)
    // so the recovery banner shows — a silent Ready here would hide a real
    // consistency problem (deletes not finalized).
    assert!(h.slot_is_some(), "fresh DB installed for retry");
    assert!(h.original_db_exists(), "fresh linguaray.db at db_path");
    assert_eq!(h.broken_archive_count(), 1, "original DB archived aside");
    assert!(
        matches!(h.readiness(), DataReadiness::MigrationIncomplete { .. }),
        "MigrationIncomplete (not Ready), got {:?}",
        h.readiness()
    );

    // The MigrationIncomplete must carry the resume_deletions checkpoint context.
    if let DataReadiness::MigrationIncomplete { checkpoint, reason } = h.readiness() {
        assert_eq!(
            checkpoint.as_deref(),
            Some("archive_database"),
            "checkpoint tagged with the recovery op name"
        );
        assert!(
            reason.contains("resume_deletions"),
            "reason carries the failing step: {reason}"
        );
    }
}

// ─── Gate concurrency (Commit C): the `data_gate` RwLock is the serialization
//     barrier between `set_key` (read guard) and `archive_database` (write guard).
//     These tests prove the barrier directly: holding one grade blocks the other.
//     The barrier is a plain `RwLock<()>` shared via `AppState`, so we exercise it
//     in isolation (the production commands `set_key` / `archive_keystore` acquire
//     the same guards through their `*_core` entry points — what matters is that
//     the lock semantics serialize the two sides).
//
// Both tests use `thread::scope` + a channel to create DETERMINISTIC overlap:
// one thread holds a guard, the other tries to acquire the conflicting guard,
// we assert it's still blocked after 200ms, then release and assert completion.
// (parking_lot RwLock guards are `Send` within a `thread::scope`.)

#[test]
fn g1_set_key_read_gate_blocks_archive_write_gate() {
    // Thread A (set_key stand-in): holds data_gate.read().
    // Thread B (archive stand-in): tries data_gate.write() — must block until A
    // releases. This proves an in-flight set_key excludes the archive/reset
    // write side.
    use std::sync::mpsc;
    use std::thread;
    use std::time::{Duration, Instant};

    let h = Harness::new_ready();
    let app = h.app.clone();

    let (acquired_tx, acquired_rx) = mpsc::channel();
    let (release_tx, release_rx) = mpsc::channel();
    let (done_tx, done_rx) = mpsc::channel();

    thread::scope(|s| {
        // Thread A: acquire read guard, signal, wait for release signal, drop.
        let app_a = app.clone();
        s.spawn(move || {
            let _read = app_a.data_gate.read();
            acquired_tx.send(()).unwrap();
            // Hold until the main thread signals release.
            let _ = release_rx.recv_timeout(Duration::from_secs(5));
            drop(_read);
        });

        // Wait for A to actually hold the read guard.
        acquired_rx
            .recv_timeout(Duration::from_secs(5))
            .expect("A acquired the read guard");

        // Thread B: try to acquire the WRITE guard — should block.
        let app_b = app.clone();
        let b_handle = s.spawn(move || {
            let _write = app_b.data_gate.write();
            // Reached only after A releases.
            done_tx.send(()).unwrap();
            drop(_write);
        });

        // Assert B is still blocked after 200ms.
        thread::sleep(Duration::from_millis(200));
        assert!(
            !b_handle.is_finished(),
            "archive write gate must block while set_key read gate is held"
        );

        // Release A's read guard.
        release_tx.send(()).unwrap();

        // Now B should complete promptly.
        let start = Instant::now();
        done_rx
            .recv_timeout(Duration::from_secs(5))
            .expect("B completed after A released the read gate");
        assert!(
            start.elapsed() < Duration::from_secs(2),
            "B unblocked promptly after release"
        );
    });
}

#[test]
fn g2_archive_write_gate_blocks_set_key_read_gate() {
    // Thread B (archive stand-in): holds data_gate.write().
    // Thread A (set_key stand-in): tries data_gate.read() — must block until B
    // releases. This proves an in-flight archive/reset excludes set_key reads.
    use std::sync::mpsc;
    use std::thread;
    use std::time::{Duration, Instant};

    let h = Harness::new_ready();
    let app = h.app.clone();

    let (acquired_tx, acquired_rx) = mpsc::channel();
    let (release_tx, release_rx) = mpsc::channel();
    let (done_tx, done_rx) = mpsc::channel();

    thread::scope(|s| {
        // Thread B: acquire WRITE guard, signal, wait for release, drop.
        let app_b = app.clone();
        s.spawn(move || {
            let _write = app_b.data_gate.write();
            acquired_tx.send(()).unwrap();
            let _ = release_rx.recv_timeout(Duration::from_secs(5));
            drop(_write);
        });

        acquired_rx
            .recv_timeout(Duration::from_secs(5))
            .expect("B acquired the write guard");

        // Thread A: try to acquire the READ guard — should block.
        let app_a = app.clone();
        let a_handle = s.spawn(move || {
            let _read = app_a.data_gate.read();
            done_tx.send(()).unwrap();
            drop(_read);
        });

        // Assert A is still blocked after 200ms.
        thread::sleep(Duration::from_millis(200));
        assert!(
            !a_handle.is_finished(),
            "set_key read gate must block while archive write gate is held"
        );

        // Release B's write guard.
        release_tx.send(()).unwrap();

        // Now A should complete promptly.
        let start = Instant::now();
        done_rx
            .recv_timeout(Duration::from_secs(5))
            .expect("A completed after B released the write gate");
        assert!(
            start.elapsed() < Duration::from_secs(2),
            "A unblocked promptly after release"
        );
    });
}

// ─── No busy/lock leak (Commit C): after every archive/reset outcome (success
//     AND every failure mode), the data_gate must be immediately read-acquirable
//     (no leaked write guard) and the readiness + slot must match the documented
//     contract. This runs the full failpoint matrix and re-checks the invariant
//     after each one — if any arm forgot to drop a guard or mis-set readiness,
//     the read-acquire below would either deadlock or the assertion would fire.

/// One row of the failpoint matrix for the lock-leak test: the failpoint to
/// inject, the readiness predicate the recovery contract requires, and the slot
/// predicate (true = Some+usable, false = None).
struct FailpointRow {
    name: &'static str,
    fp: ArchiveFailpoint,
    readiness_ok: fn(&DataReadiness) -> bool,
    slot_some: bool,
}

#[test]
fn g3_no_lock_leak_across_failpoint_matrix() {
    // For each failpoint, run the core and assert the data_gate is immediately
    // read-acquirable afterwards (no leaked write guard) AND the readiness
    // matches the documented contract.
    let matrix: Vec<FailpointRow> = vec![
        FailpointRow {
            name: "None",
            fp: ArchiveFailpoint::None,
            readiness_ok: |r| *r == DataReadiness::Ready,
            slot_some: true,
        },
        FailpointRow {
            name: "CloseError",
            fp: ArchiveFailpoint::CloseError,
            readiness_ok: |r| *r == DataReadiness::Ready,
            slot_some: true,
        },
        FailpointRow {
            name: "RenameError",
            fp: ArchiveFailpoint::RenameError,
            readiness_ok: |r| *r == DataReadiness::Ready,
            slot_some: true,
        },
        FailpointRow {
            name: "ReopenError",
            fp: ArchiveFailpoint::ReopenError,
            readiness_ok: |r| matches!(r, DataReadiness::NeedsDatabaseRecovery { .. }),
            slot_some: false,
        },
        FailpointRow {
            name: "MigrationKeystoreCorrupt",
            fp: ArchiveFailpoint::MigrationKeystoreCorrupt,
            readiness_ok: |r| matches!(r, DataReadiness::NeedsKeystoreRecovery { .. }),
            slot_some: true,
        },
        FailpointRow {
            name: "MigrationOther",
            fp: ArchiveFailpoint::MigrationOther,
            readiness_ok: |r| matches!(r, DataReadiness::MigrationIncomplete { .. }),
            slot_some: true,
        },
        FailpointRow {
            name: "ResumeDeletionsError",
            fp: ArchiveFailpoint::ResumeDeletionsError,
            readiness_ok: |r| matches!(r, DataReadiness::MigrationIncomplete { .. }),
            slot_some: true,
        },
    ];

    for row in &matrix {
        // Fresh harness per failpoint so each starts from a clean Ready state.
        let h = Harness::new_ready();
        let name = row.name;

        let _ = archive_database_core(&h.app, row.fp.clone());

        // 1. data_gate must be immediately read-acquirable (no leaked write
        //    guard). try_read succeeding instantly proves no write guard is
        //    held; a leaked guard would block forever.
        let gate = match h.app.data_gate.try_read() {
            Some(g) => g,
            None => panic!("[{name}] data_gate not read-acquirable — leaked write guard"),
        };
        drop(gate);

        // 2. Readiness matches the documented contract for this failpoint.
        let readiness = h.readiness();
        assert!(
            (row.readiness_ok)(&readiness),
            "[{name}] readiness contract held, got {:?}",
            readiness
        );

        // 3. Slot state matches the contract.
        assert_eq!(
            h.slot_is_some(),
            row.slot_some,
            "[{name}] slot contract held (Some on success, None on ReopenError)"
        );

        // 4. If the slot is Some, the DB is actually openable (usable) — not a
        //    dangling/closed handle. This catches a guard that closed the
        //    connection but didn't clear the slot.
        if row.slot_some {
            if let Some(db) = h.app.db.read().clone() {
                db.with_conn(ping)
                    .unwrap_or_else(|e| panic!("[{name}] slot DB usable: {e}"));
            } else {
                panic!("[{name}] slot expected Some but read None");
            }
        }
    }
}
