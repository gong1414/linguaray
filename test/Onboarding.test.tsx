import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { render, fireEvent, cleanup, waitFor } from "@solidjs/testing-library";

const ORDER = ["welcome", "accessibility", "provider", "history", "shortcuts", "done"] as const;

const { invokeMock, openUrlMock, hideMock } = vi.hoisted(() => ({
  invokeMock: vi.fn(),
  openUrlMock: vi.fn(async () => undefined),
  hideMock: vi.fn(async () => undefined),
}));

vi.mock("@tauri-apps/api/core", () => ({ invoke: invokeMock }));
vi.mock("@tauri-apps/api/window", () => ({
  getCurrentWindow: () => ({
    hide: hideMock,
    onFocusChanged: () => Promise.resolve(() => {}),
  }),
}));
vi.mock("@tauri-apps/plugin-opener", () => ({ openUrl: openUrlMock }));

import Onboarding from "../src/Onboarding";

/** Route every IPC command the container can issue. */
function mockIpc(overrides: Record<string, (args?: unknown) => unknown> = {}) {
  invokeMock.mockImplementation((cmd: string, args?: Record<string, unknown>) => {
    const o = overrides[cmd];
    if (o) return o(args);
    switch (cmd) {
      case "onboarding_status":
        return Promise.resolve({ complete: false, step: "welcome" });
      case "onboarding_next":
        return Promise.resolve(ORDER[Math.min(ORDER.indexOf(args?.step as never) + 1, 5)]);
      case "onboarding_complete":
        return Promise.resolve();
      case "a11y_status":
        return Promise.resolve(false);
      case "screen_capture_status":
        return Promise.resolve(false);
      case "provider_list":
        return Promise.resolve([]);
      case "shortcut_list":
        return Promise.resolve({
          entries: [
            { action: "translate_selection", combo: "Alt+Space" },
            { action: "ocr_translate", combo: "Alt+Shift+Space" },
          ],
        });
      case "history_set_enabled":
        return Promise.resolve({ enabled: true, retention_days: 30, record_count: 0 });
      default:
        return Promise.resolve(undefined);
    }
  });
}

beforeEach(() => {
  localStorage.setItem("linguaray.locale", "en");
  invokeMock.mockClear();
  openUrlMock.mockClear();
  hideMock.mockClear();
  mockIpc();
});
afterEach(() => cleanup());

