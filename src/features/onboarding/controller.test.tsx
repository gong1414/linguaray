import { act, cleanup, renderHook, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

// --- IPC + bridge mocks (controller's only side-effect seams) -------------
const { ipc, focusHandlers, unlistenMock, hideMock, openUrlMock } = vi.hoisted(() => {
  const focusHandlers: Array<(e: { payload: boolean }) => void> = [];
  return {
    focusHandlers,
    unlistenMock: vi.fn(),
    hideMock: vi.fn(),
    openUrlMock: vi.fn(),
    ipc: {
      getOnboardingStatus: vi.fn(),
      onboardingNext: vi.fn(),
      completeOnboarding: vi.fn(),
      a11yStatus: vi.fn(),
      screenCaptureStatus: vi.fn(),
      listProviders: vi.fn(),
      listShortcuts: vi.fn(),
      setHistoryEnabled: vi.fn(),
      openSettingsSection: vi.fn(),
    },
  };
});

vi.mock("./ipc", () => ipc);
vi.mock("../../bridge/window", () => ({
  getCurrentWindow: () => ({
    onFocusChanged: vi.fn(async (cb: (e: { payload: boolean }) => void) => {
      focusHandlers.push(cb);
      return unlistenMock;
    }),
    hide: hideMock,
  }),
}));
vi.mock("../../bridge/opener", () => ({ openUrl: openUrlMock }));

import { useOnboardingController } from "./controller";

beforeEach(() => {
  focusHandlers.length = 0;
  vi.clearAllMocks();
  // jsdom reports an empty navigator.platform; the controller's Apple check
  // (screen-capture permission) needs a Mac host like production.
  Object.defineProperty(window.navigator, "platform", {
    value: "MacIntel",
    configurable: true,
  });
  openUrlMock.mockResolvedValue(undefined);
  ipc.getOnboardingStatus.mockResolvedValue({ complete: false, step: "welcome" });
  ipc.a11yStatus.mockResolvedValue(true);
  ipc.screenCaptureStatus.mockResolvedValue(true);
  ipc.listProviders.mockResolvedValue([{ id: "p1" }, { id: "p2" }]);
  ipc.listShortcuts.mockResolvedValue({
    entries: [{ action: "translate_selection", combo: "Alt+D" }],
  });
  ipc.onboardingNext.mockResolvedValue("accessibility");
  ipc.completeOnboarding.mockResolvedValue(undefined);
  ipc.setHistoryEnabled.mockResolvedValue(undefined);
  ipc.openSettingsSection.mockResolvedValue(undefined);
});

afterEach(cleanup);

describe("useOnboardingController (controller + ipc integration)", () => {
  it("restores the persisted step from onboarding_status", async () => {
    ipc.getOnboardingStatus.mockResolvedValue({ complete: false, step: "provider" });
    const { result } = renderHook(() => useOnboardingController());
    await waitFor(() => expect(result.current.step).toBe("provider"));
  });

  it("maps granted/denied permissions from real command results", async () => {
    ipc.a11yStatus.mockResolvedValue(false);
    ipc.screenCaptureStatus.mockResolvedValue(true);
    const { result } = renderHook(() => useOnboardingController());
    await waitFor(() => expect(result.current.a11y).toBe("denied"));
    await waitFor(() => expect(result.current.screenCapture).toBe("granted"));
  });

  it("maps command failure to the honest error state", async () => {
    ipc.a11yStatus.mockRejectedValue(new Error("boom"));
    const { result } = renderHook(() => useOnboardingController());
    await waitFor(() => expect(result.current.a11y).toBe("error"));
  });

  it("loads provider count and shortcuts for their steps", async () => {
    ipc.getOnboardingStatus.mockResolvedValue({ complete: false, step: "shortcuts" });
    const { result } = renderHook(() => useOnboardingController());
    await waitFor(() => expect(result.current.shortcuts).toHaveLength(1));
  });

  it("advance persists the transition and adopts the returned step", async () => {
    const { result } = renderHook(() => useOnboardingController());
    await waitFor(() => expect(result.current.a11y).toBe("granted"));
    await act(async () => {
      result.current.advance("start");
    });
    expect(ipc.onboardingNext).toHaveBeenCalledWith("welcome", "start");
    await waitFor(() => expect(result.current.step).toBe("accessibility"));
    expect(ipc.completeOnboarding).not.toHaveBeenCalled();
  });

  it("reaching done completes onboarding (after the step write)", async () => {
    ipc.onboardingNext.mockResolvedValue("done");
    const { result } = renderHook(() => useOnboardingController());
    await act(async () => {
      result.current.advance("complete");
    });
    expect(ipc.onboardingNext).toHaveBeenCalledWith("welcome", "complete");
    await waitFor(() => expect(ipc.completeOnboarding).toHaveBeenCalledTimes(1));
    expect(result.current.step).toBe("done");
  });

  it("enableHistory writes history_set_enabled then advances", async () => {
    ipc.onboardingNext.mockResolvedValue("shortcuts");
    ipc.getOnboardingStatus.mockResolvedValue({ complete: false, step: "history" });
    const { result } = renderHook(() => useOnboardingController());
    await waitFor(() => expect(result.current.step).toBe("history"));
    await act(async () => {
      result.current.enableHistory();
    });
    expect(ipc.setHistoryEnabled).toHaveBeenCalledWith(true);
    expect(ipc.onboardingNext).toHaveBeenCalledWith("history", "continue");
  });

  it("finish(true) completes, opens settings, then hides the window", async () => {
    const { result } = renderHook(() => useOnboardingController());
    await act(async () => {
      result.current.finish(true);
    });
    await waitFor(() => expect(hideMock).toHaveBeenCalledTimes(1));
    expect(ipc.completeOnboarding).toHaveBeenCalledTimes(1);
    expect(ipc.openSettingsSection).toHaveBeenCalledWith("provider-center");
  });

  it("a rejected advance surfaces the error string, not a crash", async () => {
    ipc.onboardingNext.mockRejectedValue("db: locked");
    const { result } = renderHook(() => useOnboardingController());
    await act(async () => {
      result.current.advance("start");
    });
    await waitFor(() => expect(result.current.error).toBe("db: locked"));
    expect(result.current.step).toBe("welcome");
  });

  it("window focus re-checks permissions (user just toggled the grant)", async () => {
    const { result } = renderHook(() => useOnboardingController());
    await waitFor(() => expect(focusHandlers.length).toBeGreaterThan(0));
    ipc.a11yStatus.mockResolvedValue(false);
    await act(async () => {
      for (const cb of [...focusHandlers]) cb({ payload: true });
    });
    await waitFor(() => expect(result.current.a11y).toBe("denied"));
  });

  it("openA11ySettings opens the Accessibility pane via the opener", async () => {
    const { result } = renderHook(() => useOnboardingController());
    result.current.openA11ySettings();
    await waitFor(() =>
      expect(openUrlMock).toHaveBeenCalledWith(
        "x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility",
      ),
    );
  });

  it("unmount releases the focus listeners", async () => {
    const { unmount } = renderHook(() => useOnboardingController());
    // Both focus registrations (permissions + provider step) captured their
    // callbacks before unmount.
    await waitFor(() => expect(focusHandlers.length).toBe(2));
    expect(unlistenMock).not.toHaveBeenCalled();
    unmount();
    expect(unlistenMock).toHaveBeenCalledTimes(2);
  });
});
