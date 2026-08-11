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

use linguaray_lib::db::providers::{self, ProviderPatch, UpdateOutcome};
use linguaray_lib::db::schema;
use linguaray_lib::db::Database;
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
