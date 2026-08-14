use crate::ocr::{self, OcrResult};

#[tauri::command]
pub fn ocr_recognize_bytes(bytes: Vec<u8>) -> Result<OcrResult, String> {
    ocr::recognize_image_bytes(&bytes).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn ocr_from_image(path: String) -> Result<OcrResult, String> {
    let bytes = std::fs::read(&path).map_err(|e| e.to_string())?;
    ocr::recognize_image_bytes(&bytes).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn ocr_from_clipboard() -> Result<OcrResult, String> {
    ocr::recognize_clipboard_image().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn ocr_capture_region(x: i32, y: i32, width: i32, height: i32) -> Result<OcrResult, String> {
    ocr::capture_region_and_recognize(x, y, width, height).map_err(|e| e.to_string())
}

/// Show the region-select overlay. Tray and the OCR hotkey call this.
#[tauri::command]
pub fn ocr_capture(app: tauri::AppHandle) -> Result<(), String> {
    use tauri::Manager;
    if let Some(w) = app.get_webview_window("ocr") {
        let _ = w.show();
        let _ = w.set_focus();
        return Ok(());
    }
    Err("ocr window missing".into())
}
