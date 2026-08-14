use crate::db::schema;
use crate::external_api::{start_listener, ApiHooks, ExternalApiHandle, ExternalApiStatus};
use crate::ocr;
use crate::{require_database, require_database_write, AppState, Session};
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use rand::RngCore;
use std::sync::Arc;
use tauri::{AppHandle, Emitter, Manager};

pub struct ExternalApiSlot(pub parking_lot::Mutex<Option<ExternalApiHandle>>);

impl ExternalApiSlot {
    pub fn new() -> Self {
        Self(parking_lot::Mutex::new(None))
    }
}

fn mint_token() -> String {
    let mut bytes = [0u8; 32];
    rand::rngs::OsRng.fill_bytes(&mut bytes);
    URL_SAFE_NO_PAD.encode(bytes)
}

fn production_hooks(app: AppHandle) -> ApiHooks {
    let providers_app = app.clone();
    let translate_app = app.clone();
    let selection_app = app.clone();
    let input_app = app.clone();
    ApiHooks {
        health_version: env!("CARGO_PKG_VERSION").into(),
        providers: Box::new(move || {
            let state = providers_app
                .try_state::<Arc<AppState>>()
                .ok_or_else(|| "app state missing".to_string())?;
            let gate = state.data_gate.read();
            let db = require_database(&state, &gate)?;
            let rows = db
                .with_conn(|conn| crate::db::providers::list(conn))
                .map_err(|e| e.to_string())?;
            Ok(serde_json::to_value(
                rows.into_iter()
                    .map(|p| {
                        serde_json::json!({
                            "uuid": p.uuid,
                            "name": p.name,
                            "template_id": p.template_id,
                            "enabled": p.enabled,
                        })
                    })
                    .collect::<Vec<_>>(),
            )
            .unwrap_or(serde_json::json!([])))
        }),
        translate: Box::new(move |body| {
            let text = body
                .get("text")
                .and_then(|v| v.as_str())
                .ok_or_else(|| "missing text".to_string())?
                .to_string();
            let from = body
                .get("from")
                .and_then(|v| v.as_str())
                .unwrap_or("auto")
                .to_string();
            let to = body
                .get("to")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let handle = translate_app.clone();
            tauri::async_runtime::block_on(async move {
                let state = handle
                    .try_state::<Arc<AppState>>()
                    .ok_or_else(|| "app state missing".to_string())?;
                let session = handle
                    .try_state::<Arc<Session>>()
                    .ok_or_else(|| "session missing".to_string())?;
                let gate = state.data_gate.read();
                let db = require_database(&state, &gate)?;
                drop(gate);
                let client = session
                    .client
                    .as_ref()
                    .ok_or_else(|| "http client missing".to_string())?;
                let result = crate::commands::translate::run_translate_session_no_settings(
                    &db,
                    client,
                    session.keystore.as_deref(),
                    &text,
                    &from,
                    &to,
                )
                .await?;
                serde_json::to_value(result).map_err(|e| e.to_string())
            })
        }),
        ocr: Box::new(|bytes| {
            let r = ocr::recognize_image_bytes(bytes).map_err(|e| e.to_string())?;
            Ok(serde_json::json!({"text": r.text, "confidence": r.confidence}))
        }),
        selection: Box::new(move || {
            selection_app
                .emit("tray-action", "translate-selection")
                .map_err(|e| e.to_string())
        }),
        show_input: Box::new(move || {
            if let Some(w) = input_app.get_webview_window("input") {
                let _ = w.show();
                let _ = w.set_focus();
            }
            Ok(())
        }),
    }
}

#[tauri::command]
pub async fn external_api_enable(
    app: AppHandle,
    state: tauri::State<'_, Arc<AppState>>,
    session: tauri::State<'_, Arc<Session>>,
    slot: tauri::State<'_, Arc<ExternalApiSlot>>,
    port: Option<u16>,
) -> Result<String, String> {
    let port = port.unwrap_or(crate::external_api::DEFAULT_PORT);
    let token = mint_token();
    let hooks = production_hooks(app.clone());
    let handle = start_listener(port, token.clone(), hooks)
        .map_err(|_| format!("port in use: {port}"))?;
    let bound = handle.port;

    let keystore = session
        .keystore
        .as_ref()
        .ok_or_else(|| "keystore unavailable".to_string())?;
    if let Err(e) = keystore.set_external_api_token(&token) {
        drop(handle);
        return Err(e.to_string());
    }
    let app_state = state.inner().clone();
    let db_result = tauri::async_runtime::spawn_blocking(move || {
        let gate = app_state.data_gate.write();
        let db = require_database_write(&app_state, &gate)?;
        db.with_conn(|conn| {
            schema::ensure_preference_columns(conn)?;
            conn.execute(
                "UPDATE preferences SET external_api_enabled=1, external_api_port=?1 WHERE id=1",
                rusqlite::params![bound as i64],
            )?;
            Ok(())
        })
        .map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?;
    if let Err(e) = db_result {
        let _ = keystore.clear_external_api_token();
        drop(handle);
        return Err(e);
    }

    *slot.0.lock() = Some(handle);
    Ok(token)
}

#[tauri::command]
pub async fn external_api_status(
    state: tauri::State<'_, Arc<AppState>>,
    slot: tauri::State<'_, Arc<ExternalApiSlot>>,
) -> Result<ExternalApiStatus, String> {
    if let Some(h) = slot.0.lock().as_ref() {
        return Ok(ExternalApiStatus::Enabled { port: h.port });
    }
    let app_state = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        let gate = app_state.data_gate.read();
        let db = require_database(&app_state, &gate)?;
        db.with_conn(|conn| {
            schema::ensure_preference_columns(conn)?;
            let (enabled, port): (i64, i64) = conn.query_row(
                "SELECT external_api_enabled, external_api_port FROM preferences WHERE id=1",
                [],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )?;
            Ok(if enabled == 0 {
                ExternalApiStatus::Disabled
            } else {
                ExternalApiStatus::PortInUse {
                    configured_port: port as u16,
                }
            })
        })
        .map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn external_api_disable(
    state: tauri::State<'_, Arc<AppState>>,
    session: tauri::State<'_, Arc<Session>>,
    slot: tauri::State<'_, Arc<ExternalApiSlot>>,
) -> Result<(), String> {
    if let Some(mut h) = slot.0.lock().take() {
        h.stop();
    }
    if let Some(ks) = session.keystore.as_ref() {
        ks.clear_external_api_token().map_err(|e| e.to_string())?;
    }
    let app_state = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        let gate = app_state.data_gate.write();
        let db = require_database_write(&app_state, &gate)?;
        db.with_conn(|conn| {
            schema::ensure_preference_columns(conn)?;
            conn.execute(
                "UPDATE preferences SET external_api_enabled=0 WHERE id=1",
                [],
            )?;
            Ok(())
        })
        .map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub fn external_api_regenerate_token(
    session: tauri::State<'_, Arc<Session>>,
    slot: tauri::State<'_, Arc<ExternalApiSlot>>,
) -> Result<String, String> {
    let token = mint_token();
    let keystore = session
        .keystore
        .as_ref()
        .ok_or_else(|| "keystore unavailable".to_string())?;
    keystore
        .set_external_api_token(&token)
        .map_err(|e| e.to_string())?;
    if let Some(h) = slot.0.lock().as_ref() {
        h.set_token(token.clone());
    }
    Ok(token)
}
