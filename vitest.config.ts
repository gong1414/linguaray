import { defineConfig } from "vitest/config";
import react from "@vitejs/plugin-react";

// Single React test project (migration complete — the Solid project is gone).
export default defineConfig({
  plugins: [react()],
  test: {
    environment: "jsdom",
    // Fluent UI's interaction layer (Tabster) publishes a mixed ESM/CJS
    // dependency graph. Pre-bundle the browser-facing packages so Vitest's
    // jsdom runner resolves the same graph as Vite/Tauri.
    deps: {
      optimizer: {
        client: {
          enabled: true,
          include: [
            "@fluentui/react-components",
            "@fluentui/react-icons",
          ],
        },
      },
    },
    environmentOptions: {
      jsdom: {
        url: "http://localhost",
      },
    },
    globals: true,
    setupFiles: ["test/setup.ts", "test/react-setup.ts"],
    include: ["src/**/*.test.{ts,tsx}", "test/**/*.test.{ts,tsx}"],
  },
});
