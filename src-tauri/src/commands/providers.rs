//! Provider CRUD / diagnostics IPC (plugin-core PR-3).

use crate::balance::{self, BalanceResult};
use crate::db::providers::{self as db_providers, ActiveSelection, ProviderPatch, ProviderProfile};
use crate::db::readiness::DataReadiness;
use crate::{keystore, require_database, require_database_write, AppState, Session};
use serde::Serialize;
use std::sync::Arc;

#[tauri::command]
pub fn get_data_readiness(state: tauri::State<'_, Arc<AppState>>) -> DataReadiness {
    state.readiness.read().clone()
}

#[derive(Serialize)]
pub struct CatalogPresetDto {
    id: String,
    label: String,
    endpoint: String,
    default_model: String,
    needs_key: bool,
    auth: linguaray_contracts::AuthKind,
    requires_user_endpoint: bool,
    notes: Option<String>,
    console_url: Option<String>,
    support_tier: linguaray_contracts::SupportTier,
    icon: Option<String>,
}

/// Official catalog rows for the Provider Center preset grid.
/// No DB / keystore — default deny except the main window capability.
#[tauri::command]
pub fn provider_list_presets() -> Result<Vec<CatalogPresetDto>, String> {
    let file = linguaray_catalog::load().map_err(|e| e.to_string())?;
    Ok(file
        .providers
        .into_iter()
        .map(|p| CatalogPresetDto {
            id: p.id,
            label: p.label,
            endpoint: p.endpoint,
            default_model: p.default_model,
            needs_key: p.needs_key,
            auth: p.auth,
            requires_user_endpoint: p.requires_user_endpoint,
            notes: p.notes,
            console_url: p.console_url,
            support_tier: p.support_tier,
            icon: p.icon,
        })
        .collect())
}

/// List active provider profiles (`status='active'`), ordered by `sort_order`.
#[tauri::command]
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

/// Set/clear a provider's API key in the keystore. The provider row's
/// `secret_ref` names the key. Cross-store (DB read → keystore write) but the
/// two locks are never held at once: the DB read releases before the keystore
/// RMW begins (lock-order rule). Both steps run on one blocking thread.
///
/// R11: the command fast-fails on empty keys before spawning a thread, then
/// delegates to the synchronous, testable [`set_key_blocking`] core.
#[tauri::command]
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
/// `provider_list`): the readiness check + Arc clone are atomic w.r.t. the DB
/// swap because the gate read guard is held across them.
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

