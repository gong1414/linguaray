import type { Locale } from "../../app/i18n";
import type { SettingsSection } from "./model";

export type ShellCopy = {
  windowTitle: string;
  nav: Record<SettingsSection, string>;
  navLabel: string;
  a11y: {
    title: string;
    hint: string;
    recheck: string;
    openSettings: string;
  };
};

const EN: ShellCopy = {
  windowTitle: "LinguaRay",
  nav: {
    "provider-center": "Provider Center",
    "keystore-recovery": "Keystore Recovery",
    shortcuts: "Shortcuts",
    privacy: "Privacy",
    history: "History",
    vocabulary: "Vocabulary",
    dictionary: "Dictionary",
    updater: "Updater",
  },
  navLabel: "Settings sections",
  a11y: {
    title: "Accessibility permission needed",
    hint: "LinguaRay needs the macOS Accessibility permission to capture selected text. Grant it in System Settings, then re-check.",
    recheck: "Re-check",
    openSettings: "System Settings",
  },
};

const ZH: ShellCopy = {
  windowTitle: "LinguaRay",
  nav: {
    "provider-center": "服务商中心",
    "keystore-recovery": "密钥库恢复",
    shortcuts: "快捷键",
    privacy: "隐私",
    history: "历史",
    vocabulary: "生词本",
    dictionary: "词典",
    updater: "检查更新",
  },
  navLabel: "设置分区",
  a11y: {
    title: "需要辅助功能权限",
    hint: "LinguaRay 需要辅助功能权限才能捕获选中文本。请在系统设置中授权，然后重新检查。",
    recheck: "重新检查",
    openSettings: "系统设置",
  },
};

export const SHELL_COPY: Record<Locale, ShellCopy> = { en: EN, zh: ZH };
