use crate::shortcuts::{ShortcutAction, ShortcutController, ShortcutError, ShortcutSnapshot};
use std::sync::Arc;

fn shortcut_join_error(error: tauri::Error) -> ShortcutError {
    ShortcutError::DatabaseFailed {
        message: format!("shortcut worker join failed: {error}"),
    }
}

#[tauri::command]
pub async fn shortcut_list(
    state: tauri::State<'_, Arc<ShortcutController>>,
) -> Result<ShortcutSnapshot, ShortcutError> {
    let controller = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || controller.snapshot())
        .await
        .map_err(shortcut_join_error)?
}

#[tauri::command]
pub async fn shortcut_check_conflict(
    state: tauri::State<'_, Arc<ShortcutController>>,
    action: ShortcutAction,
    combo: String,
    revision: u64,
) -> Result<Option<ShortcutAction>, ShortcutError> {
    let controller = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        controller.check_conflict(action, &combo, revision)
    })
    .await
    .map_err(shortcut_join_error)?
}

#[tauri::command]
pub async fn shortcut_save(
    state: tauri::State<'_, Arc<ShortcutController>>,
    action: ShortcutAction,
    combo: String,
    expected_revision: u64,
    override_action: Option<ShortcutAction>,
) -> Result<ShortcutSnapshot, ShortcutError> {
    let controller = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        controller.save(action, &combo, expected_revision, override_action)
    })
    .await
    .map_err(shortcut_join_error)?
}

#[tauri::command]
pub async fn shortcut_reset_defaults(
    state: tauri::State<'_, Arc<ShortcutController>>,
    expected_revision: u64,
) -> Result<ShortcutSnapshot, ShortcutError> {
    let controller = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || controller.reset_defaults(expected_revision))
        .await
        .map_err(shortcut_join_error)?
}

#[tauri::command]
pub fn shortcut_recording_begin(
    state: tauri::State<'_, Arc<ShortcutController>>,
    action: ShortcutAction,
) -> Result<(), ShortcutError> {
    state.recording_begin(action)
}

#[tauri::command]
pub fn shortcut_recording_end(state: tauri::State<'_, Arc<ShortcutController>>) {
    state.recording_end();
}
