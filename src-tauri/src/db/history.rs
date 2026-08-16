//! History privacy preferences, retention cleanup, and destructive clear.
//!
//! This module deliberately keeps database and keystore locks disjoint. Enabling
//! history creates/reads the history key first, releases the keystore locks, and
//! only then acquires the database mutex to persist consent.

pub use crate::history::crypto;

use rusqlite::{Connection, OptionalExtension};
use serde::Serialize;
use thiserror::Error;

use super::{Database, DbError};
use crate::keystore::{Keystore, KeystoreError};

pub const RETENTION_30_DAYS: u32 = 30;
pub const RETENTION_90_DAYS: u32 = 90;
const SECONDS_PER_DAY: i64 = 86_400;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct HistoryPrivacyStatus {
    pub enabled: bool,
    pub retention_days: u32,
}

#[derive(Debug, Error)]
pub enum HistoryServiceError {
    #[error(transparent)]
    Db(#[from] DbError),
    #[error(transparent)]
    Keystore(#[from] KeystoreError),
}

/// Read the singleton privacy preference. Missing or malformed data fails closed.
pub fn privacy_status(conn: &mut Connection) -> Result<HistoryPrivacyStatus, DbError> {
    privacy_status_inner(conn)
}

fn privacy_status_inner(conn: &Connection) -> Result<HistoryPrivacyStatus, DbError> {
    let row: Option<(i64, i64)> = conn
        .query_row(
            "SELECT history_enabled, history_retention_days FROM preferences WHERE id=1",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .optional()?;
    let (enabled, retention_days) =
        row.ok_or_else(|| DbError::NotFound("preferences singleton id=1".to_string()))?;

    let enabled = match enabled {
        0 => false,
        1 => true,
        other => {
            return Err(DbError::Integrity(format!(
                "invalid history_enabled value: {other}"
            )))
        }
    };
    let retention_days = u32::try_from(retention_days)
        .map_err(|_| DbError::Integrity("negative history_retention_days".to_string()))?;
    validate_retention(retention_days)?;

    Ok(HistoryPrivacyStatus {
        enabled,
        retention_days,
    })
}

/// Persist only the consent flag. The singleton must already exist.
pub fn set_enabled_preference(conn: &mut Connection, enabled: bool) -> Result<(), DbError> {
    set_enabled_preference_inner(conn, enabled)
}

fn set_enabled_preference_inner(conn: &Connection, enabled: bool) -> Result<(), DbError> {
    let changed = conn.execute(
        "UPDATE preferences SET history_enabled=?1 WHERE id=1",
        [i64::from(enabled)],
    )?;
    if changed != 1 {
        return Err(DbError::NotFound("preferences singleton id=1".to_string()));
    }
    Ok(())
}

/// Enable/disable history with key-first ordering and no nested DB/keystore locks.
///
/// Enabling is fail-closed: the key must be readable/creatable before consent is
/// written. Disabling changes only the consent bit and intentionally preserves
/// both encrypted rows and the history key.
pub fn set_enabled(
    db: &Database,
    keystore: &Keystore,
    enabled: bool,
) -> Result<(), HistoryServiceError> {
    if enabled {
        // This call owns and releases the keystore locks before `with_conn` below.
        // If the later DB write fails, a dormant history key is safe: consent is
        // still disabled and no future history write is authorized.
        keystore.get_or_create_history_key()?;
    }
    db.with_conn(|conn| {
        let tx = conn.transaction()?;
        set_enabled_preference_inner(&tx, enabled)?;
        tx.commit()?;
        Ok(())
    })?;
    Ok(())
}

fn validate_retention(days: u32) -> Result<(), DbError> {
    if matches!(days, RETENTION_30_DAYS | RETENTION_90_DAYS) {
        Ok(())
    } else {
        Err(DbError::Integrity(format!(
            "unsupported history retention: {days} days"
        )))
    }
}

/// Persist one of the frozen retention values (30 or 90 days).
pub fn set_retention(conn: &mut Connection, days: u32) -> Result<(), DbError> {
    validate_retention(days)?;
    let changed = conn.execute(
        "UPDATE preferences SET history_retention_days=?1 WHERE id=1",
        [i64::from(days)],
    )?;
    if changed != 1 {
        return Err(DbError::NotFound("preferences singleton id=1".to_string()));
    }
    Ok(())
}

/// Delete non-favorite sessions strictly older than the configured cutoff.
/// Result rows are removed by the schema's ON DELETE CASCADE.
pub fn cleanup_expired(conn: &mut Connection, now_timestamp: i64) -> Result<usize, DbError> {
    let tx = conn.transaction()?;
    let status = privacy_status_inner(&tx)?;
    let window = i64::from(status.retention_days)
        .checked_mul(SECONDS_PER_DAY)
        .ok_or_else(|| DbError::Integrity("history retention overflow".to_string()))?;
    let cutoff = now_timestamp
        .checked_sub(window)
        .ok_or_else(|| DbError::Integrity("history cutoff overflow".to_string()))?;
    let removed = tx.execute(
        "DELETE FROM history_sessions WHERE timestamp < ?1 AND is_favorite=0",
        [cutoff],
    )?;
    tx.commit()?;
    Ok(removed)
}

/// Run retention cleanup using the current Unix timestamp.
pub fn cleanup_expired_now(conn: &mut Connection) -> Result<usize, DbError> {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|_| DbError::Integrity("system clock precedes Unix epoch".to_string()))?;
    let now = i64::try_from(now.as_secs())
        .map_err(|_| DbError::Integrity("system clock timestamp overflow".to_string()))?;
    cleanup_expired(conn, now)
}

/// Atomically clear all history sessions and their cascaded result rows.
/// Preferences and the history key are intentionally preserved.
pub fn clear_all(conn: &mut Connection) -> Result<usize, DbError> {
    let tx = conn.transaction()?;
    let removed = clear_all_in_transaction(&tx)?;
    tx.commit()?;
    Ok(removed)
}

fn clear_all_in_transaction(conn: &Connection) -> Result<usize, DbError> {
    Ok(conn.execute("DELETE FROM history_sessions", [])?)
}

/// Toggle the favorite flag. Returns the NEW state (`true` = favorite).
pub fn toggle_favorite(conn: &mut Connection, session_uuid: &str) -> Result<bool, DbError> {
    let tx = conn.transaction()?;
    let current: Option<i64> = tx
        .query_row(
            "SELECT is_favorite FROM history_sessions WHERE session_uuid=?1",
            [session_uuid],
            |row| row.get(0),
        )
        .optional()?;
    let current = current.ok_or_else(|| {
        DbError::NotFound(format!("history session {session_uuid}"))
    })?;
    let next = i64::from(current == 0);
    tx.execute(
        "UPDATE history_sessions SET is_favorite=?1 WHERE session_uuid=?2",
        rusqlite::params![next, session_uuid],
    )?;
    tx.commit()?;
    Ok(next == 1)
}

/// Delete one session. Result rows cascade.
pub fn delete_session(conn: &mut Connection, session_uuid: &str) -> Result<(), DbError> {
    let tx = conn.transaction()?;
    let removed = tx.execute(
        "DELETE FROM history_sessions WHERE session_uuid=?1",
        [session_uuid],
    )?;
    if removed == 0 {
        return Err(DbError::NotFound(format!("history session {session_uuid}")));
    }
    tx.commit()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::{
        sync::{mpsc, Arc},
        time::Duration,
    };

    use super::*;
    use crate::db::schema;

    fn insert_session(conn: &Connection, uuid: &str) -> Result<(), DbError> {
        conn.execute(
            "INSERT INTO history_sessions
             (session_uuid, timestamp, trigger_source, target_language, is_favorite,
              source_text_encrypted, source_text_nonce, crypto_version)
             VALUES (?1, 1, 'input', 'zh', 0, X'AA', X'000102030405060708090A0B', 1)",
            [uuid],
        )?;
        Ok(())
    }

    #[test]
    fn clear_and_writer_are_serialized_by_the_shared_database_mutex() {
        let dir = tempfile::tempdir().unwrap();
        let db = Arc::new(Database::open(&dir.path().join("history.db")).unwrap());
        db.with_conn(|conn| {
            let tx = conn.transaction()?;
            schema::create_all_tables(&tx)?;
            schema::seed_singletons(&tx)?;
            insert_session(&tx, "old")?;
            tx.commit()?;
            Ok(())
        })
        .unwrap();

        let (clear_locked_tx, clear_locked_rx) = mpsc::channel();
        let (release_clear_tx, release_clear_rx) = mpsc::channel();
        let clear_db = Arc::clone(&db);
        let clear = std::thread::spawn(move || {
            clear_db
                .with_conn(|conn| {
                    let tx = conn.transaction()?;
                    clear_all_in_transaction(&tx)?;
                    clear_locked_tx.send(()).unwrap();
                    release_clear_rx.recv().unwrap();
                    tx.commit()?;
                    Ok(())
                })
                .unwrap();
        });
        clear_locked_rx.recv().unwrap();

        let (writer_done_tx, writer_done_rx) = mpsc::channel();
        let writer_db = Arc::clone(&db);
        let writer = std::thread::spawn(move || {
            writer_db
                .with_conn(|conn| insert_session(conn, "new"))
                .unwrap();
            writer_done_tx.send(()).unwrap();
        });

        assert!(writer_done_rx
            .recv_timeout(Duration::from_millis(50))
            .is_err());
        release_clear_tx.send(()).unwrap();
        clear.join().unwrap();
        writer.join().unwrap();

        let uuids = db
            .with_conn(|conn| {
                let mut stmt = conn.prepare("SELECT session_uuid FROM history_sessions")?;
                let rows = stmt
                    .query_map([], |row| row.get::<_, String>(0))?
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(rows)
            })
            .unwrap();
        assert_eq!(uuids, vec!["new"]);
    }
}
