use crate::tts::{self, SpeechVoice};

#[tauri::command]
pub fn tts_list_voices() -> Result<Vec<SpeechVoice>, String> {
    tts::list_voices().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn tts_speak(text: String, voice_id: Option<String>) -> Result<(), String> {
    tts::speak(&text, voice_id.as_deref()).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn tts_stop() {
    tts::stop();
}
