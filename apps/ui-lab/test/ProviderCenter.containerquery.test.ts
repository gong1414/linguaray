/**
 * Provider Center — container-query boundary contract (699/700px).
 *
 * The Settings shell uses CSS container queries (@container) to collapse the
 * nav rail to icon-only at ≤699px and show full labels at ≥700px. This is a
 * LAYOUT behavior driven by the shell's own inline-size — jsdom does not
 * perform layout, so the rail width / label visibility cannot be measured by
 * reading computed styles here.
 *
 * What this test DOES assert authoritatively: the CSS rule contract that
 * drives the boundary. If the container-type declaration, the 699px icon-only
 * rule, or the 700px label rule is removed or altered, this fails. The actual
 * pixel measurement at the boundary is verified manually in a real browser
 * (see verification notes) because no headless/jsdom harness can lay out
 * container queries.
 */
import { describe, it, expect } from "vitest";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";

const cssPath = resolve(import.meta.dirname, "../src/pages/ProviderCenter.css");
const css = readFileSync(cssPath, "utf8");

describe("Provider Center — container-query boundary contract (699/700px)", () => {
  it(".pc__settings-shell declares container-type: inline-size (the query target)", () => {
    // The shell must be the container so its OWN width is the query axis.
    expect(/\.pc__settings-shell\s*\{[^}]*container-type:\s*inline-size/s.test(css)).toBe(true);
  });

  it("@container (max-width: 699px) collapses the rail to 48px and hides labels", () => {
    // Extract from the @container line to the next @container (or EOF) — the
    // outer block contains nested rule braces, so match broadly.
    const startIdx = css.indexOf("@container (max-width: 699px)");
    const nextContainerIdx = css.indexOf("@container", startIdx + 1);
    const block = css.slice(startIdx, nextContainerIdx > 0 ? nextContainerIdx : undefined);
    expect(block.length).toBeGreaterThan(0);
    // rail width → 48px
    expect(/\.pc__settings-rail\s*\{[^}]*width:\s*48px/.test(block)).toBe(true);
    // label → display: none (icon-only)
    expect(/\.pc__rail-item__label\s*\{[^}]*display:\s*none/.test(block)).toBe(true);
  });

  it("@container (min-width: 700px) restores the 180px rail and visible labels", () => {
    const startIdx = css.indexOf("@container (min-width: 700px)");
    const nextContainerIdx = css.indexOf("@container", startIdx + 1);
    const block = css.slice(startIdx, nextContainerIdx > 0 ? nextContainerIdx : undefined);
    expect(block.length).toBeGreaterThan(0);
    // rail width → 180px
    expect(/\.pc__settings-rail\s*\{[^}]*width:\s*180px/.test(block)).toBe(true);
    // label → display: inline
    expect(/\.pc__rail-item__label\s*\{[^}]*display:\s*inline/.test(block)).toBe(true);
  });

  it("the boundary is exactly 699→700 (no off-by-one gap)", () => {
    // 699 is the icon-only ceiling; 700 is the label floor. Together they
    // cover the whole range with no uncovered width.
    expect(css).toMatch(/@container\s*\(max-width:\s*699px\)/);
    expect(css).toMatch(/@container\s*\(min-width:\s*700px\)/);
  });
});
