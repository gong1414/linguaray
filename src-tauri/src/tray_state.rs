//! Task A5 (rev-11 → rev-19): the pure-Rust tray visual-state controller.
#![allow(clippy::doc_lazy_continuation)]
//!
//! This module drives the Surface 04 (pages/04-tray-menu.md) Normal / Active /
//! Error icon + tooltip states WITHOUT routing through the Web frontend. The
//! translate / clipboard flows call the [`TranslationGuard`] (rev-13 P1-2 /
//! rev-15 finish_translation merge / rev-16-2 gen guard) / [`TrayStateController`]
//! reducer (rev-12 P1-3), which owns the active-translation counter, the
//! generation-tagged error (translation flow), the switch-flow `switch_revision`
//! + `switch_error_rev` (rev-16-3 — replaces rev-15's sticky `has_error` bool so
//! concurrent switch completions are ordered by revision), the current visual
//! state, and the `PulseWorker`, and resolves the highest-priority state via
//! `recompute`. The switch-provider flow calls `begin_switch()` →
//! `finish_switch(rev, success)` (rev-16-1 distinct method names — NO overloading;
//! rev-16-3 revision-tagged) directly — it does NOT touch the translation
//! `GenerationToken` (rev-15 P1-3). The Update-available state is deferred to
//! R5/R6 per user-approved scope decision — the [`TrayVisualState::UpdateAvailable`]
//! variant is retained so the priority ordering is unit-testable, but `recompute`
//! NEVER produces it.
//!
//! rev-12 corrections over rev-11:
//! - P1-1: ActiveTranslation drives a REAL icon frame-switch pulse.
//! - P1-2: Error overlays a red-dot on the BASE icon (composited in build.rs).
//! - P1-3: TrayStateController reducer replaces the direct-override.
//! - P2:   tooltip text is localized via tray_tooltip_text(state, locale).
//!
//! rev-13 corrections over rev-12:
//! - P1-1: the `tray` field lives on `AppState`; all call sites use `app_state.tray`.
//! - P1-2: `TranslationGuard` RAII guarantees begin/end pairing on every return.
//! - P1-3: `error_gen: Option<u64>` is generation-aware (a newer gen's Retry
//!   success clears an older gen's red dot).
//! - P1-4: `visual_epoch` serializes the timer (a stale-epoch tick self-rejects).
//! - P1-5: `trait TrayRenderer` is injectable; `RecordingRenderer` is the test mock.
//!
//! rev-14 corrections over rev-13:
//! - P1-1: SYNCHRONOUS `parking_lot::Mutex` (NOT `tokio::sync::Mutex`). All
//!   controller methods are SYNC (no `async`, no `.await`). `TranslationGuard::drop`
//!   runs `finish_translation` SYNCHRONOUSLY on the calling thread (no detached
//!   spawn) — the RAII guarantee is REAL.
//! - P1-2: `recompute` only swaps the timer/worker when `new_state != current_state`
//!   (Active → Active counter bump does NOT restart the pulse).
//! - P2: `detect_system_locale()` uses `sys_locale::get_locale()` (cross-platform,
//!   NOT `std::env::var("LANG")` which is Unix-only). `TrayStateController` does
//!   NOT derive `Debug`.
//!
//! rev-15 corrections over rev-14 (the load-bearing ones):
//! - P1-1: `PulseWorker` channel-quit — replaces rev-14's infinite `loop { sleep;
//!   render }` + `stop_timer()` `join()` deadlock. The worker's body loops on
//!   `stop_rx.recv_timeout(interval)`; `Ok(())`/`Err(Disconnected)` → return,
//!   `Err(Timeout)` → toggle a frame. `PulseWorker::stop()` = `stop_tx.send(())`
//!   + `handle.take().join()` (the worker returns from `recv_timeout` on the
//!   signal so `join` completes — NO deadlock). `impl Drop for PulseWorker` calls
//!   `stop()`. Leaving `Active` = `pulse_worker.take()` (Drop → stop).
//! - P1-2: `RecordingRenderer` is `#[cfg(any(test, feature = "xproc-test-helper"))]`-gated
//!   (NOT `#[cfg(test)]`, which is invisible to the integration-test crate). The
//!   `lib.rs` re-export is gated identically.
//! - P1-3: Switch Provider does NOT call `session.gen.next()` — `GenerationToken::next()`
//!   ADVANCES the generation (verified concurrency.rs), staling any in-flight
//!   translation. The switch flow is DECOUPLED from the translation generation.
//! - P1-4: SINGLE timer model — `PulseWorker` (channel-quit) only. rev-14's
//!   `visual_epoch` field, `tick_render()` method, `stop_timer()` method, and
//!   "RenderGate" narration are DELETED. The worker holds an independent
//!   `Arc<dyn TrayRenderer>`; the stop barrier is the channel-quit (`send` + `join`).
//! - Housekeeping: `finish_translation(gen, success)` merges `end_translation` +
//!   (if `success`) clear-error + `recompute` into ONE method; `TranslationGuard::drop`
//!   calls it once.
//!
//! rev-16 corrections over rev-15 (the load-bearing ones):
//! - P1-1 (NO function overloading): rev-15 defined TWO methods named `record_error`
//!   (`record_error(&mut self, gen: u64)` for translation + `record_error(&mut self)`
//!   for switch). Rust does NOT support function overloading — this fails to
//!   compile (`E0592: duplicate definitions`). rev-16-1 renames them to DISTINCT
//!   names: `record_translation_error(gen)` (translation) + `begin_switch()` /
//!   `finish_switch(rev, success)` (switch, revision-tagged — replaces the no-gen
//!   `record_error()`/`clear_error()` overloads).
//! - P1-2 (gen guards): rev-15's `finish_translation(gen, true)` unconditionally
//!   cleared `error_gen` — a stale OLDER gen's late success would clear a NEWER
//!   gen's error. rev-16-2 adds `if self.error_gen.is_some_and(|eg| eg <= gen)` to
//!   `finish_translation` AND `if self.error_gen.is_none_or(|eg| gen >= eg)` to
//!   `record_translation_error` (a stale OLDER gen's late error cannot clobber a
//!   NEWER gen's error).
//! - P1-3 (switch revision, replaces rev-15's sticky `has_error: bool`): rev-15's
//!   sticky bool had no revision, so two concurrent switch completions that
//!   re-order would show the wrong final state. rev-16-3 replaces `has_error:
//!   bool` with `switch_revision: u64` (monotonic, incremented by `begin_switch()`)
//!   + `switch_error_rev: Option<u64>`. `begin_switch() -> u64` returns the new
//!   revision; `finish_switch(rev, success)` IGNORES the result if
//!   `rev != switch_revision` (only the latest revision can update state).
//!   `recompute_pure` ORs: `Error iff error_gen.is_some() || switch_error_rev.is_some()`.
//! - P2-1 (notify channel, replaces `thread::sleep` in tests): `PulseWorker::start`
//!   takes an `Option<Sender<()>> notify`; the worker emits `notify.send(())` per
//!   tick; tests `recv_timeout` on it to deterministically wait for N frames.
//!   The `PulseWorker::start` signature is `(renderer, interval, notify)`.
//! - P2-3: the test imports do NOT name `RenderedIcon`/`TrayRenderer` directly
//!   (unused-import clean).
//!
//! rev-17 corrections over rev-16 (4 P1 + 4 P2 — fixing the user audit notes):
//! - P1-2 (PulseEvent enum): rev-16's `notify: Option<Sender<()>>` only sent an
//!   empty signal — tests could not distinguish a Tick from a worker Stopped.
//!   rev-17-2 introduces `pub enum PulseEvent { Tick, Stopped }`; the worker sends
//!   `PulseEvent::Tick` after each frame and `PulseEvent::Stopped` before exiting.
//! - P1-3 (latest_translation_gen guard): rev-16's `record_translation_error(gen)`
//!   guard was only `gen >= error_gen` — a stale OLDER gen's late error (after a
//!   newer gen already began) could still set `error_gen`. rev-17-3 adds a
//!   `latest_translation_gen: u64` field; `record_translation_error` only records
//!   when `gen >= latest_translation_gen` (a stale gen's late error is ignored).
//! - P1-4 (delete record_switch_error/clear_switch_error): rev-16 kept both the
//!   low-level `record_switch_error()`/`clear_switch_error()` AND the
//!   revision-protected `finish_switch(rev, success)` — the former are dead code
//!   (finish_switch fully replaces them). rev-17-4 deletes them.

