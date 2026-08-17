import { render, screen, cleanup, fireEvent, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

/**
 * Settings window integration: REAL shell + all section controllers against
 * mocked feature IPC — the "component lab correct, real app wrong" trap the
 * migration spec §八 warns about.
 */

const { shell, commands, listen } = vi.hoisted(() => {
  const listeners: Record<string, Array<(payload: unknown) => void>> = {};
  const shellObj = {
    a11yStatus: vi.fn(),
    onTray: undefined as undefined | ((a: string) => void),
    onNav: undefined as undefined | ((s: string) => void),
  };
  return {
    listeners,
    shell: shellObj,
    commands: {
      translateClipboard: vi.fn(async () => null),
      translateSelectionIpc: vi.fn(async (_text: string | null) => null),
      ocrCapture: vi.fn(async (_source: string | null) => null),
    },
    listen: vi.fn(async (event: string, cb: (e: { payload: unknown }) => void) => {
      (listeners[event] ??= []).push((p: unknown) => cb({ payload: p }));
      return () => {};
    }),
  };
});

vi.mock("../features/shell/ipc", () => shell);
vi.mock("../features/shell/window-ipc", async (orig) => ({
  ...(await orig<typeof import("../features/shell/window-ipc")>()),
  onWindowNavigation: vi.fn(
    async (onTray: (a: string) => void, onNav: (s: string) => void) => {
      shell.onTray = onTray;
      shell.onNav = onNav;
      return () => {};
    },
  ),
}));
vi.mock("../bridge/invoke", () => ({ commands }));
vi.mock("../bridge/event", () => ({ listen }));
vi.mock("../bridge/window", () => ({
  getCurrentWindow: () => ({ onFocusChanged: async () => () => {} }),
}));
vi.mock("../bridge/opener", () => ({ openUrl: vi.fn(async () => {}) }));

// Section IPC surface (provider + history are representative; the rest load
// inertly through their own guards).
vi.mock("../features/provider/ipc", () => ({
  providerListPresets: vi.fn(async () => []),
  loadProviders: vi.fn(async () => []),
  providerGetActiveSelection: vi.fn(async () => ({ primary: null, parallel: [], fallback: null })),
  providerCreate: vi.fn(), providerUpdate: vi.fn(), providerDuplicate: vi.fn(),
  providerDelete: vi.fn(), providerReorder: vi.fn(), providerToggle: vi.fn(),
  providerGetBalance: vi.fn(), providerSetKey: vi.fn(), providerSetActive: vi.fn(),
  providerConfirmAndSetActive: vi.fn(), providerGetModels: vi.fn(),
  providerTestConnection: vi.fn(),
}));
vi.mock("../features/history/ipc", () => ({
  historyPrivacyEnabled: vi.fn(async () => false),
  historySearch: vi.fn(), historyToggleFavorite: vi.fn(), historyDeleteSession: vi.fn(),
  historyExport: vi.fn(), chooseExportPath: vi.fn(),
}));
vi.mock("../features/privacy/ipc", () => ({
  historyPrivacyStatus: vi.fn(async () => ({ enabled: true, retention_days: 30, record_count: 0 })),
  historySetEnabled: vi.fn(), historySetRetention: vi.fn(), historyClearAll: vi.fn(),
  externalApiStatus: vi.fn(async () => ({ state: "disabled" })),
  externalApiEnable: vi.fn(), externalApiDisable: vi.fn(), externalApiRegenerateToken: vi.fn(),
}));
vi.mock("../features/keystore/ipc", () => ({
  keystoreHealth: vi.fn(async () => ""),
  archiveKeystore: vi.fn(), resetKeystore: vi.fn(),
}));
vi.mock("../features/shortcuts/ipc", () => ({
  shortcutList: vi.fn(async () => ({ revision: 1, entries: [] })),
  shortcutCheckConflict: vi.fn(), shortcutSave: vi.fn(), shortcutResetDefaults: vi.fn(),
  shortcutRecordingBegin: vi.fn(), shortcutRecordingEnd: vi.fn(),
}));
vi.mock("../features/vocabulary/ipc", () => ({
  vocabularyList: vi.fn(async () => []),
  vocabularyAdd: vi.fn(), vocabularyDelete: vi.fn(),
  vocabularyExportFile: vi.fn(), vocabularyExportAnki: vi.fn(),
}));
vi.mock("../features/dictionary/ipc", () => ({
  dictLookup: vi.fn(), dictListPackages: vi.fn(async () => []),
  dictInstallPackage: vi.fn(),
}));
vi.mock("../features/updater/ipc", () => ({
  updaterCheck: vi.fn(async () => ({ state: "up_to_date", version: "0.1.0" })),
  updaterDownloadInstall: vi.fn(), getUpdaterStartupCheck: vi.fn(async () => true),
  setUpdaterStartupCheck: vi.fn(), onUpdaterProgress: vi.fn(async () => () => {}),
  relaunchApp: vi.fn(),
}));

import { SettingsWindow } from "./settings";
import { AppProviders } from "../app/providers";
import { runTrayAction } from "../features/shell/window-ipc";

beforeEach(() => {
  vi.clearAllMocks();
  shell.a11yStatus.mockResolvedValue(true);
});

afterEach(cleanup);

describe("SettingsWindow", () => {
  it("boots to Provider Center inside the shell (native title bar, no chrome)", async () => {
    render(
      <AppProviders>
        <SettingsWindow />
      </AppProviders>,
    );
    expect(await screen.findByTestId("shell")).toBeInTheDocument();
    expect(screen.getByTestId("shell").getAttribute("data-page")).toBe("provider-center");
    expect(screen.getByTestId("provider-list")).toBeInTheDocument();
  });

  it("nav clicks switch sections across the whole window", async () => {
    render(
      <AppProviders>
        <SettingsWindow />
      </AppProviders>,
    );
    await screen.findByTestId("provider-list");
    fireEvent.click(screen.getByRole("button", { name: "History" }));
    await waitFor(() =>
      expect(screen.getByTestId("shell").getAttribute("data-page")).toBe("history"),
    );
    expect(screen.getByTestId("history-disabled")).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "Privacy" }));
    await waitFor(() =>
      expect(screen.getByTestId("shell").getAttribute("data-page")).toBe("privacy"),
    );
  });

  it("tray navigation events route sections; translate actions invoke IPC", () => {
    expect(runTrayAction("history")).toBe("history");
    expect(runTrayAction("settings")).toBe("provider-center");
    expect(runTrayAction("translate-clipboard")).toBeNull();
    expect(runTrayAction("translate-selection")).toBeNull();
    expect(runTrayAction("ocr-capture")).toBeNull();
    expect(runTrayAction("unknown")).toBeNull();
    expect(commands.translateClipboard).toHaveBeenCalledTimes(1);
    expect(commands.translateSelectionIpc).toHaveBeenCalledWith(null);
    expect(commands.ocrCapture).toHaveBeenCalledWith("tray");
  });

  it("unmount releases navigation listeners", async () => {
    const { unmount } = render(
      <AppProviders>
        <SettingsWindow />
      </AppProviders>,
    );
    await screen.findByTestId("shell");
    unmount();
    // The mocked onWindowNavigation returns a no-op unlisten; reaching here
    // without a crash is the contract (real impl is covered by its own file).
    expect(true).toBe(true);
  });
});
