/**
 * Typed wrappers for the privacy/external-API Rust commands. Fail-closed:
 * payloads are validated with the model guards before reaching the view.
 */
import { invoke } from "../../../bridge/invoke";
import {
  isHistoryPrivacyStatus,
  type ExternalApiStatus,
  type HistoryPrivacyStatus,
  type HistoryRetentionDays,
} from "./model";

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

export const externalApiStatus = (): Promise<ExternalApiStatus> =>
  invoke<ExternalApiStatus>("external_api_status");

export const externalApiEnable = (): Promise<string> =>
  invoke<string>("external_api_enable", { port: null });

export const externalApiDisable = (): Promise<void> => invoke<void>("external_api_disable");

export const externalApiRegenerateToken = (): Promise<string> =>
  invoke<string>("external_api_regenerate_token");
