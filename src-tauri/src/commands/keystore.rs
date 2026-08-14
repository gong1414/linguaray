use crate::{apply_keystore_recovery_db_cleanup, session_keystore, AppState, Session};
use std::sync::Arc;

#[tauri::command]
pub fn set_key(
    state: tauri::State<'_, Arc<Session>>,
    app: tauri::State<'_, Arc<AppState>>,
    provider_id: String,
    key: String,
) -> Result<(), String> {
    let _gate = app.data_gate.read();
    assert_secret_ref_owned(&app, &provider_id)?;
    let keystore = session_keystore(&state)?;
    keystore
        .set_key(&provider_id, &key)
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn delete_key(
    state: tauri::State<'_, Arc<Session>>,
    app: tauri::State<'_, Arc<AppState>>,
    provider_id: String,
) -> Result<(), String> {
    let _gate = app.data_gate.read();
    assert_secret_ref_owned(&app, &provider_id)?;
    let keystore = session_keystore(&state)?;
    keystore
        .delete_key(&provider_id)
        .map_err(|e| e.to_string())?;
    Ok(())
}

fn assert_secret_ref_owned(app: &Arc<AppState>, provider_id: &str) -> Result<(), String> {
    let readiness = app.readiness.read();
    if !readiness.is_ready() {
        return Err(format!(
            "cannot set/delete key: database not ready ({:?})",
            *readiness
        ));
    }
    drop(readiness);
    let db = app
        .db
        .read()
        .clone()
        .ok_or_else(|| "cannot set/delete key: database unavailable".to_string())?;
    let owned: i64 = db
        .with_conn(|conn| -> Result<i64, crate::db::DbError> {
            let n: i64 = conn.query_row(
                "SELECT COUNT(*) FROM providers WHERE secret_ref=?1 AND status != 'deleted'",
                rusqlite::params![provider_id],
                |r| r.get(0),
            )?;
            Ok(n)
        })
        .map_err(|e| format!("cannot set/delete key: db lookup failed: {e}"))?;
    if owned == 0 {
        return Err(format!(
            "cannot set/delete key: no active provider owns secret_ref '{provider_id}'"
        ));
    }
    Ok(())
}

#[tauri::command]
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
pub fn keystore_health(state: tauri::State<'_, Arc<Session>>) -> String {
    match state.keystore.as_ref() {
        Some(ks) => match ks.load() {
            Ok(_) => String::new(),
            Err(e) => format!("{e}"),
        },
        None => "keystore unavailable: startup init failed".to_string(),
    }
}
