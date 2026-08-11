//! R2-E: optimistic-lock (`providers.version`) CAS update + v1→v2 migration.
//!
//! Covers the three behaviours the version column enables:
//! 1. A normal update bumps version 1 → 2 and returns the re-read row.
//! 2. A stale `expected_version` (someone else wrote first) yields a typed
//!    `StaleVersion` outcome carrying the row's actual version — NOT a silent
//!    overwrite and NOT a generic error.
//! 3. A missing UUID yields a typed `NotFound` outcome.
//!
//! Plus the schema migration: a v1-shaped DB (no `version` column) is upgraded
//! idempotently and every existing row lands at version 1.

use linguaray_lib::db::migration::{
    run_migration_with_identity, FailpointCell, MigrationError,
};
use linguaray_lib::db::providers::{self, ProviderPatch, UpdateOutcome};
use linguaray_lib::db::schema;
use linguaray_lib::db::{Database, DbError};
use tempfile::tempdir;

/// Open a fresh DB in a temp dir, create all tables + seed singletons. Mirrors
/// the `fresh_db` helper used across the provider test suite.
fn fresh_db() -> (tempfile::TempDir, Database) {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("test.db");
    let db = Database::open(&db_path).unwrap();
    db.with_conn(|conn| {
        let tx = conn.transaction()?;
        schema::create_all_tables(&tx)?;
        schema::seed_singletons(&tx)?;
        tx.commit()?;
        Ok(())
    })
    .unwrap();
    (dir, db)
}

/// Open a fresh DB and seed one OpenAI provider, returning the created profile.
fn fresh_with_one() -> (tempfile::TempDir, Database, linguaray_lib::db::providers::ProviderProfile) {
    let (dir, db) = fresh_db();
    let p = db
        .with_conn(|conn| {
            providers::create(conn, "openai", "OpenAI", "https://api.openai.com/v1/chat/completions", None)
        })
        .unwrap();
    (dir, db, p)
}

// ─── CAS update behaviour ─────────────────────────────────────────────────

/// Normal update: expected_version matches → Written, and the returned profile
/// has version 2 (was 1).
#[test]
fn normal_update_bumps_version_to_2() {
    let (_dir, db, p) = fresh_with_one();
    assert_eq!(p.version, 1, "freshly created provider starts at version 1");
    let patch = ProviderPatch {
        name: Some("OpenAI Renamed".into()),
        endpoint: None,
        model: None,
        enabled: None,
        sort_order: None,
        expected_version: 1,
    };
    let updated = match db
        .with_conn(|conn| providers::update(conn, &p.uuid, &patch))
        .unwrap()
    {
        UpdateOutcome::Written(p) => p,
        other => panic!("expected Written, got {other:?}"),
    };
    assert_eq!(updated.version, 2, "a successful CAS update bumps the version");
    assert_eq!(updated.name, "OpenAI Renamed");
}

/// Stale version: expected_version=1 but the row is already at 2 (a prior write
/// landed). The CAS UPDATE matches 0 rows → StaleVersion carrying actual_version.
#[test]
fn stale_version_yields_stale_outcome() {
    let (_dir, db, p) = fresh_with_one();
    // First write: bumps version 1 → 2.
    let patch1 = ProviderPatch {
        name: Some("First Write".into()),
        endpoint: None,
        model: None,
        enabled: None,
        sort_order: None,
        expected_version: 1,
    };
    let first = match db
        .with_conn(|conn| providers::update(conn, &p.uuid, &patch1))
        .unwrap()
    {
        UpdateOutcome::Written(p) => p,
        other => panic!("expected Written, got {other:?}"),
    };
    assert_eq!(first.version, 2);

    // Second write with a STALE expected_version (1, but the row is at 2):
    // must NOT overwrite — returns StaleVersion { actual_version: 2 }.
    let patch2 = ProviderPatch {
        name: Some("Stale Write".into()),
        endpoint: None,
        model: None,
        enabled: None,
        sort_order: None,
        expected_version: 1,
    };
    let outcome = db
        .with_conn(|conn| providers::update(conn, &p.uuid, &patch2))
        .unwrap();
    match outcome {
        UpdateOutcome::StaleVersion { actual_version } => {
            assert_eq!(actual_version, 2, "carries the row's real version");
        }
        other => panic!("expected StaleVersion, got {other:?}"),
    }

    // The stale attempt must NOT have clobbered the first write's name.
    let row = db.with_conn(|conn| providers::get(conn, &p.uuid)).unwrap();
    assert_eq!(row.name, "First Write", "stale write did not overwrite");
    assert_eq!(row.version, 2, "version untouched by the rejected stale write");
}

