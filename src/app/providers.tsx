import { FluentProvider, webDarkTheme, webLightTheme } from "@fluentui/react-components";
import { useEffect, useState, type ReactNode } from "react";

type ColorScheme = "light" | "dark";

function readColorScheme(): ColorScheme {
  const saved = localStorage.getItem("linguaray.theme");
  if (saved === "light" || saved === "dark") return saved;
  return window.matchMedia?.("(prefers-color-scheme: dark)").matches ? "dark" : "light";
}

/**
 * Root providers for every React window. Color scheme persists under the
 * SAME localStorage key the legacy Solid windows use ("linguaray.theme"), so
 * a user's theme choice survives the window-by-window migration and mixed
 * old/new windows stay consistent. "auto" follows prefers-color-scheme,
 * matching the legacy initTheme fallback.
 */
export function AppProviders({ children, transparent = false }: { children: ReactNode; transparent?: boolean }) {
  const [colorScheme, setColorScheme] = useState<ColorScheme>(readColorScheme);

  useEffect(() => {
    const media = window.matchMedia?.("(prefers-color-scheme: dark)");
    if (!media) return;
    const onChange = (event: MediaQueryListEvent) => {
      const saved = localStorage.getItem("linguaray.theme");
      if (saved !== "light" && saved !== "dark") setColorScheme(event.matches ? "dark" : "light");
    };
    media.addEventListener("change", onChange);
    return () => media.removeEventListener("change", onChange);
  }, []);

  return (
    <FluentProvider
      theme={colorScheme === "dark" ? webDarkTheme : webLightTheme}
      data-color-scheme={colorScheme}
      style={{ minHeight: "100%", backgroundColor: transparent ? "transparent" : undefined }}
    >
      {children}
    </FluentProvider>
  );
}
