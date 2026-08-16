//! Translate IPC commands and session helpers (plugin-core PR-3).

use crate::db::Database;
use crate::plugins::clipboard::{ClipboardHub, CLIPBOARD};
use crate::plugins::selection::{SelectionHub, SELECTION};
use crate::plugins::translation::{PersistSpec, TranslationHub, TRANSLATION};
use crate::service::{self, TranslateSessionResult};
use crate::{
    a11y, cursor, engines, popup, require_database, selection_engine, settings,
    tray_state, AppState, Session,
};
use serde::Deserialize;
use std::sync::Arc;
use tauri::Manager;

fn map_lease(err: impl std::fmt::Display) -> String {
    let text = err.to_string();
    if text.contains("unloaded") {
        "HTTP client unavailable: startup build failed (recovery required)".into()
    } else {
        text
    }
}

async fn with_translation<F, Fut, R>(app: &tauri::AppHandle, f: F) -> Result<R, String>
where
    F: FnOnce(TranslationHub) -> Fut,
    Fut: std::future::Future<Output = Result<R, String>>,
{
    let supervisor = app
        .try_state::<linguaray_kernel::Supervisor>()
        .ok_or_else(|| {
            "HTTP client unavailable: startup build failed (recovery required)".to_string()
        })?;
    let lease = supervisor.handle().lease(TRANSLATION).map_err(map_lease)?;
    lease
        .call(|tx| {
            let tx = tx.clone();
            async move { f(tx).await }
        })
        .await
        .map_err(map_lease)?
}

fn lease_clipboard(
    app: &tauri::AppHandle,
) -> Result<linguaray_kernel::ServiceLease<ClipboardHub>, String> {
    let supervisor = app
        .try_state::<linguaray_kernel::Supervisor>()
        .ok_or_else(|| "clipboard unavailable".to_string())?;
    supervisor.handle().lease(CLIPBOARD).map_err(map_lease)
}

fn lease_selection(
    app: &tauri::AppHandle,
) -> Result<linguaray_kernel::ServiceLease<SelectionHub>, String> {
    let supervisor = app
        .try_state::<linguaray_kernel::Supervisor>()
        .ok_or_else(|| "selection unavailable".to_string())?;
    supervisor.handle().lease(SELECTION).map_err(map_lease)
}

