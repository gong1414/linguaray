import { act, cleanup, renderHook, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

const { ipc } = vi.hoisted(() => ({
  ipc: {
    historyPrivacyEnabled: vi.fn(),
    historySearch: vi.fn(),
    historyToggleFavorite: vi.fn(),
    historyDeleteSession: vi.fn(),
    historyExport: vi.fn(),
    chooseExportPath: vi.fn(),
  },
}));

vi.mock("./ipc", () => ipc);

import { useHistoryController } from "./controller";

const item = (i: number, over: Record<string, unknown> = {}) => ({
  session_uuid: `s-${i}`,
  timestamp: 1,
  trigger_source: "selection",
  detected_language: "en",
  target_language: "zh",
  is_favorite: false,
  source_text: `fox ${i}`,
  results: [],
  corrupt: false,
  ...over,
});

const page = (items: unknown[], scanComplete = true) => ({
  items,
  next_cursor: scanComplete ? null : "cursor-1",
  scan_complete: scanComplete,
});

beforeEach(() => {
  vi.clearAllMocks();
  ipc.historyPrivacyEnabled.mockResolvedValue(true);
  ipc.historySearch.mockResolvedValue(page([item(1), item(2)]));
  ipc.historyToggleFavorite.mockResolvedValue(true);
  ipc.historyDeleteSession.mockResolvedValue(undefined);
});

afterEach(cleanup);

describe("useHistoryController (controller + ipc integration)", () => {
  it("loads the first page and lands on populated", async () => {
    const { result } = renderHook(() => useHistoryController());
    await waitFor(() => expect(result.current.state).toBe("populated"));
    expect(result.current.items).toHaveLength(2);
    expect(ipc.historySearch).toHaveBeenCalledWith("", null);
  });

  it("history disabled → disabled state, no search", async () => {
    ipc.historyPrivacyEnabled.mockResolvedValue(false);
    const { result } = renderHook(() => useHistoryController());
    await waitFor(() => expect(result.current.state).toBe("disabled"));
    expect(ipc.historySearch).not.toHaveBeenCalled();
  });

  it("search passes the trimmed query", async () => {
    const { result } = renderHook(() => useHistoryController());
    await waitFor(() => expect(result.current.state).toBe("populated"));
    act(() => result.current.setQuery("  fox  "));
    await act(async () => {
      result.current.search();
    });
    expect(ipc.historySearch).toHaveBeenLastCalledWith("fox", null);
  });

  it("empty result without filters → empty; with filters → search-empty", async () => {
    ipc.historySearch.mockResolvedValue(page([]));
    const { result } = renderHook(() => useHistoryController());
    await waitFor(() => expect(result.current.state).toBe("empty"));
    act(() => result.current.setQuery("zzz"));
    await act(async () => {
      result.current.search();
    });
    await waitFor(() => expect(result.current.state).toBe("search-empty"));
  });

  it("loadMore appends with the cursor and reports hasMore", async () => {
    ipc.historySearch.mockResolvedValueOnce(page([item(1)], false));
    const { result } = renderHook(() => useHistoryController());
    await waitFor(() => expect(result.current.state).toBe("populated"));
    expect(result.current.hasMore).toBe(true);
    ipc.historySearch.mockResolvedValueOnce(page([item(2)], true));
    await act(async () => {
      result.current.loadMore();
    });
    await waitFor(() => {
      expect(result.current.items).toHaveLength(2);
      expect(result.current.hasMore).toBe(false);
    });
    expect(ipc.historySearch).toHaveBeenLastCalledWith("", "cursor-1");
  });

  it("favorites-only filters pages client-side (legacy behavior)", async () => {
    ipc.historySearch.mockResolvedValue(page([item(1), item(2, { is_favorite: true })]));
    const { result } = renderHook(() => useHistoryController());
    await waitFor(() => expect(result.current.state).toBe("populated"));
    await act(async () => {
      result.current.setFavoritesOnly(true);
    });
    await waitFor(() => {
      expect(result.current.state).toBe("populated");
      expect(result.current.items).toHaveLength(1);
    });
  });

  it("toggleFavorite updates the row and unfavorites disappear in favorites-only", async () => {
    ipc.historySearch.mockResolvedValue(page([item(2, { is_favorite: true })], true));
    const { result } = renderHook(() => useHistoryController());
    await waitFor(() => expect(result.current.state).toBe("populated"));
    await act(async () => {
      result.current.setFavoritesOnly(true);
    });
    await waitFor(() => expect(result.current.items).toHaveLength(1));
    ipc.historyToggleFavorite.mockResolvedValueOnce(false);
    await act(async () => {
      result.current.toggleFavorite(result.current.items[0]);
    });
    await waitFor(() => expect(result.current.items).toHaveLength(0));
    await waitFor(() => expect(result.current.state).toBe("search-empty"));
  });

  it("remove deletes the row and flips to empty when the last one goes", async () => {
    const { result } = renderHook(() => useHistoryController());
    await waitFor(() => expect(result.current.state).toBe("populated"));
    await act(async () => {
      result.current.remove(result.current.items[0]);
    });
    await waitFor(() => expect(result.current.items).toHaveLength(1));
    await act(async () => {
      result.current.remove(result.current.items[0]);
    });
    await waitFor(() => expect(result.current.state).toBe("empty"));
  });

  it("export resolves the path, writes with the filter, and notices the result", async () => {
    ipc.chooseExportPath.mockResolvedValue("/tmp/h.csv");
    ipc.historyExport.mockResolvedValue("/tmp/h.csv");
    const { result } = renderHook(() => useHistoryController());
    await waitFor(() => expect(result.current.state).toBe("populated"));
    await act(async () => {
      result.current.exportFile("csv");
    });
    await waitFor(() => expect(result.current.notice).toContain("/tmp/h.csv"));
    expect(ipc.historyExport).toHaveBeenCalledWith("/tmp/h.csv", "csv", {
      query: null,
      favorites_only: false,
    });
  });

  it("export failure shows the localized failure notice", async () => {
    ipc.chooseExportPath.mockResolvedValue("/tmp/h.json");
    ipc.historyExport.mockRejectedValue(new Error("disk full"));
    const { result } = renderHook(() => useHistoryController());
    await waitFor(() => expect(result.current.state).toBe("populated"));
    await act(async () => {
      result.current.exportFile("json");
    });
    await waitFor(() => expect(result.current.notice).toBe("Export failed"));
  });
});
