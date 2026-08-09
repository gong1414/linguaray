import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { render, cleanup } from "@solidjs/testing-library";

const { invokeMock, listenMock, unlistenMock } = vi.hoisted(() => ({
  invokeMock: vi.fn(async (_cmd: string, _args?: unknown): Promise<unknown> => undefined),
  listenMock: vi.fn(async (_event: string, _cb: (e: { payload: unknown }) => void) => () => {}),
  unlistenMock: vi.fn(() => {}),
}));

vi.mock("@tauri-apps/api/event", () => ({
  listen: listenMock,
}));
vi.mock("@tauri-apps/api/core", () => ({ invoke: invokeMock }));
vi.mock("@tauri-apps/plugin-opener", () => ({ openUrl: vi.fn() }));

vi.mock("../src/features/settings/SettingsShell", () => ({
  default: (props: { activePage?: string; children: any }) => (
    <div data-testid="shell" data-page={props.activePage}>{props.children}</div>
  ),
}));
vi.mock("../src/features/settings/ProviderCenter", () => ({
  default: () => <div data-testid="provider-center" />,
}));
vi.mock("../src/features/settings/KeystoreRecovery", () => ({
  default: () => <div data-testid="keystore-recovery" />,
}));

import App from "../src/App";

beforeEach(() => {
  invokeMock.mockReset();
  invokeMock.mockResolvedValue(undefined);
  listenMock.mockReset();
  listenMock.mockResolvedValue(unlistenMock);
});

afterEach(() => cleanup());

async function getHandler(event: string): Promise<(e: { payload: string }) => void> {
  const call = listenMock.mock.calls.find((c) => c[0] === event);
  if (!call) throw new Error(`no listen("${event}") registered; calls: ${listenMock.mock.calls.map((c) => c[0]).join(",")}`);
  return call[1] as (e: { payload: string }) => void;
}

describe("App tray-action + navigate listeners", () => {
  // onMount is async (two sequential `await listen` calls); flush microtasks so
  // BOTH listeners register before assertions.
  async function flushListeners() {
    await Promise.resolve();
    await Promise.resolve();
  }

  it("registers tray-action and navigate listeners on mount", async () => {
    render(() => <App />);
    await flushListeners();
    const events = listenMock.mock.calls.map((c) => c[0]);
    expect(events).toContain("tray-action");
    expect(events).toContain("navigate");
  });

  it("translate-clipboard action invokes translate_clipboard", async () => {
    render(() => <App />);
    await flushListeners();
    const handler = await getHandler("tray-action");
    handler({ payload: "translate-clipboard" });
    await Promise.resolve();
    expect(invokeMock.mock.calls.some((c) => c[0] === "translate_clipboard")).toBe(true);
  });

  it("translate-selection action calls translate_selection_ipc, NOT translate_clipboard", async () => {
    render(() => <App />);
    await flushListeners();
    const handler = await getHandler("tray-action");
    handler({ payload: "translate-selection" });
    await Promise.resolve();
    expect(invokeMock.mock.calls.some((c) => c[0] === "translate_selection_ipc")).toBe(true);
    expect(invokeMock.mock.calls.some((c) => c[0] === "translate_clipboard")).toBe(false);
  });

  it("switch-provider action opens settings on the provider page", async () => {
    const { findByTestId } = render(() => <App />);
    await flushListeners();
    const handler = await getHandler("tray-action");
    handler({ payload: "switch-provider" });
    await Promise.resolve();
    const shell = await findByTestId("shell");
    expect(shell.getAttribute("data-page")).toBe("provider-center");
  });

  it("navigate event sets the active page on the shell", async () => {
    const { findByTestId } = render(() => <App />);
    await flushListeners();
    const navHandler = await getHandler("navigate");
    navHandler({ payload: "keystore-recovery" });
    await Promise.resolve();
    const shell = await findByTestId("shell");
    expect(shell.getAttribute("data-page")).toBe("keystore-recovery");
  });
});
