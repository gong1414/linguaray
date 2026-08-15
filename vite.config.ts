import { defineConfig } from "vite";
import solid from "vite-plugin-solid";
import react from "@vitejs/plugin-react";

// @ts-expect-error process is a nodejs global
const host = process.env.TAURI_DEV_HOST;

// Migration (2026-08-16): the React 19 + Mantine 9 tree lives under src/app/**
// and replaces the Solid tree window-by-window. Both plugin chains stay side
// by side until Phase 5 deletes the Solid tree; each is scoped to its own
// file set so JSX of one framework is never compiled by the other.
const REACT_INCLUDE = [/src\/app\//];

export default defineConfig(async () => ({
  plugins: [
    solid({ include: [/\.html$/, /src\/(?!app\/)/] }),
    react({ include: REACT_INCLUDE }),
  ],

  // Vite options tailored for Tauri development and only applied in `tauri dev` or `tauri build`
  //
  // 1. prevent Vite from obscuring rust errors
  clearScreen: false,
  // 2. tauri expects a fixed port, fail if that port is not available
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host
      ? {
          protocol: "ws",
          host,
          port: 1421,
        }
      : undefined,
    watch: {
      // 3. tell Vite to ignore watching `src-tauri`
      ignored: ["**/src-tauri/**"],
    },
  },
  // Multi-page build: main app + frameless popup window. Each page's entry
  // script points into either the React tree (src/app/entries) or the legacy
  // Solid tree until its window is migrated.
  build: {
    rollupOptions: {
      input: {
        main: "index.html",
        popup: "popup.html",
        input: "input.html",
        ocr: "ocr.html",
        onboarding: "onboarding.html",
      },
    },
  },
}));
