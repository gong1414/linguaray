import type { Locale } from "./copy";

export type HistoryCopy = {
  title: string;
  search: string;
  searchPlaceholder: string;
  favoritesOnly: string;
  exportCsv: string;
  exportJson: string;
  favorite: string;
  unfavorite: string;
  delete: string;
  loadMore: string;
  empty: { title: string };
  noMatches: { title: string };
  disabled: { title: string; hint: string };
  corrupt: { label: string };
  exportDone: string;
  exportFailed: string;
};

const EN: HistoryCopy = {
  title: "History",
  search: "Search",
  searchPlaceholder: "Search translations",
  favoritesOnly: "Favorites only",
  exportCsv: "Export CSV",
  exportJson: "Export JSON",
  favorite: "Favorite",
  unfavorite: "Unfavorite",
  delete: "Delete",
  loadMore: "Load more",
  empty: { title: "No history yet" },
  noMatches: { title: "No matches" },
  disabled: {
    title: "History is off",
    hint: "Enable history in Privacy to keep translations.",
  },
  corrupt: { label: "Corrupt entry" },
  exportDone: "Exported to {path}",
  exportFailed: "Export failed",
};

const ZH: HistoryCopy = {
  title: "历史",
  search: "搜索",
  searchPlaceholder: "搜索翻译",
  favoritesOnly: "仅收藏",
  exportCsv: "导出 CSV",
  exportJson: "导出 JSON",
  favorite: "收藏",
  unfavorite: "取消收藏",
  delete: "删除",
  loadMore: "加载更多",
  empty: { title: "暂无历史" },
  noMatches: { title: "无匹配" },
  disabled: {
    title: "历史记录已关闭",
    hint: "在隐私设置中开启历史即可保存翻译。",
  },
  corrupt: { label: "损坏条目" },
  exportDone: "已导出到 {path}",
  exportFailed: "导出失败",
};

export const HISTORY_COPY: Record<Locale, HistoryCopy> = { en: EN, zh: ZH };
