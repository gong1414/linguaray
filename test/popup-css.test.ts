import { describe, it, expect } from "vitest";
import { readFileSync } from "node:fs";

/**
 * P2-1: Popup.css must NOT define `.container` — App.css also defines
 * `.container` and is imported AFTER Popup.css in Popup.tsx, so App.css wins
 * (opaque background + min-height:100vh, which breaks the transparent popup).
 * Popup uses the isolated `.popup-shell` selector instead.
 */
describe("Popup CSS isolation (P2-1)", () => {
  it("Popup.css does not define a .container selector", () => {
    const css = readFileSync("src/Popup.css", "utf-8");
    // Match `.container` as a selector (rule start), not just any occurrence.
    expect(css, "Popup.css must not contain a .container selector").not.toMatch(
      /\.container\b/,
    );
    expect(css).toMatch(/\.popup-shell\b/);
  });

  it("Popup.tsx renders the popup-shell class, not container", () => {
    const tsx = readFileSync("src/Popup.tsx", "utf-8");
    expect(tsx).toContain('class="popup-shell"');
    expect(tsx).toContain('"popup-shell--compact"');
    // The generic `container` className must be gone from Popup.
    expect(tsx).not.toMatch(/class(?:Name)?=["']\s*container["']/);
    expect(tsx).not.toContain('"container--compact"');
  });
});
