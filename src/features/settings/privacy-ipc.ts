import { invoke } from "@tauri-apps/api/core";
import {
  isHistoryPrivacyStatus,
  type HistoryPrivacyStatus,
  type HistoryRetentionDays,
} from "./privacy-types";

const requireStatus = (value: unknown): HistoryPrivacyStatus => {
  if (!isHistoryPrivacyStatus(value)) {
    throw new Error("history_privacy_status returned an invalid payload");
  }
  return value;
};

export const historyPrivacyStatus = async (): Promise<HistoryPrivacyStatus> =>
  requireStatus(await invoke<unknown>("history_privacy_status"));

export const historySetEnabled = async (enabled: boolean): Promise<HistoryPrivacyStatus> =>
  requireStatus(await invoke<unknown>("history_set_enabled", { enabled }));

export const historySetRetention = async (
  days: HistoryRetentionDays,
): Promise<HistoryPrivacyStatus> =>
  requireStatus(await invoke<unknown>("history_set_retention", { days }));

export const historyClearAll = async (): Promise<HistoryPrivacyStatus> =>
  requireStatus(await invoke<unknown>("history_clear_all"));
