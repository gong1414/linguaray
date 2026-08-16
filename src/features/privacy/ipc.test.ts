import { describe, expect, it, vi } from "vitest";

// Guard coverage lives in the wrapper itself: mock only generated commands.
const { historyPrivacyStatusMock, historySetEnabledMock, commands } = vi.hoisted(() => {
  const historyPrivacyStatusMock = vi.fn();
  const historySetEnabledMock = vi.fn();
  return {
    historyPrivacyStatusMock,
    historySetEnabledMock,
    commands: {
      historyPrivacyStatus: historyPrivacyStatusMock,
      historySetEnabled: historySetEnabledMock,
    },
  };
});
vi.mock("../../bridge/invoke", () => ({ commands }));

import * as ipc from "./ipc";

describe("privacy ipc guards (fail-closed)", () => {
  it("history_privacy_status with an invalid payload throws", async () => {
    historyPrivacyStatusMock.mockResolvedValueOnce({ enabled: "yes" });
    await expect(ipc.historyPrivacyStatus()).rejects.toThrow(/invalid payload/);
  });

  it("history_set_enabled with a valid payload passes through", async () => {
    const ok = { enabled: true, retention_days: 30, record_count: 1 };
    historySetEnabledMock.mockResolvedValueOnce(ok);
    await expect(ipc.historySetEnabled(true)).resolves.toEqual(ok);
    expect(historySetEnabledMock).toHaveBeenCalledWith(true);
  });
});