/// Translate the clipboard contents ONCE, on user demand.
///
/// Reads the clipboard exactly once and surfaces the result in the popup window at
/// the current cursor position. Deliberately NOT a background listener — there is
/// no clipboard-changed subscription anywhere in the app (spec §Scope: user-initiated).
#[tauri::command]
#[specta::specta]
pub async fn translate_clipboard(
    app: tauri::AppHandle,
    state: tauri::State<'_, Arc<Session>>,
    app_state: tauri::State<'_, Arc<AppState>>,
) -> Result<(), String> {
    // latest-wins：先分配 gen（同步，保证 press 顺序）。
    let gen = state.gen.next();
    let text = {
        let _g = state.gen.selection_lock();
        lease_clipboard(&app)?
            .with(|cb| cb.get_text())
            .map_err(map_lease)??
    };
    if text.trim().is_empty() {
        return Err("clipboard empty".into());
    }
    let (x, y) = cursor::position();
    // B4: switch to the source-aware emitters so clipboard-origin results carry
    // `source_text` (P1-3: Retry needs the original text). Mirrors
    // capture_and_translate: build_popup_anchor + gen check + loading_with_source.
    let anchor = match build_popup_anchor(&app, x as f64, y as f64) {
        Some(a) => a,
        None => return Ok(()),
    };
    if !state.gen.is_latest(gen) {
        return Ok(());
    }
    let _ = popup::loading_with_source(&app, &anchor, Some(&text));
    let s = settings::load(&app);

    // Task A5: the tray Active-pulse begins here (after the clipboard preflight).
    // TranslationGuard::new calls begin_translation(gen); its Drop calls
    // finish_translation(gen, succeeded) on every return path. On a success branch
    // we call guard.mark_success(); on an error branch we call
    // record_translation_error(gen) before the guard drops.
    let mut _tray_guard = tray_state::TranslationGuard::new(&app_state.tray, gen);

    let app_arc = app_state.inner().clone();
    let db = match tauri::async_runtime::spawn_blocking(move || -> Result<Arc<Database>, String> {
        let _gate = app_arc.data_gate.read();
        require_database(&app_arc, &_gate)
    })
    .await
    {
        Ok(Ok(db)) => db,
        Ok(Err(msg)) => {
            if state.gen.is_latest(gen) {
                app_state.tray.lock().record_translation_error(gen);
                let _ = popup::set_popup_mode(&app, popup::PopupMode::Error, &anchor);
                let _ = popup::error_with_source(&app, &msg, &text);
            }
            return Ok(());
        }
        Err(e) => {
            if state.gen.is_latest(gen) {
                app_state.tray.lock().record_translation_error(gen);
                let _ = popup::set_popup_mode(&app, popup::PopupMode::Error, &anchor);
                let _ = popup::error_with_source(&app, &format!("join error: {e}"), &text);
            }
            return Ok(());
        }
    };

    let session_result = run_translate_session(
        &app,
        &db,
        &text,
        "auto",
        &s.target_language,
        "clipboard",
        Some((&state.gen, gen)),
    )
    .await;

    // latest-wins：完成后检查 gen 才发事件。
    if !state.gen.is_latest(gen) {
        return Ok(());
    }
    match session_result {
        Ok(r) => match decide_clipboard_popup(&r) {
            ClipboardPopupDecision::SingleSuccess { text: t, engine } => {
                _tray_guard.mark_success();
                let _ = popup::set_popup_mode(&app, popup::PopupMode::Single, &anchor);
                let _ = popup::result_with_source(&app, &t, &engine, &text);
            }
            ClipboardPopupDecision::Multi => {
                _tray_guard.mark_success();
                let _ = popup::set_popup_mode(&app, popup::PopupMode::Multi, &anchor);
                let _ = popup::multi_result_with_source(&app, &r.outcomes, &text);
            }
            ClipboardPopupDecision::Error(msg) => {
                app_state.tray.lock().record_translation_error(gen);
                let _ = popup::set_popup_mode(&app, popup::PopupMode::Error, &anchor);
                let _ = popup::error_with_source(&app, &msg, &text);
            }
        },
        Err(msg) => {
            app_state.tray.lock().record_translation_error(gen);
            let _ = popup::set_popup_mode(&app, popup::PopupMode::Error, &anchor);
            let _ = popup::error_with_source(&app, &msg, &text);
        }
    }
    Ok(())
}

// ─── R2a: translate_session ──────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize, specta::Type)]
pub struct TranslateSessionRequest {
    pub text: String,
    pub from: String,
    pub to: String,
}

/// Frontend-facing session result. The service keeps Rust's `Result` internally;
/// the IPC contract flattens each outcome into stable `ok/text/error` fields.
#[derive(serde::Serialize, specta::Type)]
pub struct TranslateSessionWire {
    pub outcomes: Vec<popup::TranslationOutcomeSerialized>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub actual_engine: Option<String>,
}

impl From<TranslateSessionResult> for TranslateSessionWire {
    fn from(result: TranslateSessionResult) -> Self {
        Self {
            outcomes: result
                .outcomes
                .iter()
                .map(popup::TranslationOutcomeSerialized::from)
                .collect(),
            actual_engine: result.actual_engine,
        }
    }
}

/// translate_clipboard 根据翻译结果决定发哪种 popup 事件的纯函数决策。
/// 抽出来便于测试（translate_clipboard 本身依赖 Tauri runtime 不可单测）。
#[derive(Debug)]
pub enum ClipboardPopupDecision {
    /// 单引擎成功 → 走老 popup-state 事件（向后兼容）。
    SingleSuccess { text: String, engine: String },
    /// 并行（含部分成功）→ 走 popup-multi-result 事件。
    Multi,
    /// 单引擎失败 / 并行全失败 / 核心错误 → 走 popup-state error。
    Error(String),
}

pub fn decide_clipboard_popup(result: &TranslateSessionResult) -> ClipboardPopupDecision {
    if result.outcomes.is_empty() {
        return ClipboardPopupDecision::Error("translation produced no outcomes".into());
    }
    // 单引擎路径：actual_engine=Some 表示成功。
    if let Some(engine) = &result.actual_engine {
        // 长度必为 1（run_translate_session 单引擎路径契约）。
        if let Some(o) = result.outcomes.first() {
            if let Ok(t) = &o.result {
                return ClipboardPopupDecision::SingleSuccess {
                    text: t.text.clone(),
                    engine: engine.clone(),
                };
            }
        }
        // actual_engine=Some 但 outcome 失败（理论不应发生）→ 当错误处理。
        return ClipboardPopupDecision::Error("single engine failed unexpectedly".into());
    }
    // actual_engine=None：并行路径。
    if result.outcomes.len() == 1 {
        // 退化单引擎但失败。
        if let Some(err) = result
            .outcomes
            .first()
            .and_then(|o| o.result.as_ref().err())
        {
            return ClipboardPopupDecision::Error(err.to_string());
        }
    }
    // 并行全失败？
    let all_failed = result.outcomes.iter().all(|o| o.result.is_err());
    if all_failed {
        return ClipboardPopupDecision::Error("all engines failed".into());
    }
    // 并行（含部分成功）。
    ClipboardPopupDecision::Multi
}