/// NotFound: a non-existent UUID yields the typed NotFound outcome (not a
/// DbError), so the command layer can map it to a structured Validation error.
#[test]
fn nonexistent_uuid_yields_not_found_outcome() {
    let (_dir, db, _p) = fresh_with_one();
    let patch = ProviderPatch {
        name: Some("Ghost".into()),
        endpoint: None,
        model: None,
        enabled: None,
        sort_order: None,
        expected_version: 1,
    };
    let outcome = db
        .with_conn(|conn| providers::update(conn, "no-such-uuid", &patch))
        .unwrap();
    assert!(matches!(outcome, UpdateOutcome::NotFound), "got {outcome:?}");
}

/// A correct expected_version after a prior write succeeds: the frontend would
/// re-read version 2, then echo it back, and the second write bumps to 3.
#[test]
fn second_write_with_correct_version_succeeds() {
    let (_dir, db, p) = fresh_with_one();
    let patch1 = ProviderPatch {
        name: Some("V2".into()),
        endpoint: None,
        model: None,
        enabled: None,
        sort_order: None,
        expected_version: 1,
    };
    let v2 = match db
        .with_conn(|conn| providers::update(conn, &p.uuid, &patch1))
        .unwrap()
    {
        UpdateOutcome::Written(p) => p,
        other => panic!("expected Written, got {other:?}"),
    };
    assert_eq!(v2.version, 2);

    // Echo back v2's version → matches, write succeeds, bumps to 3.
    let patch2 = ProviderPatch {
        name: Some("V3".into()),
        endpoint: None,
        model: None,
        enabled: None,
        sort_order: None,
        expected_version: 2,
    };
    let v3 = match db
        .with_conn(|conn| providers::update(conn, &p.uuid, &patch2))
        .unwrap()
    {
        UpdateOutcome::Written(p) => p,
        other => panic!("expected Written, got {other:?}"),
    };
    assert_eq!(v3.version, 3);
    assert_eq!(v3.name, "V3");
}

// ─── v1 → v2 schema migration ─────────────────────────────────────────────

/// Build a DB shaped like a v1 install: `providers` table WITHOUT the `version`
/// column, and `_schema_migrations.schema_version = 1`. Used to exercise the
/// migration path against a realistic pre-upgrade state.
fn fresh_v1_db() -> (tempfile::TempDir, Database) {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("v1.db");
    let db = Database::open(&db_path).unwrap();
    db.with_conn(|conn| {
        conn.execute_batch(
            "CREATE TABLE _schema_migrations (
                id INTEGER PRIMARY KEY CHECK (id = 1),
                schema_version INTEGER NOT NULL,
                migration_complete INTEGER NOT NULL DEFAULT 0,
                migration_checkpoint TEXT,
                migrated_at INTEGER
             );
             INSERT INTO _schema_migrations (id, schema_version, migration_complete)
             VALUES (1, 1, 1);
             CREATE TABLE providers (
                uuid TEXT PRIMARY KEY,
                template_id TEXT NOT NULL,
                name TEXT NOT NULL,
                protocol TEXT NOT NULL,
                endpoint TEXT NOT NULL,
                model TEXT,
                enabled INTEGER NOT NULL DEFAULT 1,
                sort_order INTEGER NOT NULL DEFAULT 0,
                is_local INTEGER NOT NULL DEFAULT 0,
                needs_key INTEGER NOT NULL DEFAULT 0,
                secret_ref TEXT NOT NULL UNIQUE,
                capabilities TEXT NOT NULL DEFAULT '{}',
                status TEXT NOT NULL DEFAULT 'active'
             );",
        )?;
        // Seed one pre-v2 row (no version column to populate).
        conn.execute(
            "INSERT INTO providers (uuid, template_id, name, protocol, endpoint, secret_ref) \
             VALUES ('legacy-uuid', 'openai', 'Legacy OpenAI', 'openai_chat', \
             'https://api.openai.com', 'openai')",
            [],
        )?;
        Ok(())
    })
    .unwrap();
    (dir, db)
}

/// Build a DB shaped like a v1 install whose `_schema_migrations` SINGLETON ROW
/// is MISSING (the table exists but `id=1` was never inserted — or was deleted),
/// AND whose `providers` table predates the `version` column. This is the exact
/// shape that exposed the Phase 2c skip bug: `seed_singletons` writes
/// `schema_version=2` in Phase 2, the old `stored < SCHEMA_VERSION` gate then
/// read `2 < 2` → false and skipped `migrate_v1_to_v2`, so the `providers` table
/// never received the column even though the app believed it was v2.
fn fresh_v1_db_missing_singleton() -> (tempfile::TempDir, Database) {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("v1_no_singleton.db");
    let db = Database::open(&db_path).unwrap();
    db.with_conn(|conn| {
        conn.execute_batch(
            "CREATE TABLE _schema_migrations (
                id INTEGER PRIMARY KEY CHECK (id = 1),
                schema_version INTEGER NOT NULL,
                migration_complete INTEGER NOT NULL DEFAULT 0,
                migration_checkpoint TEXT,
                migrated_at INTEGER
             );
             -- Deliberately do NOT insert the singleton row (id=1 is missing):
             -- this is the precondition that made the old Phase 2c gate skip.
             CREATE TABLE providers (
                uuid TEXT PRIMARY KEY,
                template_id TEXT NOT NULL,
                name TEXT NOT NULL,
                protocol TEXT NOT NULL,
                endpoint TEXT NOT NULL,
                model TEXT,
                enabled INTEGER NOT NULL DEFAULT 1,
                sort_order INTEGER NOT NULL DEFAULT 0,
                is_local INTEGER NOT NULL DEFAULT 0,
                needs_key INTEGER NOT NULL DEFAULT 0,
                secret_ref TEXT NOT NULL UNIQUE,
                capabilities TEXT NOT NULL DEFAULT '{}',
                status TEXT NOT NULL DEFAULT 'active'
             );",
        )?;
        // Seed one pre-v2 row (no version column to populate).
        conn.execute(
            "INSERT INTO providers (uuid, template_id, name, protocol, endpoint, secret_ref) \
             VALUES ('legacy-uuid', 'openai', 'Legacy OpenAI', 'openai_chat', \
             'https://api.openai.com', 'openai')",
            [],
        )?;
        Ok(())
    })
    .unwrap();
    (dir, db)
}

