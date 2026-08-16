import type { Locale } from "../../app/i18n";

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
  emptyTitle: string;
  noMatchesTitle: string;
  disabledTitle: string;
  disabledHint: string;
  corruptLabel: string;
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
  emptyTitle: "No history yet",
  noMatchesTitle: "No matches",
  disabledTitle: "History is off",
  disabledHint: "Enable history in Privacy to keep translations.",
  corruptLabel: "Corrupt entry",
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
  emptyTitle: "暂无历史",
  noMatchesTitle: "无匹配",
  disabledTitle: "历史记录已关闭",
  disabledHint: "在隐私设置中开启历史即可保存翻译。",
  corruptLabel: "损坏条目",
  exportDone: "已导出到 {path}",
  exportFailed: "导出失败",
};

export const HISTORY_COPY: Record<Locale, HistoryCopy> = { en: EN, zh: ZH };
