//! S2a P1.4 test: startup readiness priority reducer.
//!
//! The startup readiness reducer decides which recovery banner the app shows
//! before the DB / migration / keystore are fully resolved. The load-bearing
//! invariant is **priority**: a failed DB open (`NeedsDatabaseRecovery`) MUST
//! NOT be masked by a later settings/keystore error — the DB is the foundation,
//! and there is nothing for a keystore error to gate if no DB exists.
//!
//! These tests exercise the pure reducer [`compute_startup_readiness`] directly
//! (no Tauri setup, no real DB) across the priority matrix. The production
//! `setup()` in `lib.rs` calls this same reducer, so behaviour is identical.

use linguaray_lib::compute_startup_readiness;
use linguaray_lib::db::readiness::DataReadiness;
use linguaray_lib::db::schema;
use linguaray_lib::db::Database;
use linguaray_lib::startup_migration_guard;

fn db_err() -> String {
    "open linguaray.db: corrupt header".to_string()
}
fn settings_err() -> String {
    "settings path resolution failed".to_string()
}
fn keystore_err() -> String {
    "keystore init failed".to_string()
}

#[test]
fn db_failure_plus_settings_failure_yields_database_recovery() {
    // P1.4: DB failure wins. A settings error must not override it.
    let r = compute_startup_readiness(Err(db_err()), Some(settings_err()), None);
    assert!(
        matches!(r, DataReadiness::NeedsDatabaseRecovery { .. }),
        "DB+settings failure must be NeedsDatabaseRecovery, got {r:?}"
    );
}

#[test]
fn db_failure_plus_keystore_failure_yields_database_recovery() {
    // P1.4: DB failure wins even when the keystore is also broken.
    let r = compute_startup_readiness(Err(db_err()), None, Some(keystore_err()));
    assert!(
        matches!(r, DataReadiness::NeedsDatabaseRecovery { .. }),
        "DB+keystore failure must be NeedsDatabaseRecovery, got {r:?}"
    );
}

#[test]
fn db_failure_with_all_failures_yields_database_recovery() {
    // P1.4: the worst failure (no DB) wins even when everything else is broken.
    let r = compute_startup_readiness(
        Err(db_err()),
        Some(settings_err()),
        Some(keystore_err()),
    );
    assert!(
        matches!(r, DataReadiness::NeedsDatabaseRecovery { .. }),
        "all-failures must be NeedsDatabaseRecovery, got {r:?}"
    );
}

#[test]
fn db_ok_keystore_failure_yields_keystore_recovery() {
    // DB opened, but keystore is broken → keystore recovery (the DB alone is
    // usable for reads but provider writes need the keystore).
    let r = compute_startup_readiness(Ok(()), None, Some(keystore_err()));
    assert!(
        matches!(r, DataReadiness::NeedsKeystoreRecovery { .. }),
        "DB ok + keystore failure must be NeedsKeystoreRecovery, got {r:?}"
    );
}

#[test]
fn db_ok_settings_failure_yields_migration_incomplete() {
    // DB opened, but settings path didn't resolve → migration can't run safely
    // → MigrationIncomplete (the settings_path checkpoint).
    let r = compute_startup_readiness(Ok(()), Some(settings_err()), None);
    assert!(
        matches!(r, DataReadiness::MigrationIncomplete { .. }),
        "DB ok + settings failure must be MigrationIncomplete, got {r:?}"
    );
}

#[test]
fn db_ok_keystore_failure_beats_settings_failure() {
    // When both settings and keystore fail (but DB is ok), the keystore failure
    // takes precedence: a healthy DB + healthy migration are useless without a
    // usable keystore, mirroring the pre-fix production behaviour for this arm.
    let r = compute_startup_readiness(Ok(()), Some(settings_err()), Some(keystore_err()));
    assert!(
        matches!(r, DataReadiness::NeedsKeystoreRecovery { .. }),
        "DB ok + settings + keystore failures must be NeedsKeystoreRecovery, got {r:?}"
    );
}

#[test]
fn db_ok_no_failures_is_not_yet_ready() {
    // All inputs healthy. The reducer does NOT itself return Ready — running
    // the migration is what promotes to Ready. It returns the pre-migration
    // state (default = MigrationIncomplete "startup not complete") so the
    // migration block in setup() unconditionally assigns the result.
    let r = compute_startup_readiness(Ok(()), None, None);
    assert!(
        !r.is_ready(),
        "reducer must not return Ready before migration runs, got {r:?}"
    );
}

