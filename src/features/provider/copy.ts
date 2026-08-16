import type { Locale } from "../../app/i18n";
import type { EndpointErrorCode } from "./domain";

/** Provider Center copy (zh/en) — migrated 1:1 from the frozen Solid copy. */
export type ProviderCopy = {
  empty: { title: string; description: string };
  addProvider: string;
  tier: { setupRequired: string; unverified: string };
  insertAzureTemplate: string;
  useKimiGlobal: string;
  customAnthropic: string;
  models: string;
  fetchModels: string;
  modelFetchError: string;
  manualModelPlaceholder: string;
  testConnection: string;
  connectionOk: string;
  connectionFailed: string;
  saveFirstToTest: string;
  saveFirstToFetch: string;
  keySaved: string;
  profileSaved: string;
  keyMissing: string;
  noKeyRequired: string;
  apiKey: string;
  apiKeyPlaceholder: string;
  saveKey: string;
  saveProfile: string;
  saveFailed: string;
  reloadFailed: string;
  mutationSuccessReloadFailed: string;
  saveConflict: string;
  keyAlreadyExists: string;
  nameExists: string;
  loadFailed: string;
  retry: string;
  reload: string;
  reloading: string;
  cancel: string;
  deleteConfirmTitle: string;
  deleteConfirmMsg: string;
  delete: string;
  moveUp: string;
  moveDown: string;
  reorderReverted: string;
  duplicate: string;
  selectPrimary: string;
  setPrimary: string;
  addParallel: string;
  removeParallel: string;
  setFallback: string;
  role: { primary: string; fallback: string; parallel: string; none: string };
  enabled: string;
  disabled: string;
  name: string;
  endpoint: {
    label: string;
    placeholder: string;
    errors: Record<EndpointErrorCode, string>;
  };
  cardEdit: string;
  cardDelete: string;
  providerListLabel: string;
  detailLabel: string;
  loadingModels: string;
  toastDismiss: string;
  consent: {
    title: string;
    message: string;
    confirm: string;
    cancel: string;
    local: string;
    remote: string;
  };
  balance: {
    title: string;
    unsupportedNote: string;
    fetch: string;
    loading: string;
  };
};

const EN: ProviderCopy = {
  empty: {
    title: "Add your first provider",
    description: "Pick a preset, enter your API key, and start translating.",
  },
  addProvider: "Add provider",
  tier: { setupRequired: "Setup", unverified: "Unverified" },
  insertAzureTemplate: "Insert Azure URL template",
  useKimiGlobal: "Use global endpoint",
  customAnthropic: "Anthropic Messages API",
  models: "Model",
  fetchModels: "Fetch models",
  modelFetchError: "Failed to fetch models — enter manually",
  manualModelPlaceholder: "e.g. gpt-4o",
  testConnection: "Test",
  connectionOk: "Connected",
  connectionFailed: "Connection failed",
  saveFirstToTest: "Save changes before testing",
  saveFirstToFetch: "Save changes before fetching models",
  keySaved: "Key saved",
  profileSaved: "Profile saved",
  keyMissing: "Key missing",
  noKeyRequired: "No key required",
  apiKey: "API key",
  apiKeyPlaceholder: "sk-…",
  saveKey: "Save key",
  saveProfile: "Save profile",
  saveFailed: "Failed to save: network error",
  reloadFailed: "Reload failed",
  mutationSuccessReloadFailed: "Saved, but the list could not be refreshed. Click Reload to retry.",
  saveConflict: "This provider was modified elsewhere",
  keyAlreadyExists: "A provider with this name already exists",
  nameExists: "Another provider already uses this name",
  loadFailed: "Provider load failed",
  retry: "Retry",
  reload: "Reload",
  reloading: "Reloading…",
  cancel: "Cancel",
  deleteConfirmTitle: "Delete provider?",
  deleteConfirmMsg: "History references are preserved.",
  delete: "Delete",
  moveUp: "Move up",
  moveDown: "Move down",
  reorderReverted: "Failed to save order — reverted",
  duplicate: "Duplicate",
  selectPrimary: "Select a primary provider",
  setPrimary: "Set as primary",
  addParallel: "Add to parallel",
  removeParallel: "Remove from parallel",
  setFallback: "Set as fallback",
  role: { primary: "Primary", fallback: "Fallback", parallel: "Parallel", none: "None" },
  enabled: "Enabled",
  disabled: "Disabled",
  name: "Name",
  endpoint: {
    label: "Endpoint",
    placeholder: "https://api.example.com",
    errors: {
      "endpoint-required": "Endpoint is required",
      "endpoint-invalid-url": "Invalid URL",
      "endpoint-must-https": "Must be HTTPS (or localhost)",
    },
  },
  cardEdit: "Edit {name}",
  cardDelete: "Delete {name}",
  providerListLabel: "Provider list",
  detailLabel: "Provider detail",
  loadingModels: "Loading models…",
  toastDismiss: "Dismiss",
  consent: {
    title: "Send text to multiple providers?",
    message: "Your text will be sent to the following providers:",
    confirm: "Confirm",
    cancel: "Cancel",
    local: "local",
    remote: "remote",
  },
  balance: {
    title: "Balance",
    unsupportedNote: "Balance and quota are not supported by this provider.",
    fetch: "Fetch balance",
    loading: "Loading balance…",
  },
};

