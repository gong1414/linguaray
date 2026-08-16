//! Updater IPC. `updater_check` runs the remote check and mirrors the result
//! onto the tray dot; `updater_download_install` performs the full
//! check→download→install pipeline with a single-flight guard and throttled
//! `updater-progress` events.

use crate::updater::{self, UpdateCheck};
use crate::AppState;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tauri::{Emitter, Manager};

#[tauri::command]
#[specta::specta]
pub async fn updater_check(app: tauri::AppHandle) -> UpdateCheck {
    let state: Arc<AppState> = app.state::<Arc<AppState>>().inner().clone();
    let check = updater::check_remote(&app).await;
    state
        .tray
        .lock()
        .set_update_available(updater::tray_should_show_update(&check));
    check
}

/// Drops reset the in-flight flag, so a panic or early `?` return inside the
/// install pipeline can never wedge the guard at "already in progress".
struct InstallGuard<'a>(&'a AtomicBool);

impl<'a> InstallGuard<'a> {
    fn acquire(flag: &'a AtomicBool) -> Result<Self, ()> {
        if flag.swap(true, Ordering::SeqCst) {
            Err(())
        } else {
            Ok(Self(flag))
        }
    }
}

impl Drop for InstallGuard<'_> {
    fn drop(&mut self) {
        self.0.store(false, Ordering::SeqCst);
    }
}

#[tauri::command]
#[specta::specta]
pub async fn updater_download_install(
    app: tauri::AppHandle,
) -> Result<UpdateCheck, String> {
    let state: Arc<AppState> = app.state::<Arc<AppState>>().inner().clone();
    let _guard = InstallGuard::acquire(&state.update_install_in_flight)
        .map_err(|_| "an update download/install is already in progress".to_string())?;

    use tauri_plugin_updater::UpdaterExt;
    let updater = app
        .updater()
        .map_err(|e| format!("updater unavailable: {e}"))?;
    let update = updater
        .check()
        .await
        .map_err(|e| format!("update check failed: {e}"))?;
    let Some(update) = update else {
        // Re-check found nothing new (e.g. the release was promoted between the
        // UI's check and this call). Clear the tray dot — it is no longer true.
        let check = UpdateCheck::UpToDate {
            version: env!("CARGO_PKG_VERSION").to_string(),
        };
        state.tray.lock().set_update_available(false);
        return Ok(check);
    };
    let current = update.current_version.to_string();
    let next = update.version.to_string();
    let notes = update.body.clone().unwrap_or_default();

    // One webview event per whole percent (per MiB when the total is unknown).
    let mut downloaded: u64 = 0;
    let mut last_bucket: Option<u64> = None;
    let progress_app = app.clone();
    let on_chunk = move |chunk: usize, total: Option<u64>| {
        downloaded += chunk as u64;
        let bucket = updater::progress_bucket(downloaded, total);
        if last_bucket != Some(bucket) {
            last_bucket = Some(bucket);
            let _ = progress_app.emit(
                "updater-progress",
                serde_json::json!({ "downloaded": downloaded, "total": total, "bucket": bucket }),
            );
        }
    };
    let finish_app = app.clone();
    let on_download_finish = move || {
        // Fires after the download, before the installer takes over. The
        // webview uses it to switch its label from "downloading" to
        // "installing"; on Windows the process exits during install and this
        // is the last event it will ever see.
        let _ = finish_app.emit(
            "updater-progress",
            serde_json::json!({ "finished": true }),
        );
    };
    update
        .download_and_install(on_chunk, on_download_finish)
        .await
        .map_err(|e| format!("download/install failed: {e}"))?;
    // Reached on macOS/Linux (app bundle replaced in place; the webview then
    // relaunches via the process plugin). On Windows the NSIS installer exited
    // the process inside download_and_install.
    Ok(UpdateCheck::Available { current, next, notes })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn install_guard_rejects_second_acquirer_and_resets_on_drop() {
        let flag = AtomicBool::new(false);
        let g1 = InstallGuard::acquire(&flag);
        assert!(g1.is_ok());
        assert!(InstallGuard::acquire(&flag).is_err(), "second acquire must be rejected");
        drop(g1);
        assert!(InstallGuard::acquire(&flag).is_ok(), "flag must reset after drop");
    }
}
