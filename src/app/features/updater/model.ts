/**
 * Updater wire types + the pure UI phase machine (ported 1:1 from the Solid
 * tree; every transition stays unit-testable without network/Tauri).
 */

export type UpdateCheck =
  | { state: "up_to_date"; version: string }
  | { state: "available"; current: string; next: string; notes: string }
  | { state: "error"; message: string };

export type AvailableUpdate = Extract<UpdateCheck, { state: "available" }>;

export function isUpdateCheck(value: unknown): value is UpdateCheck {
  if (typeof value !== "object" || value === null) return false;
  const v = value as Record<string, unknown>;
  switch (v.state) {
    case "up_to_date":
      return typeof v.version === "string";
    case "available":
      return typeof v.current === "string" && typeof v.next === "string" && typeof v.notes === "string";
    case "error":
      return typeof v.message === "string";
    default:
      return false;
  }
}

/** Payload of the backend `updater-progress` event. */
export type UpdaterProgress =
  | { downloaded: number; total: number | null; bucket: number }
  | { finished: true };

export function isUpdaterProgress(value: unknown): value is UpdaterProgress {
  if (typeof value !== "object" || value === null) return false;
  const v = value as Record<string, unknown>;
  if (v.finished === true) return true;
  return (
    typeof v.downloaded === "number" &&
    (v.total === null || typeof v.total === "number") &&
    typeof v.bucket === "number"
  );
}

export type UpdaterPhase =
  | { kind: "checking" }
  | { kind: "upToDate"; version: string }
  | { kind: "available"; update: AvailableUpdate }
  | { kind: "downloading"; update: AvailableUpdate; percent: number | null; downloaded: number }
  | { kind: "installing"; update: AvailableUpdate }
  /** macOS/Linux only: install finished, the app relaunches on demand. */
  | { kind: "readyToRelaunch"; update: AvailableUpdate }
  | { kind: "error"; message: string };

export function applyCheck(phase: UpdaterPhase, check: UpdateCheck): UpdaterPhase {
  // An install pipeline is in flight — a stray late check response must not
  // knock the panel out of the downloading/installing states.
  if (phase.kind === "downloading" || phase.kind === "installing") return phase;
  switch (check.state) {
    case "up_to_date":
      return { kind: "upToDate", version: check.version };
    case "available":
      return { kind: "available", update: check };
    case "error":
      return { kind: "error", message: check.message };
  }
}

export function applyProgress(phase: UpdaterPhase, progress: UpdaterProgress): UpdaterPhase {
  if (phase.kind !== "downloading") return phase;
  if ("finished" in progress) return { kind: "installing", update: phase.update };
  const percent =
    progress.total !== null && progress.total > 0
      ? Math.min(100, Math.round((progress.downloaded / progress.total) * 100))
      : null;
  return { kind: "downloading", update: phase.update, percent, downloaded: progress.downloaded };
}

/** Terminal result of `updater_download_install` (never resolves on Windows —
 *  the installer exits the process mid-call; only the events arrive). */
export function applyInstallDone(phase: UpdaterPhase, check: UpdateCheck): UpdaterPhase {
  if (phase.kind !== "installing") return phase;
  if (check.state === "available") return { kind: "readyToRelaunch", update: check };
  if (check.state === "up_to_date") return { kind: "upToDate", version: check.version };
  return { kind: "error", message: check.message };
}

export function applyFailure(_phase: UpdaterPhase, message: string): UpdaterPhase {
  return { kind: "error", message };
}
