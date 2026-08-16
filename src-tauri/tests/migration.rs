//! S2a step 4 integration tests: the 5-phase crash-safe migration coordinator.
//!
//! Each test runs the REAL `run_migration_with_identity` against a REAL temp DB +
//! REAL temp keystore (encrypted with the injected test identity). Failpoint
//! tests:
//!   1. set `FailpointCell` to a checkpoint
//!   2. run the real coordinator → executes to the checkpoint, persists state,
//!      returns `Err(InjectedFail(point))`
//!   3. inspect the on-disk DB + keystore (separate Connection) to verify the
//!      coordinator persisted the correct intermediate state
//!   4. reset the failpoint to None, re-run → completes
//!   5. assert the final state is correct
//!
//! The keystore is driven via the `*_with_identity` test seams so tests never
//! touch the real machine identity (deterministic across hosts).

use linguaray_lib::db::migration::{
    run_migration_with_identity, Failpoint, FailpointCell, MigrationError,
};
use linguaray_lib::db::providers::Protocol;
use linguaray_lib::db::Database;
use linguaray_lib::keystore::{encrypt, store_with_identity, IdentitySource, KeystoreLoadState};
use rusqlite::Connection;
use serde_json::json;
use std::collections::HashMap;
use tempfile::tempdir;

/// Injected keystore identity — never the real machine identity.
const ID: &str = "test-machine-uuid";

// ─── Helpers ──────────────────────────────────────────────────────────────

/// Open a fresh in-memory-ish DB in a temp dir (no schema yet — the migration
/// creates it). Returns the dir (keeps it alive) and the Database.
fn fresh_db() -> (tempfile::TempDir, Database) {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("test.db");
    let db = Database::open(&db_path).unwrap();
    (dir, db)
}

/// Path to settings.json inside the harness dir.
fn settings_path(dir: &tempfile::TempDir) -> std::path::PathBuf {
    dir.path().join("settings.json")
}

/// Path to the keystore dir (sibling of settings.json). We give the keystore
/// its OWN subdir so its sidecar lock + backup file don't clutter the DB dir.
fn keystore_dir(dir: &tempfile::TempDir) -> std::path::PathBuf {
    dir.path().join("keystore")
}

/// Write a standard upgrade fixture: settings.json `{openai, zh, google}` and a
/// legacy v1 keystore holding `{openai, anthropic}`.
fn write_upgrade_fixture(dir: &tempfile::TempDir) {
    write_settings_json(&settings_path(dir), Some(("openai", "zh", Some("google"))));
    write_legacy_keystore(&keystore_dir(dir), &[("openai", "sk-a"), ("anthropic", "sk-b")]);
}

/// Write settings.json with the given defaults. `None` for `target_language` /
/// `fallback_engine` omits the key (simulating a partial settings file).
fn write_settings_json(
    path: &std::path::Path,
    body: Option<(&str, &str, Option<&str>)>,
) {
    match body {
        Some((dp, lang, fb)) => {
            let mut obj = serde_json::Map::new();
            obj.insert("default_provider".into(), json!(dp));
            obj.insert("target_language".into(), json!(lang));
            if let Some(fb) = fb {
                obj.insert("fallback_engine".into(), json!(fb));
            }
            let s = serde_json::to_string_pretty(&serde_json::Value::Object(obj)).unwrap();
            std::fs::write(path, s).unwrap();
        }
        None => {
            // Don't create the file at all (missing settings = fresh install).
        }
    }
}

/// Seed a legacy v1 keystore (flat map, no `version` field) encrypted with the
/// test identity. `store_with_identity` performs the encrypt; we pass the
/// plaintext map directly.
fn write_legacy_keystore(dir: &std::path::Path, entries: &[(&str, &str)]) {
    std::fs::create_dir_all(dir).unwrap();
    let mut map = serde_json::Map::new();
    for (k, v) in entries {
        map.insert((*k).to_string(), json!(*v));
    }
    let value = serde_json::Value::Object(map);
    store_with_identity(dir, ID, IdentitySource::MacosIoplatformuuid, &value)
        .expect("store legacy keystore");
}

/// Open a SEPARATE read-only Connection to the DB file (so a test can inspect
/// on-disk state without going through the migration's locked Database).
fn inspect_db(dir: &tempfile::TempDir) -> Connection {
    let path = dir.path().join("test.db");
    Connection::open(path).unwrap()
}