/// Set the active selection (primary, parallel, fallback). Empty primary and
/// empty parallel list mean "no selection". Validates the selection against the
/// active provider set before writing.
///
/// Review P1 #3 (multi-engine consent): when `parallel` is non-empty, the
/// backend recomputes the canonical consent scope and compares it against the
/// stored scope. A mismatch (no prior consent, or a different parallel set)
/// returns [`db_providers::ConsentError::ConsentRequired`] carrying the
/// `actual_scope`. The frontend shows the consent dialog, then calls
/// [`provider_confirm_and_set_active`] with `expected_scope = actual_scope` to
/// record the approval. A matching scope (re-affirming the same selection) is
/// written immediately.
#[tauri::command]
pub async fn provider_set_active(
    state: tauri::State<'_, Arc<AppState>>,
    app_handle: tauri::AppHandle,
    primary: String,
    parallel: Vec<String>,
    fallback: Option<String>,
) -> Result<SetActiveResult, String> {
    let app_state = state.inner().clone();
    let outcome =
        tauri::async_runtime::spawn_blocking(move || -> Result<SetActiveResult, String> {
            // Acquire the gate FIRST (see provider_list).
            let _gate = app_state.data_gate.write();
            let db = require_database_write(&app_state, &_gate)?;
            // The `with_conn` closure must return Result<_, DbError> (Database's
            // contract). We carry the consent-required signal out via a SetActiveOutcome
            // so the outer closure can map it to the frontend-facing SetActiveResult
            // without smuggling a ConsentError through the DbError boundary.
            //
            // P1 #4: ALL reads (list, compute_scope, read_consent_scope) + the
            // write run inside ONE transaction so a concurrent writer can't change
            // the active set between validation and the slot write.
            let outcome = db
                .with_conn(|conn| -> Result<SetActiveOutcome, DbErr> {
                    let tx = conn.transaction()?;
                    // Validate against the active set BEFORE writing.
                    let active = db_providers::list(&tx)?;
                    db_providers::validate_active_selection(
                        &primary,
                        &parallel,
                        fallback.as_deref(),
                        &active,
                    )?;
                    // P1 #3: parallel consent gate. A non-empty parallel selection
                    // requires explicit user consent; if the stored scope doesn't
                    // match the recomputed scope, return ConsentRequired so the
                    // frontend can prompt. A matching scope (re-affirming the same
                    // set) is allowed through without re-prompting.
                    if !parallel.is_empty() {
                        let actual = db_providers::compute_scope(&primary, &parallel, &active)
                            .map_err(consent_to_db)?;
                        let stored = db_providers::read_consent_scope(&tx)?;
                        if stored.as_deref() != Some(actual.as_str()) {
                            // No write — drop the tx (rolls back, which is a no-op
                            // since nothing was written) and surface NeedsConsent.
                            return Ok(SetActiveOutcome::NeedsConsent {
                                actual_scope: actual,
                            });
                        }
                    }
                    // Scope matches (or parallel is empty → no consent needed):
                    // write the three slots. Clear prior consent only when there's
                    // no parallel set (membership went to a non-consented shape); a
                    // matching-scope write keeps the consent as-is.
                    if parallel.is_empty() {
                        set_active_slots(&tx, &primary, &parallel, fallback.as_deref())?;
                    } else {
                        set_active_slots_keep_consent(
                            &tx,
                            &primary,
                            &parallel,
                            fallback.as_deref(),
                        )?;
                    }
                    tx.commit()?;
                    Ok(SetActiveOutcome::Written)
                })
                .map_err(|e| e.to_string())?;
            // P1.1: map the internal outcome to the serialized tagged union. The
            // consent-required path is now an Ok(SetActiveResult::NeedsConsent) so
            // the frontend gets a structured payload, not a parsed error string.
            Ok(match outcome {
                SetActiveOutcome::Written => SetActiveResult::Written,
                SetActiveOutcome::NeedsConsent { actual_scope } => {
                    SetActiveResult::NeedsConsent { actual_scope }
                }
            })
        })
        .await
        .map_err(|e| e.to_string())??;
    // rev-7-8: refresh so the status item + submenu reflect the new primary.
    crate::refresh_tray_if_available(&app_handle).await;
    Ok(outcome)
}
/// popup/popup CTAs (Surface 02/03) so they can show a friendly engine label.
/// Returns the default (all-empty) selection if the DB is not ready yet.
#[tauri::command]
pub fn provider_get_active_selection(
    app_state: tauri::State<'_, Arc<AppState>>,
) -> Result<ActiveSelection, String> {
    let app = app_state.inner().clone();
    let _gate = app.data_gate.read();
    let db = require_database(&app, &_gate)?;
    db.with_conn(|conn| db_providers::read_active_selection(conn))
        .map_err(|e| e.to_string())
}

/// A4 + rev-5-4: the sync core of `provider_set_active`, callable from the
/// tray (which cannot resolve `tauri::State`). Sets `uuid` as the sole primary,
/// no parallel, no fallback. This is the BODY the tray handler runs inside a
/// `spawn_blocking` — do NOT wrap it in another `block_on(spawn_blocking(...))`.
/// Uses the real write helper `set_active_slots`; because `parallel` is empty,
/// the consent gate is never entered and the NeedsConsent branch is unreachable
/// (kept in the match for exhaustiveness; if it ever fires it maps through).
fn set_active_primary_core(
    app_state: Arc<AppState>,
    uuid: String,
    rev: u64,
) -> Result<SetActiveResult, String> {
    let app = app_state.clone();
    let outcome = db_set_active_primary(&app, &uuid, rev)?;
    Ok(match outcome {
        SetActiveOutcome::Written => SetActiveResult::Written,
        SetActiveOutcome::NeedsConsent { actual_scope } => {
            SetActiveResult::NeedsConsent { actual_scope }
        }
    })
}

