use crate::plugins::history::{HistoryHub, HISTORY};
use crate::{require_database, require_database_write, AppState};
use serde::Serialize;
use std::sync::Arc;
use tauri::Manager;

#[derive(Debug, Clone, Serialize)]
pub struct HistoryPrivacyStatusWire {
    enabled: bool,
    retention_days: u32,
    record_count: u64,
}

fn map_lease(err: impl std::fmt::Display) -> String {
    let text = err.to_string();
    if text.contains("unloaded") {
        "Database not available".into()
    } else {
        text
    }
}

fn lease_history(
    app: &tauri::AppHandle,
) -> Result<linguaray_kernel::ServiceLease<HistoryHub>, String> {
    let supervisor = app
        .try_state::<linguaray_kernel::Supervisor>()
        .ok_or_else(|| "Database not available".to_string())?;
    supervisor.handle().lease(HISTORY).map_err(map_lease)
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
    app: tauri::AppHandle,
    state: tauri::State<'_, Arc<AppState>>,
) -> Result<HistoryPrivacyStatusWire, String> {
    let history = lease_history(&app)?;
    let app_state = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        let _ = &history;
        let gate = app_state.data_gate.read();
        let db = require_database(&app_state, &gate)?;
        read_history_privacy_status(&db)
    })
    .await
    .map_err(|error| format!("history status worker failed: {error}"))?
}

#[tauri::command]
pub async fn history_set_enabled(
    app: tauri::AppHandle,
    app_state: tauri::State<'_, Arc<AppState>>,
    enabled: bool,
) -> Result<HistoryPrivacyStatusWire, String> {
    let history = lease_history(&app)?;
    let app_state = app_state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        let gate = app_state.data_gate.read();
        let db = require_database(&app_state, &gate)?;
        history
            .with(|h| h.set_enabled(&db, enabled))
            .map_err(map_lease)??;
        read_history_privacy_status(&db)
    })
    .await
    .map_err(|error| format!("history enable worker failed: {error}"))?
}

#[tauri::command]
pub async fn history_set_retention(
    app: tauri::AppHandle,
    state: tauri::State<'_, Arc<AppState>>,
    days: u32,
) -> Result<HistoryPrivacyStatusWire, String> {
    let history = lease_history(&app)?;
    let app_state = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        let _ = &history;
        let gate = app_state.data_gate.read();
        let db = require_database(&app_state, &gate)?;
        db.with_conn(|conn| crate::db::history::set_retention(conn, days))
            .map_err(|error| error.to_string())?;
        read_history_privacy_status(&db)
    })
    .await
    .map_err(|error| format!("history retention worker failed: {error}"))?
}

#[tauri::command]
pub async fn history_clear_all(
    app: tauri::AppHandle,
    state: tauri::State<'_, Arc<AppState>>,
) -> Result<HistoryPrivacyStatusWire, String> {
    let history = lease_history(&app)?;
    let app_state = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        history
            .with(|h| h.require_writable())
            .map_err(map_lease)??;
        let gate = app_state.data_gate.write();
        let db = require_database_write(&app_state, &gate)?;
        db.with_conn(crate::db::history::clear_all)
            .map_err(|error| error.to_string())?;
        read_history_privacy_status(&db)
    })
    .await
    .map_err(|error| format!("history clear worker failed: {error}"))?
}

#[tauri::command]
pub async fn history_search(
    app: tauri::AppHandle,
    app_state: tauri::State<'_, Arc<AppState>>,
    query: String,
    cursor: Option<String>,
) -> Result<crate::history::search::HistoryPage, String> {
    let history = lease_history(&app)?;
    let state = app_state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        let _gate = state.data_gate.read();
        let db = require_database(&state, &_gate)?;
        history
            .with(|h| h.search(&db, &query, cursor.as_deref()))
            .map_err(map_lease)?
    })
    .await
    .map_err(|error| error.to_string())?
}

#[tauri::command]
pub async fn history_toggle_favorite(
    app: tauri::AppHandle,
    state: tauri::State<'_, Arc<AppState>>,
    session_uuid: String,
) -> Result<bool, String> {
    let history = lease_history(&app)?;
    let app_state = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        history
            .with(|h| h.require_writable())
            .map_err(map_lease)??;
        let gate = app_state.data_gate.write();
        let db = require_database_write(&app_state, &gate)?;
        db.with_conn(|conn| crate::db::history::toggle_favorite(conn, &session_uuid))
            .map_err(|e| e.to_string())
    })
    .await
    .map_err(|error| format!("history favorite worker failed: {error}"))?
}

#[tauri::command]
pub async fn history_delete_session(
    app: tauri::AppHandle,
    state: tauri::State<'_, Arc<AppState>>,
    session_uuid: String,
) -> Result<(), String> {
    let history = lease_history(&app)?;
    let app_state = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        history
            .with(|h| h.require_writable())
            .map_err(map_lease)??;
        let gate = app_state.data_gate.write();
        let db = require_database_write(&app_state, &gate)?;
        db.with_conn(|conn| crate::db::history::delete_session(conn, &session_uuid))
            .map_err(|e| e.to_string())
    })
    .await
    .map_err(|error| format!("history delete worker failed: {error}"))?
}

#[tauri::command]
pub async fn history_export(
    app: tauri::AppHandle,
    state: tauri::State<'_, Arc<AppState>>,
    file_path: String,
    format: String,
    filter: crate::history::export::HistoryFilter,
) -> Result<String, String> {
    let history = lease_history(&app)?;
    let app_state = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        let gate = app_state.data_gate.read();
        let db = require_database(&app_state, &gate)?;
        history
            .with(|h| h.export(&db, &file_path, &format, &filter))
            .map_err(map_lease)?
    })
    .await
    .map_err(|error| format!("history export worker failed: {error}"))?
}

#[cfg(test)]
mod remnant {
    #[test]
    fn history_commands_do_not_read_session_keystore() {
        let src = include_str!("history.rs");
        let prod = src.split("mod remnant").next().unwrap();
        assert!(
            !prod.contains("session_keystore"),
            "history writes must lease Secrets via HistoryHub"
        );
        assert!(
            !prod.contains("State<'_, Arc<Session>>"),
            "history commands no longer take Session"
        );
    }
}