/// Does the `_schema_migrations` table exist yet?
fn schema_table_exists(conn: &Connection) -> bool {
    let n: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='_schema_migrations'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    n == 1
}

/// Count provider rows with `status='active'` (excludes tombstones).
fn count_active_providers(conn: &Connection) -> i64 {
    conn.query_row(
        "SELECT COUNT(*) FROM providers WHERE status='active'",
        [],
        |r| r.get(0),
    )
    .unwrap_or(0)
}

/// `migration_complete` flag (0 / 1). Returns -1 if the table/singleton is
/// absent.
fn migration_complete_flag(conn: &Connection) -> i64 {
    conn.query_row(
        "SELECT migration_complete FROM _schema_migrations WHERE id=1",
        [],
        |r| r.get(0),
    )
    .unwrap_or(-1)
}

/// Fetch a provider row by secret_ref (legacy bare id OR `provider/<uuid>`).
fn provider_by_secret_ref(conn: &Connection, secret_ref: &str) -> Option<(String, String, bool, bool)> {
    conn.query_row(
        "SELECT uuid, protocol, enabled, needs_key FROM providers WHERE secret_ref=?1",
        rusqlite::params![secret_ref],
        |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?, r.get::<_, i64>(2)? != 0, r.get::<_, i64>(3)? != 0)),
    )
    .ok()
}

/// `preferences.target_language`.
fn pref_target_language(conn: &Connection) -> String {
    conn.query_row(
        "SELECT target_language FROM preferences WHERE id=1",
        [],
        |r| r.get(0),
    )
    .unwrap()
}

/// `preferences.primary_uuid`.
fn pref_primary(conn: &Connection) -> Option<String> {
    conn.query_row(
        "SELECT primary_uuid FROM preferences WHERE id=1",
        [],
        |r| r.get::<_, Option<String>>(0),
    )
    .unwrap_or(None)
}

/// `preferences.fallback_uuid`.
fn pref_fallback(conn: &Connection) -> Option<String> {
    conn.query_row(
        "SELECT fallback_uuid FROM preferences WHERE id=1",
        [],
        |r| r.get::<_, Option<String>>(0),
    )
    .unwrap_or(None)
}

/// Classify the on-disk keystore via the test identity.
fn ks_state(dir: &tempfile::TempDir) -> KeystoreLoadState {
    linguaray_lib::keystore::load_state_with_identity(&keystore_dir(dir), ID)
}

/// Run the migration with the given failpoint (test identity).
fn run(dir: &tempfile::TempDir, db: &Database, fp: &FailpointCell) -> Result<(), MigrationError> {
    run_migration_with_identity(db, &keystore_dir(dir), &settings_path(dir), fp, ID)
}

// ─── M1: Fresh install ────────────────────────────────────────────────────

#[test]
fn m1_fresh_install_empty_everything() {
    // Empty dir, no settings, no keystore. Migration creates the schema, seeds
    // singletons, finds no candidates, completes. Nothing was backed up (no
    // keystore; no settings file).
    let (dir, db) = fresh_db();
    let fp = FailpointCell::none();

    run(&dir, &db, &fp).expect("fresh install should complete");

    let conn = inspect_db(&dir);
    assert!(schema_table_exists(&conn), "schema created");
    assert_eq!(migration_complete_flag(&conn), 1, "marked complete");
    assert_eq!(count_active_providers(&conn), 0, "no candidates on fresh install");
    // target_language stays at the schema default 'zh' (no settings to seed).
    assert_eq!(pref_target_language(&conn), "zh");
    assert!(pref_primary(&conn).is_none());
    assert!(pref_fallback(&conn).is_none());

    // No backup files (nothing to back up).
    assert!(!settings_path(&dir).with_extension("json.bak-pre-migration").exists());
    assert!(matches!(ks_state(&dir), KeystoreLoadState::Missing));
}

// ─── M2: Upgrade with settings + legacy keystore ──────────────────────────

