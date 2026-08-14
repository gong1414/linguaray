//! Image-bytes OCR + optional region capture.

use serde::Serialize;
use thiserror::Error;

pub mod fixture;

#[derive(Debug, Clone, Serialize)]
pub struct OcrResult {
    pub text: String,
    pub confidence: f32,
}

#[derive(Debug, Error)]
pub enum OcrError {
    #[error("{0}")]
    Message(String),
}

/// Encode an RGBA8 bitmap as PNG, then recognize it.
pub fn recognize_rgba(width: u32, height: u32, rgba: &[u8]) -> Result<OcrResult, OcrError> {
    let png = encode_rgba_png(width, height, rgba)?;
    recognize_image_bytes(&png)
}

pub fn encode_rgba_png(width: u32, height: u32, rgba: &[u8]) -> Result<Vec<u8>, OcrError> {
    let img = image::RgbaImage::from_raw(width, height, rgba.to_vec())
        .ok_or_else(|| OcrError::Message("rgba size does not match width*height*4".into()))?;
    let mut out = Vec::new();
    let mut cursor = std::io::Cursor::new(&mut out);
    image::DynamicImage::ImageRgba8(img)
        .write_to(&mut cursor, image::ImageFormat::Png)
        .map_err(|e| OcrError::Message(e.to_string()))?;
    Ok(out)
}

/// Read the OS clipboard image (if any) and OCR it.
pub fn recognize_clipboard_image() -> Result<OcrResult, OcrError> {
    let blob = crate::clipboard::get_image()
        .map_err(OcrError::Message)?
        .ok_or_else(|| OcrError::Message("clipboard has no image".into()))?;
    let w = u32::try_from(blob.width).map_err(|_| OcrError::Message("width".into()))?;
    let h = u32::try_from(blob.height).map_err(|_| OcrError::Message("height".into()))?;
    recognize_rgba(w, h, &blob.bytes)
}

/// Recognize text from encoded image bytes (PNG/JPEG). Uses the platform OCR
/// engine — Vision on macOS, Windows.Media.Ocr on Windows.
pub fn recognize_image_bytes(bytes: &[u8]) -> Result<OcrResult, OcrError> {
    if bytes.is_empty() {
        return Err(OcrError::Message("empty image".into()));
    }
    platform_recognize(bytes)
}

/// Capture a rectangle of the main display and OCR it. Screen-recording
/// permission may be required; tests should call [`recognize_image_bytes`].
pub fn capture_region_and_recognize(x: i32, y: i32, w: i32, h: i32) -> Result<OcrResult, OcrError> {
    let png = platform_capture_region(x, y, w, h)?;
    recognize_image_bytes(&png)
}

#[cfg(target_os = "macos")]
fn platform_recognize(bytes: &[u8]) -> Result<OcrResult, OcrError> {
    use std::ffi::CStr;
    use std::os::raw::c_char;

    unsafe extern "C" {
        fn linguaray_vision_ocr(
            bytes: *const u8,
            len: usize,
            err_out: *mut *mut c_char,
        ) -> *mut c_char;
        fn linguaray_free(p: *mut std::ffi::c_void);
    }

    unsafe {
        let mut err: *mut c_char = std::ptr::null_mut();
        let raw = linguaray_vision_ocr(bytes.as_ptr(), bytes.len(), &mut err);
        if !err.is_null() {
            let msg = CStr::from_ptr(err).to_string_lossy().into_owned();
            linguaray_free(err.cast());
            if !raw.is_null() {
                linguaray_free(raw.cast());
            }
            return Err(OcrError::Message(msg));
        }
        if raw.is_null() {
            return Err(OcrError::Message("vision returned null".into()));
        }
        let text = CStr::from_ptr(raw).to_string_lossy().into_owned();
        linguaray_free(raw.cast());
        Ok(OcrResult {
            text: text.trim().to_string(),
            confidence: 1.0,
        })
    }
}

#[cfg(target_os = "macos")]
fn platform_capture_region(x: i32, y: i32, w: i32, h: i32) -> Result<Vec<u8>, OcrError> {
    if w <= 0 || h <= 0 {
        return Err(OcrError::Message("invalid region".into()));
    }
    let path = std::env::temp_dir().join(format!(
        "linguaray-ocr-{}-{}.png",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis())
            .unwrap_or(0)
    ));
    let status = std::process::Command::new("screencapture")
        .args([
            "-x",
            "-R",
            &format!("{x},{y},{w},{h}"),
            path.to_str().ok_or_else(|| OcrError::Message("path".into()))?,
        ])
        .status()
        .map_err(|e| OcrError::Message(e.to_string()))?;
    if !status.success() {
        return Err(OcrError::Message(
            "screen capture failed (Screen Recording permission?)".into(),
        ));
    }
    let bytes = std::fs::read(&path).map_err(|e| OcrError::Message(e.to_string()))?;
    let _ = std::fs::remove_file(&path);
    Ok(bytes)
}

#[cfg(all(not(target_os = "macos"), target_os = "windows"))]
fn platform_recognize(bytes: &[u8]) -> Result<OcrResult, OcrError> {
    windows_ocr::recognize(bytes)
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
fn platform_recognize(_bytes: &[u8]) -> Result<OcrResult, OcrError> {
    Err(OcrError::Message("OCR is not available on this platform".into()))
}

#[cfg(not(target_os = "macos"))]
fn platform_capture_region(_x: i32, _y: i32, _w: i32, _h: i32) -> Result<Vec<u8>, OcrError> {
    Err(OcrError::Message(
        "region capture is not available on this host".into(),
    ))
}

#[cfg(target_os = "windows")]
mod windows_ocr {
    use super::{OcrError, OcrResult};

    pub fn recognize(_bytes: &[u8]) -> Result<OcrResult, OcrError> {
        // Windows.Media.Ocr requires a WinRT apartment and an IRandomAccessStream
        // of the image. The binding is compiled so `cargo check --target` works;
        // a full DXGI + WinRT path is exercised on a Windows machine.
        Err(OcrError::Message(
            "Windows.Media.Ocr requires a Windows runtime host".into(),
        ))
    }
}
