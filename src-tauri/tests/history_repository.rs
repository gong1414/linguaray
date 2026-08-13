//! R3b A2: history privacy preferences, retention cleanup and Clear All.

use linguaray_lib::db::{history as db_history, schema, Database, DbError};
use linguaray_lib::keystore::Keystore;
use tempfile::TempDir;

struct Harness {
    _dir: TempDir,
    db: Database,
    keystore: Keystore,
}

impl Harness {
    fn new() -> Self {
        let dir = tempfile::tempdir().unwrap();
        let db = Database::open(&dir.path().join("history.db")).unwrap();
        db.with_conn(|conn| {
            let tx = conn.transaction()?;
            schema::create_all_tables(&tx)?;
            schema::seed_singletons(&tx)?;
            tx.commit()?;
            Ok(())
        })
        .unwrap();
        let keystore = Keystore::new(dir.path().join("keystore")).unwrap();
        Self {
            _dir: dir,
            db,
            keystore,
        }
    }

    fn insert_session(&self, uuid: &str, timestamp: i64, favorite: bool) {
        self.db
            .with_conn(|conn| {
                conn.execute(
                    "INSERT INTO history_sessions
                     (session_uuid, timestamp, trigger_source, target_language, is_favorite,
                      source_text_encrypted, source_text_nonce, crypto_version)
                     VALUES (?1, ?2, 'input', 'zh', ?3, X'AABB', X'000102030405060708090A0B', 1)",
                    rusqlite::params![uuid, timestamp, favorite as i64],
                )?;
                conn.execute(
                    "INSERT INTO history_results
                     (result_uuid, session_uuid, provider_uuid, provider_name_snapshot,
                      engine_id, elapsed_ms, outcome_tag, result_text_encrypted,
                      result_text_nonce, crypto_version)
                     VALUES (?1, ?2, 'provider-1', 'Provider', 'engine', 12, 'success',
                             X'CCDD', X'000102030405060708090A0B', 1)",
                    rusqlite::params![format!("result-{uuid}"), uuid],
                )?;
                Ok(())
            })
            .unwrap();
    }

    fn counts(&self) -> (i64, i64) {
        self.db
            .with_conn(|conn| {
                let sessions =
                    conn.query_row("SELECT COUNT(*) FROM history_sessions", [], |row| {
                        row.get(0)
                    })?;
                let results =
                    conn.query_row("SELECT COUNT(*) FROM history_results", [], |row| row.get(0))?;
                Ok((sessions, results))
            })
            .unwrap()
    }
}

#[test]
fn fresh_history_privacy_defaults_to_disabled_and_30_days() {
    let h = Harness::new();
    let status = h.db.with_conn(db_history::privacy_status).unwrap();
    assert!(!status.enabled);
    assert_eq!(status.retention_days, 30);
}

#[test]
fn invalid_retention_is_rejected_without_changing_the_value() {
    let h = Harness::new();
    assert!(h
        .db
        .with_conn(|conn| db_history::set_retention(conn, 31))
        .is_err());
    assert_eq!(
        h.db.with_conn(db_history::privacy_status)
            .unwrap()
            .retention_days,
        30
    );
    h.db.with_conn(|conn| db_history::set_retention(conn, 90))
        .unwrap();
    assert_eq!(
        h.db.with_conn(db_history::privacy_status)
            .unwrap()
            .retention_days,
        90
    );
}

#[test]
fn enable_creates_key_before_db_consent_and_is_idempotent() {
    let h = Harness::new();
    db_history::set_enabled(&h.db, &h.keystore, true).unwrap();
    let first = h.keystore.get_history_key().unwrap().unwrap();
    assert!(h.db.with_conn(db_history::privacy_status).unwrap().enabled);

    db_history::set_enabled(&h.db, &h.keystore, true).unwrap();
    assert_eq!(h.keystore.get_history_key().unwrap(), Some(first));
}

#[test]
fn corrupt_keystore_refuses_enable_and_leaves_db_disabled() {
    let h = Harness::new();
    std::fs::write(h._dir.path().join("keystore/keystore.json"), b"not-json").unwrap();

    assert!(db_history::set_enabled(&h.db, &h.keystore, true).is_err());
    assert!(!h.db.with_conn(db_history::privacy_status).unwrap().enabled);
}

#[test]
fn disable_preserves_existing_records_and_history_key() {
    let h = Harness::new();
    db_history::set_enabled(&h.db, &h.keystore, true).unwrap();
    let key = h.keystore.get_history_key().unwrap();
    h.insert_session("keep", 100, false);

    db_history::set_enabled(&h.db, &h.keystore, false).unwrap();
    assert!(!h.db.with_conn(db_history::privacy_status).unwrap().enabled);
    assert_eq!(h.counts(), (1, 1));
    assert_eq!(h.keystore.get_history_key().unwrap(), key);
}

#[test]
fn retention_deletes_strictly_before_cutoff_and_never_favorites() {
    let h = Harness::new();
    const DAY: i64 = 86_400;
    let now = 100 * DAY;
    let cutoff = now - 30 * DAY;
    h.insert_session("expired", cutoff - 1, false);
    h.insert_session("boundary", cutoff, false);
    h.insert_session("favorite-old", cutoff - 100, true);
    h.insert_session("recent", cutoff + 1, false);

    let removed =
        h.db.with_conn(|conn| db_history::cleanup_expired(conn, now))
            .unwrap();
    assert_eq!(removed, 1);
    assert_eq!(h.counts(), (3, 3));
}

#[test]
fn clear_all_cascades_results_but_keeps_consent_retention_and_key() {
    let h = Harness::new();
    db_history::set_enabled(&h.db, &h.keystore, true).unwrap();
    h.db.with_conn(|conn| db_history::set_retention(conn, 90))
        .unwrap();
    let key = h.keystore.get_history_key().unwrap();
    h.insert_session("one", 1, false);
    h.insert_session("two", 2, true);

    assert_eq!(h.db.with_conn(db_history::clear_all).unwrap(), 2);
    assert_eq!(h.counts(), (0, 0));
    assert_eq!(
        h.db.with_conn(db_history::privacy_status).unwrap(),
        db_history::HistoryPrivacyStatus {
            enabled: true,
            retention_days: 90,
        }
    );
    assert_eq!(h.keystore.get_history_key().unwrap(), key);
}

#[test]
fn clear_all_rolls_back_when_any_delete_fails() {
    let h = Harness::new();
    h.insert_session("protected", 1, false);
    h.insert_session("other", 2, false);
    h.db.with_conn(|conn| {
        conn.execute_batch(
            "CREATE TRIGGER reject_protected BEFORE DELETE ON history_sessions
                 WHEN OLD.session_uuid='protected'
                 BEGIN SELECT RAISE(ABORT, 'injected clear failure'); END;",
        )?;
        Ok(())
    })
    .unwrap();

    assert!(h.db.with_conn(db_history::clear_all).is_err());
    assert_eq!(
        h.counts(),
        (2, 2),
        "the transaction must roll back every delete"
    );
}

#[test]
fn missing_preferences_singleton_fails_closed() {
    let h = Harness::new();
    h.db.with_conn(|conn| {
        conn.execute("DELETE FROM preferences WHERE id=1", [])?;
        Ok(())
    })
    .unwrap();
    assert!(matches!(
        h.db.with_conn(db_history::privacy_status),
        Err(DbError::NotFound(_))
    ));
}