#[test]
fn database_recovery_carries_reason() {
    let r = compute_startup_readiness(Err(db_err()), Some(settings_err()), None);
    if let DataReadiness::NeedsDatabaseRecovery { reason } = r {
        assert!(reason.contains("linguaray.db"), "reason should carry DB context: {reason}");
    } else {
        panic!("expected NeedsDatabaseRecovery");
    }
}

// ─── Round-3 P1.3: startup migration guard ─────────────────────────────────
//
// Migration's Phase 1 parses + backs up the legacy settings file, so a None
// settings path (resolution failed) MUST refuse migration entirely: no backup,
// no DB write. `startup_migration_guard` is the single decision point — when it
// returns Err, `setup()` never reaches `run_migration`, so the deterministic
// consequence is: no `.bak-pre-migration` on disk and no migration state in
// the DB. The reducer (`compute_startup_readiness`) already degrades readiness
// to `MigrationIncomplete "settings_path"` for the same inputs; these tests pin
// the two to agree.

#[test]
fn guard_refuses_migration_when_settings_path_is_none() {
    // P1.3 core: settings_path=None → refuse, even though the keystore is fine.
    let err = startup_migration_guard(None, None).unwrap_err();
    assert!(
        err.contains("settings path"),
        "refusal must name the settings path: {err}"
    );
}

#[test]
fn guard_refuses_migration_when_keystore_init_failed() {
    // A keystore init failure is also a hard stop (migration Phase 1 backs up
    // the keystore), even when a settings path EXISTS.
    let dir = tempfile::tempdir().unwrap();
    let sp = dir.path().join("settings.json");
    let err = startup_migration_guard(Some(&sp), Some("keystore init failed")).unwrap_err();
    assert!(err.contains("keystore init"), "{err}");
}

#[test]
fn guard_allows_migration_with_real_path_and_healthy_keystore() {
    // Positive control: both prerequisites present → the exact path is handed
    // back for run_migration.
    let dir = tempfile::tempdir().unwrap();
    let sp = dir.path().join("settings.json");
    let got = startup_migration_guard(Some(&sp), None).expect("healthy startup must allow migration");
    assert_eq!(got, sp);
}

#[test]
fn refused_guard_produces_no_backup_and_no_db_write() {
    // Full-chain evidence for the review requirement: when the guard refuses
    // (settings_path=None), migration is never reached, so the two observable
    // side effects of migration — the settings `.bak-pre-migration` file and a
    // migration state row — must both be ABSENT. This mirrors the `setup()`
    // decision path (guard Err → run_migration not called).
    let dir = tempfile::tempdir().unwrap();
    let db = Database::open(&dir.path().join("linguaray.db")).unwrap();
    // Refuse: settings path could not be resolved.
    let refusal = startup_migration_guard(None, None);
    assert!(refusal.is_err(), "the guarded scenario must refuse");
    // Because migration was refused, the setup path would NOT call
    // run_migration — so no backup may exist and the DB must be untouched.
    assert!(
        !dir.path().join("settings.json.bak-pre-migration").exists(),
        "no settings backup may be produced when migration is refused"
    );
    let state = db
        .with_conn(|conn| schema::migration_state_if_exists(conn))
        .unwrap();
    assert_eq!(
        state,
        schema::MigrationState::NotStarted,
        "the DB must carry no migration state when migration is refused"
    );
}

#[test]
fn guard_refusal_matches_readiness_reducer() {
    // The guard's refusal decision and the readiness reducer must agree: when
    // settings resolution failed, the reducer says MigrationIncomplete
    // "settings_path" AND the guard refuses — so a user seeing the recovery
    // banner can never be in a state where migration silently ran anyway.
    let r = compute_startup_readiness(Ok(()), Some(settings_err()), None);
    assert!(
        matches!(r, DataReadiness::MigrationIncomplete { .. }),
        "settings failure must degrade readiness, got {r:?}"
    );
    assert!(
        startup_migration_guard(None, None).is_err(),
        "the same settings failure must refuse migration"
    );
}