use std::sync::Arc;

/// Tray visual state priority: `Error > Update > Active > Normal`.
///
/// The variant order is NOT the priority order — the priority is encoded by
/// [`tray_state_priority`], which makes the order explicit and keeps this enum
/// field-free.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TrayVisualState {
    Normal,
    /// Pulse — shown during an in-flight translate. rev-12 (P1-1): a REAL icon
    /// frame-switch pulse — rev-15 (P1-1): the controller's `pulse_worker`
    /// (`Option<PulseWorker>`) starts a background `std::thread` whose body loops
    /// on `mpsc::Receiver::recv_timeout(interval)`, toggling `set_icon_normal` ↔
    /// `set_icon_dimmed` on each `Timeout` (the dimmed variant is the
    /// build-time-generated `tray-active-32.png`). The worker exits via the
    /// channel signal (`stop_tx.send(())` + `join()` — NO infinite-loop + join
    /// deadlock). The localized tooltip ("Translating…"/"翻译中…") is an
    /// auxiliary signal.
    ActiveTranslation,
    /// Red-dot overlay on the tray icon. rev-12 (P1-2): a build-time-composited
    /// PNG — the app default icon (`src-tauri/icons/32x32.png`) with a ~10px
    /// `#DC2626` dot drawn at the top-right. Embedded via
    /// `include_bytes!(concat!(env!("OUT_DIR"), "/tray-error-32.png"))`.
    Error,
    /// Update-available badge — RETAINED so the priority ordering is testable,
    /// but `recompute` NEVER produces it this stage. Deferred to R5/R6 per
    /// user-approved scope decision (the updater backend does not exist).
    UpdateAvailable,
}

