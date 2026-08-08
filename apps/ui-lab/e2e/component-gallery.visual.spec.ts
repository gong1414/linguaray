import { test, expect } from "@playwright/test";

/**
 * Component-gallery visual baselines.
 *
 * Isolation guarantees:
 *  - Every section renders ONLY its target component's DOM. Portal-based
 *    components (Confirm/Dialog) are closed at mount (open=false), so their
 *    overlays never bleed into another section's screenshot.
 *  - `confirm` is captured via the dedicated `confirm-isolated` route, which
 *    renders an OPEN dialog body (not just its trigger button), giving the
 *    dialog real visual-regression coverage.
 *  - The reduced-motion test sets data-motion="reduced" AND emulates the OS
 *    prefers-reduced-motion preference, then screenshots the Spinner under a
 *    dedicated name so it cannot collide with the full-motion Spinner baseline.
 */

const COMPONENT_IDS = [
  "button", "icon-button", "segmented-control", "shortcut-chip",
  "text-field", "select", "switch",
  "status-badge", "inline-error", "toast", "confirm", "empty-state",
  "translation-card", "result-card", "provider-row", "history-row",
  "sidebar-item", "spinner", "window-chrome", "overflow-cjk",
];

// Confirm portals to <body> and is closed inside the gallery section (its
// section only shows the trigger button). To give the dialog body real visual
// coverage it is captured separately via the `confirm-isolated` route, which
// renders an open dialog with no shell. The case name stays "confirm" so the
// total test count and baseline name (${theme}-confirm.png) are unchanged.
const ISOLATED_IDS = new Set(["confirm"]);

for (const theme of ["light", "dark"] as const) {
  for (const id of COMPONENT_IDS) {
    if (ISOLATED_IDS.has(id)) continue;
    test(`${theme}/${id} visual baseline`, async ({ page }) => {
      await page.goto(`http://localhost:1421/?nav=component-gallery&theme=${theme}`);
      // 等待本地字体加载完成
      await page.evaluate(() => document.fonts.ready);
      // 设置主题
      await page.evaluate((t) => {
        document.documentElement.setAttribute("data-theme", t);
      }, theme);
      const locator = page.locator(`[data-component-id="${id}"]`);
      // 等待组件可见后再截图
      await expect(locator).toBeVisible();
      await expect(locator).toHaveScreenshot(`${theme}-${id}.png`);
    });
  }
}

// Confirm: screenshot the OPEN dialog body via the isolated route instead of
// the gallery section (which only renders the trigger button).
for (const theme of ["light", "dark"] as const) {
  test(`${theme}/confirm visual baseline`, async ({ page }) => {
    await page.goto(`http://localhost:1421/?nav=confirm-isolated&theme=${theme}`);
    await page.evaluate(() => document.fonts.ready);
    const dialog = page.locator("[role='dialog']");
    await expect(dialog).toBeVisible();
    await expect(dialog).toHaveScreenshot(`${theme}-confirm.png`);
  });
}

/**
 * Reduced-motion baseline for the Spinner. Spinner.css flips the icon→text
 * fallback under [data-motion="reduced"], so the baseline must be captured
 * separately from the full-motion Spinner screenshot. We emulate the OS
 * preference (covers the @media branch) and set the lab hook (covers the
 * explicit [data-motion] branch) so both CSS paths are exercised.
 */
for (const theme of ["light", "dark"] as const) {
  test(`${theme}/spinner reduced-motion baseline`, async ({ page }) => {
    await page.emulateMedia({ reducedMotion: "reduce" });
    await page.goto(`http://localhost:1421/?nav=component-gallery&theme=${theme}`);
    await page.evaluate(() => document.fonts.ready);
    await page.evaluate((t) => {
      const html = document.documentElement;
      html.setAttribute("data-theme", t);
      html.setAttribute("data-motion", "reduced");
    }, theme);
    const locator = page.locator('[data-component-id="spinner"]');
    await expect(locator).toBeVisible();
    await expect(locator).toHaveScreenshot(`${theme}-spinner-reduced.png`);
  });
}
