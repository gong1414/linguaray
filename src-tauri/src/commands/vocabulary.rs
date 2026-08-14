use crate::plugins::history::{HistoryHub, HISTORY};
use crate::plugins::vocabulary;
use crate::{require_database, require_database_write, AppState};
use std::sync::Arc;
use tauri::Manager;

fn map_lease(err: impl std::fmt::Display) -> String {
    err.to_string()
}

fn lease_history(
    app: &tauri::AppHandle,
) -> Result<linguaray_kernel::ServiceLease<HistoryHub>, String> {
    let supervisor = app
        .try_state::<linguaray_kernel::Supervisor>()
        .ok_or_else(|| "Database not available".to_string())?;
    supervisor.handle().lease(HISTORY).map_err(map_lease)
}

#[tauri::command]
pub async fn vocabulary_add(
    app: tauri::AppHandle,
    state: tauri::State<'_, Arc<AppState>>,
    word: String,
    definition: String,
    source_language: String,
    target_language: String,
) -> Result<vocabulary::VocabularyItem, String> {
    let history = lease_history(&app)?;
    let app_state = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        let gate = app_state.data_gate.write();
        let db = require_database_write(&app_state, &gate)?;
        history
            .with(|h| h.vocabulary_add(&db, &word, &definition, &source_language, &target_language))
            .map_err(map_lease)?
    })
    .await
    .map_err(|e| format!("vocabulary add worker failed: {e}"))?
}

#[tauri::command]
pub async fn vocabulary_list(
    app: tauri::AppHandle,
    state: tauri::State<'_, Arc<AppState>>,
    cursor: Option<String>,
) -> Result<vocabulary::VocabularyPage, String> {
    let history = lease_history(&app)?;
    let app_state = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        let gate = app_state.data_gate.read();
        let db = require_database(&app_state, &gate)?;
        history
            .with(|h| h.vocabulary_list(&db, cursor.as_deref()))
            .map_err(map_lease)?
    })
    .await
    .map_err(|e| format!("vocabulary list worker failed: {e}"))?
}

#[tauri::command]
pub async fn vocabulary_delete(
    app: tauri::AppHandle,
    state: tauri::State<'_, Arc<AppState>>,
    item_uuid: String,
) -> Result<(), String> {
    let history = lease_history(&app)?;
    let app_state = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        history
            .with(|h| h.require_writable())
            .map_err(map_lease)??;
        let gate = app_state.data_gate.write();
        let db = require_database_write(&app_state, &gate)?;
        vocabulary::delete_word(&db, &item_uuid)
    })
    .await
    .map_err(|e| format!("vocabulary delete worker failed: {e}"))?
}

#[tauri::command]
pub async fn vocabulary_export_file(
    app: tauri::AppHandle,
    state: tauri::State<'_, Arc<AppState>>,
    file_path: String,
    format: String,
) -> Result<String, String> {
    let history = lease_history(&app)?;
    let app_state = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        let gate = app_state.data_gate.read();
        let db = require_database(&app_state, &gate)?;
        history
            .with(|h| h.vocabulary_export(&db, &file_path, &format))
            .map_err(map_lease)?
    })
    .await
    .map_err(|e| format!("vocabulary export worker failed: {e}"))?
}

#[tauri::command]
pub async fn vocabulary_export_anki(
    app: tauri::AppHandle,
    state: tauri::State<'_, Arc<AppState>>,
    deck_name: String,
) -> Result<(), String> {
    let history = lease_history(&app)?;
    let app_state = state.inner().clone();
    let items = tauri::async_runtime::spawn_blocking(move || {
        let gate = app_state.data_gate.read();
        let db = require_database(&app_state, &gate)?;
        history
            .with(|h| h.vocabulary_collect(&db))
            .map_err(map_lease)?
    })
    .await
    .map_err(|e| format!("vocabulary anki collect worker failed: {e}"))??;
    vocabulary::export_anki_from_items(&items, &deck_name)
        .await
        .map_err(|e| e.to_string())
}
