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
vi.mock("@tauri-apps/plugin-clipboard-manager", () => ({
  writeText: vi.fn(async () => undefined),
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

  it("Copy action flips to Copied feedback for 1.2s via Tauri clipboard", async () => {
    const { writeText } = await import("@tauri-apps/plugin-clipboard-manager");
    const { findByLabelText, findByText } = render(() => <Popup />);
    await emitEvent("popup-state", {
      status: "result",
      text: "你好",
      engine: "deepseek/u1",
      source_text: "hello",
    });
    const copyBtn = await findByLabelText(/复制|Copy/);
    await fireEvent.click(copyBtn);
    // Flips to Copied feedback label.
    expect(await findByText(/已复制|Copied/)).toBeTruthy();
    // Writes the TRANSLATION text (not the source).
    expect(vi.mocked(writeText)).toHaveBeenCalledWith("你好");
    expect(vi.mocked(writeText)).not.toHaveBeenCalledWith("hello");
    cleanup();
  });

  it("Retry reuses the saved SOURCE text, not the translation result and not the clipboard", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    vi.mocked(invoke).mockClear();
    const { findByLabelText } = render(() => <Popup />);
    await emitEvent("popup-state", {
      status: "result",
      text: "你好",
      engine: "deepseek/u1",
      source_text: "hello",
    });
    const retryBtn = await findByLabelText(/重试|Retry/);
    await fireEvent.click(retryBtn);
    expect(vi.mocked(invoke)).toHaveBeenCalledWith("translate_selection_ipc", {
      text: "hello",
    });
    expect(vi.mocked(invoke)).not.toHaveBeenCalledWith("translate_clipboard");
    cleanup();
  });

  it("Retry is available in the error state because the error payload carries source_text (P1-3)", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    vi.mocked(invoke).mockClear();
    const { findByLabelText } = render(() => <Popup />);
    await emitEvent("popup-state", {
      status: "error",
      text: "network timeout",
      engine: "",
      source_text: "hello",
    });
    const retryBtn = await findByLabelText(/重试|Retry/);
    await fireEvent.click(retryBtn);
    expect(vi.mocked(invoke)).toHaveBeenCalledWith("translate_selection_ipc", {
      text: "hello",
    });
    cleanup();
  });

  it("Retry is hidden when there is no source text (P1-3)", async () => {
    const { queryByLabelText } = render(() => <Popup />);
    await emitEvent("popup-state", {
      status: "error",
      text: "network timeout",
      engine: "",
    });
    expect(queryByLabelText(/重试|Retry/)).toBeNull();
    cleanup();
  });

  it("Retry for a multi-result reuses the joined SOURCE text", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    vi.mocked(invoke).mockClear();
    const { findByLabelText } = render(() => <Popup />);
    await emitEvent("popup-multi-result", {
      outcomes: [
        { uuid: "u1", ok: true, text: "你好", engine: "deepseek/u1" },
        { uuid: "u2", ok: true, text: "hello", engine: "openai/u2" },
      ],
      source_text: "hello world",
    });
    const retryBtn = await findByLabelText(/重试|Retry/);
    await fireEvent.click(retryBtn);
    expect(vi.mocked(invoke)).toHaveBeenCalledWith("translate_selection_ipc", {
      text: "hello world",
    });
    cleanup();
  });

  it("config-401 error offers a settings navigation button", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    vi.mocked(invoke).mockClear();
    const { findByText } = render(() => <Popup />);
    await emitEvent("popup-state", {
      status: "error",
      text: "401 Unauthorized",
      engine: "",
      source_text: "hello",
    });
    const settingsBtn = await findByText(/打开设置|Open Settings/);
    await fireEvent.click(settingsBtn);
    expect(vi.mocked(invoke)).toHaveBeenCalledWith("open_settings_window", {
      section: "provider-center",
    });
    cleanup();
  });

  it("TTS and Favorite are aria-disabled but focusable (not native disabled)", async () => {
    const { findByLabelText } = render(() => <Popup />);
    await emitEvent("popup-state", {
      status: "result",
      text: "你好",
      engine: "deepseek/u1",
      source_text: "hello",
    });
    const speak = await findByLabelText(/朗读|Speak/);
    const favorite = await findByLabelText(/收藏|Favorite/);
    for (const btn of [speak, favorite]) {
      expect(btn.getAttribute("aria-disabled")).toBe("true");
      expect(btn.hasAttribute("disabled")).toBe(false);
    }
    cleanup();
  });

  it("rev-5-7: clipboard-origin result carries source_text so Retry re-translates via translate_selection_ipc (NOT translate_clipboard)", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    vi.mocked(invoke).mockClear();
    const { findByLabelText } = render(() => <Popup />);
    await emitEvent("popup-state", {
      status: "result",
      text: "你好",
      engine: "deepseek/u1",
      source_text: "clipboard text here",
    });
    const retryBtn = await findByLabelText(/重试|Retry/);
    await fireEvent.click(retryBtn);
    expect(vi.mocked(invoke)).toHaveBeenCalledWith("translate_selection_ipc", {
      text: "clipboard text here",
    });
    expect(vi.mocked(invoke)).not.toHaveBeenCalledWith("translate_clipboard");
    cleanup();
  });
});
