/**
 * Normalize a Tauri WebView document before React mounts.
 *
 * Keep this out of inline <style> tags: on WKWebView, a pre-existing document
 * style sheet can interfere with component-library runtime styles. Direct
 * element styles establish the window canvas without participating in the
 * style-sheet cascade.
 */
export function prepareWindowDocument(options: { transparent?: boolean } = {}) {
  const root = document.getElementById("root");
  for (const element of [document.documentElement, document.body, root]) {
    if (!element) continue;
    element.style.width = "100%";
    element.style.height = "100%";
    element.style.margin = "0";
    element.style.overflow = "hidden";
    if (options.transparent) element.style.backgroundColor = "transparent";
  }
}