/// Priority rank: higher beats lower. `Normal`=0 < `ActiveTranslation`=1 <
/// `UpdateAvailable`=2 < `Error`=3, matching `Error > Update > Active > Normal`.
pub fn tray_state_priority(state: TrayVisualState) -> u8 {
    match state {
        TrayVisualState::Normal => 0,
        TrayVisualState::ActiveTranslation => 1,
        TrayVisualState::UpdateAvailable => 2,
        TrayVisualState::Error => 3,
    }
}

// ─── rev-14: localization (system locale via sys-locale, NOT Settings) ───────

/// UI locale for tray tooltip text. rev-14: read via [`detect_system_locale`]
/// using the `sys-locale` crate (cross-platform) — NOT from `Settings` (which
/// has no `locale` field) and NOT from `std::env::var("LANG")` (Unix-only).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Locale {
    En,
    Zh,
}

/// rev-14: read the system locale via `sys_locale::get_locale()` (macOS
/// CFLocaleCopyCurrent, Windows GetUserDefaultLocaleName, Unix LANG/LC_*).
/// Returns [`Locale::Zh`] if the value starts with `"zh"`, otherwise
/// [`Locale::En`] (including when the detector returns `None`). Does NOT touch
/// `Settings`. Never panics.
pub fn detect_system_locale() -> Locale {
    match sys_locale::get_locale() {
        Some(v) if v.starts_with("zh") => Locale::Zh,
        _ => Locale::En,
    }
}

/// Localized tooltip text for a tray visual state. `Normal` is `"LinguaRay"` in
/// both locales; `ActiveTranslation`/`Error` are translated.
pub fn tray_tooltip_text(state: TrayVisualState, locale: Locale) -> &'static str {
    match (state, locale) {
        (TrayVisualState::Normal, _) => "LinguaRay",
        (TrayVisualState::ActiveTranslation, Locale::En) => "Translating…",
        (TrayVisualState::ActiveTranslation, Locale::Zh) => "翻译中…",
        (TrayVisualState::Error, Locale::En) => "LinguaRay — Error",
        (TrayVisualState::Error, Locale::Zh) => "LinguaRay — 错误",
        // recompute never produces UpdateAvailable this stage; return a stable
        // placeholder so the match is exhaustive without driving a real tooltip.
        (TrayVisualState::UpdateAvailable, _) => "LinguaRay",
    }
}

// ─── rev-13/rev-14 (P1-5): injectable renderer (rev-14: discrete methods) ────

/// The tray rendering surface, abstracted so the controller is testable WITHOUT
/// a real Tauri tray (rev-13 P1-5). Prod: [`TrayIconRenderer`] wraps a
/// `TrayIcon` looked up via `app.tray_by_id("main-tray")`. Test:
/// [`RecordingRenderer`] records every call for assertion (**rev-15 P1-2:
/// `#[cfg(any(test, feature = "xproc-test-helper"))]`-gated** — NOT `#[cfg(test)]`,
/// which is invisible to the integration-test crate).
///
/// rev-14: DISCRETE methods (NOT `set_icon(Option<Image>)` taking an enum) — the
/// renderer DECIDES which embedded PNG / default icon each variant maps to, so
/// the controller never builds an `Image` and the test mock never decodes a PNG.
/// `dyn`-compatible: all methods take `&self` and have no generics.
pub trait TrayRenderer: Send + Sync {
    /// The app default window icon (`app.default_window_icon()`).
    fn set_icon_normal(&self);
    /// The dimmed pulse frame (`tray-active-32.png`).
    fn set_icon_dimmed(&self);
    /// The red-dot error overlay (`tray-error-32.png`).
    fn set_icon_error_dot(&self);
    /// Apply a tooltip.
    fn set_tooltip(&self, text: &str);
}

