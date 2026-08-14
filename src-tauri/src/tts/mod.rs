//! System offline TTS: list / speak / stop.

use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Clone, Serialize)]
pub struct SpeechVoice {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Error)]
pub enum TtsError {
    #[error("{0}")]
    Message(String),
}

pub fn list_voices() -> Result<Vec<SpeechVoice>, TtsError> {
    platform_list()
}

pub fn speak(text: &str, voice_id: Option<&str>) -> Result<(), TtsError> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return Err(TtsError::Message("empty text".into()));
    }
    let voices = list_voices()?;
    if voices.is_empty() {
        return Err(TtsError::Message("No system voices found".into()));
    }
    if let Some(id) = voice_id {
        if !id.is_empty() && !voices.iter().any(|v| v.id == id) {
            return Err(TtsError::Message("unknown voice".into()));
        }
    }
    platform_speak(trimmed, voice_id)
}

pub fn stop() {
    platform_stop();
}

#[cfg(target_os = "macos")]
fn platform_list() -> Result<Vec<SpeechVoice>, TtsError> {
    use std::ffi::CStr;
    use std::os::raw::c_char;

    unsafe extern "C" {
        fn linguaray_tts_list_voices(err_out: *mut *mut c_char) -> *mut c_char;
        fn linguaray_free(p: *mut std::ffi::c_void);
    }

    unsafe {
        let mut err: *mut c_char = std::ptr::null_mut();
        let raw = linguaray_tts_list_voices(&mut err);
        if !err.is_null() {
            let msg = CStr::from_ptr(err).to_string_lossy().into_owned();
            linguaray_free(err.cast());
            return Err(TtsError::Message(msg));
        }
        if raw.is_null() {
            return Ok(Vec::new());
        }
        let json = CStr::from_ptr(raw).to_string_lossy().into_owned();
        linguaray_free(raw.cast());
        let ids: Vec<String> =
            serde_json::from_str(&json).map_err(|e| TtsError::Message(e.to_string()))?;
        Ok(ids
            .into_iter()
            .map(|id| {
                let name = id.rsplit('.').next().unwrap_or(id.as_str()).to_string();
                SpeechVoice { id, name }
            })
            .collect())
    }
}

#[cfg(target_os = "macos")]
fn platform_speak(text: &str, voice_id: Option<&str>) -> Result<(), TtsError> {
    use std::ffi::{CStr, CString};
    use std::os::raw::c_char;

    unsafe extern "C" {
        fn linguaray_tts_speak(
            text: *const c_char,
            voice_id: *const c_char,
            err_out: *mut *mut c_char,
        ) -> i32;
        fn linguaray_free(p: *mut std::ffi::c_void);
    }

    let text_c = CString::new(text).map_err(|_| TtsError::Message("text contains NUL".into()))?;
    let voice_c = voice_id
        .filter(|s| !s.is_empty())
        .map(|s| CString::new(s).ok())
        .flatten();
    unsafe {
        let mut err: *mut c_char = std::ptr::null_mut();
        let voice_ptr = voice_c
            .as_ref()
            .map(|c| c.as_ptr())
            .unwrap_or(std::ptr::null());
        let rc = linguaray_tts_speak(text_c.as_ptr(), voice_ptr, &mut err);
        if rc != 0 {
            let msg = if err.is_null() {
                "speak failed".into()
            } else {
                let s = CStr::from_ptr(err).to_string_lossy().into_owned();
                linguaray_free(err.cast());
                s
            };
            return Err(TtsError::Message(msg));
        }
    }
    Ok(())
}

#[cfg(target_os = "macos")]
fn platform_stop() {
    unsafe extern "C" {
        fn linguaray_tts_stop();
    }
    unsafe {
        linguaray_tts_stop();
    }
}

#[cfg(not(target_os = "macos"))]
fn platform_list() -> Result<Vec<SpeechVoice>, TtsError> {
    Ok(Vec::new())
}

#[cfg(not(target_os = "macos"))]
fn platform_speak(_text: &str, _voice_id: Option<&str>) -> Result<(), TtsError> {
    Err(TtsError::Message("No system voices found".into()))
}

#[cfg(not(target_os = "macos"))]
fn platform_stop() {}
