import { test, expect } from "@playwright/test";

test("SidebarItem: Tab focuses, Enter activates, Space activates", async ({ page }) => {
  // 使用隔离路由：只渲染一个 SidebarItem，无其他可聚焦控件
  await page.goto("http://localhost:1421/?nav=sidebar-isolated&theme=light");
  await page.evaluate(() => document.fonts.ready);
  const item = page.locator("button.sidebar-item");
  await expect(item).toBeVisible();
  // 真实 Tab 聚焦（页面中只有这一个可聚焦元素）
  await page.keyboard.press("Tab");
  await expect(item).toBeFocused();
  // Enter 激活
  let clicked = false;
  await page.exposeFunction("__trackClick", () => { clicked = true; });
  await item.evaluate((el) => el.addEventListener("click", () => (window as any).__trackClick()));
  await page.keyboard.press("Enter");
  expect(clicked).toBe(true);
  // Space 激活
  clicked = false;
  await page.keyboard.press("Space");
  expect(clicked).toBe(true);
});
