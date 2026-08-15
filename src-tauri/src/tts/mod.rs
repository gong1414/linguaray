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
        .and_then(|s| CString::new(s).ok());
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

#[cfg(target_os = "windows")]
fn platform_list() -> Result<Vec<SpeechVoice>, TtsError> {
    windows_tts::list()
}

#[cfg(target_os = "windows")]
fn platform_speak(text: &str, voice_id: Option<&str>) -> Result<(), TtsError> {
    windows_tts::speak(text, voice_id)
}

#[cfg(target_os = "windows")]
fn platform_stop() {
    windows_tts::stop();
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
fn platform_list() -> Result<Vec<SpeechVoice>, TtsError> {
    Ok(Vec::new())
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
fn platform_speak(_text: &str, _voice_id: Option<&str>) -> Result<(), TtsError> {
    Err(TtsError::Message("No system voices found".into()))
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
fn platform_stop() {}

#[cfg(target_os = "windows")]
mod windows_tts {
    use super::{SpeechVoice, TtsError};
    use windows::core::HSTRING;
    use windows::Media::SpeechSynthesis::SpeechSynthesizer;
    use windows::Win32::System::Com::{CoInitializeEx, COINIT_MULTITHREADED};

    fn ensure_com() {
        unsafe {
            let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
        }
    }

    pub fn list() -> Result<Vec<SpeechVoice>, TtsError> {
        ensure_com();
        let voices = SpeechSynthesizer::AllVoices()
            .map_err(|e| TtsError::Message(format!("Windows speech voices unavailable: {e}")))?;
        let n = voices
            .Size()
            .map_err(|e| TtsError::Message(format!("speech voice count: {e}")))?;
        let mut out = Vec::with_capacity(n as usize);
        for i in 0..n {
            let voice = voices
                .GetAt(i)
                .map_err(|e| TtsError::Message(format!("speech voice {i}: {e}")))?;
            let id = voice
                .Id()
                .map_err(|e| TtsError::Message(format!("speech voice id: {e}")))?
                .to_string();
            let name = voice
                .DisplayName()
                .map_err(|e| TtsError::Message(format!("speech voice name: {e}")))?
                .to_string();
            out.push(SpeechVoice { id, name });
        }
        Ok(out)
    }

    /// Write the synthesized WAV to a temp file, then play it via `PlaySoundW`.
    ///
    /// The previous implementation used `MediaPlayer` (Windows.Media.Playback),
    /// which does not exist on Windows Server (GitHub Actions `windows-latest`
    /// runners) and caused STATUS_ENTRYPOINT_NOT_FOUND at test-binary load.
    /// `PlaySoundW` from winmm.dll is available on every Windows edition.
    pub fn speak(text: &str, voice_id: Option<&str>) -> Result<(), TtsError> {
        use windows::Storage::Streams::DataReader;
        use windows::Win32::Media::Audio::{PlaySoundW, SND_FILENAME, SND_NODEFAULT};
        use windows::core::PCWSTR;

        stop();
        ensure_com();
        let synth = SpeechSynthesizer::new()
            .map_err(|e| TtsError::Message(format!("speech synthesizer: {e}")))?;
        if let Some(wanted) = voice_id.filter(|id| !id.is_empty()) {
            let voices = SpeechSynthesizer::AllVoices()
                .map_err(|e| TtsError::Message(format!("Windows speech voices unavailable: {e}")))?;
            let n = voices
                .Size()
                .map_err(|e| TtsError::Message(format!("speech voice count: {e}")))?;
            let mut found = false;
            for i in 0..n {
                let voice = voices
                    .GetAt(i)
                    .map_err(|e| TtsError::Message(format!("speech voice {i}: {e}")))?;
                if voice
                    .Id()
                    .map_err(|e| TtsError::Message(format!("speech voice id: {e}")))?
                    == wanted
                {
                    synth
                        .SetVoice(&voice)
                        .map_err(|e| TtsError::Message(format!("set speech voice: {e}")))?;
                    found = true;
                    break;
                }
            }
            if !found {
                return Err(TtsError::Message("unknown voice".into()));
            }
        }
        let stream = synth
            .SynthesizeTextToStreamAsync(&HSTRING::from(text))
            .map_err(|e| TtsError::Message(format!("synthesize: {e}")))?
            .get()
            .map_err(|e| TtsError::Message(format!("synthesize: {e}")))?;

        // Read the synthesized WAV bytes via DataReader.
        let size = stream
            .Size()
            .map_err(|e| TtsError::Message(format!("speech stream size: {e}")))? as u32;
        let reader = DataReader::CreateDataReader(&stream)
            .map_err(|e| TtsError::Message(format!("speech reader: {e}")))?;
        reader
            .LoadAsync(size)
            .map_err(|e| TtsError::Message(format!("speech load: {e}")))?
            .get()
            .map_err(|e| TtsError::Message(format!("speech load: {e}")))?;
        let mut wav = vec![0u8; size as usize];
        reader
            .ReadBytes(&mut wav)
            .map_err(|e| TtsError::Message(format!("speech read: {e}")))?;

        // Write to a temp file and play via PlaySoundW (winmm.dll).
        let path = std::env::temp_dir().join(format!(
            "linguaray-tts-{}-{}.wav",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_millis())
                .unwrap_or(0)
        ));
        std::fs::write(&path, &wav)
            .map_err(|e| TtsError::Message(format!("speech temp write: {e}")))?;

        let wide: Vec<u16> = path
            .as_os_str()
            .to_string_lossy()
            .encode_utf16()
            .chain(std::iter::once(0))
            .collect();
        let played = unsafe {
            PlaySoundW(PCWSTR(wide.as_ptr()), None, SND_FILENAME | SND_NODEFAULT)
        };
        let _ = std::fs::remove_file(&path);
        if !played.as_bool() {
            return Err(TtsError::Message("speech play failed".into()));
        }
        Ok(())
    }

    /// PlaySoundW is synchronous, so stop is a no-op. Kept for API parity.
    pub fn stop() {}
}
