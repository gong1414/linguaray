//! Provider key IPC — keystore set/clear (cross-store, lock-ordered: DB read
//! releases before the keystore RMW begins).

use crate::db::providers::{self as db_providers};
use crate::{keystore, require_database, AppState};
use std::sync::Arc;

/// Set/clear a provider's API key in the keystore. The provider row's
/// `secret_ref` names the key. Cross-store (DB read → keystore write) but the
/// two locks are never held at once: the DB read releases before the keystore
/// RMW begins (lock-order rule). Both steps run on one blocking thread.
///
/// R11: the command fast-fails on empty keys before spawning a thread, then
/// delegates to the synchronous, testable [`set_key_blocking`] core.
#[tauri::command]
#[specta::specta]
pub async fn provider_set_key(
    state: tauri::State<'_, Arc<AppState>>,
    uuid: String,
    key: String,
) -> Result<(), String> {
    // R11: fail fast on empty/whitespace keys at the command boundary, BEFORE
    // spawning a blocking thread — no DB or keystore work for an empty key. The
    // ORIGINAL key value is stored below (trim is validation-only).
    if key.trim().is_empty() {
        return Err("key must not be empty".to_string());
    }
    let app = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || set_key_blocking(&app, &uuid, &key))
        .await
        .map_err(|e| e.to_string())?
}

/// R11: synchronous, testable core of [`provider_set_key`]. Validates the
/// request (non-empty key, active profile, `needs_key=true`), then writes the
/// key to the keystore.
///
/// Validation order:
/// 1. Empty/whitespace key → reject (defense-in-depth; the command wrapper also
///    checks this before spawning a thread).
/// 2. Non-active status → reject (existing behavior: a row mid-deletion must not
///    resurrect a secret whose owner is being torn down).
/// 3. `needs_key=false` → reject (R11): a keyless provider must never accept a
///    key — writing one would leave a dangling secret the provider will never
///    read, and the UI now hides the key input for it.
///
/// The ORIGINAL key value is stored (`trim()` is validation-only — the user's
/// exact value is preserved). Gate-first ordering is load-bearing (see
/// `provider_list` in crud.rs): the readiness check + Arc clone are atomic
/// w.r.t. the DB swap because the gate read guard is held across them.
pub fn set_key_blocking(app: &Arc<AppState>, uuid: &str, key: &str) -> Result<(), String> {
    // R11: reject empty/whitespace keys at the core boundary too (defense in
    // depth — the command wrapper checks this before spawning a thread, but a
    // direct caller of the core would otherwise skip it).
    if key.trim().is_empty() {
        return Err("key must not be empty".to_string());
    }
    let keystore_dir = app.keystore_dir.clone();
    // Acquire the gate FIRST (see provider_list) so the readiness check + Arc
    // clone are atomic w.r.t. the DB swap.
    let _gate = app.data_gate.read();
    let db = require_database(app, &_gate)?;

    // 1. Read the secret_ref + status + needs_key under the DB Mutex, then
    //    release. Reject deleting/deleted profiles: writing a key for a row
    //    that's mid-deletion would resurrect a secret whose owner is being torn
    //    down, and the next finalize_delete would orphan it silently. Reject
    //    keyless providers (R11): a needs_key=false row must never hold a key.
    let secret_ref = db
        .with_conn(|conn| {
            let p = db_providers::get(conn, uuid)?;
            if p.status != "active" {
                return Err(crate::db::DbError::Integrity(format!(
                    "provider {} status is '{}'; cannot set key on a non-active profile",
                    uuid, p.status
                )));
            }
            if !p.needs_key {
                return Err(crate::db::DbError::Integrity(
                    "this provider type does not require a key".to_string(),
                ));
            }
            Ok(p.secret_ref)
        })
        .map_err(|e| e.to_string())?;

    // 2. Keystore RMW (flock only, DB NOT locked). Typed accessor converges
    //    the payload to v2 and handles both v1 flat-map and v2 shapes.
    let ks = keystore::Keystore::new(keystore_dir).map_err(|e| e.to_string())?;
    ks.set_key(&secret_ref, key).map_err(|e| e.to_string())?;
    Ok(())
}
