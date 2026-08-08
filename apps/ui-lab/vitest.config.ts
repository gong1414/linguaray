import { defineConfig } from "vitest/config";
import solid from "vite-plugin-solid";
import { fileURLToPath, URL } from "node:url";

export default defineConfig({
  plugins: [solid()],
  test: {
    environment: "jsdom",
    globals: true,
    setupFiles: ["./test/setup.ts"],
    // Exclude Playwright e2e tests (they run via `playwright test`, not vitest).
    exclude: ["**/node_modules/**", "**/dist/**", "**/e2e/**"],
  },
  resolve: {
    conditions: ["browser", "solid"],
    // Mirror vite.config.ts: let lab tests import the production state model
    // (`@app/features/translation/types`) for parity assertions.
    alias: {
      "@app": fileURLToPath(new URL("../../src", import.meta.url)),
    },
  },
});
