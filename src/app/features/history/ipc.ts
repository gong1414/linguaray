/** Typed wrappers for the history Rust commands + native export dialog. */
import { invoke } from "../../../bridge/invoke";
import { save } from "../../../bridge/dialog";
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
): Promise<HistoryPage> => requirePage(await invoke<unknown>("history_search", { query, cursor: cursor ?? null }));

export const historyToggleFavorite = (sessionUuid: string): Promise<boolean> =>
  invoke<boolean>("history_toggle_favorite", { sessionUuid });

export const historyDeleteSession = (sessionUuid: string): Promise<void> =>
  invoke<void>("history_delete_session", { sessionUuid });

export const historyExport = (
  filePath: string,
  format: "csv" | "json",
  filter: HistoryFilter,
): Promise<string> => invoke<string>("history_export", { filePath, format, filter });

export const historyPrivacyEnabled = async (): Promise<boolean> => {
  const status = await invoke<unknown>("history_privacy_status");
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
