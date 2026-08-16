//! Provider CRUD IPC (list / create / update / duplicate / delete / reorder /
//! toggle). All commands acquire `data_gate` FIRST, then clone the DB Arc
//! (atomic w.r.t. archive/reset swapping the handle).

use crate::db::providers::{self as db_providers, ProviderPatch, ProviderProfile};
use crate::{keystore, require_database, require_database_write, AppState};
use std::sync::Arc;

use super::ProviderCommandError;

/// List active provider profiles (`status='active'`), ordered by `sort_order`.
#[tauri::command]
#[specta::specta]
pub async fn provider_list(
    state: tauri::State<'_, Arc<AppState>>,
) -> Result<Vec<ProviderProfile>, String> {
    let app = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        // Acquire the gate FIRST so the readiness check + Arc clone are atomic
        // w.r.t. archive/reset/recovery (which take the write guard + swap the
        // DB handle). Cloning the Arc before the gate (the old shape) raced the
        // swap and could hand the command a stale DB.
        let _gate = app.data_gate.read();
        let db = require_database(&app, &_gate)?;
        db.with_conn(|conn| db_providers::list(conn))
            .map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}

/// Create a new provider from a template (preset id). The preset catalog
/// derives protocol/endpoint/default-model/needs_key; caller values override
/// endpoint/model when non-empty.
#[tauri::command]
#[specta::specta]
pub async fn provider_create(
    state: tauri::State<'_, Arc<AppState>>,
    app_handle: tauri::AppHandle,
    template_id: String,
    name: String,
    endpoint: String,
    model: Option<String>,
) -> Result<ProviderProfile, String> {
    let app_state = state.inner().clone();
    let result = tauri::async_runtime::spawn_blocking(move || {
        // Acquire the gate FIRST (see provider_list).
        let _gate = app_state.data_gate.read();
        let db = require_database(&app_state, &_gate)?;
        db.with_conn(|conn| {
            db_providers::create(conn, &template_id, &name, &endpoint, model.as_deref())
        })
        .map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())??;
    // rev-8-8: refresh the tray AFTER the write commits. Best-effort.
    crate::refresh_tray_if_available(&app_handle).await;
    Ok(result)
}

/// Apply a partial patch to a provider with optimistic-lock (CAS) semantics
/// (R2-E). An endpoint change is validated and may invalidate the parallel
/// consent (see `db_providers::update`). The patch's `expected_version` must
/// match the row's current `version`; a mismatch rejects with a structured
/// `{"error":"stale_version","actual_version":N}` so the UI can show a
/// save-conflict banner instead of clobbering the other writer's change.
#[tauri::command]
#[specta::specta]
pub async fn provider_update(
    state: tauri::State<'_, Arc<AppState>>,
    app_handle: tauri::AppHandle,
    uuid: String,
    patch: ProviderPatch,
) -> Result<ProviderProfile, ProviderCommandError> {
    let app_state = state.inner().clone();
    let result = tauri::async_runtime::spawn_blocking(
        move || -> Result<ProviderProfile, ProviderCommandError> {
            // Acquire the gate FIRST (see provider_list).
            let _gate = app_state.data_gate.read();
            let db = require_database(&app_state, &_gate).map_err(ProviderCommandError::from)?;
            // The typed `UpdateOutcome` carries the stale/not-found signals out of
            // the `with_conn` closure (whose error type is fixed to `DbError`), then
            // we map them to structured `ProviderCommandError` variants here — same
            // pattern as `ConfirmActiveOutcome` (no string-prefix parsing).
            let outcome = db.with_conn(|conn| db_providers::update(conn, &uuid, &patch));
            outcome
                .map(|o| match o {
                    db_providers::UpdateOutcome::Written(p) => Ok(p),
                    db_providers::UpdateOutcome::StaleVersion { actual_version } => {
                        Err(ProviderCommandError::StaleVersion { actual_version })
                    }
                    db_providers::UpdateOutcome::NotFound => {
                        Err(ProviderCommandError::Validation {
                            message: "provider not found".into(),
                        })
                    }
                })
                .map_err(ProviderCommandError::from)?
        },
    )
    .await
    .map_err(|e| ProviderCommandError::Db {
        message: format!("{e:?}"),
    })??;
    crate::refresh_tray_if_available(&app_handle).await;
    Ok(result)
}

/// Duplicate a provider. New UUID, new `secret_ref`, keyless (the original key
/// is never copied).
#[tauri::command]
#[specta::specta]
pub async fn provider_duplicate(
    state: tauri::State<'_, Arc<AppState>>,
    app_handle: tauri::AppHandle,
    uuid: String,
) -> Result<ProviderProfile, String> {
    let app_state = state.inner().clone();
    let result = tauri::async_runtime::spawn_blocking(move || {
        // Acquire the gate FIRST (see provider_list).
        let _gate = app_state.data_gate.read();
        let db = require_database(&app_state, &_gate)?;
        db.with_conn(|conn| db_providers::duplicate(conn, &uuid))
            .map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())??;
    crate::refresh_tray_if_available(&app_handle).await;
    Ok(result)
}

/// Begin the 3-step delete (mark `deleting`, evict from slots), purge the key
/// from the keystore, then finalize the tombstone. Each step is committed before
/// the next; the lock-order rule (DB Mutex and keystore flock never nested) is
/// preserved by releasing the DB guard between steps. All three steps run on one
/// blocking thread so the `data_gate` write guard spans the whole operation.
#[tauri::command]
#[specta::specta]
pub async fn provider_delete(
    state: tauri::State<'_, Arc<AppState>>,
    app_handle: tauri::AppHandle,
    uuid: String,
) -> Result<(), String> {
    let app_state = state.inner().clone();
    let keystore_dir = app_state.keystore_dir.clone();
    tauri::async_runtime::spawn_blocking(move || -> Result<(), String> {
        // Write guard: a delete mutates selection slots + status; no reader/other
        // writer may interleave. Held for all 3 steps (the DB Mutex + keystore
        // flock are still released between steps inside their own calls).
        // Acquire the gate FIRST (see provider_list) so the readiness check +
        // Arc clone are atomic w.r.t. the DB swap.
        let _gate = app_state.data_gate.write();
        let db = require_database_write(&app_state, &_gate)?;

        // Step 1: begin_delete under the DB Mutex → returns the secret_ref. The
        // DB guard (with_conn closure) is released before the keystore step.
        let secret_ref = db
            .with_conn(|conn| db_providers::begin_delete(conn, &uuid))
            .map_err(|e| e.to_string())?;

        // Step 2: purge the key (keystore flock only, DB NOT locked). Uses the
        // typed `delete_key` RMW so the payload converges to v2. Idempotent — a
        // missing key is a successful no-op.
        let ks = keystore::Keystore::new(keystore_dir).map_err(|e| e.to_string())?;
        ks.delete_key(&secret_ref).map_err(|e| e.to_string())?;

        // Step 3: finalize the tombstone (DB Mutex only, keystore NOT locked).
        db.with_conn(|conn| db_providers::finalize_delete(conn, &uuid))
            .map_err(|e| e.to_string())?;
        Ok(())
    })
    .await
    .map_err(|e| e.to_string())??;
    crate::refresh_tray_if_available(&app_handle).await;
    Ok(())
}

/// Re-assign `sort_order` to the given UUID order. The list MUST be exactly the
/// set of active UUIDs.
#[tauri::command]
#[specta::specta]
pub async fn provider_reorder(
    state: tauri::State<'_, Arc<AppState>>,
    app_handle: tauri::AppHandle,
    uuids: Vec<String>,
) -> Result<(), String> {
    let app_state = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        // Acquire the gate FIRST (see provider_list).
        let _gate = app_state.data_gate.write();
        let db = require_database_write(&app_state, &_gate)?;
        db.with_conn(|conn| db_providers::reorder(conn, &uuids))
            .map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())??;
    crate::refresh_tray_if_available(&app_handle).await;
    Ok(())
}

/// Flip `enabled`. Disabling also evicts the row from selection slots and
/// invalidates parallel consent (mirrors `begin_delete`).
#[tauri::command]
#[specta::specta]
pub async fn provider_toggle(
    state: tauri::State<'_, Arc<AppState>>,
    app_handle: tauri::AppHandle,
    uuid: String,
    enabled: bool,
) -> Result<(), String> {
    let app_state = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        // Acquire the gate FIRST (see provider_list).
        let _gate = app_state.data_gate.write();
        let db = require_database_write(&app_state, &_gate)?;
        db.with_conn(|conn| db_providers::toggle(conn, &uuid, enabled))
            .map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())??;
    crate::refresh_tray_if_available(&app_handle).await;
    Ok(())
}
