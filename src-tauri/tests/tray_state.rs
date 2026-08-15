//! Task A5 (rev-11 → rev-19): tray visual-state controller.
#![allow(clippy::doc_lazy_continuation)]
//!
//! Sections (33 tests):
//! 1. rev-11 priority ordering (6 pure-function tests).
//! 2. rev-12/rev-16 TrayStateController reducer concurrency (6 tests).
//! 3. rev-13/rev-14/rev-15/rev-16 (P1-2) TranslationGuard RAII (2 tests).
//! 4. rev-13/rev-16 (P1-3 + rev-16-2) generation-aware error (2 tests).
//! 4b. rev-16 + rev-17-3 gen guards (3 tests).
//! 5. rev-14/rev-15/rev-16/rev-17 (P1-2 + P1-5) TrayRenderer + PulseWorker (4 tests).
//! 6. rev-15 (P1-1) PulseWorker channel-quit (2 tests).
//! 7. rev-15/rev-16 (P1-4) worker-stop barrier (1 test).
//! 8. rev-12 (P2) + rev-14 localization (2 tests).
//! 9. rev-14/rev-15/rev-16 red-dot pixel-diff (1 test).
//! 10. rev-15/rev-16/rev-17/rev-18 switch does NOT bump generation (2 tests).
//! 11. rev-16 (P1-3) switch revision ordering (2 tests).
use linguaray_lib::tray_state::{
    detect_system_locale, recompute_pure, tray_state_priority, tray_tooltip_text, Locale,
    PulseEvent, PulseWorker, RecordingRenderer, TrayStateController,
    TrayVisualState, TranslationGuard,
};
use std::sync::Arc;

// ─── 1. rev-11: priority ordering (pure functions) ──────────────────────────

#[test]
fn normal_is_lowest_priority() {
    assert_eq!(tray_state_priority(TrayVisualState::Normal), 0);
}

#[test]
fn active_beats_normal() {
    assert!(
        tray_state_priority(TrayVisualState::ActiveTranslation)
            > tray_state_priority(TrayVisualState::Normal)
    );
}

#[test]
fn update_beats_active() {
    assert!(
        tray_state_priority(TrayVisualState::UpdateAvailable)
            > tray_state_priority(TrayVisualState::ActiveTranslation)
    );
}

#[test]
fn error_is_highest_priority() {
    assert!(
        tray_state_priority(TrayVisualState::Error)
            > tray_state_priority(TrayVisualState::UpdateAvailable)
    );
    assert!(
        tray_state_priority(TrayVisualState::Error)
            > tray_state_priority(TrayVisualState::ActiveTranslation)
    );
    assert!(
        tray_state_priority(TrayVisualState::Error)
            > tray_state_priority(TrayVisualState::Normal)
    );
}

#[test]
fn full_ordering_is_error_update_active_normal() {
    let mut ordered = [
        TrayVisualState::Normal,
        TrayVisualState::Error,
        TrayVisualState::ActiveTranslation,
        TrayVisualState::UpdateAvailable,
    ];
    ordered.sort_by_key(|s| tray_state_priority(*s));
    assert_eq!(
        ordered,
        [
            TrayVisualState::Normal,
            TrayVisualState::ActiveTranslation,
            TrayVisualState::UpdateAvailable,
            TrayVisualState::Error,
        ]
    );
}

#[test]
fn update_arm_exists_but_is_documented_deferred() {
    // The UpdateAvailable variant is RETAINED so the priority ordering is
    // testable, but `recompute` NEVER produces it this stage (deferred to
    // R5/R6 per user-approved scope decision).
    let _ = TrayVisualState::UpdateAvailable;
}

// ─── 2. rev-12/rev-13/rev-14/rev-15/rev-16: TrayStateController reducer ──────
// SYNC methods (no .await). Controller backed by a RecordingRenderer.

fn test_controller() -> TrayStateController {
    TrayStateController::with_renderer_and_interval(
        Arc::new(RecordingRenderer::default()),
        Locale::En,
        std::time::Duration::from_millis(2),
    )
}

