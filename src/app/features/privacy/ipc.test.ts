import { describe, expect, it, vi } from "vitest";

// Guard coverage lives in the wrapper itself: mock ONLY the bridge invoke.
const invokeMock = vi.hoisted(() => vi.fn());
vi.mock("../../../bridge/invoke", () => ({ invoke: invokeMock }));

import * as ipc from "./ipc";

describe("privacy ipc guards (fail-closed)", () => {
  it("history_privacy_status with an invalid payload throws", async () => {
    invokeMock.mockResolvedValueOnce({ enabled: "yes" });
    await expect(ipc.historyPrivacyStatus()).rejects.toThrow(/invalid payload/);
  });

  it("history_set_enabled with a valid payload passes through", async () => {
    const ok = { enabled: true, retention_days: 30, record_count: 1 };
    invokeMock.mockResolvedValueOnce(ok);
    await expect(ipc.historySetEnabled(true)).resolves.toEqual(ok);
    expect(invokeMock).toHaveBeenCalledWith("history_set_enabled", { enabled: true });
  });
});
