//! Encrypted vocabulary service. Uses the history key but does not enable history.

use serde::Serialize;
use zeroize::Zeroizing;

use crate::db::{self, Database};
use crate::history::crypto::{
    decrypt_field, encrypt_field, EncryptedField, HistoryField, HISTORY_NONCE_LEN,
};
use crate::keystore::Keystore;

#[derive(Debug, Clone, Serialize, specta::Type)]
pub struct VocabularyItem {
    pub item_uuid: String,
    pub timestamp: i64,
    pub source_language: String,
    pub target_language: String,
    pub word: String,
    pub definition: String,
}

#[derive(Debug, Clone, Serialize, specta::Type)]
pub struct VocabularyPage {
    pub items: Vec<VocabularyItem>,
    pub next_cursor: Option<String>,
    pub scan_complete: bool,
}

fn now_ts() -> Result<i64, String> {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .map_err(|_| "system clock precedes Unix epoch".into())
}

fn nonce_from(bytes: &[u8]) -> Result<[u8; HISTORY_NONCE_LEN], String> {
    bytes
        .try_into()
        .map_err(|_| "vocabulary nonce must be 12 bytes".into())
}

fn decrypt_item(
    key: &[u8; 32],
    row: db::vocabulary::VocabularyRow,
) -> Result<VocabularyItem, String> {
    let word = decrypt_field(
        key,
        &HistoryField::VocabularyWord {
            uuid: &row.item_uuid,
        },
        &EncryptedField {
            ciphertext: row.word_encrypted,
            nonce: nonce_from(&row.word_nonce)?,
            crypto_version: row.crypto_version as u32,
        },
    )
    .map_err(|e| e.to_string())?;
    let definition = decrypt_field(
        key,
        &HistoryField::VocabularyDefinition {
            uuid: &row.item_uuid,
        },
        &EncryptedField {
            ciphertext: row.definition_encrypted,
            nonce: nonce_from(&row.definition_nonce)?,
            crypto_version: row.crypto_version as u32,
        },
    )
    .map_err(|e| e.to_string())?;
    Ok(VocabularyItem {
        item_uuid: row.item_uuid,
        timestamp: row.timestamp,
        source_language: row.source_language,
        target_language: row.target_language,
        word: String::from_utf8(word.to_vec()).map_err(|_| "word is not utf-8")?,
        definition: String::from_utf8(definition.to_vec())
            .map_err(|_| "definition is not utf-8")?,
    })
}

pub fn add_word(
    db: &Database,
    keystore: &Keystore,
    word: &str,
    definition: &str,
    source_language: &str,
    target_language: &str,
) -> Result<VocabularyItem, String> {
    let key = Zeroizing::new(
        keystore
            .get_or_create_history_key()
            .map_err(|e| e.to_string())?
            .0,
    );
    let uuid = uuid::Uuid::new_v4().to_string();
    let timestamp = now_ts()?;
    let enc_word = encrypt_field(
        &key,
        &HistoryField::VocabularyWord { uuid: &uuid },
        word.as_bytes(),
    )
    .map_err(|e| e.to_string())?;
    let enc_def = encrypt_field(
        &key,
        &HistoryField::VocabularyDefinition { uuid: &uuid },
        definition.as_bytes(),
    )
    .map_err(|e| e.to_string())?;
    let row = db::vocabulary::VocabularyRow {
        item_uuid: uuid.clone(),
        timestamp,
        source_language: source_language.to_string(),
        target_language: target_language.to_string(),
        word_encrypted: enc_word.ciphertext,
        word_nonce: enc_word.nonce.to_vec(),
        definition_encrypted: enc_def.ciphertext,
        definition_nonce: enc_def.nonce.to_vec(),
        crypto_version: enc_word.crypto_version as i64,
    };
    db.with_conn(|conn| db::vocabulary::insert(conn, &row))
        .map_err(|e| e.to_string())?;
    Ok(VocabularyItem {
        item_uuid: uuid,
        timestamp,
        source_language: source_language.to_string(),
        target_language: target_language.to_string(),
        word: word.to_string(),
        definition: definition.to_string(),
    })
}

