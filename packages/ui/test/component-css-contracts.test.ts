/**
 * Component CSS contract assertions.
 *
 * These tests parse component CSS files (text) to enforce MASTER §7 contract
 * details that jsdom cannot compute (e.g. resolved token values, pixel sizes).
 * Lives in test/ (outside tsconfig "src" include) so node:fs is available and
 * the typecheck (tsc over src) stays green without @types/node.
 */
import { describe, it, expect } from "vitest";
import { readFileSync } from "node:fs";
import { join } from "node:path";

function readComponentCss(name: string): string {
  return readFileSync(join(process.cwd(), "src/components", name), "utf-8");
}

/** Read any component source file by relative path under src/components. */
function readComponentSource(relPath: string): string {
  return readFileSync(join(process.cwd(), "src/components", relPath), "utf-8");
}

function ruleBlock(css: string, selector: string): string {
  return css.match(new RegExp(selector.replace(/[.*+?^${}()|[\]\\]/g, "\\$&") + "\\s*\\{([^}]*)\\}"))?.[1] ?? "";
}

describe("ShortcutChip CSS contracts (MASTER §7)", () => {
  const css = readComponentCss("ShortcutChip.css");

  it("recording border uses --color-focus (not brand-default)", () => {
    const block = ruleBlock(css, ".shortcut-chip--recording");
    expect(block, "recording rule must exist").not.toBe("");
    expect(block).toMatch(/--color-focus/);
    expect(block).not.toMatch(/--color-brand-default/);
  });
});

describe("StatusBadge CSS contracts (MASTER §7)", () => {
  const css = readComponentCss("StatusBadge.css");

  it("dot is 8px (not 6px)", () => {
    const block = ruleBlock(css, ".status-badge__dot");
    expect(block, "dot rule must exist").not.toBe("");
    expect(block).toMatch(/width:\s*8px/);
    expect(block).toMatch(/height:\s*8px/);
    expect(block).not.toMatch(/6px/);
  });

  it("info variant uses indigo (brand) tokens, not blue info tokens", () => {
    const block = ruleBlock(css, ".status-badge--info");
    expect(block, "info rule must exist").not.toBe("");
    // indigo.700 fg via brand-hover, indigo.50 bg via brand-soft
    expect(block).toMatch(/--color-brand-hover/);
    expect(block).toMatch(/--color-brand-soft/);
    // must NOT consume the blue info engineering-extension tokens
    expect(block).not.toMatch(/--color-status-info/);
  });
});

describe("ProviderRow CSS contracts (MASTER §7)", () => {
  const css = readComponentCss("ProviderRow.css");

  // Active provider accent: --color-surface-selected border-left (3px), NOT
  // --color-brand-default.
  it("active border-left uses --color-surface-selected (not brand-default)", () => {
    const block = ruleBlock(css, ".provider-row--active");
    expect(block, "active rule must exist").not.toBe("");
    expect(block).toMatch(/--color-surface-selected/);
    expect(block).not.toMatch(/--color-brand-default/);
  });
});

describe("ProviderCard / ProviderRow shared presentation (MASTER §7)", () => {
  // Both components must consume the shared providerPresentation module so the
  // visual model stays in sync. ProviderCard must import from providerPresentation.
  it("ProviderCard imports from providerPresentation", () => {
    const src = readComponentSource("ProviderCard.tsx");
    expect(src).toMatch(/from\s+["']\.\/providerPresentation["']/);
    expect(src).toMatch(/\bproviderKeyStatus\b/);
  });
});
