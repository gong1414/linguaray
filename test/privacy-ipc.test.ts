import { beforeEach, describe, expect, it, vi } from "vitest";

const invokeMock = vi.hoisted(() => vi.fn());
vi.mock("@tauri-apps/api/core", () => ({ invoke: invokeMock }));

import {
  historyClearAll,
  historyPrivacyStatus,
  historySetEnabled,
  historySetRetention,
} from "../src/features/settings/privacy-ipc";

const status = { enabled: false, retention_days: 30, record_count: 2 };

describe("privacy IPC", () => {
  beforeEach(() => invokeMock.mockReset().mockResolvedValue(status));

  it("routes typed commands and arguments", async () => {
    await expect(historyPrivacyStatus()).resolves.toEqual(status);
    await historySetEnabled(true);
    await historySetRetention(90);
    await historyClearAll();
    expect(invokeMock.mock.calls).toEqual([
      ["history_privacy_status"],
      ["history_set_enabled", { enabled: true }],
      ["history_set_retention", { days: 90 }],
      ["history_clear_all"],
    ]);
  });

  it("fails closed on an invalid payload", async () => {
    invokeMock.mockResolvedValue({ enabled: "yes", retention_days: 7, record_count: -1 });
    await expect(historyPrivacyStatus()).rejects.toThrow("invalid payload");
  });
});
