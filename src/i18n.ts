import type { CopyKey } from "./features/translation/types";
import { COPY } from "./features/translation/copy";

export type Locale = "zh" | "en";

/**
 * Detect the user locale. Order: localStorage("linguaray.locale") →
 * navigator.language prefix → "en". Kept dependency-free so it is testable
 * and SSR-safe (Tauri WebView provides navigator).
 */
export function detectLocale(): Locale {
  const stored =
    typeof localStorage !== "undefined" ? localStorage.getItem("linguaray.locale") : null;
  if (stored === "zh" || stored === "en") return stored;
  if (typeof navigator !== "undefined" && navigator.language?.toLowerCase().startsWith("zh")) {
    return "zh";
  }
  return "en";
}

/** Typed accessor: `t("selection.loading", locale)`. */
export function t(key: CopyKey, locale: Locale = detectLocale()): string {
  return COPY[locale][key];
}

export { COPY };
