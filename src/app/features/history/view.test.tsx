import { render, screen, fireEvent, cleanup } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import axe from "axe-core";
import { AppProviders } from "../../app/providers";
import { HistoryView, type HistoryViewProps } from "./view";
import type { HistoryItem } from "./model";

const item = (i: number, over: Partial<HistoryItem> = {}): HistoryItem => ({
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

const base: HistoryViewProps = {
  locale: "en",
  state: "populated",
  items: [item(1), item(2, { is_favorite: true })],
  query: "",
  favoritesOnly: false,
  hasMore: false,
  notice: "",
  busy: false,
  onQueryChange: () => {},
  onSearch: () => {},
  onFavoritesOnlyChange: () => {},
  onLoadMore: () => {},
  onToggleFavorite: () => {},
  onRemove: () => {},
  onExport: () => {},
};

const renderView = (props: Partial<HistoryViewProps> = {}) =>
  render(<HistoryView {...base} {...props} />, { wrapper: AppProviders });

afterEach(cleanup);

describe("HistoryView", () => {
  it("renders populated rows with favorite state", () => {
    renderView();
    expect(screen.getByText("fox 1")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Unfavorite" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Favorite" })).toBeInTheDocument();
  });

  it.each(["empty", "search-empty", "disabled"] as const)(
    "%s state shows its dedicated empty screen",
    (state) => {
      renderView({ state, items: [] });
      expect(screen.getByTestId(`history-${state}`)).toBeInTheDocument();
    },
  );

  it("disabled state explains how to enable history", () => {
    renderView({ state: "disabled", items: [] });
    expect(screen.getByText("Enable history in Privacy to keep translations.")).toBeInTheDocument();
  });

  it("corrupt entries are flagged", () => {
    renderView({ items: [item(1, { corrupt: true })] });
    expect(screen.getByText("Corrupt entry")).toBeInTheDocument();
  });

  it("load more appears only when hasMore", () => {
    const { rerender } = render(<HistoryView {...base} />, { wrapper: AppProviders });
    expect(screen.queryByRole("button", { name: "Load more" })).toBeNull();
    rerender(<HistoryView {...base} hasMore />);
    expect(screen.getByRole("button", { name: "Load more" })).toBeInTheDocument();
  });

  it("search on Enter + favorites filter + export invoke callbacks", () => {
    const onSearch = vi.fn();
    const onFavoritesOnlyChange = vi.fn();
    const onExport = vi.fn();
    renderView({ onSearch, onFavoritesOnlyChange, onExport });
    fireEvent.change(screen.getByRole("textbox", { name: "Search" }), {
      target: { value: "fox" },
    });
    fireEvent.keyDown(screen.getByRole("textbox", { name: "Search" }), { key: "Enter" });
    expect(onSearch).toHaveBeenCalledTimes(1);
    fireEvent.click(screen.getByLabelText("Favorites only"));
    expect(onFavoritesOnlyChange).toHaveBeenCalledWith(true);
    fireEvent.click(screen.getByRole("button", { name: "Export CSV" }));
    expect(onExport).toHaveBeenCalledWith("csv");
  });

  it("notice renders as a live status", () => {
    renderView({ notice: "Exported to /tmp/x.csv" });
    expect(screen.getByTestId("history-notice")).toHaveAttribute("role", "status");
  });

  it("zh locale renders Chinese copy", () => {
    renderView({ locale: "zh" });
    expect(screen.getByText("历史")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "导出 CSV" })).toBeInTheDocument();
  });

  it("has no axe violations (populated, zh)", async () => {
    const { container } = renderView({ locale: "zh", hasMore: true });
    const results = await axe.run(container);
    expect(results.violations).toEqual([]);
  });
});
