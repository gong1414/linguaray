import { expect, test } from "@playwright/test";

/**
 * Storybook visual baselines: screenshot EVERY story (125+ states across all
 * windows/pages incl. loading/empty/error/dark/long-CJK/narrow variants).
 * Dark stories render via their own forceColorScheme provider; narrow stories
 * use Storybook's viewport — the iframe viewport here is the page viewport.
 */
type StoryIndex = { entries: Record<string, { id: string; name: string; importPath: string }> };

test.beforeEach(async ({ page }) => {
  await page.goto("http://localhost:6008/index.json");
});

test.setTimeout(600_000);

test("every story matches its visual baseline", async ({ page, request }) => {
  const index = (await (await request.get("http://localhost:6008/index.json")).json()) as StoryIndex;
  const entries = Object.values(index.entries).filter((e) => !/docs$/i.test(e.id));
  test.info().annotations.push({ type: "story-count", description: String(entries.length) });
  expect(entries.length).toBeGreaterThan(100);

  for (const entry of entries) {
    await page.goto(`http://localhost:6008/iframe.html?id=${encodeURIComponent(entry.id)}&viewMode=story`, {
      waitUntil: "load",
    });
    // Wait for the story to mount (attached, not visible — empty states like a
    // healthy keystore section legitimately have zero height).
    await page.waitForSelector("#storybook-root *", { state: "attached", timeout: 15_000 });
    await page.evaluate(() => document.fonts.ready);
    // Small settle for Mantine runtime CSS-variable injection + animations.
    await page.waitForTimeout(300);
    await expect(page).toHaveScreenshot(`${entry.id.replace(/[^a-z0-9-]/gi, "_")}.png`);
  }
});
