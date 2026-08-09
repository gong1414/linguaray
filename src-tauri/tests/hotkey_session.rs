//! Task A2: prove (1) the central `to: ""` resolver lives in run_translate_session,
//! (2) the decision router (single/multi/error) is shared, and (3) on_hotkey no
//! longer references translate_with_fallback (source-structure assertion).
use linguaray_lib::{decide_clipboard_popup, ClipboardPopupDecision, TranslateSessionResult};
use linguaray_lib::service::{Translation, TranslationOutcome};

#[test]
fn empty_target_is_resolved_centrally_to_settings_value() {
    assert_eq!(
        linguaray_lib::resolve_target_language("", "zh"),
        "zh",
        "to:\"\" must resolve to settings.target_language inside run_translate_session"
    );
}

#[test]
fn explicit_target_is_passed_through_unchanged() {
    assert_eq!(linguaray_lib::resolve_target_language("ja", "zh"), "ja");
}

#[test]
fn hotkey_routes_multi_success_to_multi_event() {
    let result = TranslateSessionResult {
        outcomes: vec![
            TranslationOutcome {
                uuid: "u1".into(),
                result: Ok(Translation { text: "你好".into(), engine: "provider/u1".into() }),
            },
            TranslationOutcome {
                uuid: "u2".into(),
                result: Ok(Translation { text: "您好".into(), engine: "provider/u2".into() }),
            },
        ],
        actual_engine: None,
    };
    let decision = decide_clipboard_popup(&result);
    assert!(matches!(decision, ClipboardPopupDecision::Multi));
}

#[test]
fn hotkey_routes_single_success_to_result_event() {
    let result = TranslateSessionResult {
        outcomes: vec![TranslationOutcome {
            uuid: "u1".into(),
            result: Ok(Translation { text: "你好".into(), engine: "openai".into() }),
        }],
        actual_engine: Some("openai".into()),
    };
    let decision = decide_clipboard_popup(&result);
    match decision {
        ClipboardPopupDecision::SingleSuccess { engine, .. } => {
            assert_eq!(engine, "openai");
        }
        other => panic!("expected SingleSuccess, got {other:?}"),
    }
}

#[test]
fn hotkey_routes_all_failed_to_error_event() {
    use linguaray_lib::Error;
    let result = TranslateSessionResult {
        outcomes: vec![
            TranslationOutcome { uuid: "u1".into(), result: Err(Error::LocalNoFallback) },
            TranslationOutcome { uuid: "u2".into(), result: Err(Error::LocalNoFallback) },
        ],
        actual_engine: None,
    };
    let decision = decide_clipboard_popup(&result);
    assert!(matches!(decision, ClipboardPopupDecision::Error(_)));
}

#[test]
fn on_hotkey_does_not_call_translate_with_fallback() {
    let src = include_str!("../src/lib.rs");
    let start = src.find("fn on_hotkey").expect("on_hotkey fn not found");
    let body = &src[start..];
    let end = body[1..]
        .find("\nfn ")
        .or_else(|| body[1..].find("\nasync fn "))
        .or_else(|| body[1..].find("\npub fn "))
        .or_else(|| body[1..].find("\npub async fn "))
        .map(|i| i + 1)
        .unwrap_or(body.len());
    let on_hotkey_body = &body[..end];
    assert!(
        !on_hotkey_body.contains("translate_with_fallback("),
        "on_hotkey must not call translate_with_fallback; it should route through capture_and_translate -> run_translate_session.\n--- on_hotkey body ---\n{}",
        on_hotkey_body,
    );
    assert!(
        on_hotkey_body.contains("capture_and_translate("),
        "on_hotkey must call capture_and_translate.\n--- on_hotkey body ---\n{}",
        on_hotkey_body,
    );
}
