import { existsSync, readdirSync, readFileSync, statSync } from "node:fs";
import { join } from "node:path";
import { describe, expect, it } from "vitest";

/**
 * UI freeze (docs/UI-RULES.md). The self-built kit (`packages/ui`) is now
 * DELETED — the migration to React + Mantine completed 2026-08-16. This gate
 * keeps it deleted (no re-birth of a parallel component kit) and keeps
 * production code free of WindowChrome-style custom window chrome.
 */

const ROOT = join(__dirname, "..");

describe("UI freeze (Phase 5)", () => {
  it("the self-built @linguaray/ui kit stays deleted", () => {
    expect(
      existsSync(join(ROOT, "packages", "ui")),
      "packages/ui was deleted in migration Phase 5. Do not resurrect a parallel " +
        "component kit — build with Mantine (docs/UI-RULES.md).",
    ).toBe(false);
  });

  it("package.json no longer depends on Solid or the legacy kit", () => {
    const pkg = JSON.parse(readFileSync(join(ROOT, "package.json"), "utf8"));
    const deps = { ...pkg.dependencies, ...pkg.devDependencies };
    for (const banned of ["solid-js", "lucide-solid", "@linguaray/ui", "vite-plugin-solid"]) {
      expect(deps[banned], `${banned} must stay removed`).toBeUndefined();
    }
  });

  it("production src/ has no WindowChrome / custom window chrome imports", () => {
    const offenders: string[] = [];
    const walk = (dir: string) => {
      for (const name of readdirSync(dir)) {
        const p = join(dir, name);
        if (statSync(p).isDirectory()) walk(p);
        else if (/\.(tsx|ts)$/.test(name)) {
          const text = readFileSync(p, "utf8");
          if (/WindowChrome/.test(text)) offenders.push(p.slice(ROOT.length + 1));
        }
      }
    };
    walk(join(ROOT, "src"));
    expect(offenders, "No custom window chrome in production code.").toEqual([]);
  });
});
