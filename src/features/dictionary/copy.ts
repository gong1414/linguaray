import type { Locale } from "../../app/i18n";

export type DictPackage = { package_id: string; name: string };
export type DictResult = { definition: string; source: string } | null;

export type DictionaryCopy = {
  title: string;
  lookup: string;
  word: string;
  noPackages: string;
  noResult: string;
  source: string;
  install: string;
  sourceDir: string;
  packageId: string;
  packageName: string;
  version: string;
  installed: string;
};

const EN: DictionaryCopy = {
  title: "Dictionary",
  lookup: "Look up",
  word: "Word",
  noPackages: "No offline packages installed",
  noResult: "No definition found",
  source: "Source: {source}",
  install: "Install package",
  sourceDir: "Folder path",
  packageId: "Package id",
  packageName: "Name",
  version: "Version",
  installed: "Installed",
};

const ZH: DictionaryCopy = {
  title: "词典",
  lookup: "查询",
  word: "单词",
  noPackages: "尚未安装离线词库",
  noResult: "未找到释义",
  source: "来源：{source}",
  install: "安装词库",
  sourceDir: "文件夹路径",
  packageId: "词库 ID",
  packageName: "名称",
  version: "版本",
  installed: "已安装",
};

export const DICTIONARY_COPY: Record<Locale, DictionaryCopy> = { en: EN, zh: ZH };
