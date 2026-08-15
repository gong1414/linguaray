/** Typed wrappers for the shortcut Rust commands. */
import { invoke } from "../../../bridge/invoke";
import type { ShortcutAction, ShortcutConflict, ShortcutSnapshot } from "./model";

export const shortcutList = (): Promise<ShortcutSnapshot> =>
  invoke<ShortcutSnapshot>("shortcut_list");

export const shortcutCheckConflict = (
  action: ShortcutAction,
  combo: string,
  revision: number,
): Promise<ShortcutConflict> =>
  invoke<ShortcutConflict>("shortcut_check_conflict", { action, combo, revision });

export const shortcutSave = (
  action: ShortcutAction,
  combo: string,
  expectedRevision: number,
  overrideAction?: ShortcutAction,
): Promise<ShortcutSnapshot> =>
  invoke<ShortcutSnapshot>("shortcut_save", {
    action,
    combo,
    expectedRevision,
    overrideAction: overrideAction ?? null,
  });

export const shortcutResetDefaults = (expectedRevision: number): Promise<ShortcutSnapshot> =>
  invoke<ShortcutSnapshot>("shortcut_reset_defaults", { expectedRevision });

/** Suspend existing native callbacks while the focused settings page records. */
export const shortcutRecordingBegin = (action: ShortcutAction): Promise<void> =>
  invoke<void>("shortcut_recording_begin", { action });

/** Always paired with `shortcutRecordingBegin`, including cancel and unmount. */
export const shortcutRecordingEnd = (): Promise<void> =>
  invoke<void>("shortcut_recording_end");
