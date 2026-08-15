import { act, cleanup, renderHook, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

const { ipc, writeTextMock } = vi.hoisted(() => ({
  writeTextMock: vi.fn(),
  ipc: {
    historyPrivacyStatus: vi.fn(),
    historySetEnabled: vi.fn(),
    historySetRetention: vi.fn(),
    historyClearAll: vi.fn(),
    externalApiStatus: vi.fn(),
    externalApiEnable: vi.fn(),
    externalApiDisable: vi.fn(),
    externalApiRegenerateToken: vi.fn(),
  },
}));

vi.mock("./ipc", () => ipc);
vi.mock("../../../bridge/clipboard", () => ({ writeText: writeTextMock }));

import { usePrivacyController } from "./controller";

const OK = { enabled: true, retention_days: 30, record_count: 7 };

beforeEach(() => {
  vi.clearAllMocks();
  writeTextMock.mockResolvedValue(undefined);
  ipc.historyPrivacyStatus.mockResolvedValue(OK);
  ipc.historySetEnabled.mockResolvedValue({ ...OK, enabled: false });
  ipc.historySetRetention.mockResolvedValue({ ...OK, retention_days: 90 });
  ipc.historyClearAll.mockResolvedValue({ ...OK, record_count: 0 });
  ipc.externalApiStatus.mockResolvedValue({ state: "disabled" });
  ipc.externalApiEnable.mockResolvedValue("lray_token_1");
  ipc.externalApiDisable.mockResolvedValue(undefined);
  ipc.externalApiRegenerateToken.mockResolvedValue("lray_token_2");
});

afterEach(cleanup);

describe("usePrivacyController (controller + ipc integration)", () => {
  it("loads privacy status and external status on mount", async () => {
    const { result } = renderHook(() => usePrivacyController());
    await waitFor(() => expect(result.current.loading).toBe(false));
    expect(result.current.status).toEqual(OK);
    expect(result.current.external).toEqual({ state: "disabled" });
  });

  it("load failure surfaces the error message", async () => {
    ipc.historyPrivacyStatus.mockRejectedValue(new Error("no db"));
    const { result } = renderHook(() => usePrivacyController());
    await waitFor(() => expect(result.current.error).toBe("no db"));
  });

  it("setEnabled / setRetention adopt the returned status", async () => {
    const { result } = renderHook(() => usePrivacyController());
    await waitFor(() => expect(result.current.loading).toBe(false));
    await act(async () => {
      result.current.setEnabled(false);
    });
    expect(ipc.historySetEnabled).toHaveBeenCalledWith(false);
    await waitFor(() => expect(result.current.status?.enabled).toBe(false));

    await act(async () => {
      result.current.setRetention(90);
    });
    expect(ipc.historySetRetention).toHaveBeenCalledWith(90);
    await waitFor(() => expect(result.current.status?.retention_days).toBe(90));
  });

  it("a failed mutation pushes a localized destructive toast and unlocks busy", async () => {
    ipc.historySetEnabled.mockRejectedValue(new Error("write failed"));
    const { result } = renderHook(() => usePrivacyController());
    await waitFor(() => expect(result.current.loading).toBe(false));
    await act(async () => {
      result.current.setEnabled(true);
    });
    await waitFor(() => {
      expect(result.current.toasts).toHaveLength(1);
      expect(result.current.toasts[0].variant).toBe("danger");
    });
    expect(result.current.busy).toBeNull();
    expect(result.current.dismissToast).toBeTypeOf("function");
  });

  it("confirmClear closes the dialog, clears, and toasts success", async () => {
    const { result } = renderHook(() => usePrivacyController());
    await waitFor(() => expect(result.current.loading).toBe(false));
    act(() => {
      result.current.openClear();
    });
    await act(async () => {
      result.current.confirmClear();
    });
    expect(ipc.historyClearAll).toHaveBeenCalledTimes(1);
    await waitFor(() => {
      expect(result.current.clearOpen).toBe(false);
      expect(result.current.status?.record_count).toBe(0);
    });
    expect(result.current.toasts.some((x) => x.variant === "success")).toBe(true);
  });

  it("enabling the external API stores the one-time token and refreshes status", async () => {
    ipc.externalApiStatus
      .mockResolvedValueOnce({ state: "disabled" })
      .mockResolvedValueOnce({ state: "enabled", port: 8787 });
    const { result } = renderHook(() => usePrivacyController());
    await waitFor(() => expect(result.current.external).toEqual({ state: "disabled" }));
    await act(async () => {
      result.current.enableExternal();
    });
    await waitFor(() => expect(result.current.tokenOnce).toBe("lray_token_1"));
    await waitFor(() => expect(result.current.external?.state).toBe("enabled"));
  });

  it("copyToken writes the token to the clipboard", async () => {
    ipc.externalApiEnable.mockResolvedValue("lray_token_1");
    const { result } = renderHook(() => usePrivacyController());
    await act(async () => {
      result.current.enableExternal();
    });
    await waitFor(() => expect(result.current.tokenOnce).toBe("lray_token_1"));
    await act(async () => {
      result.current.copyToken();
    });
    expect(writeTextMock).toHaveBeenCalledWith("lray_token_1");
    await waitFor(() => expect(result.current.tokenCopied).toBe(true));
  });
});
