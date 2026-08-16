import { render, screen, fireEvent, cleanup, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { AppProviders } from "../../app/providers";
import { InputPanelView } from "./InputPanelView";
import { useInputController } from "./inputController";
import { resetProviderNameMap } from "./providerNames";

const { inputIpc, providerList, commands } = vi.hoisted(() => {
  const providerList = vi.fn();
  return {
    inputIpc: {
      translateSession: vi.fn(),
      addVocabulary: vi.fn(),
    },
    providerList,
    commands: {
      providerList,
    },
  };
});
vi.mock("./input-ipc", () => inputIpc);
vi.mock("../../bridge/invoke", () => ({ commands }));

function Live() {
  const c = useInputController();
  return <InputPanelView c={c} />;
}

const renderLive = () => render(<Live />, { wrapper: AppProviders });

beforeEach(() => {
  vi.clearAllMocks();
  localStorage.clear();
  resetProviderNameMap();
  providerList.mockResolvedValue([{ uuid: "u1", name: "MyOpenAI" }]);
  inputIpc.addVocabulary.mockResolvedValue(undefined);
});

afterEach(cleanup);

describe("Input window (controller + view integration)", () => {
  it("restores a persisted draft and focuses the textarea with the cursor at the end", () => {
    localStorage.setItem("linguaray.input-draft", "hello world");
    renderLive();
    const area = screen.getByRole("textbox", { name: "Translate" }) as HTMLTextAreaElement;
    expect(area.value).toBe("hello world");
    expect(document.activeElement).toBe(area);
    expect(area.selectionStart).toBe("hello world".length);
  });

  it("Enter translates (shift+Enter does not); empty text blocks the button", async () => {
    inputIpc.translateSession.mockResolvedValue({
      outcomes: [{ uuid: "u1", ok: true, text: "你好", engine: "provider/u1" }],
    });
    renderLive();
    const area = screen.getByRole("textbox", { name: "Translate" });
    expect(screen.getByRole("button", { name: "Translate" })).toBeDisabled();
    fireEvent.change(area, { target: { value: "hello" } });
    fireEvent.keyDown(area, { key: "Enter", shiftKey: true });
    expect(inputIpc.translateSession).not.toHaveBeenCalled();
    fireEvent.keyDown(area, { key: "Enter" });
    await waitFor(() => expect(inputIpc.translateSession).toHaveBeenCalledWith("hello"));
    expect(await screen.findAllByTestId("input-result")).toHaveLength(1);
    // Provider name map resolves the engine label to the friendly name.
    expect(screen.getByText("MyOpenAI")).toBeInTheDocument();
  });

  it("multi-engine session renders one card per outcome with per-engine errors", async () => {
    inputIpc.translateSession.mockResolvedValue({
      outcomes: [
        { uuid: "u1", ok: true, text: "a", engine: "provider/u1" },
        { uuid: "u2", ok: false, error: "timeout", engine: "provider/u2" },
      ],
    });
    renderLive();
    fireEvent.change(screen.getByRole("textbox", { name: "Translate" }), {
      target: { value: "x" },
    });
    fireEvent.click(screen.getByRole("button", { name: "Translate" }));
    const cards = await screen.findAllByTestId("input-result");
    expect(cards).toHaveLength(2);
    expect(screen.getByText("timeout")).toBeInTheDocument();
  });

  it("a rejected session surfaces the raw error (legacy catch-all semantics)", async () => {
    inputIpc.translateSession.mockRejectedValue(new Error("network error: timeout"));
    renderLive();
    fireEvent.change(screen.getByRole("textbox", { name: "Translate" }), {
      target: { value: "x" },
    });
    fireEvent.click(screen.getByRole("button", { name: "Translate" }));
    await waitFor(() =>
      expect(screen.getByTestId("input-error")).toHaveTextContent("network error: timeout"),
    );
  });

  it("Clear wipes text+result immediately and purges the draft (no debounce race)", async () => {
    inputIpc.translateSession.mockResolvedValue({
      outcomes: [{ uuid: "u1", ok: true, text: "你好", engine: "e" }],
    });
    renderLive();
    fireEvent.change(screen.getByRole("textbox", { name: "Translate" }), {
      target: { value: "abc" },
    });
    fireEvent.click(screen.getByRole("button", { name: "Translate" }));
    await screen.findAllByTestId("input-result");
    fireEvent.click(screen.getByRole("button", { name: "Clear" }));
    expect(screen.getByRole("textbox", { name: "Translate" })).toHaveValue("");
    expect(localStorage.getItem("linguaray.input-draft")).toBeNull();
  });

  it("favorite sends vocabulary_add with the translation", async () => {
    inputIpc.translateSession.mockResolvedValue({
      outcomes: [{ uuid: "u1", ok: true, text: "你好", engine: "e" }],
    });
    renderLive();
    fireEvent.change(screen.getByRole("textbox", { name: "Translate" }), {
      target: { value: "hello" },
    });
    fireEvent.click(screen.getByRole("button", { name: "Translate" }));
    await screen.findAllByTestId("input-result");
    fireEvent.click(screen.getByRole("button", { name: "Save to vocabulary" }));
    await waitFor(() =>
      expect(inputIpc.addVocabulary).toHaveBeenCalledWith("hello", "你好", "en"),
    );
    await waitFor(() =>
      expect(screen.getByRole("button", { name: "Saved" })).toBeInTheDocument(),
    );
  });

  it("draft autosaves after the debounce window", async () => {
    vi.useFakeTimers();
    try {
      renderLive();
      fireEvent.change(screen.getByRole("textbox", { name: "Translate" }), {
        target: { value: "persisted" },
      });
      expect(localStorage.getItem("linguaray.input-draft")).toBeNull();
      vi.advanceTimersByTime(400);
      expect(localStorage.getItem("linguaray.input-draft")).toBe("persisted");
    } finally {
      vi.useRealTimers();
    }
  });
});
