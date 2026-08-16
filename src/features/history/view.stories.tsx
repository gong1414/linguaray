import type { Meta, StoryObj } from "@storybook/react-vite";
import { FluentProvider, webDarkTheme } from "@fluentui/react-components";
import { HistoryView, type HistoryViewProps } from "./view";
import type { HistoryItem } from "./model";

const item = (i: number, over: Partial<HistoryItem> = {}): HistoryItem => ({
  session_uuid: `s-${i}`,
  timestamp: 1_755_000_000 + i,
  trigger_source: "selection",
  detected_language: "en",
  target_language: "zh",
  is_favorite: i % 3 === 0,
  source_text: `The quick brown fox jumps over the lazy dog — sentence number ${i}`,
  results: [],
  corrupt: false,
  ...over,
});

const base: HistoryViewProps = {
  locale: "en",
  state: "populated",
  items: [item(1), item(2), item(3, { is_favorite: true })],
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

const meta: Meta<typeof HistoryView> = { title: "Settings/History", component: HistoryView };
export default meta;

export const Default: StoryObj<typeof HistoryView> = { args: base };

export const Loading: StoryObj<typeof HistoryView> = {
  args: { ...base, state: "loading", items: [], busy: true },
};

export const Empty: StoryObj<typeof HistoryView> = {
  args: { ...base, state: "empty", items: [] },
};

export const SearchEmpty: StoryObj<typeof HistoryView> = {
  args: { ...base, state: "search-empty", items: [], query: "zzz" },
};

export const Disabled: StoryObj<typeof HistoryView> = {
  args: { ...base, state: "disabled", items: [] },
};

export const LoadMore: StoryObj<typeof HistoryView> = {
  args: { ...base, hasMore: true },
};

export const CorruptEntry: StoryObj<typeof HistoryView> = {
  args: { ...base, items: [item(1, { corrupt: true })] },
};

export const ExportNotice: StoryObj<typeof HistoryView> = {
  args: { ...base, notice: "Exported to /Users/me/linguaray-history.csv" },
};

export const LongChinese: StoryObj<typeof HistoryView> = {
  args: {
    ...base,
    locale: "zh",
    items: [item(1, {
      source_text:
        "机器学习驱动的翻译工具在长中文文本上的表现：本条记录用于验证列表行在超长文本、标点密集与中英混排 (mixed CJK + Latin) 时不溢出、不撑破窗口。",
    })],
  },
};

export const Dark: StoryObj<typeof HistoryView> = {
  parameters: { backgrounds: { default: "dark" } },
  decorators: [
    (Story) => (
      <FluentProvider theme={webDarkTheme}>
        <Story />
      </FluentProvider>
    ),
  ],
  args: { ...base, locale: "zh" },
};

export const Narrow: StoryObj<typeof HistoryView> = {
  parameters: { viewport: { defaultViewport: "mobile1" } },
  args: { ...base, locale: "zh", hasMore: true },
};
