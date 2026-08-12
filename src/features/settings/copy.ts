/**
 * Settings shell + Provider Center (Surface 05) + Keystore Recovery (Surface 06)
 * copy dictionary.
 *
 * Locale comes from `src/i18n.ts` `detectLocale()`. No raw user-facing
 * literals in components — only copy-key lookups. The `{name}` / `{reason}` /
 * `{latency}` / `{message}` placeholders are substituted at the call site via
 * `String.prototype.replace` (NOT template literals) so this dictionary stays
 * the single source of truth.
 *
 * Provider strings ported from `apps/ui-lab/src/i18n/index.ts` (the `provider`
 * block), reconciled against `design-system/linguaray/pages/05-provider-center.md`.
 * Lab-only frame-marker keys (`frameMin`, `frameDefault`, `frameNarrow699`,
 * `frameBoundary700`) are dropped — they were lab scaffolding, not product copy.
 */

export type Locale = "zh" | "en";

/** Window chrome labels. */
type WindowCopy = {
  title: string;
  minimize: string;
  close: string;
};

/** Sidebar navigation labels + the placeholder tooltip for disabled items. */
type NavCopy = {
  providerCenter: string;
  keystoreRecovery: string;
  shortcuts: string;
  privacy: string;
  /** Tooltip shown for the disabled R3b placeholder items. */
  placeholderHint: string;
};

/** macOS Accessibility permission banner. Shown when `a11y_status` is false;
 *  selection capture needs the AX permission for both the direct-read and the
 *  simulated Cmd+C fallback. */
type A11yCopy = {
  /** Banner title. */
  title: string;
  /** Why LinguaRay needs the permission + how to grant it. */
  hint: string;
  /** Re-check button (re-invokes `a11y_status`). */
  recheck: string;
  /** Open System Settings button (deep-link to Privacy_Accessibility). */
  openSettings: string;
};

/** Keystore Recovery (Surface 06) labels.
 *  `description` carries the `{reason}` placeholder. */
type KeystoreCopy = {
  healthy: string;
  /** Corrupt banner title (no placeholder). */
  title: string;
  /** Corrupt banner description template: "Keystore unreadable: {reason}". */
  description: string;
  /** Archive & re-enter action button. */
  archive: string;
  /** Reset action button. */
  reset: string;
  /** Archived banner title. */
  archivedTitle: string;
  /** Archived banner prompt to re-enter keys. */
  archivedPrompt: string;
  /** Reset confirm dialog title. */
  resetConfirmTitle: string;
  /** Reset confirm dialog message. */
  resetConfirmMessage: string;
  /** Reset confirm dialog confirm button label. */
  resetConfirmConfirmLabel: string;
  /** Reset confirm dialog cancel button label. */
  resetConfirmCancelLabel: string;
  /** Toast when archive_keystore fails. */
  archiveFailed: string;
  /** Toast when reset_keystore fails. */
  resetFailed: string;
};

/** Provider role labels (primary / parallel / fallback / none). */
type RoleCopy = {
  primary: string;
  fallback: string;
  parallel: string;
  none: string;
};

/** Provider balance section labels. R3a limitation: balance/quota IPC not
 *  implemented, so the section renders a muted TODO note. */
type BalanceCopy = {
  title: string;
  /** Muted note shown instead of a balance value (R3a limitation). */
  unsupportedNote: string;
};

/** Provider endpoint field labels + error-code → localized message map. */
type EndpointCopy = {
  label: string;
  placeholder: string;
  /** Stable domain error code → localized message. */
  errors: {
    "endpoint-required": string;
    "endpoint-invalid-url": string;
    "endpoint-must-https": string;
  };
};

/** Provider selection (active roles) error-code → localized message map.
 *  Source: `ActiveSelectionErrorCode` in `provider-domain.ts`. */
type SelectionErrorCopy = {
  "parallel-duplicate": string;
  "parallel-contains-primary": string;
  "role-overlap": string;
  "disabled-in-slot": string;
  "fallback-not-traditional": string;
  "fallback-overlaps": string;
};

/** Consent dialog labels. */
type ConsentCopy = {
  title: string;
  message: string;
  confirm: string;
  cancel: string;
  local: string;
  remote: string;
};

/** Empty-state (no providers) labels. */
type EmptyCopy = {
  title: string;
  description: string;
};