/// `migrate_v1_to_v2` adds the `version` column, defaults every existing row to
/// 1, and bumps `_schema_migrations.schema_version` to 2.
#[test]
fn migrate_v1_to_v2_adds_column_and_seeds_version() {
    let (_dir, db) = fresh_v1_db();
    db.with_conn(|conn| {
        let tx = conn.transaction()?;
        schema::migrate_v1_to_v2(&tx)?;
        tx.commit()?;
        Ok(())
    })
    .unwrap();

    db.with_conn(|conn| {
        // The version column exists and the pre-existing row defaulted to 1.
        let (ver, name): (i64, String) = conn.query_row(
            "SELECT version, name FROM providers WHERE uuid='legacy-uuid'",
            [],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )?;
        assert_eq!(ver, 1, "existing v1 row seeds to version 1");
        assert_eq!(name, "Legacy OpenAI");

        // schema_version recorded as 2.
        let sv: i64 = conn.query_row(
            "SELECT schema_version FROM _schema_migrations WHERE id=1",
            [],
            |r| r.get(0),
        )?;
        assert_eq!(sv, 2, "schema_version bumped to 2");
        Ok(())
    })
    .unwrap();
}

/// The migration is idempotent: running it twice (or against an already-v2 DB)
/// is a no-op — the column guard skips the ALTER and the version write is a
/// benign re-write.
#[test]
fn migrate_v1_to_v2_is_idempotent() {
    let (_dir, db) = fresh_v1_db();
    // First run adds the column.
    db.with_conn(|conn| {
        let tx = conn.transaction()?;
        schema::migrate_v1_to_v2(&tx)?;
        tx.commit()?;
        Ok(())
    })
    .unwrap();
    // Second run must not error (and must not re-ALTER).
    db.with_conn(|conn| {
        let tx = conn.transaction()?;
        schema::migrate_v1_to_v2(&tx)?;
        tx.commit()?;
        Ok(())
    })
    .unwrap();

    let count: i64 = db
        .with_conn(|conn| {
            let mut stmt = conn.prepare("PRAGMA table_info(providers)")?;
            let mut count = 0i64;
            let rows = stmt.query_map([], |r| r.get::<_, String>(1))?;
            for name in rows {
                if name? == "version" {
                    count += 1;
                }
            }
            Ok(count)
        })
        .unwrap();
    assert_eq!(count, 1, "exactly one version column after double-migration");
}

/// After migration, a CAS update works against a migrated v1 row (the row is at
/// version 1, so expected_version=1 matches and bumps to 2). This proves the
/// optimistic lock is live end-to-end on upgraded DBs.
#[test]
fn cas_update_works_after_migration() {
    let (_dir, db) = fresh_v1_db();
    db.with_conn(|conn| {
        let tx = conn.transaction()?;
        schema::migrate_v1_to_v2(&tx)?;
        tx.commit()?;
        Ok(())
    })
    .unwrap();

    let patch = ProviderPatch {
        name: Some("Migrated + Renamed".into()),
        endpoint: None,
        model: None,
        enabled: None,
        sort_order: None,
        expected_version: 1,
    };
    let updated = match db
        .with_conn(|conn| providers::update(conn, "legacy-uuid", &patch))
        .unwrap()
    {
        UpdateOutcome::Written(p) => p,
        other => panic!("expected Written, got {other:?}"),
    };
    assert_eq!(updated.version, 2);
    assert_eq!(updated.name, "Migrated + Renamed");
}

/// A fresh v2 DB (created via `create_all_tables`) already has the version
/// column — the migration guard must treat it as already-current and NOT error.
#[test]
fn migrate_on_fresh_v2_db_is_noop() {
    let (_dir, db) = fresh_db();
    db.with_conn(|conn| {
        let tx = conn.transaction()?;
        schema::migrate_v1_to_v2(&tx)?;
        tx.commit()?;
        Ok(())
    })
    .unwrap();
    // The fresh DB's seeded schema_version was already 2; the UPDATE is benign.
    let sv: i64 = db
        .with_conn(|conn| {
            Ok(conn.query_row(
                "SELECT schema_version FROM _schema_migrations WHERE id=1",
                [],
                |r| r.get(0),
            )?)
        })
        .unwrap();
    assert_eq!(sv, schema::SCHEMA_VERSION as i64);
}

