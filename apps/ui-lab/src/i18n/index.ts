/**
 * UI Lab i18n — minimal dictionary (zh/en). Not the production i18n layer; this
 * only exists so the lab can exercise both languages against the same screens.
 * Production translation strings live in the spec (S0) and are localized there.
 */

export type Locale = "zh" | "en";

export type LabStrings = {
  appTitle: string;
  appSubtitle: string;
  nav: {
    selectionPopup: string;
    inputWindow: string;
    providerCenter: string;
    ocrOverlay: string;
    history: string;
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
  controls: {
    state: string;
    locale: string;
    theme: string;
    themeLight: string;
    themeDark: string;
    motion: string;
    motionFull: string;
    motionReduced: string;
    windowSize: string;
    size400x300: string;
    size600x400: string;
  };
  selection: {
    // S0 §4.1 states
    states: Record<SelectionState, string>;
    loading: string;
    noSelection: string;
    noProvider: string;
    networkError: string;
    configError: string;
    noPermission: string;
    keystoreCorrupt: string;
    offline: string;
    pinned: string;
    copy: string;
    speak: string;
    pin: string;
    unpin: string;
    favorite: string;
    retry: string;
    goSettings: string;
    dualTitle: string;
    multiTitle: string;
    engineA: string;
    engineB: string;
    engineC: string;
    engineFallback: string;
  };
};

export type SelectionState =
  | "loading"
  | "success-single"
  | "success-dual"
  | "success-multi"
  | "partial"
  | "error-network"
  | "error-config"
  | "error-no-selection"
  | "error-no-provider"
  | "error-no-permission"
  | "keystore-corrupt"
  | "offline"
  | "pinned";

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
    controls: {
      state: "State",
      locale: "Language",
      theme: "Theme",
      themeLight: "Light",
      themeDark: "Dark",
      motion: "Motion",
      motionFull: "Full",
      motionReduced: "Reduced",
      windowSize: "Window size",
      size400x300: "400×300 (single)",
      size600x400: "600×400 (expanded)",
    },
    selection: {
      states: {
        loading: "Loading",
        "success-single": "Success · single",
        "success-dual": "Success · 2 engines",
        "success-multi": "Success · 3 engines",
        partial: "Partial success",
        "error-network": "Error · network",
        "error-config": "Error · API key missing",
        "error-no-selection": "Error · no selection",
        "error-no-provider": "Error · no provider",
        "error-no-permission": "Error · no permission",
        "keystore-corrupt": "Keystore corrupt",
        offline: "Offline",
        pinned: "Pinned",
      },
      loading: "Translating…",
      noSelection: "No text selected",
      noProvider: "No translation provider configured",
      networkError: "Network error",
      configError: "API key missing",
      noPermission: "Grant Accessibility permission",
      keystoreCorrupt: "Keystore unreadable",
      offline: "Offline",
      pinned: "Pinned",
      copy: "Copy",
      speak: "Speak",
      pin: "Pin",
      unpin: "Unpin",
      favorite: "Add to vocabulary",
      retry: "Retry",
      goSettings: "Open settings",
      dualTitle: "Dual-engine result",
      multiTitle: "Multi-engine result",
      engineA: "DeepSeek",
      engineB: "OpenAI",
      engineC: "Google",
      engineFallback: "Offline dictionary",
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
    controls: {
      state: "状态",
      locale: "语言",
      theme: "主题",
      themeLight: "浅色",
      themeDark: "深色",
      motion: "动效",
      motionFull: "完整",
      motionReduced: "减弱",
      windowSize: "窗口尺寸",
      size400x300: "400×300（单结果）",
      size600x400: "600×400（展开）",
    },
    selection: {
      states: {
        loading: "加载中",
        "success-single": "成功 · 单引擎",
        "success-dual": "成功 · 双引擎",
        "success-multi": "成功 · 三引擎",
        partial: "部分成功",
        "error-network": "错误 · 网络异常",
        "error-config": "错误 · 缺少 API 密钥",
        "error-no-selection": "错误 · 未选中文本",
        "error-no-provider": "错误 · 无可用服务商",
        "error-no-permission": "错误 · 缺少权限",
        "keystore-corrupt": "密钥库损坏",
        offline: "离线",
        pinned: "已固定",
      },
      loading: "正在翻译…",
      noSelection: "未选中文本",
      noProvider: "未配置翻译服务",
      networkError: "网络错误",
      configError: "缺少 API 密钥",
      noPermission: "请授予辅助功能权限",
      keystoreCorrupt: "密钥库无法读取",
      offline: "离线",
      pinned: "已固定",
      copy: "复制",
      speak: "朗读",
      pin: "固定",
      unpin: "取消固定",
      favorite: "加入生词本",
      retry: "重试",
      goSettings: "打开设置",
      dualTitle: "双引擎结果",
      multiTitle: "多引擎结果",
      engineA: "DeepSeek",
      engineB: "OpenAI",
      engineC: "Google",
      engineFallback: "离线词典",
    },
  },
};