#[test]
fn recompute_pure_normal_when_idle() {
    let c = test_controller();
    assert_eq!(recompute_pure(&c), TrayVisualState::Normal);
}

#[test]
fn begin_then_finish_two_translations_keeps_active_until_last_finishes() {
    let mut c = test_controller();
    c.begin_translation(1);
    c.begin_translation(2);
    assert_eq!(recompute_pure(&c), TrayVisualState::ActiveTranslation);
    c.finish_translation(1, false);
    assert_eq!(
        recompute_pure(&c),
        TrayVisualState::ActiveTranslation,
        "still Active while one translation remains"
    );
    c.finish_translation(2, false);
    assert_eq!(recompute_pure(&c), TrayVisualState::Normal);
}

#[test]
fn finish_translation_saturates_at_zero() {
    let mut c = test_controller();
    c.finish_translation(1, false);
    c.finish_translation(2, false);
    assert_eq!(c.active_translations(), 0);
    assert_eq!(recompute_pure(&c), TrayVisualState::Normal);
}

#[test]
fn translation_error_overrides_active_and_survives_finish_false() {
    let mut c = test_controller();
    c.begin_translation(1);
    c.record_translation_error(1);
    assert_eq!(recompute_pure(&c), TrayVisualState::Error);
    c.finish_translation(1, false);
    assert_eq!(
        recompute_pure(&c),
        TrayVisualState::Error,
        "Error must NOT be cleared by finish_translation(false) alone"
    );
}

#[test]
fn recompute_never_produces_update_available() {
    let mut c = test_controller();
    c.begin_translation(1);
    assert_ne!(recompute_pure(&c), TrayVisualState::UpdateAvailable);
    c.record_translation_error(1);
    assert_ne!(recompute_pure(&c), TrayVisualState::UpdateAvailable);
    let rev = c.begin_switch();
    c.finish_switch(rev, false);
    assert_ne!(recompute_pure(&c), TrayVisualState::UpdateAvailable);
}

#[test]
fn switch_flow_error_is_independent_of_translation_error_gen() {
    let mut c = test_controller();
    let rev = c.begin_switch();
    c.finish_switch(rev, false);
    assert_eq!(c.switch_error_rev(), Some(rev));
    assert_eq!(c.error_gen(), None);
    assert_eq!(recompute_pure(&c), TrayVisualState::Error);
    c.finish_switch(rev, true);
    assert_eq!(c.switch_error_rev(), None);
    assert_eq!(recompute_pure(&c), TrayVisualState::Normal);
}

// ─── 3. rev-13/rev-14/rev-15 (P1-2): TranslationGuard RAII ──────────────────

#[test]
fn guard_drop_finishes_translation_on_every_return_path() {
    let controller = Arc::new(parking_lot::Mutex::new(test_controller()));
    {
        let _guard = TranslationGuard::new(&controller, 1);
        assert_eq!(controller.lock().active_translations(), 1);
    }
    assert_eq!(
        controller.lock().active_translations(),
        0,
        "guard Drop decremented the counter synchronously"
    );
}

#[test]
fn guard_marks_success_and_clears_prior_gen_error() {
    let controller = Arc::new(parking_lot::Mutex::new(test_controller()));
    controller.lock().record_translation_error(1);
    {
        let mut guard = TranslationGuard::new(&controller, 2);
        guard.mark_success();
    }
    assert_eq!(
        controller.lock().error_gen(),
        None,
        "successful Retry of a new gen clears the prior gen's error (1 <= 2)"
    );
}

// ─── 4. rev-13/rev-16 (P1-3 + rev-16-2): generation-aware error ─────────────

#[test]
fn retry_of_new_gen_clears_prior_red_dot() {
    let mut c = test_controller();
    c.begin_translation(1);
    c.record_translation_error(1);
    assert_eq!(c.error_gen(), Some(1));
    c.begin_translation(2);
    assert_eq!(
        c.error_gen(),
        None,
        "begin_translation of a newer gen clears the older gen's error"
    );
    // Two begins → two finishes to bring the counter back to 0.
    c.finish_translation(1, true);
    c.finish_translation(2, true);
    assert_eq!(recompute_pure(&c), TrayVisualState::Normal);
}

