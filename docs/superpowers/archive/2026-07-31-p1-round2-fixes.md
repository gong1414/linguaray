Archived-on: 2026-08-14 · reason: superseded by linguaray-plugin-core-design / completed, see git history

# P1 Round-2 Fixes + Phase 4 rev-3 Plan — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development. Steps use checkbox (`- [ ]`) syntax for tracking.

**Status:** the 2026-07-31 round-2 review found the round-1 P1-fix was NOT fully closed (6 code P1s remain) AND Phase 4 plan rev-2 was un-executable. This plan addresses both: (Part A) fix the 6 remaining code P1s, (Part B) rewrite Phase 4 to rev-3. **Order per reviewer:** code fixes → Phase 4 rev-3 → re-review → manual E2E → Phase 4 execution.

**Why round-1 under-delivered (honest note):** I declared "11 P1 closed, APPROVED" but the final reviewer caught that (a) OS image-restore is still a no-op, (b) hotkey registration's real failure point is the plugin's setup-time `manager.register` (not `with_shortcut` parse), (c) `new()` cleans tmp outside the lock, (d) Reset deletes unrecoverably vs the §A archive protocol, (e) AX is raw FFI not the spec-decided vendored `get-selected-text`, (f) the xproc test is same-process only. Each is a real miss. This plan closes them.

**Spec reference:** `docs/superpowers/specs/2026-07-30-linguaray-v1-design.md` rev-4 (§A, §B "vendor get-selected-text"). **Facts verified:** tauri-plugin-global-shortcut 2.3.2 registers in `setup` via `manager.register(shortcut)?` (lib.rs:398) → propagates to `.run().expect()`; the `global_hotkey::GlobalHotKeyManager` has runtime `register`/`unregister` we can call post-setup; `arboard` supports `Image` get/set; Tauri updater keys ≠ Windows Authenticode; `macos-13` retired 2025-12-04.

---

## Part A — 6 remaining code P1 fixes

### Task A1: Real OS image restore (P1 #1)
**Files:** `src-tauri/src/clipboard.rs`, `src-tauri/src/selection.rs`, `src-tauri/src/selection_engine.rs`
- [ ] `clipboard.rs`: add `get_image() -> Result<Option<Vec<u8>>>` + `set_image(&[u8])` via `arboard::Clipboard::get_image`/`set_image` (`arboard::Image { width, height, bytes }` → flatten/restore RGBA dimensions; store width/height in a small header for round-trip).
- [ ] `selection.rs`: `OsClipboard` impls `get_image`/`set_image` (remove the TODO no-op).
- [ ] `selection_engine.rs` `Saved::restore_if_owned`: the `else if` makes text+image mutually exclusive — restore BOTH if both were present (text set, then image set, in the right order for the OS clipboard; or whichever the OS supports). Verify the image-only test still passes and add a text+image restore test.
- [ ] Tests: extend the Fake + a real-arboard round-trip test where feasible (arboard on a headless test env may not have a clipboard — if so, test the Saved::restore_if_owned logic with a Fake that tracks both, asserting both restored). Commit.

### Task A2: Hotkey registration fault-tolerant (P1 #2) — the REAL fix
**Files:** `src-tauri/src/lib.rs`
- [ ] The real registration is `manager.register(shortcut)` in the plugin's `setup`, NOT `with_shortcut`. Parse-time tolerance (round-1) is insufficient. Switch to **post-setup runtime registration**: do NOT pass shortcuts to the Builder; instead, in the app `setup()` (after the plugin is initialized), call `app.global_shortcut().register("Alt+Space", handler)` and `.register("Ctrl+Space", handler)` individually, catching each `Result` — a conflict logs + continues, the app stays running. Verify the resolved tauri-plugin-global-shortcut exposes `GlobalShortcutExt::register` (or `on_shortcut`) on the AppHandle at runtime; if it only offers builder-time, fall back to wrapping the plugin `setup` registration in a way that catches per-shortcut (likely not possible) — report which API you use. The KEY behavior: one shortcut conflict must NOT prevent app startup or the other shortcut.
- [ ] Commit.

### Task A3: stale-tmp cleanup under the cross-process lock (P1 #3)
**Files:** `src-tauri/src/keystore.rs`
- [ ] Move the stale-tmp deletion OUT of `new()` (it currently runs before any lock — a second instance could delete the first's in-progress tmp). Do the cleanup inside `with_locks` (it's already there in `update_keys`; ensure EVERY entry point that could conflict does it under the lock). Remove the bare `new()` tmp deletion; keep `new()` to just dir + perms + open-lock-file. Add a test or rationale that the lock is acquired before any fs op. Commit.