/// `insert_or_ignore` on a fresh v2 DB leaves the row at version 1 (the column's
/// DEFAULT), proving create/insert paths don't need to set version explicitly.
#[test]
fn created_provider_starts_at_version_1() {
    let (_dir, db, p) = fresh_with_one();
    let row = db.with_conn(|conn| providers::get(conn, &p.uuid)).unwrap();
    assert_eq!(row.version, 1);
}

/// `ProviderPatch::expected_version` is REQUIRED: a patch JSON without it must
/// fail deserialization (deny_unknown_fields + non-Option field), so the IPC
/// layer can never accidentally bypass the optimistic lock.
#[test]
fn patch_without_expected_version_rejected() {
    let json = r#"{"name": "no-version"}"#;
    let err = serde_json::from_str::<ProviderPatch>(json);
    assert!(err.is_err(), "expected_version is required, got Ok");
    // And the inverse: a typo'd field is rejected (deny_unknown_fields).
    let err2 = serde_json::from_str::<ProviderPatch>(
        r#"{"name":"x","expected_version":1,"typo":true}"#,
    );
    assert!(err2.is_err(), "deny_unknown_fields rejects typos");
}

/// Sanity: a stale attempt surfaces actual_version even when the read path
/// would otherwise look like NotFound (the row exists, just at a higher version).
/// This guards the SELECT-after-0-rows branch in `update`.
#[test]
fn stale_version_reports_actual_not_default() {
    let (_dir, db, p) = fresh_with_one();
    // Bump to version 5 directly so the actual_version is distinguishable from
    // any accidental default.
    db.with_conn(|conn| {
        conn.execute(
            "UPDATE providers SET version=5 WHERE uuid=?1",
            rusqlite::params![p.uuid],
        )?;
        Ok(())
    })
    .unwrap();
    let patch = ProviderPatch {
        name: Some("Stale".into()),
        endpoint: None,
        model: None,
        enabled: None,
        sort_order: None,
        expected_version: 1,
    };
    match db
        .with_conn(|conn| providers::update(conn, &p.uuid, &patch))
        .unwrap()
    {
        UpdateOutcome::StaleVersion { actual_version } => {
            assert_eq!(actual_version, 5, "must report the real version, not a default");
        }
        other => panic!("expected StaleVersion, got {other:?}"),
    }
}

/// Compile-time guard: NotFound is a typed OUTCOME, never a leaked DbError. The
/// `update` function converts `get()`'s NotFound into the outcome internally so
/// the command layer can map it to a structured Validation error.
#[test]
fn update_returns_outcome_not_dberror_on_not_found() {
    let (_dir, db) = fresh_db();
    let patch = ProviderPatch {
        name: None,
        endpoint: None,
        model: None,
        enabled: None,
        sort_order: None,
        expected_version: 1,
    };
    let res = db.with_conn(|conn| providers::update(conn, "missing", &patch));
    // Must be Ok(NotFound), NOT Err(DbError::NotFound).
    assert!(
        matches!(res, Ok(UpdateOutcome::NotFound)),
        "NotFound is an outcome, not an error: got {res:?}"
    );
}

// ─── P1-2: fail-closed version column structure validation ────────────────

/// Build a v1-shaped DB whose `providers` table already carries a `version`
/// column with a NON-v2 shape. `version_decl` is the column declaration text
/// (e.g. `"version TEXT"`). The rest of the schema matches v1 so the table is
/// realistic. Used by the negative tests below.
fn fresh_v1_db_with_version_column(version_decl: &str) -> (tempfile::TempDir, Database) {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("v1_malformed.db");
    let db = Database::open(&db_path).unwrap();
    let create = format!(
        "CREATE TABLE _schema_migrations (
            id INTEGER PRIMARY KEY CHECK (id = 1),
            schema_version INTEGER NOT NULL,
            migration_complete INTEGER NOT NULL DEFAULT 0,
            migration_checkpoint TEXT,
            migrated_at INTEGER
         );
         INSERT INTO _schema_migrations (id, schema_version, migration_complete)
         VALUES (1, 1, 1);
         CREATE TABLE providers (
            uuid TEXT PRIMARY KEY,
            template_id TEXT NOT NULL,
            name TEXT NOT NULL,
            protocol TEXT NOT NULL,
            endpoint TEXT NOT NULL,
            model TEXT,
            enabled INTEGER NOT NULL DEFAULT 1,
            sort_order INTEGER NOT NULL DEFAULT 0,
            is_local INTEGER NOT NULL DEFAULT 0,
            needs_key INTEGER NOT NULL DEFAULT 0,
            secret_ref TEXT NOT NULL UNIQUE,
            capabilities TEXT NOT NULL DEFAULT '{{}}',
            status TEXT NOT NULL DEFAULT 'active',
            {version_decl}
         );"
    );
    db.with_conn(|conn| {
        conn.execute_batch(&create)?;
        conn.execute(
            "INSERT INTO providers (uuid, template_id, name, protocol, endpoint, secret_ref) \
             VALUES ('legacy-uuid', 'openai', 'Legacy OpenAI', 'openai_chat', \
             'https://api.openai.com', 'openai')",
            [],
        )?;
        Ok(())
    })
    .unwrap();
    (dir, db)
}

