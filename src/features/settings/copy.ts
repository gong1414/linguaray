/**
 * Settings shell + Keystore Recovery (Surface 06) copy dictionary.
 *
 * Locale comes from `src/i18n.ts` `detectLocale()`. No raw user-facing
 * literals in components — only copy-key lookups. The `{name}` / `{reason}` /
 * `{latency}` / `{message}` placeholders are substituted at the call site via
 * `String.prototype.replace` (NOT template literals) so this dictionary stays
 * the single source of truth.
 *
 * Task 2 extends `SETTINGS_COPY` with the `provider` group (Surface 05).
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

export type SettingsCopy = {
  window: WindowCopy;
  nav: NavCopy;
  keystore: KeystoreCopy;
};

export const SETTINGS_COPY: Record<Locale, SettingsCopy> = {
  zh: {
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
  },
  en: {
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
  },
};
