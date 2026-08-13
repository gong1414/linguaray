export const SHORTCUT_ACTIONS = [
  "translate_selection",
  "translate_input",
  "translate_clipboard",
  "ocr_translate",
] as const;

export type ShortcutAction = (typeof SHORTCUT_ACTIONS)[number];

/** Frozen, platform-neutral defaults. Canonical modifier order is
 * Ctrl+Alt+Shift+Super+Key. */
export const DEFAULT_SHORTCUT_MAP: Readonly<Record<ShortcutAction, string>> = {
  translate_selection: "Alt+Space",
  translate_input: "Ctrl+Space",
  translate_clipboard: "Ctrl+Alt+Space",
  ocr_translate: "Alt+Shift+Space",
};

export type ShortcutMap = Record<ShortcutAction, string>;
export type ShortcutRegistrationState =
  | "registered"
  | "registration_failed"
  | "unavailable";

export type ShortcutEntry = {
  action: ShortcutAction;
  combo: string;
  available: boolean;
  registration_state: ShortcutRegistrationState;
  registration_error: string | null;
};

export type ShortcutSnapshot = {
  revision: number;
  entries: ShortcutEntry[];
};

export type ShortcutConflict = ShortcutAction | null;

export type ShortcutCommandError = {
  error?: "registration_failed" | "stale_revision" | "invalid_combo" | string;
  action?: ShortcutAction;
  message?: string;
  expected?: number;
  actual?: number;
};

export const mapFromSnapshot = (snapshot: ShortcutSnapshot): ShortcutMap => {
  const map = { ...DEFAULT_SHORTCUT_MAP } as ShortcutMap;
  for (const entry of snapshot.entries) map[entry.action] = entry.combo;
  return map;
};

export const isShortcutAction = (value: unknown): value is ShortcutAction =>
  typeof value === "string" &&
  (SHORTCUT_ACTIONS as readonly string[]).includes(value);

export const isRegistrationFailure = (value: unknown): value is ShortcutCommandError => {
  if (!value || typeof value !== "object") return false;
  return (value as ShortcutCommandError).error === "registration_failed";
};

export const isStaleShortcutRevision = (value: unknown): value is ShortcutCommandError => {
  if (!value || typeof value !== "object") return false;
  return (value as ShortcutCommandError).error === "stale_revision";
};