/// A pre-existing `version TEXT` column must NOT be silently adopted — the
/// migration fails closed with an Integrity error instead of trusting a column
/// that would mis-serialize the optimistic-lock version.
#[test]
fn migrate_v1_to_v2_rejects_text_version_column() {
    let (_dir, db) = fresh_v1_db_with_version_column("version TEXT");
    let res = db.with_conn(|conn| {
        let tx = conn.transaction()?;
        schema::migrate_v1_to_v2(&tx)?;
        tx.commit()?;
        Ok(())
    });
    assert!(
        matches!(res, Err(DbError::Integrity(_))),
        "expected Integrity error for TEXT version column, got {res:?}"
    );
}

/// A nullable `version INTEGER` column (no NOT NULL) must fail closed — a NULL
/// version would break every CAS update (the WHERE version=? match misses NULL).
#[test]
fn migrate_v1_to_v2_rejects_nullable_version_column() {
    let (_dir, db) = fresh_v1_db_with_version_column("version INTEGER");
    let res = db.with_conn(|conn| {
        let tx = conn.transaction()?;
        schema::migrate_v1_to_v2(&tx)?;
        tx.commit()?;
        Ok(())
    });
    assert!(
        matches!(res, Err(DbError::Integrity(_))),
        "expected Integrity error for nullable version column, got {res:?}"
    );
}

/// A `version` column with the wrong default (0 instead of 1) must fail closed
/// — a default of 0 would let new rows start at a version that never matches a
/// freshly-created row's expected_version=1.
#[test]
fn migrate_v1_to_v2_rejects_wrong_default_version_column() {
    let (_dir, db) =
        fresh_v1_db_with_version_column("version INTEGER NOT NULL DEFAULT 0");
    let res = db.with_conn(|conn| {
        let tx = conn.transaction()?;
        schema::migrate_v1_to_v2(&tx)?;
        tx.commit()?;
        Ok(())
    });
    assert!(
        matches!(res, Err(DbError::Integrity(_))),
        "expected Integrity error for default-0 version column, got {res:?}"
    );
}

// ─── P1-1: FULL-pipeline v1→v2 migration via run_migration ────────────────

/// Injected keystore identity for the full-pipeline test (never the real
/// machine identity — deterministic across hosts).
const PIPE_ID: &str = "pipeline-test-identity";

/// Drive the FULL migration coordinator (`run_migration_with_identity`) against
/// a genuine v1 DB (old schema WITHOUT the version column, singleton seeded at
/// schema_version=1, one pre-existing provider row). With an empty keystore +
/// no settings, the coordinator must: Phase 2 create/seed (no-op on the
/// existing tables), Phase 2c ALTER the version column + bump schema_version,
/// Phase 5 mark complete. Then every existing row lands at version 1 and the
/// optimistic lock is live (CAS works with expected_version=1).
#[test]
fn full_pipeline_v1_to_v2_migration_via_run_migration() {
    let (dir, db) = fresh_v1_db();
    let keystore_dir = dir.path().join("keystore");
    // No settings.json + no keystore dir contents (fresh install keystore-wise);
    // the existing v1 provider row stays (no candidates enumerated).
    let settings_path = dir.path().join("settings.json");
    let fp = FailpointCell::none();

    run_migration_with_identity(&db, &keystore_dir, &settings_path, &fp, PIPE_ID)
        .expect("full pipeline should complete on a v1 DB");

    db.with_conn(|conn| {
        // version column exists; the pre-existing v1 row defaulted to 1.
        let (ver, name): (i64, String) = conn.query_row(
            "SELECT version, name FROM providers WHERE uuid='legacy-uuid'",
            [],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )?;
        assert_eq!(ver, 1, "existing v1 row seeds to version 1 after migration");
        assert_eq!(name, "Legacy OpenAI");

        // schema_version recorded as 2 + migration_complete=1.
        let (sv, mc): (i64, i64) = conn.query_row(
            "SELECT schema_version, migration_complete FROM _schema_migrations WHERE id=1",
            [],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )?;
        assert_eq!(sv, 2, "schema_version bumped to 2");
        assert_eq!(mc, 1, "migration marked complete");
        Ok(())
    })
    .unwrap();

    // End-to-end: the optimistic lock is live on the migrated row. A CAS update
    // with expected_version=1 must succeed (bumps to 2); the provider read path
    // works against the migrated column.
    let row = db
        .with_conn(|conn| providers::get(conn, "legacy-uuid"))
        .unwrap();
    assert_eq!(row.version, 1);

    let patch = ProviderPatch {
        name: Some("Pipeline Renamed".into()),
        endpoint: None,
        model: None,
        enabled: None,
        sort_order: None,
        expected_version: 1,
    };
    let updated = match db
        .with_conn(|conn| providers::update(conn, "legacy-uuid", &patch))
        .unwrap()
    {
        UpdateOutcome::Written(p) => p,
        other => panic!("expected Written after migration, got {other:?}"),
    };
    assert_eq!(updated.version, 2, "CAS bumps the migrated row 1 → 2");
    assert_eq!(updated.name, "Pipeline Renamed");
}