#[test]
fn same_gen_retry_does_not_clear_error() {
    let mut c = test_controller();
    c.begin_translation(1);
    c.record_translation_error(1);
    c.begin_translation(1);
    assert_eq!(c.error_gen(), Some(1), "same-gen begin must NOT clear error");
}

// ─── 4b. rev-16 + rev-17-3: gen guards + latest_translation_gen ──────────────

#[test]
fn older_success_does_not_clear_newer_error() {
    let mut c = test_controller();
    c.begin_translation(1);
    c.record_translation_error(1);
    c.begin_translation(2);
    c.record_translation_error(2);
    assert_eq!(c.error_gen(), Some(2));
    c.finish_translation(1, true);
    assert_eq!(
        c.error_gen(),
        Some(2),
        "older gen's success must NOT clear a newer gen's error (rev-16-2 gen guard)"
    );
    assert_eq!(recompute_pure(&c), TrayVisualState::Error);
}

#[test]
fn older_error_does_not_replace_newer_error() {
    let mut c = test_controller();
    c.begin_translation(2);
    c.record_translation_error(2);
    assert_eq!(c.error_gen(), Some(2));
    c.record_translation_error(1);
    assert_eq!(
        c.error_gen(),
        Some(2),
        "older gen's late error must NOT clobber a newer gen's error (rev-16-2 gen guard)"
    );
}

#[test]
fn stale_gen_error_ignored_after_newer_begin() {
    let mut c = test_controller();
    c.begin_translation(1);
    c.begin_translation(2);
    assert_eq!(c.error_gen(), None);
    c.record_translation_error(1);
    assert_eq!(
        c.error_gen(),
        None,
        "a stale gen's late error must be ignored after a newer gen began (rev-17-3 latest_translation_gen guard)"
    );
    assert_eq!(recompute_pure(&c), TrayVisualState::ActiveTranslation);
    c.record_translation_error(2);
    assert_eq!(c.error_gen(), Some(2));
}

// ─── 5. rev-14/rev-15/rev-16/rev-17 (P1-2 + P1-5): TrayRenderer + PulseWorker ─

fn controller_with_notify() -> (
    TrayStateController,
    Arc<RecordingRenderer>,
    std::sync::mpsc::Receiver<PulseEvent>,
) {
    let (notify_tx, notify_rx) = std::sync::mpsc::channel();
    let renderer = Arc::new(RecordingRenderer::default());
    let c = TrayStateController::with_renderer_interval_and_notify(
        renderer.clone(),
        Locale::En,
        std::time::Duration::from_millis(2),
        Some(notify_tx),
    );
    (c, renderer, notify_rx)
}

#[test]
fn active_emits_alternating_frames_on_the_renderer() {
    let (mut c, renderer, notify_rx) = controller_with_notify();
    c.begin_translation(1);
    match notify_rx.recv_timeout(std::time::Duration::from_millis(50)) {
        Ok(PulseEvent::Tick) => {}
        other => panic!("expected PulseEvent::Tick (frame 1), got {other:?}"),
    }
    match notify_rx.recv_timeout(std::time::Duration::from_millis(50)) {
        Ok(PulseEvent::Tick) => {}
        other => panic!("expected PulseEvent::Tick (frame 2), got {other:?}"),
    }
    let calls = renderer.calls();
    assert!(
        calls.iter().any(|(icon, _)| icon.is_dimmed()),
        "expected at least one dimmed pulse frame"
    );
    assert!(
        calls.iter().any(|(icon, _)| icon.is_normal()),
        "expected at least one normal pulse frame"
    );
    c.finish_translation(1, true);
}

