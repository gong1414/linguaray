/**
 * App mount smoke test — asserts the legacy settings/translate window has been
 * fully replaced by the SettingsShell + Provider Center mount, and that no
 * legacy elements (textarea, settings-group, translate_clipboard) remain.
 */
import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { render, fireEvent, cleanup, screen, waitFor } from "@solidjs/testing-library";

const { invokeMock, listenMock } = vi.hoisted(() => ({
  invokeMock: vi.fn(async (_cmd: string, _args?: unknown): Promise<unknown> => undefined),
  // A4: App.onMount registers tray-action + navigate listeners. Mock the event
  // API so the real Tauri bridge is never touched in jsdom (would throw).
  listenMock: vi.fn(async (_event: string, _cb: (e: { payload: unknown }) => void) => () => {}),
}));

vi.mock("@tauri-apps/api/core", () => ({ invoke: invokeMock }));
vi.mock("@tauri-apps/api/event", () => ({ listen: listenMock }));

import App from "../src/App";

const flush = () => new Promise((r) => setTimeout(r, 0));

beforeEach(() => {
  invokeMock.mockReset();
  invokeMock.mockImplementation(async (cmd: string) => {
    if (cmd === "provider_list") return [];
    if (cmd === "key_status") return ({});
    if (cmd === "keystore_health") return "";
    if (cmd === "shortcut_list") return { revision: 1, entries: [] };
    if (cmd === "history_privacy_status") return { enabled: false, retention_days: 30, record_count: 0 };
    return undefined;
  });
});

afterEach(() => cleanup());

describe("App mount (R3a)", () => {
  it("renders SettingsShell with Provider Center active by default", async () => {
    render(() => <App />);
    // Window title.
    await waitFor(() => expect(screen.getByText("LinguaRay")).toBeTruthy());
    // Provider Center nav button has aria-current="page".
    const pcBtn = screen.getByText("Provider Center").closest("button")!;
    expect(pcBtn.getAttribute("aria-current")).toBe("page");
  });

  it("clicking Keystore Recovery nav swaps content to KeystoreRecovery", async () => {
    render(() => <App />);
    await waitFor(() => expect(screen.getByText("LinguaRay")).toBeTruthy());
    // Click Keystore Recovery nav.
    fireEvent.click(screen.getByText("Keystore Recovery").closest("button")!);
    await flush();
    // KeystoreRecovery renders its section (keystore_health "" = healthy → no
    // banner, but the surface is mounted). Assert aria-current moved.
    const ksBtn = screen.getByText("Keystore Recovery").closest("button")!;
    expect(ksBtn.getAttribute("aria-current")).toBe("page");
  });

  it("mounts the live Shortcuts and Privacy destinations", async () => {
    render(() => <App />);
    await waitFor(() => expect(screen.getByText("LinguaRay")).toBeTruthy());
    fireEvent.click(screen.getByText("Shortcuts").closest("button")!);
    await waitFor(() => expect(screen.getByRole("heading", { name: "Keyboard Shortcuts" })).toBeTruthy());
    fireEvent.click(screen.getByText("Privacy").closest("button")!);
    await waitFor(() => expect(screen.getByRole("heading", { name: "Privacy & Data" })).toBeTruthy());
  });

  it("no legacy elements remain", async () => {
    const { container } = render(() => <App />);
    await flush();
    // No <textarea> (legacy translate box).
    expect(container.querySelector("textarea")).toBeNull();
    // No .settings-group (legacy class).
    expect(container.querySelector(".settings-group")).toBeNull();
    // No "Translate clipboard" button text.
    expect(screen.queryByText(/Translate clipboard/i)).toBeNull();
  });

  it("does not call translate/translate_clipboard on mount", async () => {
    render(() => <App />);
    await flush();
    const cmds = invokeMock.mock.calls.map((c) => c[0]);
    // Only provider/keystore commands should be invoked.
    expect(cmds).not.toContain("translate");
    expect(cmds).not.toContain("translate_clipboard");
    expect(cmds).not.toContain("list_engines");
  });
});
