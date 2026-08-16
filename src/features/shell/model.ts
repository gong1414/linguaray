export type SettingsSection =
  | "provider-center"
  | "keystore-recovery"
  | "shortcuts"
  | "privacy"
  | "history"
  | "vocabulary"
  | "dictionary"
  | "updater";

export const SETTINGS_SECTIONS: SettingsSection[] = [
  "provider-center",
  "keystore-recovery",
  "shortcuts",
  "privacy",
  "history",
  "vocabulary",
  "dictionary",
  "updater",
];

/** null = still checking / unknown. */
export type A11yState = boolean | null;
