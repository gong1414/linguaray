//! OCR IPC. `ocr_capture` starts a capture session:
//!
//! - **macOS** — the SYSTEM region picker (`screencapture -i -x`, pot-desktop's
//!   approach): no WebView overlay window exists at all, so none can leak at
//!   startup. Cancel (Esc) is silent; a successful pick is OCR'd and fed into
//!   the standard selection-translate pipeline (cursor-anchored popup).
//! - **Windows** — the region-select overlay is created ON DEMAND here. The
//!   pre-declared config window was removed: a hidden pre-created WebView
//!   still boots ocr.html at startup (tauri#10950) and shipped builds
//!   occasionally surfaced it as a stray normal window. The overlay destroys
//!   itself when the user finishes or cancels.

use crate::ocr::{self, OcrResult};
use crate::{AppState, Session};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

#[tauri::command]
#[specta::specta]
pub fn ocr_recognize_bytes(bytes: Vec<u8>) -> Result<OcrResult, String> {
    ocr::recognize_image_bytes(&bytes).map_err(|e| e.to_string())
}

#[tauri::command]
#[specta::specta]
pub fn ocr_from_image(path: String) -> Result<OcrResult, String> {
    let bytes = std::fs::read(&path).map_err(|e| e.to_string())?;
    ocr::recognize_image_bytes(&bytes).map_err(|e| e.to_string())
}

#[tauri::command]
#[specta::specta]
pub fn ocr_from_clipboard() -> Result<OcrResult, String> {
    ocr::recognize_clipboard_image().map_err(|e| e.to_string())
}

#[tauri::command]
#[specta::specta]
pub fn ocr_capture_region(x: i32, y: i32, width: i32, height: i32) -> Result<OcrResult, String> {
    ocr::capture_region_and_recognize(x, y, width, height).map_err(|e| e.to_string())
}

/// Single-flight capture-session guard: a second trigger while a session is
/// live (modal system picker on macOS, open overlay on Windows) must never
/// stack a second one. Drop-released, so a panic inside the pipeline cannot
/// wedge the flag at "already active".
pub(crate) struct CaptureGuard<'a>(&'a AtomicBool);

impl<'a> CaptureGuard<'a> {
    pub(crate) fn acquire(flag: &'a AtomicBool) -> Result<Self, &'static str> {
        if flag.swap(true, Ordering::SeqCst) {
            Err("an OCR capture session is already active")
        } else {
            Ok(Self(flag))
        }
    }
}

impl Drop for CaptureGuard<'_> {
    fn drop(&mut self) {
        self.0.store(false, Ordering::SeqCst);
    }
}

static CAPTURE_IN_FLIGHT: AtomicBool = AtomicBool::new(false);

/// Start an OCR capture session. Tray menu and the OCR hotkey call this.
/// `source` records the trigger origin so a stray overlay in the logs can be
/// attributed (tray / shortcut / startup anomaly).
#[tauri::command]
#[specta::specta]
pub async fn ocr_capture(
    app: tauri::AppHandle,
    state: tauri::State<'_, Arc<Session>>,
    app_state: tauri::State<'_, Arc<AppState>>,
    source: Option<String>,
) -> Result<(), String> {
    let source = source.unwrap_or_else(|| "unknown".into());
    log::info!("ocr_capture start (source: {source})");
    let _guard = CaptureGuard::acquire(&CAPTURE_IN_FLIGHT).map_err(|e| {
        log::warn!("ocr_capture rejected (source: {source}): {e}");
        e.to_string()
    })?;
    let session = state.inner().clone();
    let app_state = app_state.inner().clone();
    let outcome = run_capture(&app, &session, &app_state).await;
    if let Err(e) = &outcome {
        log::error!("ocr_capture failed (source: {source}): {e}");
    }
    outcome
}

#[cfg(target_os = "macos")]
async fn run_capture(
    app: &tauri::AppHandle,
    session: &Arc<Session>,
    app_state: &Arc<AppState>,
) -> Result<(), String> {
    let path = std::env::temp_dir().join(format!(
        "linguaray-ocr-pick-{}-{}.png",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis())
            .unwrap_or(0)
    ));
    // Modal system picker — run it OFF the async runtime thread.
    let picker_path = path.clone();
    let status = tauri::async_runtime::spawn_blocking(move || {
        std::process::Command::new("/usr/sbin/screencapture")
            .args(["-i", "-x"])
            .arg(&picker_path)
            .status()
    })
    .await
    .map_err(|e| format!("screencapture spawn: {e}"))?
    .map_err(|e| format!("screencapture: {e}"))?;

    let bytes = std::fs::read(&path).unwrap_or_default();
    let _ = std::fs::remove_file(&path);
    // Esc / empty selection: nonzero exit or no file content. Silent cancel —
    // no error popup, no result window (acceptance: "user cancel is quiet").
    if !status.success() || bytes.is_empty() {
        log::info!(
            "ocr_capture cancelled (screencapture success={}, bytes={})",
            status.success(),
            bytes.len()
        );
        return Ok(());
    }
    let result = ocr::recognize_image_bytes(&bytes).map_err(|e| e.to_string())?;
    if result.text.trim().is_empty() {
        log::info!("ocr_capture: region contained no text");
        return Ok(());
    }
    // Same pipeline the Retry path uses: popup at the cursor + a normal
    // translation session (errors surface in the popup, not a stray dialog).
    let gen = session.gen.next();
    crate::commands::translate::capture_and_translate(
        app,
        session,
        app_state,
        Some(result.text),
        None,
        None,
        gen,
    )
    .await;
    Ok(())
}

/// Build (or surface) the on-demand OCR overlay window. Shared by the Windows
/// capture path and the LINGUARAY_AUTOSHOW_OCR testability hook.
pub(crate) async fn ensure_overlay_window(app: &tauri::AppHandle) -> Result<(), String> {
    use tauri::{Manager, WebviewUrl, WebviewWindowBuilder};
    if let Some(w) = app.get_webview_window("ocr") {
        // Overlay from a previous trigger still open — bring it forward
        // instead of stacking a second one.
        let _ = w.show();
        let _ = w.set_focus();
        return Ok(());
    }
    let _win = WebviewWindowBuilder::new(app, "ocr", WebviewUrl::App("ocr.html".into()))
        .title("")
        .fullscreen(true)
        .decorations(false)
        .transparent(true)
        .always_on_top(true)
        .skip_taskbar(true)
        .resizable(false)
        .focused(true)
        // Built hidden; the overlay shows itself once its DOM is ready, so a
        // cold WebView never flashes white/gray before content exists.
        .visible(false)
        .build()
        .map_err(|e| format!("ocr overlay build: {e}"))?;
    Ok(())
}

#[cfg(target_os = "windows")]
async fn run_capture(
    app: &tauri::AppHandle,
    _session: &Arc<Session>,
    _app_state: &Arc<AppState>,
) -> Result<(), String> {
    ensure_overlay_window(app).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn capture_guard_rejects_second_session_and_resets_on_drop() {
        let flag = AtomicBool::new(false);
        let g1 = CaptureGuard::acquire(&flag);
        assert!(g1.is_ok());
        assert!(
            CaptureGuard::acquire(&flag).is_err(),
            "second acquire must be rejected"
        );
        drop(g1);
        assert!(CaptureGuard::acquire(&flag).is_ok(), "flag resets after drop");
    }
}
