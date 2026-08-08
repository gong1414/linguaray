import { defineConfig } from "vite";
import solid from "vite-plugin-solid";
import { fileURLToPath, URL } from "node:url";

export default defineConfig({
  plugins: [solid()],
  server: {
    port: 1430,
    strictPort: false,
  },
  resolve: {
    alias: {
      // Lets the lab import the production state model (e.g. TranslationState)
      // without duplicating it, enforcing lab↔production parity at the type
      // level. `@app/features/translation/types` → `<repo>/src/...`.
      "@app": fileURLToPath(new URL("../../src", import.meta.url)),
    },
  },
});
