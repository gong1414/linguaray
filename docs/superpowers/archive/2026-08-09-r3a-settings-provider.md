Archived-on: 2026-08-14 · reason: superseded by linguaray-plugin-core-design / completed, see git history

# R3a Settings Shell + Provider Center (Surface 05) + Keystore Recovery (Surface 06) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking. Each task is strict TDD: write the failing test (RED) → implement (GREEN) → commit.

**Goal:** Replace the legacy monolithic `src/App.tsx` settings/translation window with a proper Settings shell (`WindowChrome` + `SidebarItem`) hosting two real surfaces driven by the existing S2a provider IPC: **Surface 05 Provider Center** (ported from `apps/ui-lab` and rewired to real IPC) and **Surface 06 Keystore Recovery**. Shortcuts and Privacy nav items are placeholders filled in R3b.

**Architecture:** A new `src/features/settings/` feature folder owns the provider front-end state model (`provider-types.ts`), the IPC layer (`provider-ipc.ts`), the ported pure domain logic (consent scope, endpoint validation — vendored copy of `apps/ui-lab/src/pages/provider-domain.ts`), the Keystore Recovery component, the Provider Center component, and the Settings shell. `App.tsx` becomes a thin mount that wires `WindowChrome` + the shell. The backend (`src-tauri/`) is NOT modified — R3a consumes the existing S2a/P1 IPC surface only.

**Tech Stack:** SolidJS 1.9, `@linguaray/ui` (workspace package — already resolved by the R2b vitest/vite config), Tauri 2 IPC (`@tauri-apps/api/core` `invoke`), Vitest 2 + `@solidjs/testing-library` + jsdom (installed by R2b Task 0).

---

## Global Constraints

These apply to every task. Each task's requirements implicitly include this section.

- **Semantic tokens only.** Production `src/` MUST consume colors/spacing/radius/shadow via CSS variables (`--color-*`, `--radius-*`, `--shadow-*`, etc.) as defined by `@linguaray/ui` token CSS. No hardcoded hex may be introduced in any file under `src/`. The existing `test/no-hardcoded-hex.test.ts` scan (already in the repo) covers `src/**/*.css|tsx|ts`; new `src/features/settings/*.css` files are in-scope for that scan and MUST pass. Hex inside a `var(--token, #fallback)` fallback slot is the only permitted form.
- **Backend untouched.** No task may modify any file under `src-tauri/`. R3a consumes the existing IPC surface only. Do NOT alter IPC command signatures, event names, or the `ProviderProfile` / `ProviderPatch` / `SetActiveResult` / `ProviderCommandError` wire shapes — they are the source of truth and are mirrored as TS types. If a needed capability is missing from the backend (see "Known R3a limitations" below), mark it `TODO` and degrade gracefully; do NOT edit `src-tauri/`.
- **IPC contract (decode, do not redefine).** Wire shapes mirrored as TS types in `provider-types.ts`:
  - `provider_list` → `ProviderProfile[]` (see struct below).
  - `provider_create(template_id, name, endpoint, model?)` → `ProviderProfile`.
  - `provider_update(uuid, patch)` → `ProviderProfile`; `patch: ProviderPatch = { name?, endpoint?, model?, enabled?, sort_order? }` (`#[serde(deny_unknown_fields)]` — do not send unknown keys).
  - `provider_duplicate(uuid)` → `ProviderProfile` (new UUID, new `secret_ref`, keyless).
  - `provider_delete(uuid)` → `()` (3-step: begin_delete → purge key → finalize tombstone; all on one blocking thread under the write gate).
  - `provider_reorder(uuids)` → `()` (must be exactly the active UUIDs).
  - `provider_toggle(uuid, enabled)` → `()`.
  - `provider_set_key(uuid, key)` → `()`; rejects if `status != "active"`.
  - `provider_set_active(primary, parallel, fallback)` → `SetActiveResult = { outcome: "written" } | { outcome: "needs_consent", actual_scope: string }`.
  - `provider_confirm_and_set_active(primary, parallel, fallback, expected_scope)` → `i64` (consent version); rejects with `ProviderCommandError = { error: "stale_scope", actual_scope } | { error: "db", message } | { error: "validation", message }`.
  - `provider_get_models(uuid)` → `ModelInfo[]` where `ModelInfo = { id, label }` (preset-derived; full HTTP fetch is S3 scope).
  - `provider_test_connection(uuid)` → `ConnectionResult = { ok, message }` (best-effort reachability; full probe is S3 scope).
  - `keystore_health` → `string` (`""` = healthy/first-run; non-empty = fail-closed reason).
  - `archive_keystore` → `string` (archived path).
  - `reset_keystore` → `Option<string>` (archived path, or `null`).
  - `key_status` → `Record<string, boolean>` keyed by `secret_ref`.
  - `ProviderProfile` (TS mirror): `{ uuid, template_id, name, protocol: "openai_chat"|"anthropic"|"gemini"|"google_translate"|"custom_http", endpoint, model: string|null, enabled, sort_order: number, is_local, needs_key, secret_ref, capabilities: { balance, quota, model_list }, status: "active"|"deleting"|"deleted" }`.
- **Known R3a limitations (degrade gracefully + TODO, do NOT touch backend):**
  1. **No active-selection read IPC.** `read_active_selection` (primary/parallel/fallback) is backend-internal only — there is no Tauri command exposing it to the frontend. R3a therefore mirrors the selection in client memory: on mount the role badges render as "none", and once the user assigns roles via the Provider Center actions (which call `provider_set_active` / `provider_confirm_and_set_active`), the client tracks them for the session. Cold-load role rendering (showing stored roles from DB) is deferred until a backend read command exists. Each Provider Center file must carry a `// TODO(r3b): no read-active-selection IPC — roles are session-only` note at the role-rendering site. Do NOT add the read command.
  2. **Balance / quota introspection not implemented.** The balance section renders a `TODO(r3b): balance/quota IPC not yet implemented` `InlineError`/muted note instead of the ui-lab mock states. The connection test renders the real `provider_test_connection` result (reachability only).
