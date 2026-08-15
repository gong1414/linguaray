import { fireEvent, render, waitFor } from "@solidjs/testing-library";
import { beforeEach, describe, expect, it, vi } from "vitest";

const invokeMock = vi.hoisted(() => vi.fn());
const listenMock = vi.hoisted(() => vi.fn());
const relaunchMock = vi.hoisted(() => vi.fn());

vi.mock("@tauri-apps/api/core", () => ({ invoke: invokeMock }));
vi.mock("@tauri-apps/api/event", () => ({
  listen: listenMock.mockResolvedValue(() => {}),
}));
vi.mock("@tauri-apps/plugin-process", () => ({ relaunch: relaunchMock }));

import UpdaterPanel from "../src/features/settings/UpdaterPanel";

const available = {
  state: "available" as const,
  current: "0.1.0",
  next: "0.2.0",
  notes: "release notes body",
};

/** Captures the listener the panel registered for updater-progress. */
const progressSink = (): ((payload: unknown) => void) => {
  const calls = listenMock.mock.calls;
  const last = calls[calls.length - 1];
  // listen("updater-progress", handler)
  return last[1] as (payload: unknown) => void;
};

describe("UpdaterPanel", () => {
  beforeEach(() => {
    localStorage.setItem("linguaray.locale", "en");
    invokeMock.mockReset().mockImplementation((command: string) => {
      if (command === "updater_check") return Promise.resolve({ state: "up_to_date", version: "0.1.0" });
      if (command === "get_settings")
        return Promise.resolve({
          default_provider: "openai",
          target_language: "zh",
          fallback_engine: null,
          check_updates_on_startup: true,
        });
      if (command === "set_setting") return Promise.resolve();
      return Promise.resolve(null);
    });
    listenMock.mockClear().mockResolvedValue(() => {});
    relaunchMock.mockClear().mockResolvedValue(undefined);
  });

  it("runs a check on mount and reports up-to-date", async () => {
    const { getByTestId, findByText } = render(() => <UpdaterPanel />);
    expect(await findByText("You are on the latest version.")).toBeInTheDocument();
    expect(getByTestId("updater-current-version")).toHaveTextContent("0.1.0");
    expect(invokeMock).toHaveBeenCalledWith("updater_check");
  });

  it("surfaces a failed check as a non-blocking alert", async () => {
    invokeMock.mockImplementation((command: string) => {
      if (command === "updater_check") return Promise.reject(new Error("offline"));
      if (command === "get_settings") return Promise.resolve({ check_updates_on_startup: true });
      return Promise.resolve(null);
    });
    const { findByRole } = render(() => <UpdaterPanel />);
    expect(await findByRole("alert")).toHaveTextContent("Update check failed");
  });

  it("drives download → progress → install → relaunch", async () => {
    let resolveInstall: (v: unknown) => void = () => {};
    invokeMock.mockImplementation((command: string) => {
      if (command === "updater_check") return Promise.resolve(available);
      if (command === "updater_download_install")
        return new Promise((res) => {
          resolveInstall = res;
        });
      if (command === "get_settings") return Promise.resolve({ check_updates_on_startup: true });
      return Promise.resolve(null);
    });

    const { getByTestId } = render(() => <UpdaterPanel />);
    // The panel registers its updater-progress listener via dynamic import —
    // wait for that registration before driving events through it.
    await waitFor(() => expect(listenMock).toHaveBeenCalled());
    const download = await waitFor(() => getByTestId("updater-download"));
    expect(getByTestId("updater-next")).toHaveTextContent("0.2.0");
    fireEvent.click(download);
    await waitFor(() => expect(invokeMock).toHaveBeenCalledWith("updater_download_install"));

    // Halfway through the download the backend reports 50%. The captured
    // handler receives a Tauri Event — the payload rides on `.payload`.
    const sink = progressSink();
    sink({ payload: { downloaded: 5_000_000, total: 10_000_000, bucket: 50 } });
    await waitFor(() => expect(getByTestId("updater-progress")).toHaveTextContent("50%"));
    expect(getByTestId("updater-check-again")).toBeDisabled();

    // Download finished → installer takes over.
    sink({ payload: { finished: true } });
    await waitFor(() => expect(getByTestId("updater-installing")).toBeInTheDocument());

    // The command resolved (macOS path) → relaunch CTA.
    resolveInstall(available);
    const relaunch = await waitFor(() => getByTestId("updater-relaunch"));
    fireEvent.click(relaunch);
    await waitFor(() => expect(relaunchMock).toHaveBeenCalled());
  });

  it("propagates install failures without losing the error", async () => {
    invokeMock.mockImplementation((command: string) => {
      if (command === "updater_check") return Promise.resolve(available);
      if (command === "updater_download_install")
        return Promise.reject(new Error("already in progress"));
      if (command === "get_settings") return Promise.resolve({ check_updates_on_startup: true });
      return Promise.resolve(null);
    });
    const { getByTestId, findByRole } = render(() => <UpdaterPanel />);
    fireEvent.click(await waitFor(() => getByTestId("updater-download")));
    expect(await findByRole("alert")).toHaveTextContent("already in progress");
  });

  it("persists the startup-check toggle and reverts on store failure", async () => {
    const { getByTestId } = render(() => <UpdaterPanel />);
    const box = await waitFor(() => getByTestId("updater-autocheck") as HTMLInputElement);
    expect(box.checked).toBe(true);
    fireEvent.click(box);
    await waitFor(() =>
      expect(invokeMock).toHaveBeenCalledWith("set_setting", {
        key: "check_updates_on_startup",
        value: "false",
      }),
    );

    // A failing store write must revert the checkbox.
    invokeMock.mockImplementation((command: string) => {
      if (command === "updater_check") return Promise.resolve({ state: "up_to_date", version: "0.1.0" });
      if (command === "set_setting") return Promise.reject(new Error("store locked"));
      return Promise.resolve(null);
    });
    fireEvent.click(getByTestId("updater-autocheck"));
    await waitFor(() => expect(getByTestId("updater-autocheck-error")).toBeInTheDocument());
    expect((getByTestId("updater-autocheck") as HTMLInputElement).checked).toBe(true);
  });

  it("reflects a persisted opt-out (unchecked) on mount", async () => {
    invokeMock.mockImplementation((command: string) => {
      if (command === "updater_check") return Promise.resolve({ state: "up_to_date", version: "0.1.0" });
      if (command === "get_settings") return Promise.resolve({ check_updates_on_startup: false });
      return Promise.resolve(null);
    });
    const { getByTestId } = render(() => <UpdaterPanel />);
    const box = await waitFor(() => getByTestId("updater-autocheck") as HTMLInputElement);
    // get_settings resolves after mount — wait for the box to flip unchecked.
    await waitFor(() => expect(box.checked).toBe(false));
  });
});