pub fn list_words(
    db: &Database,
    keystore: &Keystore,
    cursor: Option<&str>,
) -> Result<VocabularyPage, String> {
    let key = Zeroizing::new(
        keystore
            .get_or_create_history_key()
            .map_err(|e| e.to_string())?
            .0,
    );
    let (cursor_ts, cursor_uuid) = match cursor {
        Some(raw) => {
            let (ts, uuid) = raw.split_once(':').ok_or("invalid vocabulary cursor")?;
            (
                ts.parse::<i64>().map_err(|_| "invalid vocabulary cursor")?,
                uuid.to_string(),
            )
        }
        None => (i64::MAX, "\u{10ffff}".into()),
    };
    let rows = db
        .with_conn(|conn| db::vocabulary::read_page(conn, cursor_ts, &cursor_uuid))
        .map_err(|e| e.to_string())?;
    let scan_complete = rows.len() < db::vocabulary::VOCABULARY_PAGE;
    let last = rows
        .last()
        .map(|r| format!("{}:{}", r.timestamp, r.item_uuid));
    let mut items = Vec::new();
    for row in rows {
        items.push(decrypt_item(&key, row)?);
    }
    Ok(VocabularyPage {
        next_cursor: if scan_complete { None } else { last },
        scan_complete,
        items,
    })
}

pub fn delete_word(db: &Database, item_uuid: &str) -> Result<(), String> {
    db.with_conn(|conn| db::vocabulary::delete(conn, item_uuid))
        .map_err(|e| e.to_string())
}

pub fn collect_all(db: &Database, keystore: &Keystore) -> Result<Vec<VocabularyItem>, String> {
    let mut all = Vec::new();
    let mut cursor = None;
    loop {
        let page = list_words(db, keystore, cursor.as_deref())?;
        all.extend(page.items);
        if page.scan_complete {
            break;
        }
        cursor = page.next_cursor;
    }
    Ok(all)
}

pub fn export_file(
    db: &Database,
    keystore: &Keystore,
    path: &str,
    format: &str,
) -> Result<String, String> {
    let all = collect_all(db, keystore)?;
    let content = match format {
        "csv" => {
            let mut out = String::from("word,definition,source_language,target_language\n");
            for i in &all {
                out.push_str(&format!(
                    "\"{}\",\"{}\",{},{}\n",
                    i.word.replace('"', "\"\""),
                    i.definition.replace('"', "\"\""),
                    i.source_language,
                    i.target_language
                ));
            }
            out
        }
        "json" => serde_json::to_string_pretty(&all).map_err(|e| e.to_string())?,
        _ => return Err("invalid format".into()),
    };
    std::fs::write(path, content).map_err(|e| e.to_string())?;
    Ok(path.to_string())
}

const ANKI_URL: &str = "http://127.0.0.1:8765";
const ANKI_TIMEOUT_SECS: u64 = 10;

#[derive(Debug)]
pub enum AnkiError {
    Request(String),
    Response(String),
}

impl std::fmt::Display for AnkiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Request(s) => write!(f, "AnkiConnect request failed: {s}"),
            Self::Response(s) => write!(f, "AnkiConnect returned error: {s}"),
        }
    }
}

impl std::error::Error for AnkiError {}

/// POST decrypted items to AnkiConnect. Never writes a plaintext file.
pub async fn export_anki_from_items(
    items: &[VocabularyItem],
    deck_name: &str,
) -> Result<(), AnkiError> {
    export_anki_from_items_url(items, deck_name, ANKI_URL).await
}

/// Testable variant that accepts the AnkiConnect URL (wiremock).
pub async fn export_anki_from_items_url(
    items: &[VocabularyItem],
    deck_name: &str,
    url: &str,
) -> Result<(), AnkiError> {
    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .timeout(std::time::Duration::from_secs(ANKI_TIMEOUT_SECS))
        .build()
        .map_err(|e| AnkiError::Request(e.to_string()))?;
    for item in items {
        let body = serde_json::json!({
            "action": "addNote",
            "version": 6,
            "params": {
                "note": {
                    "deckName": deck_name,
                    "modelName": "Basic",
                    "fields": {
                        "Front": item.word,
                        "Back": item.definition,
                    }
                }
            }
        });
        let resp = client
            .post(url)
            .json(&body)
            .send()
            .await
            .map_err(|e| AnkiError::Request(e.to_string()))?;
        let status = resp.status();
        if status.is_redirection() {
            return Err(AnkiError::Request(format!("redirect rejected: {status}")));
        }
        if !status.is_success() {
            return Err(AnkiError::Request(format!("http {status}")));
        }
        let json: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| AnkiError::Request(e.to_string()))?;
        if let Some(err) = json.get("error").and_then(|v| v.as_str()) {
            if !err.is_empty() {
                return Err(AnkiError::Response(err.to_string()));
            }
        }
    }
    Ok(())
}