/// Central sentinel resolver: the frontend passes `to: ""` to mean "use the
/// stored target language". Exposed so the hotkey contract is locked by an
/// integration test.
pub fn resolve_target_language(to: &str, settings_target: &str) -> String {
    if to.is_empty() {
        settings_target.to_string()
    } else {
        to.to_string()
    }
}

/// 翻译会话核心逻辑（纯函数，无 Tauri State 依赖）。
///
/// 被两个入口共享：
/// - `translate_session` 命令（带 AppHandle，从 settings 读 fallback_engine）
/// - `translate_clipboard`（同上）
/// - 测试用 `run_translate_session_no_settings`（fallback=None，避开 settings）
///
/// 流程见 plan Task 4：read_active_selection → list → 过滤 active+enabled →
/// 单引擎 or translate_parallel。
///
/// 8 个参数是既定签名（`translate_session`/`translate_clipboard` 位置调用），
/// 收敛成 ctx struct 会让所有调用点更绕；与 `translate_parallel` 的扁平签名
/// 保持一致更易读。clippy 的 7 参数阈值是经验值，这里故意放宽。
#[allow(clippy::too_many_arguments)]
pub(crate) async fn run_translate_session(
    app: &tauri::AppHandle,
    db: &Arc<Database>,
    text: &str,
    from: &str,
    to: &str,
    trigger_source: &str,
    gen: Option<(&crate::concurrency::GenerationToken, u64)>,
) -> Result<TranslateSessionResult, String> {
    let settings_target = settings::load(app).target_language;
    let to = resolve_target_language(to, &settings_target);
    let fallback_box = settings::load(app)
        .fallback_engine
        .as_deref()
        .and_then(engines::find);
    let fallback: Option<Arc<dyn engines::TraditionalEngine>> =
        fallback_box.map(Arc::<dyn engines::TraditionalEngine>::from);
    let db = db.clone();
    let text = text.to_string();
    let from = from.to_string();
    let trigger_source = trigger_source.to_string();
    with_translation(app, |tx| {
        let persist = PersistSpec {
            trigger_source,
            gen,
        };
        let db = db.clone();
        let text = text.clone();
        let from = from.clone();
        let to = to.clone();
        async move {
            tx.run_session(&db, &text, &from, &to, fallback, Some(persist))
                .await
        }
    })
    .await
}

/// 测试入口：不读 settings，fallback 直接传 None（聚焦核心路径）。
pub async fn run_translate_session_no_settings(
    db: &Arc<Database>,
    client: &reqwest::Client,
    keystore: Option<&crate::keystore::Keystore>,
    text: &str,
    from: &str,
    to: &str,
) -> Result<TranslateSessionResult, String> {
    let store: Option<&dyn service::SecretStore> =
        keystore.map(|ks| ks as &dyn service::SecretStore);
    service::run_session_with_fallback(db, client, store, text, from, to, None).await
}

/// 并行/单引擎翻译命令（R2a）。前端用 `invoke('translate_session', { req })` 调用。
///
/// 从 AppState 读 active selection + providers，从 Session 读 client/keystore，
/// 从 settings 读 fallback_engine。parallel 为空时退化为单引擎（actual_engine=Some）。
#[tauri::command]
#[specta::specta]
pub async fn translate_session(
    app: tauri::AppHandle,
    _state: tauri::State<'_, Arc<Session>>,
    app_state: tauri::State<'_, Arc<AppState>>,
    req: TranslateSessionRequest,
) -> Result<TranslateSessionWire, String> {
    let app_arc = app_state.inner().clone();
    let db = tauri::async_runtime::spawn_blocking(move || -> Result<Arc<Database>, String> {
        let _gate = app_arc.data_gate.read();
        let db = require_database(&app_arc, &_gate)?;
        Ok(db)
    })
    .await
    .map_err(|e| e.to_string())??;
    run_translate_session(&app, &db, &req.text, &req.from, &req.to, "input", None)
        .await
        .map(TranslateSessionWire::from)
}