#[test]
fn second_begin_does_not_churn_the_worker() {
    let (mut c, _renderer, notify_rx) = controller_with_notify();
    assert_eq!(c.worker_start_count(), 0, "no worker started before any begin");
    c.begin_translation(1);
    let count_after_first = c.worker_start_count();
    assert_eq!(count_after_first, 1, "first begin started exactly one worker");
    assert!(c.is_pulsing(), "worker running after first begin");
    assert_eq!(c.current_state(), TrayVisualState::ActiveTranslation);
    match notify_rx.recv_timeout(std::time::Duration::from_millis(50)) {
        Ok(PulseEvent::Tick) => {}
        other => panic!("expected PulseEvent::Tick after first begin, got {other:?}"),
    }
    c.begin_translation(2);
    assert_eq!(
        c.worker_start_count(),
        count_after_first,
        "second begin did NOT churn the worker (recompute early-returned on Active→Active)"
    );
    assert!(c.is_pulsing(), "the worker is still running after the second begin");
    assert_eq!(
        c.current_state(),
        TrayVisualState::ActiveTranslation,
        "state stays Active across the second begin (no churn)"
    );
    match notify_rx.recv_timeout(std::time::Duration::from_millis(50)) {
        Ok(PulseEvent::Tick) => {}
        other => panic!("expected PulseEvent::Tick from the same worker after second begin, got {other:?}"),
    }
    c.finish_translation(1, true);
    c.finish_translation(2, true);
}

#[test]
fn last_finish_stops_the_worker() {
    let (mut c, renderer, notify_rx) = controller_with_notify();
    c.begin_translation(1);
    c.finish_translation(1, true);
    assert!(!c.is_pulsing());
    let calls_before = renderer.calls().len();
    match notify_rx.recv_timeout(std::time::Duration::from_millis(50)) {
        Ok(PulseEvent::Stopped) => {}
        other => panic!("expected PulseEvent::Stopped after worker drop, got {other:?}"),
    }
    assert_eq!(
        renderer.calls().len(),
        calls_before,
        "no further frames after the last finish (worker stopped via channel-quit)"
    );
}

#[test]
fn error_produces_no_active_pulse_frame() {
    let (mut c, renderer, _notify_rx) = controller_with_notify();
    c.record_translation_error(1);
    assert!(
        !renderer.calls().iter().any(|(icon, _)| icon.is_dimmed()),
        "Error must not start the Active pulse"
    );
    assert!(
        renderer.calls().iter().any(|(icon, _)| icon.is_error_dot()),
        "Error must emit the red-dot overlay"
    );
}

// ─── 6. rev-15 (P1-1): PulseWorker channel-quit ─────────────────────────────

#[test]
fn stop_signal_joins_the_worker() {
    let renderer = Arc::new(RecordingRenderer::default());
    let (notify_tx, notify_rx) = std::sync::mpsc::channel();
    let mut worker = PulseWorker::start(
        renderer.clone(),
        std::time::Duration::from_millis(2),
        Some(notify_tx),
    );
    match notify_rx.recv_timeout(std::time::Duration::from_millis(50)) {
        Ok(PulseEvent::Tick) => {}
        other => panic!("expected PulseEvent::Tick (worker running) before stop, got {other:?}"),
    }
    worker.stop();
    match notify_rx.recv_timeout(std::time::Duration::from_millis(50)) {
        Ok(PulseEvent::Stopped) => {}
        other => panic!("expected PulseEvent::Stopped after stop(), got {other:?}"),
    }
    worker.stop();
    match notify_rx.recv_timeout(std::time::Duration::from_millis(50)) {
        Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {}
        other => panic!("expected Disconnected after worker drop, got {other:?}"),
    }
}

#[test]
fn drop_stops_the_worker() {
    let renderer = Arc::new(RecordingRenderer::default());
    let (notify_tx, notify_rx) = std::sync::mpsc::channel();
    {
        let _worker = PulseWorker::start(
            renderer.clone(),
            std::time::Duration::from_millis(2),
            Some(notify_tx),
        );
        match notify_rx.recv_timeout(std::time::Duration::from_millis(50)) {
            Ok(PulseEvent::Tick) => {}
            other => panic!("expected PulseEvent::Tick (worker running) before drop, got {other:?}"),
        }
    }
    match notify_rx.recv_timeout(std::time::Duration::from_millis(50)) {
        Ok(PulseEvent::Stopped) => {}
        other => panic!("expected PulseEvent::Stopped after drop (NOT Disconnected), got {other:?}"),
    }
    match notify_rx.recv_timeout(std::time::Duration::from_millis(50)) {
        Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {}
        other => panic!("expected Disconnected after the Stopped event, got {other:?}"),
    }
}