- **Keyboard + focus.** Every interactive element must be keyboard-operable: Tab order = visual order; the destructive Keystore Reset `Confirm` dialog lands initial focus on **Cancel** (the `@linguaray/ui` `Confirm` component already does this for `variant="destructive"` — do not override its `onOpenAutoFocus`); the SidebarItem nav is a native `<button>` (Enter/Space fires natively); role-assignment / delete / consent `Confirm` dialogs restore focus to their trigger (or a documented fallback) on close.
- **Reduced motion.** Respect `prefers-reduced-motion`. Use the frozen `Spinner`/`Button` (`loading`) from `@linguaray/ui` (which render a static + text fallback under reduced motion) rather than re-implementing loading indicators. Do not add CSS keyframe animations in `src/`.
- **No emoji in code.** Code comments and copy keys use plain text.
- **i18n (zh / en).** Every user-visible string lives in `src/features/settings/copy.ts` as `SettingsCopy` with `zh` and `en` maps (mirrors the R2b `src/features/translation/copy.ts` pattern). Locale comes from the existing `src/i18n.ts` `detectLocale()`. No raw user-facing English/Chinese literals in components — only copy-key lookups. The `{name}` / `{reason}` / `{latency}` / `{message}` placeholders are substituted at the call site.
- **CSP compliance.** The active CSP is `default-src 'self'; img-src 'self' data:; style-src 'self' 'unsafe-inline'; script-src 'self'; connect-src 'self' ipc: http://ipc.localhost`. Inline `style="..."` is allowed but prefer CSS classes; lucide-solid icons are SVG (allowed). No remote fonts/images.
- **Workspace import.** `@linguaray/ui` resolves via the workspace alias already configured in `vitest.config.ts` and (by R2b) `tsconfig.json`/`vite.config.ts`. Do not re-add the dependency. Re-use the 24 R1 components; this plan uses: `WindowChrome`, `SidebarItem`, `ProviderRow`, `Switch`, `StatusBadge`, `TextField`, `Select`, `Confirm`, `Banner`, `Toast`, `Button`, `EmptyState`, `Spinner`, `Tooltip`, `InlineError`.
- **TDD.** Every code task writes the failing test first (RED), runs it to confirm it fails, then implements (GREEN), then commits. Domain logic (`provider-domain.ts`, `provider-types.ts` decoding) gets unit tests under `src/features/settings/*.test.ts`; components (`KeystoreRecovery`, `ProviderCenter`, `SettingsShell`) get component tests under `test/*.test.tsx` with `invoke` mocked.

---

## File Structure

