import { detectLocale } from "./i18n";

const LIGHT_THEME_COLOR = "#F8FAFC";
const DARK_THEME_COLOR = "#020617";

/**
 * Read once at first paint: theme, motion preference, and locale. Sets three
 * attributes on documentElement so @linguaray/ui token CSS ([data-theme=...]
 * blocks in tokens.css) and base.css ([data-motion=reduced]) resolve BEFORE the
 * first component renders, avoiding a flash of unstyled/wrong-theme content.
 *
 * P2: keeps BOTH theme-color metas in the DOM but DISABLES the non-current one
 * (media="disabled") so the browser chrome honors only the resolved scheme. The
 * current meta keeps its prefers-color-scheme media and gets its content
 * re-asserted to the resolved token.
 *
 * Safe to call in any entry (popup/input/settings) and in jsdom tests.
 */
export function initTheme(): void {
  const root = document.documentElement;

  let theme: "light" | "dark";
  const stored =
    typeof localStorage !== "undefined" ? localStorage.getItem("linguaray.theme") : null;
  if (stored === "light" || stored === "dark") {
    theme = stored;
  } else if (
    typeof window !== "undefined" &&
    typeof window.matchMedia === "function" &&
    window.matchMedia("(prefers-color-scheme: dark)").matches
  ) {
    theme = "dark";
  } else {
    theme = "light";
  }
  root.dataset.theme = theme;

  const reduced =
    typeof window !== "undefined" &&
    typeof window.matchMedia === "function" &&
    window.matchMedia("(prefers-reduced-motion: reduce)").matches;
  root.dataset.motion = reduced ? "reduced" : "full";

  root.lang = detectLocale();

  if (typeof document !== "undefined") {
    syncThemeColorMetas(theme);
  }
}

/**
 * rev-5-6: activate the resolved-scheme meta, disable the other.
 *
 * The CURRENT meta gets `media="all"` (always wins) — NOT
 * `(prefers-color-scheme: <current>)`. The rev-4 form kept the prefers media,
 * which breaks when the user FORCES a theme that disagrees with the OS: OS
 * Light + forced Dark → the Dark meta keeps `media="(prefers-color-scheme: dark)"`
 * (does NOT match the OS Light preference) and the Light meta gets
 * `media="disabled"`, so NO meta wins and the OS chrome falls back to the
 * browser default. With `media="all"` the current meta always applies, and the
 * other meta is `media="disabled"` so it never overrides.
 */
function syncThemeColorMetas(theme: "light" | "dark"): void {
  const metas = document.querySelectorAll<HTMLMetaElement>('meta[name="theme-color"]');
  const currentColor = theme === "dark" ? DARK_THEME_COLOR : LIGHT_THEME_COLOR;
  if (metas.length === 0) {
    const m = document.createElement("meta");
    m.setAttribute("name", "theme-color");
    m.setAttribute("media", "all");
    m.setAttribute("content", currentColor);
    m.setAttribute("data-theme-scheme", theme);
    document.head.appendChild(m);
    return;
  }
  for (const m of Array.from(metas)) {
    // Match by data-theme-scheme attribute (set in index.html), not by parsing
    // the media string — robust against media format changes / HMR reordering.
    const scheme = m.getAttribute("data-theme-scheme");
    const isCurrent = scheme === theme;
    if (isCurrent) {
      m.setAttribute("media", "all");
      m.setAttribute("content", currentColor);
    } else {
      m.setAttribute("media", "disabled");
    }
  }
}