/// A4 (P1-5): translate the live OS selection (fresh capture) OR a
/// caller-supplied SOURCE text (Retry). Distinct from `translate_clipboard`
/// which reads the clipboard — this NEVER reads the clipboard. The tray
/// `translate-selection` action and the popup Retry both route here.
///
/// `text = Some(t)` (Retry): skip capture, use the saved SOURCE text.
/// `text = None` (tray): capture the selection fresh under the selection lock.
///
/// Returns `Result<(), ()>` because the popup state is emitted via events —
/// there is no useful payload to return to the caller. Allocation of the
/// generation token + the capture+translate pipeline run on the async runtime
/// so the IPC call does not block on capture_selection (which simulates Cmd+C).
#[tauri::command]
#[specta::specta]
pub async fn translate_selection_ipc(
    app: tauri::AppHandle,
    state: tauri::State<'_, Arc<Session>>,
    app_state: tauri::State<'_, Arc<AppState>>,
    text: Option<String>,
) -> Result<(), ()> {
    let state = state.inner().clone();
    let app_state = app_state.inner().clone();
    let gen = state.gen.next();
    tauri::async_runtime::spawn(async move {
        // The cursor read + capture_selection happen together under ONE lock
        // inside capture_and_translate (None coords → helper reads cursor under
        // the lock). For the Retry path (supplied text), the popup is re-shown
        // at the current cursor, so None coords are correct there too.
        capture_and_translate(&app, &state, &app_state, text, None, None, gen).await;
    });
    Ok(())
}

/// Outcome of the capture step. The popup UI for the non-`Selected` variants is
/// rendered AFTER the `selection_lock` guard drops and AFTER the `is_latest(gen)`
/// check, so the lock covers capture-only (P1-1). The physical cursor coords are
/// carried alongside each variant so the post-lock UI code can place the popup.
enum CaptureOutcome {
    Selected(String, f64, f64),
    NoSelection(f64, f64),
    CaptureError(String, f64, f64),
}

