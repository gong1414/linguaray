import { render, fireEvent, waitFor } from "@solidjs/testing-library";
import { beforeEach, describe, expect, it, vi } from "vitest";

const invokeMock = vi.hoisted(() => vi.fn());
vi.mock("@tauri-apps/api/core", () => ({ invoke: invokeMock }));

import PrivacyData, { PrivacyDataView } from "../src/features/settings/PrivacyData";

const off = { enabled: false, retention_days: 30 as const, record_count: 3 };

describe("PrivacyData", () => {
  beforeEach(() => {
    localStorage.setItem("linguaray.locale", "en");
    invokeMock.mockReset().mockImplementation((command: string) => {
      if (command === "history_privacy_status") return Promise.resolve(off);
      if (command === "history_set_enabled") return Promise.resolve({ ...off, enabled: true });
      if (command === "history_clear_all") return Promise.resolve({ ...off, record_count: 0 });
      return Promise.resolve(off);
    });
  });

  it("keeps Clear All available while history is disabled", async () => {
    const { getByRole } = render(() => <PrivacyData />);
    const clear = await waitFor(() => getByRole("button", { name: "Clear All" }));
    expect(clear).not.toBeDisabled();
    expect(getByRole("switch", { name: "Enable history" })).toHaveAttribute("aria-checked", "false");
  });

  it("enables history through the typed command", async () => {
    const { getByRole } = render(() => <PrivacyData />);
    const toggle = await waitFor(() => getByRole("switch", { name: "Enable history" }));
    fireEvent.click(toggle);
    await waitFor(() => expect(invokeMock).toHaveBeenCalledWith("history_set_enabled", { enabled: true }));
  });

  it("shows an explicit load failure with retry", async () => {
    invokeMock.mockRejectedValueOnce(new Error("offline"));
    const { findByRole, getByRole } = render(() => <PrivacyData />);
    expect(await findByRole("alert")).toHaveTextContent("Privacy settings could not be loaded");
    fireEvent.click(getByRole("button", { name: "Retry" }));
    await waitFor(() => expect(invokeMock).toHaveBeenCalledTimes(2));
  });

  it("renders the frozen retention and shared External API boundary", () => {
    const { getByText, getByRole } = render(() => (
      <PrivacyDataView
        status={{ enabled: true, retention_days: 90, record_count: 1 }}
        loading={false}
        error={null}
        busy={null}
        clearOpen={false}
        toasts={[]}
        onRetry={() => {}}
        onEnabledChange={() => {}}
        onRetentionChange={() => {}}
        onOpenClear={() => {}}
        onCloseClear={() => {}}
        onConfirmClear={() => {}}
        onDismissToast={() => {}}
      />
    ));
    expect(getByText("90 days")).toBeInTheDocument();
    expect(getByRole("heading", { name: "External API" })).toBeInTheDocument();
  });
});
