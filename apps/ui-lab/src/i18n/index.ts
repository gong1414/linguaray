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
  },
};
