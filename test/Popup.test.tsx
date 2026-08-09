import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, fireEvent, cleanup } from "@solidjs/testing-library";
import Popup from "../src/Popup";

// Stub Tauri event + window APIs at the module the controller imports.
vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn(async () => () => {}),
}));
vi.mock("@tauri-apps/api/window", () => {
  // Cache a single window instance so the same `hide` mock is shared between
  // the controller and the test assertion (getCurrentWindow() otherwise
  // returns a fresh object per call).
  const win = {
    onFocusChanged: vi.fn(async () => () => {}),
    hide: vi.fn(async () => {}),
    setFocus: vi.fn(async () => {}),
  };
  return { getCurrentWindow: () => win };
});
vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(async () => ({ outcomes: [], actual_engine: undefined })),
}));

// Helper: emit a decoded state by reaching into the listen mock.
async function emitEvent(name: string, payload: unknown) {
  const { listen } = await import("@tauri-apps/api/event");
  const calls = vi.mocked(listen).mock.calls;
  // Find the most recent listener registered for this event name.
  for (let i = calls.length - 1; i >= 0; i--) {
    if (calls[i][0] === name) {
      const handler = calls[i][1] as (e: { payload: unknown }) => void;
      handler({ payload });
      return;
    }
  }
}

beforeEach(() => {
  vi.clearAllMocks();
});

describe("Popup (Surface 01)", () => {
  it("renders loading spinner before any event", () => {
    const { getByRole, container } = render(() => <Popup />);
    // Spinner renders an svg.lr-spinner__icon + visually-hidden label.
    expect(container.querySelector(".lr-spinner__icon")).toBeTruthy();
    expect(getByRole("region")).toBeTruthy();
    cleanup();
  });

  it("renders single-success ResultCard on popup-state result", async () => {
    const { findByText, getByRole } = render(() => <Popup />);
    await emitEvent("popup-state", { status: "result", text: "你好", engine: "deepseek/u1" });
    expect(await findByText("你好")).toBeTruthy();
    // The region's aria-label should reflect a success state, not loading.
    const region = getByRole("region");
    expect(region.getAttribute("aria-busy")).toBeFalsy();
    cleanup();
  });

  it("renders error EmptyState on popup-state network error", async () => {
    const { findByText } = render(() => <Popup />);
    await emitEvent("popup-state", { status: "error", text: "network timeout", engine: "" });
    // The zh/en copy "网络错误"/"Network error" should appear.
    const alert = await findByText(/网络错误|Network error/);
    expect(alert).toBeTruthy();
    cleanup();
  });

  it("renders multi-success on popup-multi-result with two ok outcomes", async () => {
    const { findAllByText } = render(() => <Popup />);
    await emitEvent("popup-multi-result", {
      outcomes: [
        { uuid: "u1", ok: true, text: "你好", engine: "deepseek/u1" },
        { uuid: "u2", ok: true, text: "hello", engine: "openai/u2" },
      ],
    });
    const cards = await findAllByText(/你好|hello/);
    expect(cards.length).toBeGreaterThanOrEqual(2);
    cleanup();
  });

  it("hides the window on Escape", async () => {
    const { getCurrentWindow } = await import("@tauri-apps/api/window");
    const { container } = render(() => <Popup />);
    fireEvent.keyDown(container.querySelector("main") ?? document.body, { key: "Escape" });
    expect(vi.mocked(getCurrentWindow().hide)).toHaveBeenCalled();
    cleanup();
  });

  it("renders friendly engine label, not secret_ref/uuid", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    vi.mocked(invoke).mockImplementation(async (cmd: string) => {
      if (cmd === "provider_list") {
        return [
          { uuid: "u1", name: "My OpenAI", secret_ref: "provider/u1", template_id: "openai", protocol: "openai_chat", endpoint: "", model: null, enabled: true, sort_order: 0, is_local: false, needs_key: true, status: "active", capabilities: { balance: false, quota: false, model_list: false } },
        ];
      }
      return { outcomes: [], actual_engine: undefined };
    });

    const { findByText } = render(() => <Popup />);
    // Flush the mount-time provider_list so the name map resolves before the
    // popup-state event fires.
    await Promise.resolve();
    await Promise.resolve();
    await emitEvent("popup-state", { status: "result", text: "你好", engine: "provider/u1", source_text: "hello" });
    expect(await findByText("My OpenAI")).toBeTruthy();
    expect(document.body.textContent).not.toContain("provider/u1");
    cleanup();
  });
});
