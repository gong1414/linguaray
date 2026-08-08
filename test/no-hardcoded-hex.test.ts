import { describe, it, expect } from "vitest";
import { readFileSync, readdirSync } from "node:fs";
import { join } from "node:path";

/** Scan all .css/.tsx/.ts files under src/ for raw hex color literals.
 *  Token values use var(--...) — hex is only allowed in the fallback slot
 *  of a var() declaration (e.g. var(--token, #fallback)). */
const HEX_OUTSIDE_VAR = /(?<!var\([^)]*,\s*)(#[0-9a-fA-F]{3,8})\b/g;

function walkCss(dir: string): string[] {
  const out: string[] = [];
  for (const entry of readdirSync(dir, { withFileTypes: true })) {
    const full = join(dir, entry.name);
    if (entry.isDirectory()) out.push(...walkCss(full));
    else if (/\.(css|tsx?|ts)$/.test(entry.name) && !entry.name.endsWith(".test.ts")) {
      out.push(full);
    }
  }
  return out;
}

describe("no hardcoded hex outside var() fallback in src/", () => {
  it("src/App.css has no raw hex outside var() fallback", () => {
    const css = readFileSync("src/App.css", "utf-8");
    // Strip var() fallbacks: var(--token, #hex) → var(--token)
    const stripped = css.replace(/var\((--[^,)]+),\s*#[^)]+\)/g, "var($1)");
    const matches = stripped.match(HEX_OUTSIDE_VAR);
    expect(matches, `raw hex found: ${matches?.join(", ")}`).toBeNull();
  });

  it("no .css file under src/ has raw hex outside var() fallback", () => {
    const files = walkCss("src").filter((f) => f.endsWith(".css"));
    for (const f of files) {
      const css = readFileSync(f, "utf-8");
      const stripped = css.replace(/var\((--[^,)]+),\s*#[^)]+\)/g, "var($1)");
      const matches = stripped.match(HEX_OUTSIDE_VAR);
      expect(matches, `${f}: raw hex ${matches?.join(", ")}`).toBeNull();
    }
  });
});
