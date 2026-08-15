/**
 * React-tree locale (shared resolution order with the legacy tree until it
 * is deleted: localStorage("linguaray.locale") → navigator.language → en).
 */
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
