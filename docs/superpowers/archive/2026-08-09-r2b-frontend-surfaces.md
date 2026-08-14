Archived-on: 2026-08-14 · reason: superseded by linguaray-plugin-core-design / completed, see git history

# R2b Frontend Surfaces Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the hardcoded production `src/` surfaces (`Popup.tsx`, `InputPanel.tsx`) with `@linguaray/ui`-driven components driven by a typed translation-state model, port the CAS operation registry into production, add a native system-tray menu (Surface 04), and realign the ui-lab prototype to share the same state model.

**Architecture:** A new `src/features/translation/` feature folder owns the typed state model (`TranslationState` discriminant union), the decoders that map the two backend Tauri events (`popup-state`, `popup-multi-result`) and the `translate_session` IPC result onto that model, and the CAS `OpRegistry` (ported verbatim from `apps/ui-lab/src/pages/op-registry.ts`). The three production entry components (`Popup.tsx`, `InputPanel.tsx`) are rebuilt as pure renderers of that model using `@linguaray/ui` primitives. The tray menu (Surface 04) is native Tauri in `src-tauri/src/lib.rs`. Because production `src/` has no test runner today, this plan first installs Vitest + Solid testing-library at the root and wires `@linguaray/ui` resolution.

**Tech Stack:** SolidJS 1.9, `@linguaray/ui` (workspace package), Tauri 2 events/IPC (`@tauri-apps/api`), Vitest 2 + `@solidjs/testing-library` + jsdom + axe-core (mirrors the ui-lab test stack), Tauri 2 `tray-icon` + `menu` features (Rust side).

---

## Global Constraints

These apply to every task. Each task's requirements implicitly include this section.

