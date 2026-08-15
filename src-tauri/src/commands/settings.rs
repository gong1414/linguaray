use tauri::{Emitter, Manager};

#[tauri::command]
pub async fn open_settings_window(
    app: tauri::AppHandle,
    section: Option<String>,
) -> Result<(), String> {
    let page = section.unwrap_or_else(|| "provider-center".to_string());
    let app2 = app.clone();
    tauri::async_runtime::spawn(async move {
        if let Some(w) = app2.get_webview_window("main") {
            let _ = w.show();
            let _ = w.set_focus();
            let _ = w.emit("navigate", page);
        }
    })
    .await
    .map_err(|e| format!("join error: {e}"))?;
    Ok(())
}

#[tauri::command]
pub fn get_settings(app: tauri::AppHandle) -> crate::settings::Settings {
    crate::settings::load(&app)
}

#[tauri::command]
pub fn set_setting(app: tauri::AppHandle, key: String, value: String) -> Result<(), String> {
    let mut s = crate::settings::load(&app);
    match key.as_str() {
        "default_provider" => s.default_provider = value,
        "target_language" => s.target_language = value,
        "fallback_engine" => {
            if value.is_empty() {
                s.fallback_engine = None;
            } else if crate::engines::find(&value).is_some() {
                s.fallback_engine = Some(value);
            } else {
                return Err(format!("unknown fallback engine: {value}"));
            }
        }
        "check_updates_on_startup" => {
            s.check_updates_on_startup = crate::settings::parse_bool_setting(&value)?;
        }
        _ => return Err(format!("unknown setting: {key}")),
    }
    crate::settings::save(&app, &s)
}

#[tauri::command]
pub fn a11y_status() -> bool {
    crate::a11y::enabled()
}
