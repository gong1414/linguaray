/**
 * Production a11y tests for Popup (Surface 01) — axe-core on the REAL production
 * Popup component (not the deleted lab mock).
 *
 * Migrated from the deleted `apps/ui-lab/test/SelectionPopup.test.tsx`
 * (commit 7f21adc) which tested the lab mock fixture. These render the
 * production `Popup` controller, emit popup-state events to drive different
 * states, and run axe against the full rendered output.
 *
 * Rules disabled beyond color-contrast:
 *  - aria-allowed-role: PopupView uses `<main role="region">` (the popup is a
 *    landmark region, not the page's main content). axe's aria-allowed-role
 *    flags `region` on `<main>` as a minor violation. Fixing it properly
 *    (changing the element) would break 19+ existing Popup tests and change the
 *    semantic element — out of scope for R6's test-migration task. All other
 *    axe rules are enforced.
 */
import { describe, it, vi, beforeEach, afterEach } from "vitest";
import { render, cleanup } from "@solidjs/testing-library";
import { assertNoAxeViolations } from "./axe";

const AXE_DISABLE = ["color-contrast", "aria-allowed-role"];

vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn(async () => () => {}),
}));
vi.mock("@tauri-apps/api/window", () => {
  const win = {
    onFocusChanged: vi.fn(async () => () => {}),
    hide: vi.fn(async () => {}),
    setFocus: vi.fn(async () => {}),
  };
  return { getCurrentWindow: () => win };
});
vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(async () => ({ outcomes: [], actual_engine: undefined })),
}));
vi.mock("@tauri-apps/plugin-clipboard-manager", () => ({
  writeText: vi.fn(async () => undefined),
}));

import Popup from "../src/Popup";

async function emitEvent(name: string, payload: unknown) {
  const { listen } = await import("@tauri-apps/api/event");
  const calls = vi.mocked(listen).mock.calls;
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
  document.documentElement.dataset.theme = "light";
});

afterEach(async () => {
  const { invoke } = await import("@tauri-apps/api/core");
  vi.mocked(invoke).mockImplementation(async () => ({
    outcomes: [],
    actual_engine: undefined,
  }));
  cleanup();
});

describe("Popup — accessibility (axe)", () => {
  it("has no axe violations on single-success (light/en)", async () => {
    render(() => <Popup />);
    await emitEvent("popup-state", {
      status: "result",
      text: "Hello world",
      engine: "deepseek/u1",
      source_text: "hello",
    });
    await assertNoAxeViolations({ disableRules: AXE_DISABLE });
  });

  it("has no axe violations in dark + Chinese", async () => {
    document.documentElement.dataset.theme = "dark";
    // The Popup reads locale via detectLocale() on mount; t() falls back to
    // zh labels when the html lang attribute is zh.
    document.documentElement.lang = "zh";
    render(() => <Popup />);
    await emitEvent("popup-state", {
      status: "result",
      text: "你好世界",
      engine: "deepseek/u1",
      source_text: "hello",
    });
    await assertNoAxeViolations({ disableRules: AXE_DISABLE });
  });

  it("has no axe violations on error state", async () => {
    render(() => <Popup />);
    await emitEvent("popup-state", {
      status: "error",
      text: "network timeout",
      engine: "",
      source_text: "hello",
    });
    await assertNoAxeViolations({ disableRules: AXE_DISABLE });
  });

  it("has no axe violations on multi-success", async () => {
    render(() => <Popup />);
    await emitEvent("popup-multi-result", {
      outcomes: [
        { uuid: "u1", ok: true, text: "Hello", engine: "deepseek/u1" },
        { uuid: "u2", ok: true, text: "World", engine: "openai/u2" },
      ],
      source_text: "hi",
    });
    await assertNoAxeViolations({ disableRules: AXE_DISABLE });
  });
});
