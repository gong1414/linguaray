/**
 * Production contract tests for Popup (Surface 01).
 *
 * Migrated from the deleted `apps/ui-lab/test/SelectionPopup.interactions.test.tsx`
 * (commit 7f21adc) which tested the lab mock fixture with fake timers. These
 * drive the REAL production Popup controller + PopupView and verify
 * async-safety + aria contracts:
 *
 *  - stale Retry: a stale translate_selection_ipc completion does not overwrite
 *    a newer popup state.
 *  - pin/unpin: the Pin/Unpin button sets aria-pressed correctly.
 */
import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { render, fireEvent, cleanup, waitFor } from "@solidjs/testing-library";

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
});

afterEach(async () => {
  const { invoke } = await import("@tauri-apps/api/core");
  vi.mocked(invoke).mockImplementation(async () => ({
    outcomes: [],
    actual_engine: undefined,
  }));
  cleanup();
});

describe("Popup — production interaction contracts", () => {
  it("stale Retry completion does not overwrite a newer popup state", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    const invokeMock = vi.mocked(invoke);

    // translate_selection_ipc hangs forever (never resolves) so the stale Retry
    // remains "in flight" while a newer event arrives.
    invokeMock.mockImplementation(async (cmd: string) => {
      if (cmd === "translate_selection_ipc") return new Promise(() => {});
      return { outcomes: [], actual_engine: undefined };
    });

    const { getByText, queryByRole } = render(() => <Popup />);

    // Initial success result.
    await emitEvent("popup-state", {
      status: "result",
      text: "First result",
      engine: "deepseek/u1",
      source_text: "hello",
    });
    await waitFor(() => expect(getByText("First result")).toBeTruthy());

    // Click Retry → translate_selection_ipc fires (hangs). The popup enters
    // loading state.
    const retryBtn = await waitFor(() => getByText(/Retry|重试/));
    fireEvent.click(retryBtn);
    await waitFor(() =>
      expect(invokeMock).toHaveBeenCalledWith("translate_selection_ipc", {
        text: "hello",
      }),
    );

    // A NEWER popup-state arrives (e.g. user selected new text → backend
    // emitted a fresh result). This must REPLACE the loading state, not be
    // overwritten by the stale Retry's eventual completion.
    await emitEvent("popup-state", {
      status: "result",
      text: "Newer result",
      engine: "deepseek/u1",
      source_text: "world",
    });
    await waitFor(() => expect(getByText("Newer result")).toBeTruthy());
    // The loading state is gone (no spinner region with aria-busy).
    const region = getByText("Newer result").closest("[role='region']");
    expect(region?.getAttribute("aria-busy")).toBeFalsy();
    // No error alert.
    expect(queryByRole("alert")).toBeNull();
  });

  it("pin/unpin sets aria-pressed correctly", async () => {
    const { getByRole, queryByRole } = render(() => <Popup />);
    await emitEvent("popup-state", {
      status: "result",
      text: "Hello",
      engine: "deepseek/u1",
      source_text: "hi",
    });

    // Initially unpinned → Pin button, aria-pressed absent (not pressed).
    const pin = await waitFor(() => getByRole("button", { name: /Pin|固定/ }));
    // aria-pressed is undefined when not active (no attribute rendered).
    expect(pin.hasAttribute("aria-pressed")).toBe(false);

    // Click Pin → becomes pinned. The button label flips to Unpin and
    // aria-pressed="true".
    fireEvent.click(pin);
    const unpin = await waitFor(() => getByRole("button", { name: /Unpin|取消固定/ }));
    expect(unpin.getAttribute("aria-pressed")).toBe("true");

    // Click Unpin → back to Pin. aria-pressed is absent again (not "true").
    fireEvent.click(unpin);
    const pinAgain = await waitFor(() => getByRole("button", { name: /Pin|固定/ }));
    expect(pinAgain.hasAttribute("aria-pressed")).toBe(false);
    // The Unpin button is gone.
    expect(queryByRole("button", { name: /Unpin|取消固定/ })).toBeNull();
  });
});
