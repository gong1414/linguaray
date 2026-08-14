use crate::{require_ready_gated, require_ready_gated_write, session_keystore, AppState, Session};
use serde::Serialize;
use std::sync::Arc;

#[derive(Debug, Clone, Serialize)]
pub struct HistoryPrivacyStatusWire {
    enabled: bool,
    retention_days: u32,
    record_count: u64,
}

fn read_history_privacy_status(
    db: &crate::db::Database,
) -> Result<HistoryPrivacyStatusWire, String> {
    db.with_conn(|conn| {
        let status = crate::db::history::privacy_status(conn)?;
        let count: i64 = conn.query_row("SELECT COUNT(*) FROM history_sessions", [], |row| {
            row.get(0)
        })?;
        let record_count = u64::try_from(count)
            .map_err(|_| crate::db::DbError::Integrity("negative history count".into()))?;
        Ok(HistoryPrivacyStatusWire {
            enabled: status.enabled,
            retention_days: status.retention_days,
            record_count,
        })
    })
    .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn history_privacy_status(
    state: tauri::State<'_, Arc<AppState>>,
) -> Result<HistoryPrivacyStatusWire, String> {
    let app_state = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        let gate = app_state.data_gate.read();
        let db = require_ready_gated(&app_state, &gate)?;
        read_history_privacy_status(&db)
    })
    .await
    .map_err(|error| format!("history status worker failed: {error}"))?
}

#[tauri::command]
pub async fn history_set_enabled(
    app_state: tauri::State<'_, Arc<AppState>>,
    session: tauri::State<'_, Arc<Session>>,
    enabled: bool,
) -> Result<HistoryPrivacyStatusWire, String> {
    let app_state = app_state.inner().clone();
    let session = session.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        let gate = app_state.data_gate.read();
        let db = require_ready_gated(&app_state, &gate)?;
        let keystore = session_keystore(&session)?;
        crate::db::history::set_enabled(&db, keystore, enabled)
            .map_err(|error| error.to_string())?;
        read_history_privacy_status(&db)
    })
    .await
    .map_err(|error| format!("history enable worker failed: {error}"))?
}

#[tauri::command]
pub async fn history_set_retention(
    state: tauri::State<'_, Arc<AppState>>,
    days: u32,
) -> Result<HistoryPrivacyStatusWire, String> {
    let app_state = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        let gate = app_state.data_gate.read();
        let db = require_ready_gated(&app_state, &gate)?;
        db.with_conn(|conn| crate::db::history::set_retention(conn, days))
            .map_err(|error| error.to_string())?;
        read_history_privacy_status(&db)
    })
    .await
    .map_err(|error| format!("history retention worker failed: {error}"))?
}

#[tauri::command]
pub async fn history_clear_all(
    state: tauri::State<'_, Arc<AppState>>,
) -> Result<HistoryPrivacyStatusWire, String> {
    let app_state = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        let gate = app_state.data_gate.write();
        let db = require_ready_gated_write(&app_state, &gate)?;
        db.with_conn(crate::db::history::clear_all)
            .map_err(|error| error.to_string())?;
        read_history_privacy_status(&db)
    })
    .await
    .map_err(|error| format!("history clear worker failed: {error}"))?
}

#[tauri::command]
pub async fn history_search(
    state: tauri::State<'_, Arc<Session>>,
    app_state: tauri::State<'_, Arc<AppState>>,
    query: String,
    cursor: Option<String>,
) -> Result<crate::history::search::HistoryPage, String> {
    session_keystore(&state)?;
    let session = state.inner().clone();
    let state = app_state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        let _gate = state.data_gate.read();
        let db = require_ready_gated(&state, &_gate)?;
        let keystore = session_keystore(&session)?;
        crate::history::search::search(&db, keystore, &query, cursor.as_deref())
            .map_err(|error| error.to_string())
    })
    .await
    .map_err(|error| error.to_string())?
}
