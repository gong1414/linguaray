/** Privacy & Data domain model (mirrors the Rust command contract). */

export type HistoryRetentionDays = 30 | 90;

export type HistoryPrivacyStatus = {
  enabled: boolean;
  retention_days: HistoryRetentionDays;
  record_count: number;
};

export type ExternalApiStatus = { state: string; port?: number };

export const isHistoryPrivacyStatus = (value: unknown): value is HistoryPrivacyStatus => {
  if (!value || typeof value !== "object") return false;
  const status = value as Record<string, unknown>;
  return (
    typeof status.enabled === "boolean" &&
    (status.retention_days === 30 || status.retention_days === 90) &&
    Number.isSafeInteger(status.record_count) &&
    (status.record_count as number) >= 0
  );
};

/** Which mutation is in flight (drives per-control loading/disabled). */
export type PrivacyBusy = "enabled" | "retention" | "clear" | null;
