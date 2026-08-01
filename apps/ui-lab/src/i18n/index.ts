/**
 * UI Lab i18n — minimal dictionary (zh/en). Not the production i18n layer; this
 * only exists so the lab can exercise both languages against the same screens.
 * Production translation strings live in the spec (S0) and are localized there.
 *
 * Every user-facing string rendered by the lab MUST come from here — no inline
 * English in components, so switching locale also switches what assistive
 * technology announces.
 */

export type Locale = "zh" | "en";

/**
 * S0 §4.1 Selection Popup — complete state matrix. Every row in the spec table
 * is represented. `initial-hidden` renders NO popup window/region at all (the
 * popup does not exist until a selection is made).
 */
export type SelectionState =
  | "initial-hidden"
  | "loading"
  | "success-single"
  | "success-dual"
  | "success-multi"
  | "partial"
  | "error-network"
  | "error-config-key"
  | "error-config-401"
  | "error-no-selection"
  | "error-no-provider"
  | "error-no-permission"
  | "keystore-corrupt"
  | "offline-fallback"
  | "offline-error"
  | "pinned";

/**
 * S0 §4.3 Provider Center — all 23 states (model-fetch-error and
 * model-manual-entry are separate rows, not merged).
 */
export type ProviderState =
  | "empty"
  | "loading-models"
  | "model-fetch-error"
  | "model-manual-entry"
  | "connection-testing"
  | "connection-ok"
  | "connection-failed"
  | "key-saved"
  | "key-missing"
  | "duplicate"
  | "saving"
  | "save-failed"
  | "save-conflict"
  | "delete-confirm"
  | "deleting"
  | "delete-retry"
  | "drag-reorder"
  | "reorder-failed"
  | "balance-loading"
  | "balance-unsupported"
  | "balance-rate-limited"
  | "balance-error"
  | "endpoint-invalid";

export type LabStrings = {
  appTitle: string;
  appSubtitle: string;
  nav: {
    selectionPopup: string;
    inputWindow: string;
    providerCenter: string;
    ocrOverlay: string;
    history: string;
    trayMenubar: string;
    onboarding: string;
    multiResult: string;
    shortcuts: string;
    privacy: string;
    keystore: string;
    vocabulary: string;
    dictionary: string;
    tts: string;
    externalApi: string;
    updater: string;
  };
  navGroupLabel: string;
  upcomingSlice: string;
  notImplemented: string;
  controls: {
    state: string;
    locale: string;
    localeGroup: string;
    theme: string;
    themeGroup: string;
    themeLight: string;
    themeDark: string;
    motion: string;
    motionGroup: string;
    motionFull: string;
    motionReduced: string;
    windowSize: string;
    windowSizeGroup: string;
    size400x300: string;
    size600x400: string;
  };
  selection: {
    states: Record<SelectionState, string>;
    loading: string;
    initialHidden: string;
    noSelection: string;
    noProvider: string;
    networkError: string;
    configErrorKey: string;
    configError401: string;
    noPermission: string;
    keystoreCorrupt: string;
    offlineError: string;
    pinned: string;
    copy: string;
    copied: string;
    speak: string;
    stop: string;
    pin: string;
    unpin: string;
    favorite: string;
    favorited: string;
    retry: string;
    retrying: string;
    goSettings: string;
    openingSettings: string;
    settingsOpened: string;
    fallbackSuffix: string;
    dualTitle: string;
    multiTitle: string;
    engineA: string;
    engineB: string;
    engineC: string;
  };
  provider: {
    states: Record<ProviderState, string>;
    addFirst: string;
    addFirstDesc: string;
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
    keySaved: string;
    keyMissing: string;
    enterKey: string;
    apiKey: string;
    apiKeyPlaceholder: string;
    saveKey: string;
    saving: string;
    saveFailed: string;
    saveConflict: string;
    reload: string;
    cancel: string;
    deleteConfirmTitle: string;
    deleteConfirmMsg: string;
    delete: string;
    deleting: string;
    deleteRetry: string;
    moveUp: string;
    moveDown: string;
    dragHandle: string;
    reorderReverted: string;
    balanceLoading: string;
    balanceUnsupported: string;
    balanceRateLimited: string;
    balanceError: string;
    endpointInvalid: string;
    setPrimary: string;
    addParallel: string;
    removeParallel: string;
    setFallback: string;
    consentTitle: string;
    consentMsg: string;
    consentConfirm: string;
    consentCancel: string;
    selectPrimary: string;
    duplicate: string;
    primary: string;
    fallback: string;
    parallel: string;
    enabled: string;
    disabled: string;
    endpoint: string;
    name: string;
    // ProviderCard label templates
    cardKeySaved: string;
    cardKeyMissing: string;
    cardEdit: string; // {name} substituted
    cardDelete: string; // {name} substituted
    // aria-labels
    providerListLabel: string;
    detailLabel: string;
    loadingModels: string;
    toastDismiss: string;
    movedUp: string;
    movedDown: string;
    consentLocal: string;
    consentRemote: string;
    // Settings nav rail
    navProviderCenter: string;
    navShortcuts: string;
    navPrivacy: string;
    navSettings: string;
    presetOllama: string;
    frameMin: string;
    frameDefault: string;
  };
};

