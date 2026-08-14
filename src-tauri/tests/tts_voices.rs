use linguaray_lib::tts;

#[test]
fn list_speak_stop_on_real_path() {
    let voices = tts::list_voices().expect("list voices");
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
