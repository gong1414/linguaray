//! Fixed-batch, in-memory search over encrypted history.

use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use unicode_casefold::UnicodeCaseFold;
use unicode_normalization::UnicodeNormalization;
use zeroize::Zeroizing;

use crate::db::{Database, DbError};
use crate::history::crypto::{decrypt_field, EncryptedField, HistoryCryptoError, HistoryField};
use crate::keystore::{Keystore, KeystoreError};

pub const HISTORY_SEARCH_BATCH: usize = 200;
const SECONDS_PER_DAY: i64 = 86_400;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct DecryptedHistoryResult {
    pub result_uuid: String,
    pub provider_uuid: String,
    pub provider_name: String,
    pub engine_id: String,
    pub elapsed_ms: u64,
    pub outcome_tag: String,
    pub text: Option<String>,
    pub error_kind: Option<String>,
    pub error_message: Option<String>,
    pub corrupt: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct DecryptedHistorySession {
    pub session_uuid: String,
    pub timestamp: i64,
    pub trigger_source: String,
    pub detected_language: Option<String>,
    pub target_language: String,
    pub is_favorite: bool,
    pub source_text: Option<String>,
    pub results: Vec<DecryptedHistoryResult>,
    pub corrupt: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct HistoryPage {
    pub items: Vec<DecryptedHistorySession>,
    pub next_cursor: Option<String>,
    pub scan_complete: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Cursor {
    timestamp: i64,
    session_uuid: String,
}

#[derive(Debug, Error)]
pub enum HistorySearchError {
    #[error(transparent)]
    Db(#[from] DbError),
    #[error(transparent)]
    Keystore(#[from] KeystoreError),
    #[error("history key is missing")]
    MissingKey,
    #[error("invalid history cursor")]
    Cursor,
    #[error("system clock precedes Unix epoch")]
    Clock,
}

#[derive(Debug)]
struct RawSession {
    uuid: String,
    timestamp: i64,
    trigger_source: String,
    detected_language: Option<String>,
    target_language: String,
    favorite: bool,
    source: Vec<u8>,
    source_nonce: Vec<u8>,
    crypto_version: u32,
}

#[derive(Debug)]
struct RawResult {
    uuid: String,
    provider_uuid: String,
    provider_name: String,
    engine_id: String,
    elapsed_ms: u64,
    outcome_tag: String,
    result: Option<Vec<u8>>,
    result_nonce: Option<Vec<u8>>,
    error_kind: Option<String>,
    error: Option<Vec<u8>>,
    error_nonce: Option<Vec<u8>>,
    crypto_version: u32,
}

/// Search one fixed batch. Only this batch is decrypted in memory.
pub fn search(
    db: &Database,
    keystore: &Keystore,
    query: &str,
    cursor: Option<&str>,
) -> Result<HistoryPage, HistorySearchError> {
    let key = Zeroizing::new(
        keystore
            .get_history_key()?
            .ok_or(HistorySearchError::MissingKey)?
            .0,
    );
    let cursor = cursor.map(decode_cursor).transpose()?;
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|_| HistorySearchError::Clock)?;
    let now = i64::try_from(now.as_secs()).map_err(|_| HistorySearchError::Clock)?;

    let (sessions, retention_days) = db.with_conn(|conn| {
        let retention: i64 = conn.query_row(
            "SELECT history_retention_days FROM preferences WHERE id=1",
            [],
            |row| row.get(0),
        )?;
        let sessions = read_batch(conn, now, retention, cursor.as_ref())?;
        Ok((sessions, retention))
    })?;
    let _ = retention_days;

    let scanned_last = sessions.last().map(|s| Cursor {
        timestamp: s.timestamp,
        session_uuid: s.uuid.clone(),
    });
    let scan_complete = sessions.len() < HISTORY_SEARCH_BATCH;
    let needle = normalize(query);
    let mut items = Vec::new();

    for raw in sessions {
        let mut corrupt = false;
        let source_text = decrypt_text(
            &key,
            HistoryField::SessionSource { uuid: &raw.uuid },
            raw.source,
            raw.source_nonce,
            raw.crypto_version,
        )
        .map_err(|_| ())
        .map_or_else(
            |_| {
                corrupt = true;
                None
            },
            Some,
        );
        let raw_results = db.with_conn(|conn| read_results(conn, &raw.uuid))?;
        let mut results = Vec::with_capacity(raw_results.len());
        for result in raw_results {
            let mut result_corrupt = false;
            let text = decrypt_optional(
                &key,
                HistoryField::ResultText { uuid: &result.uuid },
                result.result,
                result.result_nonce,
                result.crypto_version,
            )
            .unwrap_or_else(|_| {
                result_corrupt = true;
                None
            });
            let error_message = decrypt_optional(
                &key,
                HistoryField::ResultError { uuid: &result.uuid },
                result.error,
                result.error_nonce,
                result.crypto_version,
            )
            .unwrap_or_else(|_| {
                result_corrupt = true;
                None
            });
            corrupt |= result_corrupt;
            results.push(DecryptedHistoryResult {
                result_uuid: result.uuid,
                provider_uuid: result.provider_uuid,
                provider_name: result.provider_name,
                engine_id: result.engine_id,
                elapsed_ms: result.elapsed_ms,
                outcome_tag: result.outcome_tag,
                text,
                error_kind: result.error_kind,
                error_message,
                corrupt: result_corrupt,
            });
        }
        let matches = needle.is_empty()
            || source_text
                .as_deref()
                .is_some_and(|value| normalize(value).contains(&needle))
            || results.iter().any(|result| {
                result
                    .text
                    .as_deref()
                    .is_some_and(|value| normalize(value).contains(&needle))
            });
        // A corrupt row cannot be searched safely, but it must remain visible
        // instead of being silently omitted from a search page.
        if matches || corrupt {
            items.push(DecryptedHistorySession {
                session_uuid: raw.uuid,
                timestamp: raw.timestamp,
                trigger_source: raw.trigger_source,
                detected_language: raw.detected_language,
                target_language: raw.target_language,
                is_favorite: raw.favorite,
                source_text,
                results,
                corrupt,
            });
        }
    }

    Ok(HistoryPage {
        items,
        next_cursor: if scan_complete {
            None
        } else {
            scanned_last.as_ref().map(encode_cursor).transpose()?
        },
        scan_complete,
    })
}

fn read_batch(
    conn: &Connection,
    now: i64,
    retention_days: i64,
    cursor: Option<&Cursor>,
) -> Result<Vec<RawSession>, DbError> {
    let window = retention_days
        .checked_mul(SECONDS_PER_DAY)
        .ok_or_else(|| DbError::Integrity("history retention overflow".into()))?;
    let cutoff = now
        .checked_sub(window)
        .ok_or_else(|| DbError::Integrity("history cutoff overflow".into()))?;
    let cursor_ts = cursor.map_or(i64::MAX, |value| value.timestamp);
    let cursor_uuid = cursor.map_or("\u{10ffff}", |value| value.session_uuid.as_str());
    let mut stmt = conn.prepare(
        "SELECT session_uuid, timestamp, trigger_source, detected_language,
                target_language, is_favorite, source_text_encrypted,
                source_text_nonce, crypto_version
         FROM history_sessions
         WHERE (timestamp >= ?1 OR is_favorite=1)
           AND (timestamp < ?2 OR (timestamp = ?2 AND session_uuid < ?3))
         ORDER BY timestamp DESC, session_uuid DESC
         LIMIT 200",
    )?;
    let rows = stmt
        .query_map(params![cutoff, cursor_ts, cursor_uuid], |row| {
            Ok(RawSession {
                uuid: row.get(0)?,
                timestamp: row.get(1)?,
                trigger_source: row.get(2)?,
                detected_language: row.get(3)?,
                target_language: row.get(4)?,
                favorite: row.get::<_, i64>(5)? == 1,
                source: row.get(6)?,
                source_nonce: row.get(7)?,
                crypto_version: row.get(8)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

fn read_results(conn: &Connection, session_uuid: &str) -> Result<Vec<RawResult>, DbError> {
    let mut stmt = conn.prepare(
        "SELECT result_uuid, provider_uuid, provider_name_snapshot, engine_id,
                elapsed_ms, outcome_tag, result_text_encrypted, result_text_nonce,
                error_kind, error_message_encrypted, error_message_nonce, crypto_version
         FROM history_results WHERE session_uuid=?1 ORDER BY rowid ASC",
    )?;
    let rows = stmt
        .query_map([session_uuid], |row| {
            let elapsed: i64 = row.get(4)?;
            Ok(RawResult {
                uuid: row.get(0)?,
                provider_uuid: row.get(1)?,
                provider_name: row.get(2)?,
                engine_id: row.get(3)?,
                elapsed_ms: u64::try_from(elapsed).unwrap_or_default(),
                outcome_tag: row.get(5)?,
                result: row.get(6)?,
                result_nonce: row.get(7)?,
                error_kind: row.get(8)?,
                error: row.get(9)?,
                error_nonce: row.get(10)?,
                crypto_version: row.get(11)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

fn decrypt_optional(
    key: &[u8; 32],
    field: HistoryField<'_>,
    ciphertext: Option<Vec<u8>>,
    nonce: Option<Vec<u8>>,
    crypto_version: u32,
) -> Result<Option<String>, HistoryCryptoError> {
    match (ciphertext, nonce) {
        (None, None) => Ok(None),
        (Some(ciphertext), Some(nonce)) => {
            decrypt_text(key, field, ciphertext, nonce, crypto_version).map(Some)
        }
        _ => Err(HistoryCryptoError::Authentication),
    }
}

fn decrypt_text(
    key: &[u8; 32],
    field: HistoryField<'_>,
    ciphertext: Vec<u8>,
    nonce: Vec<u8>,
    crypto_version: u32,
) -> Result<String, HistoryCryptoError> {
    let nonce: [u8; 12] = nonce
        .try_into()
        .map_err(|_| HistoryCryptoError::Authentication)?;
    let plaintext = decrypt_field(
        key,
        &field,
        &EncryptedField {
            ciphertext,
            nonce,
            crypto_version,
        },
    )?;
    String::from_utf8(plaintext).map_err(|_| HistoryCryptoError::Authentication)
}

fn normalize(value: &str) -> String {
    // Full Unicode case folding (e.g. ß → ss), surrounded by NFKC so both
    // compatibility characters and fold expansions use stable search forms.
    value.nfkc().case_fold().nfkc().collect()
}

fn encode_cursor(cursor: &Cursor) -> Result<String, HistorySearchError> {
    let bytes = serde_json::to_vec(cursor).map_err(|_| HistorySearchError::Cursor)?;
    Ok(URL_SAFE_NO_PAD.encode(bytes))
}

fn decode_cursor(value: &str) -> Result<Cursor, HistorySearchError> {
    let bytes = URL_SAFE_NO_PAD
        .decode(value)
        .map_err(|_| HistorySearchError::Cursor)?;
    serde_json::from_slice(&bytes).map_err(|_| HistorySearchError::Cursor)
}
