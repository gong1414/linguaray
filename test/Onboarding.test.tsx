import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { render, fireEvent, cleanup, waitFor } from "@solidjs/testing-library";

const { invokeMock, openUrlMock } = vi.hoisted(() => ({
  invokeMock: vi.fn(async (cmd: string, args?: { step?: string; event?: string }) => {
    if (cmd === "onboarding_next") {
      const order = ["welcome", "accessibility", "provider", "history", "shortcuts", "done"];
      const i = order.indexOf(args?.step ?? "welcome");
      return order[Math.min(i + 1, order.length - 1)];
    }
    return undefined;
  }),
  openUrlMock: vi.fn(async () => undefined),
}));

vi.mock("@tauri-apps/api/core", () => ({ invoke: invokeMock }));
vi.mock("@tauri-apps/api/window", () => ({
  getCurrentWindow: () => ({ hide: vi.fn(async () => undefined) }),
}));
vi.mock("@tauri-apps/plugin-opener", () => ({ openUrl: openUrlMock }));

import Onboarding from "../src/Onboarding";

beforeEach(() => {
  localStorage.setItem("linguaray.locale", "en");
  invokeMock.mockClear();
  openUrlMock.mockClear();
});
afterEach(() => cleanup());

describe("Onboarding", () => {
  it("Open System Settings opens Accessibility via plugin-opener, not a11y_status", async () => {
    const { getByText } = render(() => <Onboarding />);
    fireEvent.click(getByText("Get started"));
    await waitFor(() => expect(getByText("Open System Settings")).toBeTruthy());
    fireEvent.click(getByText("Open System Settings"));
    await waitFor(() =>
      expect(openUrlMock).toHaveBeenCalledWith(
        "x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility",
      ),
    );
    expect(invokeMock.mock.calls.some((c) => c[0] === "a11y_status")).toBe(false);
  });
});