// ─── 7. rev-15/rev-16 (P1-4): worker-stop barrier ───────────────────────────

#[test]
fn leaving_active_stops_the_worker_no_stale_frames() {
    let (mut c, renderer, notify_rx) = controller_with_notify();
    c.begin_translation(1);
    match notify_rx.recv_timeout(std::time::Duration::from_millis(50)) {
        Ok(PulseEvent::Tick) => {}
        other => panic!("expected PulseEvent::Tick before the Error transition, got {other:?}"),
    }
    let dimmed_before = renderer.calls().iter().filter(|(i, _)| i.is_dimmed()).count();
    c.record_translation_error(1);
    assert!(!c.is_pulsing(), "the worker was dropped on the Active → Error transition");
    match notify_rx.recv_timeout(std::time::Duration::from_millis(50)) {
        Ok(PulseEvent::Stopped) => {}
        other => panic!("expected PulseEvent::Stopped after Active→Error worker drop, got {other:?}"),
    }
    let dimmed_after = renderer.calls().iter().filter(|(i, _)| i.is_dimmed()).count();
    assert_eq!(
        dimmed_after, dimmed_before,
        "no new dimmed frames after Error — the worker was stopped (channel-quit barrier)"
    );
}

// ─── 8. rev-12 (P2) + rev-14: localization ──────────────────────────────────

#[test]
fn tooltip_text_is_localized() {
    assert_eq!(tray_tooltip_text(TrayVisualState::Normal, Locale::En), "LinguaRay");
    assert_eq!(tray_tooltip_text(TrayVisualState::Normal, Locale::Zh), "LinguaRay");
    assert_eq!(
        tray_tooltip_text(TrayVisualState::ActiveTranslation, Locale::En),
        "Translating…"
    );
    assert_eq!(
        tray_tooltip_text(TrayVisualState::ActiveTranslation, Locale::Zh),
        "翻译中…"
    );
    assert_eq!(
        tray_tooltip_text(TrayVisualState::Error, Locale::En),
        "LinguaRay — Error"
    );
    assert_eq!(
        tray_tooltip_text(TrayVisualState::Error, Locale::Zh),
        "LinguaRay — 错误"
    );
}

#[test]
fn detect_system_locale_never_panics() {
    let _ = detect_system_locale();
}

// ─── 9. rev-14/rev-15: red-dot pixel-diff ───────────────────────────────────

#[test]
fn red_dot_overlay_preserves_base_icon_outside_the_dot() {
    let error_png = concat!(env!("OUT_DIR"), "/tray-error-32.png");
    let base_png = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("icons/32x32.png");
    let error_img = image::open(error_png)
        .unwrap_or_else(|e| panic!("build.rs output not found: {error_png} ({e})"));
    let base_img = image::open(&base_png)
        .unwrap_or_else(|e| panic!("base icon not found: {} ({e})", base_png.display()));
    let error_rgba = error_img.to_rgba8();
    let base_rgba = base_img.to_rgba8();
    let dot_center = (26i32, 6i32);
    let dot_radius = 5i32;
    let mut base_unchanged = 0;
    let mut dot_pixels = 0;
    let mut red_dot_pixels = 0;
    for y in 0..32i32 {
        for x in 0..32i32 {
            let dx = x - dot_center.0;
            let dy = y - dot_center.1;
            let in_dot = dx * dx + dy * dy <= dot_radius * dot_radius;
            let b = base_rgba.get_pixel(x as u32, y as u32);
            let e = error_rgba.get_pixel(x as u32, y as u32);
            if in_dot {
                dot_pixels += 1;
                if e.0 == [220, 38, 38, 255] {
                    red_dot_pixels += 1;
                }
            } else if b == e {
                base_unchanged += 1;
            }
        }
    }
    assert!(
        red_dot_pixels > 0,
        "the dot region must contain #DC2626 pixels (found {red_dot_pixels})"
    );
    assert!(
        base_unchanged >= 32 * 32 - dot_pixels - 4,
        "base icon pixels OUTSIDE the dot must be unchanged"
    );
}