#[test]
fn m2_upgrade_settings_and_legacy_keystore() {
    // settings.json `{openai, zh, google}` + legacy keystore `{openai, anthropic}`.
    // Candidates (BTreeSet order): anthropic < google < openai.
    //   - anthropic: preset → OpenaiChat-protocol? no, Anthropic; enabled, needs_key
    //   - google: traditional → GoogleTranslate; enabled, needs_key=false
    //   - openai: preset → OpenaiChat; enabled, needs_key
    // Selection: primary=openai (default_provider), fallback=google (fallback_engine).
    let (dir, db) = fresh_db();
    write_upgrade_fixture(&dir);
    let fp = FailpointCell::none();

    run(&dir, &db, &fp).expect("upgrade should complete");

    let conn = inspect_db(&dir);
    assert_eq!(migration_complete_flag(&conn), 1);
    assert_eq!(count_active_providers(&conn), 3, "anthropic + google + openai");

    // Each candidate inserted with its preset shape.
    let openai = provider_by_secret_ref(&conn, "openai").expect("openai row");
    assert_eq!(openai.1, Protocol::OpenaiChat.as_db_str());
    assert!(openai.2, "openai enabled");
    assert!(openai.3, "openai needs_key");

    let anthropic = provider_by_secret_ref(&conn, "anthropic").expect("anthropic row");
    assert_eq!(anthropic.1, Protocol::Anthropic.as_db_str());

    // google is a traditional engine keyed by provider/<uuid>.
    let google_rows: Vec<String> = conn
        .prepare("SELECT protocol FROM providers WHERE template_id='google'")
        .unwrap()
        .query_map([], |r| r.get::<_, String>(0))
        .unwrap()
        .filter_map(std::result::Result::ok)
        .collect();
    assert_eq!(google_rows, vec![Protocol::GoogleTranslate.as_db_str()]);

    // Selection seeded from settings (write-guarded: openai + google are both
    // active+enabled+valid-protocol).
    assert_eq!(pref_target_language(&conn), "zh");
    let primary = pref_primary(&conn).expect("primary seeded");
    assert_eq!(primary, openai.0, "primary = openai uuid");
    let fallback = pref_fallback(&conn).expect("fallback seeded");
    assert!(
        fallback
            == conn
                .query_row(
                    "SELECT uuid FROM providers WHERE template_id='google'",
                    [],
                    |r| r.get::<_, String>(0),
                )
                .unwrap(),
        "fallback = google uuid"
    );

    // Keystore rewritten to v2.
    assert!(matches!(ks_state(&dir), KeystoreLoadState::CurrentV2(_)));
    // Settings + keystore backups exist.
    assert!(linguaray_lib::db::migration_settings_bak_path(
        &settings_path(&dir)
    )
    .exists());
    assert!(linguaray_lib::keystore::backup_path_in(&keystore_dir(&dir)).exists());
}

// ─── M3: Idempotent re-run of M2 ──────────────────────────────────────────

#[test]
fn m3_idempotent_rerun() {
    // Running the completed migration a second time is a no-op (early return in
    // preflight). The provider set + selection must be byte-for-byte unchanged.
    let (dir, db) = fresh_db();
    write_upgrade_fixture(&dir);
    let fp = FailpointCell::none();

    run(&dir, &db, &fp).expect("first run");
    let conn = inspect_db(&dir);
    let before_count = count_active_providers(&conn);
    let before_primary = pref_primary(&conn);
    let before_fallback = pref_fallback(&conn);
    drop(conn);

    // Second run: must succeed and leave the state identical.
    run(&dir, &db, &fp).expect("second run");

    let conn = inspect_db(&dir);
    assert_eq!(count_active_providers(&conn), before_count);
    assert_eq!(pref_primary(&conn), before_primary);
    assert_eq!(pref_fallback(&conn), before_fallback);
    assert_eq!(migration_complete_flag(&conn), 1);
}

// ─── M4: AfterBackup failpoint ────────────────────────────────────────────