/// Shared selection-capture + translate-session pipeline. Used by on_hotkey,
/// translate_selection_ipc (tray + Retry). Emits the popup state per outcome.
///
/// - `supplied_text = Some(t)` (Retry): skip capture, use the saved SOURCE text.
/// - `supplied_text = None` (hotkey/tray): run the selection_lock +
///   capture_selection block on_hotkey used.
/// - `x`, `y`: `Option<f64>` — `Some` = caller-supplied PHYSICAL cursor coords
///   (Retry, tray: the caller captured coordinates itself); `None` = the helper
///   reads `cursor::position()` itself, INSIDE the single `selection_lock()`
///   guard that ALSO runs `capture_selection`, so the cursor read and the
///   clipboard-touching capture are atomic — a second rapid press cannot
///   interleave clipboard save/restore between them (P1-1). The helper resolves
///   the cursor's monitor via `monitor_from_point` and converts to logical via
///   THAT monitor's scale_factor (rev-7-1).
/// - `gen`: the generation token. Checked at every await boundary so a stale
///   run never overwrites a fresher popup (P1-1).
///
/// `to` is passed as `""` so run_translate_session's central resolver handles it.
#[allow(clippy::too_many_arguments)]
pub(crate) async fn capture_and_translate(
    app: &tauri::AppHandle,
    state: &Arc<Session>,
    app_state: &Arc<AppState>,
    supplied_text: Option<String>,
    x: Option<f64>,
    y: Option<f64>,
    gen: u64,
) {
    // 1. Acquire text (capture or supplied).
    let (text, anchor) = match supplied_text {
        Some(t) if !t.is_empty() => {
            let cx = x.unwrap_or(0.0);
            let cy = y.unwrap_or(0.0);
            let anchor = match build_popup_anchor(app, cx, cy) {
                Some(a) => a,
                None => return,
            };
            (t, anchor)
        }
        _ => {
            // The selection_lock covers capture-only: the cursor read and
            // capture_selection run under ONE guard so two rapid presses cannot
            // interleave clipboard save/restore between them. The popup UI for
            // the NoSelection / CaptureError branches is rendered AFTER the guard
            // drops and AFTER the is_latest(gen) check (P1-1).
            let captured: CaptureOutcome = {
                let _g = state.gen.selection_lock();
                // Read the cursor under the SAME guard as capture_selection so two
                // rapid presses cannot interleave clipboard save/restore between the
                // cursor read and the capture (the capture touches the clipboard).
                let (cx, cy) = match (x, y) {
                    (Some(cx), Some(cy)) => (cx, cy),
                    // hotkey/tray path: no pre-captured coords; read live now.
                    _ => {
                        let pos = cursor::position();
                        (pos.0 as f64, pos.1 as f64)
                    }
                };
                #[cfg(target_os = "windows")]
                let owner = match app
                    .get_webview_window("main")
                    .ok_or_else(|| "main window unavailable".to_string())
                    .and_then(|w| w.hwnd().map(|h| h.0).map_err(|e| e.to_string()))
                {
                    Ok(h) => h,
                    Err(e) => {
                        log::warn!("clipboard restore skipped: no owner HWND ({e})");
                        return;
                    }
                };
                #[cfg(not(target_os = "windows"))]
                let owner = ();
                match lease_selection(app) {
                    Err(e) => CaptureOutcome::CaptureError(e, cx, cy),
                    Ok(lease) => match lease.with(|sel| sel.capture_selection(800, owner)) {
                        Ok(Ok(selection_engine::Capture::Selected(t))) => {
                            CaptureOutcome::Selected(t, cx, cy)
                        }
                        Ok(Ok(selection_engine::Capture::NoSelection)) => {
                            CaptureOutcome::NoSelection(cx, cy)
                        }
                        Ok(Err(e)) => CaptureOutcome::CaptureError(e, cx, cy),
                        Err(e) => CaptureOutcome::CaptureError(e.to_string(), cx, cy),
                    },
                }
            };
            // Stale-run guard BEFORE any popup UI: a superseded capture must never
            // paint the popup (P1-1).
            if !state.gen.is_latest(gen) {
                return;
            }
            let (text, cx, cy) = match captured {
                CaptureOutcome::Selected(t, cx, cy) => (t, cx, cy),
                CaptureOutcome::NoSelection(cx, cy) => {
                    let anchor = match build_popup_anchor(app, cx, cy) {
                        Some(a) => a,
                        None => return,
                    };
                    let (px, py, pw, ph) =
                        popup::compute_popup_geometry_logical(popup::PopupMode::Error, &anchor);
                    let _ = popup::show_at_sized(app, px, py, pw, ph);
                    let _ = popup::error(
                        app,
                        if !a11y::enabled() {
                            "No selection captured. Grant Accessibility in System Settings → Privacy → Accessibility."
                        } else {
                            "No text selected."
                        },
                    );
                    return;
                }
                CaptureOutcome::CaptureError(e, cx, cy) => {
                    let anchor = match build_popup_anchor(app, cx, cy) {
                        Some(a) => a,
                        None => return,
                    };
                    let (px, py, pw, ph) =
                        popup::compute_popup_geometry_logical(popup::PopupMode::Error, &anchor);
                    let _ = popup::show_at_sized(app, px, py, pw, ph);
                    let _ = popup::error(app, &e);
                    return;
                }
            };
            let anchor = match build_popup_anchor(app, cx, cy) {
                Some(a) => a,
                None => return,
            };
            (text, anchor)
        }
    };

    // 2. Show loading popup sized + clamped, carrying the source (P1-3).
    if !state.gen.is_latest(gen) {
        return;
    }
    let _ = popup::loading_with_source(app, &anchor, Some(&text));

    // Task A5: the tray Active-pulse begins here. TranslationGuard::new calls
    // begin_translation(gen); its Drop calls finish_translation(gen, succeeded)
    // on EVERY return path below (early returns, error branches, success). On a
    // success branch we call guard.mark_success() so Drop clears any prior-gen
    // error; on an error branch we call record_translation_error(gen) BEFORE the
    // guard drops (Drop then only decrements + recomputes, leaving the error).
    let mut _tray_guard = tray_state::TranslationGuard::new(&app_state.tray, gen);

    // rev-9-1: acquire the db Arc via spawn_blocking (gate guard INSIDE the closure).
    let app_arc = app_state.clone();
    let db = match tauri::async_runtime::spawn_blocking(move || -> Result<Arc<Database>, String> {
        let _gate = app_arc.data_gate.read();
        require_database(&app_arc, &_gate)
    })
    .await
    {
        Ok(Ok(db)) => db,
        Ok(Err(msg)) => {
            if state.gen.is_latest(gen) {
                app_state.tray.lock().record_translation_error(gen);
                let _ = popup::set_popup_mode(app, popup::PopupMode::Error, &anchor);
                let _ = popup::error_with_source(app, &msg, &text);
            }
            return;
        }
        Err(e) => {
            if state.gen.is_latest(gen) {
                app_state.tray.lock().record_translation_error(gen);
                let _ = popup::set_popup_mode(app, popup::PopupMode::Error, &anchor);
                let _ = popup::error_with_source(app, &format!("join error: {e}"), &text);
            }
            return;
        }
    };

    if !state.gen.is_latest(gen) {
        return;
    }

    // 4. run_translate_session — to:"" is resolved centrally inside it.
    let session_result = match run_translate_session(
        app,
        &db,
        &text,
        "auto",
        "",
        "selection",
        Some((&state.gen, gen)),
    )
    .await
    {
        Ok(r) => r,
        Err(msg) => {
            if state.gen.is_latest(gen) {
                app_state.tray.lock().record_translation_error(gen);
                let _ = popup::set_popup_mode(app, popup::PopupMode::Error, &anchor);
                let _ = popup::error_with_source(app, &msg, &text);
            }
            return;
        }
    };
    if !state.gen.is_latest(gen) {
        return;
    }

    // 5. Route per decision + size per state.
    match decide_clipboard_popup(&session_result) {
        ClipboardPopupDecision::SingleSuccess { text: t, engine } => {
            _tray_guard.mark_success();
            let _ = popup::set_popup_mode(app, popup::PopupMode::Single, &anchor);
            let _ = popup::result_with_source(app, &t, &engine, &text);
        }
        ClipboardPopupDecision::Multi => {
            _tray_guard.mark_success();
            let _ = popup::set_popup_mode(app, popup::PopupMode::Multi, &anchor);
            let _ = popup::multi_result_with_source(app, &session_result.outcomes, &text);
        }
        ClipboardPopupDecision::Error(msg) => {
            app_state.tray.lock().record_translation_error(gen);
            let _ = popup::set_popup_mode(app, popup::PopupMode::Error, &anchor);
            let _ = popup::error_with_source(app, &msg, &text);
        }
    }
}

