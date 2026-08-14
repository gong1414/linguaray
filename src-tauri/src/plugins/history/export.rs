//! Batched export iterator for encrypted history.

use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::db::Database;
use crate::history::search::{search, DecryptedHistorySession, HistorySearchError};
use crate::keystore::Keystore;

/// Filter applied during export. Empty `query` means all retained sessions.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HistoryFilter {
    pub query: Option<String>,
    #[serde(default)]
    pub favorites_only: bool,
}

#[derive(Debug, Clone, Copy)]
pub enum ExportFormat {
    Csv,
    Json,
}

impl ExportFormat {
    pub fn parse(raw: &str) -> Result<Self, HistorySearchError> {
        match raw {
            "csv" => Ok(Self::Csv),
            "json" => Ok(Self::Json),
            _ => Err(HistorySearchError::Cursor),
        }
    }
}

/// Iterate matching sessions via repeated `search()` calls (200/batch).
pub fn export_all(
    db: &Database,
    keystore: &Keystore,
    filter: &HistoryFilter,
) -> Result<Vec<DecryptedHistorySession>, HistorySearchError> {
    let query = filter.query.as_deref().unwrap_or("");
    let mut all = Vec::new();
    let mut cursor: Option<String> = None;
    loop {
        let page = search(db, keystore, query, cursor.as_deref())?;
        let batch_size = page.items.len();
        if filter.favorites_only {
            all.extend(page.items.into_iter().filter(|s| s.is_favorite));
        } else {
            all.extend(page.items);
        }
        if page.scan_complete || page.next_cursor.is_none() {
            break;
        }
        cursor = page.next_cursor;
        if batch_size == 0 {
            break;
        }
    }
    Ok(all)
}

pub fn write_export_file(
    sessions: &[DecryptedHistorySession],
    path: &Path,
    format: ExportFormat,
) -> Result<(), std::io::Error> {
    let content = match format {
        ExportFormat::Csv => format_csv(sessions),
        ExportFormat::Json => serde_json::to_string_pretty(sessions).unwrap_or_else(|_| "[]".into()),
    };
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)?;
        }
    }
    std::fs::write(path, content)
}

fn format_csv(sessions: &[DecryptedHistorySession]) -> String {
    let mut out = String::from(
        "session_uuid,timestamp,trigger_source,target_language,is_favorite,source_text\n",
    );
    for s in sessions {
        let source = s.source_text.as_deref().unwrap_or("").replace('"', "\"\"");
        out.push_str(&format!(
            "{},{},{},{},{},\"{}\"\n",
            s.session_uuid,
            s.timestamp,
            s.trigger_source,
            s.target_language,
            i64::from(s.is_favorite),
            source,
        ));
    }
    out
}
