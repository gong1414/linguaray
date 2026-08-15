import type { Locale } from "../../i18n";
import type { ShortcutAction } from "./model";

export type ShortcutsCopy = {
  title: string;
  loading: string;
  loadFailed: string;
  retry: string;
  actions: Record<ShortcutAction, string>;
  change: string;
  changeLabel: string;
  recordingPrompt: string;
  cancel: string;
  conflictMessage: string;
  override: string;
  registrationFailed: string;
  unavailable: string;
  resetDefaults: string;
  resetConfirmTitle: string;
  resetConfirmMessage: string;
  useDefaults: string;
  resetFailed: string;
  saveFailed: string;
};

const EN: ShortcutsCopy = {
  title: "Keyboard Shortcuts",
  loading: "Loading shortcuts…",
  loadFailed: "Shortcuts couldn't be loaded.",
  retry: "Retry",
  actions: {
    translate_selection: "Translate Selection",
    translate_input: "Translate Input",
    translate_clipboard: "Translate Clipboard",
    ocr_translate: "OCR Translate",
  },
  change: "Change",
  changeLabel: "Change {action}",
  recordingPrompt: "Press a key combo…",
  cancel: "Cancel",
  conflictMessage: "Conflicts with {action}",
  override: "Override",
  registrationFailed: "This combo couldn't be registered (system reserved)",
  unavailable: "Unavailable on this system",
  resetDefaults: "Reset to Defaults",
  resetConfirmTitle: "Reset keyboard shortcuts?",
  resetConfirmMessage: "Your custom shortcuts will be replaced with the defaults.",
  useDefaults: "Use Defaults",
  resetFailed: "Shortcuts couldn't be reset. Try again.",
  saveFailed: "The shortcut couldn't be saved. Try again.",
};

const ZH: ShortcutsCopy = {
  title: "键盘快捷键",
  loading: "正在加载快捷键…",
  loadFailed: "无法加载快捷键。",
  retry: "重试",
  actions: {
    translate_selection: "翻译选区",
    translate_input: "输入翻译",
    translate_clipboard: "翻译剪贴板",
    ocr_translate: "OCR 翻译",
  },
  change: "修改",
  changeLabel: "修改{action}",
  recordingPrompt: "按下组合键…",
  cancel: "取消",
  conflictMessage: "与 {action} 冲突",
  override: "覆盖",
  registrationFailed: "此组合无法注册（系统保留）",
  unavailable: "此系统上不可用",
  resetDefaults: "恢复默认",
  resetConfirmTitle: "恢复默认快捷键？",
  resetConfirmMessage: "你的自定义快捷键将被默认值替换。",
  useDefaults: "使用默认",
  resetFailed: "无法恢复快捷键，请重试。",
  saveFailed: "无法保存快捷键，请重试。",
};

export const SHORTCUTS_COPY: Record<Locale, ShortcutsCopy> = { en: EN, zh: ZH };