/// rev-5-4: the gate + transaction that `set_active_primary_core` and the tray
/// share. Acquires the write gate, runs `validate_active_selection` + the
/// `set_active_slots` write inside ONE transaction. Returns the internal
/// `SetActiveOutcome` so the caller can map it to the serialized result.
///
/// P1-3 (Task A3): `pub` + revision-guarded. The caller passes the `rev`
/// captured via `tray.lock().begin_switch()`. This function checks `rev`
/// against `tray.switch_revision()` BEFORE acquiring the write gate AND
/// re-checks AFTER acquiring it, so a stale/late switch request (an older
/// click whose DB write lands after a newer click already committed) is
/// rejected at the DB level — guaranteeing last-click-wins.
pub fn db_set_active_primary(
    app: &Arc<AppState>,
    uuid: &str,
    rev: u64,
) -> Result<SetActiveOutcome, String> {
    // P1-3: check revision BEFORE acquiring the write gate. If a newer switch
    // already bumped the revision, this stale request must NOT write.
    {
        let controller = app.tray.lock();
        if controller.switch_revision() != rev {
            return Err(format!(
                "stale switch revision {rev} (current {})",
                controller.switch_revision()
            ));
        }
    }
    let _gate = app.data_gate.write();
    // Re-check after acquiring the gate: another switch may have bumped the
    // revision between the first check and the gate acquisition.
    {
        let controller = app.tray.lock();
        if controller.switch_revision() != rev {
            return Err(format!(
                "stale switch revision {rev} (current {})",
                controller.switch_revision()
            ));
        }
    }
    let db = require_database_write(app, &_gate)?;
    let outcome = db
        .with_conn(|conn| -> Result<SetActiveOutcome, DbErr> {
            let tx = conn.transaction()?;
            let active = db_providers::list(&tx)?;
            db_providers::validate_active_selection(uuid, &[], None, &active)?;
            // parallel is empty → set_active_slots (clears prior consent).
            set_active_slots(&tx, uuid, &[], None)?;
            tx.commit()?;
            Ok(SetActiveOutcome::Written)
        })
        .map_err(|e| e.to_string())?;
    Ok(outcome)
}

/// SYNC core of tray switch-provider: write primary, then finish_switch.
///
/// DB write and tray controller, NO AppHandle (the testable entry). Calls
/// `set_active_primary_core(...)` directly (SYNC), then
/// `tray.lock().finish_switch(rev, success)` (rev-16-3 revision-tagged; a stale
/// `rev != switch_revision` is ignored). The `rev` is captured via
/// `begin_switch()` BEFORE the DB call so a concurrent switch's late result
/// cannot clobber this one. R2-B (P1-3 residual): the revision is now allocated
/// by the SYNC menu callback (`handle_tray_menu_event`) BEFORE `spawn_blocking`
/// so revision order = click order regardless of OS thread scheduling; the
/// pre-allocated `rev` is passed in here (the core no longer calls
/// `begin_switch()` itself). Does NOT touch the translation `GenerationToken`
/// (rev-15 P1-3) and does NOT `.await` anything (SYNC).
pub fn handle_switch_provider_core(
    app_state: &Arc<AppState>,
    uuid: &str,
    rev: u64,
) -> Result<(), String> {
    let result = set_active_primary_core(app_state.clone(), uuid.to_string(), rev);
    let success = result.is_ok();
    app_state.tray.lock().finish_switch(rev, success);
    // P1-3 (Task A3): if the DB write failed because a newer switch already
    // bumped the revision, the newer click has already won — this is expected,
    // not an error. Swallow the stale-revision error so the caller sees Ok
    // (the DB already reflects the last click).
    if let Err(ref e) = result {
        if e.contains("stale switch revision") {
            return Ok(());
        }
    }
    result.map(|_| ())
}

