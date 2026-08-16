import { render, screen, fireEvent, cleanup, act } from "@testing-library/react";
import { renderHook, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { AppProviders } from "../../app/providers";
import { VocabularyView } from "./view";
import { useVocabularyController } from "./controller";

const { ipc } = vi.hoisted(() => ({
  ipc: {
    vocabularyList: vi.fn(),
    vocabularyAdd: vi.fn(),
    vocabularyDelete: vi.fn(),
    vocabularyExportFile: vi.fn(),
    vocabularyExportAnki: vi.fn(),
  },
}));
vi.mock("./ipc", () => ipc);
vi.mock("../../bridge/dialog", () => ({ save: vi.fn() }));

beforeEach(() => {
  vi.clearAllMocks();
  // The ipc module is mocked wholesale — return the MAPPED shape (the real
  // wrapper unwraps {items} inside ./ipc).
  ipc.vocabularyList.mockResolvedValue([
    { item_uuid: "i1", word: "hello", definition: "你好", source_language: "en", target_language: "zh" },
  ]);
  ipc.vocabularyAdd.mockResolvedValue(undefined);
  ipc.vocabularyDelete.mockResolvedValue(undefined);
});

afterEach(cleanup);

describe("useVocabularyController", () => {
  it("loads items on mount", async () => {
    const { result } = renderHook(() => useVocabularyController());
    await waitFor(() => expect(result.current.items).toHaveLength(1));
  });

  it("add trims, clears inputs, reloads; empty word is a no-op", async () => {
    const { result } = renderHook(() => useVocabularyController());
    await waitFor(() => expect(result.current.items).toHaveLength(1));
    act(() => result.current.setWord("  world  "));
    act(() => result.current.setDefinition("世界"));
    await act(async () => {
      result.current.add();
    });
    expect(ipc.vocabularyAdd).toHaveBeenCalledWith("world", "世界");
    await waitFor(() => expect(result.current.word).toBe(""));

    await act(async () => {
      result.current.add(); // word now empty → no-op
    });
    expect(ipc.vocabularyAdd).toHaveBeenCalledTimes(1);
  });

  it("remove deletes and reloads", async () => {
    const { result } = renderHook(() => useVocabularyController());
    await waitFor(() => expect(result.current.items).toHaveLength(1));
    await act(async () => {
      result.current.remove("i1");
    });
    expect(ipc.vocabularyDelete).toHaveBeenCalledWith("i1");
    expect(ipc.vocabularyList).toHaveBeenCalledTimes(2);
  });
});

describe("VocabularyView", () => {
  // Render the view against a REAL controller (integration per spec §八).
  function VocabularyLive() {
    const c = useVocabularyController();
    return <VocabularyView c={c} />;
  }

  it("renders the list and delete actions (controller + view integration)", async () => {
    render(<VocabularyLive />, { wrapper: AppProviders });
    expect(await screen.findByText("hello")).toBeInTheDocument();
    expect(screen.getByText("你好")).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "Delete" }));
    await waitFor(() => expect(ipc.vocabularyDelete).toHaveBeenCalledWith("i1"));
  });

  it("empty state explains how to save words", async () => {
    ipc.vocabularyList.mockResolvedValue([]);
    render(<VocabularyLive />, { wrapper: AppProviders });
    await waitFor(() => expect(screen.getByTestId("vocabulary-empty")).toBeInTheDocument());
  });

  it("Add is disabled while the word input is empty", async () => {
    render(<VocabularyLive />, { wrapper: AppProviders });
    await waitFor(() => expect(screen.getByRole("button", { name: "Add" })).toBeDisabled());
  });
});