/// Production renderer: wraps a `tauri::AppHandle`. Looks up the `main-tray` on
/// each call (the tray may be created lazily).
pub struct TrayIconRenderer {
    app: tauri::AppHandle,
}

impl TrayIconRenderer {
    pub fn new(app: tauri::AppHandle) -> Self {
        Self { app }
    }

    /// Helper: look up the main tray and pass it to `f`. Logs + no-ops if absent.
    fn with_tray<F: FnOnce(&tauri::tray::TrayIcon)>(&self, f: F) {
        let Some(tray) = self.app.tray_by_id("main-tray") else {
            log::debug!("TrayIconRenderer: main-tray not present");
            return;
        };
        f(&tray);
    }

    /// Helper: set the icon to the embedded PNG at `bytes`, decoded.
    fn set_icon_bytes(&self, bytes: &'static [u8]) {
        self.with_tray(|tray| match tauri::image::Image::from_bytes(bytes) {
            Ok(img) => {
                if let Err(e) = tray.set_icon(Some(img)) {
                    log::debug!("TrayIconRenderer: set_icon failed: {e}");
                }
            }
            Err(e) => log::debug!("TrayIconRenderer: decode failed: {e}"),
        });
    }
}

impl TrayRenderer for TrayIconRenderer {
    fn set_icon_normal(&self) {
        self.with_tray(|tray| {
            if let Some(icon) = self.app.default_window_icon().cloned() {
                if let Err(e) = tray.set_icon(Some(icon)) {
                    log::debug!("TrayIconRenderer: set_icon(normal) failed: {e}");
                }
            }
        });
    }

    fn set_icon_dimmed(&self) {
        let bytes = include_bytes!(concat!(env!("OUT_DIR"), "/tray-active-32.png"));
        self.set_icon_bytes(bytes);
    }

    fn set_icon_error_dot(&self) {
        let bytes = include_bytes!(concat!(env!("OUT_DIR"), "/tray-error-32.png"));
        self.set_icon_bytes(bytes);
    }

    fn set_tooltip(&self, text: &str) {
        self.with_tray(|tray| {
            if let Err(e) = tray.set_tooltip(Some(text)) {
                log::debug!("TrayIconRenderer: set_tooltip failed: {e}");
            }
        });
    }
}

/// A tagged icon variant the test mock records (rev-14). Prod never builds
/// these — the discrete `TrayRenderer` methods keep the controller free of
/// `Image` construction. The test mock records which method was called.
/// rev-15 P1-2: gated behind `any(test, feature = "xproc-test-helper")` so the
/// integration-test crate (compiled under `--features xproc-test-helper`) sees
/// it and `cargo build` (no feature) does not compile it.
#[cfg(any(test, feature = "xproc-test-helper"))]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RenderedIcon {
    Normal,
    Dimmed,
    ErrorDot,
}

#[cfg(any(test, feature = "xproc-test-helper"))]
impl RenderedIcon {
    pub fn is_dimmed(&self) -> bool {
        matches!(self, RenderedIcon::Dimmed)
    }
    pub fn is_normal(&self) -> bool {
        matches!(self, RenderedIcon::Normal)
    }
    pub fn is_error_dot(&self) -> bool {
        matches!(self, RenderedIcon::ErrorDot)
    }
}

/// Test mock renderer (rev-13 P1-5; **rev-15 P1-2:
/// `#[cfg(any(test, feature = "xproc-test-helper"))]`-gated** — NOT `#[cfg(test)]`,
/// which is invisible to the integration-test crate `src-tauri/tests/tray_state.rs`):
/// records every `set_icon_*`/`set_tooltip` call so the PulseWorker-lifecycle
/// tests can assert the exact frame sequence. Visible to integration tests because
/// the module is `pub` and the test harness compiles with `--features xproc-test-helper`.
#[cfg(any(test, feature = "xproc-test-helper"))]
#[derive(Default)]
pub struct RecordingRenderer {
    calls: std::sync::Mutex<Vec<(RenderedIcon, Option<String>)>>,
}