/// A5 Step 10 (rev-18-1): the SYNC wrapper — calls the core + best-effort tray
/// refresh + failure tooltip. The tray.switch arm runs this via
/// `tauri::async_runtime::spawn_blocking` (offloads the SYNC SQLite I/O).
///
/// P1-2: this wrapper stays SYNC (it runs inside `spawn_blocking` and CANNOT
/// `.await`). The tray refresh is now async ([`refresh_tray_if_available`]), so
/// it is detached via `tauri::async_runtime::spawn` — the DB write commits
/// synchronously and is returned immediately; the best-effort tray refresh runs
/// as a fire-and-forget task on the runtime. The rev-19-5 ordering (refresh
/// FIRST, THEN override the tooltip on failure) is preserved inside the spawned
/// task. `result` is cloned into the task so the synchronous `Ok`/`Err` can be
/// returned to the caller.
pub fn handle_switch_provider(
    app: &tauri::AppHandle,
    app_state: &Arc<AppState>,
    uuid: &str,
    rev: u64,
) -> Result<(), String> {
    let result = handle_switch_provider_core(app_state, uuid, rev);
    let app_clone = app.clone();
    let result_for_refresh = result.clone();
    tauri::async_runtime::spawn(async move {
        match &result_for_refresh {
            Ok(_) => {
                crate::refresh_tray_if_available(&app_clone).await;
            }
            Err(msg) => {
                // rev-19-5: refresh FIRST (restores the pre-switch tooltip), THEN
                // override with the failure tooltip (rev-21-2: prefixed).
                crate::refresh_tray_if_available(&app_clone).await;
                if let Some(tray) = app_clone.tray_by_id("main-tray") {
                    let _ = tray.set_tooltip(Some(&format!("Switch failed: {msg}")));
                }
            }
        }
    });
    result
}

/// Confirm the user's explicit consent for a parallel selection and write it
/// atomically (P1 #3).
///
/// Single DB transaction that:
/// 1. Re-reads ALL active providers (inside the tx — no TOCTOU between the
///    `provider_set_active` probe and this confirm).
/// 2. Validates the candidate selection (`validate_active_selection`).
/// 3. Backend recomputes canonical scope via `compute_scope`.
/// 4. Asserts the frontend's `expected_scope` matches the backend's
///    `actual_scope` (rejects a stale frontend that raced a provider change).
/// 5. Writes the selection + consent scope + bumped version in the SAME tx.
#[tauri::command]
pub async fn provider_confirm_and_set_active(
    state: tauri::State<'_, Arc<AppState>>,
    app_handle: tauri::AppHandle,
    primary: String,
    parallel: Vec<String>,
    fallback: Option<String>,
    expected_scope: String,
) -> Result<i64, ProviderCommandError> {
    let app_state = state.inner().clone();
    let version =
        tauri::async_runtime::spawn_blocking(move || -> Result<i64, ProviderCommandError> {
            // Acquire the gate FIRST (see provider_list).
            let _gate = app_state.data_gate.write();
            let db =
                require_database_write(&app_state, &_gate).map_err(ProviderCommandError::from)?;
            let outcome = db.with_conn(|conn| -> Result<ConfirmActiveOutcome, DbErr> {
                let tx = conn.transaction()?;
                let active = db_providers::list(&tx)?;
                db_providers::validate_active_selection(
                    &primary,
                    &parallel,
                    fallback.as_deref(),
                    &active,
                )?;
                let actual_scope = db_providers::compute_scope(&primary, &parallel, &active)
                    .map_err(consent_to_db)?;
                if expected_scope != actual_scope {
                    // Stale frontend: the scope it asserts doesn't match what the
                    // backend recomputes (it raced a provider change). Carried out
                    // as a typed variant — no sentinel string to parse.
                    return Ok(ConfirmActiveOutcome::StaleScope { actual_scope });
                }
                let new_version = write_consented_selection(
                    &tx,
                    &primary,
                    &parallel,
                    fallback.as_deref(),
                    &actual_scope,
                )?;
                tx.commit()?;
                Ok(ConfirmActiveOutcome::Written {
                    version: new_version,
                })
            });
            // Map the typed outcome: StaleScope → ProviderCommandError::StaleScope
            // (structured wire error), Written → the consent version. Everything
            // else (real DB errors) stays an error.
            outcome
                .map(|o| match o {
                    ConfirmActiveOutcome::Written { version } => Ok(version),
                    ConfirmActiveOutcome::StaleScope { actual_scope } => {
                        Err(ProviderCommandError::StaleScope { actual_scope })
                    }
                })
                .map_err(ProviderCommandError::from)?
        })
        .await
        .map_err(|e| ProviderCommandError::Db {
            message: format!("{e:?}"),
        })??;
    // rev-8-8: refresh so the status item + submenu reflect the new primary.
    crate::refresh_tray_if_available(&app_handle).await;
    Ok(version)
}

