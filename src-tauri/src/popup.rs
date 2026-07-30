//! Show/move/hide the frameless popup window; push a payload (loading / result).
use tauri::{Emitter, Manager, WebviewWindow};

const POPUP: &str = "popup";

fn window(app: &tauri::AppHandle) -> Result<WebviewWindow, String> {
    app.get_webview_window(POPUP).ok_or_else(|| "no popup window".to_string())
}

pub fn show_at(app: &tauri::AppHandle, x: i32, y: i32) -> Result<(), String> {
    let win = window(app)?;
    win.set_position(tauri::PhysicalPosition { x, y }).map_err(|e| e.to_string())?;
    win.emit("popup-state", Payload { status: "loading", text: "", engine: "" }).map_err(|e| e.to_string())?;
    win.show().map_err(|e| e.to_string())?;
    win.set_focus().map_err(|e| e.to_string())?;
    Ok(())
}

pub fn result(app: &tauri::AppHandle, text: &str, engine: &str) -> Result<(), String> {
    let win = window(app)?;
    win.emit("popup-state", Payload { status: "result", text, engine }).map_err(|e| e.to_string())?;
    Ok(())
}

pub fn error(app: &tauri::AppHandle, msg: &str) -> Result<(), String> {
    let win = window(app)?;
    win.emit("popup-state", Payload { status: "error", text: msg, engine: "" }).map_err(|e| e.to_string())?;
    Ok(())
}

pub fn hide(app: &tauri::AppHandle) -> Result<(), String> {
    let win = window(app)?;
    win.hide().map_err(|e| e.to_string())?;
    Ok(())
}

#[derive(Clone, serde::Serialize)]
struct Payload<'a> { status: &'a str, text: &'a str, engine: &'a str }
