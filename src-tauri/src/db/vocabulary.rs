//! Encrypted vocabulary row CRUD.

use rusqlite::{params, Connection, OptionalExtension};

use super::DbError;

pub const VOCABULARY_PAGE: usize = 50;

pub struct VocabularyRow {
    pub item_uuid: String,
    pub timestamp: i64,
    pub source_language: String,
    pub target_language: String,
    pub word_encrypted: Vec<u8>,
    pub word_nonce: Vec<u8>,
    pub definition_encrypted: Vec<u8>,
    pub definition_nonce: Vec<u8>,
    pub crypto_version: i64,
}

pub fn insert(conn: &Connection, row: &VocabularyRow) -> Result<(), DbError> {
    conn.execute(
        "INSERT INTO vocabulary
         (item_uuid, timestamp, source_language, target_language,
          word_encrypted, word_nonce, definition_encrypted, definition_nonce, crypto_version)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        params![
            row.item_uuid,
            row.timestamp,
            row.source_language,
            row.target_language,
            row.word_encrypted,
            row.word_nonce,
            row.definition_encrypted,
            row.definition_nonce,
            row.crypto_version,
        ],
    )?;
    Ok(())
}

pub fn read_page(
    conn: &Connection,
    cursor_ts: i64,
    cursor_uuid: &str,
) -> Result<Vec<VocabularyRow>, DbError> {
    let mut stmt = conn.prepare(
        "SELECT item_uuid, timestamp, source_language, target_language,
                word_encrypted, word_nonce, definition_encrypted, definition_nonce, crypto_version
         FROM vocabulary
         WHERE timestamp < ?1 OR (timestamp = ?1 AND item_uuid < ?2)
         ORDER BY timestamp DESC, item_uuid DESC
         LIMIT 50",
    )?;
    let rows = stmt
        .query_map(params![cursor_ts, cursor_uuid], |row| {
            Ok(VocabularyRow {
                item_uuid: row.get(0)?,
                timestamp: row.get(1)?,
                source_language: row.get(2)?,
                target_language: row.get(3)?,
                word_encrypted: row.get(4)?,
                word_nonce: row.get(5)?,
                definition_encrypted: row.get(6)?,
                definition_nonce: row.get(7)?,
                crypto_version: row.get(8)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

pub fn delete(conn: &Connection, item_uuid: &str) -> Result<(), DbError> {
    let n = conn.execute("DELETE FROM vocabulary WHERE item_uuid=?1", [item_uuid])?;
    if n == 0 {
        return Err(DbError::NotFound(format!("vocabulary item {item_uuid}")));
    }
    Ok(())
}

pub fn count(conn: &Connection) -> Result<i64, DbError> {
    Ok(conn.query_row("SELECT COUNT(*) FROM vocabulary", [], |r| r.get(0))?)
}

pub fn get(conn: &Connection, item_uuid: &str) -> Result<Option<VocabularyRow>, DbError> {
    conn.query_row(
        "SELECT item_uuid, timestamp, source_language, target_language,
                word_encrypted, word_nonce, definition_encrypted, definition_nonce, crypto_version
         FROM vocabulary WHERE item_uuid=?1",
        [item_uuid],
        |row| {
            Ok(VocabularyRow {
                item_uuid: row.get(0)?,
                timestamp: row.get(1)?,
                source_language: row.get(2)?,
                target_language: row.get(3)?,
                word_encrypted: row.get(4)?,
                word_nonce: row.get(5)?,
                definition_encrypted: row.get(6)?,
                definition_nonce: row.get(7)?,
                crypto_version: row.get(8)?,
            })
        },
    )
    .optional()
    .map_err(DbError::from)
}
