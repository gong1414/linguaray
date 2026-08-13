import { cleanup, render, waitFor } from "@solidjs/testing-library";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { assertNoAxeViolations } from "./axe";

const invokeMock = vi.hoisted(() => vi.fn());
vi.mock("@tauri-apps/api/core", () => ({ invoke: invokeMock }));

import PrivacyData from "../src/features/settings/PrivacyData";

beforeEach(() => {
  localStorage.setItem("linguaray.locale", "en");
  document.documentElement.dataset.theme = "light";
  invokeMock.mockReset().mockResolvedValue({
    enabled: false,
    retention_days: 30,
    record_count: 4,
  });
});

afterEach(() => cleanup());

describe("PrivacyData — accessibility", () => {
  it("has no axe violations while history is disabled and retained data can be cleared", async () => {
    const { getByRole } = render(() => <PrivacyData />);
    await waitFor(() => expect(getByRole("button", { name: "Clear All" })).toBeTruthy());
    await assertNoAxeViolations({ disableRules: ["color-contrast"] });
  });

  it("has no axe violations in dark + Chinese", async () => {
    localStorage.setItem("linguaray.locale", "zh");
    document.documentElement.dataset.theme = "dark";
    invokeMock.mockResolvedValue({ enabled: true, retention_days: 90, record_count: 1 });
    const { getByRole } = render(() => <PrivacyData />);
    await waitFor(() => expect(getByRole("heading", { name: "隐私与数据" })).toBeTruthy());
    await assertNoAxeViolations({ disableRules: ["color-contrast"] });
  });
});
