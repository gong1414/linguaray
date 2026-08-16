/** Typed wrappers for the history Rust commands + native export dialog. */
import { commands } from "../../bridge/invoke";
import { save } from "../../bridge/dialog";
import { isHistoryPrivacyStatus } from "../privacy/model";
import type { HistoryFilter, HistoryPage } from "./model";

const requirePage = (value: unknown): HistoryPage => {
  if (!value || typeof value !== "object") {
    throw new Error("history_search returned an invalid payload");
  }
  const page = value as Record<string, unknown>;
  if (!Array.isArray(page.items) || typeof page.scan_complete !== "boolean") {
    throw new Error("history_search returned an invalid payload");
  }
  return value as HistoryPage;
};

export const historySearch = async (
  query: string,
  cursor?: string | null,
): Promise<HistoryPage> => requirePage(await commands.historySearch(query, cursor ?? null));

export const historyToggleFavorite = (sessionUuid: string): Promise<boolean> =>
  commands.historyToggleFavorite(sessionUuid);

export const historyDeleteSession = (sessionUuid: string): Promise<void> =>
  commands.historyDeleteSession(sessionUuid).then(() => undefined);

export const historyExport = (
  filePath: string,
  format: "csv" | "json",
  filter: HistoryFilter,
): Promise<string> =>
  commands.historyExport(filePath, format, {
    query: filter.query ?? null,
    favorites_only: filter.favorites_only,
  });

export const historyPrivacyEnabled = async (): Promise<boolean> => {
  const status = await commands.historyPrivacyStatus();
  if (!isHistoryPrivacyStatus(status)) {
    throw new Error("history_privacy_status returned an invalid payload");
  }
  return status.enabled;
};

/** Native save dialog; falls back to the default name if the host has none. */
export async function chooseExportPath(defaultName: string): Promise<string | null> {
  try {
    const picked = await save({ defaultPath: defaultName });
    return picked ?? null;
  } catch {
    return defaultName;
  }
}
