//! Provider diagnostics IPC: model list, connection probe, balance.

use crate::balance::{self, BalanceResult};
use crate::db::providers::{self as db_providers};
use crate::{require_database, AppState, Session};
use serde::Serialize;
use std::sync::Arc;

/// One selectable model for a provider. Assembled from the local profile
/// (current model + catalog default) then extended by an HTTP GET to
/// `models_request_url` when the origin matches.
#[derive(Debug, Clone, Serialize, specta::Type)]
pub struct ModelInfo {
    pub id: String,
    pub label: String,
}

/// Result of a connection probe (P1 #8). `ok` + a human-readable message; the
/// full connection-test HTTP flow is S3 scope, so the current implementation is
/// a best-effort "reachable" check.
///
/// `latency_ms` is `Some(ms)` only on the reachable arm (a real Instant probe
/// of the HTTP round-trip); it is `None` on every early-exit failure arm
/// (empty/invalid endpoint, transport error, missing HTTP client).
#[derive(Debug, Clone, Serialize, specta::Type)]
pub struct ConnectionResult {
    pub ok: bool,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latency_ms: Option<u32>,
}

/// Measure elapsed time since `start` as whole milliseconds, using a saturating
/// conversion so a probe that somehow exceeds `u128` → `u32` range clamps to
/// `u32::MAX` rather than truncating via `as u32` (which silently wraps).
/// Used by `provider_test_connection` to populate `ConnectionResult::latency_ms`.
pub fn measure_latency_ms(start: std::time::Instant) -> u32 {
    u32::try_from(start.elapsed().as_millis()).unwrap_or(u32::MAX)
}

/// List the models a provider can use (P1 #8 + plugin-core §7.4).
///
/// Local list (current model + catalog default) is always assembled first.
/// HTTP GET uses `models_request_url`: if that URL's origin differs from
/// `profile.endpoint`, we return an error and **never** attach a key.
#[tauri::command]
#[specta::specta]
pub async fn provider_get_models(
    state: tauri::State<'_, Arc<AppState>>,
    session: tauri::State<'_, Arc<Session>>,
    uuid: String,
) -> Result<Vec<ModelInfo>, String> {
    let app = state.inner().clone();
    let profile = tauri::async_runtime::spawn_blocking(
        move || -> Result<db_providers::ProviderProfile, String> {
            let _gate = app.data_gate.read();
            let db = require_database(&app, &_gate)?;
            db.with_conn(|conn| db_providers::get(conn, &uuid))
                .map_err(|e| e.to_string())
        },
    )
    .await
    .map_err(|e| e.to_string())??;

    let mut out = local_model_list(&profile);
    // Empty Azure/Custom endpoints have no origin to fetch from. Resolve
    // `models_request_url` only after this check — `Url::parse("")` would
    // otherwise fail and hide the local list behind an error.
    if profile.endpoint.is_empty() {
        return Ok(out);
    }
    let url = match db_providers::models_request_url(&profile) {
        Ok(u) => u,
        Err(e) => return Err(e),
    };
    let Some(client) = session.client.clone() else {
        return Ok(out);
    };
    let key = if profile.needs_key {
        let ks = session.keystore.as_ref().ok_or("keystore unavailable")?;
        match ks.get_key(&profile.secret_ref).map_err(|e| e.to_string())? {
            Some(k) => k,
            None => return Ok(out),
        }
    } else {
        String::new()
    };
    let auth = profile
        .capabilities
        .auth
        .unwrap_or(linguaray_contracts::AuthKind::Bearer);
    let mut req = client.get(&url);
    if profile.needs_key {
        req = crate::plugins::drivers::apply_auth(req, auth, &key);
        if profile.protocol == db_providers::Protocol::Anthropic {
            req = req.header("anthropic-version", "2023-06-01");
        }
    }
    let resp = req.send().await.map_err(|e| e.to_string())?;
    let status = resp.status().as_u16();
    if status == 401 || status == 403 {
        return Err(format!("auth failed ({status})"));
    }
    if !resp.status().is_success() {
        return Err(format!("models endpoint returned {status}"));
    }
    let body: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
    for id in parse_model_ids(&body) {
        if !out.iter().any(|m| m.id == id) {
            out.push(ModelInfo {
                id: id.clone(),
                label: id,
            });
        }
    }
    Ok(out)
}

pub(crate) fn local_model_list(profile: &db_providers::ProviderProfile) -> Vec<ModelInfo> {
    let mut out = Vec::new();
    if let Some(m) = &profile.model {
        if !m.is_empty() {
            out.push(ModelInfo {
                id: m.clone(),
                label: m.clone(),
            });
        }
    }
    if let Some(p) = crate::providers::presets()
        .into_iter()
        .find(|p| p.id == profile.template_id)
    {
        if !p.default_model.is_empty() && profile.model.as_deref() != Some(p.default_model.as_str())
        {
            out.push(ModelInfo {
                id: p.default_model.clone(),
                label: format!("{} (default)", p.default_model),
            });
        }
    }
    out
}