/** Provider Center (Surface 05) labels. `cardEdit`/`cardDelete` carry `{name}`. */
type ProviderCopy = {
  empty: EmptyCopy;
  addProvider: string;
  models: string;
  fetchModels: string;
  modelFetchError: string;
  manualModelEntry: string;
  manualModelPlaceholder: string;
  testConnection: string;
  testing: string;
  connectionOk: string;
  connectionFailed: string;
  /** R8-P1: hint shown next to a disabled Test button when unsaved drafts
   *  exist. The Test runs against the BACKEND's stored config, so testing with
   *  unsaved edits would probe a config the user no longer sees — block it and
   *  tell them to save first. */
  saveFirstToTest: string;
  /** R9: hint shown next to a disabled Fetch Models button when unsaved drafts
   *  exist. Same rationale as `saveFirstToTest` — Fetch reads the BACKEND's
   *  stored config. */
  saveFirstToFetch: string;
  keySaved: string;
  profileSaved: string;
  keyMissing: string;
  enterKey: string;
  apiKey: string;
  apiKeyPlaceholder: string;
  saveKey: string;
  saveProfile: string;
  saving: string;
  saveFailed: string;
  /** R8-P2-2: the mutation (create/duplicate/delete) succeeded on the backend,
   *  but the post-mutation list refresh failed. The destructive `saveFailed`
   *  toast from refreshCore already surfaced the reload failure; this warning
   *  adds the accurate context that the write itself went through, so the user
   *  knows to click Reload rather than re-issue the mutation. */
  mutationSuccessReloadFailed: string;
  /** R2-E: save-conflict banner — the provider was modified elsewhere and the
   *  optimistic-lock CAS rejected this save. Paired with a Reload button. */
  saveConflict: string;
  /** Surfaced when a save rejects with a UNIQUE constraint violation
   *  (duplicate name / secret_ref). Carries no placeholder. */
  keyAlreadyExists: string;
  /** Surfaced when the user edits a provider name to one another provider
   *  already uses (structured duplicate-name conflict, checked client-side
   *  before save). Carries no placeholder. */
  nameExists: string;
  /** Cold-load failure banner title (provider list OR active selection read failed). */
  loadFailed: string;
  /** Retry button on the cold-load failure banner. */
  retry: string;
  reload: string;
  /** R6-P1-1: accessible label for the Reload button's spinner while a Reload
   *  (global mutation lock) is in-flight. Shown while all sidebar mutations are
   *  blocked. */
  reloading: string;
  cancel: string;
  deleteConfirmTitle: string;
  deleteConfirmMsg: string;
  delete: string;
  deleting: string;
  moveUp: string;
  moveDown: string;
  reorderReverted: string;
  /** Suffix appended to a duplicated provider's name: "OpenAI (copy)". */
  copySuffix: string;
  duplicate: string;
  selectPrimary: string;
  /** Set-as-primary / add-to-parallel / set-as-fallback action labels. */
  setPrimary: string;
  addParallel: string;
  removeParallel: string;
  setFallback: string;
  /** Role badges + status labels. */
  role: RoleCopy;
  enabled: string;
  disabled: string;
  /** Field labels. */
  name: string;
  endpoint: EndpointCopy;
  /** Card / row aria-label templates with `{name}`. */
  cardEdit: string;
  cardDelete: string;
  /** Accessibility / section labels. */
  providerListLabel: string;
  detailLabel: string;
  loadingModels: string;
  toastDismiss: string;
  movedUp: string;
  movedDown: string;
  /** Selection / consent. */
  selectionErrors: SelectionErrorCopy;
  consent: ConsentCopy;
  /** Balance (R3a limitation: muted note, no fetch). */
  balance: BalanceCopy;
};

export type SettingsCopy = {
  window: WindowCopy;
  nav: NavCopy;
  a11y: A11yCopy;
  keystore: KeystoreCopy;
  provider: ProviderCopy;
};

// --- Shared leaf lookups (used by both locales) -------------------------

const EN: SettingsCopy = {
  window: {
    title: "LinguaRay",
    minimize: "Minimize",
    close: "Close",
  },
  nav: {
    providerCenter: "Provider Center",
    keystoreRecovery: "Keystore Recovery",
    shortcuts: "Shortcuts",
    privacy: "Privacy",
    placeholderHint: "Coming in R3b",
  },
  a11y: {
    title: "Accessibility permission needed",
    hint: "LinguaRay needs the macOS Accessibility permission to capture selected text. Grant it in System Settings, then re-check.",
    recheck: "Re-check",
    openSettings: "System Settings",
  },
  keystore: {
    healthy: "Keystore healthy",
    title: "Keystore unreadable",
    description: "Keystore unreadable: {reason}",
    archive: "Archive & re-enter",
    reset: "Reset",
    archivedTitle: "Keys archived",
    archivedPrompt: "Enter your keys again",
    resetConfirmTitle: "Reset keystore?",
    resetConfirmMessage: "History will become undecryptable. Continue?",
    resetConfirmConfirmLabel: "Reset",
    resetConfirmCancelLabel: "Cancel",
    archiveFailed: "Archive failed",
    resetFailed: "Reset failed",
  },
  provider: {
    empty: {
      title: "Add your first provider",
      description: "Pick a preset, enter your API key, and start translating.",
    },
    addProvider: "Add provider",
    models: "Model",
    fetchModels: "Fetch models",
    modelFetchError: "Failed to fetch models — enter manually",
    manualModelEntry: "Enter model manually",
    manualModelPlaceholder: "e.g. gpt-4o",
    testConnection: "Test",
    testing: "Testing…",
    connectionOk: "Connected",
    connectionFailed: "Connection failed",
    saveFirstToTest: "Save changes before testing",
    saveFirstToFetch: "Save changes before fetching models",
    keySaved: "Key saved",
    profileSaved: "Profile saved",
    keyMissing: "Key missing",
    enterKey: "Enter key",
    apiKey: "API key",
    apiKeyPlaceholder: "sk-…",
    saveKey: "Save key",
    saveProfile: "Save profile",
    saving: "Saving…",
    saveFailed: "Failed to save: network error",
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
    deleting: "Deleting…",
    moveUp: "Move up",
    moveDown: "Move down",
    reorderReverted: "Failed to save order — reverted",
    copySuffix: "(copy)",
    duplicate: "Duplicate",
    selectPrimary: "Select a primary provider",
    setPrimary: "Set as primary",
    addParallel: "Add to parallel",
    removeParallel: "Remove from parallel",
    setFallback: "Set as fallback",
    role: {
      primary: "Primary",
      fallback: "Fallback",
      parallel: "Parallel",
      none: "None",
    },
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
    movedUp: "moved up",
    movedDown: "moved down",
    selectionErrors: {
      "parallel-duplicate": "Parallel list contains a duplicate provider",
      "parallel-contains-primary": "Parallel list must not contain the primary provider",
      "role-overlap": "A provider cannot hold two roles",
      "disabled-in-slot": "Provider is disabled or deleted",
      "fallback-not-traditional": "Fallback must be a traditional MT engine",
      "fallback-overlaps": "Fallback must not overlap primary or parallel",
    },
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
      // TODO(r3b): balance/quota IPC not yet implemented — render this muted note.
      unsupportedNote: "Balance and quota are not yet available.",
    },
  },
};

