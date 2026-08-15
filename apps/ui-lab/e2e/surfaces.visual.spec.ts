import { test, expect } from "@playwright/test";

/**
 * Surface visual baselines (P1-10): every real browser-rendered surface at
 * 600 / 699 / 700 / 800 px widths × light / dark themes.
 *
 * The tray / menu-bar (Surface 06b) is native OS UI and CANNOT be
 * browser-screenshotted — it is documented as "manual screenshot acceptance
 * required" in the D5 commit message body (per the project no-new-docs rule).
 *
 * Surfaces covered:
 *  - Provider Center (empty + populated fixtures)
 *  - Keystore Recovery (healthy + corrupt fixtures)
 *  - Privacy & Data history controls (enabled + disabled fixtures)
 *  - Keyboard Shortcuts (default / recording / conflict / registration failure)
 *  - Selection Popup (loading / single / multi / partial / error-network)
 *  - Input Window (idle / multi / partial / error)
 *
 * Fixtures reuse the PRODUCTION presentational Views (InputPanelView +
 * KeystoreRecoveryView) via the `@app` alias, so a production UI regression
 * surfaces here as a baseline mismatch.
 *
 * Isolation: every fixture deep-links via `?nav=`/`?fixture=`/`?state=`/
 * `?theme=`, so only the target surface renders. The lab shell's header/nav
 * are part of every shot (intentional — they show the theme tokens in context),
 * captured with fullPage so the state bar + frame are both visible.
 */

const WIDTHS = [600, 699, 700, 800] as const;
const THEMES = ["light", "dark"] as const;
const BASE = "http://localhost:1421";

const SETTINGS_SURFACES = [
  { nav: "provider-center", fixture: "populated", label: "provider-center-populated" },
  { nav: "provider-center", fixture: "empty", label: "provider-center-empty" },
  { nav: "keystore", fixture: "healthy", label: "keystore-recovery-healthy" },
  { nav: "keystore", fixture: "corrupt", label: "keystore-recovery-corrupt" },
  { nav: "privacy", fixture: "enabled", label: "privacy-history-enabled" },
  { nav: "privacy", fixture: "disabled", label: "privacy-history-disabled" },
  { nav: "shortcuts", fixture: "default", label: "shortcuts-default" },
  { nav: "shortcuts", fixture: "recording", label: "shortcuts-recording" },
  { nav: "shortcuts", fixture: "conflict", label: "shortcuts-conflict" },
  { nav: "shortcuts", fixture: "failure", label: "shortcuts-registration-failed" },
  { nav: "updater", fixture: "available", label: "updater-available" },
  { nav: "updater", fixture: "downloading", label: "updater-downloading" },
  { nav: "updater", fixture: "error", label: "updater-error" },
] as const;

// R6: the redesigned onboarding (600×400 window) in both locales, plus the
// rewritten OCR overlay toolbar. Sampled at the window's native width and one
// wide breakpoint instead of the full 4-width matrix.
const ONBOARDING_SURFACES = [
  { fixture: "welcome", locale: "en", label: "onboarding-welcome-en" },
  { fixture: "welcome", locale: "zh", label: "onboarding-welcome-zh" },
  { fixture: "a11y-denied", locale: "en", label: "onboarding-a11y-denied-en" },
  { fixture: "a11y-denied", locale: "zh", label: "onboarding-a11y-denied-zh" },
] as const;

const ONBOARDING_WIDTHS = [600, 800] as const;

const POPUP_SURFACES = [
  { state: "loading", label: "popup-loading" },
  { state: "success-single", label: "popup-single" },
  { state: "success-multi", label: "popup-multi" },
  { state: "partial", label: "popup-partial" },
  { state: "error-network", label: "popup-error" },
] as const;

const INPUT_SURFACES = [
  { state: "idle", label: "input-idle" },
  { state: "multi", label: "input-multi" },
  { state: "partial", label: "input-partial" },
  { state: "error", label: "input-error" },
] as const;

