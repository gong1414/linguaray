import { defineConfig } from "vitest/config";
import solid from "vite-plugin-solid";
import react from "@vitejs/plugin-react";
import { fileURLToPath, URL } from "node:url";

// Migration (2026-08-16): two isolated test projects so the Solid and React
// toolchains never share a transform pipeline.
//  - "solid" — legacy tree (src/** except src/app, test/**): vite-plugin-solid
//    with the browser/solid resolve conditions and @linguaray/ui aliases.
//  - "react" — the React 19 + Mantine tree (src/app/**): @vitejs/plugin-react.
// Both inherit the shared jsdom environment + setup from the root config.
export default defineConfig({
  test: {
    environment: "jsdom",
    environmentOptions: {
      jsdom: {
        url: "http://localhost",
      },
    },
    globals: true,
    setupFiles: ["test/setup.ts"],
    projects: [
      {
        extends: true,
        test: {
          name: "solid",
          include: ["test/**/*.test.{ts,tsx}", "src/**/*.test.{ts,tsx}"],
          exclude: ["src/app/**", "node_modules/**"],
        },
        plugins: [solid()],
        resolve: {
          alias: {
            // Entry files (onboarding-entry etc.) import the design-system
            // stylesheet; map the subpath for vitest (CSS is inert under jsdom).
            // MUST precede the bare "@linguaray/ui" alias — vite matches prefixes.
            "@linguaray/ui/styles": fileURLToPath(
              new URL("./packages/ui/src/styles/index.css", import.meta.url),
            ),
            "@linguaray/ui": fileURLToPath(
              new URL("./packages/ui/src/index.ts", import.meta.url),
            ),
          },
          // Force the Solid client build (not the server/SSR entry) so component
          // modules that import lucide-solid / Solid web APIs evaluate in jsdom.
          conditions: ["browser", "solid"],
        },
      },
      {
        extends: true,
        test: {
          name: "react",
          include: ["src/app/**/*.test.{ts,tsx}"],
        },
        plugins: [react()],
      },
    ],
  },
});