pub(crate) fn parse_model_ids(body: &serde_json::Value) -> Vec<String> {
    if let Some(arr) = body.get("data").and_then(|v| v.as_array()) {
        return arr
            .iter()
            .filter_map(|m| m.get("id").and_then(|id| id.as_str()).map(str::to_string))
            .collect();
    }
    if let Some(arr) = body.as_array() {
        return arr
            .iter()
            .filter_map(|m| m.get("id").and_then(|id| id.as_str()).map(str::to_string))
            .collect();
    }
    Vec::new()
}

/// Probe whether a provider is reachable (P1 #8).
///
/// Reads the profile snapshot in `spawn_blocking`, then runs an async HEAD-ish
/// request against the endpoint. Full connection testing (auth-balanced probe,
/// latency buckets, quota introspection) is S3 scope; for now this is a simple
/// "could we establish a TCP/TLS connection" check that classifies the outcome.
#[tauri::command]
#[specta::specta]
pub async fn provider_test_connection(
    state: tauri::State<'_, Arc<AppState>>,
    session: tauri::State<'_, Arc<Session>>,
    uuid: String,
) -> Result<ConnectionResult, String> {
    let app = state.inner().clone();
    // Read the profile on a blocking thread, then hand the endpoint back to the
    // async caller for the HTTP probe.
    let profile = tauri::async_runtime::spawn_blocking(
        move || -> Result<db_providers::ProviderProfile, String> {
            // Acquire the gate FIRST (see provider_list in crud.rs).
            let _gate = app.data_gate.read();
            let db = require_database(&app, &_gate)?;
            db.with_conn(|conn| db_providers::get(conn, &uuid))
                .map_err(|e| e.to_string())
        },
    )
    .await
    .map_err(|e| e.to_string())??;

    if profile.endpoint.is_empty() {
        return Ok(ConnectionResult {
            ok: false,
            message: "endpoint not configured".into(),
            latency_ms: None,
        });
    }
    // Validate the endpoint shape before sending any bytes.
    if let Err(e) = crate::providers::validate_endpoint(&profile.endpoint) {
        return Ok(ConnectionResult {
            ok: false,
            message: format!("invalid endpoint: {e}"),
            latency_ms: None,
        });
    }
    // Best-effort reachability probe. We don't care about the response body —
    // any HTTP response (even a 401/404) means the endpoint is reachable; only
    // a transport-level failure (connect/timeout/TLS) counts as "not ok".
    let client = match session.client.as_ref() {
        Some(c) => c,
        None => {
            return Ok(ConnectionResult {
                ok: false,
                message: "HTTP client unavailable: startup build failed".into(),
                latency_ms: None,
            })
        }
    };
    // Time only the actual HTTP round-trip (the reachable arm). Early-exit
    // failure arms above carry `latency_ms: None`.
    let probe_start = std::time::Instant::now();
    let req = client.get(&profile.endpoint).send().await;
    match req {
        Ok(resp) => Ok(ConnectionResult {
            ok: true,
            message: format!("reachable (HTTP {})", resp.status().as_u16()),
            latency_ms: Some(measure_latency_ms(probe_start)),
        }),
        Err(e) => Ok(ConnectionResult {
            ok: false,
            message: format!("connection failed: {e}"),
            latency_ms: None,
        }),
    }
}

#[tauri::command]
#[specta::specta]
pub async fn provider_get_balance(
    state: tauri::State<'_, Arc<AppState>>,
    session: tauri::State<'_, Arc<Session>>,
    uuid: String,
) -> Result<BalanceResult, String> {
    let app = state.inner().clone();
    let profile = tauri::async_runtime::spawn_blocking(move || {
        let gate = app.data_gate.read();
        let db = require_database(&app, &gate)?;
        db.with_conn(|conn| db_providers::get(conn, &uuid))
            .map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())??;

    if !balance::should_fetch(profile.capabilities.balance) {
        return Ok(BalanceResult::Unsupported);
    }
    let keystore = session
        .keystore
        .as_ref()
        .ok_or_else(|| "keystore unavailable".to_string())?;
    let key = keystore
        .get_key(&profile.secret_ref)
        .map_err(|e| e.to_string())?
        .unwrap_or_default();
    let url = profile
        .capabilities
        .models_url
        .as_deref()
        .and_then(|u| url::Url::parse(u).ok())
        .and_then(|u| u.origin().ascii_serialization().parse::<url::Url>().ok())
        .map(|o| format!("{}/v1/dashboard/billing/credit_grants", o))
        .unwrap_or_else(|| format!("{}/../dashboard/billing/credit_grants", profile.endpoint));
    Ok(balance::fetch_balance_url(&url, &key).await)
}
