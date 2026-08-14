use std::sync::Arc;

use crate::dict::{self, DictLookupResult, DictPackageInfo};
use crate::{require_database, require_database_write, AppState};

#[tauri::command]
pub async fn dict_lookup(
    state: tauri::State<'_, Arc<AppState>>,
    word: String,
) -> Result<Option<DictLookupResult>, String> {
    let app_state = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        let gate = app_state.data_gate.read();
        let db = require_database(&app_state, &gate)?;
        let dict_dir = dict::dictionaries_dir(&app_state.db_path);
        dict::lookup(&db, &dict_dir, &word).map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| format!("dict lookup worker failed: {e}"))?
}

#[tauri::command]
pub async fn dict_list_packages(
    state: tauri::State<'_, Arc<AppState>>,
) -> Result<Vec<DictPackageInfo>, String> {
    let app_state = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        let gate = app_state.data_gate.read();
        let db = require_database(&app_state, &gate)?;
        db.with_conn(|conn| dict::list_packages(conn))
            .map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| format!("dict list worker failed: {e}"))?
}

#[tauri::command]
pub async fn dict_install_package(
    state: tauri::State<'_, Arc<AppState>>,
    source_dir: String,
    package_id: String,
    name: String,
    version: String,
) -> Result<(), String> {
    let app_state = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        let gate = app_state.data_gate.write();
        let db = require_database_write(&app_state, &gate)?;
        let dest_root = dict::dictionaries_dir(&app_state.db_path);
        dict::install_package(
            &db,
            std::path::Path::new(&source_dir),
            &dest_root,
            &package_id,
            &name,
            &version,
        )
        .map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| format!("dict install worker failed: {e}"))?
}