export const strings: Record<Locale, LabStrings> = {
  en: {
    appTitle: "LinguaRay · UI Lab",
    appSubtitle: "Clickable mock prototypes (mock data only)",
    nav: {
      selectionPopup: "Selection Popup",
      inputWindow: "Input Window",
      providerCenter: "Provider Center",
      ocrOverlay: "OCR Overlay",
      history: "History",
      trayMenubar: "Tray / Menu-bar",
      onboarding: "Onboarding",
      multiResult: "Multi-Result Panel",
      shortcuts: "Shortcuts",
      privacy: "Privacy & Data",
      keystore: "Keystore Recovery",
      vocabulary: "Vocabulary",
      dictionary: "Dictionary",
      tts: "TTS",
      externalApi: "External API",
      updater: "Updater",
    },
    navGroupLabel: "Prototypes",
    upcomingSlice: "Upcoming slice",
    notImplemented: "Not yet implemented",
    controls: {
      state: "State",
      locale: "Language",
      localeGroup: "Language",
      theme: "Theme",
      themeGroup: "Theme",
      themeLight: "Light",
      themeDark: "Dark",
      motion: "Motion",
      motionGroup: "Motion",
      motionFull: "Full",
      motionReduced: "Reduced",
      windowSize: "Window size",
      windowSizeGroup: "Window size",
      size400x300: "400×300 (single)",
      size600x400: "600×400 (expanded)",
    },
    selection: {
      states: {
        "initial-hidden": "Initial (hidden)",
        loading: "Loading",
        "success-single": "Success · single",
        "success-dual": "Success · 2 engines",
        "success-multi": "Success · 3 engines",
        partial: "Partial success",
        "error-network": "Error · network",
        "error-config-key": "Error · API key missing",
        "error-config-401": "Error · 401 Unauthorized",
        "error-no-selection": "Error · no selection",
        "error-no-provider": "Error · no provider",
        "error-no-permission": "Error · no permission",
        "keystore-corrupt": "Keystore corrupt",
        "offline-fallback": "Offline · engine fallback",
        "offline-error": "Offline · no fallback",
        pinned: "Pinned",
      },
      loading: "Translating…",
      initialHidden: "Popup hidden until a selection is made",
      noSelection: "No text selected",
      noProvider: "No translation provider configured",
      networkError: "Network error",
      configErrorKey: "API key missing",
      configError401: "401 Unauthorized — check your API key",
      noPermission: "Grant Accessibility permission",
      keystoreCorrupt: "Keystore unreadable",
      offlineError: "Offline",
      pinned: "Pinned",
      copy: "Copy",
      copied: "Copied",
      speak: "Speak",
      stop: "Stop",
      pin: "Pin",
      unpin: "Unpin",
      favorite: "Add to vocabulary",
      favorited: "Added to vocabulary",
      retry: "Retry",
      retrying: "Retrying…",
      goSettings: "Open settings",
      openingSettings: "Opening settings…",
      settingsOpened: "Settings opened (mock)",
      fallbackSuffix: "offline fallback",
      dualTitle: "Dual-engine result",
      multiTitle: "Multi-engine result",
      engineA: "DeepSeek",
      engineB: "OpenAI",
      engineC: "Google",
    },
    provider: {
      states: {
        empty: "Empty (no providers)",
        "loading-models": "Loading models",
        "model-fetch-error": "Model fetch error",
        "model-manual-entry": "Model manual entry",
        "connection-testing": "Connection testing",
        "connection-ok": "Connection OK",
        "connection-failed": "Connection failed",
        "key-saved": "Key saved",
        "key-missing": "Key missing",
        duplicate: "Duplicate",
        saving: "Saving",
        "save-failed": "Save failed",
        "save-conflict": "Save conflict",
        "delete-confirm": "Delete confirm",
        deleting: "Deleting",
        "delete-retry": "Delete retry",
        "drag-reorder": "Drag to reorder",
        "reorder-failed": "Reorder failed",
        "balance-loading": "Balance loading",
        "balance-unsupported": "Balance unsupported",
        "balance-rate-limited": "Balance rate-limited",
        "balance-error": "Balance error",
        "endpoint-invalid": "Endpoint invalid",
      },
      addFirst: "Add your first provider",
      addFirstDesc: "Pick a preset, enter your API key, and start translating.",
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
      keySaved: "Key saved",
      keyMissing: "Key missing",
      enterKey: "Enter key",
      apiKey: "API key",
      apiKeyPlaceholder: "sk-…",
      saveKey: "Save key",
      saving: "Saving…",
      saveFailed: "Failed to save: network error",
      saveConflict: "This provider was modified elsewhere. Reload?",
      reload: "Reload",
      cancel: "Cancel",
      deleteConfirmTitle: "Delete provider?",
      deleteConfirmMsg: "History references are preserved.",
      delete: "Delete",
      deleting: "Deleting…",
      deleteRetry: "Delete failed — retry?",
      moveUp: "Move up",
      moveDown: "Move down",
      dragHandle: "Drag to reorder",
      reorderReverted: "Failed to save order — reverted",
      balanceLoading: "Loading balance…",
      balanceUnsupported: "—",
      balanceRateLimited: "Rate limited — try later",
      balanceError: "Error fetching balance",
      endpointInvalid: "Must be HTTPS (or localhost)",
      setPrimary: "Set as primary",
      addParallel: "Add to parallel",
      removeParallel: "Remove from parallel",
      setFallback: "Set as fallback",
      consentTitle: "Send text to multiple providers?",
      consentMsg: "Your text will be sent to the following providers:",
      consentConfirm: "Confirm",
      consentCancel: "Cancel",
      selectPrimary: "Select a primary provider",
      duplicate: "Duplicate",
      primary: "Primary",
      fallback: "Fallback",
      parallel: "Parallel",
      enabled: "Enabled",
      disabled: "Disabled",
      endpoint: "Endpoint",
      name: "Name",
      cardKeySaved: "Key saved",
      cardKeyMissing: "Key missing",
      cardEdit: "Edit {name}",
      cardDelete: "Delete {name}",
      providerListLabel: "Provider list",
      detailLabel: "Provider detail",
      loadingModels: "Loading models…",
      toastDismiss: "Dismiss",
      movedUp: "moved up",
      movedDown: "moved down",
      consentLocal: "local",
      consentRemote: "remote",
      navProviderCenter: "Provider Center",
      navShortcuts: "Shortcuts",
      navPrivacy: "Privacy & Data",
      navSettings: "Settings navigation",
      presetOllama: "Ollama",
      frameMin: "600×400 (min)",
      frameDefault: "800×600 (default)",
    },
  },
  zh: {
    appTitle: "LinguaRay · UI 实验室",
    appSubtitle: "可点击原型（仅模拟数据）",
    nav: {
      selectionPopup: "划词翻译弹窗",
      inputWindow: "输入窗口",
      providerCenter: "服务商中心",
      ocrOverlay: "OCR 覆盖层",
      history: "历史记录",
      trayMenubar: "托盘 / 菜单栏",
      onboarding: "新手引导",
      multiResult: "多引擎结果面板",
      shortcuts: "快捷键",
      privacy: "隐私与数据",
      keystore: "密钥库恢复",
      vocabulary: "生词本",
      dictionary: "词典",
      tts: "朗读",
      externalApi: "外部 API",
      updater: "更新",
    },
    navGroupLabel: "原型列表",
    upcomingSlice: "后续切片",
    notImplemented: "尚未实现",
    controls: {
      state: "状态",
      locale: "语言",
      localeGroup: "语言",
      theme: "主题",
      themeGroup: "主题",
      themeLight: "浅色",
      themeDark: "深色",
      motion: "动效",
      motionGroup: "动效",
      motionFull: "完整",
      motionReduced: "减弱",
      windowSize: "窗口尺寸",
      windowSizeGroup: "窗口尺寸",
      size400x300: "400×300（单结果）",
      size600x400: "600×400（展开）",
    },
    selection: {
      states: {
        "initial-hidden": "初始（隐藏）",
        loading: "加载中",
        "success-single": "成功 · 单引擎",
        "success-dual": "成功 · 双引擎",
        "success-multi": "成功 · 三引擎",
        partial: "部分成功",
        "error-network": "错误 · 网络异常",
        "error-config-key": "错误 · 缺少 API 密钥",
        "error-config-401": "错误 · 401 未授权",
        "error-no-selection": "错误 · 未选中文本",
        "error-no-provider": "错误 · 无可用服务商",
        "error-no-permission": "错误 · 缺少权限",
        "keystore-corrupt": "密钥库损坏",
        "offline-fallback": "离线 · 引擎回退",
        "offline-error": "离线 · 无回退",
        pinned: "已固定",
      },
      loading: "正在翻译…",
      initialHidden: "未选中文本前弹窗保持隐藏",
      noSelection: "未选中文本",
      noProvider: "未配置翻译服务",
      networkError: "网络错误",
      configErrorKey: "缺少 API 密钥",
      configError401: "401 未授权 —— 请检查 API 密钥",
      noPermission: "请授予辅助功能权限",
      keystoreCorrupt: "密钥库无法读取",
      offlineError: "离线",
      pinned: "已固定",
      copy: "复制",
      copied: "已复制",
      speak: "朗读",
      stop: "停止",
      pin: "固定",
      unpin: "取消固定",
      favorite: "加入生词本",
      favorited: "已加入生词本",
      retry: "重试",
      retrying: "重试中…",
      goSettings: "打开设置",
      openingSettings: "正在打开设置…",
      settingsOpened: "设置已打开（模拟）",
      fallbackSuffix: "离线回退",
      dualTitle: "双引擎结果",
      multiTitle: "多引擎结果",
      engineA: "DeepSeek",
      engineB: "OpenAI",
      engineC: "Google",
    },
    provider: {
      states: {
        empty: "空（无服务商）",
        "loading-models": "加载模型",
        "model-fetch-error": "模型获取失败",
        "model-manual-entry": "手动输入模型",
        "connection-testing": "连接测试中",
        "connection-ok": "连接成功",
        "connection-failed": "连接失败",
        "key-saved": "密钥已保存",
        "key-missing": "缺少密钥",
        duplicate: "复制",
        saving: "保存中",
        "save-failed": "保存失败",
        "save-conflict": "保存冲突",
        "delete-confirm": "删除确认",
        deleting: "删除中",
        "delete-retry": "删除重试",
        "drag-reorder": "拖拽排序",
        "reorder-failed": "排序失败",
        "balance-loading": "余额加载中",
        "balance-unsupported": "不支持余额",
        "balance-rate-limited": "余额限流",
        "balance-error": "余额错误",
        "endpoint-invalid": "端点无效",
      },
      addFirst: "添加你的第一个服务商",
      addFirstDesc: "选择预设，输入 API 密钥，即可开始翻译。",
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
      keySaved: "密钥已保存",
      keyMissing: "缺少密钥",
      enterKey: "输入密钥",
      apiKey: "API 密钥",
      apiKeyPlaceholder: "sk-…",
      saveKey: "保存密钥",
      saving: "保存中…",
      saveFailed: "保存失败：网络错误",
      saveConflict: "此服务商已在其他位置修改。重新加载？",
      reload: "重新加载",
      cancel: "取消",
      deleteConfirmTitle: "删除服务商？",
      deleteConfirmMsg: "历史引用将被保留。",
      delete: "删除",
      deleting: "删除中…",
      deleteRetry: "删除失败 —— 重试？",
      moveUp: "上移",
      moveDown: "下移",
      dragHandle: "拖拽排序",
      reorderReverted: "排序保存失败 —— 已恢复",
      balanceLoading: "正在加载余额…",
      balanceUnsupported: "—",
      balanceRateLimited: "限流 —— 请稍后重试",
      balanceError: "获取余额出错",
      endpointInvalid: "必须为 HTTPS（或 localhost）",
      setPrimary: "设为主引擎",
      addParallel: "加入并行",
      removeParallel: "移出并行",
      setFallback: "设为回退",
      consentTitle: "将文本发送给多个服务商？",
      consentMsg: "你的文本将发送给以下服务商：",
      consentConfirm: "确认",
      consentCancel: "取消",
      selectPrimary: "请选择主引擎",
      duplicate: "复制",
      primary: "主引擎",
      fallback: "回退",
      parallel: "并行",
      enabled: "已启用",
      disabled: "已禁用",
      endpoint: "端点",
      name: "名称",
      cardKeySaved: "密钥已保存",
      cardKeyMissing: "缺少密钥",
      cardEdit: "编辑{name}",
      cardDelete: "删除{name}",
      providerListLabel: "服务商列表",
      detailLabel: "服务商详情",
      loadingModels: "正在加载模型…",
      toastDismiss: "关闭",
      movedUp: "已上移",
      movedDown: "已下移",
      consentLocal: "本地",
      consentRemote: "远程",
      navProviderCenter: "服务商中心",
      navShortcuts: "快捷键",
      navPrivacy: "隐私与数据",
      navSettings: "设置导航",
      presetOllama: "Ollama",
      frameMin: "600×400（最小）",
      frameDefault: "800×600（默认）",
    },
  },
};
