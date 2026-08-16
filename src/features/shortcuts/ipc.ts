/** Typed wrappers for the shortcut Rust commands. */
import { commands } from "../../bridge/invoke";
import type { ShortcutAction, ShortcutConflict, ShortcutSnapshot } from "./model";

export const shortcutList = (): Promise<ShortcutSnapshot> =>
  commands.shortcutList();

export const shortcutCheckConflict = (
  action: ShortcutAction,
  combo: string,
  revision: number,
): Promise<ShortcutConflict> =>
  commands.shortcutCheckConflict(action, combo, revision);

export const shortcutSave = (
  action: ShortcutAction,
  combo: string,
  expectedRevision: number,
  overrideAction?: ShortcutAction,
): Promise<ShortcutSnapshot> =>
  commands.shortcutSave(action, combo, expectedRevision, overrideAction ?? null);

export const shortcutResetDefaults = (expectedRevision: number): Promise<ShortcutSnapshot> =>
  commands.shortcutResetDefaults(expectedRevision);

/** Suspend existing native callbacks while the focused settings page records. */
export const shortcutRecordingBegin = (action: ShortcutAction): Promise<void> =>
  commands.shortcutRecordingBegin(action).then(() => undefined);

/** Always paired with `shortcutRecordingBegin`, including cancel and unmount. */
export const shortcutRecordingEnd = (): Promise<void> =>
  commands.shortcutRecordingEnd();