#[test]
fn m4_failpoint_after_backup() {
    // Both backups exist, but NO tables in the DB (schema phase hasn't run yet).
    let (dir, db) = fresh_db();
    write_upgrade_fixture(&dir);
    let fp = FailpointCell::none();
    fp.set(Failpoint::AfterBackup);

    let err = run(&dir, &db, &fp).unwrap_err();
    assert!(matches!(err, MigrationError::InjectedFail(Failpoint::AfterBackup)));

    let conn = inspect_db(&dir);
    assert!(!schema_table_exists(&conn), "no schema yet — backup is the first action");

    // Both backups exist.
    assert!(linguaray_lib::db::migration_settings_bak_path(
        &settings_path(&dir)
    )
    .exists());
    assert!(linguaray_lib::keystore::backup_path_in(&keystore_dir(&dir)).exists());

    // Re-run with fp=None → completes.
    fp.set(Failpoint::None);
    run(&dir, &db, &fp).expect("rerun completes");
    let conn = inspect_db(&dir);
    assert_eq!(migration_complete_flag(&conn), 1);
}

// ─── M5: AfterSchema failpoint ────────────────────────────────────────────

#[test]
fn m5_failpoint_after_schema() {
    // Tables exist (schema created), but NO preferences seeded from settings yet
    // (target_language still 'zh' schema default — Phase 2b hasn't run).
    let (dir, db) = fresh_db();
    write_upgrade_fixture(&dir);
    let fp = FailpointCell::none();
    fp.set(Failpoint::AfterSchema);

    let err = run(&dir, &db, &fp).unwrap_err();
    assert!(matches!(err, MigrationError::InjectedFail(Failpoint::AfterSchema)));

    let conn = inspect_db(&dir);
    assert!(schema_table_exists(&conn), "schema created");
    assert_eq!(pref_target_language(&conn), "zh", "preferences not seeded yet");
    assert_eq!(count_active_providers(&conn), 0, "no profiles yet");
    assert_eq!(migration_complete_flag(&conn), 0, "not complete");

    // Re-run → completes, preferences now seeded with 'zh' from settings (same).
    fp.set(Failpoint::None);
    run(&dir, &db, &fp).expect("rerun completes");
    let conn = inspect_db(&dir);
    assert_eq!(migration_complete_flag(&conn), 1);
    assert_eq!(count_active_providers(&conn), 3);
}

// ─── M6: AfterPreferences failpoint ───────────────────────────────────────

#[test]
fn m6_failpoint_after_preferences() {
    // Tables + preferences seeded, but NO profiles yet (Phase 3 hasn't run).
    let (dir, db) = fresh_db();
    write_upgrade_fixture(&dir);
    let fp = FailpointCell::none();
    fp.set(Failpoint::AfterPreferences);

    let err = run(&dir, &db, &fp).unwrap_err();
    assert!(matches!(err, MigrationError::InjectedFail(Failpoint::AfterPreferences)));

    let conn = inspect_db(&dir);
    assert!(schema_table_exists(&conn));
    assert_eq!(pref_target_language(&conn), "zh", "preferences seeded");
    assert_eq!(count_active_providers(&conn), 0, "no profiles yet");

    fp.set(Failpoint::None);
    run(&dir, &db, &fp).expect("rerun completes");
    let conn = inspect_db(&dir);
    assert_eq!(count_active_providers(&conn), 3);
    assert_eq!(migration_complete_flag(&conn), 1);
}

// ─── M7: AfterProfileInsert("anthropic") ──────────────────────────────────

#[test]
fn m7_failpoint_after_profile_insert_anthropic() {
    // BTreeSet order: anthropic < google < openai. Only anthropic inserted;
    // google + openai NOT yet. Keystore still v1 (Phase 4 hasn't run).
    let (dir, db) = fresh_db();
    write_upgrade_fixture(&dir);
    let fp = FailpointCell::none();
    fp.set(Failpoint::AfterProfileInsert("anthropic".to_string()));

    let err = run(&dir, &db, &fp).unwrap_err();
    assert!(matches!(
        err,
        MigrationError::InjectedFail(Failpoint::AfterProfileInsert(ref s)) if s == "anthropic"
    ));

    let conn = inspect_db(&dir);
    assert_eq!(count_active_providers(&conn), 1, "only anthropic inserted");
    assert!(provider_by_secret_ref(&conn, "anthropic").is_some());
    assert!(
        provider_by_secret_ref(&conn, "openai").is_none(),
        "openai not yet inserted (comes after anthropic alphabetically)"
    );
    // No google traditional row yet either.
    let google_n: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM providers WHERE template_id='google'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(google_n, 0);

    // Keystore still v1 (Phase 4 hasn't run).
    assert!(matches!(ks_state(&dir), KeystoreLoadState::LegacyV1(_)));

    fp.set(Failpoint::None);
    run(&dir, &db, &fp).expect("rerun completes");
    let conn = inspect_db(&dir);
    assert_eq!(count_active_providers(&conn), 3);
    assert!(matches!(ks_state(&dir), KeystoreLoadState::CurrentV2(_)));
}