describe("Onboarding", () => {
  it("restores the persisted step on mount (onboarding_status)", async () => {
    mockIpc({ onboarding_status: () => Promise.resolve({ complete: false, step: "history" }) });
    const { findByText } = render(() => <Onboarding />);
    expect(await findByText(/Translations can be stored locally/)).toBeInTheDocument();
  });

  it("queries the REAL a11y + screen-recording status (not a fake Continue)", async () => {
    // The container gates the Screen Recording check on Apple platforms —
    // fake the platform so both badges take the real check path.
    Object.defineProperty(navigator, "platform", { value: "MacIntel", configurable: true });
    const { findByText, findAllByText } = render(() => <Onboarding />);
    fireEvent.click(await findByText("Get started"));
    // Both permission badges surface the denied truth.
    expect(await findAllByText("Not granted")).toHaveLength(2);
    expect(invokeMock).toHaveBeenCalledWith("a11y_status");
    expect(invokeMock).toHaveBeenCalledWith("screen_capture_status");
    Object.defineProperty(navigator, "platform", { value: "", configurable: true });
  });

  it("labels the escape hatch honestly and opens both permission panes", async () => {
    Object.defineProperty(navigator, "platform", { value: "MacIntel", configurable: true });
    const { findByText, findAllByText, getByText } = render(() => <Onboarding />);
    fireEvent.click(await findByText("Get started"));
    expect(await findByText("Set up later")).toBeInTheDocument();
    // Wait for BOTH permission checks to resolve — the screen-recording
    // button is disabled while its state is "checking". Click the BUTTON
    // elements directly: after the step transition, clicking the label SPAN
    // hits a jsdom-only Solid event-delegation quirk on the second card.
    await findAllByText("Not granted");
    fireEvent.click(
      (getByText("Open Accessibility Settings") as HTMLElement).closest("button")!,
    );
    // Let the first open panes call land before the second click (also matches
    // real user pacing between two separate buttons).
    await waitFor(() => expect(openUrlMock).toHaveBeenCalledTimes(1));
    fireEvent.click(
      (getByText("Open Screen Recording Settings") as HTMLElement).closest("button")!,
    );
    await waitFor(() =>
      expect(openUrlMock).toHaveBeenCalledWith(
        "x-apple.systempreferences:com.apple.preference.security?Privacy_ScreenCapture",
      ),
    );
    Object.defineProperty(navigator, "platform", { value: "", configurable: true });
  });

  it("provider step shows the real count and a settings entry point", async () => {
    mockIpc({
      onboarding_status: () => Promise.resolve({ complete: false, step: "provider" }),
      provider_list: () => Promise.resolve([{ uuid: "a" }, { uuid: "b" }]),
    });
    const { findByText } = render(() => <Onboarding />);
    expect(await findByText("2 providers configured")).toBeInTheDocument();
    expect(await findByText("Open Provider Settings")).toBeInTheDocument();
  });

  it("provider zero state is explicit, not a fake success", async () => {
    mockIpc({ onboarding_status: () => Promise.resolve({ complete: false, step: "provider" }) });
    const { findByText } = render(() => <Onboarding />);
    expect(await findByText(/No provider yet/)).toBeInTheDocument();
  });

  it("history enable only advances after the write succeeds", async () => {
    mockIpc({ onboarding_status: () => Promise.resolve({ complete: false, step: "history" }) });
    const { findByText } = render(() => <Onboarding />);
    fireEvent.click(await findByText("Enable history"));
    await waitFor(() =>
      expect(invokeMock).toHaveBeenCalledWith("history_set_enabled", { enabled: true }),
    );
    // Advanced to shortcuts (the next persisted step) on success.
    expect(await findByText("Finish setup")).toBeInTheDocument();
  });

  it("history write failure keeps the step and surfaces the error", async () => {
    mockIpc({
      onboarding_status: () => Promise.resolve({ complete: false, step: "history" }),
      history_set_enabled: () => Promise.reject(new Error("keystore locked")),
    });
    const { findByText, queryByText } = render(() => <Onboarding />);
    fireEvent.click(await findByText("Enable history"));
    expect(await findByText(/keystore locked/)).toBeInTheDocument();
    expect(queryByText("Finish setup")).toBeNull();
  });

  it("shortcuts step lists the loaded defaults", async () => {
    mockIpc({ onboarding_status: () => Promise.resolve({ complete: false, step: "shortcuts" }) });
    const { findByText } = render(() => <Onboarding />);
    expect(await findByText("Alt+Space")).toBeInTheDocument();
    expect(await findByText("Alt+Shift+Space")).toBeInTheDocument();
  });

  it("a failing onboarding_next does NOT advance and shows the error", async () => {
    mockIpc({ onboarding_next: () => Promise.reject(new Error("db busy")) });
    const { findByText, queryByText } = render(() => <Onboarding />);
    fireEvent.click(await findByText("Get started"));
    expect(await findByText(/db busy/)).toBeInTheDocument();
    expect(queryByText("Grant permissions")).toBeNull();
  });

  it("done step completes the flow and hides the window only after writes", async () => {
    mockIpc({ onboarding_status: () => Promise.resolve({ complete: false, step: "done" }) });
    const { findByText } = render(() => <Onboarding />);
    fireEvent.click(await findByText("Start using LinguaRay"));
    await waitFor(() => expect(invokeMock).toHaveBeenCalledWith("onboarding_complete"));
    await waitFor(() => expect(hideMock).toHaveBeenCalled());
  });
});