/// Build a PopupAnchor from the physical cursor coords. The scale factor used
/// to convert the work area AND the cursor is the TARGET MONITOR's
/// `scale_factor()` — NOT the popup window's.
fn build_popup_anchor(
    app: &tauri::AppHandle,
    x_phys: f64,
    y_phys: f64,
) -> Option<popup::PopupAnchor> {
    let win = app.get_webview_window("popup")?;

    let monitor = app
        .monitor_from_point(x_phys, y_phys)
        .ok()
        .flatten()
        .or_else(|| app.primary_monitor().ok().flatten());

    let mut sf = match &monitor {
        Some(m) => m.scale_factor(),
        None => win.scale_factor().unwrap_or(1.0),
    };
    if !(sf > 0.0 && sf.is_finite()) {
        sf = 1.0;
    }
    let cursor_logical = (x_phys / sf, y_phys / sf);

    let work_area_logical = if let Some(m) = &monitor {
        let wa = m.work_area();
        let pos = &wa.position;
        let sz = &wa.size;
        let left = pos.x as f64 / sf;
        let top = pos.y as f64 / sf;
        let right = left + sz.width as f64 / sf;
        let bottom = top + sz.height as f64 / sf;
        popup::LogicalWorkArea {
            left,
            top,
            right,
            bottom,
        }
    } else {
        let (cx, cy) = cursor_logical;
        popup::LogicalWorkArea {
            left: cx,
            top: cy,
            right: cx + 1.0,
            bottom: cy + 1.0,
        }
    };

    Some(popup::PopupAnchor {
        cursor_logical,
        work_area: work_area_logical,
        scale_factor: sf,
    })
}

#[cfg(test)]
mod remnant {
    #[test]
    fn translate_commands_do_not_read_session_client_or_keystore() {
        let src = include_str!("translate.rs");
        let prod = src.split("mod remnant").next().unwrap();
        assert!(
            !prod.contains("session_client"),
            "translate path must lease Translation, not Session.client"
        );
        assert!(
            !prod.contains("session_keystore"),
            "translate path must not read Session.keystore"
        );
        assert!(
            !prod.contains("state.client"),
            "capture_and_translate must not read Session.client"
        );
        assert!(
            !prod.contains("state.keystore"),
            "capture_and_translate must not read Session.keystore"
        );
    }
}