// ─── M8: AfterProfiles ───────────────────────────────────────────────────

#[test]
fn m8_failpoint_after_profiles() {
    // All profiles in DB, keystore still v1 (Phase 4 hasn't run).
    let (dir, db) = fresh_db();
    write_upgrade_fixture(&dir);
    let fp = FailpointCell::none();
    fp.set(Failpoint::AfterProfiles);

    let err = run(&dir, &db, &fp).unwrap_err();
    assert!(matches!(err, MigrationError::InjectedFail(Failpoint::AfterProfiles)));

    let conn = inspect_db(&dir);
    assert_eq!(count_active_providers(&conn), 3, "all profiles inserted");
    assert_eq!(migration_complete_flag(&conn), 0, "not complete");
    assert!(
        matches!(ks_state(&dir), KeystoreLoadState::LegacyV1(_)),
        "keystore still v1 — Phase 4 hasn't run"
    );

    fp.set(Failpoint::None);
    run(&dir, &db, &fp).expect("rerun completes");
    assert!(matches!(ks_state(&dir), KeystoreLoadState::CurrentV2(_)));
}

// ─── M9: AfterKeystoreRewrite ─────────────────────────────────────────────

#[test]
fn m9_failpoint_after_keystore_rewrite() {
    // Profiles in DB, keystore now v2, but complete=0 (Phase 5 hasn't committed).
    let (dir, db) = fresh_db();
    write_upgrade_fixture(&dir);
    let fp = FailpointCell::none();
    fp.set(Failpoint::AfterKeystoreRewrite);

    let err = run(&dir, &db, &fp).unwrap_err();
    assert!(matches!(err, MigrationError::InjectedFail(Failpoint::AfterKeystoreRewrite)));

    let conn = inspect_db(&dir);
    assert_eq!(count_active_providers(&conn), 3);
    assert_eq!(migration_complete_flag(&conn), 0, "not yet complete");
    assert!(
        matches!(ks_state(&dir), KeystoreLoadState::CurrentV2(_)),
        "keystore rewritten to v2"
    );

    fp.set(Failpoint::None);
    run(&dir, &db, &fp).expect("rerun completes");
    let conn = inspect_db(&dir);
    assert_eq!(migration_complete_flag(&conn), 1);
}

// ─── M10: AfterCompleteCommit ─────────────────────────────────────────────

#[test]
fn m10_failpoint_after_complete_commit() {
    // complete=1 committed. Re-running is a no-op (preflight early return).
    let (dir, db) = fresh_db();
    write_upgrade_fixture(&dir);
    let fp = FailpointCell::none();
    fp.set(Failpoint::AfterCompleteCommit);

    let err = run(&dir, &db, &fp).unwrap_err();
    assert!(matches!(err, MigrationError::InjectedFail(Failpoint::AfterCompleteCommit)));

    let conn = inspect_db(&dir);
    assert_eq!(migration_complete_flag(&conn), 1, "complete committed before failpoint");

    fp.set(Failpoint::None);
    run(&dir, &db, &fp).expect("rerun is a no-op and succeeds");
}

// ─── M11: complete=1 early return ─────────────────────────────────────────

#[test]
fn m11_complete_rerun_is_early_return() {
    // A second full run after completion must early-return WITHOUT re-doing any
    // phase. We assert this indirectly: mutate the keystore to a fresh legacy
    // map AFTER completion; an early-return migration must NOT rewrite it.
    let (dir, db) = fresh_db();
    write_upgrade_fixture(&dir);
    let fp = FailpointCell::none();
    run(&dir, &db, &fp).expect("first run completes");

    // Snapshot the keystore bytes; an early-return rerun must leave them intact.
    let ks_path = keystore_dir(&dir).join("keystore.json");
    let bytes_before = std::fs::read(&ks_path).unwrap();

    run(&dir, &db, &fp).expect("rerun succeeds (early return)");

    let bytes_after = std::fs::read(&ks_path).unwrap();
    assert_eq!(bytes_before, bytes_after, "early-return rerun must not touch the keystore");
}

