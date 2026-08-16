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
    // 0.03 absorbs cross-run font antialiasing jitter (long-CJK line-clamp
    // text wobbles ~2% of pixels between Playwright's two internal captures
    // on CI font sets) while still failing real layout regressions.
    toHaveScreenshot: {
      maxDiffPixelRatio: 0.03,
      threshold: 0.2,
      animations: "disabled",
      caret: "hide",
    },
  },
  webServer: {
    command: "npx http-server storybook-static -p 6008 --silent",
    url: "http://localhost:6008/index.json",
    reuseExistingServer: !process.env.CI,
    timeout: 60_000,
  },
});
