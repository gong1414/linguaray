# Rayline R3b–R7 Completion Plan

**Status:** in progress  
**Branch:** `codex/rayline-r3b-r7`  
**Base:** `main@fed93d2`  
**Worktree:** `/Users/daoyu/Code/projects/islandpot/.worktrees/rayline-r3b-r7`

## Objective

Complete the frozen product roadmap after the merged R2/R3a audit work:

- R3b / S2b–S3: Surface 07 Shortcuts; Surface 08 Privacy & Data history controls;
  encrypted history foundation.
- R4 / S4: Surface 09 History, Surface 10 Vocabulary, Surface 11 Dictionary.
- R5 / S5–S6: Surface 12 OCR Overlay, Surface 13 Text-to-Speech.
- R6 / S6: Surface 14 Onboarding, the single shared External API service used by
  Surface 08 and Surface 15, and Surface 16 Updater.
- R7 / S7: integrated accessibility, privacy, security, packaging and real-platform
  acceptance.

The implementation follows `design-system/linguaray/MASTER.md`, pages 07–16 and
`docs/superpowers/specs/2026-08-01-linguaray-product-baseline.md`. Product baseline
dependency and security rules win over the older high-level parallel R3/R4 diagram.

## Reconciled ownership decisions

1. **S2b is part of R3b Task A.** History controls cannot ship before encrypted
   persistence, consent gating and retention exist.
2. **History disable preserves existing encrypted records.** It stops future writes.
   Clear All remains available while disabled so the user can delete retained data.
3. **Keystore reset preserves provider metadata but removes irrecoverable encrypted
   history/vocabulary in the same exclusive recovery operation and disables history.**
   This matches the existing provider-recovery UX and avoids keeping ciphertext that
   can no longer be decrypted.
4. **External API has one backend and one shared panel.** Surface 08 owns the privacy
   summary/control placement; Surface 15 owns detailed lifecycle/endpoint status.
   The shared service is implemented in R6. Surface 08 is not declared fully complete
   until that panel is connected in R6.
5. **No unapproved R3 scope expansion.** Local-only mode, clipboard policy,
   diagnostics and history export are not added to Surface 08; export belongs to R4.
6. **Provider balance remains a separate Provider Center backlog until a typed,
   provider-neutral backend contract is frozen.** It cannot block encrypted history,
   but it must be resolved before final R7 product completion.
7. **Shortcut action IDs and defaults are frozen for implementation:**
   `translate_selection=Alt+Space`, `translate_input=Ctrl+Space`,
   `translate_clipboard=Ctrl+Alt+Space`, `ocr_translate=Alt+Shift+Space` on both
   supported platforms. Canonical modifier order is `Ctrl+Alt+Shift+Super+Key`.
   Selection/Input/Clipboard register in R3b; OCR is persisted and shown as unavailable
   until R5 wires its real handler. Assigning another action's combo with Override swaps
   the two actions' mappings, so no action becomes silently unassigned.
8. **Shortcut IPC is additive and typed.** `shortcut_list` returns a revisioned
   snapshot with action, combo, availability, registration state and optional error.
   Save/reset require `expected_revision`. Recording begin/end sets a controller flag
   that makes existing global callbacks no-op while the settings page captures keys.
   OS-reserved conflicts are authoritative only when real registration fails.
9. **ListRow and warning InlineError graduate from backlog in R3b.** They become shared
   package components; Surface 07 does not create page-local imitations.

## Baseline at start

- `pnpm typecheck`: pass.
- `pnpm test:all`: 568 pass (root 285, UI 218, ui-lab 65).
- `pnpm build`: pass.
- Playwright: 152 pass.
- `cargo test --features xproc-test-helper`: 457 pass.
- strict feature Clippy: pass.
- `git diff --check`: pass.

Known jsdom `scrollTo`/canvas messages are non-failing environment noise.

## R3b — encrypted privacy and shortcuts

### Task A1 — History crypto and typed keystore accessors

- Add `src-tauri/src/history/crypto.rs` and module exports.
- Add atomic `history_key` get-or-create/read/clear operations to the typed keystore.
- AES-256-GCM, fresh 12-byte nonce, pre-generated UUIDs and exact domain-separated
  AAD for session source, result text, error text and vocabulary fields.
