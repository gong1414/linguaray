import { act, cleanup, renderHook, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

const { a11yStatusMock, openUrlMock, focusHandlers, unlistenMock } = vi.hoisted(() => {
  const focusHandlers: Array<(e: { payload: boolean }) => void> = [];
  return {
    focusHandlers,
    unlistenMock: vi.fn(),
    a11yStatusMock: vi.fn(),
    openUrlMock: vi.fn(),
  };
});

vi.mock("./ipc", () => ({ a11yStatus: a11yStatusMock }));
vi.mock("../../../bridge/opener", () => ({ openUrl: openUrlMock }));
vi.mock("../../../bridge/window", () => ({
  getCurrentWindow: () => ({
    onFocusChanged: vi.fn(async (cb: (e: { payload: boolean }) => void) => {
      focusHandlers.push(cb);
      return unlistenMock;
    }),
  }),
}));

import { useShellController } from "./controller";

beforeEach(() => {
  focusHandlers.length = 0;
  vi.clearAllMocks();
  a11yStatusMock.mockResolvedValue(true);
  openUrlMock.mockResolvedValue(undefined);
});

afterEach(cleanup);

describe("useShellController", () => {
  it("loads the a11y grant on mount", async () => {
    const { result } = renderHook(() => useShellController());
    await waitFor(() => expect(result.current.a11yGranted).toBe(true));
  });

  it("a11y_status=false surfaces the banner state", async () => {
    a11yStatusMock.mockResolvedValue(false);
    const { result } = renderHook(() => useShellController());
    await waitFor(() => expect(result.current.a11yGranted).toBe(false));
  });

  it("a11y_status failure keeps the banner hidden (non-blocking)", async () => {
    a11yStatusMock.mockRejectedValue(new Error("no ipc"));
    const { result } = renderHook(() => useShellController());
    await waitFor(() => expect(result.current.a11yGranted).toBe(true));
  });

  it("window focus re-checks the grant", async () => {
    const { result } = renderHook(() => useShellController());
    await waitFor(() => expect(focusHandlers.length).toBe(1));
    a11yStatusMock.mockResolvedValue(false);
    await act(async () => {
      focusHandlers[0]({ payload: true });
    });
    await waitFor(() => expect(result.current.a11yGranted).toBe(false));
  });

  it("unmount releases the focus listener (race-safe)", async () => {
    const { unmount } = renderHook(() => useShellController());
    await waitFor(() => expect(focusHandlers.length).toBe(1));
    unmount();
    expect(unlistenMock).toHaveBeenCalledTimes(1);
  });

  it("openSystemSettings opens the Accessibility pane", () => {
    const { result } = renderHook(() => useShellController());
    result.current.openSystemSettings();
    expect(openUrlMock).toHaveBeenCalledWith(
      "x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility",
    );
  });

  it("uncontrolled navigation updates active; onNavigate fires", () => {
    const onNavigate = vi.fn();
    const { result } = renderHook(() => useShellController({ onNavigate }));
    expect(result.current.active).toBe("provider-center");
    act(() => {
      result.current.setActive("privacy");
    });
    expect(result.current.active).toBe("privacy");
    expect(onNavigate).toHaveBeenCalledWith("privacy");
  });

  it("controlled activePage wins over internal state", () => {
    const { result, rerender } = renderHook(
      ({ page }: { page: "provider-center" | "history" }) => useShellController({ activePage: page }),
      { initialProps: { page: "provider-center" as "provider-center" | "history" } },
    );
    expect(result.current.active).toBe("provider-center");
    act(() => {
      result.current.setActive("privacy");
    });
    // Parent still owns the state → controlled value wins over the click.
    expect(result.current.active).toBe("provider-center");
    rerender({ page: "history" });
    expect(result.current.active).toBe("history");
  });
});
