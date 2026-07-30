# P1 Fixes (Concurrency / Security / Correctness) — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Fix all 11 P1 issues from the 2026-07-31 review — these are real bugs in merged code (concurrency races, clipboard-loss, error mis-classification, missing timeout, dead default model, non-startup-safe hotkey registration, missing keystore recovery). Plus rewrite the Phase 4 plan (signing/CI/CSP/Windows-ACL were wrong). P2 items (options wiring, chunking, UI gaps, fsync, zeroize completeness, per-provider model) are explicitly OUT of this plan — they go to a separate backlog (functional/hardening, not release-blockers).

**Architecture:** Most fixes are localized. The concurrency cluster (gen-token sync allocation, clipboard mutex + latest-wins across selection AND clipboard paths, atomic keystore read-modify-write) is the spine — fix those together since they share the `Session`/generation machinery. Then the independent fixes (selection cleanup-guard, wire 4xx classification, reqwest timeout, Gemini model, hotkey fault-tolerance, keystore reset/recover UI).

**Tech Stack:** Rust 1.95 · Tauri 2 · `tauri-plugin-single-instance` (new dep — the review established multi-instance breaks the "single-process" keystore-lock assumption) · existing modules.

**Review reference:** the 2026-07-31 review (12 P1 findings; #11 is "rewrite Phase 4 plan" not code). Spec = `docs/superpowers/specs/2026-07-30-islandpot-v1-design.md` rev-4 (approved).

**Facts verified:** `gemini-2.0-flash` shut down 2026-06-01, replace with `gemini-2.5-flash` (Google deprecations page). `gen.next()` is inside `spawn` (lib.rs:299) — race confirmed. `store()` takes no lock (keystore.rs:305). `translate_clipboard` neither locks selection mutex nor allocates gen (lib.rs:138). Wire maps all non-401/403 non-2xx to FallbackEligible (wire.rs:108). No single-instance plugin.

---

## File Structure

**Modify:**
- `src-tauri/src/concurrency.rs` — `GenerationToken` gains `next()` that's called sync at hotkey entry; the token is passed into the async task (not allocated there).
- `src-tauri/src/lib.rs` — (a) allocate gen in the sync handler, pass into spawn; (b) `translate_clipboard` joins the selection mutex + generation latest-wins; (c) reqwest `.timeout(Duration::from_secs(30))`; (d) hotkey registration becomes fault-tolerant (don't `.expect` the app to death on conflict); (e) add `tauri-plugin-single-instance`; (f) new `reset_keystore` + `archive_keystore` commands; (g) propagate the ACTUAL engine id from fallback (service returns it).
- `src-tauri/src/selection_engine.rs` — unified cleanup guard: every failure branch (copy fail, get_text fail) restores via the sequence guard; save text + image.
- `src-tauri/src/clipboard.rs` — add `get_image`/`set_image` (arboard supports it) for the image-restore promise.
- `src-tauri/src/wire.rs` — restrict FallbackEligible to net/timeout/429/5xx/parse; 4xx (400/404/etc) → Config (InvalidModel/invalid-request).
- `src-tauri/src/error.rs` — make `FallbackKind::Timeout` actually constructible; add `ConfigKind::InvalidRequest` for 4xx.
- `src-tauri/src/service.rs` — return the actual engine id (primary vs fallback) — change return to a struct or `Result<(String /*text*/, String /*engine*/), Error>`.
- `src-tauri/src/providers.rs` — `gemini-2.0-flash` → `gemini-2.5-flash`.
- `src-tauri/src/keystore.rs` — new `update_keys<F>(mutator)` doing locked read-modify-write; `reset()` + `archive()`; `store()` takes the in_proc lock; stale-tmp cleanup under lock.
- `src/App.tsx` — `key_status` failure no longer aborts onMount; show fail-closed banner + "Reset keystore" button.

**Create/modify:**
- `src-tauri/tests/selection_engine.rs` — add error-branch restore tests + image-save test.
- `src-tauri/tests/wire.rs` — add 4xx→Config test.
- `src-tauri/tests/keystore.rs` — add `update_keys` concurrency test.

---

## Task 1: Concurrency cluster — sync gen allocation + clipboard/latest-wins sharing + single-instance

**Files:** `concurrency.rs`, `lib.rs`, `Cargo.toml`

This is the spine fix. Per the review, `gen.next()` must happen **synchronously in the handler**, before spawn; `translate_clipboard` must participate in the same selection mutex + latest-wins; and multi-instance must be prevented (single-instance plugin) since the keystore lock model assumes one process.

- [ ] **Step 1: Add `tauri-plugin-single-instance`.** In `Cargo.toml` `[dependencies]` add `tauri-plugin-single-instance = "2"`. In `lib.rs` `run()`, as the FIRST plugin: `.plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| { /* focus the main window */ let _ = app.get_webview_window("main").map(|w| { let _ = w.show(); w.set_focus() }); }))`.

- [ ] **Step 2: Move gen allocation to the sync handler.** In `on_hotkey` (lib.rs:288), call `let gen = state.gen.next();` BEFORE `spawn`, then move `gen` into the async block (remove the `let gen = state.gen.next();` at lib.rs:299). Same for `on_input_hotkey` if it ever translates (it doesn't currently — just shows window; no gen needed there). Confirm `state` is accessible synchronously in the handler (it is — `app.state::<Arc<Session>>()` works in the sync handler).

- [ ] **Step 3: `translate_clipboard` joins selection mutex + latest-wins.** It currently reads the clipboard with NO lock and NO gen. Refactor: allocate a gen, take the selection lock ONLY around the clipboard read (so it can't read a sentinel mid-selection-capture), then do the translate with gen checks like the hotkey path. Concretely, in `translate_clipboard`:
```rust
let gen = state.gen.next();
let text = {
    let _g = state.gen.selection_lock();
    clipboard::get_text()?
};
if text.trim().is_empty() { return Err("clipboard empty".into()); }
// ... translate_with_fallback ...
// before showing result/error: if !state.gen.is_latest(gen) { return Ok(()); }
```
This closes the race where clipboard-translate reads `__islandpot_sel_*__` mid-selection-capture, and prevents the two paths clobbering one popup.

- [ ] **Step 4: cargo check + cargo test.** No regression. Commit:
```bash
git checkout -b p1-fixes && git add -A && git -c user.name=daoyu -c user.email=daoyu@local commit -m "fix(concurrency): sync gen allocation + clipboard shares mutex/latest-wins + single-instance"
```

---

## Task 2: Atomic keystore read-modify-write (`update_keys`) + lock in store() + reset/archive

**Files:** `keystore.rs`, `lib.rs`

`set_key`/`delete_key` do `load()` then a separate unlocked `store()` — concurrent commands interleave and clobber. Add `update_keys` that holds the in_proc lock + (future) cross-proc lock for the whole RMW; `store()` takes the lock; add `reset()`/`archive()`.

- [ ] **Step 1: `update_keys<F>` in keystore.rs.**
```rust
/// Atomic read-modify-write under the in-proc lock. The mutator receives the
/// current keys map and returns the new one. Stale-tmp cleanup also happens here
/// (under lock). This is the only sanctioned way to mutate the keystore.
pub fn update_keys<F>(&self, mutator: F) -> Result<(), KeystoreError>
where F: FnOnce(&mut serde_json::Value) {
    let _g = self.in_proc.lock();
    // stale-tmp cleanup under the lock
    let tmp = self.dir.join(TMP);
    if tmp.exists() { let _ = std::fs::remove_file(&tmp); }
    let mut keys = if self.file().exists() {
        let bytes = std::fs::read(self.file())?;
        let env: Envelope = serde_json::from_slice(&bytes)
            .map_err(|e| KeystoreError::Envelope(format!("malformed: {e}")))?;
        decrypt(&env, IdentitySource::CURRENT)?
    } else { serde_json::json!({}) };
    if !keys.is_object() { keys = serde_json::json!({}); }
    mutator(&mut keys);
    // store-in-place under the SAME lock (store() itself must NOT re-lock — see step 2)
    self.store_locked(&keys)?;
    Ok(())
}
```
- [ ] **Step 2: split `store()` into a locked entry + internal `store_locked`.** `store()` currently takes the in_proc lock — but `update_keys` already holds it; re-entrant locking with parking_lot deadlocks. Make `store_locked` (no lock, called by update_keys under the held lock) and have `store()` (public) take the lock then call `store_locked`. `set_key`/`delete_key` switch to calling `update_keys`.

- [ ] **Step 3: `reset()` + `archive()`.**
```rust
/// Move keystore.json to keystore.json.broken-<ts> (user-initiated recovery).
pub fn archive(&self) -> Result<PathBuf, KeystoreError> { ... rename ... }
/// Delete the keystore entirely (fresh start).
pub fn reset(&self) -> Result<(), KeystoreError> { ... remove keystore.json + tmp ... }
```

- [ ] **Step 4: lib.rs commands.** `set_key`/`delete_key` → use `state.keystore.update_keys(|k| { ... })`. Add `reset_keystore` + `archive_keystore` commands.

- [ ] **Step 5: test.** `tests/keystore.rs` — spawn two concurrent `update_keys` (one adds "a", one adds "b"); assert BOTH keys present (no clobber). Use `std::thread::spawn` sharing a `Keystore` (it's `Send + Sync` via Mutex).

- [ ] **Step 6: cargo test + commit.**
```bash
git add -A && git -c user.name=daoyu -c user.email=daoyu@local commit -m "fix(keystore): atomic update_keys (locked RMW) + reset/archive + no clobber"
```

---

## Task 3: Selection — unified cleanup guard + image save/restore

`copy()?` failure leaves the sentinel; `get_text()?` failure doesn't restore; image-only clipboard is lost via `unwrap_or_default()`.

- [ ] **Step 1: extend `ClipboardLike`** with image: add `fn get_image(&self) -> Result<Option<Vec<u8>>, String>` and `fn set_image(&self, img: &[u8]) -> Result<(), String>`. Implement in clipboard.rs via arboard (`arboard::Image`).

- [ ] **Step 2: rewrite `capture` with a cleanup guard.** Save text AND image (if present). Build a `restore_if_owned` closure that runs on EVERY exit path (success, NoSelection, copy-fail, get_text-fail). Use a guard struct (Drop) or explicit calls at each return. The sentinel must be restored-from unless a newer writer landed.

- [ ] **Step 3: tests.** Add: (a) copy() returns Err → saved restored, no sentinel left; (b) get_text() returns Err after successful copy → saved restored; (c) image-only clipboard → image restored (text path got None). Update the fake clipboard to support image ops.

- [ ] **Step 4: cargo test + commit.**
```bash
git add -A && git -c user.name=daoyu -c user.email=daoyu@local commit -m "fix(selection): restore on ALL failure branches + image save/restore"
```

---

## Task 3b: §B AX-first capture + Accessibility onboarding (review P1 #5)

**Files:** `selection.rs`, `lib.rs`, `App.tsx`/`Popup.tsx` (+ a small onboarding affordance)

Currently `selection.rs` only simulates Cmd/Ctrl+C. The spec §B (rev-4) mandates a **hybrid**: macOS AXUIElement `kAXSelectedTextAttribute` direct-read FIRST, simulated-copy fallback. Plus the Accessibility permission must be requested + user-guided on first launch, and capture errors must surface somewhere the user actually sees (not just a popup that isn't shown yet).

- [ ] **Step 1: Vendor `get-selected-text`** (or re-implement the macOS AX read) into `src-tauri/src/selection.rs` / a new `src-tauri/src/a11y.rs`. The macOS AX path: get the system-wide focused UI element (`AXUIElementCopySystemSetting`/`AXUIElementCreateApplication` of the frontmost app via `NSWorkspace` → pid → `AXUIElementCreateApplication`), then `AXUIElementCopyAttributeValue` with `kAXSelectedTextAttribute`. Read the upstream `yetone/get-selected-text` macOS source for the exact calls; port to Rust (via the `accessibility` crate OR raw `objc2`/`core-foundation` FFI against the ApplicationServices framework). Prefer the `accessibility` crate if it exposes `kAXSelectedTextAttribute`; else raw FFI.
  - `capture_selection` becomes: try AX read first → on success return it (NO clipboard touch at all — cleanest); on AX failure/empty, fall back to the existing sentinel simulate-copy path (Task 3).

- [ ] **Step 2: Accessibility permission check + onboarding.** Add `a11y_enabled() -> bool` (macOS: `AXIsProcessTrusted` from ApplicationServices). In `run()` setup, if not trusted, surface an onboarding state to the main window: a banner "IslandPot needs Accessibility to read your selection — [Open System Settings]" (the button opens `x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility`). Re-check on focus. Non-macOS: always enabled (Windows uses simulate-copy only).

- [ ] **Step 3: surface capture errors visibly.** When AX+copy both fail / not trusted / NoSelection, the error must go somewhere the user sees — since the popup isn't shown, either (a) show the popup with the error, or (b) a main-window status. Pick (a): popup::error shows even on NoSelection/failure (currently NoSelection just hides — change to show a brief "no selection / permission needed" popup, auto-hide after a few seconds, OR surface via the main window). Document the chosen behavior.

- [ ] **Step 4: tests.** The AX path is OS-FFI (manual-only per §I), BUT the *fallback decision* (AX-empty → sentinel path) is testable: structure `capture_selection` so the AX-attempt is behind an injectable trait (like `ClipboardLike`), and test that an AX "empty/Err" result routes to the simulate-copy fallback. Add that test.

- [ ] **Step 5: cargo check + test + commit.**
```bash
git add -A && git -c user.name=daoyu -c user.email=daoyu@local commit -m "feat(selection): macOS AX-first capture (§B hybrid) + Accessibility onboarding"
```

---

## Task 4: Wire — restrict FallbackEligible + Timeout

**Files:** `wire.rs`, `error.rs`, `tests/wire.rs`

All non-401/403 non-2xx → FallbackEligible is wrong (400/404 are config errors; retrying with a 2nd provider sends text needlessly).

- [ ] **Step 1: error.rs.** Make `FallbackKind::Timeout` reachable (it exists but is never built — reqwest timeout will produce it). Add `ConfigKind::InvalidRequest { status }` for 4xx (covers 400/404/invalid-model/etc).

- [ ] **Step 2: wire.rs status mapping.** net/timeout → `FallbackEligible(Timeout)` (timeout needs the reqwest timeout from Task 6 to actually fire; but classify it now). 429/5xx → `FallbackEligible(ProviderStatus)`. 401/403 → `Config(AuthFailed)`. **other 4xx (400/404/etc) → `Config(InvalidRequest { status })`** — NOT fallback. parse-fail → `FallbackEligible(Parse)`.

- [ ] **Step 3: test.** `tests/wire.rs` — add: 404 → `Config(InvalidRequest)` (assert NOT FallbackEligible). 400 → Config. (429/5xx/401 already covered.)

- [ ] **Step 4: cargo test + commit.**
```bash
git add -A && git -c user.name=daoyu -c user.email=daoyu@local commit -m "fix(wire): 4xx → Config (no needless fallback); classify Timeout"
```

---

## Task 5: Gemini default model + actual-engine-id return

**Files:** `providers.rs`, `service.rs`, `lib.rs`

- [ ] **Step 1: providers.rs.** `gemini-2.0-flash` → `gemini-2.5-flash` (2.0 shut down 2026-06-01).

- [ ] **Step 2: service returns the actual engine.** Change `translate_with_fallback` (and `translate`) to return `Result<Translation, Error>` where `struct Translation { text: String, engine: String }` — `engine` = primary preset id on success, fallback engine id when the fallback produced it. Update lib.rs call sites to use `result.engine` for the popup tag (not the hardcoded `preset.id`).

- [ ] **Step 3: update fallback test** (tests/fallback.rs) to assert `engine == "fake"` when the fallback fired, `engine == primary` otherwise.

- [ ] **Step 4: cargo test + commit.**
```bash
git add -A && git -c user.name=daoyu -c user.email=daoyu@local commit -m "fix: Gemini model 2.0→2.5 (2.0 retired); return actual producing engine id"
```

---

## Task 6: reqwest 30s timeout

**Files:** `lib.rs`

- [ ] **Step 1:** In `run()` setup, the `reqwest::Client::builder()` chain: add `.timeout(std::time::Duration::from_secs(30))` (and `.connect_timeout(...)` if desired). This makes `FallbackKind::Timeout` (Task 4) actually fire on a hung connection.

- [ ] **Step 2: cargo check + test + commit.**
```bash
git add -A && git -c user.name=daoyu -c user.email=daoyu@local commit -m "fix(http): 30s request timeout on shared client"
```

---

## Task 7: Hotkey fault-tolerance (don't kill startup on conflict)

**Files:** `lib.rs`

- [ ] **Step 1:** The global-shortcut registration currently propagates errors to `.run().expect(...)`. Wrap registration so a conflict (another app owns the shortcut) logs + continues, and surfaces a "shortcut conflict — rebind" state to the UI rather than crashing. Match the real plugin API for catching the registration error (it may be a Result from `Builder::build` or from a post-setup `register`). 

- [ ] **Step 2:** (If the plugin makes per-shortcut registration a runtime call rather than build-time, register in `setup()` and catch per-shortcut errors.) Report what the real API allows. The KEY behavior: a shortcut conflict must not prevent app startup.

- [ ] **Step 3: cargo check + test + commit.**
```bash
git add -A && git -c user.name=daoyu -c user.email=daoyu@local commit -m "fix(hotkey): tolerate shortcut conflict (no startup crash)"
```

> NOTE: full rebindability (user-configurable shortcuts) is a P2/feature, NOT this task. This task only ensures a conflict doesn't brick startup. Configurable rebinding goes to backlog.

---

## Task 8: Keystore fail-closed recovery UI

**Files:** `lib.rs`, `App.tsx`

- [ ] **Step 1:** `key_status` (lib.rs) currently propagates the keystore error → frontend onMount throws → blank. Make `key_status` return a structured result (or catch + return an empty map + a separate `keystore_health` command) so onMount doesn't abort. Add a `keystore_health` command returning `Ok` / `Corrupt` / `AuthFailed`.

- [ ] **Step 2:** App.tsx — on `keystore_health` failure, show a banner: "Keystore unreadable (reason). [Reset keystore]" calling `reset_keystore` (Task 2). Don't crash onMount.

- [ ] **Step 3:** `pnpm build` + commit.
```bash
git add -A && git -c user.name=daoyu -c user.email=daoyu@local commit -m "fix(ui): keystore fail-closed recovery (health + reset, no onMount abort)"
```

---

## Task 9: Rewrite Phase 4 plan (signing / CSP / Windows ACL / Intel)

**Files:** `docs/superpowers/plans/2026-07-30-phase4-windows-parity-packaging.md` (rewrite), `src-tauri/tauri.conf.json` (CSP — as part of the rewrite, not now)

This is the doc rewrite (review #11). NOT executed now — only the plan is corrected; execution comes after P1 code fixes pass review.

The rewrite must:
- **Remove `icacls`.** Windows ACL = `SetNamedSecurityInfoW` with a DACL of one explicit ACE for the current user's SID (full control, including delete) + `PROTECTED_DACL_SECURITY_INFORMATION` (no inheritance). Add real tests (dir, first-create, update, inheritance-off).
- **Fix signing config.** Remove `${APPLE_SIGNING_IDENTITY}` from JSON (Tauri doesn't shell-substitute it). Drive signing purely via env vars. CI: import the certificate into a temp keychain (`APPLE_CERTIFICATE`/`APPLE_CERTIFICATE_PASSWORD` → `security import`), pass `APPLE_SIGNING_IDENTITY` etc. Windows: add `TAURI_SIGNING_PRIVATE_KEY`/`TAURI_SIGNING_PRIVATE_KEY_PASSWORD` for MSI/NSIS signing + Smartscreen (or document EV-certs). 
- **Add CSP + per-window minimal capabilities** as an explicit Phase-4 hardening task (csp: null → a real policy; split capabilities by window; drop unused opener/global-shortcut/store where not needed).
- **Add Intel macOS** (`x86_64-apple-darwin`) to the matrix OR universal2 bundle.
- **fsync** in the atomic-write sequence (file sync_all before rename; macOS dir fsync) — note this overlaps P2 but belongs with the Windows-write-through task.

Commit the rewritten plan only (no code in this task).

```bash
git add docs/superpowers/plans/2026-07-30-phase4-windows-parity-packaging.md && git -c user.name=daoyu -c user.email=daoyu@local commit -m "plan(phase4): rewrite — real Windows ACL API, correct signing/CI, CSP, Intel, fsync"
```

---

## Task 10: Final review + merge

- [ ] **Step 1:** opus code-review of all P1 code fixes (Tasks 1-8) against the review's findings — confirm EACH P1 item is genuinely closed, with tests.
- [ ] **Step 2:** Address any blockers, then merge `p1-fixes` to main.
- [ ] **Step 3:** At this point the rewritten Phase 4 plan (Task 9) is ready for execution, but DO NOT execute Phase 4 until the user re-reviews/approves the fixes.

---

## Self-Review (run after writing; fix inline)

- **Review coverage:** P1 #1 (keystore RMW) → Task 2. #2 (selection restore all-branches + image) → Task 3. #3 (clipboard shares mutex+gen) → Task 1. #4 (sync gen) → Task 1. #5 (AX-first capture + onboarding) → **GAP: not in this plan.** The vendored `get-selected-text` AX-first path + Accessibility onboarding is a larger body of work than a "fix" — it's the real §B implementation (currently we have simulate-copy only). This is genuinely P1 (rich-text/no-copy apps won't work) but it's substantial. **Decision: surface to the user** — either include a Task for it (vendoring + AX + onboarding, ~1 task's worth) or acknowledge it as a known gap and prioritize. The review is right that it's P1; I'm flagging it for explicit triage rather than silently dropping.
- #6 (keystore recovery) → Task 8. #7 (timeout) → Task 6. #8 (4xx classification) → Task 4. #9 (Gemini model) → Task 5. #10 (hotkey conflict) → Task 7. #11 (Phase 4 plan rewrite) → Task 9. #12 (Phase 4 icacls/signing) → folded into Task 9.
- **P2 explicitly deferred (separate backlog):** fsync (note in Phase 4 rewrite), envelope-size cap, full zeroize (Value/Vec/derived key), per-engine chunking, options/model wiring, actual-engine-id (kept — that's correctness), fallback_engine + dict UI, Intel build (in Phase 4).
- **Placeholder scan:** Task 7 step 2 acknowledges uncertainty about the real plugin API for runtime registration — resolve at execution. Task 9 is a doc rewrite with concrete content. The AX/onboarding gap is surfaced, not hidden.
- **Ordering:** Concurrency (Task 1) before the others that touch gen/mutex. Keystore (Task 2) before the UI recovery (Task 8) since it needs reset/archive.

## Execution Handoff

Plan saved to `docs/superpowers/plans/2026-07-31-p1-fixes-concurrency-security-correctness.md`. **ONE OPEN QUESTION for the user before execution** (the §B AX/onboarding item — see Self-Review gap). Subagent-Driven recommended once that's resolved.
