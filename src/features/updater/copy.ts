import type { Locale } from "../../app/i18n";

/** Updater copy (zh/en). The privacy note is load-bearing: the startup check
 *  is the app's ONLY unsolicited network request and must stay disclosed. */
export type UpdaterCopy = {
  title: string;
  currentVersion: string;
  autoCheckLabel: string;
  autoCheckHint: string;
  status: { checking: string; upToDate: string; available: string; errorPrefix: string };
  action: { checkAgain: string; downloadInstall: string; relaunch: string };
  progress: { downloading: string; unknownSize: string; installing: string; installedHint: string };
  releaseNotes: string;
};

const EN: UpdaterCopy = {
  title: "Updates",
  currentVersion: "Current version",
  autoCheckLabel: "Check for updates on startup",
  autoCheckHint:
    "The only network request LinguaRay makes on its own (to GitHub Releases). Turn off for fully offline use.",
  status: {
    checking: "Checking for updates…",
    upToDate: "You are on the latest version.",
    available: "A new version is available.",
    errorPrefix: "Update check failed",
  },
  action: {
    checkAgain: "Check for Updates",
    downloadInstall: "Download & Install",
    relaunch: "Relaunch to Finish",
  },
  progress: {
    downloading: "Downloading",
    unknownSize: "Downloading…",
    installing: "Installing…",
    installedHint: "Installed — relaunch to finish the update.",
  },
  releaseNotes: "Release notes",
};

const ZH: UpdaterCopy = {
  title: "检查更新",
  currentVersion: "当前版本",
  autoCheckLabel: "启动时自动检查更新",
  autoCheckHint:
    "这是 LinguaRay 唯一主动发出的网络请求（访问 GitHub Releases）。如需完全离线可关闭。",
  status: {
    checking: "正在检查更新…",
    upToDate: "已是最新版本。",
    available: "发现新版本。",
    errorPrefix: "检查更新失败",
  },
  action: {
    checkAgain: "检查更新",
    downloadInstall: "下载并安装",
    relaunch: "重启以完成更新",
  },
  progress: {
    downloading: "正在下载",
    unknownSize: "正在下载…",
    installing: "正在安装…",
    installedHint: "安装完成 — 重启应用以完成更新。",
  },
  releaseNotes: "更新说明",
};

export const UPDATER_COPY: Record<Locale, UpdaterCopy> = { en: EN, zh: ZH };

/** Convert backend/network exception text into stable user-facing copy. */
export function updaterErrorText(locale: Locale, error: unknown): string {
  const raw = String(error).replace(/^Error:\s*/i, "").trim().toLowerCase();
  if (locale === "zh") {
    if (raw.includes("valid release json") || raw.includes("invalid payload")) {
      return "无法读取更新信息，请稍后重试。";
    }
    if (
      raw.includes("offline") ||
      raw.includes("network") ||
      raw.includes("fetch") ||
      raw.includes("timeout") ||
      raw.includes("connection")
    ) {
      return "无法连接更新服务器，请检查网络后重试。";
    }
    return "更新操作失败，请稍后重试。";
  }
  if (raw.includes("valid release json") || raw.includes("invalid payload")) {
    return "The update information could not be read. Please try again later.";
  }
  if (
    raw.includes("offline") ||
    raw.includes("network") ||
    raw.includes("fetch") ||
    raw.includes("timeout") ||
    raw.includes("connection")
  ) {
    return "Could not reach the update server. Check your connection and try again.";
  }
  return "The update operation failed. Please try again later.";
}
