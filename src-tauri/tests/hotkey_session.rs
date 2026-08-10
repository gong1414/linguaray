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

#[test]
fn on_hotkey_does_not_hold_selection_lock_and_helper_reads_cursor_under_lock() {
    let src = include_str!("../src/lib.rs");

    // (a) on_hotkey must NOT acquire selection_lock — the lock + cursor read +
    //     capture_selection all live inside capture_and_translate now, under ONE
    //     guard, so two rapid presses cannot interleave clipboard save/restore.
    let on_start = src.find("fn on_hotkey").expect("on_hotkey fn not found");
    let on_body = &src[on_start..];
    let on_end = on_body[1..]
        .find("\nfn ")
        .or_else(|| on_body[1..].find("\nasync fn "))
        .or_else(|| on_body[1..].find("\npub fn "))
        .or_else(|| on_body[1..].find("\npub async fn "))
        .map(|i| i + 1)
        .unwrap_or(on_body.len());
    let on_hotkey_body = &on_body[..on_end];
    assert!(
        !on_hotkey_body.contains("selection_lock"),
        "on_hotkey must NOT acquire selection_lock itself; the lock moved into capture_and_translate so cursor+capture are atomic.\n--- on_hotkey body ---\n{}",
        on_hotkey_body,
    );

    // (b) capture_and_translate must read cursor::position() INSIDE its
    //     selection_lock block. Find the fn, then within it find the locked
    //     region (from `selection_lock()` to the matching drop at the `};` that
    //     closes the `let captured` block) and assert `cursor::position()` is
    //     textually within that region.
    let cap_start = src.find("async fn capture_and_translate").expect("capture_and_translate fn not found");
    let cap_body = &src[cap_start..];
    let cap_end = cap_body[1..]
        .find("\nfn ")
        .or_else(|| cap_body[1..].find("\nasync fn "))
        .map(|i| i + 1)
        .unwrap_or(cap_body.len());
    let helper_body = &cap_body[..cap_end];
    assert!(
        helper_body.contains("selection_lock"),
        "capture_and_translate must acquire selection_lock",
    );
    // The locked block runs cursor::position() and capture_selection under one guard.
    // Assert the cursor read appears AFTER the selection_lock() call within the helper.
    let lock_idx = helper_body.find("selection_lock").expect("selection_lock in helper");
    let after_lock = &helper_body[lock_idx..];
    assert!(
        after_lock.contains("cursor::position()"),
        "cursor::position() must be read INSIDE the selection_lock block in capture_and_translate, not before it.\n--- helper body after lock ---\n{}",
        after_lock,
    );
}

#[test]
fn capture_lock_block_excludes_ui_operations() {
    let src = include_str!("../src/lib.rs");
    let cap_start = src.find("async fn capture_and_translate")
        .expect("capture_and_translate fn not found");
    let cap_body = &src[cap_start..];
    let captured_var = cap_body.find("let captured")
        .or_else(|| cap_body.find("let outcome"))
        .expect("expected capture assignment");
    let block_open = cap_body[captured_var..].find('{')
        .expect("expected `{` after capture assignment");
    let block_start = captured_var + block_open;
    let mut depth = 0i32;
    let mut block_end = block_start;
    for (i, ch) in cap_body[block_start..].char_indices() {
        match ch {
            '{' => depth += 1,
            '}' => { depth -= 1; if depth == 0 { block_end = block_start + i + 1; break; } }
            _ => {}
        }
    }
    let lock_block = &cap_body[block_start..block_end];
    assert!(lock_block.contains("capture_selection"), "lock block must call capture_selection");
    assert!(!lock_block.contains("build_popup_anchor"), "build_popup_anchor must NOT be inside the selection_lock block (P1-1)");
    assert!(!lock_block.contains("show_at_sized"), "show_at_sized must NOT be inside the selection_lock block");
    assert!(!lock_block.contains("popup::error"), "popup::error must NOT be inside the selection_lock block");
    assert!(!lock_block.contains("compute_popup_geometry_logical"), "compute_popup_geometry_logical must NOT be inside the selection_lock block");
    // Verify is_latest is checked BEFORE build_popup_anchor in the post-lock code.
    let post_lock = &cap_body[block_end..];
    let is_latest_pos = post_lock.find("is_latest(gen)").unwrap_or(usize::MAX);
    let anchor_pos = post_lock.find("build_popup_anchor").unwrap_or(usize::MAX);
    assert!(is_latest_pos < anchor_pos, "is_latest(gen) must be checked BEFORE build_popup_anchor in the post-lock code");
}
