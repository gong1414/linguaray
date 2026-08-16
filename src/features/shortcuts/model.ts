/** Shortcut domain model — wire types, canonical combo format, helpers. */

export const SHORTCUT_ACTIONS = [
  "translate_selection",
  "translate_input",
  "translate_clipboard",
  "ocr_translate",
] as const;

export type ShortcutAction = (typeof SHORTCUT_ACTIONS)[number];

/** Frozen, platform-neutral defaults (canonical order Ctrl+Alt+Shift+Super+Key). */
export const DEFAULT_SHORTCUT_MAP: Readonly<Record<ShortcutAction, string>> = {
  translate_selection: "Alt+Space",
  translate_input: "Ctrl+Space",
  translate_clipboard: "Ctrl+Alt+Space",
  ocr_translate: "Alt+Shift+Space",
};

export type ShortcutMap = Record<ShortcutAction, string>;
export type ShortcutRegistrationState = "registered" | "registration_failed" | "unavailable";

export type ShortcutEntry = {
  action: ShortcutAction;
  combo: string;
  available: boolean;
  registration_state: ShortcutRegistrationState;
  registration_error: string | null;
};

export type ShortcutSnapshot = { revision: number; entries: ShortcutEntry[] };
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

export const isRegistrationFailure = (value: unknown): value is ShortcutCommandError =>
  !!value && typeof value === "object" && (value as ShortcutCommandError).error === "registration_failed";

export const isStaleShortcutRevision = (value: unknown): value is ShortcutCommandError =>
  !!value && typeof value === "object" && (value as ShortcutCommandError).error === "stale_revision";

export type ShortcutConflictState = {
  action: ShortcutAction;
  otherAction: ShortcutAction;
  combo: string;
};

const MODIFIER_ONLY = new Set(["Alt", "AltGraph", "Control", "Meta", "OS", "Shift"]);

/** Convert a browser keydown into the frozen Ctrl+Alt+Shift+Super+Key format. */
export function canonicalCombo(event: {
  key: string;
  code: string;
  ctrlKey: boolean;
  altKey: boolean;
  shiftKey: boolean;
  metaKey: boolean;
  repeat?: boolean;
  isComposing?: boolean;
}): string | null {
  if (event.repeat || event.isComposing || MODIFIER_ONLY.has(event.key)) return null;
  if (!event.ctrlKey && !event.altKey && !event.shiftKey && !event.metaKey) return null;

  let key: string;
  if (event.code.startsWith("Key")) key = event.code.slice(3);
  else if (event.code.startsWith("Digit")) key = event.code.slice(5);
  else if (event.code === "Space" || event.key === " ") key = "Space";
  else if (/^F(?:[1-9]|1[0-9]|2[0-4])$/.test(event.key)) key = event.key;
  else if (event.key.length === 1) key = event.key.toUpperCase();
  else key = event.key;

  const parts: string[] = [];
  if (event.ctrlKey) parts.push("Ctrl");
  if (event.altKey) parts.push("Alt");
  if (event.shiftKey) parts.push("Shift");
  if (event.metaKey) parts.push("Super");
  parts.push(key);
  return parts.join("+");
}
