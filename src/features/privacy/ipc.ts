/**
 * Typed wrappers for the privacy/external-API Rust commands. Fail-closed:
 * payloads are validated with the model guards before reaching the view.
 */
import { commands } from "../../bridge/invoke";
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
  requireStatus(await commands.historyPrivacyStatus());

export const historySetEnabled = async (enabled: boolean): Promise<HistoryPrivacyStatus> =>
  requireStatus(await commands.historySetEnabled(enabled));

export const historySetRetention = async (
  days: HistoryRetentionDays,
): Promise<HistoryPrivacyStatus> =>
  requireStatus(await commands.historySetRetention(days));

export const historyClearAll = async (): Promise<HistoryPrivacyStatus> =>
  requireStatus(await commands.historyClearAll());

export const externalApiStatus = (): Promise<ExternalApiStatus> =>
  commands.externalApiStatus();

export const externalApiEnable = (): Promise<string> =>
  commands.externalApiEnable(null);

export const externalApiDisable = (): Promise<void> =>
  commands.externalApiDisable().then(() => undefined);

export const externalApiRegenerateToken = (): Promise<string> =>
  commands.externalApiRegenerateToken();