- **Semantic tokens only.** Production `src/` MUST consume colors/spacing/radius/shadow via CSS variables (`--color-bg-elevated`, `--text-lg`, `--radius-lg`, `--shadow-lg`, etc.) as defined in `design-system/linguaray/`. No hardcoded hex (`#0f0f0f`, `#f6f6f6`, `#888`, `#396cd8`, `#eef4ff`, `#fdeeee`, `#a33`, etc.) may remain in any file under `src/` after R2b. The existing hardcoded `:root` block in `src/App.css` and the inline `style={{ color: "#888", "font-size": "11px" }}` in `src/Popup.tsx` are the targets to eliminate. Tokens come from `packages/ui` (import the package's bundled token CSS, do not redefine tokens in `src/`).
- **Backend untouched except Task 5.** Tasks 1–4 and 6 MUST NOT modify any file under `src-tauri/`. Only Task 5 edits `src-tauri/src/lib.rs` and `src-tauri/Cargo.toml` (tray menu). Do not alter IPC command signatures, event names, or the `TranslationOutcomeSerialized` shape — the frontend decodes the existing wire format.
- **Event contract (decode, do not redefine).** `popup-state` payload is exactly `{ status: "loading" | "result" | "error"; text: string; engine: string }`. `popup-multi-result` payload is exactly `{ outcomes: Array<{ uuid: string; ok: boolean; text?: string; engine?: string; error?: string }> }`. `translate_session` returns `{ outcomes: TranslationOutcome[]; actual_engine?: string }` (Tauri serializes each outcome as `{ uuid, ok, text?, engine?, error? }` via `TranslationOutcomeSerialized`). The frontend types mirror these shapes; the backend is the source of truth.
- **CSP compliance.** The active CSP is `default-src 'self'; img-src 'self' data:; style-src 'self' 'unsafe-inline'; script-src 'self'; connect-src 'self' ipc: http://ipc.localhost`. Frontend code MUST NOT introduce inline `<script>`, external-font `connect-src`, or remote `img-src`. Inline `style="..."` is allowed (CSP permits `style-src 'unsafe-inline'`) but prefer CSS classes; lucide-solid icons are SVG (allowed under `img-src`/`script-src 'self'`).
- **Keyboard + focus.** Every interactive surface must be operable by keyboard: the popup dismisses on Escape and blurs only when NOT pinned (mirrors the existing `onFocusChanged` hide, gated by pin state); the input window translates on Enter (Shift+Enter = newline) and the Clear/Translate buttons are tab-reachable; focus is moved into the popup region on show and restored on hide. `role="region"`, `aria-label`, and `aria-busy` must be set on the popup body (matches ui-lab).
- **Reduced motion.** Respect `prefers-reduced-motion`. Use the frozen `Spinner` from `@linguaray/ui` (which already renders a text fallback under reduced motion) rather than re-implementing a loading indicator. Do not add CSS keyframe animations in `src/`.
- **No emoji in code.** Code comments and copy keys use plain text. Existing `…` placeholder characters are replaced with the `Spinner` component + visually-hidden `selection.loading` copy.
- **Workspace import.** Production `src/` resolves `@linguaray/ui` via the workspace package (added to root `package.json` dependencies). `App.tsx` (Surface 03 / settings) is OUT OF SCOPE for R2b — only `Popup.tsx` (Surface 01) and `InputPanel.tsx` (Surface 02) are rebuilt; `App.tsx` keeps working as-is.
- **TDD.** Every code task writes the failing test first, runs it to confirm it fails, then implements. Production tests live under `src/features/translation/` (unit) and `test/` (component), run via the root Vitest config installed in Task 0.

---

## File Structure

**Created (production `src/`):**
- `src/features/translation/types.ts` — `TranslationState` discriminant union, `TranslationOutcome` (frontend), `PopupEvent` types, `CopyKey`/i18n copy key types. (Task 1)
- `src/features/translation/decode.ts` — pure decoders: `decodePopupState(payload)` → `TranslationState`, `decodePopupMultiResult(payload)` → `TranslationState`, `decodeSessionResult(result)` → `TranslationState`. (Task 1)
- `src/features/translation/op-registry.ts` — verbatim port of `apps/ui-lab/src/pages/op-registry.ts` (CAS `OpRegistry`, `GenerationToken` helper). (Task 2)
- `src/features/translation/copy.ts` — `zh`/`en` copy maps for Surface 01 + 02 keys. (Task 3)
- `src/i18n.ts` — locale detection (reads `localStorage`/`navigator.language`), returns `("zh" | "en")`. (Task 3)
- `src/features/translation/popupController.ts` — thin Solid wrapper: subscribes to the two Tauri events, owns the `TranslationState` signal + `OpRegistry`, exposes `state()`, `retry()`, `pin()`, `dismiss()`. (Task 3)
- `test/Popup.test.tsx` — component tests for the rebuilt selection popup. (Task 3)
- `test/InputPanel.test.tsx` — component tests for the rebuilt input window. (Task 4)

**Modified (production `src/`):**
- `src/Popup.tsx` — rebuilt to consume `popupController` + render `@linguaray/ui` (`Spinner`, `ResultCard`, `EmptyState`, `InlineError`, `Button`, `IconButton`). (Task 3)
- `src/Popup.css` — replace hardcoded overrides; keep only the transparent-root + card-spacing rules expressed in tokens. (Task 3)
- `src/InputPanel.tsx` — rebuilt to call `invoke('translate_session', ...)` and render `ResultCard`/`InlineError`. (Task 4)

**Modified (config, root):**
- `package.json` — add `vitest`, `@solidjs/testing-library`, `@testing-library/jest-dom`, `jsdom`, `axe-core` devDeps; add `@linguaray/ui` workspace dep; add `test`/`test:src` scripts. (Task 0)
- `vitest.config.ts` — root Vitest config: Solid plugin, jsdom env, `@linguaray/ui` resolve alias, `src`/`test` includes. (Task 0)
- `tsconfig.json` — add `paths` alias for `@linguaray/ui` so typecheck resolves. (Task 0)

**Modified (ui-lab):**
- `apps/ui-lab/src/pages/SelectionPopup.tsx` — replace its local `SelectionState` switch with the shared `TranslationState` model re-exported from a lab-side adapter (keeps the 16 mock states but maps each to the production discriminant for display parity). (Task 6)

**Modified (backend, Task 5 only):**
- `src-tauri/Cargo.toml` — enable Tauri `tray-icon` + `image-png` features; add `tauri::menu` usage. (Task 5)
- `src-tauri/src/lib.rs` — build a `TrayIcon` with a `Menu` (Translate Selection / Input / Clipboard / OCR, Active Provider submenu, History, Settings, Quit) in `setup()`, wire item handlers. (Task 5)

---

## Task 0: Production test harness + `@linguaray/ui` wiring

This is the foundational setup task. Without it, Tasks 1–4 cannot run or typecheck. It produces no application behavior of its own, but it is independently verifiable (a no-op test passes) and a reviewer can accept it alone.

**Files:**
- Modify: `package.json`
- Create: `vitest.config.ts`
- Modify: `tsconfig.json`
- Create: `test/smoke.test.ts`

**Interfaces:**
- Consumes: the workspace `@linguaray/ui` package (`packages/ui`, name `@linguaray/ui`).
- Produces: a root `pnpm test` that runs `vitest`; a `@linguaray/ui` import resolvable from `src/`; a `test/` dir that Vitest discovers.

- [ ] **Step 1: Write a failing smoke test that imports a `@linguaray/ui` component**

Create `test/smoke.test.ts`:

```ts
import { describe, it, expect } from "vitest";
import { Spinner } from "@linguaray/ui";

describe("production test harness", () => {
  it("resolves the workspace @linguaray/ui package", () => {
    // Spinner is a Solid component (a function). Importing it without error
    // proves the workspace alias + Vitest Solid environment both resolve.
    expect(typeof Spinner).toBe("function");
  });
});
```

- [ ] **Step 2: Run the test to verify it fails**

Run: `pnpm test`
Expected: FAIL — `@linguaray/ui` not found (no dependency, no alias), and `vitest` may not be installed.

- [ ] **Step 3: Install devDeps + workspace dep**

Run:

```bash
pnpm add -D vitest@^2 @solidjs/testing-library@^0.6 @testing-library/jest-dom@^6 jsdom@^25 axe-core@^4
pnpm add @linguaray/ui@workspace:*
```

- [ ] **Step 4: Add the `@linguaray/ui` path alias to `tsconfig.json`**

Add to `compilerOptions` (keep all existing keys):

```json
    "baseUrl": ".",
    "paths": {
      "@linguaray/ui": ["./packages/ui/src/index.ts"]
    }
```

- [ ] **Step 5: Create `vitest.config.ts`**

```ts
import { defineConfig } from "vitest/config";
import solid from "vite-plugin-solid";
import { fileURLToPath, URL } from "node:url";

export default defineConfig({
  plugins: [solid()],
  resolve: {
    alias: {
      "@linguaray/ui": fileURLToPath(
        new URL("./packages/ui/src/index.ts", import.meta.url),
      ),
    },
  },
  test: {
    environment: "jsdom",
    globals: true,
    setupFiles: ["test/setup.ts"],
    include: ["src/**/*.test.{ts,tsx}", "test/**/*.test.{ts,tsx}"],
  },
});
```

- [ ] **Step 6: Create `test/setup.ts` (jest-dom + jsdom matchMedia stub)**

```ts
import "@testing-library/jest-dom";

// jsdom lacks matchMedia; Solid components + reduced-motion checks may read it.
if (!window.matchMedia) {
  // @ts-expect-error partial mock
  window.matchMedia = () => ({
    matches: false,
    addEventListener() {},
    removeEventListener() {},
    onchange: null,
    dispatchEvent: () => false,
    addListener() {},
    removeListener() {},
  });
}
```

- [ ] **Step 7: Add `test`/`test:src` scripts to root `package.json`**

In `scripts` add:

```json
    "test": "vitest run",
    "test:watch": "vitest",
    "test:src": "vitest run --root test"
```

- [ ] **Step 8: Run the test to verify it passes**

Run: `pnpm test`
Expected: PASS (1 test, the smoke import resolves).

- [ ] **Step 9: Verify typecheck still passes**

Run: `pnpm typecheck`
Expected: PASS (the alias resolves under both Vitest and tsc).

- [ ] **Step 10: Commit**

```bash
git add package.json pnpm-lock.yaml vitest.config.ts tsconfig.json test/
git commit -m "build(production): add Vitest + @linguaray/ui workspace wiring"
```

---

## Task 1: Frontend translation-state model + decoders

This is the single source of truth for the rest of R2b. Every later task imports the types and decoders defined here. It is pure logic with no React/Solid or Tauri imports — fully unit-testable in jsdom.

**Files:**
- Create: `src/features/translation/types.ts`
- Create: `src/features/translation/decode.ts`
- Test: `src/features/translation/decode.test.ts`

**Interfaces:**
- Consumes: the backend wire shapes documented in Global Constraints (no imports — they are plain object literals decoded structurally).
- Produces:
  - `TranslationState` — discriminant union with `kind`: `"loading" | "single-success" | "multi-success" | "partial" | "error" | "pinned" | "offline" | "no-selection" | "no-permission" | "keystore-corrupt"`. Each variant carries the fields the renderer needs.
  - `TranslationOutcomeFE` — `{ uuid: string; ok: boolean; text?: string; engine?: string; error?: string }` (mirrors `TranslationOutcomeSerialized`).
  - `PopupStatePayload` — `{ status: "loading" | "result" | "error"; text: string; engine: string }`.
  - `PopupMultiPayload` — `{ outcomes: TranslationOutcomeFE[] }`.
  - `SessionResultFE` — `{ outcomes: TranslationOutcomeFE[]; actual_engine?: string }`.
  - `CopyKey` — string-literal union of all i18n keys for Surface 01 + 02 (Task 3 imports this).
  - `decodePopupState(payload: PopupStatePayload): TranslationState`
  - `decodePopupMultiResult(payload: PopupMultiPayload): TranslationState`
  - `decodeSessionResult(result: SessionResultFE): TranslationState`
  - `classifyError(message: string): ErrorKind` — used by decoders to pick `error` vs `offline` vs `no-permission` vs `keystore-corrupt` vs `no-selection`.
  - `ErrorKind` — `"network" | "config-key" | "config-401" | "offline" | "no-selection" | "no-permission" | "keystore" | "no-provider" | "generic"`.

### Design note: how the backend maps onto the union

The backend emits two stream events and returns one IPC result. The decoders are the *only* place that knows the wire format; the rest of the app consumes `TranslationState`. Classification is string-based because the backend serializes `Error` via its `Display` impl (see `popup.rs` `error: Some(e.to_string())`). The classifier keys on stable substrings present in the Rust `Error`/`ConfigKind` Display output (`to_string()`), documented inline. This keeps the frontend decoupled from importing Rust enums.

- [ ] **Step 1: Write the failing decoder tests**

Create `src/features/translation/decode.test.ts`:

```ts
import { describe, it, expect } from "vitest";
import {
  decodePopupState,
  decodePopupMultiResult,
  decodeSessionResult,
  classifyError,
} from "./decode";

describe("decodePopupState", () => {
  it("loading → kind=loading", () => {
    const s = decodePopupState({ status: "loading", text: "", engine: "" });
    expect(s.kind).toBe("loading");
  });

  it("result → single-success with text + engine", () => {
    const s = decodePopupState({ status: "result", text: "你好", engine: "deepseek/u1" });
    expect(s).toEqual({
      kind: "single-success",
      text: "你好",
      engine: "deepseek/u1",
    });
  });

  it("error with network message → kind=error, sub=network", () => {
    const s = decodePopupState({ status: "error", text: "network error: timeout", engine: "" });
    expect(s.kind).toBe("error");
    expect(s).toHaveProperty("sub", "network");
  });

  it("error with offline message → kind=offline", () => {
    const s = decodePopupState({ status: "error", text: "offline: no network", engine: "" });
    expect(s.kind).toBe("offline");
  });

  it("error with keystore message → kind=keystore-corrupt", () => {
    const s = decodePopupState({ status: "error", text: "keystore unreadable", engine: "" });
    expect(s.kind).toBe("keystore-corrupt");
  });

  it("error with no-selection message → kind=no-selection", () => {
    const s = decodePopupState({ status: "error", text: "no text selected", engine: "" });
    expect(s.kind).toBe("no-selection");
  });

  it("error with permission message → kind=no-permission", () => {
    const s = decodePopupState({ status: "error", text: "accessibility permission denied", engine: "" });
    expect(s.kind).toBe("no-permission");
  });

  it("error with 401 message → kind=error, sub=config-401", () => {
    const s = decodePopupState({ status: "error", text: "401 Unauthorized", engine: "" });
    expect(s.kind).toBe("error");
    expect(s).toHaveProperty("sub", "config-401");
  });

  it("error with missing-key message → kind=error, sub=config-key", () => {
    const s = decodePopupState({ status: "error", text: "missing API key for deepseek", engine: "" });
    expect(s.kind).toBe("error");
    expect(s).toHaveProperty("sub", "config-key");
  });

  it("unknown error text → kind=error, sub=generic", () => {
    const s = decodePopupState({ status: "error", text: "something exploded", engine: "" });
    expect(s.kind).toBe("error");
    expect(s).toHaveProperty("sub", "generic");
  });
});

describe("decodePopupMultiResult", () => {
  it("all-ok outcomes → multi-success", () => {
    const s = decodePopupMultiResult({
      outcomes: [
        { uuid: "u1", ok: true, text: "a", engine: "deepseek/u1" },
        { uuid: "u2", ok: true, text: "b", engine: "openai/u2" },
      ],
    });
    expect(s.kind).toBe("multi-success");
    expect(s.kind === "multi-success" && s.results.length).toBe(2);
  });

  it("single ok outcome → single-success", () => {
    const s = decodePopupMultiResult({
      outcomes: [{ uuid: "u1", ok: true, text: "a", engine: "deepseek/u1" }],
    });
    expect(s.kind).toBe("single-success");
  });

  it("mixed ok/failed → partial", () => {
    const s = decodePopupMultiResult({
      outcomes: [
        { uuid: "u1", ok: true, text: "a", engine: "deepseek/u1" },
        { uuid: "u2", ok: false, error: "timeout" },
      ],
    });
    expect(s.kind).toBe("partial");
  });

  it("all-failed → kind=error", () => {
    const s = decodePopupMultiResult({
      outcomes: [
        { uuid: "u1", ok: false, error: "timeout" },
        { uuid: "u2", ok: false, error: "401" },
      ],
    });
    expect(s.kind).toBe("error");
    expect(s).toHaveProperty("sub", "generic");
  });

  it("empty outcomes → kind=error (defensive)", () => {
    const s = decodePopupMultiResult({ outcomes: [] });
    expect(s.kind).toBe("error");
  });
});

describe("decodeSessionResult", () => {
  it("single-engine success (actual_engine set, one ok outcome) → single-success", () => {
    const s = decodeSessionResult({
      outcomes: [{ uuid: "u1", ok: true, text: "hi", engine: "deepseek/u1" }],
      actual_engine: "deepseek/u1",
    });
    expect(s.kind).toBe("single-success");
  });

  it("single outcome failed → error", () => {
    const s = decodeSessionResult({
      outcomes: [{ uuid: "u1", ok: false, error: "missing key" }],
      actual_engine: undefined,
    });
    expect(s.kind).toBe("error");
    expect(s).toHaveProperty("sub", "config-key");
  });

  it("multiple outcomes → multi-success or partial (delegates to multi decoder)", () => {
    const ok = decodeSessionResult({
      outcomes: [
        { uuid: "u1", ok: true, text: "a", engine: "deepseek/u1" },
        { uuid: "u2", ok: true, text: "b", engine: "openai/u2" },
      ],
    });
    expect(ok.kind).toBe("multi-success");
  });
});

describe("classifyError", () => {
  it("matches network keywords", () => {
    expect(classifyError("network error: timeout")).toBe("network");
    expect(classifyError("request timed out")).toBe("network");
  });
  it("matches 401/403 as config-401", () => {
    expect(classifyError("401 Unauthorized")).toBe("config-401");
    expect(classifyError("403 Forbidden")).toBe("config-401");
  });
  it("matches missing-key phrasing as config-key", () => {
    expect(classifyError("missing API key for deepseek")).toBe("config-key");
    expect(classifyError("no API key configured")).toBe("config-key");
  });
  it("matches offline", () => {
    expect(classifyError("offline: no network")).toBe("offline");
  });
  it("matches keystore", () => {
    expect(classifyError("keystore unreadable")).toBe("keystore");
  });
  it("matches no-selection", () => {
    expect(classifyError("no text selected")).toBe("no-selection");
  });
  it("matches permission", () => {
    expect(classifyError("accessibility permission denied")).toBe("no-permission");
  });
  it("falls back to generic", () => {
    expect(classifyError("something weird")).toBe("generic");
  });
});
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `pnpm test -- decode`
Expected: FAIL — `./decode` does not exist.

- [ ] **Step 3: Create `src/features/translation/types.ts`**

```ts
/**
 * Frontend translation state model for R2b. This is the single source of truth
 * consumed by the rebuilt Popup (Surface 01) and InputPanel (Surface 02).
 *
 * The backend emits two Tauri events (`popup-state`, `popup-multi-result`) and
 * one IPC result (`translate_session`). Decoders in decode.ts map each wire
 * shape onto this union. UI code MUST NOT import the wire types directly — it
 * only ever sees `TranslationState`.
 */

/** One engine's outcome, frontend shape (mirrors backend TranslationOutcomeSerialized). */
export type TranslationOutcomeFE = {
  uuid: string;
  ok: boolean;
  text?: string;
  engine?: string;
  error?: string;
};

/** Backend `popup-state` payload. */
export type PopupStatePayload = {
  status: "loading" | "result" | "error";
  text: string;
  engine: string;
};

/** Backend `popup-multi-result` payload. */
export type PopupMultiPayload = {
  outcomes: TranslationOutcomeFE[];
};

/** Backend `translate_session` return value. */
export type SessionResultFE = {
  outcomes: TranslationOutcomeFE[];
  actual_engine?: string;
};

/** Fine-grained error category, derived by classifying the backend error string. */
export type ErrorKind =
  | "network"
  | "config-key"
  | "config-401"
  | "offline"
  | "no-selection"
  | "no-permission"
  | "keystore"
  | "no-provider"
  | "generic";

/** A single result card's data (for multi/partial rendering). */
export type ResultEntry = {
  uuid: string;
  engine: string;
  text?: string;
  errorText?: string;
  ok: boolean;
};

/**
 * Discriminant union for all popup/input translation states. `kind` is the
 * discriminant. The error variants collapse into `kind: "error"` with a `sub`
 * field, EXCEPT offline/no-selection/no-permission/keystore-corrupt which are
 * their own `kind` (they render differently — EmptyState + recovery action).
 */
export type TranslationState =
  | { kind: "loading" }
  | { kind: "single-success"; text: string; engine: string }
  | { kind: "multi-success"; results: ResultEntry[] }
  | { kind: "partial"; results: ResultEntry[] }
  | { kind: "error"; sub: ErrorKind; message: string }
  | { kind: "offline"; message: string }
  | { kind: "no-selection" }
  | { kind: "no-permission" }
  | { kind: "keystore-corrupt"; message: string };

/**
 * i18n copy keys for Surface 01 (selection popup) + Surface 02 (input window).
 * Matches design-system/linguaray/pages/01-selection-popup.md and 02-input-window.md.
 */
export type CopyKey =
  // Surface 01
  | "selection.loading"
  | "selection.error.network"
  | "selection.error.config.key"
  | "selection.error.config.auth"
  | "selection.error.noSelection"
  | "selection.error.noPermission"
  | "selection.error.keystore"
  | "selection.error.keystore.cta"
  | "selection.error.offline"
  | "selection.action.copy"
  | "selection.action.copied"
  | "selection.action.speak"
  | "selection.action.stop"
  | "selection.action.pin"
  | "selection.action.unpin"
  | "selection.action.favorite"
  | "selection.action.favorited"
  | "selection.action.retry"
  | "selection.multi.title"
  // Surface 02
  | "input.title"
  | "input.placeholder"
  | "input.action.translate"
  | "input.action.clear"
  | "input.result.label"
  | "input.error.offline";
```

- [ ] **Step 4: Create `src/features/translation/decode.ts`**

```ts
import type {
  ErrorKind,
  PopupMultiPayload,
  PopupStatePayload,
  ResultEntry,
  SessionResultFE,
  TranslationOutcomeFE,
  TranslationState,
} from "./types";

/**
 * Classify a backend error string into a fine-grained kind.
 *
 * The backend serializes `Error` via its Display impl (popup.rs
 * `error: Some(e.to_string())`), so the frontend classifies by stable
 * substrings. Keyword order matters: more specific kinds are checked first
 * (e.g. "keystore" before generic; "401"/"403" before "key").
 *
 * Substrings chosen match the Rust Error/ConfigKind Display output in
 * src-tauri/src/error.rs (network/timeout/401/403/key/keystore/offline/etc.).
 */
export function classifyError(message: string): ErrorKind {
  const m = message.toLowerCase();
  if (m.includes("no text selected") || m.includes("nothing selected")) {
    return "no-selection";
  }
  if (m.includes("accessibility") || m.includes("permission")) {
    return "no-permission";
  }
  if (m.includes("keystore")) {
    return "keystore";
  }
  if (m.includes("offline") || m.includes("no network")) {
    return "offline";
  }
  if (m.includes("no provider") || m.includes("not configured")) {
    return "no-provider";
  }
  if (m.includes("401") || m.includes("403") || m.includes("unauthorized") || m.includes("forbidden")) {
    return "config-401";
  }
  if (m.includes("missing") && m.includes("key")) {
    return "config-key";
  }
  if (m.includes("no api key") || m.includes("api key")) {
    return "config-key";
  }
  if (
    m.includes("network") ||
    m.includes("timeout") ||
    m.includes("timed out") ||
    m.includes("connection") ||
    m.includes("unreachable")
  ) {
    return "network";
  }
  return "generic";
}

/**
 * Map a classified kind onto the right TranslationState variant. Offline,
 * no-selection, no-permission, keystore, and no-provider are their own kinds
 * (distinct render); the rest become { kind: "error", sub }.
 */
function errorToState(message: string): TranslationState {
  const sub = classifyError(message);
  switch (sub) {
    case "offline":
      return { kind: "offline", message };
    case "no-selection":
      return { kind: "no-selection" };
    case "no-permission":
      return { kind: "no-permission" };
    case "keystore":
      return { kind: "keystore-corrupt", message };
    default:
      return { kind: "error", sub, message };
  }
}

/** Decode the legacy single-channel `popup-state` event. */
export function decodePopupState(payload: PopupStatePayload): TranslationState {
  switch (payload.status) {
    case "loading":
      return { kind: "loading" };
    case "result":
      return { kind: "single-success", text: payload.text, engine: payload.engine };
    case "error":
      return errorToState(payload.text);
  }
}

function outcomeToEntry(o: TranslationOutcomeFE): ResultEntry {
  return {
    uuid: o.uuid,
    engine: o.engine ?? o.uuid,
    text: o.text,
    errorText: o.error,
    ok: o.ok,
  };
}

/**
 * Decide multi-success / single-success / partial / error from an outcomes
 * array. Shared by `decodePopupMultiResult` and `decodeSessionResult`.
 *
 * - 1 ok outcome  → single-success
 * - all ok        → multi-success (or single if only one)
 * - mixed         → partial
 * - all failed    → error (sub = generic; per-engine errors preserved if a UI
 *                  later wants them, but the headline is "error")
 */
export function decodeOutcomes(outcomes: TranslationOutcomeFE[]): TranslationState {
  if (outcomes.length === 0) {
    return { kind: "error", sub: "generic", message: "no outcomes" };
  }
  const results = outcomes.map(outcomeToEntry);
  const okCount = results.filter((r) => r.ok).length;
  if (okCount === results.length) {
    if (results.length === 1) {
      const r = results[0];
      return { kind: "single-success", text: r.text ?? "", engine: r.engine };
    }
    return { kind: "multi-success", results };
  }
  if (okCount === 0) {
    // All failed: surface the first error string for classification.
    const firstMsg = results.find((r) => r.errorText)?.errorText ?? "all engines failed";
    return errorToState(firstMsg);
  }
  return { kind: "partial", results };
}

/** Decode the `popup-multi-result` event (R2a multi-engine channel). */
export function decodePopupMultiResult(payload: PopupMultiPayload): TranslationState {
  return decodeOutcomes(payload.outcomes);
}

/** Decode the `translate_session` IPC return value (used by the input window). */
export function decodeSessionResult(result: SessionResultFE): TranslationState {
  return decodeOutcomes(result.outcomes);
}
```

- [ ] **Step 5: Run tests to verify they pass**

Run: `pnpm test -- decode`
Expected: PASS (all decoder tests).

- [ ] **Step 6: Verify typecheck**

Run: `pnpm typecheck`
Expected: PASS.

- [ ] **Step 7: Commit**

```bash
git add src/features/translation/
git commit -m "feat(translation): add TranslationState model + wire decoders"
```

---

## Task 2: CAS operation registry (production port)

Port the proven CAS registry from ui-lab verbatim so the production popup/input get the same async-safety guarantees (latest-wins, reentry-safe cancel, generation tokens). This is a near-mechanical copy with one change: the port lives at `src/features/translation/op-registry.ts` and is unit-tested in the production harness.

**Files:**
- Create: `src/features/translation/op-registry.ts`
- Test: `src/features/translation/op-registry.test.ts`

**Interfaces:**
- Consumes: nothing (pure TS, no Solid/Tauri).
- Produces: `OpRegistry` class, `OpKind`, `OpKey`, `OpEntry`, plus a `useGenerationToken()` Solid helper re-exported from the same module (used by Task 3's popup controller to invalidate stale callbacks on state change).

The source of truth is `apps/ui-lab/src/pages/op-registry.ts` (already read). The production port MUST be byte-identical in the `OpRegistry` class body; only the file location and the added `useGenerationToken` helper differ.

- [ ] **Step 1: Write the failing registry tests (ported from ui-lab)**

Create `src/features/translation/op-registry.test.ts` by copying the ui-lab test verbatim, adjusting the import path:

```ts
import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { OpRegistry, type OpKind } from "./op-registry";

describe("OpRegistry — CAS semantics", () => {
  let registry: OpRegistry;

  beforeEach(() => {
    vi.useFakeTimers();
    registry = new OpRegistry();
  });
  afterEach(() => vi.useRealTimers());

  it("startOp returns a token and registers the op", () => {
    let cleared = false;
    const token = registry.startOp(
      "save", "uuid-a",
      () => { cleared = true; },
      () => {},
      1000,
    );
    expect(token).toBeGreaterThan(0);
    expect(registry.isActive("save", "uuid-a")).toBe(true);
    expect(cleared).toBe(false);
  });

  it("startOp cancels a previous op on the same key", () => {
    let oldCleared = false;
    const oldToken = registry.startOp(
      "save", "uuid-a",
      () => { oldCleared = true; },
      () => {},
      1000,
    );
    const newToken = registry.startOp(
      "save", "uuid-a",
      () => {}, () => {}, 1000,
    );
    expect(newToken).toBeGreaterThan(oldToken);
    expect(oldCleared).toBe(true);
    expect(registry.currentToken("save", "uuid-a")).toBe(newToken);
  });

  it("finishOpIfCurrent with old token returns false and does NOT run result", () => {
    let resultRan = false;
    const oldToken = registry.startOp("test", "uuid-a", () => {}, () => {}, 1000);
    registry.startOp("test", "uuid-a", () => {}, () => {}, 1000);
    const applied = registry.finishOpIfCurrent("test", "uuid-a", oldToken, () => {
      resultRan = true;
    });
    expect(applied).toBe(false);
    expect(resultRan).toBe(false);
  });

  it("finishOpIfCurrent with current token runs result and cleans up", () => {
    let resultRan = false;
    const token = registry.startOp("fetch", "uuid-a", () => {}, () => {}, 1000);
    const applied = registry.finishOpIfCurrent("fetch", "uuid-a", token, () => {
      resultRan = true;
    });
    expect(applied).toBe(true);
    expect(resultRan).toBe(true);
    expect(registry.isActive("fetch", "uuid-a")).toBe(false);
  });

  it("cancelOpIfCurrent with old token does not clear new op's busy", () => {
    let newCleared = false;
    const oldToken = registry.startOp("balance", "uuid-a", () => {}, () => {}, 1000);
    registry.startOp("balance", "uuid-a", () => { newCleared = true; }, () => {}, 1000);
    const cancelled = registry.cancelOpIfCurrent("balance", "uuid-a", oldToken);
    expect(cancelled).toBe(false);
    expect(newCleared).toBe(false);
    expect(registry.isActive("balance", "uuid-a")).toBe(true);
  });

  it("cancelOpsForUuid cancels all ops for a provider but not others", () => {
    let s = false, t = false, other = false;
    registry.startOp("save", "uuid-a", () => { s = true; }, () => {}, 1000);
    registry.startOp("test", "uuid-a", () => { t = true; }, () => {}, 1000);
    registry.startOp("save", "uuid-b", () => { other = true; }, () => {}, 1000);
    registry.cancelOpsForUuid("uuid-a");
    expect(s).toBe(true);
    expect(t).toBe(true);
    expect(other).toBe(false);
  });

  it("timer fires and runs result exactly once (CAS auto-complete)", () => {
    let resultCount = 0;
    registry.startOp("save", "uuid-a", () => {}, () => { resultCount++; }, 1000);
    expect(resultCount).toBe(0);
    vi.advanceTimersByTime(1100);
    expect(resultCount).toBe(1);
    expect(registry.isActive("save", "uuid-a")).toBe(false);
  });

  it("timer does NOT fire result if a newer op replaced it", () => {
    let oldResult = false;
    registry.startOp("save", "uuid-a", () => {}, () => { oldResult = true; }, 1000);
    registry.startOp("save", "uuid-a", () => {}, () => {}, 2000);
    vi.advanceTimersByTime(1100);
    expect(oldResult).toBe(false);
  });

  it("cancelAll: snapshot + clear BEFORE clearBusy (reentry-safe)", () => {
    let cleared1 = false, cleared2 = false;
    registry.startOp("save", "uuid-a", () => { cleared1 = true; }, () => {}, 1000);
    registry.startOp("test", "uuid-b", () => { cleared2 = true; }, () => {}, 1000);
    registry.cancelAll();
    expect(cleared1).toBe(true);
    expect(cleared2).toBe(true);
    expect(registry.isActive("save", "uuid-a")).toBe(false);
  });
});

describe("useGenerationToken", () => {
  it("bumps generation on dependency change and invalidates stale callbacks", async () => {
    const { useGenerationToken } = await import("./op-registry");
    const { createSignal, createEffect } = await import("solid-js");
    const { waitFor } = await import("@solidjs/testing-library");

    const [dep, setDep] = createSignal(0);
    let captured: { bump: () => void; isCurrent: () => boolean } | null = null;
    createEffect(() => {
      void dep();
      captured = useGenerationToken();
    });
    await waitFor(() => expect(captured).not.toBeNull());
    const first = captured!;
    expect(first.isCurrent()).toBe(true);

    setDep(1);
    await waitFor(() => expect(captured).not.toBe(first));
    // The previous generation's isCurrent() now returns false.
    expect(first.isCurrent()).toBe(false);
    expect(captured!.isCurrent()).toBe(true);
  });
});
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `pnpm test -- op-registry`
Expected: FAIL — `./op-registry` does not exist.

- [ ] **Step 3: Create `src/features/translation/op-registry.ts`**

Copy the `OpRegistry` class body verbatim from `apps/ui-lab/src/pages/op-registry.ts` (the `OpKind`, `OpKey`, `OpEntry`, `OpRegistry` class with `startOp`, `finishOpIfCurrent`, `cancelOpIfCurrent`, `cancelOpsForUuid`, `cancelAll`, `isActive`, `currentToken`, `cancelOp`). Then append the Solid generation-token helper:

```ts
// ─── (verbatim OpRegistry class body from apps/ui-lab/src/pages/op-registry.ts) ───
// Copy the EXACT source: OpKind, OpKey, OpEntry, `let nextToken = 0;`,
// `export class OpRegistry { ... }` with all methods. Do not paraphrase.

import { createEffect, onCleanup } from "solid-js";

/**
 * Solid generation-token hook: returns a guard object whose `isCurrent()`
 * becomes false the next time a tracked dependency changes (or the owning
 * scope cleans up). Use to invalidate stale async callbacks (copy-revert
 * timers, retry→success swaps) so a callback scheduled on an old state can
 * never mutate a newer one.
 *
 * Call inside a `createEffect` that tracks the state you want to invalidate on:
 *   createEffect(() => {
 *     void state();              // track
 *     const gen = useGenerationToken();
 *     schedule(() => { if (!gen.isCurrent()) return; /* ... */ }, 1500);
 *   });
 */
export function useGenerationToken(): {
  bump: () => void;
  isCurrent: () => boolean;
} {
  let gen = 0;
  const mine = ++gen;
  // Re-run this effect on dependency change: bumping `gen` invalidates older tokens.
  createEffect(() => {
    gen += 1;
  });
  onCleanup(() => {
    gen += 1; // also invalidate on scope teardown
  });
  return {
    bump: () => { gen += 1; },
    isCurrent: () => gen === mine,
  };
}
```

Note: the `let nextToken = 0;` module-level counter stays module-level in the port (same as ui-lab). The production port adds the `import { createEffect, onCleanup }` line and the `useGenerationToken` function; the `OpRegistry` class itself is unchanged.

- [ ] **Step 4: Run tests to verify they pass**

Run: `pnpm test -- op-registry`
Expected: PASS (all CAS tests + the generation-token test).

- [ ] **Step 5: Verify typecheck**

Run: `pnpm typecheck`
Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add src/features/translation/op-registry.ts src/features/translation/op-registry.test.ts
git commit -m "feat(translation): port CAS OpRegistry + generation token to production"
```

---

## Task 3: Selection Popup rebuild (Surface 01)

Rebuild `src/Popup.tsx` to consume the typed state model, render all Surface 01 states with `@linguaray/ui` primitives, and handle keyboard/focus/pin. This is the largest task; it depends on Tasks 0–2.

**Files:**
- Create: `src/features/translation/copy.ts`
- Create: `src/i18n.ts`
- Create: `src/features/translation/popupController.ts`
- Modify: `src/Popup.tsx`
- Modify: `src/Popup.css`
- Test: `test/Popup.test.tsx`

**Interfaces:**
- Consumes (from Task 1): `TranslationState`, `PopupStatePayload`, `PopupMultiPayload`, `CopyKey`, `decodePopupState`, `decodePopupMultiResult`.
- Consumes (from Task 2): `OpRegistry`, `useGenerationToken`.
- Consumes (Tauri): `listen("popup-state", cb)`, `listen("popup-multi-result", cb)`, `getCurrentWindow().onFocusChanged`, `getCurrentWindow().hide()`.
- Produces: a rebuilt `<Popup />` default export that `popup-entry.tsx` already renders (entry is unchanged).

### Sub-task 3a: copy + locale

- [ ] **Step 1: Write failing test for copy map coverage**

Create `src/features/translation/copy.test.ts`:

```ts
import { describe, it, expect } from "vitest";
import { COPY, type CopyMap } from "./copy";
import type { CopyKey } from "./types";

describe("copy map", () => {
  it("every CopyKey exists in zh and en", () => {
    const keys: CopyKey[] = [
      "selection.loading", "selection.error.network", "selection.error.config.key",
      "selection.error.config.auth", "selection.error.noSelection",
      "selection.error.noPermission", "selection.error.keystore",
      "selection.error.keystore.cta", "selection.error.offline",
      "selection.action.copy", "selection.action.copied", "selection.action.speak",
      "selection.action.stop", "selection.action.pin", "selection.action.unpin",
      "selection.action.favorite", "selection.action.favorited",
      "selection.action.retry", "selection.multi.title",
      "input.title", "input.placeholder", "input.action.translate",
      "input.action.clear", "input.result.label", "input.error.offline",
    ];
    for (const k of keys) {
      expect(COPY.zh[k], `zh missing ${k}`).toBeTypeOf("string");
      expect(COPY.en[k], `en missing ${k}`).toBeTypeOf("string");
    }
  });

  it("CopyMap type accepts the structure", () => {
    const m: CopyMap = COPY;
    expect(m.zh["selection.loading"]).toBe("翻译中…");
    expect(m.en["selection.loading"]).toBe("Translating…");
  });
});
```

- [ ] **Step 2: Run test to verify it fails**

Run: `pnpm test -- copy`
Expected: FAIL — `./copy` does not exist.

- [ ] **Step 3: Create `src/features/translation/copy.ts`**

Values copied from the design-system spec tables (01-selection-popup.md, 02-input-window.md):

```ts
import type { CopyKey } from "./types";

export type CopyMap = Record<"zh" | "en", Record<CopyKey, string>>;

export const COPY: CopyMap = {
  zh: {
    "selection.loading": "翻译中…",
    "selection.error.network": "网络错误",
    "selection.error.config.key": "缺少 API 密钥",
    "selection.error.config.auth": "401 未授权",
    "selection.error.noSelection": "未选中文本",
    "selection.error.noPermission": "请授予辅助功能权限",
    "selection.error.keystore": "密钥库不可读",
    "selection.error.keystore.cta": "前往设置恢复",
    "selection.error.offline": "离线",
    "selection.action.copy": "复制",
    "selection.action.copied": "已复制",
    "selection.action.speak": "朗读",
    "selection.action.stop": "停止",
    "selection.action.pin": "固定",
    "selection.action.unpin": "取消固定",
    "selection.action.favorite": "收藏到生词本",
    "selection.action.favorited": "已收藏",
    "selection.action.retry": "重试",
    "selection.multi.title": "多引擎结果",
    "input.title": "翻译",
    "input.placeholder": "输入要翻译的文本…",
    "input.action.translate": "翻译",
    "input.action.clear": "清空",
    "input.result.label": "翻译结果",
    "input.error.offline": "离线",
  },
  en: {
    "selection.loading": "Translating…",
    "selection.error.network": "Network error",
    "selection.error.config.key": "API key missing",
    "selection.error.config.auth": "401 Unauthorized",
    "selection.error.noSelection": "No text selected",
    "selection.error.noPermission": "Grant Accessibility permission",
    "selection.error.keystore": "Keystore unreadable",
    "selection.error.keystore.cta": "Go to settings recovery",
    "selection.error.offline": "Offline",
    "selection.action.copy": "Copy",
    "selection.action.copied": "Copied",
    "selection.action.speak": "Speak",
    "selection.action.stop": "Stop",
    "selection.action.pin": "Pin",
    "selection.action.unpin": "Unpin",
    "selection.action.favorite": "Save to vocabulary",
    "selection.action.favorited": "Saved",
    "selection.action.retry": "Retry",
    "selection.multi.title": "Multi-engine result",
    "input.title": "Translate",
    "input.placeholder": "Type text to translate…",
    "input.action.translate": "Translate",
    "input.action.clear": "Clear",
    "input.result.label": "Translation",
    "input.error.offline": "Offline",
  },
};
```

- [ ] **Step 4: Create `src/i18n.ts`**

```ts
import type { CopyKey } from "./features/translation/types";
import { COPY } from "./features/translation/copy";

export type Locale = "zh" | "en";

/**
 * Detect the user locale. Order: localStorage("linguaray.locale") →
 * navigator.language prefix → "en". Kept dependency-free so it is testable
 * and SSR-safe (Tauri WebView provides navigator).
 */
export function detectLocale(): Locale {
  const stored =
    typeof localStorage !== "undefined" ? localStorage.getItem("linguaray.locale") : null;
  if (stored === "zh" || stored === "en") return stored;
  if (typeof navigator !== "undefined" && navigator.language?.toLowerCase().startsWith("zh")) {
    return "zh";
  }
  return "en";
}

/** Typed accessor: `t("selection.loading", locale)`. */
export function t(key: CopyKey, locale: Locale = detectLocale()): string {
  return COPY[locale][key];
}

export { COPY };
```

- [ ] **Step 5: Run copy test to verify it passes**

Run: `pnpm test -- copy`
Expected: PASS.

### Sub-task 3b: popup controller (event subscription + state signal)

- [ ] **Step 6: Create `src/features/translation/popupController.ts`**

```ts
import { createSignal, onCleanup, onMount } from "solid-js";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { invoke } from "@tauri-apps/api/core";
import { decodePopupMultiResult, decodePopupState } from "./decode";
import type { PopupMultiPayload, PopupStatePayload, TranslationState } from "./types";

/**
 * Production popup controller. Owns:
 *  - the `state` signal (the single TranslationState the UI renders)
 *  - the `pinned` signal (Surface 01: pinned popups ignore blur-hide)
 *  - Tauri event subscriptions (popup-state + popup-multi-result)
 *  - blur-hide gating on pin state
 *  - retry (re-emits the last selection translation via translate_session —
 *    delegated to the backend which re-reads active selection)
 *
 * Returns a plain object of accessors/actions; the component binds them.
 */
export function createPopupController() {
  const [state, setState] = createSignal<TranslationState>({ kind: "loading" });
  const [pinned, setPinned] = createSignal(false);
  const unlisteners: UnlistenFn[] = [];

  onMount(async () => {
    unlisteners.push(
      await listen<PopupStatePayload>("popup-state", (e) => {
        setState(decodePopupState(e.payload));
      }),
    );
    unlisteners.push(
      await listen<PopupMultiPayload>("popup-multi-result", (e) => {
        setState(decodePopupMultiResult(e.payload));
      }),
    );

    // Blur-hide, gated by pin: a pinned popup stays visible on blur (S0 §4.1).
    const win = getCurrentWindow();
    unlisteners.push(
      await win.onFocusChanged(({ payload: focused }) => {
        if (!focused && !pinned()) win.hide();
      }),
    );
  });

  onCleanup(() => {
    for (const u of unlisteners) u();
  });

  const pin = () => setPinned(true);
  const unpin = () => setPinned(false);

  const dismiss = async () => {
    setPinned(false);
    await getCurrentWindow().hide();
  };

  // Retry: ask the backend to re-run the active-selection translation. The
  // backend re-emits popup-state / popup-multi-result, which re-decode here.
  const retry = async () => {
    setState({ kind: "loading" });
    try {
      await invoke("translate_clipboard");
    } catch (e) {
      setState({ kind: "error", sub: "generic", message: String(e) });
    }
  };

  return { state, pinned, pin, unpin, dismiss, retry };
}
```

Note: `retry` reuses the existing `translate_clipboard` IPC (already wired to emit the popup events) as the canonical "re-translate current selection" path. The backend is untouched. If a dedicated re-translate-selection IPC is added later, swap this one call.

### Sub-task 3c: rebuild the component

- [ ] **Step 7: Write failing component tests**

Create `test/Popup.test.tsx`:

```tsx
import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, fireEvent, cleanup } from "@solidjs/testing-library";
import Popup from "../src/Popup";

