//! db_schema tests — verify all 8 tables, singletons, pragmas, constraints.
//! (S2a verification gate — schema sub-tests.)

use linguaray_lib::db::{Database, DbError};
use linguaray_lib::db::schema;
use rusqlite::OptionalExtension;
use tempfile::tempdir;

/// Helper: open a fresh DB in a temp dir and run create_all_tables + seed_singletons.
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
    }).unwrap();
    (dir, db)
}

#[test]
fn all_eight_tables_exist() {
    let (_dir, db) = fresh_db();
    db.with_conn(|conn| {
        let tables: Vec<String> = conn
            .prepare("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name")?
            .query_map([], |r| r.get(0))?
            .collect::<Result<Vec<_>, _>>()?;
        let expected = [
            "_schema_migrations", "dict_packages", "history_results",
            "history_sessions", "preferences", "providers", "shortcuts", "vocabulary",
        ];
        for t in &expected {
            assert!(tables.iter().any(|x| x == t), "table '{}' missing", t);
        }
        Ok(())
    }).unwrap();
}

#[test]
fn singletons_exist() {
    let (_dir, db) = fresh_db();
    db.with_conn(|conn| {
        // _schema_migrations singleton
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM _schema_migrations WHERE id=1", [], |r| r.get(0)
        ).unwrap();
        assert_eq!(count, 1, "_schema_migrations singleton must exist");

        // preferences singleton
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM preferences WHERE id=1", [], |r| r.get(0)
        ).unwrap();
        assert_eq!(count, 1, "preferences singleton must exist");
        Ok(())
    }).unwrap();
}

#[test]
fn fresh_preferences_has_null_slots() {
    let (_dir, db) = fresh_db();
    db.with_conn(|conn| {
        let (primary, parallel, fallback): (Option<String>, String, Option<String>) =
            conn.query_row(
                "SELECT primary_uuid, parallel_uuids, fallback_uuid FROM preferences WHERE id=1",
                [], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
            ).unwrap();
        assert!(primary.is_none(), "fresh primary_uuid should be NULL");
        assert_eq!(parallel, "[]", "fresh parallel_uuids should be '[]'");
        assert!(fallback.is_none(), "fresh fallback_uuid should be NULL");
        Ok(())
    }).unwrap();
}

#[test]
fn create_all_tables_is_idempotent() {
    let (_dir, db) = fresh_db();
    // Run create_all_tables again — should not error.
    db.with_conn(|conn| {
        schema::create_all_tables(conn)?;
        schema::seed_singletons(conn)?;
        Ok::<(), DbError>(())
    }).unwrap();
    // Still exactly one singleton row each:
    db.with_conn(|conn| {
        let c1: i64 = conn.query_row("SELECT COUNT(*) FROM _schema_migrations", [], |r| r.get(0))?;
        let c2: i64 = conn.query_row("SELECT COUNT(*) FROM preferences", [], |r| r.get(0))?;
        assert_eq!(c1, 1);
        assert_eq!(c2, 1);
        Ok(())
    }).unwrap();
}

#[test]
fn foreign_keys_are_on() {
    let (_dir, db) = fresh_db();
    db.with_conn(|conn| {
        let fk: i64 = conn.pragma_query_value(None, "foreign_keys", |r| r.get(0))?;
        assert_eq!(fk, 1, "PRAGMA foreign_keys must be ON");
        Ok(())
    }).unwrap();
}

#[test]
fn journal_mode_is_delete() {
    let (_dir, db) = fresh_db();
    db.with_conn(|conn| {
        let mode: String = conn.pragma_query_value(None, "journal_mode", |r| r.get(0))?;
        assert_eq!(mode.to_lowercase(), "delete", "PRAGMA journal_mode must be DELETE");
        Ok(())
    }).unwrap();
}

#[test]
fn synchronous_is_full() {
    let (_dir, db) = fresh_db();
    db.with_conn(|conn| {
        let sync: i64 = conn.pragma_query_value(None, "synchronous", |r| r.get(0))?;
        assert_eq!(sync, 2, "PRAGMA synchronous must be FULL (2)");
        Ok(())
    }).unwrap();
}

#[test]
fn secret_ref_is_unique() {
    let (_dir, db) = fresh_db();
    db.with_conn(|conn| {
        conn.execute(
            "INSERT INTO providers (uuid, template_id, name, protocol, endpoint, needs_key, secret_ref)
             VALUES ('u1', 'openai', 'A', 'openai_chat', 'https://a.com', 1, 'ref1')",
            [],
        )?;
        // Duplicate secret_ref → error
        let result = conn.execute(
            "INSERT INTO providers (uuid, template_id, name, protocol, endpoint, needs_key, secret_ref)
             VALUES ('u2', 'openai', 'B', 'openai_chat', 'https://b.com', 1, 'ref1')",
            [],
        );
        assert!(result.is_err(), "duplicate secret_ref must be rejected");
        Ok(())
    }).unwrap();
}

