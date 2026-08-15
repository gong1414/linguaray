import { invoke } from "../../bridge/invoke";
import type { HistoryFilter, HistoryPage } from "./history-types";

export const historySearch = (
  query: string,
  cursor?: string | null,
): Promise<HistoryPage> =>
  invoke<HistoryPage>("history_search", { query, cursor: cursor ?? null });

export const historyToggleFavorite = (sessionUuid: string): Promise<boolean> =>
  invoke<boolean>("history_toggle_favorite", { sessionUuid });

export const historyDeleteSession = (sessionUuid: string): Promise<void> =>
  invoke<void>("history_delete_session", { sessionUuid });

export const historyExport = (
  filePath: string,
  format: "csv" | "json",
  filter: HistoryFilter,
): Promise<string> =>
  invoke<string>("history_export", { filePath, format, filter });

export async function chooseExportPath(defaultName: string): Promise<string | null> {
  try {
    const mod = await import("../../bridge/dialog");
    const picked = await mod.save({ defaultPath: defaultName });
    return picked ?? null;
  } catch {
    return defaultName;
  }
}