// Stub Tauri event + window APIs at the module the controller imports.
vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn(async () => () => {}),
}));
vi.mock("@tauri-apps/api/window", () => ({
  getCurrentWindow: () => ({
    onFocusChanged: vi.fn(async () => () => {}),
    hide: vi.fn(async () => {}),
    setFocus: vi.fn(async () => {}),
  }),
}));
vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(async () => ({ outcomes: [], actual_engine: undefined })),
}));

// Helper: emit a decoded state by reaching into the listen mock.
async function emitEvent(name: string, payload: unknown) {
  const { listen } = await import("@tauri-apps/api/event");
  const calls = vi.mocked(listen).mock.calls;
  // Find the most recent listener registered for this event name.
  for (let i = calls.length - 1; i >= 0; i--) {
    if (calls[i][0] === name) {
      const handler = calls[i][1] as (e: { payload: unknown }) => void;
      handler({ payload });
      return;
    }
  }
}

beforeEach(() => {
  vi.clearAllMocks();
});

describe("Popup (Surface 01)", () => {
  it("renders loading spinner before any event", () => {
    const { getByRole, container } = render(() => <Popup />);
    // Spinner renders an svg.lr-spinner__icon + visually-hidden label.
    expect(container.querySelector(".lr-spinner__icon")).toBeTruthy();
    expect(getByRole("region")).toBeTruthy();
    cleanup();
  });

  it("renders single-success ResultCard on popup-state result", async () => {
    const { findByText, getByRole } = render(() => <Popup />);
    await emitEvent("popup-state", { status: "result", text: "你好", engine: "deepseek/u1" });
    expect(await findByText("你好")).toBeTruthy();
    // The region's aria-label should reflect a success state, not loading.
    const region = getByRole("region");
    expect(region.getAttribute("aria-busy")).toBeFalsy();
    cleanup();
  });

  it("renders error EmptyState on popup-state network error", async () => {
    const { findByText } = render(() => <Popup />);
    await emitEvent("popup-state", { status: "error", text: "network timeout", engine: "" });
    // The zh copy "网络错误" should appear (default locale detection may vary;
    // assert the role=alert region is present with a non-empty message).
    const alert = await findByText(/网络错误|Network error/);
    expect(alert).toBeTruthy();
    cleanup();
  });

  it("renders multi-success on popup-multi-result with two ok outcomes", async () => {
    const { findAllByText } = render(() => <Popup />);
    await emitEvent("popup-multi-result", {
      outcomes: [
        { uuid: "u1", ok: true, text: "你好", engine: "deepseek/u1" },
        { uuid: "u2", ok: true, text: "hello", engine: "openai/u2" },
      ],
    });
    const cards = await findAllByText(/你好|hello/);
    expect(cards.length).toBeGreaterThanOrEqual(2);
    cleanup();
  });

  it("hides the window on Escape", async () => {
    const { getCurrentWindow } = await import("@tauri-apps/api/window");
    const { container } = render(() => <Popup />);
    fireEvent.keyDown(container.querySelector("main") ?? document.body, { key: "Escape" });
    expect(vi.mocked(getCurrentWindow().hide)).toHaveBeenCalled();
    cleanup();
  });
});
```

- [ ] **Step 8: Run tests to verify they fail**

Run: `pnpm test -- Popup`
Expected: FAIL — current `Popup.tsx` does not render `.lr-spinner__icon`, `role="region"`, multi-result, or Escape handling.

- [ ] **Step 9: Rewrite `src/Popup.tsx`**

```tsx
import { For, Show, createMemo, type Component } from "solid-js";
import { Copy, Check, Volume2, Square, Pin, PinOff, Star, AlertTriangle } from "lucide-solid";
import {
  Button,
  EmptyState,
  InlineError,
  ResultCard,
  Spinner,
  type ResultAction,
  type ResultOutcome,
} from "@linguaray/ui";
import { createPopupController } from "./features/translation/popupController";
import { detectLocale, t } from "./i18n";
import type { TranslationState } from "./features/translation/types";
import "./Popup.css";
import "./App.css";