const ZH: SettingsCopy = {
  window: {
    title: "LinguaRay",
    minimize: "最小化",
    close: "关闭",
  },
  nav: {
    providerCenter: "Provider Center",
    keystoreRecovery: "Keystore Recovery",
    shortcuts: "Shortcuts",
    privacy: "Privacy",
    placeholderHint: "将在 R3b 中提供",
  },
  a11y: {
    title: "需要辅助功能权限",
    hint: "LinguaRay 需要辅助功能权限才能捕获选中文本。请在系统设置中授权，然后重新检查。",
    recheck: "重新检查",
    openSettings: "系统设置",
  },
  keystore: {
    healthy: "密钥库正常",
    title: "密钥库不可读",
    description: "密钥库不可读：{reason}",
    archive: "归档并重新输入",
    reset: "重置",
    archivedTitle: "密钥已归档",
    archivedPrompt: "请重新输入您的密钥",
    resetConfirmTitle: "重置密钥库？",
    resetConfirmMessage: "历史将无法解密。继续？",
    resetConfirmConfirmLabel: "重置",
    resetConfirmCancelLabel: "取消",
    archiveFailed: "归档失败",
    resetFailed: "重置失败",
  },
  provider: {
    empty: {
      title: "添加你的第一个服务商",
      description: "选择预设，输入 API 密钥，即可开始翻译。",
    },
    addProvider: "添加服务商",
    models: "模型",
    fetchModels: "获取模型",
    modelFetchError: "获取模型失败 —— 请手动输入",
    manualModelEntry: "手动输入模型",
    manualModelPlaceholder: "例如 gpt-4o",
    testConnection: "测试",
    testing: "测试中…",
    connectionOk: "已连接",
    connectionFailed: "连接失败",
    saveFirstToTest: "请先保存更改再测试",
    saveFirstToFetch: "请先保存更改再获取模型",
    keySaved: "密钥已保存",
    profileSaved: "配置已保存",
    keyMissing: "缺少密钥",
    enterKey: "输入密钥",
    apiKey: "API 密钥",
    apiKeyPlaceholder: "sk-…",
    saveKey: "保存密钥",
    saveProfile: "保存配置",
    saving: "保存中…",
    saveFailed: "保存失败：网络错误",
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
    deleting: "删除中…",
    moveUp: "上移",
    moveDown: "下移",
    reorderReverted: "排序保存失败 —— 已恢复",
    copySuffix: "（副本）",
    duplicate: "复制",
    selectPrimary: "请选择主引擎",
    setPrimary: "设为主引擎",
    addParallel: "加入并行",
    removeParallel: "移出并行",
    setFallback: "设为回退",
    role: {
      primary: "主引擎",
      fallback: "回退",
      parallel: "并行",
      none: "无",
    },
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
    movedUp: "已上移",
    movedDown: "已下移",
    selectionErrors: {
      "parallel-duplicate": "并行列表包含重复服务商",
      "parallel-contains-primary": "并行列表不得包含主引擎",
      "role-overlap": "一个服务商不能同时担任两个角色",
      "disabled-in-slot": "服务商已禁用或已删除",
      "fallback-not-traditional": "回退必须是传统机器翻译引擎",
      "fallback-overlaps": "回退不得与主引擎或并行重复",
    },
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
      // TODO(r3b): balance/quota IPC not yet implemented — render this muted note.
      unsupportedNote: "余额与配额暂不可用。",
    },
  },
};

export const SETTINGS_COPY: Record<Locale, SettingsCopy> = {
  zh: ZH,
  en: EN,
};