// ─── M12: existing .bak → backup no-op ────────────────────────────────────

#[test]
fn m12_existing_settings_bak_is_not_overwritten() {
    // A pre-existing settings.json.bak-pre-migration must NOT be overwritten
    // (create-new semantics). The original settings file is also preserved.
    let (dir, db) = fresh_db();
    write_upgrade_fixture(&dir);

    // Pre-create the backup with VALID JSON sentinel content. The backup_settings
    // validator parses the existing backup as JSON (fail-closed: a corrupt
    // existing backup is rejected, not silently trusted), so the sentinel must
    // be a parseable JSON value to be accepted as authoritative.
    let bak = linguaray_lib::db::migration_settings_bak_path(&settings_path(&dir));
    std::fs::write(&bak, b"{\"pre-existing\":\"sentinel\"}").unwrap();
    let sentinel_before = std::fs::read(&bak).unwrap();

    // Capture original settings content too.
    let settings_before = std::fs::read(settings_path(&dir)).unwrap();

    let fp = FailpointCell::none();
    run(&dir, &db, &fp).expect("completes");

    // Sentinel backup untouched; original settings file untouched.
    assert_eq!(std::fs::read(&bak).unwrap(), sentinel_before, "prior backup not overwritten");
    assert_eq!(
        std::fs::read(settings_path(&dir)).unwrap(),
        settings_before,
        "original settings preserved"
    );
}

#[test]
fn m12b_existing_settings_bak_invalid_json_is_rejected() {
    // Fail-closed: a pre-existing backup that isn't valid JSON (empty /
    // truncated / corrupt) must NOT be silently accepted. backup_settings
    // surfaces a BackupFailed error rather than treating the corrupt file as
    // authoritative.
    let (dir, db) = fresh_db();
    write_upgrade_fixture(&dir);

    let bak = linguaray_lib::db::migration_settings_bak_path(&settings_path(&dir));
    std::fs::write(&bak, b"").unwrap();

    let fp = FailpointCell::none();
    let err = run(&dir, &db, &fp).unwrap_err();
    match err {
        MigrationError::BackupFailed(msg) => {
            assert!(
                msg.contains("not valid JSON") || msg.contains("existing backup"),
                "error must mention the invalid existing backup: {msg}"
            );
        }
        other => panic!("expected BackupFailed for corrupt existing backup, got {other:?}"),
    }
    // The corrupt backup file is untouched (never overwritten).
    assert_eq!(std::fs::read(&bak).unwrap(), b"");
}

// ─── M13: settings corrupt JSON ───────────────────────────────────────────

#[test]
fn m13_settings_corrupt_json_errors() {
    // A settings.json that isn't valid JSON → SettingsCorrupt. Phase 1 fails
    // BEFORE any schema work, so no tables exist.
    let (dir, db) = fresh_db();
    write_upgrade_fixture(&dir);
    // Overwrite settings.json with garbage.
    std::fs::write(settings_path(&dir), b"{ this is not json").unwrap();

    let fp = FailpointCell::none();
    let err = run(&dir, &db, &fp).unwrap_err();
    assert!(matches!(err, MigrationError::SettingsCorrupt(_)));

    let conn = inspect_db(&dir);
    assert!(!schema_table_exists(&conn), "no schema — Phase 1 failed first");
}

// ─── M14: keystore corrupt → NeedsKeystoreRecovery ───────────────────────

#[test]
fn m14_keystore_corrupt_errors_recovery() {
    // A keystore file that isn't a valid envelope → Corrupt → backup_keystore
    // returns Err → migration surfaces NeedsKeystoreRecovery. Phase 1 fails
    // before schema work.
    let (dir, db) = fresh_db();
    write_upgrade_fixture(&dir);
    // Corrupt the keystore.
    std::fs::create_dir_all(keystore_dir(&dir)).unwrap();
    std::fs::write(keystore_dir(&dir).join("keystore.json"), b"not json at all").unwrap();

    let fp = FailpointCell::none();
    let err = run(&dir, &db, &fp).unwrap_err();
    assert!(
        matches!(err, MigrationError::NeedsKeystoreRecovery(_)),
        "expected NeedsKeystoreRecovery, got {err:?}"
    );

    let conn = inspect_db(&dir);
    assert!(!schema_table_exists(&conn), "no schema — Phase 1 failed");
}

