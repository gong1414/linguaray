use crate::db::schema;
use crate::onboarding::{self, OnboardingEvent, OnboardingStep};
use crate::{require_database, require_database_write, AppState};
use rusqlite::params;
use std::sync::Arc;

#[tauri::command]
#[specta::specta]
pub async fn onboarding_status(
    state: tauri::State<'_, Arc<AppState>>,
) -> Result<OnboardingStatus, String> {
    let app = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        let gate = app.data_gate.read();
        let db = require_database(&app, &gate)?;
        db.with_conn(|conn| {
            schema::ensure_preference_columns(conn)?;
            let (complete, step): (i64, String) = conn.query_row(
                "SELECT onboarding_complete, onboarding_step FROM preferences WHERE id=1",
                [],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )?;
            // Tolerate a stale/invalid stored step (e.g. hand-edited DB):
            // fall back to Welcome rather than failing onboarding outright.
            let step: OnboardingStep = serde_json::from_str(&format!("\"{step}\""))
                .unwrap_or(OnboardingStep::Welcome);
            Ok(OnboardingStatus {
                complete: complete != 0,
                step: if complete != 0 {
                    OnboardingStep::Done
                } else {
                    step
                },
            })
        })
        .map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}

#[derive(serde::Serialize, specta::Type)]
pub struct OnboardingStatus {
    pub complete: bool,
    pub step: OnboardingStep,
}

/// Pure state-machine transition + persistence: the DB is the step source of
/// truth, so a mid-flow window close resumes at the persisted step (R6).
/// The pure `onboarding::next_step` stays separately unit-tested.
#[tauri::command]
#[specta::specta]
pub async fn onboarding_next(
    state: tauri::State<'_, Arc<AppState>>,
    step: OnboardingStep,
    event: OnboardingEvent,
) -> Result<OnboardingStep, String> {
    let next = onboarding::next_step(step, event);
    let app = state.inner().clone();
    let to_persist = next;
    tauri::async_runtime::spawn_blocking(move || persist_step(&app, to_persist))
        .await
        .map_err(|e| e.to_string())??;
    Ok(next)
}

fn persist_step(app: &Arc<AppState>, step: OnboardingStep) -> Result<(), String> {
    let gate = app.data_gate.write();
    let db = require_database_write(app, &gate)?;
    let json = serde_json::to_string(&step).map_err(|e| e.to_string())?;
    db.with_conn(|conn| {
        schema::ensure_preference_columns(conn)?;
        conn.execute(
            "UPDATE preferences SET onboarding_step=?1 WHERE id=1",
            params![json.trim_matches('"')],
        )?;
        Ok(())
    })
    .map_err(|e| e.to_string())
}

#[tauri::command]
#[specta::specta]
pub async fn onboarding_complete(
    state: tauri::State<'_, Arc<AppState>>,
) -> Result<(), String> {
    let app = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        let gate = app.data_gate.write();
        let db = require_database_write(&app, &gate)?;
        db.with_conn(|conn| {
            schema::ensure_preference_columns(conn)?;
            conn.execute(
                "UPDATE preferences SET onboarding_complete=1, onboarding_step='done' WHERE id=1",
                [],
            )?;
            Ok(())
        })
        .map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}