// ─── 10. rev-15/rev-16/rev-17/rev-18: switch does NOT bump generation ───────

#[test]
fn switch_handler_does_not_call_gen_next() {
    use linguaray_lib::concurrency::GenerationToken;
    use linguaray_lib::db::Database;
    use linguaray_lib::db::readiness::DataReadiness;
    use linguaray_lib::db::providers as db_providers;
    use linguaray_lib::db::schema;

    // rev-19-2: fresh_db pattern — open + create_all_tables + seed_singletons.
    let dir = tempfile::tempdir().expect("tempdir");
    let db_path = dir.path().join("linguaray.db");
    let db = Database::open(&db_path).expect("Database::open");
    db.with_conn(|conn| {
        let tx = conn.transaction()?;
        schema::create_all_tables(&tx)?;
        schema::seed_singletons(&tx)?;
        tx.commit()?;
        Ok(())
    })
    .expect("create_all_tables + seed_singletons");
    let uuid = {
        let profile = db
            .with_conn(|conn| db_providers::create(conn, "custom", "Test Provider", "http://localhost:11434", None))
            .expect("db_providers::create");
        profile.uuid
    };

    let renderer = Arc::new(RecordingRenderer::default());
    let app_state = Arc::new(linguaray_lib::AppState {
        db: parking_lot::RwLock::new(Some(Arc::new(db))),
        data_gate: parking_lot::RwLock::new(()),
        readiness: parking_lot::RwLock::new(DataReadiness::Ready),
        db_path: db_path.clone(),
        keystore_dir: dir.path().join("keystore"),
        settings_path: Some(dir.path().join("settings.json")),
        tray: Arc::new(parking_lot::Mutex::new(
            TrayStateController::with_renderer(renderer.clone(), Locale::En),
        )),
        update_install_in_flight: std::sync::atomic::AtomicBool::new(false),
    });

    let token = GenerationToken::new();
    let g1 = token.next();
    assert!(token.is_latest(g1));

    // R2-B: the revision is now allocated by the SYNC caller (mirroring the menu
    // callback) and passed into the core; the core no longer calls begin_switch.
    let rev = app_state.tray.lock().begin_switch();
    let result = linguaray_lib::handle_switch_provider_core(&app_state, &uuid, rev);

    assert!(
        token.is_latest(g1),
        "switch-provider must NOT bump the translation GenerationToken (rev-15 P1-3 / rev-16 P1-3 / rev-18-1 SYNC core)"
    );

    assert!(result.is_ok(), "switch to an existing provider succeeds: {:?}", result);
    let db_read = app_state.db.read().clone().expect("db slot Some");
    let selection = db_read
        .with_conn(|conn| db_providers::read_active_selection(conn))
        .expect("read_active_selection");
    assert_eq!(
        selection.primary,
        Some(uuid.clone()),
        "the switch core wrote primary_uuid = the switched provider's uuid"
    );

    {
        let c = app_state.tray.lock();
        assert_eq!(c.switch_error_rev(), None, "a successful switch clears switch_error_rev");
        assert_eq!(c.current_state(), linguaray_lib::tray_state::TrayVisualState::Normal);
        assert!(c.switch_revision() >= 1, "begin_switch bumped switch_revision");
    }
    let _ = g1;

    // FAILURE path: switching to an UNKNOWN uuid leaves the DB unchanged AND
    // surfaces the error in the tray.
    let token2 = GenerationToken::new();
    let g2 = token2.next();
    let fail_rev = app_state.tray.lock().begin_switch();
    let fail_result =
        linguaray_lib::handle_switch_provider_core(&app_state, "nonexistent-uuid", fail_rev);
    assert!(fail_result.is_err(), "switch to an unknown uuid fails");
    assert!(token2.is_latest(g2), "the failed switch also does NOT bump the token");
    let selection_after_fail = db_read
        .with_conn(|conn| db_providers::read_active_selection(conn))
        .expect("read_active_selection after fail");
    assert_eq!(
        selection_after_fail.primary,
        Some(uuid),
        "the failed switch did NOT change the DB primary (transaction rolled back)"
    );
    {
        let c = app_state.tray.lock();
        assert_eq!(
            c.switch_error_rev(),
            Some(c.switch_revision()),
            "a failed switch sets switch_error_rev = the (latest) revision"
        );
        assert_eq!(
            c.current_state(),
            linguaray_lib::tray_state::TrayVisualState::Error,
            "a failed switch drives the tray to Error (red dot)"
        );
    }
}