#[cfg(any(test, feature = "xproc-test-helper"))]
impl RecordingRenderer {
    /// Snapshot of the recorded (icon, tooltip) pairs, in call order.
    pub fn calls(&self) -> Vec<(RenderedIcon, Option<String>)> {
        self.calls.lock().expect("RecordingRenderer poisoned").clone()
    }
}

#[cfg(any(test, feature = "xproc-test-helper"))]
impl TrayRenderer for RecordingRenderer {
    fn set_icon_normal(&self) {
        let mut g = self.calls.lock().expect("RecordingRenderer poisoned");
        g.push((RenderedIcon::Normal, None));
    }
    fn set_icon_dimmed(&self) {
        let mut g = self.calls.lock().expect("RecordingRenderer poisoned");
        g.push((RenderedIcon::Dimmed, None));
    }
    fn set_icon_error_dot(&self) {
        let mut g = self.calls.lock().expect("RecordingRenderer poisoned");
        g.push((RenderedIcon::ErrorDot, None));
    }
    fn set_tooltip(&self, text: &str) {
        // Fold the tooltip into the most recent icon record if its tooltip slot
        // is empty; otherwise append as a no-op icon record.
        let mut g = self.calls.lock().expect("RecordingRenderer poisoned");
        if let Some(last) = g.last_mut() {
            if last.1.is_none() {
                last.1 = Some(text.to_owned());
                return;
            }
        }
        g.push((RenderedIcon::Normal, Some(text.to_owned())));
    }
}

// ─── rev-14 (P1-2 + P1-3 + P1-4): TrayStateController reducer ────────────────

/// The tray visual-state reducer (rev-12 P1-3, rev-13 generation-aware, rev-14
/// SYNC + current_state-gated worker swap, rev-15 PulseWorker + no-gen switch +
/// finish_translation merge, rev-16 NO overloading + gen guards + switch revision,
/// rev-17 latest_translation_gen guard + PulseEvent notify). Owns the active-
/// translation counter, the generation-tagged error (translation flow), the
/// switch-flow `switch_revision` + `switch_error_rev`, the CURRENT resolved
/// visual state, the `PulseWorker`, the injected renderer, and the locale.
/// Stored in `Arc<parking_lot::Mutex<TrayStateController>>` on `AppState`
/// (rev-14: synchronous `parking_lot::Mutex`, NOT `tokio::sync::Mutex`).
///
/// rev-14 P2: does NOT derive `Debug` (holds `Arc<dyn TrayRenderer>`).
pub struct TrayStateController {
    active_translations: u32,
    error_gen: Option<u64>,
    /// rev-17-3: the gen of the MOST RECENT `begin_translation` call.
    latest_translation_gen: u64,
    switch_revision: u64,
    switch_error_rev: Option<u64>,
    current_state: TrayVisualState,
    pulse_worker: Option<PulseWorker>,
    tick_interval: std::time::Duration,
    renderer: Arc<dyn TrayRenderer>,
    notify_tx: Option<std::sync::mpsc::Sender<PulseEvent>>,
    locale: Locale,
    /// rev-19-4: monotonic counter incremented each time `recompute` starts a
    /// new `PulseWorker` (asserted NOT to increase on an Active→Active bump).
    worker_start_count: u32,
}

impl TrayStateController {
    /// Production constructor: wraps the real tray. `locale` is read from the
    /// system here. The tick interval is 800ms. `notify_tx` is `None` in prod.
    pub fn new(app: tauri::AppHandle) -> Self {
        Self::with_renderer_interval_and_notify(
            Arc::new(TrayIconRenderer::new(app)),
            detect_system_locale(),
            std::time::Duration::from_millis(800),
            None,
        )
    }

    /// rev-13 (P1-5): constructor with an injected renderer (test entry point).
    pub fn with_renderer(renderer: Arc<dyn TrayRenderer>, locale: Locale) -> Self {
        Self::with_renderer_interval_and_notify(
            renderer, locale, std::time::Duration::from_millis(800), None,
        )
    }

    /// rev-14/rev-15: constructor with an injected renderer AND a custom tick
    /// interval (test entry point).
    pub fn with_renderer_and_interval(
        renderer: Arc<dyn TrayRenderer>,
        locale: Locale,
        tick_interval: std::time::Duration,
    ) -> Self {
        Self::with_renderer_interval_and_notify(renderer, locale, tick_interval, None)
    }

