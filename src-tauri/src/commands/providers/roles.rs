//! Active-selection / consent / tray-switch IPC. Owns the revision-guarded
//! switch core and the consent transaction helpers.

use crate::db::providers::{self as db_providers, ActiveSelection};
use crate::{require_database, require_database_write, AppState};
use std::sync::Arc;

use super::{ProviderCommandError, SetActiveResult};

/// Type alias so the closures above can name the error without importing the
/// full path each time.
type DbErr = crate::db::DbError;

/// Type alias for the consent-computation error (P1 #3).
type ConsentErr = db_providers::ConsentError;

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
#[specta::specta]
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
            // Acquire the gate FIRST (see provider_list in crud.rs).
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
#[specta::specta]
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
#[specta::specta]
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
            // Acquire the gate FIRST (see provider_list in crud.rs).
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

// ─── consent transaction helpers ────────────────────────────────────────────

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