### Task A4: Reset = archive, not delete (P1 #4)
**Files:** `src-tauri/src/keystore.rs`, `src/App.tsx`
- [ ] §A fail-closed: an explicit Reset must MOVE the canonical file to `.broken-<ts>` (recoverable), never `remove_file`. Change `reset()` to call the same archive-then-clear logic (archive the existing file if present, then the store starts empty). Keep `archive()` as the named recovery op; `reset()` = archive-if-present + remove tmp. Update App.tsx copy to reflect "archived, recoverable" not "deleted". Commit.

### Task A5: Vendor `get-selected-text` (P1 #5 — spec-decision deviation)
**Files:** `src-tauri/src/a11y.rs` (replace or gate), vendored source under `src-tauri/vendor/`, LICENSE.
- [ ] The approved spec §B decision is "vendor `get-selected-text`, reject self-impl." The raw FFI in `a11y.rs` is an unapproved architecture change. Two acceptable resolutions (pick one, document in the file + memory):
  - **(a) Vendor** `yetone/get-selected-text` (or its macOS AX logic) into `src-tauri/vendor/get-selected-text/` with its LICENSE, and call THAT instead of raw FFI. This honors the spec decision literally.
  - **(b) Formally amend the spec**: update `docs/superpowers/specs/...v1-design.md` §B to "self-implemented AX via accessibility-sys FFI (rationale: ...)" and note this as a spec change requiring re-review.
  - Recommend (a) to stay faithful to the approved decision; only choose (b) if vendoring proves impractical (e.g. the upstream is too entangled). Report which + why.
- [ ] Commit (vendored source + LICENSE + the call-site change, OR the spec amendment).

### Task A6: Real cross-process lock test (P1 #6)
**Files:** `src-tauri/tests/keystore.rs`
- [ ] Add a **child-process** test: spawn a second test binary (a small `#[test]`-driven helper or a `cargo run` of a tiny bin) that holds the keystore.lock on a shared temp dir; assert the parent's `update_keys` blocks-until-released (use a timeout + a flag file the child writes while holding the lock). This genuinely proves cross-process mutual exclusion. Also strengthen the different-dirs test to actually hold dir1's lock across a dir2 write and assert dir2 completed (timing/flag-based).
- [ ] Commit.

### Task A7: Clippy clean
**Files:** various
- [ ] `cargo clippy --all-targets -- -D warnings` (ignoring the known objc-cfg warnings) must pass. Fix `new_without_default` (impl Default for Keystore or `#[allow]` with rationale), test-module placement, redundant closures. Commit.

---

## Part B — Phase 4 plan rev-3

Rewrite `docs/superpowers/plans/2026-07-30-phase4-windows-parity-packaging.md` to rev-3 fixing all review issues:
- **Windows atomic_replace**: it's a STUB (keystore.rs:421 returns Err) — rev-2 wrongly claimed it was done. Add the real `MoveFileExW` (first-create) + `ReplaceFileW` (update) task + Windows tests.
- **Windows Authenticode signing**: `TAURI_SIGNING_PRIVATE_KEY*` is the Tauri UPDATER key, not Authenticode. Installer signing needs PFX import + `certificateThumbprint` or a custom `signCommand` (e.g. signtool). Document both (updater key vs Authenticode cert).
- **macOS CI YAML**: fix the two-sibling-`env:` bug; use the official cert-import structure (not `mktemp /tmp/cert.p12`); reference Tauri's documented flow.
- **macOS runner**: `macos-13` retired 2025-12-04 → use `macos-15-intel` (or current Intel label) for x86_64-apple-darwin.
- **Capabilities**: custom `invoke_handler` commands are callable from ALL local windows by default; per-window restriction requires `AppManifest::commands` / explicit command permissions, not just splitting `permissions` arrays. Correct the plan's capability-split task to use the real mechanism (or drop the claim and document that v1 leaves commands globally callable with a CSP + minimal plugin perms).
- Commit the rev-3 plan only.

---

## Task C — Re-review, then E2E, then Phase 4 execution

- [ ] After Part A + Part B: **stop and re-review with the user.** Do NOT execute Phase 4 or do E2E until the user re-approves.
- [ ] On approval: manual E2E (Phases 2/3 + all P1 fixes) → then execute Phase 4 rev-3.

---

## Self-Review

- **Coverage:** P1 #1→A1, #2→A2, #3→A3, #4→A4, #5→A5, #6→A6, + clippy (A7). Phase 4 plan rev-3 = Part B.
- **Honest unknowns:** A2's exact runtime-register API (resolve at execution); A5 vendoring practicality (offer fallback (b)); A6 child-process test mechanics (spawn a helper bin). None are silent TODOs.
- **No false "done" claims:** this plan explicitly retracts the round-1 "11 P1 closed" claim.

## Execution Handoff
Subagent-Driven. **Pause for user re-review after Part A + B.**