#[test]
fn switch_arm_source_has_no_gen_next_call() {
    /// rev-22-2: extract a function body by its exact signature prefix.
    fn extract_function_body<'a>(src: &'a str, signature: &str) -> &'a str {
        let start = src.find(signature)
            .unwrap_or_else(|| panic!("rev-22-2: expected `{signature}` in lib.rs"));
        let brace_offset = src[start..].find('{')
            .unwrap_or_else(|| panic!("rev-22-2: expected `{{` after `{signature}`"));
        let brace_start = start + brace_offset;
        let mut depth = 0i32;
        let mut end = brace_start + 1;
        for (i, ch) in src[brace_start..].char_indices() {
            match ch {
                '{' => depth += 1,
                '}' => {
                    depth -= 1;
                    if depth == 0 {
                        end = brace_start + i + 1;
                        break;
                    }
                }
                _ => {}
            }
        }
        assert!(
            depth == 0,
            "rev-22-2: unbalanced braces in the body of `{signature}`"
        );
        &src[start..end]
    }

    let src = include_str!("../src/lib.rs");
    let providers_src = include_str!("../src/commands/providers.rs");

    let handler_body = extract_function_body(src, "fn handle_tray_menu_event(");
    let core_body = extract_function_body(providers_src, "pub fn handle_switch_provider_core(");
    let wrapper_body = extract_function_body(providers_src, "pub fn handle_switch_provider(");

    let handler_preview: String = handler_body.chars().take(500).collect();
    let core_preview: String = core_body.chars().take(500).collect();
    let wrapper_preview: String = wrapper_body.chars().take(500).collect();

    assert!(
        !core_body.contains("session.gen")
            && !core_body.contains(".gen.next()")
            && !core_body.contains(".gen .next()"),
        "rev-22-3: handle_switch_provider_core must NOT acquire the translation GenerationToken / call `.gen.next()` (switch is decoupled from translation gen — rev-15 P1-3 / rev-16-1) (first 500 chars of core body: {core_preview})"
    );
    assert!(
        !core_body.contains(".await"),
        "rev-22-3: handle_switch_provider_core must be SYNC (set_active_primary_core is SYNC) — no `.await` in its body (rev-18-1) (first 500 chars of core body: {core_preview})"
    );

    assert!(
        !wrapper_body.contains("pub async fn"),
        "rev-22-3: handle_switch_provider must be `pub fn` (SYNC), not `pub async fn` (rev-18-1) (first 500 chars of wrapper body: {wrapper_preview})"
    );
    assert!(
        wrapper_body.contains("spawn(async move"),
        "Task A2 (P1-2): handle_switch_provider must detach the now-async tray refresh via `spawn(async move {{ ... refresh_tray_if_available(...).await; }})`. The wrapper itself stays SYNC (`pub fn`, runs inside spawn_blocking, does the DB write synchronously — rev-18-1 SYNC model) and therefore cannot `.await` directly; spawning the async refresh is the only legitimate way to drive it from this sync context. (first 500 chars of wrapper body: {wrapper_preview})"
    );

    assert!(
        !handler_body.contains(".gen.next()")
            && !handler_body.contains(".gen .next()")
            && !handler_body.contains("session.gen"),
        "rev-22-3: the tray.switch- arm in handle_tray_menu_event must NOT call `.gen.next()` / acquire the translation GenerationToken (rev-16 P1-3 / rev-18-1) (first 500 chars of handler body: {handler_preview})"
    );
    assert!(
        !handler_body.contains("spawn(async move"),
        "rev-22-3: the tray.switch- arm must NOT spawn(async move {{ ... .await }}) — it uses spawn_blocking for a SYNC fn (rev-18-1) (first 500 chars of handler body: {handler_preview})"
    );

    assert!(
        src.contains("build_switch_provider_submenu"),
        "rev-19 P2-1: the dynamic Switch Provider submenu builder `build_switch_provider_submenu` must exist in lib.rs"
    );
    assert!(
        src.contains("\"tray.switch-{uuid}\"") || src.contains("tray.switch-{uuid}"),
        "rev-19 P2-1: the submenu must format item ids as `tray.switch-{{uuid}}` (one per provider)"
    );
}

