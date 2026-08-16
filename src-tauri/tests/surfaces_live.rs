//! Structural check: tray OCR/History and TTS commands are shipped.

#[test]
fn tray_ocr_and_history_are_not_coming_later() {
    let host = include_str!("../src/bootstrap/tray.rs");
    assert!(host.contains("\"OCR Translate\""));
    assert!(!host.contains("OCR Translate (Coming later)"));
    assert!(host.contains("\"History\""));
    assert!(!host.contains("History (Coming later)"));
}

#[test]
fn tts_and_ocr_commands_are_in_handler() {
    let host = include_str!("../src/lib.rs");
    let handler = host
        .split("collect_commands![")
        .nth(1)
        .and_then(|s| s.split(']').next())
        .unwrap_or("");
    for cmd in [
        "ocr_capture",
        "ocr_from_image",
        "ocr_recognize_bytes",
        "ocr_from_clipboard",
        "tts_speak",
        "tts_stop",
        "updater_check",
    ] {
        assert!(handler.contains(cmd), "{cmd}");
    }
}
