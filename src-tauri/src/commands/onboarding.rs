use crate::db::schema;
use crate::onboarding::{self, OnboardingEvent, OnboardingStep};
use crate::{require_database, require_database_write, AppState};
use std::sync::Arc;

#[tauri::command]
pub async fn onboarding_status(
    state: tauri::State<'_, Arc<AppState>>,
) -> Result<OnboardingStatus, String> {
    let app = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        let gate = app.data_gate.read();
        let db = require_database(&app, &gate)?;
        db.with_conn(|conn| {
            schema::ensure_preference_columns(conn)?;
            let complete: i64 = conn.query_row(
                "SELECT onboarding_complete FROM preferences WHERE id=1",
                [],
                |r| r.get(0),
            )?;
            Ok(OnboardingStatus {
                complete: complete != 0,
                step: if complete != 0 {
                    OnboardingStep::Done
                } else {
                    OnboardingStep::Welcome
                },
            })
        })
        .map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}

#[derive(serde::Serialize)]
pub struct OnboardingStatus {
    pub complete: bool,
    pub step: OnboardingStep,
}

#[tauri::command]
pub fn onboarding_next(step: OnboardingStep, event: OnboardingEvent) -> OnboardingStep {
    onboarding::next_step(step, event)
}

#[tauri::command]
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
                "UPDATE preferences SET onboarding_complete=1 WHERE id=1",
                [],
            )?;
            Ok(())
        })
        .map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}