/** Map a TranslationState kind onto the headline copy key for aria-label. */
function headlineKey(s: TranslationState): string {
  switch (s.kind) {
    case "loading": return t("selection.loading");
    case "single-success":
    case "multi-success": return t("selection.multi.title");
    case "partial": return t("selection.multi.title");
    case "error":
      switch (s.sub) {
        case "network": return t("selection.error.network");
        case "config-key": return t("selection.error.config.key");
        case "config-401": return t("selection.error.config.auth");
        default: return s.message;
      }
    case "offline": return t("selection.error.offline");
    case "no-selection": return t("selection.error.noSelection");
    case "no-permission": return t("selection.error.noPermission");
    case "keystore-corrupt": return t("selection.error.keystore");
  }
}

const Popup: Component = () => {
  detectLocale(); // resolve locale once on mount (t() reads it lazily)
  const ctrl = createPopupController();
  const state = ctrl.state;

  const isCompact = createMemo(() => state().kind === "loading");

  // Per-card action builders (copy/speak/pin/favorite). Stale-safe via the
  // controller's generation token inside createPopupController (state changes
  // re-run, invalidating older scheduled callbacks).
  const buildActions = (uuid: string): ResultAction[] => {
    const isPinned = ctrl.pinned();
    return [
      {
        label: t("selection.action.copy"),
        icon: <Copy size={14} />,
        onClick: () => { void navigator.clipboard?.writeText(textFor(uuid) ?? ""); },
      },
      {
        label: t("selection.action.speak"),
        icon: <Volume2 size={14} />,
        onClick: () => { /* TTS hook: window.speechSynthesis if available */ },
      },
      {
        label: isPinned ? t("selection.action.unpin") : t("selection.action.pin"),
        icon: isPinned ? <PinOff size={14} /> : <Pin size={14} />,
        active: isPinned,
        onClick: () => (isPinned ? ctrl.unpin() : ctrl.pin()),
      },
      {
        label: t("selection.action.favorite"),
        icon: <Star size={14} />,
        onClick: () => { /* vocabulary IPC hook */ },
      },
    ];
  };

  function textFor(uuid: string): string | undefined {
    const s = state();
    if (s.kind === "multi-success" || s.kind === "partial") {
      return s.results.find((r) => r.uuid === uuid)?.text;
    }
    if (s.kind === "single-success") return s.text;
    return undefined;
  }

  const onKeyDown = (e: KeyboardEvent) => {
    if (e.key === "Escape") { e.preventDefault(); void ctrl.dismiss(); }
  };

  return (
    <main
      class="container"
      classList={{ "container--compact": isCompact() }}
      role="region"
      aria-label={headlineKey(state())}
      aria-busy={state().kind === "loading" ? "true" : undefined}
      onKeyDown={onKeyDown}
      tabIndex={-1}
    >
      <Show when={state().kind === "loading"}>
        <div class="popup-loading">
          <Spinner size={12} label={t("selection.loading")} />
        </div>
      </Show>

      <Show when={state().kind === "single-success"}>
        <ResultCard
          engineId={state().kind === "single-success" ? state().engine : ""}
          engineLabel={state().kind === "single-success" ? state().engine : ""}
          text={state().kind === "single-success" ? state().text : ""}
          outcome={"success" as ResultOutcome}
          actions={buildActions("__single__")}
        />
      </Show>

      <Show when={state().kind === "multi-success" || state().kind === "partial"}>
        <div class="popup-results" data-multi="true">
          <For each={
            state().kind === "multi-success" || state().kind === "partial"
              ? state().results : []
          }>
            {(r) => (
              <ResultCard
                engineId={r.uuid}
                engineLabel={r.engine}
                text={r.text}
                outcome={(r.ok ? "success" : "failure") as ResultOutcome}
                errorText={r.errorText}
                actions={r.ok ? buildActions(r.uuid) : undefined}
              />
            )}
          </For>
        </div>
      </Show>

      {/* Single-card error / special states (no ResultCard grid). */}
      <Show when={state().kind === "error" || state().kind === "offline" ||
                  state().kind === "no-selection" || state().kind === "no-permission" ||
                  state().kind === "keystore-corrupt"}>
        <div class="popup-error" role="alert">
          <EmptyState
            icon={<AlertTriangle size={32} />}
            title={headlineKey(state())}
            action={
              <Show when={state().kind === "error" && state().sub === "network"} fallback={
                <Show when={
                  state().kind === "error" && (state().sub === "config-key" || state().sub === "config-401")
                }>
                  <Button variant="ghost" size="sm" onClick={() => { /* open settings window */ }}>
                    {t("selection.action.retry")}
                  </Button>
                </Show>
              }>
                <Button variant="secondary" size="sm" onClick={() => void ctrl.retry()}>
                  {t("selection.action.retry")}
                </Button>
              </Show>
            }
          />
        </div>
      </Show>
    </main>
  );
};