#[test]
fn history_results_success_requires_text() {
    let (_dir, db) = fresh_db();
    db.with_conn(|conn| {
        // Insert a session first.
        conn.execute(
            "INSERT INTO history_sessions (session_uuid, timestamp, trigger_source, target_language,
             is_favorite, source_text_encrypted, source_text_nonce, crypto_version)
             VALUES ('s1', 1, 'selection', 'zh', 0, X'AABB', X'CCDD', 1)",
            [],
        )?;
        // success with NULL text → CHECK error
        let result = conn.execute(
            "INSERT INTO history_results (result_uuid, session_uuid, provider_uuid,
             provider_name_snapshot, engine_id, elapsed_ms, outcome_tag, crypto_version)
             VALUES ('r1', 's1', 'p1', 'OpenAI', 'openai', 100, 'success', 1)",
            [],
        );
        assert!(result.is_err(), "success with NULL result_text must be rejected");
        Ok(())
    }).unwrap();
}

#[test]
fn history_results_failure_with_text_rejected() {
    let (_dir, db) = fresh_db();
    db.with_conn(|conn| {
        conn.execute(
            "INSERT INTO history_sessions (session_uuid, timestamp, trigger_source, target_language,
             is_favorite, source_text_encrypted, source_text_nonce, crypto_version)
             VALUES ('s2', 1, 'selection', 'zh', 0, X'AABB', X'CCDD', 1)",
            [],
        )?;
        // failure WITH result_text → CHECK error
        let result = conn.execute(
            "INSERT INTO history_results (result_uuid, session_uuid, provider_uuid,
             provider_name_snapshot, engine_id, elapsed_ms, outcome_tag,
             result_text_encrypted, result_text_nonce, error_kind, crypto_version)
             VALUES ('r2', 's2', 'p1', 'OpenAI', 'openai', 100, 'failure',
                     X'1234', X'5678', 'Network', 1)",
            [],
        );
        assert!(result.is_err(), "failure with result_text must be rejected");
        Ok(())
    }).unwrap();
}

#[test]
fn migration_state_not_started_on_fresh_db() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("test.db");
    let db = Database::open(&db_path).unwrap();
    // Before create_all_tables:
    let state = db.with_conn(|conn| schema::migration_state_if_exists(conn)).unwrap();
    assert_eq!(state, schema::MigrationState::NotStarted);
}

#[test]
fn migration_state_incomplete_after_schema() {
    let (_dir, db) = fresh_db();
    let state = db.with_conn(|conn| schema::migration_state_if_exists(conn)).unwrap();
    assert_eq!(state, schema::MigrationState::Incomplete);
}

#[test]
fn migration_state_complete_after_flag() {
    let (_dir, db) = fresh_db();
    db.with_conn(|conn| {
        schema::set_migration_complete(conn)?;
        Ok(())
    }).unwrap();
    let state = db.with_conn(|conn| schema::migration_state_if_exists(conn)).unwrap();
    assert_eq!(state, schema::MigrationState::Complete);
}

#[test]
fn corrupt_db_header_propagates_error() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("corrupt.db");
    // Write garbage bytes (not a valid SQLite header):
    std::fs::write(&db_path, b"this is not a sqlite database file!!!").unwrap();
    // Opening should fail or the first query should fail:
    let result = Database::open(&db_path);
    // rusqlite may open the file but queries will fail. Either way, it's an error,
    // not a silent NotStarted. Let's check both cases:
    match result {
        Err(_) => { /* open failed — good */ }
        Ok(db) => {
            // Open "succeeded" but queries should fail:
            let query_result = db.with_conn(|conn| schema::migration_state_if_exists(conn));
            assert!(query_result.is_err(), "corrupt DB must produce an error, not NotStarted");
        }
    }
}

#[test]
fn invalid_migration_complete_value_is_rejected() {
    let (_dir, db) = fresh_db();
    db.with_conn(|conn| {
        // The CHECK constraint should reject an invalid value at the DB level.
        // A direct UPDATE to an out-of-range value must fail:
        let result = conn.execute(
            "UPDATE _schema_migrations SET migration_complete=5 WHERE id=1", []
        );
        assert!(result.is_err(), "CHECK constraint must reject migration_complete=5");
        Ok(())
    }).unwrap();

    // If the value somehow bypasses CHECK (e.g. external tampering of the DB file),
    // the preflight reader must still catch it. Simulate by dropping the CHECK
    // and inserting a bad value, then verify the reader returns Integrity error.
    // We do this on a separate DB with a table lacking the CHECK:
    let dir2 = tempdir().unwrap();
    let db2 = Database::open(&dir2.path().join("t2.db")).unwrap();
    db2.with_conn(|conn| {
        conn.execute_batch(
            "CREATE TABLE _schema_migrations (id INTEGER PRIMARY KEY, schema_version INTEGER, migration_complete INTEGER);
             INSERT INTO _schema_migrations (id, schema_version, migration_complete) VALUES (1, 1, 5);"
        )?;
        let result = schema::migration_state_if_exists(conn);
        match result {
            Err(DbError::Integrity(_)) => Ok(()),
            other => panic!("expected Integrity error for value=5, got {other:?}"),
        }
    }).unwrap();
}
