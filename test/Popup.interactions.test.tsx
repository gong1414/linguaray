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
  it("R7-P1-2: stale Retry reject does not overwrite a newer popup state", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    const invokeMock = vi.mocked(invoke);

    // translate_selection_ipc returns a deferred promise we control so we can
    // reject it AFTER a newer event arrives.
    let rejectRetry!: (e: Error) => void;
    invokeMock.mockImplementation(async (cmd: string) => {
      if (cmd === "translate_selection_ipc") {
        return new Promise((_res, rej) => {
          rejectRetry = rej;
        });
      }
      return { outcomes: [], actual_engine: undefined };
    });

    const { getByText, queryByText, queryByRole } = render(() => <Popup />);

    // Initial success result with a saved source.
    await emitEvent("popup-state", {
      status: "result",
      text: "First result",
      engine: "deepseek/u1",
      source_text: "hello",
    });
    await waitFor(() => expect(getByText("First result")).toBeTruthy());

    // Click Retry → translate_selection_ipc fires (deferred — pending).
    const retryBtn = await waitFor(() => getByText(/Retry|重试/));
    fireEvent.click(retryBtn);
    await waitFor(() =>
      expect(invokeMock).toHaveBeenCalledWith("translate_selection_ipc", {
        text: "hello",
      }),
    );

    // A NEWER popup-state arrives (e.g. user selected new text → backend
    // emitted a fresh result). This bumps the state generation, making the
    // in-flight Retry stale.
    await emitEvent("popup-state", {
      status: "result",
      text: "Newer result",
      engine: "deepseek/u1",
      source_text: "world",
    });
    await waitFor(() => expect(getByText("Newer result")).toBeTruthy());

    // Now the stale Retry's IPC rejects. The generation guard (myGen !==
    // stateGeneration) prevents the catch block from overwriting the newer
    // state with an error.
    rejectRetry(new Error("IPC timeout"));
    await new Promise((r) => setTimeout(r, 50));

    // The popup STILL shows the newer result (not an error).
    expect(getByText("Newer result")).toBeTruthy();
    expect(queryByText("IPC timeout")).toBeNull();
    expect(queryByRole("alert")).toBeNull();
  });

  it("R7-P1-2: stale Retry resolve does not overwrite a newer popup state", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    const invokeMock = vi.mocked(invoke);

    // translate_selection_ipc returns a deferred promise we control so we can
    // resolve it AFTER a newer event arrives.
    let resolveRetry!: (v: unknown) => void;
    invokeMock.mockImplementation(async (cmd: string) => {
      if (cmd === "translate_selection_ipc") {
        return new Promise((res) => {
          resolveRetry = res;
        });
      }
      return { outcomes: [], actual_engine: undefined };
    });

    const { getByText } = render(() => <Popup />);

    // Initial success result.
    await emitEvent("popup-state", {
      status: "result",
      text: "First result",
      engine: "deepseek/u1",
      source_text: "hello",
    });
    await waitFor(() => expect(getByText("First result")).toBeTruthy());

    // Click Retry → translate_selection_ipc fires (deferred — pending).
    const retryBtn = await waitFor(() => getByText(/Retry|重试/));
    fireEvent.click(retryBtn);

    // A NEWER popup-state arrives → bumps generation → Retry is now stale.
    await emitEvent("popup-state", {
      status: "result",
      text: "Newer result",
      engine: "deepseek/u1",
      source_text: "world",
    });
    await waitFor(() => expect(getByText("Newer result")).toBeTruthy());

    // Resolve the stale Retry. The generation guard prevents it from touching
    // state (the newer event already set it). No loading or stale state.
    resolveRetry({ outcomes: [], actual_engine: undefined });
    await new Promise((r) => setTimeout(r, 50));

    // The popup STILL shows the newer result (not reverted to loading/stale).
    expect(getByText("Newer result")).toBeTruthy();
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
