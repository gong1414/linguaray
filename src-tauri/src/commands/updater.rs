use crate::updater::{self, UpdateCheck};
use crate::AppState;
use std::sync::Arc;

#[tauri::command]
pub fn updater_check(state: tauri::State<'_, Arc<AppState>>) -> UpdateCheck {
    let check = updater::check_current();
    state
        .tray
        .lock()
        .set_update_available(updater::tray_should_show_update(&check));
    check
}
