import type { Locale } from "../../i18n";

export type VocabularyItem = {
  item_uuid: string;
  word: string;
  definition: string;
  source_language: string;
  target_language: string;
};

export type VocabularyCopy = {
  title: string;
  empty: string;
  hint: string;
  word: string;
  definition: string;
  add: string;
  delete: string;
  exportCsv: string;
  exportJson: string;
  exportAnki: string;
  exportDone: string;
  exportFailed: string;
};

const EN: VocabularyCopy = {
  title: "Vocabulary",
  empty: "No saved words yet",
  hint: "Save words from translations to build your list.",
  word: "Word",
  definition: "Definition",
  add: "Add",
  delete: "Delete",
  exportCsv: "Export CSV",
  exportJson: "Export JSON",
  exportAnki: "AnkiConnect",
  exportDone: "Export complete",
  exportFailed: "Export failed",
};

const ZH: VocabularyCopy = {
  title: "生词本",
  empty: "暂无保存的单词",
  hint: "从翻译中保存单词以建立列表。",
  word: "单词",
  definition: "释义",
  add: "添加",
  delete: "删除",
  exportCsv: "导出 CSV",
  exportJson: "导出 JSON",
  exportAnki: "AnkiConnect",
  exportDone: "导出完成",
  exportFailed: "导出失败",
};

export const VOCABULARY_COPY: Record<Locale, VocabularyCopy> = { en: EN, zh: ZH };
