import { test, expect } from "@playwright/test";

/**
 * Real SettingsShell keyboard e2e (P1-4).
 *
 * Unlike `sidebar-keyboard.spec.ts` (which drives an ISOLATED SidebarItem),
 * this spec exercises the REAL production SettingsShell imported from
 * `@app/features/settings/SettingsShell` via the hidden `?nav=settings-keyboard`
 * fixture route. The shell is rendered in controlled mode
 * (activePage/onNavigate), so every Tab/Enter here drives the actual
 * shell's SidebarItem nav + data-page transitions — a regression in the
 * shell's focus management or navigation gating surfaces here as a
 * failed assertion, not just a missed unit test.
 *
 * The fixture renders ONLY the shell (no lab header/nav/state-bar), so the
 * first Tab lands directly on the shell's sidebar. The SettingsShell sidebar
 * exposes four items in DOM order:
 *   1. Provider Center   (enabled, initial active)
 *   2. Keystore Recovery (enabled)
 *   3. Shortcuts         (enabled in R3b)
 *   4. Privacy           (enabled in R3b)
 */

const BASE = "http://localhost:1421";

/** Selector for the focused sidebar item WITHIN the real shell. Scoping under
 *  `[data-testid="shell"]` guarantees we hit the production shell's own nav,
 *  not any stray lab control. */
const FOCUSED_ITEM =
  "[data-testid='shell'] .settings-shell__nav .sidebar-item:focus";

/** Press Tab until a sidebar item in the real shell receives focus.
 *  Caps at 12 presses (the fixture has no preceding focusables, so this
 *  resolves on the first press; the cap is a defensive guard against future
 *  DOM changes). Throws if no sidebar item is reached. */
async function tabToSidebarItem(page: import("@playwright/test").Page, cap = 12) {
  for (let i = 0; i < cap; i++) {
    await page.keyboard.press("Tab");
    const focused = page.locator(FOCUSED_ITEM);
    if ((await focused.count()) > 0) return focused;
  }
  throw new Error(`No sidebar item focused after ${cap} Tab presses`);
}

test("SettingsShell: Tab focuses nav and Enter reaches every R3b destination", async ({
  page,
}) => {
  // Wide viewport → data-layout="full" (labels visible).
  await page.setViewportSize({ width: 1024, height: 700 });
  await page.goto(`${BASE}/?nav=settings-keyboard&theme=light`);
  await page.evaluate(() => document.fonts.ready);
  await page.waitForSelector("[data-testid='shell']");
  await expect(page.locator("[data-testid='shell']")).toHaveAttribute(
    "data-page",
    "provider-center",
  );

  // 1. Real Tab focuses the first sidebar item. Assert it is NOT disabled.
  const first = await tabToSidebarItem(page);
  await expect(first).toHaveAttribute("aria-label", "Provider Center");
  await expect(first).not.toHaveAttribute("aria-disabled", "true");
  const firstLabel = (await first.getAttribute("aria-label"))!;

  // 2. Tab again → a DIFFERENT item with a different aria-label.
  await page.keyboard.press("Tab");
  const second = page.locator(FOCUSED_ITEM);
  await expect(second).toHaveAttribute("aria-label", "Keystore Recovery");
  await expect(second).not.toHaveAttribute("aria-disabled", "true");
  const secondLabel = (await second.getAttribute("aria-label"))!;
  expect(secondLabel).not.toEqual(firstLabel);

  // 3. Enter on the enabled 2nd item → data-page CHANGES.
  await page.keyboard.press("Enter");
  await expect(page.locator("[data-testid='shell']")).toHaveAttribute(
    "data-page",
    "keystore-recovery",
  );

  // 4. Tab back (Shift+Tab) to the first item → Enter → data-page CHANGES back.
  await page.keyboard.press("Shift+Tab");
  const back = page.locator(FOCUSED_ITEM);
  await expect(back).toHaveAttribute("aria-label", "Provider Center");
  await page.keyboard.press("Enter");
  await expect(page.locator("[data-testid='shell']")).toHaveAttribute(
    "data-page",
    "provider-center",
  );

  // 5. Tab forward to Shortcuts → Enter → live R3b destination.
  //    From Provider Center: Tab → Keystore Recovery, Tab → Shortcuts.
  await page.keyboard.press("Tab"); // → Keystore Recovery
  await page.keyboard.press("Tab"); // → Shortcuts
  const shortcuts = page.locator(FOCUSED_ITEM);
  await expect(shortcuts).not.toHaveAttribute("aria-disabled", "true");
  await page.keyboard.press("Enter");
  await expect(page.locator("[data-testid='shell']")).toHaveAttribute(
    "data-page",
    "shortcuts",
  );
});

test("SettingsShell rail mode @699px: keyboard nav still works, items collapse to icons", async ({
  page,
}) => {
  // 699px → matchMedia("(min-width: 700px)") is false → data-layout="rail".
  // Every item is wrapped in a Tooltip in rail mode; this verifies the Tooltip
  // wrapper does not steal focus from the underlying sidebar-item button.
  await page.setViewportSize({ width: 699, height: 700 });
  await page.goto(`${BASE}/?nav=settings-keyboard&theme=light`);
  await page.evaluate(() => document.fonts.ready);
  await page.waitForSelector("[data-testid='shell']");
  await expect(page.locator("[data-testid='shell']")).toHaveAttribute(
    "data-layout",
    "rail",
  );

  // Tab to first item; it is focusable and not disabled even in rail mode.
  const first = await tabToSidebarItem(page);
  await expect(first).toHaveAttribute("aria-label", "Provider Center");
  await expect(first).not.toHaveAttribute("aria-disabled", "true");

  // Tab to the second item and activate it → data-page changes.
  await page.keyboard.press("Tab");
  const second = page.locator(FOCUSED_ITEM);
  await expect(second).toHaveAttribute("aria-label", "Keystore Recovery");
  await page.keyboard.press("Enter");
  await expect(page.locator("[data-testid='shell']")).toHaveAttribute(
    "data-page",
    "keystore-recovery",
  );

  // Shortcuts remains a live keyboard destination in rail mode too.
  await page.keyboard.press("Tab"); // → Shortcuts
  const shortcuts = page.locator(FOCUSED_ITEM);
  await expect(shortcuts).not.toHaveAttribute("aria-disabled", "true");
  await page.keyboard.press("Enter");
  await expect(page.locator("[data-testid='shell']")).toHaveAttribute(
    "data-page",
    "shortcuts",
  );
});
