import { render, screen, fireEvent, cleanup, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { AppProviders } from "../../app/providers";
import { PopupView } from "./PopupView";
import { usePopupController } from "./popupController";
import type { PopupMultiPayload, PopupStatePayload } from "./types";

const { listeners, emit, hideMock, commands, listen, writeText, tts, settings, vocab } = vi.hoisted(() => {
  const listeners: Record<string, Array<(payload: unknown) => void>> = {};
  const emit = (event: string, payload: unknown) => {
    for (const cb of [...(listeners[event] ?? [])]) cb(payload);
  };
  const hideMock = vi.fn();
  return {
    listeners,
    emit,
    hideMock,
    commands: {
      providerList: vi.fn(async () => [{ uuid: "u1", name: "MyOpenAI" }]),
    },
    listen: vi.fn(async (event: string, cb: (e: { payload: unknown }) => void) => {
      (listeners[event] ??= []).push((p: unknown) => cb({ payload: p }));
      return () => {};
    }),
    writeText: vi.fn(async () => {}),
    tts: { speak: vi.fn(async () => {}), stop: vi.fn(async () => {}) },
    settings: { open: vi.fn(async () => {}) },
    vocab: { add: vi.fn(async () => {}) },
  };
});

vi.mock("../../bridge/invoke", () => ({ commands }));
vi.mock("../../bridge/event", () => ({ listen }));
vi.mock("../../bridge/window", () => ({
  getCurrentWindow: () => ({
    onFocusChanged: async () => () => {},
    hide: hideMock,
  }),
}));
vi.mock("../../bridge/clipboard", () => ({ writeText }));
vi.mock("./popup-ipc", () => ({
  translateSelection: vi.fn(async () => {}),
  ttsSpeak: tts.speak,
  ttsStop: tts.stop,
  openSettingsWindow: settings.open,
}));
vi.mock("./input-ipc", () => ({
  translateSession: vi.fn(),
  addVocabulary: vocab.add,
}));

import { translateSelection } from "./popup-ipc";
import { resetProviderNameMap } from "./providerNames";

function Live() {
  const c = usePopupController();
  return <PopupView c={c} />;
}

const renderLive = () => render(<Live />, { wrapper: AppProviders });

beforeEach(() => {
  vi.clearAllMocks();
  // The provider-name map is a shared module-level cache now — reset it so
  // each test's provider_list fixture populates a fresh map.
  resetProviderNameMap();
  // Drop listeners captured by previous tests so events only reach this one.
  for (const key of Object.keys(listeners)) listeners[key].length = 0;
});

afterEach(cleanup);

const state = (p: Partial<PopupStatePayload>) => p as PopupStatePayload;

describe("popup (controller + view integration)", () => {
  it("renders loading by default and adopts source_text from a loading event", async () => {
    renderLive();
    expect(screen.getByTestId("popup-loading")).toBeInTheDocument();
    emit("popup-state", state({ status: "loading", text: "", engine: "", source_text: "原文" }));
    await waitFor(() => expect(screen.getByRole("button", { name: "Retry" })).toBeInTheDocument());
  });

  it("single result renders with the friendly provider label", async () => {
    renderLive();
    emit("popup-state", state({ status: "result", text: "你好", engine: "provider/u1" }));
    expect(await screen.findByText("你好")).toBeInTheDocument();
    await waitFor(() => expect(screen.getByText("MyOpenAI")).toBeInTheDocument());
  });

  it("multi-result renders per-engine cards; failures show their error", async () => {
    renderLive();
    const payload: PopupMultiPayload = {
      outcomes: [
        { uuid: "u1", ok: true, text: "a", engine: "provider/u1" },
        { uuid: "u2", ok: false, error: "timeout", engine: "provider/u2" },
      ],
      source_text: "src",
    };
    emit("popup-multi-result", payload);
    const cards = await screen.findAllByTestId("popup-card");
    expect(cards).toHaveLength(2);
    expect(screen.getByText("timeout")).toBeInTheDocument();
  });

  it("Escape dismisses (unpins + hides the window)", async () => {
    renderLive();
    emit("popup-state", state({ status: "result", text: "x", engine: "e" }));
    await screen.findAllByTestId("popup-card");
    fireEvent.keyDown(screen.getByTestId("popup-shell"), { key: "Escape" });
    await waitFor(() => expect(hideMock).toHaveBeenCalledTimes(1));
  });

  it("Copy writes the TRANSLATION (never the source) and flashes Copied", async () => {
    renderLive();
    emit("popup-state", state({ status: "result", text: "trans", engine: "e", source_text: "src" }));
    await screen.findAllByTestId("popup-card");
    fireEvent.click(screen.getByRole("button", { name: "Copy" }));
    await waitFor(() => expect(writeText).toHaveBeenCalledWith("trans"));
    await waitFor(() =>
      expect(screen.getByRole("button", { name: "Copied" })).toBeInTheDocument(),
    );
  });

  it("favorite saves source→translation into vocabulary", async () => {
    renderLive();
    emit("popup-state", state({ status: "result", text: "trans", engine: "e", source_text: "原文" }));
    await screen.findAllByTestId("popup-card");
    fireEvent.click(screen.getByRole("button", { name: "Save to vocabulary" }));
    await waitFor(() => expect(vocab.add).toHaveBeenCalled());
    const args = vocab.add.mock.calls[0] as string[];
    expect(args[0]).toBe("原文");
    expect(args[1]).toBe("trans");
  });

  it("speak toggles to Stop; stop calls tts_stop", async () => {
    renderLive();
    emit("popup-state", state({ status: "result", text: "hello", engine: "e" }));
    await screen.findAllByTestId("popup-card");
    const speak = screen.getByRole("button", { name: "Speak" });
    fireEvent.click(speak);
    await waitFor(() => expect(tts.speak).toHaveBeenCalledWith("hello"));
    await waitFor(() => expect(screen.getByRole("button", { name: "Stop" })).toBeInTheDocument());
    fireEvent.click(screen.getByRole("button", { name: "Stop" }));
    await waitFor(() => expect(tts.stop).toHaveBeenCalledTimes(1));
  });

  it("network error + source offers Retry via translate_selection_ipc with the SOURCE", async () => {
    renderLive();
    emit("popup-state", state({ status: "error", text: "network error: timeout", engine: "", source_text: "原文" }));
    await screen.findByTestId("popup-error");
    fireEvent.click(screen.getByRole("button", { name: "Retry" }));
    await waitFor(() => expect(translateSelection).toHaveBeenCalledWith("原文"));
  });

  it("config-key error offers Open Settings → provider-center", async () => {
    renderLive();
    emit("popup-state", state({ status: "error", text: "missing API key", engine: "" }));
    await screen.findByTestId("popup-error");
    fireEvent.click(screen.getByRole("button", { name: "Open Settings" }));
    await waitFor(() => expect(settings.open).toHaveBeenCalledWith("provider-center"));
  });

  it("keystore-corrupt offers the dedicated recovery CTA", async () => {
    renderLive();
    emit("popup-state", state({ status: "error", text: "keystore unreadable", engine: "" }));
    await screen.findByTestId("popup-keystore");
    fireEvent.click(screen.getByRole("button", { name: "Recover Keystore" }));
    await waitFor(() => expect(settings.open).toHaveBeenCalledWith("keystore-recovery"));
  });

  it("a stale Retry rejection never overwrites a newer event (generation guard)", async () => {
    let rejectRetry!: (e: Error) => void;
    (translateSelection as ReturnType<typeof vi.fn>).mockImplementationOnce(
      () => new Promise((_res, rej) => (rejectRetry = rej)),
    );
    renderLive();
    emit("popup-state", state({ status: "error", text: "network error", engine: "", source_text: "原文" }));
    await screen.findByTestId("popup-error");
    fireEvent.click(screen.getByRole("button", { name: "Retry" }));
    // A NEWER event lands while the Retry is in flight.
    emit("popup-state", state({ status: "result", text: "newer", engine: "e" }));
    await screen.findByText("newer");
    rejectRetry(new Error("stale ipc failure"));
    await waitFor(() => {});
    // The success from the newer event survives; the stale error is dropped.
    expect(screen.getByTestId("popup-card")).toHaveTextContent("newer");
    expect(screen.queryByTestId("popup-error")).toBeNull();
  });
});