/// Re-running the full pipeline after it completed is a no-op (the Complete
/// preflight short-circuits before any phase). The version column + schema
/// version are untouched.
#[test]
fn full_pipeline_is_idempotent_on_replay() {
    let (dir, db) = fresh_v1_db();
    let keystore_dir = dir.path().join("keystore");
    let settings_path = dir.path().join("settings.json");
    let fp = FailpointCell::none();

    run_migration_with_identity(&db, &keystore_dir, &settings_path, &fp, PIPE_ID)
        .expect("first run should complete");
    // Second run: preflight sees Complete → no phases run → Ok.
    run_migration_with_identity(&db, &keystore_dir, &settings_path, &fp, PIPE_ID)
        .expect("second run should be a no-op Complete");

    let count: i64 = db
        .with_conn(|conn| {
            let mut stmt = conn.prepare("PRAGMA table_info(providers)")?;
            let mut count = 0i64;
            let rows = stmt.query_map([], |r| r.get::<_, String>(1))?;
            for name in rows {
                if name? == "version" {
                    count += 1;
                }
            }
            Ok(count)
        })
        .unwrap();
    assert_eq!(count, 1, "exactly one version column after double pipeline run");
}

/// If the `_schema_migrations` singleton ROW is missing on an otherwise-v2 DB
/// (tables created by `create_all_tables`, so the version column already
/// exists), `seed_singletons` recreates the row and the pipeline completes
/// correctly. (On a fresh/v2-shaped DB the version column is inline, so the
/// Phase 2c skip is safe.)
#[test]
fn full_pipeline_recreates_missing_schema_migrations_row() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("missing_row.db");
    let db = Database::open(&db_path).unwrap();
    // Create the full v2 schema + seed, then DELETE the singleton row to
    // simulate a corrupted-but-recoverable state.
    db.with_conn(|conn| {
        let tx = conn.transaction()?;
        schema::create_all_tables(&tx)?;
        schema::seed_singletons(&tx)?;
        tx.commit()?;
        conn.execute("DELETE FROM _schema_migrations WHERE id=1", [])?;
        Ok(())
    })
    .unwrap();

    let keystore_dir = dir.path().join("keystore");
    let settings_path = dir.path().join("settings.json");
    let fp = FailpointCell::none();
    run_migration_with_identity(&db, &keystore_dir, &settings_path, &fp, PIPE_ID)
        .expect("pipeline should complete after recreating the singleton row");

    let (sv, mc): (i64, i64) = db
        .with_conn(|conn| {
            Ok(conn.query_row(
                "SELECT schema_version, migration_complete FROM _schema_migrations WHERE id=1",
                [],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )?)
        })
        .unwrap();
    assert_eq!(sv, schema::SCHEMA_VERSION as i64, "singleton row recreated at v2");
    assert_eq!(mc, 1, "migration marked complete");
}