/// One selectable model for a provider. Assembled from the local profile
/// (current model + catalog default) then extended by an HTTP GET to
/// `models_request_url` when the origin matches.
#[derive(Debug, Clone, Serialize)]
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
#[derive(Debug, Clone, Serialize)]
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
            // Acquire the gate FIRST (see provider_list).
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

/// User-initiated database recovery (P1 #8).
///
/// Thin wrapper around [`crate::db::recovery::archive_database_core`] (the
/// shared close/rename/reopen/migrate pipeline) with the production failpoint
/// ([`crate::db::recovery::ArchiveFailpoint::None`]). The core owns the whole
/// state machine so the production path and the recovery failpoint tests
/// exercise the SAME logic.
///
/// Pipeline (see the core for the full contract):
/// 1. PREFLIGHT `settings_path` BEFORE any destructive op (S2a P1). If the
///    path is `None`, refuse immediately — the DB is untouched and usable. This
///    avoids closing/renaming a working DB only to discover we can't migrate
///    because the settings path couldn't be resolved at startup.
/// 2. Acquire `data_gate.write()` (blocks every provider command).
/// 3. `Arc::try_unwrap` + `Database::close` — release the SQLite file handle.
/// 4. `fs::rename(db_path, broken_path)`.
/// 5. Open a fresh DB + run migration + `resume_deletions`.
/// 6. Install the new handle + `Ready`.
///
/// Any failure AFTER the rename leaves the slot `None` (or, for migration
/// failures, `Some` fresh DB) and a non-`Ready` readiness. Any failure BEFORE
/// the rename leaves the original DB untouched and usable.
#[tauri::command]
pub async fn archive_database(state: tauri::State<'_, Arc<AppState>>) -> Result<String, String> {
    let app = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || -> Result<String, String> {
        // Thin wrapper: delegate to the shared core with the production failpoint
        // (None). The core owns the close → rename → reopen → migrate → resume
        // pipeline + the settings-path preflight + the readiness transitions, so
        // the production path and the recovery failpoint tests exercise the SAME
        // logic (no drift).
        crate::db::recovery::archive_database_core(
            &app,
            crate::db::recovery::ArchiveFailpoint::None,
        )
    })
    .await
    .map_err(|e| e.to_string())?
}

// ─── provider command helpers ──────────────────────────────────────────────

/// Type alias so the closures above can name the error without importing the
/// full path each time.
type DbErr = crate::db::DbError;

/// Type alias for the consent-computation error (P1 #3).
type ConsentErr = db_providers::ConsentError;

