import { MantineProvider, localStorageColorSchemeManager } from "@mantine/core";
import type { ReactNode } from "react";
import { linguaTheme } from "../ui/theme";

/**
 * Root providers for every React window. Color scheme persists under the
 * SAME localStorage key the legacy Solid windows use ("linguaray.theme"), so
 * a user's theme choice survives the window-by-window migration and mixed
 * old/new windows stay consistent. "auto" follows prefers-color-scheme,
 * matching the legacy initTheme fallback.
 */
export function AppProviders({ children }: { children: ReactNode }) {
  return (
    <MantineProvider
      theme={linguaTheme}
      colorSchemeManager={localStorageColorSchemeManager({ key: "linguaray.theme" })}
      defaultColorScheme="auto"
    >
      {children}
    </MantineProvider>
  );
}
