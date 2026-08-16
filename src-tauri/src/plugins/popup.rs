//! Show/move/hide the frameless popup window; push a payload (loading / result).
use std::sync::Arc;
use tauri::{Emitter, Manager, WebviewWindow};

const POPUP_WINDOW: &str = "popup";
/// R2a: 多结果事件名（独立于 popup-state，向后兼容老前端）。
const POPUP_MULTI_EVENT: &str = "popup-multi-result";

fn window(app: &tauri::AppHandle) -> Result<WebviewWindow, String> {
    app.get_webview_window(POPUP_WINDOW)
        .ok_or_else(|| "no popup window".to_string())
}

pub fn show_at(app: &tauri::AppHandle, x: i32, y: i32) -> Result<(), String> {
    let win = window(app)?;
    win.set_position(tauri::PhysicalPosition { x, y })
        .map_err(|e| e.to_string())?;
    win.emit(
        "popup-state",
        Payload {
            status: "loading",
            text: "",
            engine: "",
            source_text: None,
        },
    )
    .map_err(|e| e.to_string())?;
    win.show().map_err(|e| e.to_string())?;
    win.set_focus().map_err(|e| e.to_string())?;
    Ok(())
}

pub fn result(app: &tauri::AppHandle, text: &str, engine: &str) -> Result<(), String> {
    let win = window(app)?;
    win.emit(
        "popup-state",
        Payload {
            status: "result",
            text,
            engine,
            source_text: None,
        },
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn error(app: &tauri::AppHandle, msg: &str) -> Result<(), String> {
    let win = window(app)?;
    win.emit(
        "popup-state",
        Payload {
            status: "error",
            text: msg,
            engine: "",
            source_text: None,
        },
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn hide(app: &tauri::AppHandle) -> Result<(), String> {
    let win = window(app)?;
    win.hide().map_err(|e| e.to_string())?;
    Ok(())
}

#[derive(Clone, serde::Serialize)]
struct Payload<'a> {
    status: &'a str,
    text: &'a str,
    engine: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    source_text: Option<&'a str>,
}

// ─── R2a: 多结果事件 ──────────────────────────────────────────────────────

/// 单个引擎结果的序列化形态（前端友好：Result 拆成 ok + text/error 扁平字段）。
#[derive(Clone, serde::Serialize, specta::Type)]
pub struct TranslationOutcomeSerialized {
    pub uuid: String,
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub engine: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl From<&crate::service::TranslationOutcome> for TranslationOutcomeSerialized {
    fn from(o: &crate::service::TranslationOutcome) -> Self {
        match &o.result {
            Ok(t) => Self {
                uuid: o.uuid.clone(),
                ok: true,
                text: Some(t.text.clone()),
                engine: Some(t.engine.clone()),
                error: None,
            },
            Err(e) => Self {
                uuid: o.uuid.clone(),
                ok: false,
                text: None,
                engine: None,
                error: Some(e.to_string()),
            },
        }
    }
}

#[derive(Clone, serde::Serialize)]
struct PopupMultiPayload {
    outcomes: Vec<TranslationOutcomeSerialized>,
    #[serde(skip_serializing_if = "Option::is_none")]
    source_text: Option<String>,
}

/// 推送多引擎翻译结果（R2a）。emit `popup-multi-result` 事件，
/// payload = `{ "outcomes": [ { uuid, ok, text?, engine?, error? }, ... ] }`。
/// 老前端只听 `popup-state`，不受影响；新前端监听本事件渲染 Surface 03。
pub fn multi_result(
    app: &tauri::AppHandle,
    outcomes: &[crate::service::TranslationOutcome],
) -> Result<(), String> {
    let win = window(app)?;
    let payload = PopupMultiPayload {
        outcomes: outcomes
            .iter()
            .map(TranslationOutcomeSerialized::from)
            .collect(),
        source_text: None,
    };
    win.emit(POPUP_MULTI_EVENT, payload)
        .map_err(|e| e.to_string())?;
    Ok(())
}

/// A2: emit result WITH source (B4 refines the serialization). Carries source_text
/// so Retry always has the original text (P1-3).
pub fn result_with_source(
    app: &tauri::AppHandle,
    text: &str,
    engine: &str,
    source_text: &str,
) -> Result<(), String> {
    let win = window(app)?;
    win.emit(
        "popup-state",
        Payload {
            status: "result",
            text,
            engine,
            source_text: Some(source_text),
        },
    )
    .map_err(|e| e.to_string())
}

/// A2: emit multi-result WITH source.
pub fn multi_result_with_source(
    app: &tauri::AppHandle,
    outcomes: &[crate::service::TranslationOutcome],
    source_text: &str,
) -> Result<(), String> {
    let win = window(app)?;
    let payload = PopupMultiPayload {
        outcomes: outcomes
            .iter()
            .map(TranslationOutcomeSerialized::from)
            .collect(),
        source_text: Some(source_text.to_owned()),
    };
    win.emit(POPUP_MULTI_EVENT, payload)
        .map_err(|e| e.to_string())
}

/// A2: emit error WITH source so the popup can still offer Retry (P1-3).
pub fn error_with_source(
    app: &tauri::AppHandle,
    msg: &str,
    source_text: &str,
) -> Result<(), String> {
    let win = window(app)?;
    win.emit(
        "popup-state",
        Payload {
            status: "error",
            text: msg,
            engine: "",
            source_text: Some(source_text),
        },
    )
    .map_err(|e| e.to_string())
}

// ─── A3: native sizing + work-area clamping (P1-2 unified units) ─────────

/// A monitor work area in LOGICAL (CSS) pixels. Callers fill this from
/// `Monitor::work_area()` (rev-6-1: the REAL `PhysicalRect<i32, u32>` returned
/// by Tauri 2.11.5) divided by scale_factor.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct LogicalWorkArea {
    pub left: f64,
    pub top: f64,
    pub right: f64,
    pub bottom: f64,
}

impl LogicalWorkArea {
    pub fn width(&self) -> f64 {
        self.right - self.left
    }
    pub fn height(&self) -> f64 {
        self.bottom - self.top
    }
}

/// P1-2: the single source of popup geometry. `cursor_logical` and `work_area`
/// are BOTH logical (CSS) px; `scale_factor` converts to physical. Every mode
/// change recomputes size AND position from this anchor.
#[derive(Debug, Clone, Copy)]
pub struct PopupAnchor {
    pub cursor_logical: (f64, f64),
    pub work_area: LogicalWorkArea,
    pub scale_factor: f64,
}

/// The four popup sizes the UI requests. Matched 1:1 to the loading /
/// single-success / multi-result / error UI states. Dimensions are LOGICAL.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PopupMode {
    Loading,
    Single,
    Multi,
    Error,
}

impl PopupMode {
    /// Logical (CSS) (width, height). Callers convert to physical via scale_factor.
    pub fn size_logical(&self) -> (u32, u32) {
        match self {
            PopupMode::Loading => (200, 40),
            PopupMode::Single => (400, 300),
            PopupMode::Multi => (600, 400),
            PopupMode::Error => (400, 300),
        }
    }
}

/// Margin between the popup and any work-area edge (logical px).
const CLAMP_MARGIN: f64 = 8.0;

/// Pure geometry: pick the popup's (x, y, width, height) PHYSICAL pixels for a
/// given mode + anchor. Clamps in LOGICAL space, then multiplies by scale_factor.
/// Pure on purpose — fully testable without a Tauri runtime. (P1-2)
pub fn compute_popup_geometry_logical(
    mode: PopupMode,
    anchor: &PopupAnchor,
) -> (i32, i32, u32, u32) {
    let (lw, lh) = mode.size_logical();
    let (lwf, lhf) = (lw as f64, lh as f64);
    let (cx, cy) = anchor.cursor_logical;
    debug_assert!(
        anchor.scale_factor.is_finite() && anchor.scale_factor > 0.0,
        "scale_factor must be finite and positive, got {}",
        anchor.scale_factor
    );

    let right_limit = anchor.work_area.right - CLAMP_MARGIN - lwf;
    let left_limit = anchor.work_area.left + CLAMP_MARGIN;
    let x_logical = cx.clamp(left_limit, right_limit.max(left_limit));

    let bottom_limit = anchor.work_area.bottom - CLAMP_MARGIN - lhf;
    let top_limit = anchor.work_area.top + CLAMP_MARGIN;
    let y_logical = cy.clamp(top_limit, bottom_limit.max(top_limit));

    let sf = anchor.scale_factor;
    let phys = |v: f64| -> i32 { (v * sf).round() as i32 };
    let phys_u = |v: f64| -> u32 {
        let p = phys(v);
        if p < 0 {
            0
        } else {
            p as u32
        }
    };
    (phys(x_logical), phys(y_logical), phys_u(lwf), phys_u(lhf))
}

/// Show the popup at an explicit PHYSICAL (x, y, width, height). Order is
/// load-bearing (P1-2): set_max_size FIRST, then set_size, then set_position.
pub fn show_at_sized(
    app: &tauri::AppHandle,
    x: i32,
    y: i32,
    width: u32,
    height: u32,
) -> Result<(), String> {
    let win = window(app)?;
    let size = tauri::PhysicalSize { width, height };
    win.set_max_size(Some(size)).map_err(|e| e.to_string())?;
    win.set_size(size).map_err(|e| e.to_string())?;
    win.set_position(tauri::PhysicalPosition { x, y })
        .map_err(|e| e.to_string())?;
    win.emit(
        "popup-state",
        Payload {
            status: "loading",
            text: "",
            engine: "",
            source_text: None,
        },
    )
    .map_err(|e| e.to_string())?;
    win.show().map_err(|e| e.to_string())?;
    win.set_focus().map_err(|e| e.to_string())?;
    Ok(())
}

/// Show loading popup sized for the given anchor, carrying the source text so
/// the frontend can save it for Retry (P1-3). A3 ships this so A2 can depend on
/// A3 without waiting on B4.
pub fn loading_with_source(
    app: &tauri::AppHandle,
    anchor: &PopupAnchor,
    source_text: Option<&str>,
) -> Result<(), String> {
    let (x, y, w, h) = compute_popup_geometry_logical(PopupMode::Loading, anchor);
    let win = window(app)?;
    let size = tauri::PhysicalSize {
        width: w,
        height: h,
    };
    win.set_max_size(Some(size)).map_err(|e| e.to_string())?;
    win.set_size(size).map_err(|e| e.to_string())?;
    win.set_position(tauri::PhysicalPosition { x, y })
        .map_err(|e| e.to_string())?;
    win.emit(
        "popup-state",
        Payload {
            status: "loading",
            text: "",
            engine: "",
            source_text,
        },
    )
    .map_err(|e| e.to_string())?;
    win.show().map_err(|e| e.to_string())?;
    win.set_focus().map_err(|e| e.to_string())?;
    Ok(())
}

/// Resize + reposition the popup for a UI mode. Recomputes BOTH size AND
/// position from the anchor (P1-2: a mode change is a geometry change).
pub fn set_popup_mode(
    app: &tauri::AppHandle,
    mode: PopupMode,
    anchor: &PopupAnchor,
) -> Result<(), String> {
    let (x, y, w, h) = compute_popup_geometry_logical(mode, anchor);
    let win = window(app)?;
    let size = tauri::PhysicalSize {
        width: w,
        height: h,
    };
    win.set_max_size(Some(size)).map_err(|e| e.to_string())?;
    win.set_size(size).map_err(|e| e.to_string())?;
    win.set_position(tauri::PhysicalPosition { x, y })
        .map_err(|e| e.to_string())?;
    Ok(())
}

use futures::future::BoxFuture;
use linguaray_kernel::{
    ActivationContext, CapabilityPlugin, EffectDisposer, PluginDescriptor, PluginError, PluginId,
    ServiceId, ServiceKey,
};

pub static POPUP: ServiceKey<PopupHub> = ServiceKey::new("linguaray.popup");
static PROVIDES: &[ServiceId] = &[ServiceId("linguaray.popup")];

pub struct PopupHub {
    app: tauri::AppHandle,
}

impl PopupHub {
    pub fn hide(&self) -> Result<(), String> {
        hide(&self.app)
    }
}

pub struct PopupPlugin {
    app: tauri::AppHandle,
}

impl PopupPlugin {
    pub fn new(app: tauri::AppHandle) -> Self {
        Self { app }
    }
}

impl CapabilityPlugin for PopupPlugin {
    fn descriptor(&self) -> PluginDescriptor {
        PluginDescriptor {
            id: PluginId("popup"),
            required: &[],
            optional: &[],
            provides: PROVIDES,
            manifest: None,
            restart_on_optional_change: false,
        }
    }

    fn config_fingerprint(&self) -> u64 {
        1
    }

    fn activate(&self, ctx: ActivationContext) -> BoxFuture<'_, Result<(), PluginError>> {
        let app = self.app.clone();
        Box::pin(async move {
            ctx.stage_provide(POPUP, Arc::new(PopupHub { app: app.clone() }))?;
            ctx.install_effect("popup.hide", move || {
                let app = app.clone();
                async move {
                    Ok(EffectDisposer::from_fn(move || {
                        let _ = hide(&app);
                    }))
                }
            })
            .await
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::{ConfigKind, Error, FallbackKind};
    use crate::service::{Translation, TranslationOutcome};

    fn ok_outcome(uuid: &str, text: &str, engine: &str) -> TranslationOutcome {
        TranslationOutcome {
            uuid: uuid.into(),
            result: Ok(Translation {
                text: text.into(),
                engine: engine.into(),
            }),
        }
    }

    fn err_outcome(uuid: &str, err: Error) -> TranslationOutcome {
        TranslationOutcome {
            uuid: uuid.into(),
            result: Err(err),
        }
    }

    #[test]
    fn serialize_ok_outcome() {
        let o = ok_outcome("u1", "你好", "provider/u1");
        let s = TranslationOutcomeSerialized::from(&o);
        assert!(s.ok);
        assert_eq!(s.text.as_deref(), Some("你好"));
        assert_eq!(s.engine.as_deref(), Some("provider/u1"));
        assert!(s.error.is_none());
        assert_eq!(s.uuid, "u1");
    }

    #[test]
    fn serialize_err_outcome_carries_message() {
        let o = err_outcome("u2", Error::LocalNoFallback);
        let s = TranslationOutcomeSerialized::from(&o);
        assert!(!s.ok);
        assert!(s.text.is_none());
        assert!(s.engine.is_none());
        let err = s.error.expect("error message present");
        assert!(err.contains("no fallback"), "got: {err}");
        assert_eq!(s.uuid, "u2");
    }

    #[test]
    fn serialize_config_error_outcome() {
        let o = err_outcome(
            "u3",
            Error::Config(ConfigKind::AuthFailed {
                provider: "p".into(),
                status: 401,
            }),
        );
        let s = TranslationOutcomeSerialized::from(&o);
        assert!(!s.ok);
        assert!(s.error.as_deref().unwrap().contains("401"));
    }

    #[test]
    fn serialize_fallback_eligible_outcome() {
        let o = err_outcome(
            "u4",
            Error::FallbackEligible(FallbackKind::ProviderStatus { status: 500 }),
        );
        let s = TranslationOutcomeSerialized::from(&o);
        assert!(!s.ok);
        assert!(s.error.as_deref().unwrap().contains("500"));
    }

    #[test]
    fn multi_result_payload_shape_is_outcomes_array() {
        let outcomes = [
            ok_outcome("u1", "a", "provider/u1"),
            err_outcome("u2", Error::LocalNoFallback),
        ];
        let payload = PopupMultiPayload {
            outcomes: outcomes
                .iter()
                .map(TranslationOutcomeSerialized::from)
                .collect(),
            source_text: None,
        };
        let json = serde_json::to_string(&payload).unwrap();
        assert!(json.contains("\"outcomes\""), "{json}");
        assert!(json.contains("\"u1\""), "{json}");
        assert!(json.contains("\"u2\""), "{json}");
        assert!(json.contains("\"text\":\"a\""), "{json}");
        assert!(json.contains("\"error\""), "{json}");
    }

    #[test]
    fn multi_result_emits_named_event() {
        assert_eq!(POPUP_MULTI_EVENT, "popup-multi-result");
    }
}