for (const width of WIDTHS) {
  for (const theme of THEMES) {
    for (const s of SETTINGS_SURFACES) {
      test(`visual: ${s.label} @ ${width}px ${theme}`, async ({ page }) => {
        await page.setViewportSize({ width, height: 800 });
        await page.goto(`${BASE}/?nav=${s.nav}&fixture=${s.fixture}&theme=${theme}`);
        await page.evaluate(() => document.fonts.ready);
        // Apply theme on <html> in case the query param read races the effect.
        await page.evaluate((tt) => {
          document.documentElement.setAttribute("data-theme", tt);
        }, theme);
        await page.waitForSelector(
          "[data-testid='lab-root'], .pc__body, .keystore-recovery, .privacy-data, .shortcuts, .updater-panel",
          { timeout: 10_000 },
        );
        await expect(page).toHaveScreenshot(`${s.label}-${width}-${theme}.png`, {
          maxDiffPixelRatio: 0.01,
          fullPage: true,
        });
      });
    }
    for (const s of POPUP_SURFACES) {
      test(`visual: ${s.label} @ ${width}px ${theme}`, async ({ page }) => {
        await page.setViewportSize({ width, height: 800 });
        await page.goto(`${BASE}/?nav=selection-popup&state=${s.state}&theme=${theme}`);
        await page.evaluate(() => document.fonts.ready);
        await page.evaluate((tt) => {
          document.documentElement.setAttribute("data-theme", tt);
        }, theme);
        await page.waitForSelector(".sel-popup__body, .lab__hidden-note", { timeout: 10_000 });
        await expect(page).toHaveScreenshot(`${s.label}-${width}-${theme}.png`, {
          maxDiffPixelRatio: 0.01,
          fullPage: true,
        });
      });
    }
    for (const s of INPUT_SURFACES) {
      test(`visual: ${s.label} @ ${width}px ${theme}`, async ({ page }) => {
        await page.setViewportSize({ width, height: 800 });
        await page.goto(`${BASE}/?nav=input-window&state=${s.state}&theme=${theme}`);
        await page.evaluate(() => document.fonts.ready);
        await page.evaluate((tt) => {
          document.documentElement.setAttribute("data-theme", tt);
        }, theme);
        await page.waitForSelector(".input-shell", { timeout: 10_000 });
        await expect(page).toHaveScreenshot(`${s.label}-${width}-${theme}.png`, {
          maxDiffPixelRatio: 0.01,
          fullPage: true,
        });
      });
    }
  }
}

// R6 onboarding baselines: sampled widths × themes × locales, outside the
// main WIDTHS×THEMES loop (own width set + a locale query param).
for (const width of ONBOARDING_WIDTHS) {
  for (const theme of THEMES) {
    for (const s of ONBOARDING_SURFACES) {
      test(`visual: ${s.label} @ ${width}px ${theme}`, async ({ page }) => {
        await page.setViewportSize({ width, height: 800 });
        await page.goto(
          `${BASE}/?nav=onboarding&fixture=${s.fixture}&locale=${s.locale}&theme=${theme}`,
        );
        await page.evaluate(() => document.fonts.ready);
        await page.evaluate((tt) => {
          document.documentElement.setAttribute("data-theme", tt);
        }, theme);
        await page.waitForSelector('[data-testid="onboarding"]', { timeout: 10_000 });
        await expect(page).toHaveScreenshot(`${s.label}-${width}-${theme}.png`, {
          maxDiffPixelRatio: 0.01,
          fullPage: true,
        });
      });
    }
    test(`visual: ocr-overlay @ ${width}px ${theme}`, async ({ page }) => {
      await page.setViewportSize({ width, height: 800 });
      await page.goto(`${BASE}/?nav=ocr-overlay&theme=${theme}`);
      await page.evaluate(() => document.fonts.ready);
      await page.evaluate((tt) => {
        document.documentElement.setAttribute("data-theme", tt);
      }, theme);
      await page.waitForSelector(".ocr-overlay", { timeout: 10_000 });
      await expect(page).toHaveScreenshot(`ocr-overlay-${width}-${theme}.png`, {
        maxDiffPixelRatio: 0.01,
        fullPage: true,
      });
    });
  }
}

test("no horizontal overflow at 699px (provider center populated)", async ({ page }) => {
  await page.setViewportSize({ width: 699, height: 800 });
  await page.goto(`${BASE}/?nav=provider-center&fixture=populated&theme=light`);
  await page.waitForSelector(".pc__body", { timeout: 10_000 });
  const scrollWidth = await page.evaluate(() => document.documentElement.scrollWidth);
  const clientWidth = await page.evaluate(() => document.documentElement.clientWidth);
  expect(scrollWidth, "horizontal overflow at 699px").toBeLessThanOrEqual(clientWidth);
});
