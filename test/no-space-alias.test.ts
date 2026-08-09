import { describe, it, expect } from "vitest";
import { readFileSync, readdirSync } from "node:fs";
import { join } from "node:path";

const ALIAS = /--space-[0-9]/;

function walk(dir: string): string[] {
  const out: string[] = [];
  for (const entry of readdirSync(dir, { withFileTypes: true })) {
    const full = join(dir, entry.name);
    if (entry.isDirectory()) out.push(...walk(full));
    else if (/\.(css|tsx?|ts)$/.test(entry.name) && !entry.name.endsWith(".test.ts")) {
      out.push(full);
    }
  }
  return out;
}

describe("no --space-N legacy aliases in src/", () => {
  it("no src/ file uses --space-1/2/3/etc.", () => {
    const files = walk("src");
    const offenders: string[] = [];
    for (const f of files) {
      const src = readFileSync(f, "utf-8");
      const stripped = src
        .replace(/\/\*[\s\S]*?\*\//g, "")
        .replace(/\/\/.*$/gm, "");
      if (ALIAS.test(stripped)) offenders.push(f);
    }
    expect(offenders, `legacy --space-N alias found in: ${offenders.join(", ")}`).toEqual([]);
  });
});