    /// rev-16 P2-1 / rev-17-2: the SOLE constructor that initializes `notify_tx`.
    /// `new`, `with_renderer`, and `with_renderer_and_interval` all delegate here
    /// passing `None`.
    pub fn with_renderer_interval_and_notify(
        renderer: Arc<dyn TrayRenderer>,
        locale: Locale,
        tick_interval: std::time::Duration,
        notify_tx: Option<std::sync::mpsc::Sender<PulseEvent>>,
    ) -> Self {
        let mut c = Self {
            active_translations: 0,
            error_gen: None,
            latest_translation_gen: 0,
            switch_revision: 0,
            switch_error_rev: None,
            current_state: TrayVisualState::Normal,
            pulse_worker: None,
            tick_interval,
            renderer,
            notify_tx,
            locale,
            worker_start_count: 0,
        };
        c.render();
        c
    }

    // ── test accessors ──────────────────────────────────────────────────────

    pub fn active_translations(&self) -> u32 {
        self.active_translations
    }

    pub fn error_gen(&self) -> Option<u64> {
        self.error_gen
    }

    pub fn latest_translation_gen(&self) -> u64 {
        self.latest_translation_gen
    }

    pub fn switch_revision(&self) -> u64 {
        self.switch_revision
    }

    pub fn switch_error_rev(&self) -> Option<u64> {
        self.switch_error_rev
    }

    pub fn current_state(&self) -> TrayVisualState {
        self.current_state
    }

    /// rev-15: true iff a `PulseWorker` is currently running (i.e. the controller
    /// is in `ActiveTranslation`).
    pub fn is_pulsing(&self) -> bool {
        self.pulse_worker.is_some()
    }

    pub fn worker_start_count(&self) -> u32 {
        self.worker_start_count
    }

    // ── real mutators (drive the tray via recompute — ALL SYNC) ─────────────

    /// A translation started (rev-13: gen-tagged). If `error_gen` belongs to an
    /// OLDER generation, clear it. Then increment + recompute. Does NOT touch the
    /// switch flow. rev-17-3: also updates `latest_translation_gen` to `max(self,
    /// gen)` so a LATE error from an OLDER gen is ignored by
    /// `record_translation_error`.
    pub fn begin_translation(&mut self, gen: u64) {
        if gen > self.latest_translation_gen {
            self.latest_translation_gen = gen;
        }
        if self.error_gen.is_some_and(|e| e < gen) {
            self.error_gen = None;
        }
        self.active_translations = self.active_translations.saturating_add(1);
        self.recompute();
    }

    /// rev-15 (merge) + rev-16-2 (gen guard): finish a translation in ONE atomic
    /// call. Decrements the counter; if `success`, clears `error_gen` ONLY when
    /// `error_gen <= gen` (an OLDER gen's late success must NOT clear a NEWER
    /// gen's error). Always recomputes. Called by `TranslationGuard::drop`.
    pub fn finish_translation(&mut self, gen: u64, success: bool) {
        self.active_translations = self.active_translations.saturating_sub(1);
        // rev-16-2 gen guard: only clear an error that belongs to this gen or an
        // OLDER gen. A newer gen's error must survive a stale older-gen success.
        if success && self.error_gen.is_some_and(|eg| eg <= gen) {
            self.error_gen = None;
        }
        let _ = gen;
        self.recompute();
    }

    /// rev-13 + rev-16-1 (renamed) + rev-16-2 (gen guard) + rev-17-3
    /// (latest_translation_gen guard): record that generation `gen` produced a
    /// TRANSLATION-FLOW error. Sets `error_gen = Some(gen)` ONLY if BOTH
    /// `gen >= latest_translation_gen` AND `gen >= error_gen`.
    pub fn record_translation_error(&mut self, gen: u64) {
        if gen >= self.latest_translation_gen && self.error_gen.is_none_or(|eg| gen >= eg) {
            self.error_gen = Some(gen);
        }
        self.recompute();
    }

    /// rev-16-3: begin a new switch revision. Bumps `switch_revision` and returns
    /// the new value. The switch flow does NOT touch the translation
    /// `GenerationToken`. SYNC.
    pub fn begin_switch(&mut self) -> u64 {
        self.switch_revision = self.switch_revision.saturating_add(1);
        self.switch_revision
    }

