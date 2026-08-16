use crate::{apply_keystore_recovery_db_cleanup, AppState, Session};
use std::sync::Arc;

#[tauri::command]
#[specta::specta]
pub async fn archive_keystore(state: tauri::State<'_, Arc<AppState>>) -> Result<String, String> {
    let app = state.inner().clone();
    let ks_dir = app.keystore_dir.clone();
    tauri::async_runtime::spawn_blocking(move || -> Result<String, String> {
        let _gate = app.data_gate.write();
        let ks = crate::keystore::Keystore::new(ks_dir.clone()).map_err(|e| e.to_string())?;
        let dst = ks.archive().map_err(|e| e.to_string())?;
        let dst_str = dst.to_string_lossy().into_owned();
        apply_keystore_recovery_db_cleanup(&app)?;
        Ok(dst_str)
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
#[specta::specta]
pub async fn reset_keystore(
    state: tauri::State<'_, Arc<AppState>>,
) -> Result<Option<String>, String> {
    let app = state.inner().clone();
    let ks_dir = app.keystore_dir.clone();
    tauri::async_runtime::spawn_blocking(move || -> Result<Option<String>, String> {
        let _gate = app.data_gate.write();
        let ks = crate::keystore::Keystore::new(ks_dir.clone()).map_err(|e| e.to_string())?;
        let archived = ks
            .reset()
            .map_err(|e| e.to_string())?
            .map(|p| p.to_string_lossy().into_owned());
        apply_keystore_recovery_db_cleanup(&app)?;
        Ok(archived)
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
#[specta::specta]
pub fn key_status(
    state: tauri::State<'_, Arc<Session>>,
) -> std::collections::HashMap<String, bool> {
    let refs = match state.keystore.as_ref() {
        Some(ks) => match ks.list_provider_key_refs() {
            Ok(r) => r,
            Err(_) => return std::collections::HashMap::new(),
        },
        None => return std::collections::HashMap::new(),
    };
    refs.into_iter().map(|r| (r, true)).collect()
}

#[tauri::command]
#[specta::specta]
pub fn keystore_health(state: tauri::State<'_, Arc<Session>>) -> String {
    match state.keystore.as_ref() {
        Some(ks) => match ks.load() {
            Ok(_) => String::new(),
            Err(e) => format!("{e}"),
        },
        None => "keystore unavailable: startup init failed".to_string(),
    }
}