/// Result of [`provider_set_active`] (P1.1). A serializable tagged union so the
/// frontend distinguishes "written" from "needs consent" via a structured
/// payload instead of parsing an error string.
///
/// Wire shapes:
/// - `Written` → `{"outcome":"written"}`
/// - `NeedsConsent { actual_scope }` → `{"outcome":"needs_consent","actual_scope":"..."}`
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
#[serde(tag = "outcome", rename_all = "snake_case")]
pub enum SetActiveResult {
    /// The selection was written (no consent needed, or scope already matched).
    Written,
    /// A non-empty parallel selection needs explicit consent; carries the
    /// canonical scope the frontend must echo back via
    /// `provider_confirm_and_set_active`.
    NeedsConsent { actual_scope: String },
}

/// Structured error type for provider IPC commands (P1.1 fix).
/// Replaces free-form `String` errors so the frontend can pattern-match
/// instead of parsing string prefixes.
/// Wire shape for StaleScope: `{"error":"stale_scope","actual_scope":"..."}`
/// Wire shape for StaleVersion (R2-E): `{"error":"stale_version","actual_version":N}`
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
#[serde(tag = "error", rename_all = "snake_case")]
pub enum ProviderCommandError {
    StaleScope {
        actual_scope: String,
    },
    /// Optimistic-lock mismatch (R2-E): the provider row was modified elsewhere
    /// since the frontend last read it. Carries the row's actual version so the
    /// UI can prompt a reload.
    StaleVersion {
        actual_version: i64,
    },
    /// Generic database or validation error.
    Db {
        message: String,
    },
    /// Provider not found, invalid selection, etc.
    Validation {
        message: String,
    },
}

impl std::fmt::Display for ProviderCommandError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::StaleScope { actual_scope } => {
                write!(f, "stale scope: {actual_scope}")
            }
            Self::StaleVersion { actual_version } => {
                write!(f, "stale version: row is at version {actual_version}")
            }
            Self::Db { message } => write!(f, "{message}"),
            Self::Validation { message } => write!(f, "{message}"),
        }
    }
}

impl From<crate::db::DbError> for ProviderCommandError {
    fn from(e: crate::db::DbError) -> Self {
        ProviderCommandError::Db {
            message: e.to_string(),
        }
    }
}

impl From<String> for ProviderCommandError {
    fn from(message: String) -> Self {
        ProviderCommandError::Validation { message }
    }
}

/// Outcome of a `provider_set_active` DB transaction (P1 #3). Carries the
/// consent-required signal out of the `with_conn` closure (whose error type is
/// fixed to `DbError`) so the command can surface it to the frontend.
///
/// `pub` + `Debug` (Task A3 / P1-3): exposed so the integration test for the
/// revision-guarded `db_set_active_primary` can name and inspect the return.
#[derive(Debug)]
pub enum SetActiveOutcome {
    /// Selection written (no consent needed, or scope already matched).
    Written,
    /// A non-empty parallel selection needs explicit consent; carries the
    /// canonical scope the frontend must echo back via
    /// `provider_confirm_and_set_active`.
    NeedsConsent { actual_scope: String },
}

/// Outcome of a `provider_confirm_and_set_active` DB transaction (round-3
/// cleanup #1). Replaces the old `__stale_scope__:` string sentinel smuggled
/// through `DbError::Integrity`: the stale-scope signal now rides out of the
/// `with_conn` closure as a first-class variant, so the outer mapping is a
/// plain `match` with no string-prefix parsing to get wrong.
enum ConfirmActiveOutcome {
    /// Consent written; carries the new `parallel_consent_version`.
    Written { version: i64 },
    /// The frontend's `expected_scope` didn't match the backend-recomputed
    /// canonical scope (it raced a provider change). Carries the actual scope
    /// the frontend must re-echo.
    StaleScope { actual_scope: String },
}