/// REAL v1 bug fixture: `_schema_migrations` table exists but the singleton ROW
/// (`id=1`) is MISSING, AND `providers` has NO `version` column. Before the
/// P1-1 fix, the Phase 2c `stored < SCHEMA_VERSION` gate read `2 < 2` → false
/// (seed_singletons had just written schema_version=2) and skipped
/// `migrate_v1_to_v2`, leaving the providers table without the version column
/// while the app believed it was v2. With the unconditional Phase 2c call, the
/// full pipeline must ALTER the column in, seed every existing row to version 1,
/// and converge to a complete v2 DB where list + CAS update work.
#[test]
fn full_pipeline_v1_providers_no_version_with_missing_singleton() {
    let (dir, db) = fresh_v1_db_missing_singleton();
    let keystore_dir = dir.path().join("keystore");
    let settings_path = dir.path().join("settings.json");
    let fp = FailpointCell::none();

    run_migration_with_identity(&db, &keystore_dir, &settings_path, &fp, PIPE_ID)
        .expect("full pipeline should add the version column + complete on a missing-singleton v1 DB");

    db.with_conn(|conn| {
        // The version column now exists; the pre-existing v1 row defaulted to 1.
        let (ver, name): (i64, String) = conn.query_row(
            "SELECT version, name FROM providers WHERE uuid='legacy-uuid'",
            [],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )?;
        assert_eq!(ver, 1, "existing v1 row seeds to version 1 after migration");
        assert_eq!(name, "Legacy OpenAI");

        // schema_version recorded as 2 + migration_complete=1.
        let (sv, mc): (i64, i64) = conn.query_row(
            "SELECT schema_version, migration_complete FROM _schema_migrations WHERE id=1",
            [],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )?;
        assert_eq!(sv, 2, "schema_version bumped to 2");
        assert_eq!(mc, 1, "migration marked complete");
        Ok(())
    })
    .unwrap();

    // The optimistic lock is live end-to-end: list works against the migrated
    // column, and a CAS update with expected_version=1 succeeds (bumps to 2).
    let listed = db.with_conn(|conn| providers::list(conn)).unwrap();
    assert_eq!(listed.len(), 1, "list sees the migrated row");
    assert_eq!(listed[0].version, 1);

    let patch = ProviderPatch {
        name: Some("Pipeline Renamed".into()),
        endpoint: None,
        model: None,
        enabled: None,
        sort_order: None,
        expected_version: 1,
    };
    let updated = match db
        .with_conn(|conn| providers::update(conn, "legacy-uuid", &patch))
        .unwrap()
    {
        UpdateOutcome::Written(p) => p,
        other => panic!("expected Written after missing-singleton migration, got {other:?}"),
    };
    assert_eq!(updated.version, 2, "CAS bumps the migrated row 1 → 2");
    assert_eq!(updated.name, "Pipeline Renamed");
}

// ─── P1-2: Complete-preflight structural validation ──────────────────────

/// Build a DB whose `_schema_migrations` reports Complete (`schema_version=2`,
/// `migration_complete=1`) but whose `providers.version` column is shaped per
/// `version_decl` — or omitted entirely when `version_decl` is empty. Used by
/// the P1-2 Complete-preflight tests to prove `run_migration` fails closed
/// instead of trusting a "Complete" singleton whose tables are actually corrupt.
fn fresh_complete_db_with_version_decl(version_decl: &str) -> (tempfile::TempDir, Database) {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("complete_version.db");
    let db = Database::open(&db_path).unwrap();
    // When version_decl is empty, the providers table omits the version column
    // entirely (the trailing comma after `status` is dropped too).
    let version_clause = if version_decl.is_empty() {
        String::new()
    } else {
        format!(", {version_decl}")
    };
    let create = format!(
        "CREATE TABLE _schema_migrations (
            id INTEGER PRIMARY KEY CHECK (id = 1),
            schema_version INTEGER NOT NULL,
            migration_complete INTEGER NOT NULL DEFAULT 0,
            migration_checkpoint TEXT,
            migrated_at INTEGER
         );
         INSERT INTO _schema_migrations (id, schema_version, migration_complete)
         VALUES (1, 2, 1);
         CREATE TABLE providers (
            uuid TEXT PRIMARY KEY,
            template_id TEXT NOT NULL,
            name TEXT NOT NULL,
            protocol TEXT NOT NULL,
            endpoint TEXT NOT NULL,
            model TEXT,
            enabled INTEGER NOT NULL DEFAULT 1,
            sort_order INTEGER NOT NULL DEFAULT 0,
            is_local INTEGER NOT NULL DEFAULT 0,
            needs_key INTEGER NOT NULL DEFAULT 0,
            secret_ref TEXT NOT NULL UNIQUE,
            capabilities TEXT NOT NULL DEFAULT '{{}}',
            status TEXT NOT NULL DEFAULT 'active'{version_clause}
         );"
    );
    db.with_conn(|conn| {
        conn.execute_batch(&create)?;
        Ok(())
    })
    .unwrap();
    (dir, db)
}

/// Helper: drive the full pipeline against a Complete DB and assert it errors.
/// The Complete preflight short-circuits before Phase 1, so no keystore/settings
/// are needed — the failure happens at the structural validation step.
fn run_complete_pipeline_expects_integrity(dir: &tempfile::TempDir, db: &Database) {
    let keystore_dir = dir.path().join("keystore");
    let settings_path = dir.path().join("settings.json");
    let fp = FailpointCell::none();
    let res = run_migration_with_identity(db, &keystore_dir, &settings_path, &fp, PIPE_ID);
    match res {
        Err(MigrationError::Db(DbError::Integrity(_))) => { /* expected */ }
        other => panic!(
            "expected MigrationError::Db(Integrity) from Complete-preflight validation, got {other:?}"
        ),
    }
}

/// A "Complete" DB whose providers table is missing the `version` column must
/// fail the Complete preflight — the singleton alone is not proof the tables
/// match schema_version=2.
#[test]
fn complete_db_with_missing_version_column_fails_preflight() {
    let (dir, db) = fresh_complete_db_with_version_decl("");
    run_complete_pipeline_expects_integrity(&dir, &db);
}

