use crate::tts;

#[tauri::command]
#[specta::specta]
pub fn tts_speak(text: String, voice_id: Option<String>) -> Result<(), String> {
    tts::speak(&text, voice_id.as_deref()).map_err(|e| e.to_string())
}

#[tauri::command]
#[specta::specta]
pub fn tts_stop() {
    tts::stop();
}