/// Map a [`ConsentError`] (other than `ConsentRequired`, which is handled by
/// the caller via `SetActiveOutcome`) into a [`DbError`] so it can cross the
/// `with_conn` boundary. `ConsentRequired` is only ever surfaced by
/// `provider_set_active` (as `SetActiveOutcome::NeedsConsent`), never by
/// `provider_confirm_and_set_active` (whose stale-scope path is now a typed
/// `ConfirmActiveOutcome` variant, not an error string).
fn consent_to_db(e: ConsentErr) -> DbErr {
    match e {
        ConsentErr::Db(d) => d,
        other => DbErr::Integrity(other.to_string()),
    }
}

/// Write the primary/parallel/fallback slots in `preferences` + null consent.
/// Runs against the caller's transaction (P1 #4: reads + writes in ONE tx) —
/// no inner transaction is opened.
fn set_active_slots(
    tx: &rusqlite::Transaction<'_>,
    primary: &str,
    parallel: &[String],
    fallback: Option<&str>,
) -> Result<(), DbErr> {
    let primary_val = if primary.is_empty() {
        None
    } else {
        Some(primary)
    };
    let parallel_json = serde_json::to_string(parallel).unwrap_or_else(|_| "[]".into());
    let fallback_val = fallback.filter(|s| !s.is_empty());
    tx.execute(
        "UPDATE preferences SET primary_uuid=?1, parallel_uuids=?2, fallback_uuid=?3, \
         parallel_consent_version=NULL, parallel_consent_scope=NULL WHERE id=1",
        rusqlite::params![primary_val, parallel_json, fallback_val,],
    )?;
    Ok(())
}

/// Like [`set_active_slots`] but PRESERVES the prior parallel consent
/// (version + scope). Used by `provider_set_active` when the recomputed scope
/// matches the stored scope (re-affirming the same selection): we update the
/// slot pointers without invalidating consent. Runs against the caller's
/// transaction (P1 #4) — no inner transaction is opened.
fn set_active_slots_keep_consent(
    tx: &rusqlite::Transaction<'_>,
    primary: &str,
    parallel: &[String],
    fallback: Option<&str>,
) -> Result<(), DbErr> {
    let primary_val = if primary.is_empty() {
        None
    } else {
        Some(primary)
    };
    let parallel_json = serde_json::to_string(parallel).unwrap_or_else(|_| "[]".into());
    let fallback_val = fallback.filter(|s| !s.is_empty());
    tx.execute(
        "UPDATE preferences SET primary_uuid=?1, parallel_uuids=?2, fallback_uuid=?3 WHERE id=1",
        rusqlite::params![primary_val, parallel_json, fallback_val],
    )?;
    Ok(())
}

/// Write the active selection AND record the consent (scope + bumped version)
/// against the caller's transaction (P1 #3 + P1 #4: ALL reads + writes in ONE
/// tx). Returns the new consent version. The caller owns the transaction and
/// commits it.
fn write_consented_selection(
    tx: &rusqlite::Transaction<'_>,
    primary: &str,
    parallel: &[String],
    fallback: Option<&str>,
    scope: &str,
) -> Result<i64, DbErr> {
    let primary_val = if primary.is_empty() {
        None
    } else {
        Some(primary)
    };
    let parallel_json = serde_json::to_string(parallel).unwrap_or_else(|_| "[]".into());
    let fallback_val = fallback.filter(|s| !s.is_empty());
    // Bump the version: COALESCE(NULL, 0) + 1 so the first consent is version 1.
    tx.execute(
        "UPDATE preferences SET primary_uuid=?1, parallel_uuids=?2, fallback_uuid=?3, \
         parallel_consent_version=COALESCE(parallel_consent_version, 0) + 1, \
         parallel_consent_scope=?4 WHERE id=1",
        rusqlite::params![primary_val, parallel_json, fallback_val, scope],
    )?;
    let new_version: i64 = tx.query_row(
        "SELECT parallel_consent_version FROM preferences WHERE id=1",
        [],
        |r| r.get(0),
    )?;
    Ok(new_version)
}
