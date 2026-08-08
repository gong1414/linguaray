# LinguaRay Rayline Redesign — Design & Development Plan

**Status:** R0 frozen · R1 complete (rev-4.3.2) · **Date:** 2026-08-08

**Design handoff:** [RAYLINE-REDESIGN.md](../../../design-system/linguaray/RAYLINE-REDESIGN.md)

**Product baseline:** [2026-08-01-linguaray-product-baseline.md](../specs/2026-08-01-linguaray-product-baseline.md)

## Outcome

Replace the current two-screen prototype with the complete 16-surface Rayline experience while preserving the S0 privacy, security, dual-platform, and no-public-release gates. Work is ordered by shared contracts and user journeys rather than by drawing order.

The existing implementation already provides a useful base:

- `packages/ui`: 12 tested primitives, including Button, TextField, Select, Switch, Dialog, Toast, Banner, ProviderCard, and ResultCard.
- `apps/ui-lab`: implemented Selection popup and Provider center prototypes with interaction tests.
- root Solid/Tauri app: selection popup, input translation, provider CRUD, consent, settings, selection capture, clipboard, keystore and database recovery.
- backend gaps for the redesign: parallel translation orchestration, history/vocabulary storage, production dictionary wiring, OCR, TTS, shortcut persistence UI, external API, updater UI, and onboarding state.

## Delivery order

```text
R0 Contract freeze
   ↓
R1 Tokens + shared shell + components
   ↓
R2 Core translation loop
   ├───────────────┐
   ↓               ↓
R3 Control & trust R4 Knowledge
   └───────┬───────┘
           ↓
R5 OCR & media
           ↓
R6 System boundaries
           ↓
R7 Cross-platform release gate
```

## R0 — Reconcile and freeze the contract

**Purpose:** remove ambiguity between the 2026-08-01 production master and the new Penpot source before UI code changes.

- Approve Rayline visual direction and the 16-surface inventory.
- Reconcile `design-system/linguaray/MASTER.md` with the Core + Semantic dual-collection model: **Core 97** primitives (colors 44 + spacing 14 + radius 9 + border-width 2 + opacity 2 + font-family 3 + font-size 9 + font-weight 4 + typography 10) + **Semantic 28 (Light) + 28 (Dark)**. Penpot is the only design source; `neutral` (not slate); `color.core.white`/`black` are standalone (not neutral.0); indigo 400/500/600/700 pinned.
- Generate a checked token mapping from Penpot names to CSS custom properties; preserve aliases during migration so existing screens do not break mid-slice. Shadows surface as **`shadow.raised` / `shadow.overlay`** (not sm/md/lg). Engineering extensions — clearly flagged, not written back to Penpot — include **`color.border.control`** (`#64748B` both themes, UI 3:1) for control edges and the **strong-fill** set (`color.strong-fill-{success,warning,danger,info}` + matching `strong-on-*`, unified across both themes) for Banner/Button strong fills.
- Record target sizes, component variants, state matrices, and localized copy keys per surface under `design-system/linguaray/pages/`.
- Add a handoff manifest containing Penpot file ID, page IDs, surface IDs, and last approved revision.

**Gate:** no unresolved token name, surface ownership, window size, or state behavior; design and engineering sign off the same manifest.

## R1 — Foundations, shell, and reusable components

**Status:** ✅ Complete (rev-4.3.2).

**Primary paths:** `packages/ui/src/styles/`, `packages/ui/src/components/`, `apps/ui-lab/` (R1 does **not** touch production `src/`; that is deferred to R2).

1. Replace flat production variables with primitive and semantic layers; add `data-theme="light|dark|system"` resolution.
2. Add typography, spacing, radius, shadow, focus, and motion tokens from Rayline.
3. Implement the 9 missing shared patterns: SegmentedControl, ShortcutChip, StatusBadge, InlineError, WindowChrome, SidebarItem, HistoryRow, ProviderRow, TranslationCard. **Out of R1:** AppSidebar and ProgressRail are not part of R1.
4. Align existing Button/TextField/Select/Switch/Dialog/Toast/ResultCard APIs to Penpot variants without breaking current consumers in one commit.
5. Build a UI Lab state gallery (ComponentGallery) for every component and both themes.

**Component inventory (rev-4.3.2, 24 exports from `@linguaray/ui`):** 9 pre-existing (Button, IconButton, TextField, Select, Switch, Toast, Confirm, EmptyState, ResultCard) + 9 new (SegmentedControl, ShortcutChip, StatusBadge, InlineError, WindowChrome, SidebarItem, HistoryRow, ProviderRow, TranslationCard) + 6 auxiliary (Banner, Dialog, Tooltip, Spinner, ProviderCard, VisuallyHidden). TextArea/Checkbox/Card/ListRow remain **backlog** (not in R1).

**Tests:** component unit tests (ui 204, ui-lab 201), keyboard/focus tests, axe checks, reduced-motion snapshots, light/dark visual snapshots, Chinese overflow fixtures. **Visual baseline: 42 screenshots (20 components × 2 themes + 2 reduced-motion) and 43 Playwright cases (40 visual + 2 reduced-motion + 1 keyboard).**

**Gate:** all 18 design component contracts (9 pre-existing + 9 new) available from `@linguaray/ui`; no surface-specific hardcoded color or spacing values.

## R2 — Core translation loop (surfaces 01–04)

**Primary paths:** `src/Popup.tsx`, `src/InputPanel.tsx`, new `src/features/translation/`, `src-tauri/src/popup.rs`, `src-tauri/src/service.rs`

