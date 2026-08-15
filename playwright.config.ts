import { defineConfig } from "@playwright/test";

/**
 * Visual baselines over the BUILT Storybook (production compositions per
 * migration spec §八): every story is screenshotted on chromium/macOS and the
 * baselines live in e2e/storybook.visual.spec.ts-snapshots/. Regenerate with
 * `npx playwright test --update-snapshots` (never auto-update in CI).
 */
export default defineConfig({
  testDir: "./e2e",
  timeout: 30_000,
  fullyParallel: true,
  workers: process.env.CI ? 2 : undefined,
  retries: 0,
  use: {
    viewport: { width: 800, height: 600 },
    colorScheme: "light",
  },
  expect: {
    toHaveScreenshot: { maxDiffPixelRatio: 0.01 },
  },
  webServer: {
    command: "npx http-server storybook-static -p 6008 --silent",
    url: "http://localhost:6008/index.json",
    reuseExistingServer: !process.env.CI,
    timeout: 60_000,
  },
});
