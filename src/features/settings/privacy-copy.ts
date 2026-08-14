import type { Locale } from "../../i18n";

export type PrivacyCopy = {
  title: string;
  historyTitle: string;
  historyEnable: string;
  historyDisabledNotice: string;
  historyEnabledNotice: string;
  retention: string;
  retention30: string;
  retention90: string;
  clearAll: string;
  clearConfirmTitle: string;
  clearConfirmMessage: string;
  cancel: string;
  loading: string;
  loadFailed: string;
  retry: string;
  updateFailed: string;
  cleared: string;
  records: string;
  externalTitle: string;
  externalDeferred: string;
  externalOff: string;
  externalOn: string;
  externalEnable: string;
  externalDisable: string;
  externalRegen: string;
  externalTokenOnce: string;
};

export const PRIVACY_COPY: Record<Locale, PrivacyCopy> = {
  en: {
    title: "Privacy & Data",
    historyTitle: "Translation History",
    historyEnable: "Enable history",
    historyDisabledNotice: "When off, new translations are not stored. Existing encrypted history is retained until you clear it.",
    historyEnabledNotice: "History is encrypted and stored locally only.",
    retention: "Retention period",
    retention30: "30 days",
    retention90: "90 days",
    clearAll: "Clear All",
    clearConfirmTitle: "Clear all history?",
    clearConfirmMessage: "This permanently removes all encrypted history and cannot be undone.",
    cancel: "Cancel",
    loading: "Loading privacy settings…",
    loadFailed: "Privacy settings could not be loaded.",
    retry: "Retry",
    updateFailed: "Privacy settings could not be updated.",
    cleared: "History cleared",
    records: "{count} encrypted records",
    externalTitle: "External API",
    externalDeferred: "Local HTTP API for scripts. Off by default.",
    externalOff: "External API: Off",
    externalOn: "External API: On (port {port})",
    externalEnable: "Enable",
    externalDisable: "Disable",
    externalRegen: "Regenerate token",
    externalTokenOnce: "Copy now — you won't see it again",
  },
  zh: {
    title: "隐私与数据",
    historyTitle: "翻译历史",
    historyEnable: "启用历史",
    historyDisabledNotice: "关闭时不再存储新的翻译。已有加密历史会保留，直到您主动清除。",
    historyEnabledNotice: "历史经过加密，仅存储在本机。",
    retention: "保留期",
    retention30: "30 天",
    retention90: "90 天",
    clearAll: "全部清除",
    clearConfirmTitle: "清除全部历史？",
    clearConfirmMessage: "这将永久删除全部加密历史，且无法撤销。",
    cancel: "取消",
    loading: "正在加载隐私设置…",
    loadFailed: "无法加载隐私设置。",
    retry: "重试",
    updateFailed: "无法更新隐私设置。",
    cleared: "历史已清除",
    records: "{count} 条加密记录",
    externalTitle: "外部 API",
    externalDeferred: "供本地脚本使用的 HTTP API，默认关闭。",
    externalOff: "外部 API：关",
    externalOn: "外部 API：开（端口 {port}）",
    externalEnable: "启用",
    externalDisable: "停用",
    externalRegen: "重新生成令牌",
    externalTokenOnce: "请立即复制 — 之后不会再显示",
  },
};
