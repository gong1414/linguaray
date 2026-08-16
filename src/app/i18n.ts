/**
 * React-tree locale (shared resolution order with the legacy tree until it
 * is deleted: localStorage("linguaray.locale") → navigator.language → en).
 */
import { COPY } from "../features/translation/copy";
import type { CopyKey } from "../features/translation/types";

export type Locale = "zh" | "en";

export function detectLocale(): Locale {
  const stored =
    typeof localStorage !== "undefined" ? localStorage.getItem("linguaray.locale") : null;
  if (stored === "zh" || stored === "en") return stored;
  if (typeof navigator !== "undefined" && navigator.language?.toLowerCase().startsWith("zh")) {
    return "zh";
  }
  return "en";
}

/** Translation-surface copy lookup (popup + input window). */
export function t(key: CopyKey): string {
  return COPY[detectLocale()][key];
}
