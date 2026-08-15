import { defineConfig } from "vitest/config";
import solid from "vite-plugin-solid";
import { fileURLToPath, URL } from "node:url";

export default defineConfig({
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
  test: {
    environment: "jsdom",
    environmentOptions: {
      jsdom: {
        url: "http://localhost",
      },
    },
    globals: true,
    setupFiles: ["test/setup.ts"],
    include: ["src/**/*.test.{ts,tsx}", "test/**/*.test.{ts,tsx}"],
  },
});