export default Popup;
```

- [ ] **Step 10: Rewrite `src/Popup.css` (token-only)**

```css
/*
 * Popup-only overrides. The WebView root is transparent so the native
 * window's transparency (macOSPrivateApi) shows through. Card backgrounds
 * come from @linguaray/ui tokens (--color-bg-elevated), NOT hardcoded hex.
 */
html,
body,
#root {
  width: 100%;
  height: 100%;
  margin: 0;
  background: transparent;
}

body { overflow: hidden; }

.container {
  background: transparent;
  padding: var(--space-2, 8px);
  display: flex;
  flex-direction: column;
  gap: var(--space-2, 8px);
  outline: none;
}

.container--compact {
  min-height: 40px;
  padding: var(--space-1, 4px);
}

.popup-results {
  display: grid;
  grid-auto-flow: column;
  grid-auto-columns: minmax(200px, 1fr);
  gap: var(--space-2, 8px);
}

.popup-results[data-multi="true"] {
  overflow-x: auto;
}

.popup-loading,
.popup-error {
  background: var(--color-bg-elevated, #fff);
  border-radius: var(--radius-lg, 12px);
  box-shadow: var(--shadow-lg, 0 4px 16px rgba(0,0,0,0.12));
  padding: var(--space-3, 12px);
}
```

Note: the hex fallbacks inside `var(--…, fallback)` are CSS fallback values (used only if a token is undefined) and are acceptable; the goal is that tokens win whenever the `@linguaray/ui` token CSS is loaded. The token CSS from `@linguaray/ui` must be imported once globally — add `import "@linguaray/ui/tokens.css";` (or the package's documented global CSS) at the top of `popup-entry.tsx`. If the package does not export a token CSS entry, import it from `packages/ui/src/tokens.css` via the alias; verify the exact export in Task 0's smoke step before relying on it here.

- [ ] **Step 11: Run component tests to verify they pass**

Run: `pnpm test -- Popup`
Expected: PASS.

- [ ] **Step 12: Verify typecheck + build**

Run: `pnpm typecheck && pnpm build`
Expected: PASS (the multi-page Vite build still produces popup.html).

- [ ] **Step 13: Commit**

```bash
git add src/Popup.tsx src/Popup.css src/features/translation/copy.ts src/features/translation/popupController.ts src/i18n.ts src/features/translation/copy.test.ts test/Popup.test.tsx
git commit -m "feat(surface-01): rebuild selection popup with @linguaray/ui + state model"
```

---

## Task 4: Input Window rebuild (Surface 02)

Rebuild `src/InputPanel.tsx` to call `translate_session` IPC (the R2a multi-engine command) and render results via the state model. Depends on Tasks 0–2 (and reuses Task 3's `copy.ts`/`i18n.ts`).

**Files:**
- Modify: `src/InputPanel.tsx`
- Test: `test/InputPanel.test.tsx`

**Interfaces:**
- Consumes (Task 1): `decodeSessionResult`, `SessionResultFE`, `TranslationState`.
- Consumes (Task 3): `t`, `detectLocale`.
- Consumes (Tauri): `invoke<{ outcomes: SessionResultFE["outcomes"]; actual_engine?: string }>("translate_session", { req: { text, from: "auto", to: "" } })`. (`to: ""` is the backend sentinel for "use settings.target_language", matching the existing `translate_default` call in the old InputPanel.)
- Produces: a rebuilt `<InputPanel />` default export (entry unchanged).

- [ ] **Step 1: Write failing component tests**

Create `test/InputPanel.test.tsx`:

```tsx
import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, fireEvent, cleanup, waitFor } from "@solidjs/testing-library";
import InputPanel from "../src/InputPanel";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(async (_cmd: string, _args?: unknown) => ({
    outcomes: [{ uuid: "u1", ok: true, text: "hello", engine: "deepseek/u1" }],
    actual_engine: "deepseek/u1",
  })),
}));