// ─── 11. rev-16 (P1-3): switch revision ordering ────────────────────────────

#[test]
fn two_concurrent_switches_second_wins() {
    let mut c = test_controller();
    let rev_a = c.begin_switch();
    let rev_b = c.begin_switch();
    c.finish_switch(rev_a, true);
    c.finish_switch(rev_b, false);
    assert_eq!(
        c.switch_error_rev(),
        Some(rev_b),
        "the LATEST revision (B, failed) wins — its error is recorded"
    );
    assert_eq!(recompute_pure(&c), TrayVisualState::Error);
}

#[test]
fn stale_switch_result_ignored() {
    let mut c = test_controller();
    let rev1 = c.begin_switch();
    let rev2 = c.begin_switch();
    c.finish_switch(rev2, true);
    assert_eq!(c.switch_error_rev(), None);
    assert_eq!(recompute_pure(&c), TrayVisualState::Normal);
    c.finish_switch(rev1, false);
    assert_eq!(
        c.switch_error_rev(),
        None,
        "stale revision's late result is ignored — the latest revision's success stands"
    );
    assert_eq!(recompute_pure(&c), TrayVisualState::Normal);
}

// ─── 12. rev-A4 (P2-2): pulse first frame Dimmed → Normal (not Dimmed → Dimmed) ─
// NOTE: the assertions snapshot the call count BEFORE each tick and assert on the
// frame at that exact index, so the test pins the SPECIFIC frame each tick
// produced (rather than `.any()` over the full history, which is satisfied by the
// constructor's idle-state Normal frame and could not catch the P2-2 bug).

#[test]
fn pulse_sequence_is_dimmed_then_normal_then_dimmed() {
    let (mut c, renderer, notify_rx) = controller_with_notify();
    c.begin_translation(1);
    // Initial render (the begin_translation frame) must be Dimmed — it is the
    // last frame in the history at this point.
    let n0 = renderer.calls().len();
    assert!(
        renderer
            .calls()
            .last()
            .map(|(icon, _)| icon.is_dimmed())
            .unwrap_or(false),
        "initial render must be Dimmed, got {:?}",
        renderer.calls()
    );
    // Tick 1: must render Normal (not repeat Dimmed). The frame at index n0 is
    // the first frame the worker pushed (deterministic: the worker's first
    // recv_timeout(2ms) cannot have elapsed before this <1µs snapshot).
    match notify_rx.recv_timeout(std::time::Duration::from_millis(50)) {
        Ok(PulseEvent::Tick) => {}
        other => panic!("expected Tick, got {other:?}"),
    }
    let calls1 = renderer.calls();
    assert!(
        calls1.get(n0).map(|(icon, _)| icon.is_normal()).unwrap_or(false),
        "tick 1 must render Normal (P2-2), got {calls1:?}"
    );
    // Tick 2: Dimmed.
    let n1 = renderer.calls().len();
    match notify_rx.recv_timeout(std::time::Duration::from_millis(50)) {
        Ok(PulseEvent::Tick) => {}
        other => panic!("expected Tick, got {other:?}"),
    }
    let calls2 = renderer.calls();
    assert!(
        calls2.get(n1).map(|(icon, _)| icon.is_dimmed()).unwrap_or(false),
        "tick 2 must render Dimmed, got {calls2:?}"
    );
    c.finish_translation(1, true);
}
