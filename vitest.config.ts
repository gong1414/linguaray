import { defineConfig } from "vitest/config";
import react from "@vitejs/plugin-react";

// Single React test project (migration complete — the Solid project is gone).
export default defineConfig({
  plugins: [react()],
  test: {
    environment: "jsdom",
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