/// A "Complete" DB with a `version TEXT` column (wrong type) fails the preflight.
#[test]
fn complete_db_with_text_version_column_fails_preflight() {
    let (dir, db) = fresh_complete_db_with_version_decl("version TEXT");
    run_complete_pipeline_expects_integrity(&dir, &db);
}

/// A "Complete" DB with a nullable `version INTEGER` column (no NOT NULL) fails.
#[test]
fn complete_db_with_nullable_version_column_fails_preflight() {
    let (dir, db) = fresh_complete_db_with_version_decl("version INTEGER");
    run_complete_pipeline_expects_integrity(&dir, &db);
}

/// A "Complete" DB with `version INTEGER NOT NULL DEFAULT 0` (wrong default)
/// fails the preflight.
#[test]
fn complete_db_with_wrong_default_version_column_fails_preflight() {
    let (dir, db) =
        fresh_complete_db_with_version_decl("version INTEGER NOT NULL DEFAULT 0");
    run_complete_pipeline_expects_integrity(&dir, &db);
}

/// A genuinely-correct "Complete" v2 DB (version column present + correct shape)
/// passes the preflight and the pipeline is a no-op Ok.
#[test]
fn complete_db_with_correct_version_column_passes_preflight() {
    let (dir, db) =
        fresh_complete_db_with_version_decl("version INTEGER NOT NULL DEFAULT 1");
    let keystore_dir = dir.path().join("keystore");
    let settings_path = dir.path().join("settings.json");
    let fp = FailpointCell::none();
    run_migration_with_identity(&db, &keystore_dir, &settings_path, &fp, PIPE_ID)
        .expect("correct Complete DB should pass preflight validation");
    let mc: i64 = db
        .with_conn(|conn| {
            Ok(conn.query_row(
                "SELECT migration_complete FROM _schema_migrations WHERE id=1",
                [],
                |r| r.get(0),
            )?)
        })
        .unwrap();
    assert_eq!(mc, 1, "unchanged: still complete");
}

// ─── P2-1: CAS barrier — concurrent writers on the same version via update() ─

/// Two threads sharing the same `Arc<Database>` rendezvous at a `Barrier`, then
/// both race the PRODUCTION `providers::update()` with `expected_version=1`. The
/// `Database` Mutex serializes the two read-modify-write transactions, so the
/// first writer's CAS matches (bumps 1→2 → `Written`) and the second finds the
/// row already at 2 (`StaleVersion { actual_version: 2 }`). This proves the
/// optimistic lock holds end-to-end through the production update path under
/// real thread contention — exactly one `Written`, exactly one `StaleVersion`.
#[test]
fn concurrent_same_version_only_one_wins_via_update() {
    use std::sync::Arc;
    use std::sync::Barrier;
    use std::thread;

    let (_dir, db, p) = fresh_with_one();
    let provider_uuid = p.uuid.clone();
    assert_eq!(p.version, 1, "freshly created provider starts at version 1");
    let db = Arc::new(db);
    let barrier = Arc::new(Barrier::new(2));
    let mut handles = vec![];
    for i in 0..2 {
        let db_clone = Arc::clone(&db);
        let barrier_clone = Arc::clone(&barrier);
        let uuid = provider_uuid.clone();
        let new_name = format!("Updater-{i}");
        handles.push(thread::spawn(move || {
            // Rendezvous so both writers race with the same expected_version=1.
            barrier_clone.wait();
            // Each thread drives the PRODUCTION update path (read-modify-write +
            // CAS inside one Mutex-held transaction). The first to acquire the
            // Mutex bumps 1→2 (Written); the second reads 2, its CAS with
            // expected_version=1 matches 0 rows → StaleVersion.
            db_clone.with_conn(|conn| {
                providers::update(conn, &uuid, &ProviderPatch {
                    name: Some(new_name),
                    endpoint: None,
                    model: None,
                    enabled: None,
                    sort_order: None,
                    expected_version: 1,
                })
            })
        }));
    }
    let results: Vec<Result<UpdateOutcome, DbError>> =
        handles.into_iter().map(|h| h.join().unwrap()).collect();
    // Exactly one Written (CAS matched, bumped 1→2); exactly one StaleVersion
    // (the row had moved to 2 under the Mutex before the second CAS).
    let written = results
        .iter()
        .filter(|r| matches!(r, Ok(UpdateOutcome::Written(_))))
        .count();
    let stale = results
        .iter()
        .filter(|r| matches!(r, Ok(UpdateOutcome::StaleVersion { .. })))
        .count();
    assert_eq!(written, 1, "exactly one Written; got {results:?}");
    assert_eq!(stale, 1, "exactly one StaleVersion; got {results:?}");
    // Final on-disk version is 2 (one successful bump).
    let final_version: i64 = db
        .with_conn(|conn| {
            Ok(conn.query_row(
                "SELECT version FROM providers WHERE uuid=?1",
                rusqlite::params![&provider_uuid],
                |r| r.get(0),
            )?)
        })
        .unwrap();
    assert_eq!(final_version, 2, "exactly one bump landed; got version {final_version}");
}