// ─── M15: unknown legacy_id → custom_http repair profile ──────────────────

#[test]
fn m15_unknown_legacy_id_is_repair_profile() {
    // A legacy keystore key that isn't a known preset or traditional engine
    // builds a `custom_http` repair profile: enabled=false, needs_key=true,
    // endpoint empty. It must NEVER be written into primary/fallback (even if
    // it's the default_provider).
    let (dir, db) = fresh_db();
    // settings points default_provider at an unknown id; fallback at unknown too.
    write_settings_json(&settings_path(&dir), Some(("mystery-provider", "zh", Some("also-unknown"))));
    write_legacy_keystore(&keystore_dir(&dir), &[("mystery-provider", "sk-x")]);

    let fp = FailpointCell::none();
    run(&dir, &db, &fp).expect("completes");

    let conn = inspect_db(&dir);
    assert_eq!(migration_complete_flag(&conn), 1);
    let row = provider_by_secret_ref(&conn, "mystery-provider").expect("repair row exists");
    assert_eq!(row.1, Protocol::CustomHttp.as_db_str());
    assert!(!row.2, "repair profile disabled");
    assert!(row.3, "repair profile needs_key");

    // Selection NOT seeded with the repair profile (write-guard rejects it).
    assert!(pref_primary(&conn).is_none(), "repair profile never becomes primary");
    assert!(pref_fallback(&conn).is_none(), "repair profile never becomes fallback");
}

// ─── M16: Ollama no key → preset profile, needs_key=0 ─────────────────────

#[test]
fn m16_ollama_no_key_preset_needs_key_false() {
    // Ollama is a keyless preset (local). Coming from settings default_provider,
    // it builds an enabled preset profile with needs_key=false. No keystore key
    // is required for it to verify (verify_key_bearing_profiles only checks keys
    // actually present in the keystore — ollama has none, which is fine).
    let (dir, db) = fresh_db();
    write_settings_json(&settings_path(&dir), Some(("ollama", "zh", None)));
    // No keystore file at all (fresh install with ollama settings).
    let fp = FailpointCell::none();
    run(&dir, &db, &fp).expect("completes");

    let conn = inspect_db(&dir);
    assert_eq!(migration_complete_flag(&conn), 1);
    let row: (String, String, bool, bool) = conn
        .query_row(
            "SELECT uuid, protocol, enabled, needs_key FROM providers WHERE template_id='ollama'",
            [],
            |r| Ok((r.get(0)?, r.get(1)?, r.get::<_, i64>(2)? != 0, r.get::<_, i64>(3)? != 0)),
        )
        .expect("ollama row exists");
    assert_eq!(row.1, Protocol::OpenaiChat.as_db_str(), "ollama uses the OpenAI-compatible chat protocol");
    assert!(row.2, "ollama enabled");
    assert!(!row.3, "ollama needs_key=false (keyless local)");
    // primary seeded (ollama is active+enabled+valid protocol).
    assert_eq!(pref_primary(&conn).as_deref(), Some(row.0.as_str()));
}

// ─── M17: DB-loss recovery (v2 keystore, empty DB) ────────────────────────