- RED tests: roundtrip, nonce uniqueness, UUID/AAD swap, tamper, missing/corrupt key,
  no plaintext in persisted bytes.

### Task A2 — History preferences, retention and clear repository

- Add typed repository/service for `history_enabled`, retention 30/90, clear-all,
  startup cleanup and favorite-preserving retention.
- Enable ordering: get/create key first, then DB transaction enables history.
- Disable preserves key/records and stops future writes.
- Clear All deletes sessions transactionally (results cascade), keeps key and setting.
- RED tests: defaults, invalid retention, enable fail-closed, idempotent key, cutoff
  boundary, favorite preservation, rollback and clear/write serialization.

### Task A3 — Persist translation sessions

- Extend the shared translation orchestration with provider snapshot, elapsed time,
  trigger source and outcome metadata required by the existing schema.
- Persist only after a successful explicit history consent recheck under the DB gate.
- Never fail the user-visible translation when optional history persistence fails;
  log only classification/UUID, never content.
- Cover selection, clipboard, input and External API entry points through one shared hook.
- RED tests: off=zero writes, single/multi/failure/fallback identity, delete-provider
  snapshot, disable-vs-write, plaintext/log scan.

### Task A4 — History privacy IPC and recovery integration

- Commands: `history_privacy_status`, `history_set_enabled`,
  `history_set_retention`, `history_clear_all`.
- Add Tauri permissions, main capability and manifest registration.
- Keystore archive/reset takes the data write gate, disables history and deletes
  history/vocabulary before a key can be lost; failure leaves readiness degraded.
- RED tests: wire shape, capability isolation, recovery barrier and cleanup rollback.

### Task A5 — S2b encrypted search gate

- Fixed 200-row cursor batches ordered by `(timestamp, session_uuid)`.
- Bounded in-memory AES-GCM decryption with full NFKC + Unicode case folding.
- Search source/result text without a plaintext index; corrupt records remain visible
  with `corrupt=true` and do not abort the batch.
- Main-only `history_search` typed IPC; legacy `translate`/`translate_default`
  capabilities are removed so production windows cannot bypass consent-aware session
  persistence. R4 reuses this service for Surface 09 instead of reimplementing crypto.

### Task B1 — Runtime shortcut service

- Define fixed actions: selection, input, OCR, clipboard. OCR may be stored before R5
  but is registered only when its command is available.
- Seed platform-neutral defaults in the existing `shortcuts` table.
- Commands: `shortcut_list`, `shortcut_check_conflict`, `shortcut_save`,
  `shortcut_reset_defaults`.
- Parse canonical key combinations; detect internal duplicates and OS registration
  conflicts.
- Registration update is atomic from the user's perspective: validate all, unregister
  changed old bindings, register all new bindings, persist only after registration;
  on any failure restore old registrations and DB values.
- Startup loads DB values instead of hardcoded strings, with per-action safe fallback.
- RED tests: canonicalization, internal conflict, system-reserved failure, rollback,
  persistence, startup restore and rapid consecutive updates.

### Task B2 — Surface 07 production UI

- Add `ShortcutsPage.tsx/.css` plus typed IPC wrapper and localized copy.
- Implement Default, Recording, Conflict and Registration-failed states using
  ListRow/ShortcutChip/Button/InlineError.
- Keyboard recording ignores bare modifiers, supports Escape cancellation and makes
  conflict Override explicit.
- Reset Defaults uses a confirmation when user mappings differ.
- Enable the SettingsShell Shortcuts route and remove its placeholder copy.
- RED tests: all four states, focus restoration, keyboard-only flow, busy/concurrency,
  zh/en and axe.

### Task C1 — Surface 08 history privacy UI

- Add shared `PrivacyDataPage` and `HistoryPrivacyPanel`.
- Implement loading/error/retry, off/on, 30/90 retention, Clear All destructive
  confirmation in both on/off states, pending locks and success/error toasts.
- The External API slot renders the shared panel only when R6 provides it; until then
  it is explicitly tracked as an incomplete Surface 08 subfeature, not a completed mock.
- Enable SettingsShell Privacy navigation.
- RED tests: consent failure rollback, retention rollback, clear confirmation,
  off-state deletion, stale async completion guards, zh/en and axe.