    /// rev-16-3: finish a switch revision. If `rev != self.switch_revision`, this
    /// is a STALE/late switch result — IGNORE it. Otherwise set `switch_error_rev`
    /// based on `success`. Recomputes. SYNC.
    pub fn finish_switch(&mut self, rev: u64, success: bool) {
        if rev != self.switch_revision {
            return; // stale switch result — ignore
        }
        self.switch_error_rev = if success { None } else { Some(rev) };
        self.recompute();
    }

    /// Resolve the highest-priority state, and ONLY if it differs from
    /// `current_state`: drop the old `PulseWorker` (if leaving Active), start a
    /// new one (if entering Active), update `current_state`, and `render()`.
    /// `UpdateAvailable` is NEVER produced (deferred to R5/R6). rev-14 P1-2 /
    /// rev-15/rev-16: a counter bump that keeps the state at Active does NOT swap
    /// the worker.
    fn recompute(&mut self) {
        let new_state = recompute_pure(self);
        if new_state == self.current_state {
            return;
        }
        if self.current_state == TrayVisualState::ActiveTranslation {
            // Leaving Active: take() drops the PulseWorker → Drop → stop() →
            // stop_tx.send(()) + join. The worker is DEAD before the new-state
            // render runs (channel-quit barrier, NOT an epoch check).
            self.pulse_worker.take();
        }
        if new_state == TrayVisualState::ActiveTranslation {
            self.pulse_worker = Some(PulseWorker::start(
                self.renderer.clone(),
                self.tick_interval,
                self.notify_tx.clone(),
            ));
            self.worker_start_count = self.worker_start_count.saturating_add(1);
        }
        self.current_state = new_state;
        self.render();
    }

    /// rev-15: the SINGLE sync entry point that writes icon + tooltip based on
    /// `current_state`. Called ONLY by `recompute` (inside the controller's
    /// `&mut self` lock). The `PulseWorker`'s per-tick writes go through its OWN
    /// renderer clone directly (serialized against this `render` by the worker
    /// being stopped BEFORE `recompute` renders a new state).
    fn render(&mut self) {
        match self.current_state {
            TrayVisualState::Normal => {
                self.renderer.set_icon_normal();
                self.renderer.set_tooltip(tray_tooltip_text(self.current_state, self.locale));
            }
            TrayVisualState::ActiveTranslation => {
                // The PulseWorker drives the visible icon swaps on each tick; this
                // initial render sets the first dimmed frame + tooltip for instant
                // feedback.
                self.renderer.set_icon_dimmed();
                self.renderer.set_tooltip(tray_tooltip_text(self.current_state, self.locale));
            }
            TrayVisualState::Error => {
                self.renderer.set_icon_error_dot();
                self.renderer.set_tooltip(tray_tooltip_text(self.current_state, self.locale));
            }
            TrayVisualState::UpdateAvailable => {
                log::warn!(
                    "render(UpdateAvailable) invoked — this state is deferred to R5/R6 per \
                     user-approved scope decision and should not be reached this stage"
                );
            }
        }
    }
}

// ─── rev-15/rev-16/rev-17 (P1-1): PulseEvent + PulseWorker (channel-quit + notify) ──

/// rev-17-2: the events a [`PulseWorker`] emits on its optional `notify` channel.
/// `Tick` is sent after each frame toggle; `Stopped` is sent immediately before
/// the worker thread returns. In prod `notify` is `None` and no events fire.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PulseEvent {
    Tick,
    Stopped,
}

/// rev-15 P1-1 + rev-16 P2-1 + rev-17-2 + rev-19-3: a background pulse worker
/// that exits via an `mpsc` channel signal — NOT an infinite `loop { sleep;
/// render }` whose `join()` would deadlock (the rev-14 bug). Holds an independent
/// `Arc<dyn TrayRenderer>` and toggles dimmed/normal on each `recv_timeout`
/// `Timeout`. `stop()` sends the signal and joins; `Drop` calls `stop()`.
///
/// rev-19-3: the struct does NOT hold a `notify` field — the `notify` Sender
/// passed to `start` is MOVED into the worker thread closure. The struct holds
/// ONLY `stop_tx` + `handle` (no `dead_code` warning).
pub struct PulseWorker {
    stop_tx: std::sync::mpsc::Sender<()>,
    handle: Option<std::thread::JoinHandle<()>>,
}