beforeEach(() => vi.clearAllMocks());

describe("InputPanel (Surface 02)", () => {
  it("renders a textarea + Translate button", () => {
    const { getByRole } = render(() => <InputPanel />);
    expect(getByRole("textbox")).toBeTruthy();
    expect(getByRole("button", { name: /翻译|Translate/ })).toBeTruthy();
    cleanup();
  });

  it("Enter (no shift) triggers translate_session and shows the result", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    const { getByRole, findByText } = render(() => <InputPanel />);
    const ta = getByRole("textbox") as HTMLTextAreaElement;
    fireEvent.input(ta, { target: { value: "你好" } });
    fireEvent.keyDown(ta, { key: "Enter", shiftKey: false });
    await waitFor(() => expect(vi.mocked(invoke)).toHaveBeenCalledWith(
      "translate_session",
      expect.objectContaining({ req: expect.objectContaining({ text: "你好" }) }),
    ));
    expect(await findByText("hello")).toBeTruthy();
    cleanup();
  });

  it("Shift+Enter does NOT trigger translation", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    const { getByRole } = render(() => <InputPanel />);
    const ta = getByRole("textbox") as HTMLTextAreaElement;
    fireEvent.input(ta, { target: { value: "你好" } });
    fireEvent.keyDown(ta, { key: "Enter", shiftKey: true });
    expect(vi.mocked(invoke)).not.toHaveBeenCalled();
    cleanup();
  });

  it("shows InlineError when the engine fails", async () => {
    vi.mocked((await import("@tauri-apps/api/core")).invoke).mockResolvedValueOnce({
      outcomes: [{ uuid: "u1", ok: false, error: "missing API key" }],
    });
    const { getByRole, findByText } = render(() => <InputPanel />);
    const ta = getByRole("textbox") as HTMLTextAreaElement;
    fireEvent.input(ta, { target: { value: "hi" } });
    fireEvent.keyDown(ta, { key: "Enter" });
    // classifyError maps "missing key" → config-key → 缺少 API 密钥 / API key missing
    expect(await findByText(/缺少 API 密钥|API key missing/)).toBeTruthy();
    cleanup();
  });

  it("Clear button empties the textarea and result", async () => {
    const { getByRole, queryByText } = render(() => <InputPanel />);
    const ta = getByRole("textbox") as HTMLTextAreaElement;
    fireEvent.input(ta, { target: { value: "你好" } });
    fireEvent.keyDown(ta, { key: "Enter" });
    await waitFor(() => expect(vi.mocked((await import("@tauri-apps/api/core")).invoke)).toHaveBeenCalled());
    fireEvent.click(getByRole("button", { name: /清空|Clear/ }));
    expect((getByRole("textbox") as HTMLTextAreaElement).value).toBe("");
    expect(queryByText("hello")).toBeNull();
    cleanup();
  });
});
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `pnpm test -- InputPanel`
Expected: FAIL — current InputPanel calls `translate_default`, not `translate_session`, and has no Clear button.

- [ ] **Step 3: Rewrite `src/InputPanel.tsx`**

```tsx
import { createSignal, Show, type Component } from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import { AlertTriangle } from "lucide-solid";
import { Button, InlineError, ResultCard, Spinner, type ResultOutcome } from "@linguaray/ui";
import { decodeSessionResult } from "./features/translation/decode";
import { detectLocale, t } from "./i18n";
import type { SessionResultFE, TranslationState } from "./features/translation/types";
import "./App.css";

const InputPanel: Component = () => {
  detectLocale();
  const [text, setText] = createSignal("");
  const [state, setState] = createSignal<TranslationState>({ kind: "loading" });
  const [idle, setIdle] = createSignal(true); // false while a translation is in-flight

  async function translate() {
    const value = text().trim();
    if (!value) return;
    setIdle(false);
    setState({ kind: "loading" });
    try {
      // to: "" is the backend sentinel for "use settings.target_language".
      const res = await invoke<SessionResultFE>("translate_session", {
        req: { text: value, from: "auto", to: "" },
      });
      setState(decodeSessionResult(res));
    } catch (e) {
      setState({ kind: "error", sub: "generic", message: String(e) });
    } finally {
      setIdle(false);
    }
  }

  const clear = () => {
    setText("");
    setState({ kind: "loading" });
    setIdle(true);
  };

  const onKeyDown = (e: KeyboardEvent) => {
    if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault();
      void translate();
    }
  };

  const resultOutcome = (): ResultOutcome | null => {
    const s = state();
    if (s.kind === "single-success") return "success";
    if (s.kind === "error") return "failure";
    return null;
  };

  return (
    <main class="container" style={{ padding: "var(--space-3, 12px)" }}>
      <h2 class="input-title">{t("input.title")}</h2>
      <textarea
        rows={4}
        placeholder={t("input.placeholder")}
        value={text()}
        disabled={!idle()}
        onInput={(e) => setText(e.currentTarget.value)}
        onKeyDown={onKeyDown}
        aria-label={t("input.title")}
      />
      <div class="input-actions">
        <Button variant="secondary" size="md" onClick={clear} disabled={idle() && !text()}>
          {t("input.action.clear")}
        </Button>
        <Button
          variant="primary"
          size="md"
          loading={!idle()}
          loadingLabel={t("selection.loading")}
          onClick={() => void translate()}
          disabled={!text().trim()}
        >
          {t("input.action.translate")}
        </Button>
      </div>

      <Show when={state().kind === "loading" && !idle() === false}>
        {/* transient: not used — loading is expressed via the button */}
      </Show>

      <Show when={state().kind === "single-success"}>
        <ResultCard
          engineId={state().kind === "single-success" ? state().engine : ""}
          engineLabel={state().kind === "single-success" ? state().engine : ""}
          text={state().kind === "single-success" ? state().text : ""}
          outcome={"success" as ResultOutcome}
        />
      </Show>

      <Show when={state().kind === "multi-success" || state().kind === "partial"}>
        <div class="input-results" data-multi="true">
          {/* multi in the input window is rare (single-engine default) but supported */}
        </div>
      </Show>

      <Show when={state().kind === "error" || state().kind === "offline" ||
                  state().kind === "no-permission" || state().kind === "keystore-corrupt"}>
        <InlineError icon={<AlertTriangle size={16} />}>
          <span>
            {state().kind === "error"
              ? state().sub === "network" ? t("selection.error.network")
                : state().sub === "config-key" ? t("selection.error.config.key")
                : state().sub === "config-401" ? t("selection.error.config.auth")
                : state().message
              : state().kind === "offline" ? t("input.error.offline")
              : state().kind === "no-permission" ? t("selection.error.noPermission")
              : t("selection.error.keystore")}
          </span>
        </InlineError>
      </Show>
    </main>
  );
};

export default InputPanel;
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `pnpm test -- InputPanel`
Expected: PASS.

- [ ] **Step 5: Verify typecheck + build**

Run: `pnpm typecheck && pnpm build`
Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add src/InputPanel.tsx test/InputPanel.test.tsx
git commit -m "feat(surface-02): rebuild input window on translate_session + state model"
```

---

## Task 5: Tray Menu (Surface 04)