- Rebuild Selection popup and Input window with the shared shell and state model.
- Add expandable multi-result mode in the popup; keep provider ordering stable while results resolve.
- Add pin, copy, save, TTS entry, retry, and provider provenance actions.
- Replace ad-hoc error strings with typed frontend states: no selection, permission, missing key, auth, network, offline, partial, and recovered fallback.
- Rebuild the tray/menu-bar interaction with quick actions, active-provider status, recent history entry, update badge, and local-only indicator.
- Introduce an operation registry so stale requests cannot overwrite newer popup content.

**Backend dependencies:** parallel provider fan-out with consent scope, cancellation/latest-wins, typed failure classification, actual-producing-provider metadata.

**Gate:** selection, input, clipboard, fallback, and parallel-result journeys pass on macOS and Windows with clipboard restoration and keyboard-only operation.

## R3 — Provider, credentials, shortcuts, privacy (surfaces 05–08)

**Primary paths:** `apps/ui-lab/src/pages/ProviderCenter.tsx`, new `src/features/settings/`, `src-tauri/src/db/providers.rs`, `src-tauri/src/settings.rs`, `src-tauri/src/keystore.rs`

- Port Provider center to production and split data/controller logic from presentation.
- Implement provider health summary, credential state, model discovery, latency/quota status, duplicate/reorder/toggle, and stable rollback feedback.
- Rebuild keystore recovery around readiness states and the existing archive/reset commands; preserve explicit consequences before destructive actions.
- Add persisted global-shortcut editing with recording, conflict detection, per-shortcut registration errors, and defaults reset.
- Add Privacy & data controls for local-only mode, history retention, clipboard policy, diagnostics, export, and local deletion.

**Gate:** provider CRUD/reorder/consent/recovery tests remain green; each settings mutation has pending, success, failure, retry, and concurrency behavior.

## R4 — Knowledge loop (surfaces 09–11)

**Primary paths:** new `src/features/knowledge/`, new `src-tauri/src/history.rs`, `vocabulary.rs`, and expanded `dict.rs`

- Add encrypted, opt-in history schema with retention cleanup, search, pagination, export, and delete.
- Add vocabulary save/remove/tag/review metadata and CSV/JSON/AnkiConnect export adapters.
- Wire `lookup_dictionary` into the command manifest and UI; add offline package inventory/install progress and source attribution.
- Connect Save actions from popup, input, and multi-result to history/vocabulary without blocking translation.

**Gate:** no source text is stored before consent; retention and delete are testable with a fake clock; export files round-trip; dictionary works without network.

## R5 — OCR and media (surfaces 12–13)

**Primary paths:** new `src/features/ocr/`, `src/features/tts/`, `src-tauri/src/ocr/`, `src-tauri/src/tts/`

- Implement display-aware OCR overlay lifecycle: initial, selecting, capturing, processing, success, no-text, permission error, and cancel.
- Add image/clipboard OCR entry points reusing the same recognition pipeline.
- Implement system offline TTS voices, playback progress, stop/seek, speed, and queue controls.
- Route OCR output through the same translation operation model as selection and expose local/cloud provenance.

**Gate:** multi-display and scaling tests, protected-content failure on Windows, Screen Recording recovery on macOS, offline TTS, and cancellation without orphan windows.

## R6 — Onboarding, external API, updater (surfaces 14–16)

**Primary paths:** new `src/features/onboarding/`, `external-api/`, `updater/`; Tauri plugins/config and new Rust services

- Persist an idempotent onboarding state machine: welcome, permission, provider, history consent, shortcuts, complete; allow safe skip/resume.
- Add the default-off `127.0.0.1:61742` API with bearer authentication, rate limiting, one-time token display, token rotation, and request log metadata only.
- Add signed updater states: checking, available, downloading, verifying, ready, restart, failure/retry, and current version.
- Surface tray badges and settings deep links for permission, API, and update states.

**Gate:** external API security tests, localhost-only binding test, token redaction, signed update verification, interrupted-download recovery, onboarding rerun/idempotency.

## R7 — Integrated acceptance and release readiness

- Run repository tests: `pnpm typecheck`, `pnpm test`, `pnpm build`, Rust unit/integration tests, and Tauri bundle checks.
- Add screenshot baselines for all 16 surfaces in Light/Dark and Chinese/English at target logical sizes.
- Run keyboard traversal, screen reader labeling, contrast, 200% scaling, reduced motion, long copy, empty data, offline, and degraded-provider suites.
- Perform real-machine journeys on current macOS and supported Windows versions; do not substitute one platform's CI for physical verification.
- Verify app permissions/capabilities expose only commands needed by each window.
- Keep the S0 rule: no public tag or GitHub Release until every R7 acceptance item is signed off.

## Slice completion checklist

Each slice ships as one vertical, reviewable unit:

- Penpot surface and states approved.
- Shared component work lands before screen composition.
- Frontend uses typed state and localized copy.
- Backend command/capability/security behavior is tested.
- Light/Dark, Chinese/English, keyboard, reduced motion, and error recovery verified.
- UI Lab mock and production screen agree; obsolete prototype code is removed only after migration.
- `MASTER.md`, page contract, command permissions, and test matrix are updated in the same slice.

## Explicit sequencing decisions

- Do not implement 16 isolated pages in parallel before R1; that would duplicate shells, state badges, and error handling.
- Do not start OCR/TTS/API/updater UI before their backend state contracts are typed.
- Do not rewrite the existing secure provider/keystore/data-readiness core merely to match the new visuals; wrap it with the new presentation state model.
- Do not delete `apps/ui-lab` after porting the first screens. It becomes the permanent visual/state acceptance harness for all 16 surfaces.