impl PulseWorker {
    /// Start a new pulse worker. The worker immediately begins toggling the
    /// renderer every `interval` (first tick after one `interval`). `notify` is
    /// `Some` in tests (the test `recv_timeout`s on `PulseEvent::Tick` per frame
    /// and `PulseEvent::Stopped` on exit) and `None` in prod.
    pub fn start(
        renderer: Arc<dyn TrayRenderer>,
        interval: std::time::Duration,
        notify: Option<std::sync::mpsc::Sender<PulseEvent>>,
    ) -> Self {
        let (stop_tx, stop_rx) = std::sync::mpsc::channel();
        // rev-19-3: move (not clone) — the struct no longer holds a notify field.
        let notify_for_thread = notify;
        let handle = std::thread::spawn(move || {
            let mut dimmed = false;
            loop {
                match stop_rx.recv_timeout(interval) {
                    Ok(()) | Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                        if let Some(tx) = notify_for_thread.as_ref() {
                            let _ = tx.send(PulseEvent::Stopped);
                        }
                        return;
                    }
                    Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                        dimmed = !dimmed;
                        if dimmed {
                            renderer.set_icon_dimmed();
                        } else {
                            renderer.set_icon_normal();
                        }
                        if let Some(tx) = notify_for_thread.as_ref() {
                            let _ = tx.send(PulseEvent::Tick);
                        }
                    }
                }
            }
        });
        Self { stop_tx, handle: Some(handle) }
    }

    /// Stop the worker: send the quit signal, then join the handle. The worker
    /// returns from `recv_timeout` on the signal, so `join` completes — NO
    /// deadlock. Idempotent: a second call is a no-op (the handle was taken).
    pub fn stop(&mut self) {
        let _ = self.stop_tx.send(());
        if let Some(h) = self.handle.take() {
            let _ = h.join();
        }
    }
}

impl Drop for PulseWorker {
    fn drop(&mut self) {
        self.stop();
    }
}

/// The pure resolution function (no renderer side-effects) — extracted so the
/// reducer logic is unit-testable. `Error > Active > Normal` (`UpdateAvailable`
/// is never produced — deferred to R5/R6). rev-16-3: reads BOTH `error_gen`
/// (translation flow) AND `switch_error_rev` (switch flow) — either triggers
/// `Error`.
pub fn recompute_pure(c: &TrayStateController) -> TrayVisualState {
    if c.error_gen.is_some() || c.switch_error_rev.is_some() {
        TrayVisualState::Error
    } else if c.active_translations > 0 {
        TrayVisualState::ActiveTranslation
    } else {
        TrayVisualState::Normal
    }
}

// ─── rev-13/rev-14/rev-15 (P1-2): TranslationGuard RAII (synchronous Drop) ────

/// RAII guard guaranteeing `finish_translation` runs exactly once per
/// `begin_translation`, on EVERY return path (early return, `?`, panic).
///
/// Construct AFTER the preflight (text captured + anchor built) so a capture or
/// stale-gen failure does NOT begin a translation that then has to be finished.
/// The constructor calls `begin_translation(gen)`; `Drop` calls
/// `finish_translation(gen, succeeded)` (rev-15 merge — ONE atomic method).
///
/// rev-14/rev-15: SYNCHRONOUS — the controller mutex is `parking_lot::Mutex`,
/// whose `lock()` is a blocking sync call, so `Drop` runs `finish_translation`
/// on the CALLING THREAD before `Drop` returns (no `spawn`, no detached future).
pub struct TranslationGuard<'a> {
    controller: &'a Arc<parking_lot::Mutex<TrayStateController>>,
    gen: u64,
    succeeded: bool,
}

impl<'a> TranslationGuard<'a> {
    /// Begin a translation (gen-tagged). rev-14/rev-15: SYNCHRONOUS.
    pub fn new(controller: &'a Arc<parking_lot::Mutex<TrayStateController>>, gen: u64) -> Self {
        controller.lock().begin_translation(gen);
        Self {
            controller,
            gen,
            succeeded: false,
        }
    }

    /// Mark the guarded translation as succeeded — the guard's `Drop` then calls
    /// `finish_translation(gen, true)`, which clears `error_gen` IF
    /// `error_gen <= gen` (rev-16-2 gen guard). Called on the success branch,
    /// BEFORE the guard drops. Idempotent.
    pub fn mark_success(&mut self) {
        self.succeeded = true;
    }
}

impl Drop for TranslationGuard<'_> {
    fn drop(&mut self) {
        // rev-15 merge: ONE atomic finish_translation call — decrement + (if
        // succeeded) clear error_gen + recompute. No spawn, no detached future.
        let mut c = self.controller.lock();
        c.finish_translation(self.gen, self.succeeded);
    }
}
