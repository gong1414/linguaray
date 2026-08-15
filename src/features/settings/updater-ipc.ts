/**
 * Updater IPC wrappers. All Tauri bridges are lazily imported so jsdom tests
 * and non-Tauri contexts never touch the native layer at module load.
 */
import { invoke } from "../../bridge/invoke";
import { isUpdateCheck, isUpdaterProgress, type UpdateCheck, type UpdaterProgress } from "./updater-types";

const requireCheck = (value: unknown): UpdateCheck => {
  if (!isUpdateCheck(value)) {
    throw new Error("updater command returned an invalid payload");
  }
  return value;
};

export const updaterCheck = async (): Promise<UpdateCheck> =>
  requireCheck(await invoke<unknown>("updater_check"));

export const updaterDownloadInstall = async (): Promise<UpdateCheck> =>
  requireCheck(await invoke<unknown>("updater_download_install"));

export type AppSettings = {
  default_provider: string;
  target_language: string;
  fallback_engine: string | null;
  check_updates_on_startup: boolean;
};

export const getUpdaterStartupCheck = async (): Promise<boolean> => {
  const s = await invoke<unknown>("get_settings");
  if (typeof s !== "object" || s === null) return true;
  const flag = (s as Record<string, unknown>).check_updates_on_startup;
  // Absent key = default ON (mirrors the Rust load() fallback).
  return typeof flag === "boolean" ? flag : true;
};

export const setUpdaterStartupCheck = async (enabled: boolean): Promise<void> => {
  await invoke("set_setting", {
    key: "check_updates_on_startup",
    value: enabled ? "true" : "false",
  });
};

/** Subscribe to backend `updater-progress` events. Returns an unlisten fn. */
export const onUpdaterProgress = (
  cb: (progress: UpdaterProgress) => void,
): Promise<() => void> =>
  import("../../bridge/event")
    .then(({ listen }) =>
      listen<unknown>("updater-progress", (event) => {
        if (isUpdaterProgress(event.payload)) cb(event.payload);
      }),
    )
    .catch(() => Promise.resolve(() => {}));

/** Relaunch after an in-place update (macOS/Linux path). */
export const relaunchApp = (): Promise<void> =>
  import("../../bridge/process")
    .then(({ relaunch }) => relaunch())
    .catch(() => {});