const ZH: ProviderCopy = {
  empty: {
    title: "添加你的第一个服务商",
    description: "选择预设，输入 API 密钥，即可开始翻译。",
  },
  addProvider: "添加服务商",
  tier: { setupRequired: "需配置", unverified: "未认证" },
  insertAzureTemplate: "插入 Azure URL 模板",
  useKimiGlobal: "改用全球端点",
  customAnthropic: "Anthropic Messages API",
  models: "模型",
  fetchModels: "获取模型",
  modelFetchError: "获取模型失败 —— 请手动输入",
  manualModelPlaceholder: "例如 gpt-4o",
  testConnection: "测试",
  connectionOk: "已连接",
  connectionFailed: "连接失败",
  saveFirstToTest: "请先保存更改再测试",
  saveFirstToFetch: "请先保存更改再获取模型",
  keySaved: "密钥已保存",
  profileSaved: "配置已保存",
  keyMissing: "缺少密钥",
  noKeyRequired: "无需密钥",
  apiKey: "API 密钥",
  apiKeyPlaceholder: "sk-…",
  saveKey: "保存密钥",
  saveProfile: "保存配置",
  saveFailed: "保存失败：网络错误",
  reloadFailed: "重新加载失败",
  mutationSuccessReloadFailed: "已保存，但列表刷新失败。点击重新加载重试。",
  saveConflict: "此服务商已在别处修改",
  keyAlreadyExists: "该名称的服务商已存在",
  nameExists: "该名称已被其他提供商使用",
  loadFailed: "加载失败",
  retry: "重试",
  reload: "重新加载",
  reloading: "正在重新加载…",
  cancel: "取消",
  deleteConfirmTitle: "删除服务商？",
  deleteConfirmMsg: "历史引用将被保留。",
  delete: "删除",
  moveUp: "上移",
  moveDown: "下移",
  reorderReverted: "排序保存失败 —— 已恢复",
  duplicate: "复制",
  selectPrimary: "请选择主引擎",
  setPrimary: "设为主引擎",
  addParallel: "加入并行",
  removeParallel: "移出并行",
  setFallback: "设为回退",
  role: { primary: "主引擎", fallback: "回退", parallel: "并行", none: "无" },
  enabled: "已启用",
  disabled: "已禁用",
  name: "名称",
  endpoint: {
    label: "端点",
    placeholder: "https://api.example.com",
    errors: {
      "endpoint-required": "端点不能为空",
      "endpoint-invalid-url": "URL 无效",
      "endpoint-must-https": "必须为 HTTPS（或 localhost）",
    },
  },
  cardEdit: "编辑{name}",
  cardDelete: "删除{name}",
  providerListLabel: "服务商列表",
  detailLabel: "服务商详情",
  loadingModels: "正在加载模型…",
  toastDismiss: "关闭",
  consent: {
    title: "将文本发送给多个服务商？",
    message: "你的文本将发送给以下服务商：",
    confirm: "确认",
    cancel: "取消",
    local: "本地",
    remote: "远程",
  },
  balance: {
    title: "余额",
    unsupportedNote: "此服务商不支持余额与配额。",
    fetch: "查询余额",
    loading: "正在查询余额…",
  },
};

export const PROVIDER_COPY: Record<Locale, ProviderCopy> = { en: EN, zh: ZH };