**Created (production `src/`):**
- `src/features/settings/provider-types.ts` — TS mirrors of `ProviderProfile`, `ProviderPatch`, `ModelInfo`, `ConnectionResult`, `SetActiveResult`, `ProviderCommandError`; the frontend `ProviderProfileFE` (frontend-augmented profile with `hasKey`), `ActiveSelection`, `ConsentScope`/`ConsentRecipient` types; typed `invoke<T>` wrappers live in `provider-ipc.ts` (Task 2).
- `src/features/settings/provider-domain.ts` — vendored port of `apps/ui-lab/src/pages/provider-domain.ts`: `validateActiveSelection`, `buildConsentScope`, `consentScopeKey`, `resolveConsentKey`, `validateEndpoint`, `normalizeOrigin`, `TRADITIONAL_TEMPLATES`, `ProviderTemplate`. Operates on `ProviderProfileFE` (adapted from the `MockProvider` shape — `uuid`/`template`→`template_id`/`endpoint`/`model`/`enabled`/`isLocal`/`hasKey`/`status`/`sortOrder`). (Task 2)
- `src/features/settings/provider-ipc.ts` — typed `invoke<T>` wrappers for all 12 provider commands + `keystore_health`/`archive_keystore`/`reset_keystore`/`key_status`; a `loadProviders()` helper that calls `provider_list` + `key_status` and joins them into `ProviderProfileFE[]`. (Task 2)
- `src/features/settings/copy.ts` — `SettingsCopy` type + `SETTINGS_COPY: { zh; en }` with all Surface 05 + 06 keys (Source: `design-system/linguaray/pages/05-provider-center.md` and `06-keystore-recovery.md` copy tables). (Task 2)
- `src/features/settings/KeystoreRecovery.tsx` — Surface 06 (Task 1).
- `src/features/settings/KeystoreRecovery.css` — layout only, token-based.
- `src/features/settings/SettingsShell.tsx` — `WindowChrome` + `SidebarItem` nav + content slot; adaptive ≥700px / 600–699px (Task 0).
- `src/features/settings/SettingsShell.css` — layout only, token-based.
- `src/features/settings/ProviderCenter.tsx` — Surface 05, real IPC (Task 3).
- `src/features/settings/ProviderCenter.css` — ported + trimmed from `apps/ui-lab/src/pages/ProviderCenter.css` (drop the lab's settings-rail rules now owned by `SettingsShell`).
- `src/features/settings/provider-domain.test.ts` — unit tests for the ported domain logic (Task 2).
- `src/features/settings/provider-ipc.test.ts` — unit tests for the `invoke` wrappers (mocked `@tauri-apps/api/core`) (Task 2).
- `src/features/settings/copy.test.ts` — parity test: every `SettingsCopy` key present in both `zh` and `en`; `{name}`/`{reason}`/`{latency}`/`{message}` placeholders match across locales (Task 2).

**Created (component tests, `test/`):**
- `test/SettingsShell.test.tsx` — adaptive rendering, nav switching, placeholder items (Task 0).
- `test/KeystoreRecovery.test.tsx` — the 4 states + Confirm focus-on-Cancel + IPC calls (Task 1).
- `test/ProviderCenter.test.tsx` — empty / list / editing / key-saving / deleting / consent / reorder / connection-test flows against mocked IPC (Task 3).
- `test/App.test.tsx` — smoke: App renders `SettingsShell` with Provider Center active by default; no legacy `.settings-group` / `<select>` / `<textarea>` elements remain (Task 4).

**Modified (production `src/`):**
- `src/App.tsx` — **fully replaced.** New version mounts `SettingsShell` (defaulting to Provider Center) + `KeystoreRecovery`. No `translate_clipboard`, no legacy `<select>`/`<textarea>`/key input, no inline `confirm()`. (Task 4)
- `src/App.css` — **trimmed.** Delete `.settings-group`, `.result`, `.error`, the `select,textarea,button` block, and the `:root` font block (now owned by `@linguaray/ui` token CSS / `WindowChrome`). Keep only what the new App mount needs (or delete the file if App mounts no direct layout). (Task 4)

**Unchanged:** `src-tauri/` (all of it), `packages/ui/`, `apps/ui-lab/`, `src/Popup.tsx`, `src/InputPanel.tsx`, `src/features/translation/`, `src/i18n.ts`, `index.html`, `vitest.config.ts`, `vite.config.ts`, `tsconfig.json`.

---

## Task 0: Settings Shell

**Goal:** A `WindowChrome`-based shell with a `SidebarItem` nav (Provider Center / Keystore Recovery / Shortcuts / Privacy) and an adaptive sidebar that is full-label at ≥700px and icon-only at 600–699px. Shortcuts and Privacy are placeholders (disabled, with a tooltip "Coming in R3b"); the shell is content-agnostic (children slot).

**Why before Keystore/Provider:** Both Surface 05 and 06 render inside this shell; building it first lets those tasks render into a stable frame and lets the App-mount task be trivial.

### Step 0.1 — RED: adaptive + nav tests

- [ ] Create `test/SettingsShell.test.tsx`. Import `render`, `screen`, `fireEvent` from `@solidjs/testing-library`; mock `@linguaray/ui` is NOT needed (real components resolve via the vitest alias).

  Tests (all must fail before implementation):
  1. `renders WindowChrome with title "LinguaRay"` — assert `screen.getByText("LinguaRay")` (the title) is present.
  2. `renders all four nav items` — assert `getByRole("button", { name: "Provider Center" })`, `...Keystore Recovery"`, `...Shortcuts"`, `...Privacy"` are all present. Use the `aria-label` carried by `SidebarItem` (the component renders `<button aria-current?>` with `aria-label`? — NO: `SidebarItem` uses the visible `label` text as the accessible name; assert via `getByText` then `.closest("button")`, or assert the button's text content).
  3. `Shortcuts and Privacy are disabled (placeholder)` — assert the two buttons have `disabled` attribute (the `SidebarItem` component sets `disabled={props.disabled}` on the native `<button>`). They are NOT `aria-disabled` fakes — `SidebarItem` already renders a native disabled button.
  4. `clicking Provider Center calls onNavigate("provider-center")` — render with an `onNavigate` spy, click the Provider Center button, assert spy called with `"provider-center"`.
  5. `clicking Keystore Recovery calls onNavigate("keystore-recovery")`.
  6. `at ≥700px viewport, sidebar labels are visible` — use `window.matchMedia` mock (already stubbed in `test/setup.ts` to `matches: false`). For this test, override `matchMedia` to return `matches: true` for the `(min-width: 700px)` query, re-render, assert the label `<span class="sidebar-item__label">` is visible (not `aria-hidden`, not `display:none`). Assert the shell root has a `data-layout="full"` attribute (the component emits this for the CSS hook + test hook).
  7. `at 600–699px viewport, sidebar collapses to icon rail` — override `matchMedia` to return `matches: false`, assert the shell root has `data-layout="rail"`, and assert the label spans carry `aria-hidden="true"` OR are visually hidden (the component hides labels via CSS driven by `data-layout="rail"`; for testability, assert the `data-layout` attribute and that a `<Tooltip>` wraps each item — assert `role="tooltip"` presence or the `aria-describedby` linkage). Document the chosen contract in the component so the test is deterministic.
  8. `initial active section defaults to "provider-center"` — assert the Provider Center button has `aria-current="page"`.

  Run: `pnpm test -- SettingsShell`
  Expected: all FAIL (component does not exist).

- [ ] Commit: `test(r3a): SettingsShell RED tests (adaptive nav shell)`

### Step 0.2 — GREEN: implement SettingsShell

- [ ] Create `src/features/settings/SettingsShell.tsx`:

  ```tsx
  import { createSignal, Show, type Component, type JSX } from "solid-js";
  import { Server, Shield, Keyboard, ShieldCheck } from "lucide-solid";
  import { WindowChrome, SidebarItem, Tooltip } from "@linguaray/ui";
  import { SETTINGS_COPY } from "./copy";
  import { detectLocale } from "../../i18n";
  import "./SettingsShell.css";

  export type SettingsSection = "provider-center" | "keystore-recovery" | "shortcuts" | "privacy";

  export type SettingsShellProps = {
    /** Initial active section (default: "provider-center"). */
    initialSection?: SettingsSection;
    /** Called when the user clicks a nav item. */
    onNavigate?: (section: SettingsSection) => void;
    /** Content for the currently-active section. */
    children: JSX.Element;
  };
  // ... component body: detectLocale(), createSignal<SettingsSection>(initialSection ?? "provider-center"),
  // matchMedia("(min-width: 700px)") for data-layout, SidebarItem list, aria-current on active,
  // disabled+Tooltip on shortcuts/privacy placeholders.
  ```

  Adaptive strategy: read `window.matchMedia("(min-width: 700px)").matches` once on mount and subscribe to its `change` event (cleanup on `onCleanup`). Drive a `data-layout="full" | "rail"` attribute on the shell root. In `SettingsShell.css`, at `[data-layout="rail"]` hide `.sidebar-item__label` (`display: none`) and shrink the sidebar width to the icon-rail token. The `SidebarItem` native button keeps its label text in the DOM (just visually hidden), preserving the accessible name when the `Tooltip` is absent; when `data-layout="rail"`, wrap each item in `<Tooltip>` so hover/focus shows the label.

  Wire `onNavigate`: `SidebarItem` `onClick` sets the active signal AND calls `props.onNavigate?.(section)`. The disabled placeholders do NOT call `onNavigate` (native `disabled` button drops click).

  **WindowChrome wiring:** `<WindowChrome title="LinguaRay" sidebar={<nav>...SidebarItems...</nav>} labels={{ minimize: t.minimize, close: t.close }}>`. For `onClose`/`onMinimize`, call the Tauri window APIs (`getCurrentWindow().close()` / `.minimize()`) — import lazily inside the handler so jsdom tests don't require the Tauri bridge. If the Tauri import throws in a non-Tauri context, swallow it (test environment).

- [ ] Create `src/features/settings/SettingsShell.css` — token-based layout only (no hex outside `var()` fallbacks). Sidebar width tokens: `--settings-sidebar-full: 200px; --settings-sidebar-rail: 56px;` defined inline via existing spacing tokens if available, else as local custom properties set from `--space-*`. The `[data-layout="rail"]` rule hides `.sidebar-item__label` and sets width to the rail value.

- [ ] Create `src/features/settings/copy.ts` now with ONLY the shell keys needed for Task 0 (`nav.providerCenter`, `nav.keystoreRecovery`, `nav.shortcuts`, `nav.privacy`, `nav.placeholderHint`, `window.title`, `window.minimize`, `window.close`). Task 2 extends this file with the full Surface 05 + 06 dictionary. Define `SettingsCopy` as a partial type for now (Task 2 makes it complete) OR define the full shape up front with all keys optional except the shell ones — choose the latter to avoid churn. **Decision:** define the full `SettingsCopy` shape up front (all Surface 05 + 06 keys from the design copy tables) and populate `zh`/`en` for the shell keys now; mark the remaining as `// Task 2` TODOs but DO NOT leave them undefined — the file must compile, so provide all values now and Task 2 just verifies parity. (This avoids a broken intermediate commit.)

- [ ] Run: `pnpm test -- SettingsShell`
  Expected: all PASS.

- [ ] Run: `pnpm test -- no-hardcoded-hex`
  Expected: PASS (no new hex in `src/`).

- [ ] Commit: `feat(r3a): SettingsShell adaptive nav (WindowChrome + SidebarItem)`

---

## Task 1: Keystore Recovery (Surface 06)

**Goal:** A component rendering the 4 states from `design-system/linguaray/pages/06-keystore-recovery.md`: `healthy` / `corrupt` / `archived` / `reset-confirm`. It calls `keystore_health`, `archive_keystore`, `reset_keystore`. The reset `Confirm` is `variant="destructive"` so initial focus lands on Cancel.

**Why Task 1 before Provider Center:** Keystore Recovery is self-contained (3 IPC calls, no provider state) and the smaller TDD loop; it also validates the `Banner` + `Confirm` + `Button` integration against real IPC before the larger Provider Center task.

### Step 1.1 — RED: state + IPC tests

- [ ] Create `test/KeystoreRecovery.test.tsx`. Mock `@tauri-apps/api/core` `invoke` with `vi.fn()`.

  Tests (all must fail before implementation):
  1. `on mount, calls keystore_health once` — render, assert `invoke` called with `"keystore_health"`.
  2. `healthy: renders nothing (no banner)` — mock `invoke` to resolve `""` for `keystore_health`; assert no element with `role="alert"` and no `Banner` title present.
  3. `corrupt: renders destructive banner with the reason` — mock `keystore_health` to resolve `"corrupt: header mismatch"`; assert `getByText(/Keystore unreadable/)` (title) and `getByText(/corrupt: header mismatch/)` (description, interpolating `{reason}`), and assert two buttons: "Archive & re-enter" and "Reset".
  4. `corrupt → Archive: calls archive_keystore, transitions to archived state` — from the corrupt state, click "Archive & re-enter", assert `invoke` called with `"archive_keystore"`, then assert the info banner "Keys archived" + "Enter your keys again" appears.
  5. `corrupt → Reset: opens destructive Confirm; initial focus on Cancel` — click "Reset", assert the `Confirm` dialog is open (`getByText("Reset keystore?")`), assert the Cancel button has focus (`document.activeElement` is the Cancel button). Use `await new Promise(r => setTimeout(r, 0))` after open for Kobalte's focus settle.
  6. `reset Confirm → Cancel: closes dialog, stays corrupt` — open reset dialog, click Cancel, assert dialog closed and still in corrupt state.
  7. `reset Confirm → Confirm: calls reset_keystore, transitions to archived` — open reset dialog, click Confirm, assert `invoke` called with `"reset_keystore"`, assert archived banner.
  8. `archive_keystore rejects: shows error toast` — mock `archive_keystore` to reject `"io error"`; click Archive, assert a destructive Toast with the failure message.
  9. `uses zh copy when locale is zh` — stub `detectLocale` to return `"zh"`, assert Chinese title text.

  Run: `pnpm test -- KeystoreRecovery`
  Expected: all FAIL.

- [ ] Commit: `test(r3a): KeystoreRecovery RED tests (Surface 06)`

### Step 1.2 — GREEN: implement KeystoreRecovery

- [ ] Create `src/features/settings/KeystoreRecovery.tsx`:

  ```tsx
  import { createSignal, onMount, Show, type Component } from "solid-js";
  import { invoke } from "@tauri-apps/api/core";
  import { Banner, Confirm, Button, Toast } from "@linguaray/ui";
  import { SETTINGS_COPY } from "./copy";
  import { detectLocale } from "../../i18n";
  import "./KeystoreRecovery.css";

  type KsState = "healthy" | "corrupt" | "archived";
  // reset-confirm is a transient dialog open-state, not a top-level state.

  const KeystoreRecovery: Component = () => {
    const locale = detectLocale();
    const t = SETTINGS_COPY[locale].keystore;
    const [state, setState] = createSignal<KsState>("healthy");
    const [reason, setReason] = createSignal("");
    const [resetOpen, setResetOpen] = createSignal(false);
    const [busy, setBusy] = createSignal<"archive" | "reset" | null>(null);
    const [toasts, setToasts] = createSignal<{ id: number; variant: "info"|"success"|"warning"|"destructive"; message: string }[]>([]);
    // ... onMount: invoke("keystore_health") → "" healthy, else corrupt + reason.
    // ... handlers: handleArchive (invoke archive_keystore → archived), handleResetConfirm (invoke reset_keystore → archived),
    //               focus-restore refs for the Reset/Archive trigger buttons.
  };
  ```

  - `onMount`: `const h = await invoke<string>("keystore_health");` — `h === ""` → `setState("healthy")`; else `setState("corrupt"); setReason(h);`. Wrap in try/catch: a thrown `keystore_health` itself is a corrupt signal — treat as corrupt with `reason = String(e)`.
  - `healthy`: render nothing (or a `<section aria-label="...">` with no banner). The design matrix says "no banner; settings normal" — render a neutral section so the shell has a content target; no `Banner`.
  - `corrupt`: `<Banner variant="destructive" title={t.title} action={<>ArchiveButton ResetButton</>}>{t.description.replace("{reason}", reason())}</Banner>`. The `Banner` component accepts `title`, `description`/children, `action`. Verify against `packages/ui/src/components/Banner.tsx` props before committing to the exact prop name (the ui-lab uses `Banner` with `title` + `action` + children — match that). The description text: `t.description` is the template `"Keystore unreadable: {reason}"` / `"密钥库不可读：{reason}"` — substitute via `.replace("{reason}", reason())` (NOT template literals, so the copy dictionary stays the single source).
  - `archived`: `<Banner variant="info" title={t.archivedTitle}>{t.archivedPrompt}</Banner>`.
  - Reset button `onClick`: `setResetOpen(true)` (open the Confirm; do NOT call reset yet). Keep a ref to the Reset button for focus restore.
  - `<Confirm open={resetOpen()} variant="destructive" title={t.resetConfirmTitle} message={t.resetConfirmMessage} confirmLabel={t.resetConfirmConfirmLabel} cancelLabel={t.resetConfirmCancelLabel} onConfirm={handleResetConfirm} onCancel={() => setResetOpen(false)} triggerRef={resetTriggerRef}>`. The `variant="destructive"` already focuses Cancel (component contract; do not override `onOpenAutoFocus`).
  - `handleResetConfirm`: `setBusy("reset")` → `await invoke("reset_keystore")` → `setState("archived"); setResetOpen(false)` → on error, push a destructive toast with `t.resetFailed`. Clear busy in finally.
  - `handleArchive`: same pattern with `invoke("archive_keystore")`.
  - Toasts: a stack at the bottom; each `Toast` has `onDismiss`.

- [ ] Create `src/features/settings/KeystoreRecovery.css` — token-based; banner + toast stack spacing only.

- [ ] Run: `pnpm test -- KeystoreRecovery`
  Expected: all PASS.

- [ ] Run: `pnpm test -- no-hardcoded-hex`
  Expected: PASS.

- [ ] Commit: `feat(r3a): KeystoreRecovery (Surface 06, 4 states + destructive reset)`

---

## Task 2: Provider Center front-end state model + IPC + copy

**Goal:** Vendor the pure provider domain logic from ui-lab, type the IPC layer, and complete the copy dictionary. No component rendering yet — this is the data foundation Task 3 consumes.

**Why before Task 3:** The Provider Center component is large; isolating the pure domain + IPC layer gives small, fast TDD loops and lets Task 3 focus purely on wiring.

### Step 2.1 — RED: domain logic tests

- [ ] Create `src/features/settings/provider-domain.test.ts`. This is a near-verbatim port of `apps/ui-lab` domain tests (find them at `apps/ui-lab/src/pages/provider-domain.test.ts` if present — read that file first and port the test cases; otherwise write fresh).

  Tests (must fail before implementation):
  1. `validateActiveSelection: rejects disabled primary` → `{ code: "disabled-in-slot" }`.
  2. `validateActiveSelection: rejects parallel containing primary` → `"parallel-contains-primary"`.
  3. `validateActiveSelection: rejects duplicate in parallel` → `"parallel-duplicate"`.
  4. `validateActiveSelection: rejects non-traditional fallback` → `"fallback-not-traditional"`.
  5. `validateActiveSelection: rejects fallback overlapping primary/parallel` → `"fallback-overlaps"`.
  6. `validateActiveSelection: accepts a valid selection` → `{ ok: true }`.
  7. `buildConsentScope: dedupes + sorts recipients by uuid; excludes fallback`.
  8. `consentScopeKey: stable across recipient reordering`.
  9. `resolveConsentKey: approved key matching new scope → preserved`.
  10. `resolveConsentKey: scope change without approval → null (never auto-mint)`.
  11. `validateEndpoint: https ok; http loopback ok; http non-loopback rejected ("endpoint-must-https"); empty rejected ("endpoint-required"); garbage URL rejected ("endpoint-invalid-url")`.

  The domain functions operate on `ProviderProfileFE` (the frontend-augmented shape), NOT `MockProvider`. The vendored copy must be adapted: `MockProvider.template` → `ProviderProfileFE.template_id`; `MockProvider.sortOrder` → `sort_order`; the field set otherwise matches. Keep the function signatures backward-compatible with the component by accepting `ProviderProfileFE`.

  Run: `pnpm test -- provider-domain`
  Expected: all FAIL.

- [ ] Commit: `test(r3a): provider-domain RED tests (vendored from ui-lab)`

### Step 2.2 — GREEN: vendor provider-domain + provider-types

- [ ] Create `src/features/settings/provider-types.ts` with the TS mirrors listed in "IPC contract" above (`ProviderProfile`, `ProviderPatch`, `ModelInfo`, `ConnectionResult`, `SetActiveResult`, `ProviderCommandError`, `ProviderProtocol`). Add `ProviderProfileFE = ProviderProfile & { hasKey: boolean }` and `ActiveSelection = { primaryUuid: string | null; parallelUuids: string[]; fallbackUuid: string | null }` (matches the ui-lab shape so the ported domain logic is minimal-diff).

- [ ] Create `src/features/settings/provider-domain.ts` by copying `apps/ui-lab/src/pages/provider-domain.ts` and:
  - Rename `MockProvider` references to `ProviderProfileFE`.
  - Change `template: ProviderTemplate` to `template_id: string` (the backend sends template ids as strings; keep `TRADITIONAL_TEMPLATES` as a `Set<string>` of the traditional ids: `google`, `deepl`, `microsoft`, `baidu`, `youdao`, `tencent` — verify against `src-tauri/src/db/providers.rs` `traditional_lookup`).
  - Change `sortOrder` to `sort_order`.
  - Keep `validateActiveSelection`, `buildConsentScope`, `consentScopeKey`, `resolveConsentKey`, `isConsentValid`, `validateEndpoint`, `normalizeOrigin`, `normalizeOrigin` exports.
  - Drop the `provider-domain.ts` header comment's "mock UI Lab" framing; replace with a one-line note that it is vendored from the lab and operates on the production `ProviderProfileFE`.

- [ ] Run: `pnpm test -- provider-domain`
  Expected: all PASS.

- [ ] Commit: `feat(r3a): vendor provider-domain (consent scope + endpoint validation)`

### Step 2.3 — RED: IPC wrapper tests

- [ ] Create `src/features/settings/provider-ipc.test.ts`. Mock `@tauri-apps/api/core` `invoke` with `vi.fn()`.

  Tests (must fail):
  1. `loadProviders() calls provider_list + key_status and joins hasKey` — mock `invoke` to resolve a 2-element `ProviderProfile[]` for `"provider_list"` and `{"provider/u1": true}` for `"key_status"`; assert the result has `hasKey: true` on the matching profile and `false` on the other. Verify `invoke` called with exactly `"provider_list"` then `"key_status"` (no args).
  2. `providerCreate invokes with { templateId, name, endpoint, model }` (Tauri camelCases → snake_case: the invoke arg names MUST match the Rust parameter names as Tauri expects — Tauri converts `template_id` param to `templateId` in JS. **Verify:** Tauri v2 converts snake_case Rust params to camelCase JS keys. So `invoke("provider_create", { templateId, name, endpoint, model })`). Assert the call shape.
  3. `providerUpdate invokes with { uuid, patch }`.
  4. `providerDuplicate`, `providerDelete`, `providerReorder` (`{ uuids }`), `providerToggle` (`{ uuid, enabled }`), `providerSetKey` (`{ uuid, key }`).
  5. `providerSetActive returns SetActiveResult` — mock resolves `{ outcome: "needs_consent", actual_scope: "v1:{...}" }`; assert the wrapper returns it typed.
  6. `providerConfirmAndSetActive returns i64` and `providerConfirmAndSetActive surfaces stale_scope error` — mock rejects with `{ error: "stale_scope", actual_scope: "..." }`; assert the wrapper re-throws the structured error (or resolves it into a typed `ProviderCommandError` — pick one contract and document it; recommended: let the rejection propagate as-is, the component catches and narrows on `e.error === "stale_scope"`).
  7. `providerGetModels`, `providerTestConnection`.
  8. `keystoreHealth`, `archiveKeystore`, `resetKeystore`, `keyStatus`.

  Run: `pnpm test -- provider-ipc`
  Expected: all FAIL.

- [ ] Commit: `test(r3a): provider-ipc RED tests (typed invoke wrappers)`

### Step 2.4 — GREEN: implement provider-ipc

- [ ] Create `src/features/settings/provider-ipc.ts`:

  ```ts
  import { invoke } from "@tauri-apps/api/core";
  import type {
    ProviderProfile, ProviderProfileFE, ProviderPatch, ModelInfo,
    ConnectionResult, SetActiveResult, ProviderCommandError,
  } from "./provider-types";

  export async function loadProviders(): Promise<ProviderProfileFE[]> {
    const [profiles, keyMap] = await Promise.all([
      invoke<ProviderProfile[]>("provider_list"),
      invoke<Record<string, boolean>>("key_status"),
    ]);
    return profiles.map((p) => ({ ...p, hasKey: !!keyMap[p.secret_ref] }));
  }
  export const providerCreate = (templateId: string, name: string, endpoint: string, model?: string) =>
    invoke<ProviderProfile>("provider_create", { templateId, name, endpoint, model: model ?? null });
  export const providerUpdate = (uuid: string, patch: ProviderPatch) =>
    invoke<ProviderProfile>("provider_update", { uuid, patch });
  export const providerDuplicate = (uuid: string) => invoke<ProviderProfile>("provider_duplicate", { uuid });
  export const providerDelete = (uuid: string) => invoke<void>("provider_delete", { uuid });
  export const providerReorder = (uuids: string[]) => invoke<void>("provider_reorder", { uuids });
  export const providerToggle = (uuid: string, enabled: boolean) => invoke<void>("provider_toggle", { uuid, enabled });
  export const providerSetKey = (uuid: string, key: string) => invoke<void>("provider_set_key", { uuid, key });
  export const providerSetActive = (primary: string, parallel: string[], fallback: string | null) =>
    invoke<SetActiveResult>("provider_set_active", { primary, parallel, fallback });
  export const providerConfirmAndSetActive = (primary: string, parallel: string[], fallback: string | null, expectedScope: string) =>
    invoke<number>("provider_confirm_and_set_active", { primary, parallel, fallback, expectedScope });
  export const providerGetModels = (uuid: string) => invoke<ModelInfo[]>("provider_get_models", { uuid });
  export const providerTestConnection = (uuid: string) => invoke<ConnectionResult>("provider_test_connection", { uuid });
  export const keystoreHealth = () => invoke<string>("keystore_health");
  export const archiveKeystore = () => invoke<string>("archive_keystore");
  export const resetKeystore = () => invoke<string | null>("reset_keystore");
  export const keyStatus = () => invoke<Record<string, boolean>>("key_status");
  ```

  Re-export `ProviderCommandError` type for component catch-narrowing. Document the stale-scope contract: the wrapper does NOT swallow the rejection; the component catches and narrows on `e?.error === "stale_scope"`.

- [ ] Run: `pnpm test -- provider-ipc`
  Expected: all PASS.

- [ ] Commit: `feat(r3a): provider-ipc typed invoke wrappers`

### Step 2.5 — RED+GREEN: complete the copy dictionary

- [ ] Create `src/features/settings/copy.test.ts`:
  1. `every SettingsCopy key is present in both zh and en` — for each top-level group, assert `SETTINGS_COPY.zh[group]` and `SETTINGS_COPY.en[group]` have the same key sets.
  2. `placeholder tokens ({name}, {reason}, {latency}, {message}) match across locales` — for each string containing a `{token}`, assert the other locale has the same `{token}`.
  3. `Surface 05 + 06 design copy keys are all present` — assert specific keys from `design-system/linguaray/pages/05-provider-center.md` and `06-keystore-recovery.md` exist (e.g. `provider.empty.title`, `keystore.reset.confirm.title`).

- [ ] Complete `src/features/settings/copy.ts` `SETTINGS_COPY`: port the full provider string set from `apps/ui-lab/src/i18n/index.ts` (the `provider` block, ~95 keys) and the keystore block from `06-keystore-recovery.md`. Replace the lab-only keys (`frameMin`, `frameDefault`, etc.) — drop them. Source of truth for copy is the design `.md` copy tables; the lab strings are a faithful superset, so porting them is correct, but reconcile any drift against the `.md` (the `.md` wins).

- [ ] Run: `pnpm test -- copy`
  Expected: PASS.

- [ ] Run: `pnpm test` (full suite so far)
  Expected: PASS — SettingsShell, KeystoreRecovery, provider-domain, provider-ipc, copy.

- [ ] Commit: `feat(r3a): settings copy dictionary (Surface 05 + 06, zh/en)`

---

## Task 3: Provider Center component (Surface 05)

**Goal:** Port `apps/ui-lab/src/pages/ProviderCenter.tsx` into production `src/features/settings/ProviderCenter.tsx`, rewiring every mock-data path to real IPC via the Task 2 wrappers. Cover the core states from the design matrix: empty / list / editing / key-saving / deleting / consent / reorder / connection-test.

**Adaptations from the lab prototype (document each in the component):**
- Drop the `props.state` / `props.t` / `props.locale` demo-driver props — the component owns its own real state. The lab's `createEffect(on(() => props.state, ...))` fixture-injection is DELETED entirely.
- Drop the mock fixtures (`initialProviders`, `mockUuid`, `DEFAULT_SELECTION`).
- `providers()` is loaded via `loadProviders()` on mount; mutations call the IPC wrappers then re-fetch (or apply the returned `ProviderProfile` optimistically with rollback on error — pick optimistic-with-rollback for toggle/reorder, re-fetch for create/delete to get the canonical tombstone state).
- `selection()` is the client mirror (Known R3a limitation #1): start `{ primaryUuid: null, parallelUuids: [], fallbackUuid: null }`; role assignments call `providerSetActive`/`providerConfirmAndSetActive`. Cold-load roles render as "none". Add the `// TODO(r3b)` note.
- Balance section renders the muted TODO note (Known R3a limitation #2), not the lab's mock states.
- Connection test calls `providerTestConnection` and renders the real `{ ok, message }`.
- Drop the lab's `OpRegistry`/`scheduleTracked` mock timers — real IPC awaits replace them. Per-UUID busy signals remain (they drive `Button loading`), but they flip on await-start and clear on await-settle (no `setTimeout`).
- Drop the lab's `ProviderCard` + role action icons in favor of the R1 `ProviderRow`? **Decision:** the lab uses `ProviderCard` (NOT exported from `@linguaray/ui` — only `ProviderRow` is). The task spec says "用 @linguaray/ui 的 ProviderRow". So **rebuild the provider list rows with `ProviderRow`** instead of porting `ProviderCard`. This changes the row layout: `ProviderRow` renders name + template + StatusBadge + Switch + edit + delete. Role assignment (primary/parallel/fallback) is NOT a `ProviderRow` feature — add role actions as separate icon buttons in the row wrapper (mirroring the lab's `extraActions` pattern but `ProviderRow` has no `extraActions` slot, so render them in a sibling element within the row's container). Document this layout divergence from the lab.

### Step 3.1 — RED: core flow tests

- [ ] Create `test/ProviderCenter.test.tsx`. Mock `@tauri-apps/api/core` `invoke` with a `vi.fn()` whose implementation routes by command name to fixtures.

  Tests (must fail before implementation):
  1. `on mount, calls provider_list + key_status` — assert both invoked.
  2. `empty: shows EmptyState + preset grid` — mock `provider_list` → `[]`; assert `getByText(t.empty.title)` and the preset buttons (OpenAI, Anthropic, Gemini, DeepSeek, Google Translate, DeepL, Ollama).
  3. `empty → click preset: calls provider_create, re-fetches` — click "OpenAI"; assert `invoke` called with `"provider_create"` `{ templateId: "openai", name: "OpenAI", endpoint: ..., model: null }`; then `provider_list` called again.
  4. `list: renders rows in sort_order with name + template` — mock `provider_list` → 2 profiles; assert both names render, order matches `sort_order`.
  5. `toggle: calls provider_toggle, optimistically flips, rolls back on error` — flip a row's switch; assert `"provider_toggle"` called; on success the switch reflects new state; on reject, the switch reverts and a destructive toast shows.
  6. `edit: selecting a row opens detail with endpoint + model fields` — click edit on a row; assert the detail panel shows the endpoint `TextField` and model `Select`.
  7. `endpoint invalid: shows error, Save disabled` — type `http://evil.com` into endpoint; assert the error text `t.endpointErrors["endpoint-must-https"]` and that Save is disabled/not committing.
  8. `save profile: calls provider_update with patch` — set a valid endpoint, click Save; assert `"provider_update"` called with `{ uuid, patch: { endpoint: ... } }`.
  9. `key missing: shows key input + Save key button; saving calls provider_set_key` — select a no-key provider; type a key; click Save key; assert `"provider_set_key"` called with `{ uuid, key }`; on success the key-saved indicator shows and the input clears.
  10. `key never re-read after submit: input cleared on submit start, even on failure` — assert the input value is empty after click regardless of resolve/reject.
  11. `delete: opens Confirm; confirm calls provider_delete` — click delete; assert Confirm open; confirm; assert `"provider_delete"` called; row enters deleting state then is removed after re-fetch.
  12. `set primary: calls provider_set_active({ primary, parallel: [], fallback })` — click set-primary on a row; assert the call; on `outcome:"written"` the role badge updates; on `outcome:"needs_consent"` (only relevant for parallel) the consent dialog opens.
  13. `add parallel → needs_consent → consent Confirm → provider_confirm_and_set_active` — click add-parallel; mock `provider_set_active` resolves `{ outcome: "needs_consent", actual_scope }`; assert consent dialog open with recipient list; confirm; assert `"provider_confirm_and_set_active"` called with `{ primary, parallel, fallback, expectedScope: actual_scope }`.
  14. `add parallel → stale_scope on confirm → toast, selection reverted` — mock confirm rejects `{ error: "stale_scope", actual_scope }`; assert destructive toast and the parallel add undone.
  15. `reorder: move up calls provider_reorder with new uuid order` — click move-up on row 2; assert `"provider_reorder"` called with the swapped uuid list; on reject, order reverts + destructive toast.
  16. `connection test: calls provider_test_connection; ok → green badge with latency-ish message; failed → red` — click Test; assert the call; on `{ok:true}` assert the connected indicator; on `{ok:false}` assert the failed indicator.
  17. `balance section: renders TODO note, no fetch button` — assert the muted TODO text and that no balance-fetch button exists.
  18. `uses zh copy when locale zh` — stub locale, assert a Chinese label.
  19. `no role badges on cold load (session-only roles)` — fresh mount with 2 providers; assert neither shows a primary/parallel/fallback badge until assigned.

  Run: `pnpm test -- ProviderCenter`
  Expected: all FAIL.

- [ ] Commit: `test(r3a): ProviderCenter RED tests (Surface 05 real IPC)`

### Step 3.2 — GREEN: implement ProviderCenter

- [ ] Create `src/features/settings/ProviderCenter.tsx`:
  - On mount: `const list = await loadProviders(); setProviders(list);`. Catch → push destructive toast, render `InlineError` retry.
  - State signals: `providers: ProviderProfileFE[]`, `selection: ActiveSelection` (session mirror, null on cold load), `selectedUuid`, per-UUID `keyInput`, `endpointDraft`, `modelDraft`, `saveState`, `connState`, `deleteConfirmUuid`, `consentState` (`{ open, pendingParallelUuid, actualScope? }`), `toasts`.
  - `roleFor(uuid)` uses `selection()` — same logic as the lab.
  - Mutations (all `async`):
    - `handleToggle(uuid, enabled)`: optimistic flip → `providerToggle` → on error revert + toast; on success keep (the backend already evicts slots, but our session mirror must also evict: if disabled, clear from selection).
    - `handleSetPrimary(uuid)`: build candidate selection → `providerSetActive` → on `Written` commit session selection; on `NeedsConsent` (only if parallel non-empty — but set-primary alone has empty parallel so this path is for add-parallel). For pure set-primary, parallel is empty so it always writes.
    - `handleAddParallel(uuid)`: build candidate with `parallelUuids: [...prev, uuid]` → `providerSetActive` → on `Written` commit; on `NeedsConsent { actual_scope }` open consent dialog storing `actualScope`.
    - `confirmConsent()`: call `providerConfirmAndSetActive(primary, parallel, fallback, expectedScope = storedActualScope)` → on `number` (version) commit selection + close dialog; on reject `{ error: "stale_scope" }` toast + revert + close.
    - `handleSetFallback`, `handleRemoveParallel`: same `providerSetActive` pattern.
    - `handleAddPreset(preset)`: call `providerCreate(templateId, name, endpoint, model)` → push returned profile → re-fetch for canonical order, or insert at end with `sort_order = providers().length` optimistically.
    - `handleSaveProfile(uuid)`: `validateEndpoint` locally first; on ok call `providerUpdate(uuid, { endpoint, model })` → commit returned profile; on error toast + keep draft.
    - `handleSaveKey()`: capture uuid; clear `keyInput[uuid]` IMMEDIATELY; `setSaveState[uuid]="saving"` → `providerSetKey(uuid, key)` → on success `setSaveState[uuid]="saved"`, re-fetch `keyStatus` to update `hasKey`; on error `setSaveState[uuid]="failed"` + toast. Key is NEVER re-read; input stays cleared.
    - `handleFetchModels(uuid)`: `providerGetModels` → populate a per-UUID model-options store; on error show manual-entry `TextField`.
    - `handleTestConnection(uuid)`: `providerTestConnection` → set `connState[uuid] = { ok, message }`.
    - `handleDelete(uuid)`: open Confirm; `confirmDelete()` → `providerDelete(uuid)` → on success set row to deleting then re-fetch (the backend returns `()` after tombstone); on error retry toast.
    - `handleReorder(fromUuid, toUuid, pos)` / `moveProvider(uuid, dir)`: compute new uuid order locally → optimistic reorder → `providerReorder(newUuids)` → on error revert + toast.
    - `handleDuplicate(uuid)`: `providerDuplicate` → push returned profile (keyless).
  - Rendering: sidebar (provider list using `ProviderRow` + role-action icon buttons in a sibling container) + detail panel (endpoint `TextField`, model `Select`/manual `TextField`, key section, connection-test `Button` + result indicator, balance TODO note). Empty state when `providers().length === 0`. Toasts stack. Delete + consent `Confirm` dialogs at the end.
  - i18n: all strings via `SETTINGS_COPY[detectLocale()].provider`. The `{name}` / `{latency}` / `{message}` placeholders substituted at the call site via `.replace("{name}", p.name)` etc.

- [ ] Create `src/features/settings/ProviderCenter.css` — port from `apps/ui-lab/src/pages/ProviderCenter.css`, dropping the `pc__settings-shell`/`pc__settings-rail`/`pc__content` rules (now owned by `SettingsShell`) and the lab frame-marker rules. Keep list/detail/balance/conn/consent layout. Token-based; no hex outside `var()` fallbacks.

- [ ] Run: `pnpm test -- ProviderCenter`
  Expected: all PASS.

- [ ] Run: `pnpm test -- no-hardcoded-hex`
  Expected: PASS.

- [ ] Commit: `feat(r3a): ProviderCenter (Surface 05, real IPC)`

---

## Task 4: App.tsx full replacement + App.css cleanup

**Goal:** Replace the legacy `src/App.tsx` with a thin mount of `SettingsShell` + `ProviderCenter` (+ `KeystoreRecovery` reachable via nav). Delete the legacy `.settings-group`/translate UI from `App.css`. The translation/clipboard functionality that lived in legacy App.tsx is NOT in scope for R3a (it belonged to the legacy combined window; the popup/input surfaces from R2b own live translation).

### Step 4.1 — RED: App mount test

- [ ] Create `test/App.test.tsx`:
  1. `renders SettingsShell with Provider Center active by default` — assert `getByText("LinguaRay")` (title) and a provider-center nav button with `aria-current="page"`.
  2. `clicking Keystore Recovery nav swaps content to KeystoreRecovery` — click the nav; assert the keystore section appears (e.g. the section `aria-label`).
  3. `no legacy elements remain` — assert NO `<textarea>`, NO element with class `settings-group`, NO `translate_clipboard`-bearing button (assert `queryAllByRole("textbox")` are only the provider-center TextFields, or assert the document does not contain text "Translate clipboard").
  4. `does not call translate/translate_clipboard on mount` — assert the `invoke` mock is only called with provider/keystore commands.

  Run: `pnpm test -- App`
  Expected: FAIL (legacy App still mounted).

- [ ] Commit: `test(r3a): App mount RED tests (shell swap, no legacy)`

### Step 4.2 — GREEN: replace App.tsx + trim App.css

- [ ] Replace `src/App.tsx` entirely:

  ```tsx
  import { createSignal, type Component } from "solid-js";
  import SettingsShell, { type SettingsSection } from "./features/settings/SettingsShell";
  ProviderCenter from "./features/settings/ProviderCenter";
  KeystoreRecovery from "./features/settings/KeystoreRecovery";
  import "./App.css"; // (only if any root-level rules remain after trim)

  const App: Component = () => {
    const [section, setSection] = createSignal<SettingsSection>("provider-center");
    return (
      <SettingsShell initialSection="provider-center" onNavigate={setSection}>
        {section() === "provider-center" ? <ProviderCenter /> : section() === "keystore-recovery" ? <KeystoreRecovery /> : <PlaceholderSection />}
      </SettingsShell>
    );
  };
  export default App;
  ```

  `PlaceholderSection` (inline): a simple `<section aria-label="...">{t.comingSoon}</section>` for shortcuts/privacy (R3b). No mock content.

- [ ] Edit `src/App.css`: delete `.settings-group`, `.result`, `.error`, the `select,textarea,button` shared block, the `label` block, the `.subtitle` block, the `.container` block (now owned by WindowChrome), and the `:root` font block. If nothing remains, delete the file and remove the import from `App.tsx`. If a minimal root reset is still needed (e.g. `#root { height: 100%; }`), keep it as token-based rules only.

- [ ] Run: `pnpm test -- App`
  Expected: PASS.

- [ ] Run: `pnpm test -- no-hardcoded-hex`
  Expected: PASS.

- [ ] Commit: `feat(r3a): replace App.tsx with SettingsShell mount; trim App.css`

---

## Final Verification

After all five tasks are complete, run the full verification suite. Every command MUST pass.

- [ ] **Step 1: Production unit + component tests**

  Run: `pnpm test`
  Expected: PASS — SettingsShell, KeystoreRecovery, provider-domain, provider-ipc, copy, ProviderCenter, App, plus the existing R2b tests (decode, op-registry, Popup, InputPanel, smoke, no-hardcoded-hex) still green.

- [ ] **Step 2: `@linguaray/ui` package tests (regression check)**

  Run: `pnpm --filter @linguaray/ui test`
  Expected: PASS.

- [ ] **Step 3: ui-lab tests (regression check — lab is NOT modified, must still pass)**

  Run: `pnpm --filter @linguaray/ui-lab test`
  Expected: PASS.

- [ ] **Step 4: Full workspace typecheck**

  Run: `pnpm typecheck`
  Expected: PASS across root + `@linguaray/ui` + `@linguaray/ui-lab`. The new `src/features/settings/*` files typecheck cleanly against the mirrored IPC wire types.

- [ ] **Step 5: Production build**

  Run: `pnpm build`
  Expected: PASS — produces `dist/` with the `main` entry. No Vite errors about `@linguaray/ui`, missing token CSS, or unresolved `src/features/settings/*` imports.

- [ ] **Step 6: Hex-free scan**

  Run: `pnpm test -- no-hardcoded-hex`
  Expected: PASS — no raw hex outside `var()` fallbacks anywhere under `src/`, including the new `src/features/settings/*.css`.

- [ ] **Step 7: Manual smoke (documented, not automated)**

  Launch the settings window (`pnpm tauri dev`). Verify:
  - Sidebar nav switches between Provider Center, Keystore Recovery, and the two disabled placeholders.
  - At ≥700px the sidebar shows labels; drag to 600–699px and confirm it collapses to the icon rail with tooltips.
  - Provider Center: add a preset, set a key, toggle, assign primary, test connection, reorder, delete (with Confirm focus on Cancel).
  - Keystore Recovery: with a healthy keystore, no banner; (manual corrupt is hard to simulate without breaking the keystore — leave as code-reviewed).

---

## Rev-4 Retroactive Status (2026-08-09)

Appended by the R2/R3a contract audit (docs/superpowers/plans/2026-08-09-r2-r3-contract-audit-fixes.md).
Historical RED states are preserved as-written; this table records the actual
shipped state and where gaps are closed. Each "Shipped?" claim was verified
against the current source tree (file/function grep) at append time.

| Original task | Shipped? | Gap closed in (audit task) |
|---|---|---|
| Task 0: Settings Shell | yes — `src/features/settings/SettingsShell.tsx`; controlled `activePage` derivation + `data-page` + matchMedia rail + WindowChrome + Tooltip + close/minimize | C5 (rail-mode aria-disabled + keyboard nav), C4 (entry styling + window sizing + theme-color meta) |
| Task 1: Keystore Recovery (Surface 06) | yes — `src/features/settings/KeystoreRecovery.tsx`; healthy/corrupt/archived states + fail-closed banner | — |
| Task 2: Provider Center front-end state model + IPC + copy | yes — `src/features/settings/copy.ts` (`connectionOk/Failed`, `{latency}`/`{message}` placeholders); state model + IPC in `ProviderCenter.tsx` | C3c (latency `{latency}ms` chip + saturation + Instant probe), C2 (Google/DeepL dropped from presets) |
| Task 3: Provider Center component (Surface 05) | yes — `src/features/settings/ProviderCenter.tsx` (42KB: add preset, set key, toggle, primary, test connection, reorder, delete, model fetch) | C3a–C3h (8 sub-tasks: duplicate, empty-key/conflict, connection+latency, delete focus, disabled roles, balance placeholder, model fetch, toast+tooltip), C1 (cold-start active selection fail-closed) |
| Task 4: App.tsx full replacement + App.css cleanup | yes — `src/App.tsx` hosts tray-action + navigate listeners + controlled `activePage`; `src/App.css` token-only (no hardcoded hex) | A4 (tray-action + navigate event wiring), A1 (theme bootstrap), D1 (`--space-N` aliases swept; no hardcoded hex guard) |
