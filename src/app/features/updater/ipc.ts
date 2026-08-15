/** Updater IPC wrappers (fail-closed payload guards). */
import { invoke } from "../../../bridge/invoke";
import { relaunch } from "../../../bridge/process";
import { isUpdateCheck, isUpdaterProgress, type UpdateCheck, type UpdaterProgress } from "./model";

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
export const onUpdaterProgress = (cb: (progress: UpdaterProgress) => void): Promise<() => void> =>
  import("../../../bridge/event")
    .then(({ listen }) =>
      listen<unknown>("updater-progress", (event) => {
        if (isUpdaterProgress(event.payload)) cb(event.payload);
      }),
    )
    .catch(() => Promise.resolve(() => {}));

export const relaunchApp = (): Promise<void> => relaunch().catch(() => {});