### Task C2 — R3b visual and integration gate

- Add production View fixtures to ui-lab.
- Add light/dark at 600/699/700/800, zh/en long-copy, reduced-motion and keyboard tests.
- Run full frontend/Rust/build/Clippy/Playwright/diff/status matrix.
- macOS real hotkey smoke is recorded locally; Windows automated and real-machine
  evidence is required before S3 is signed off.

## R4 — knowledge loop

### Task D1 — History service and Surface 09

- Cursor pagination by `(timestamp, session_uuid)`, fixed 200-row batches; bounded
  decrypt-in-memory NFKC casefold search; corrupt-row tolerance.
- List/search/detail/favorite/delete/export commands and all seven page states.
- CSV/JSON export never writes unintended plaintext outside the user-selected target.

### Task D2 — Vocabulary and Surface 10

- Encrypted CRUD using the same history key/AAD discipline.
- Wire Popup/Input favorite action to a non-blocking add operation.
- CSV/JSON/AnkiConnect export; Anki plaintext stays in memory and only targets
  `127.0.0.1:8765` after a user action.

### Task D3 — Dictionary and Surface 11

- Promote the existing macOS system lookup to a registered typed command.
- Add cross-platform offline package inventory/install/lookup with source attribution.
- Validate package paths/content before atomic installation; no remote lookup.

### Task D4 — R4 gate

- Production/unit/axe/visual coverage for Surfaces 09–11 and both platforms.

## R5 — OCR and media

### Task E1 — OCR core and overlay

- Multi-monitor overlay state machine, coordinate/scaling normalization, cancel and
  permission/degraded paths.
- macOS ScreenCaptureKit/Vision and Windows DXGI/Windows.Media.Ocr adapters behind a
  common tested interface; file/image OCR uses the same limits and result type.

### Task E2 — Text-to-Speech

- Typed voice list/speak/stop service with macOS/Windows offline adapters.
- Surface 13 settings plus Popup/Input result-card integration and stale completion
  guards.

### Task E3 — R5 gate

- Permission, multi-monitor, scaling, offline voice and two-platform real-machine gate.

## R6 — system boundaries

### Task F1 — Onboarding

- Six frozen states, resumable/idempotent progress and reuse of real Provider,
  History and Shortcut services. Platform-specific accessibility step.

### Task F2 — Shared External API service (Surfaces 08 and 15)

- Schema v3 preferences for enabled/port with fail-closed idempotent migration.
- One runtime controller; bind-before-persist enable, one-time base64url 32-byte token,
  constant-time auth, atomic regenerate, crash-safe disable/startup recovery.
- Loopback Host allowlist, reject all Origin, JSON content type/body/image limits,
  sliding 60/minute limiter, unified content-free errors and frozen endpoints.
- Shared ExternalApiPanel in Surface 08; full lifecycle/detail Surface 15.

### Task F3 — Updater

- Tauri updater runtime/state UI, independent signature verification, download progress,
  restart/later/failure recovery and tray UpdateAvailable integration.

### Task F4 — R6 gate

- External API adversarial security tests, updater signature/inruption tests,
  onboarding resume tests and both-platform evidence.

## R7 — acceptance and release readiness

- Resolve Provider balance with a frozen typed contract or explicitly remove the
  unsupported visual promise from the final product contract.
- At least 16 surfaces × light/dark × zh/en baselines, plus required states/sizes.
- Keyboard, screen reader, WCAG contrast, 200% zoom, reduced motion, long copy,
  empty/offline/degraded states.
- Capability least privilege, log/plaintext scan, migration/cross-version, dependency
  and external-API security audit.
- macOS and Windows real-machine matrix; signed packages and signed update install.
- Font OFL files verified in release packages.
- No public tag or release until every R7 item has evidence and sign-off.

## Commit and hygiene rules

- Never use `git add -A`; stage explicit files only.
- Never stage `.mimosa`, `.pnpm-store`, `dist`, `test-results`, Playwright reports or
  local credentials.
- Each task follows RED → GREEN → refactor and receives independent spec plus quality
  review before its stage is closed.
- Frozen design documents are not silently changed. Reconciliations live in this plan
  until explicitly promoted to a new frozen decision record.
