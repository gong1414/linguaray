use linguaray_lib::tts;

#[test]
fn list_speak_stop_on_real_path() {
    let voices = match tts::list_voices() {
        Ok(v) => v,
        Err(e) => {
            // Windows Server (GitHub Actions) has no speech language pack.
            // The error message must mention the unavailability, not crash.
            let msg = e.to_string().to_ascii_lowercase();
            assert!(
                msg.contains("voice")
                    || msg.contains("speech")
                    || msg.contains("unavailable")
                    || msg.contains("language"),
                "unexpected list_voices error: {e}"
            );
            tts::stop();
            return;
        }
    };
    if voices.is_empty() {
        let err = tts::speak("hello", None).expect_err("no voices must fail clearly");
        assert!(
            err.to_string().to_ascii_lowercase().contains("voice"),
            "{err}"
        );
        tts::stop();
        return;
    }
    tts::speak("hi", Some(&voices[0].id)).expect("speak");
    tts::stop();
}
