import type { Locale } from "../../app/i18n";

export type KsState = "healthy" | "corrupt" | "archived";
export type KsToastEntry = { id: number; variant: "info" | "success" | "warning" | "destructive"; message: string };

export type KeystoreCopy = {
  pageTitle: string;
  healthy: string;
  title: string;
  description: string;
  archive: string;
  reset: string;
  archivedTitle: string;
  archivedPrompt: string;
  resetConfirmTitle: string;
  resetConfirmMessage: string;
  resetConfirmConfirmLabel: string;
  resetConfirmCancelLabel: string;
  archiveFailed: string;
  resetFailed: string;
};

const EN: KeystoreCopy = {
  pageTitle: "Keystore Recovery",
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
};

const ZH: KeystoreCopy = {
  pageTitle: "密钥库恢复",
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
};

export const KEYSTORE_COPY: Record<Locale, KeystoreCopy> = { en: EN, zh: ZH };