#[test]
fn m17_db_loss_recovery_from_v2_keystore() {
    // Simulate DB-loss: a v2 keystore carrying `provider/<uuid>` keys but an
    // empty DB. The migration must build a repair profile per recovered key
    // (ProviderKey arm) and complete.
    let (dir, db) = fresh_db();
    // A v2 keystore with one provider/<uuid> key.
    let recovered_uuid = "11111111-1111-1111-1111-111111111111";
    let secret_ref = format!("provider/{recovered_uuid}");
    let mut provider_keys = HashMap::new();
    provider_keys.insert(secret_ref.clone(), "sk-recovered".to_string());
    let data = linguaray_lib::keystore::KeystoreData::new_v2(provider_keys);
    let env = encrypt(
        ID,
        IdentitySource::MacosIoplatformuuid,
        &data.to_value().unwrap(),
    )
    .unwrap();
    std::fs::create_dir_all(keystore_dir(&dir)).unwrap();
    // Write the v2 envelope directly (store_with_identity re-encrypts, but we
    // already have the envelope — write its serialized bytes to disk).
    let env_bytes = serde_json::to_vec(&env).unwrap();
    std::fs::write(keystore_dir(&dir).join("keystore.json"), env_bytes).unwrap();

    let fp = FailpointCell::none();
    run(&dir, &db, &fp).expect("completes");

    let conn = inspect_db(&dir);
    assert_eq!(migration_complete_flag(&conn), 1);
    // The recovered key built a repair profile keyed by the embedded UUID.
    let row: (String, String, bool) = conn
        .query_row(
            "SELECT uuid, protocol, enabled FROM providers WHERE secret_ref=?1",
            rusqlite::params![secret_ref],
            |r| Ok((r.get(0)?, r.get(1)?, r.get::<_, i64>(2)? != 0)),
        )
        .expect("recovered-key repair row exists");
    assert_eq!(row.0, recovered_uuid, "embedded UUID preserved");
    assert_eq!(row.1, Protocol::CustomHttp.as_db_str());
    assert!(!row.2, "recovered-key repair row is disabled");
    assert!(pref_primary(&conn).is_none(), "repair profile never becomes primary");

    // Keystore stays v2 (no rewrite needed — it was already v2).
    assert!(matches!(ks_state(&dir), KeystoreLoadState::CurrentV2(_)));
}

// ─── Bonus: parse_settings_raw + settings_bak_path direct checks ──────────

#[test]
fn parse_settings_raw_missing_file_is_none() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("nope.json");
    let s = linguaray_lib::db::migration::parse_settings_raw(&path).unwrap();
    assert!(s.is_none());
}

#[test]
fn parse_settings_raw_valid_json_parses_fields() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("settings.json");
    std::fs::write(&path, br#"{"default_provider":"openai","target_language":"ja","fallback_engine":"google"}"#).unwrap();
    let s = linguaray_lib::db::migration::parse_settings_raw(&path).unwrap().unwrap();
    assert_eq!(s.default_provider.as_deref(), Some("openai"));
    assert_eq!(s.target_language.as_deref(), Some("ja"));
    assert_eq!(s.fallback_engine.as_deref(), Some("google"));
}

#[test]
fn parse_settings_raw_corrupt_json_errors() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("settings.json");
    std::fs::write(&path, b"not json").unwrap();
    let err = linguaray_lib::db::migration::parse_settings_raw(&path).unwrap_err();
    assert!(matches!(err, MigrationError::SettingsCorrupt(_)));
}

#[test]
fn parse_settings_raw_non_object_root_errors() {
    // A JSON array / number / null is a corrupt settings root.
    let dir = tempdir().unwrap();
    let path = dir.path().join("settings.json");
    std::fs::write(&path, b"[1,2,3]").unwrap();
    let err = linguaray_lib::db::migration::parse_settings_raw(&path).unwrap_err();
    assert!(matches!(err, MigrationError::SettingsCorrupt(_)));
}

#[test]
fn failpoint_cell_after_profile_insert_value_equality() {
    // maybe_fail compares the FULL Failpoint value, so AfterProfileInsert("openai")
    // must NOT match AfterProfileInsert("anthropic").
    let cell = FailpointCell::none();
    cell.set(Failpoint::AfterProfileInsert("openai".to_string()));
    // Matches the set point.
    let err = cell.maybe_fail(Failpoint::AfterProfileInsert("openai".to_string())).unwrap_err();
    assert!(matches!(err, MigrationError::InjectedFail(Failpoint::AfterProfileInsert(ref s)) if s == "openai"));
    // Does NOT match a different id.
    cell.maybe_fail(Failpoint::AfterProfileInsert("anthropic".to_string())).expect("different id does not fire");
    // Does NOT match a different discriminant.
    cell.maybe_fail(Failpoint::AfterProfiles).expect("different discriminant does not fire");
}

#[test]
fn failpoint_cell_none_never_fires() {
    let cell = FailpointCell::none();
    cell.maybe_fail(Failpoint::AfterBackup).expect("None cell never fires");
    cell.maybe_fail(Failpoint::AfterCompleteCommit).expect("None cell never fires");
}
