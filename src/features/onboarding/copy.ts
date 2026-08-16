/**
 * Onboarding copy (zh/en), migrated 1:1 from the frozen Solid tree so no user
 * -visible string changes during the React migration. Status strings stay
 * HONEST: an ungranted permission is never labeled done, and the escape hatch
 * is explicitly "later" rather than a fake Continue.
 */

import type { Locale } from "../../app/i18n";
export type { Locale };

export type OnboardingCopy = {
  brand: string;
  stepLabels: Record<import("./model").OnboardingStepName, string>;
  welcome: { title: string; body: string; start: string };
  a11y: {
    title: string;
    body: string;
    openSettings: string;
    openScreenSettings: string;
    recheck: string;
    screenTitle: string;
    screenBody: string;
    status: { checking: string; granted: string; denied: string; error: string; unsupported: string };
    later: string;
    continue: string;
  };
  provider: {
    title: string;
    body: string;
    noneBody: string;
    openSettings: string;
    count: (n: number) => string;
    checking: string;
    continue: string;
  };
  history: {
    title: string;
    body: string;
    enable: string;
    enabling: string;
    skip: string;
  };
  shortcuts: {
    title: string;
    body: string;
    openSettings: string;
    combos: Record<string, string>;
    continue: string;
  };
  done: { title: string; body: string; openApp: string; tray: string };
  errorPrefix: string;
};

const en: OnboardingCopy = {
  brand: "LinguaRay",
  stepLabels: {
    welcome: "Welcome",
    accessibility: "Permissions",
    provider: "Provider",
    history: "History",
    shortcuts: "Shortcuts",
    done: "Done",
  },
  welcome: {
    title: "Welcome to LinguaRay",
    body: "A privacy-first, AI-native translation tool for your menu bar. This takes about a minute.",
    start: "Get started",
  },
  a11y: {
    title: "Grant permissions",
    body: "Accessibility lets selection translate read highlighted text; Screen Recording lets region OCR see the screen.",
    openSettings: "Open Accessibility Settings",
    openScreenSettings: "Open Screen Recording Settings",
    recheck: "Re-check",
    screenTitle: "Screen Recording (OCR)",
    screenBody: "Needed only for OCR region capture. Skip if you don't use OCR.",
    status: {
      checking: "Checking…",
      granted: "Granted",
      denied: "Not granted",
      error: "Check failed",
      unsupported: "Not needed on this platform",
    },
    later: "Set up later",
    continue: "Continue",
  },
  provider: {
    title: "Add a translation provider",
    body: "Pick a preset (OpenAI, Anthropic, Gemini, local Ollama…), paste an API key, done.",
    noneBody: "No provider yet — add one and come back here.",
    openSettings: "Open Provider Settings",
    count: (n) => `${n} provider${n === 1 ? "" : "s"} configured`,
    checking: "Checking…",
    continue: "Continue",
  },
  history: {
    title: "Encrypted history",
    body: "Translations can be stored locally, AES-256 encrypted. You can change this any time in Settings.",
    enable: "Enable history",
    enabling: "Enabling…",
    skip: "Skip for now",
  },
  shortcuts: {
    title: "Shortcuts",
    body: "Defaults below — customize any time in Settings.",
    openSettings: "Open Shortcut Settings",
    combos: {
      translate_selection: "Translate selection",
      translate_input: "Input translate",
      translate_clipboard: "Translate clipboard",
      ocr_translate: "OCR translate",
    },
    continue: "Finish setup",
  },
  done: {
    title: "You're all set!",
    body: "LinguaRay lives in your menu bar. Select text anywhere and press the shortcut to translate.",
    openApp: "Open settings",
    tray: "Start using LinguaRay",
  },
  errorPrefix: "Something went wrong",
};

const zh: OnboardingCopy = {
  brand: "LinguaRay",
  stepLabels: {
    welcome: "欢迎",
    accessibility: "权限",
    provider: "服务商",
    history: "历史记录",
    shortcuts: "快捷键",
    done: "完成",
  },
  welcome: {
    title: "欢迎使用 LinguaRay",
    body: "一款隐私优先、AI 原生的菜单栏翻译工具。设置只需一分钟左右。",
    start: "开始使用",
  },
  a11y: {
    title: "授予权限",
    body: "辅助功能用于划词翻译读取选中文本；屏幕录制用于 OCR 区域识别。",
    openSettings: "打开辅助功能设置",
    openScreenSettings: "打开屏幕录制设置",
    recheck: "重新检查",
    screenTitle: "屏幕录制（OCR）",
    screenBody: "仅 OCR 截图识别需要。不用 OCR 可以跳过。",
    status: {
      checking: "检查中…",
      granted: "已授权",
      denied: "未授权",
      error: "检查失败",
      unsupported: "此平台无需此权限",
    },
    later: "稍后设置",
    continue: "继续",
  },
  provider: {
    title: "添加翻译服务商",
    body: "选择预设（OpenAI、Anthropic、Gemini、本地 Ollama……），填入 API Key 即可。",
    noneBody: "还没有服务商——添加后回到这里继续。",
    openSettings: "打开服务商设置",
    count: (n) => `已配置 ${n} 个服务商`,
    checking: "检查中…",
    continue: "继续",
  },
  history: {
    title: "加密历史记录",
    body: "翻译记录可以本地保存（AES-256 加密）。之后随时可在设置中更改。",
    enable: "启用历史记录",
    enabling: "正在启用…",
    skip: "暂不启用",
  },
  shortcuts: {
    title: "快捷键",
    body: "以下为默认快捷键——可随时在设置中自定义。",
    openSettings: "打开快捷键设置",
    combos: {
      translate_selection: "划词翻译",
      translate_input: "输入翻译",
      translate_clipboard: "剪贴板翻译",
      ocr_translate: "OCR 翻译",
    },
    continue: "完成设置",
  },
  done: {
    title: "设置完成！",
    body: "LinguaRay 常驻菜单栏。在任何应用里选中文本、按下快捷键即可翻译。",
    openApp: "打开设置",
    tray: "开始使用 LinguaRay",
  },
  errorPrefix: "出错了",
};

export const ONBOARDING_COPY: Record<Locale, OnboardingCopy> = { en, zh };