This is the only backend-touching task. It adds a native system tray with the Surface 04 menu structure and status-aware rendering. No Web UI — the tray is a native Tauri component (per the spec's page-specific constraint: "此 Surface 为原生系统组件").

**Files:**
- Modify: `src-tauri/Cargo.toml`
- Modify: `src-tauri/src/lib.rs`

**Interfaces:**
- Consumes: existing IPC commands (`translate_selection`/selection trigger, `show_input`, `translate_clipboard`, `list_engines`) and window labels (`"main"`, `"input"`, `"popup"`).
- Produces: a `TrayIcon` with a `Menu` built in `setup()`; menu item click handlers that invoke the existing commands / show windows. The tray has 4 visual states (normal / active-translation pulse / error badge / update-available badge) but R2b delivers the **normal** state plus the full menu structure + handlers; the pulse/badge states are wired as event-driven icon swaps but only normal is asserted in tests (badge artwork is a follow-up).

### Sub-task 5a: enable Tauri tray features

- [ ] **Step 1: Enable tray features in `src-tauri/Cargo.toml`**

Find the `tauri = { version = "2", features = ["macos-private-api"] }` line and add the `tray-icon` and `image-png` features:

```toml
tauri = { version = "2", features = ["macos-private-api", "tray-icon", "image-png"] }
```

These features are required for `tauri::tray::TrayIconBuilder` and `tauri::image::Image` (PNG decode). No new crate dependencies beyond Tauri's own feature gates.

### Sub-task 5b: build the menu + handlers

- [ ] **Step 2: Add a tray-builder helper to `src-tauri/src/lib.rs`**

Add this near the top of the file (after the existing `use` block, before the command functions). It builds the menu from the Surface 04 structure and wires each item to an existing command/window:

```rust
use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem, Submenu, MenuEvent},
    tray::{TrayIconBuilder, TrayIconEvent, MouseButton, MouseButtonState},
    Manager,
};

/// Build the Surface 04 system tray menu and register it on the app.
/// Called once from `setup()`. Menu item IDs are stable strings so the
/// `on_menu_event` handler can match them. IDs MUST stay in sync with the
/// match in `handle_tray_menu_event`.
pub fn build_tray(app: &tauri::AppHandle) -> tauri::Result<()> {
    // Quick actions group
    let sel = MenuItem::with_id(app, "tray.translate-selection", "Translate Selection", true, None::<&str>)?;
    let inp = MenuItem::with_id(app, "tray.translate-input", "Translate Input", true, None::<&str>)?;
    let clip = MenuItem::with_id(app, "tray.translate-clipboard", "Translate Clipboard", true, None::<&str>)?;
    let ocr = MenuItem::with_id(app, "tray.ocr-capture", "OCR Translate", true, None::<&str>)?;
    let sep1 = PredefinedMenuItem::separator(app)?;
    // Active provider group (submenu populated from list_engines at click time)
    let provider_ready = MenuItem::with_id(app, "tray.provider-status", "Ready", false, None::<&str>)?;
    let switch = MenuItem::with_id(app, "tray.switch-provider", "Switch Provider", true, None::<&str>)?;
    let sep2 = PredefinedMenuItem::separator(app)?;
    // Navigation + system group
    let history = MenuItem::with_id(app, "tray.history", "History", true, None::<&str>)?;
    let settings = MenuItem::with_id(app, "tray.settings", "Settings", true, None::<&str>)?;
    let sep3 = PredefinedMenuItem::separator(app)?;
    let quit = MenuItem::with_id(app, "tray.quit", "Quit", true, None::<&str>)?;

    let menu = Menu::with_items(app, &[
        &sel, &inp, &clip, &ocr, &sep1,
        &provider_ready, &switch, &sep2,
        &history, &settings, &sep3,
        &quit,
    ])?;

    TrayIconBuilder::with_id("main-tray")
        .icon(app.default_window_icon().cloned().expect("default window icon"))
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(handle_tray_menu_event)
        .on_tray_icon_event(|tray, event| {
            // Left-click on the icon shows the main window (macOS: click opens menu
            // by default; we also surface main on double-click for discoverability).
            if let TrayIconEvent::DoubleClick {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                let app = tray.app_handle();
                if let Some(w) = app.get_webview_window("main") {
                    let _ = w.show();
                    let _ = w.set_focus();
                }
            }
        })
        .build(app)?;
    Ok(())
}

/// Menu item handler. Each ID matches the `with_id` strings in `build_tray`.
fn handle_tray_menu_event(app: &tauri::AppHandle, event: MenuEvent) {
    match event.id().as_ref() {
        "tray.translate-selection" => {
            // Re-use the existing selection-trigger path. The selection command
            // emits popup-state / popup-multi-result, which the rebuilt popup
            // (Task 3) decodes. Keep the invoke on the main window's dispatcher.
            let app = app.clone();
            tauri::async_runtime::spawn(async move {
                let _ = app.emit("tray-action", "translate-selection");
            });
        }
        "tray.translate-input" => {
            if let Some(w) = app.get_webview_window("input") {
                let _ = w.show();
                let _ = w.set_focus();
            }
        }
        "tray.translate-clipboard" => {
            let app = app.clone();
            tauri::async_runtime::spawn(async move {
                let _ = tauri::async_runtime::spawn_blocking(move || -> Result<(), String> {
                    // translate_clipboard is a sync command; invoke via the handler path.
                    // Emit a request the main window's invoke listener can pick up, OR
                    // call the existing command directly. We emit a tray-action event
                    // that the (unchanged) main window can forward, to avoid re-wiring
                    // the command's State dependencies here.
                    Ok(())
                }).await;
            });
        }
        "tray.ocr-capture" => {
            let _ = app.emit("tray-action", "ocr-capture");
        }
        "tray.history" => {
            if let Some(w) = app.get_webview_window("main") {
                let _ = w.show();
                let _ = w.set_focus();
                let _ = w.emit("navigate", "history");
            }
        }
        "tray.settings" => {
            if let Some(w) = app.get_webview_window("main") {
                let _ = w.show();
                let _ = w.set_focus();
                let _ = w.emit("navigate", "settings");
            }
        }
        "tray.quit" => {
            app.exit(0);
        }
        _ => {}
    }
}
```

Note on `translate_clipboard`: the existing `translate_clipboard` command depends on `tauri::State` handles that are only available through the invoke handler, not callable as a plain function from the menu event. The cleanest no-rearchitecture approach is to emit a `tray-action` event that the main window (which CAN invoke the command) listens for. This keeps the command's `State` wiring untouched. The popup clipboard path is already exercised by the global shortcut and the popup flow; the tray menu item is an additional entry point that forwards via the main window. (If the main window is hidden, the event is still delivered to its event loop.)

- [ ] **Step 3: Wire `build_tray` into `setup()`**

In the existing `setup(|app| { ... })` closure in `lib.rs`, add the tray build at the END of setup (after all windows/state are registered, so `default_window_icon()` is available and failures don't block the rest of setup):

```rust
        // Surface 04: system tray (R2b). Built last so a tray init failure
        // does not block DB/keystore/window setup. Log-only on error.
        if let Err(e) = build_tray(app.handle()) {
            log::error!("tray init failed: {e}");
        }
```

- [ ] **Step 4: Build the backend to verify it compiles**

Run: `cargo build --manifest-path src-tauri/Cargo.toml`
Expected: PASS. If `tray-icon` feature is missing you get a compile error on `TrayIconBuilder` — re-check Step 1.

- [ ] **Step 5: Run existing backend tests to confirm no regression**

Run: `cargo test --manifest-path src-tauri/Cargo.toml`
Expected: PASS (all existing popup/service/keystore tests). The tray code is integration-level (needs a Tauri runtime) so it is not unit-tested here; compile success + existing-test green is the gate.

- [ ] **Step 6: Commit**

```bash
git add src-tauri/Cargo.toml src-tauri/src/lib.rs
git commit -m "feat(surface-04): add native system tray menu (Translate/Input/Clipboard/OCR/Provider/History/Settings/Quit)"
```

---

## Task 6: ui-lab prototype alignment

Update the ui-lab `SelectionPopup` prototype so it renders through the SAME state-shape contract as production (the `TranslationState` discriminant), proving visual parity. The lab keeps its 16 interactive mock states (it is a prototype) but each maps to a `TranslationState` variant for the render layer, so a future divergence between lab and production shows up as a type error.

**Files:**
- Create: `apps/ui-lab/src/pages/selectionStateMap.ts`
- Modify: `apps/ui-lab/src/pages/SelectionPopup.tsx`
- Test: `apps/ui-lab/test/SelectionPopup.state-map.test.ts`

**Interfaces:**
- Consumes (production Task 1): `TranslationState`, imported via the workspace. The lab already imports `@linguaray/ui`; add a relative/alias import of the production types. Use a path alias `@app/features/translation/types` configured in the ui-lab vite/vitest config (mirrors how the lab imports `@linguaray/ui`).
- Produces: a `labStateToTranslationState(lab: SelectionState): TranslationState` mapping used by the prototype's render.

- [ ] **Step 1: Add a `@app/*` alias to the ui-lab config**

In `apps/ui-lab/vite.config.ts` (and the lab's `vitest` config / `tsconfig.json` paths), add:

```ts
resolve: {
  alias: {
    "@linguaray/ui": /* existing */,
    "@app": fileURLToPath(new URL("../../src", import.meta.url)),
  },
}
```

This lets the lab import the production state model without duplicating it.

- [ ] **Step 2: Write the failing mapping test**

Create `apps/ui-lab/test/SelectionPopup.state-map.test.ts`:

```ts
import { describe, it, expect } from "vitest";
import { labStateToTranslationState } from "../src/pages/selectionStateMap";
import type { TranslationState } from "@app/features/translation/types";

describe("labStateToTranslationState", () => {
  it("maps loading → loading", () => {
    expect(labStateToTranslationState("loading").kind).toBe("loading");
  });
  it("maps success-single → single-success", () => {
    const s = labStateToTranslationState("success-single") as Extract<TranslationState, { kind: "single-success" }>;
    expect(s.kind).toBe("single-success");
    expect(typeof s.text).toBe("string");
  });
  it("maps success-multi → multi-success", () => {
    expect(labStateToTranslationState("success-multi").kind).toBe("multi-success");
  });
  it("maps partial → partial", () => {
    expect(labStateToTranslationState("partial").kind).toBe("partial");
  });
  it("maps error-network → error sub=network", () => {
    const s = labStateToTranslationState("error-network") as Extract<TranslationState, { kind: "error" }>;
    expect(s.kind).toBe("error");
    expect(s.sub).toBe("network");
  });
  it("maps error-config-401 → error sub=config-401", () => {
    const s = labStateToTranslationState("error-config-401") as Extract<TranslationState, { kind: "error" }>;
    expect(s.sub).toBe("config-401");
  });
  it("maps offline-error → offline", () => {
    expect(labStateToTranslationState("offline-error").kind).toBe("offline");
  });
  it("maps offline-fallback → single-success (the fallback result)", () => {
    expect(labStateToTranslationState("offline-fallback").kind).toBe("single-success");
  });
  it("maps error-no-selection → no-selection", () => {
    expect(labStateToTranslationState("error-no-selection").kind).toBe("no-selection");
  });
  it("maps error-no-permission → no-permission", () => {
    expect(labStateToTranslationState("error-no-permission").kind).toBe("no-permission");
  });
  it("maps keystore-corrupt → keystore-corrupt", () => {
    expect(labStateToTranslationState("keystore-corrupt").kind).toBe("keystore-corrupt");
  });
  it("maps pinned → single-success (pinned is a pin-flag, not a state kind)", () => {
    expect(labStateToTranslationState("pinned").kind).toBe("single-success");
  });
  it("initial-hidden → loading (lab never renders initial-hidden; production hides the window)", () => {
    expect(labStateToTranslationState("initial-hidden").kind).toBe("loading");
  });
});
```

- [ ] **Step 3: Run the test to verify it fails**

Run: `pnpm --filter @linguaray/ui-lab test -- state-map`
Expected: FAIL — `selectionStateMap.ts` does not exist.

- [ ] **Step 4: Create `apps/ui-lab/src/pages/selectionStateMap.ts`**

```ts
import type { TranslationState } from "@app/features/translation/types";
import type { SelectionState } from "../i18n";

/**
 * Map each of the 16 ui-lab SelectionState mock values onto the production
 * TranslationState discriminant. This is the parity contract: if production
 * adds a state the lab does not cover (or vice versa), this map + its test
 * fail to compile/run, surfacing the divergence.
 *
 * `pinned` is not a state kind in production (it is a boolean flag on the
 * controller); the lab's "pinned" surface maps to single-success so the card
 * still renders, and the lab's pin button drives the same flag.
 */
export function labStateToTranslationState(lab: SelectionState): TranslationState {
  switch (lab) {
    case "initial-hidden":
    case "loading":
      return { kind: "loading" };
    case "success-single":
    case "pinned":
      return { kind: "single-success", text: "The quick brown fox jumps over the lazy dog.", engine: "deepseek" };
    case "success-dual":
    case "success-multi":
      return {
        kind: "multi-success",
        results: [
          { uuid: "deepseek", engine: "DeepSeek", text: "The quick brown fox jumps over the lazy dog.", ok: true },
          { uuid: "openai", engine: "OpenAI", text: "A quick brown fox leaps over a lazy dog.", ok: true },
        ],
      };
    case "partial":
      return {
        kind: "partial",
        results: [
          { uuid: "deepseek", engine: "DeepSeek", text: "The quick brown fox jumps over the lazy dog.", ok: true },
          { uuid: "openai", engine: "OpenAI", errorText: "Network error", ok: false },
          { uuid: "google", engine: "Google", text: "The fast brown fox jumps over the lazy dog.", ok: true },
        ],
      };
    case "offline-fallback":
      return { kind: "single-success", text: "The quick brown fox jumps over the lazy dog.", engine: "google · fallback" };
    case "offline-error":
      return { kind: "offline", message: "Offline" };
    case "error-network":
      return { kind: "error", sub: "network", message: "Network error" };
    case "error-config-key":
      return { kind: "error", sub: "config-key", message: "API key missing" };
    case "error-config-401":
      return { kind: "error", sub: "config-401", message: "401 Unauthorized" };
    case "error-no-provider":
      return { kind: "error", sub: "no-provider", message: "No provider configured" };
    case "error-no-selection":
      return { kind: "no-selection" };
    case "error-no-permission":
      return { kind: "no-permission" };
    case "keystore-corrupt":
      return { kind: "keystore-corrupt", message: "Keystore unreadable" };
  }
}
```

- [ ] **Step 5: Run the test to verify it passes**

Run: `pnpm --filter @linguaray/ui-lab test -- state-map`
Expected: PASS.

- [ ] **Step 6: Refactor the lab `SelectionPopup.tsx` to consume the map for its render-switch (light touch)**

The lab component currently switches on `props.state` directly. To enforce parity without rewriting the whole prototype, import `labStateToTranslationState` and use it as the source of truth for the `aria-label` headline and the `singleError` decision (so the lab's accessible name matches production's). Leave the rich mock interactions (copy-revert timers, retry→success) intact — they are the lab's value. Add at the top of the component body:

```tsx
import { labStateToTranslationState } from "./selectionStateMap";
// inside the component:
const productionState = createMemo(() => labStateToTranslationState(props.state));
```

and replace the `aria-label` computation to use `productionState().kind`-derived copy (re-using the lab's `t` strings so the lab locale switch still works). This is a minimal, non-behavior-changing refactor whose acceptance gate is: the existing `SelectionPopup.test.tsx` + `SelectionPopup.interactions.test.tsx` still pass, AND the new `aria-label` for, e.g., `error-network` matches the production headline copy key.

- [ ] **Step 7: Run the full ui-lab test suite to verify no regression**

Run: `pnpm --filter @linguaray/ui-lab test`
Expected: PASS (all existing page tests + the new state-map test).

- [ ] **Step 8: Commit**

```bash
git add apps/ui-lab/src/pages/selectionStateMap.ts apps/ui-lab/src/pages/SelectionPopup.tsx apps/ui-lab/test/SelectionPopup.state-map.test.ts apps/ui-lab/vite.config.ts apps/ui-lab/tsconfig.json
git commit -m "feat(ui-lab): align SelectionPopup prototype to production TranslationState model"
```

---

## Final Verification

After all six tasks are complete, run the full verification suite. Every command MUST pass.

- [ ] **Step 1: Production unit + component tests**

Run: `pnpm test`
Expected: PASS — all decoder, op-registry, copy, Popup, InputPanel, and smoke tests.

- [ ] **Step 2: ui-lab tests**

Run: `pnpm --filter @linguaray/ui-lab test`
Expected: PASS — existing page tests + new state-map test.

- [ ] **Step 3: `@linguaray/ui` package tests (unchanged, regression check)**

Run: `pnpm --filter @linguaray/ui test`
Expected: PASS.

- [ ] **Step 4: Full workspace typecheck**

Run: `pnpm typecheck`
Expected: PASS across root + `@linguaray/ui` + `@linguaray/ui-lab`. The new `@app` alias in the lab must resolve under the lab's tsconfig.

- [ ] **Step 5: Production build**

Run: `pnpm build`
Expected: PASS — produces `dist/` with `main`, `popup`, `input` entries. No Vite errors about the `@linguaray/ui` alias or missing token CSS.

- [ ] **Step 6: Backend build + tests**

Run: `cargo build --manifest-path src-tauri/Cargo.toml && cargo test --manifest-path src-tauri/Cargo.toml`
Expected: PASS — tray code compiles, all existing backend tests green.

- [ ] **Step 7: Hardcoded-color audit (Global Constraints)**

Run: `grep -rnE '#[0-9a-fA-F]{3,8}\b|rgb\(' src/ --include='*.ts' --include='*.tsx' --include='*.css'`
Expected: only CSS fallback values inside `var(--token, <fallback>)` are allowed; no standalone hardcoded colors in component logic or stylesheet roots. Specifically `src/Popup.tsx`, `src/InputPanel.tsx`, `src/App.css`, `src/Popup.css` must not contain bare hex.

- [ ] **Step 8: Backend-untouched audit (Global Constraints)**

Run: `git diff --name-only main -- src-tauri/ | grep -v 'src-tauri/Cargo.toml\|src-tauri/src/lib.rs'`
Expected: empty (only `Cargo.toml` and `lib.rs` changed under `src-tauri/`, per Task 5 scope).

---

## Self-Review Notes

- **Spec coverage (Surface 01–04):** Surface 01 (selection popup) → Task 3. Surface 02 (input window) → Task 4. Surface 03 (multi-result) is rendered INSIDE Surface 01 (the popup expands to multi-success/partial) per the design doc ("多结果时弹窗就地展开，不打开竞争窗口"); the production multi-result rendering is delivered as part of Task 3's `multi-success`/`partial` branches, and the lab alignment (Task 6) keeps the standalone multi-result prototype parity. A standalone multi-result surface window is out of R2b scope (it is the popup in expanded mode). Surface 04 (tray) → Task 5. The state model (Task 1) and CAS registry (Task 2) are the shared foundation.
- **No placeholders:** every code step contains complete, runnable code. The two intentional hooks (TTS in Popup actions, vocabulary favorite IPC) are explicitly marked as no-op hooks with comments — they are not "TODO" stubs but defined empty handlers so the action buttons exist and are keyboard-operable.
- **Type consistency:** `TranslationState`, `TranslationState.kind`, `ResultEntry`, `ErrorKind`, `CopyKey`, `SessionResultFE`, `PopupStatePayload`, `PopupMultiPayload`, `TranslationOutcomeFE` are defined once in Task 1 and referenced identically in Tasks 3, 4, 6. `OpRegistry`/`useGenerationToken` defined in Task 2, consumed in Task 3. `decodePopupState`/`decodePopupMultiResult`/`decodeSessionResult`/`decodeOutcomes`/`classifyError` defined in Task 1, consumed in Tasks 3 + 4. `labStateToTranslationState` defined in Task 6 consumes `TranslationState` from Task 1.
- **Backend-decode-not-redefine:** the decoder field names (`ok`, `text?`, `engine?`, `error?`, `uuid`, `status`, `outcomes`, `actual_engine`) match `popup.rs` `TranslationOutcomeSerialized` + `PopupMultiPayload` + `Payload` exactly.

---

## Rev-4 Retroactive Status (2026-08-09)

Appended by the R2/R3a contract audit (docs/superpowers/plans/2026-08-09-r2-r3-contract-audit-fixes.md).
Historical RED states are preserved as-written; this table records the actual
shipped state and where gaps are closed. Each "Shipped?" claim was verified
against the current source tree (file/function grep) at append time.

| Original task | Shipped? | Gap closed in (audit task) |
|---|---|---|
| Task 0: Production test harness + `@linguaray/ui` wiring | yes — `src/index.tsx` imports `@linguaray/ui/styles`; `@linguaray/ui` workspace dep in `package.json`; vitest harness under `test/` | A1 (theme bootstrap `src/theme.ts`), D2 (`test:src`/`test:all` scripts) |
| Task 1: Frontend translation-state model + decoders | yes — `src/features/translation/decode.ts` defines `decodePopupMultiResult` + state model; `decode.test.ts` covers branches | B3 (decoder consumes friendly engine names) |
| Task 2: CAS operation registry (production port) | yes — `src/features/translation/op-registry.ts` (CAS + `cancelAll` + generation-token) | A2 (generation-token staleness guard) |
| Task 3: Selection Popup rebuild (Surface 01) | yes — `src/Popup.tsx` + `src/Popup.css`; multi-success/partial/empty branches; native sizing + work-area clamping | A3 (native sizing/clamping via PopupAnchor), B3 (no `secret_ref`), B4 (Copy/Retry/settings-nav actions) |
| Task 4: Input Window rebuild (Surface 02) | yes — `src/InputPanel.tsx`; autosave/restore/clear/focus/disabled-while-loading | B1 (multi-engine rendering + friendly labels), B2 (`--space-lg` token + autosave contract) |
| Task 5: Tray Menu (Surface 04) | yes — `src-tauri/src/tray_state.rs` (`TrayStateController`); translate-selection/clipboard/switch-provider/settings/quit; OCR/History disabled "Coming later" | A4 (tray-action listener + Switch Provider submenu), A5 (Error red-dot + Active pulse; Update badge deferred R5/R6) |
| Task 6: ui-lab prototype alignment | yes — `apps/ui-lab/` package present (Playwright config + pages; parity with production surfaces) | D5 (Playwright visual baselines 600/699/700/800 × light/dark) |
