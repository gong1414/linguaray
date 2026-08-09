import { describe, it, expect } from "vitest";
import { readFileSync } from "node:fs";
import { join } from "node:path";

// Entry styling (R2 audit): the settings window's <head> must brand LinguaRay,
// carry a favicon, and expose both light + dark theme-color metas (the dark
// token is the slate-950 scale #020617, NOT #000000). index.html is the single
// source for these metas; this test guards against regressions to the Tauri
// starter default ("Tauri + Solid + Typescript App").
const HTML = readFileSync(
  join(process.cwd(), "index.html"),
  "utf-8",
);

function meta(name: string, media?: string): string | undefined {
  const lines = HTML.split("\n");
  for (const line of lines) {
    if (
      line.includes(`name="${name}"`) &&
      (!media || line.includes(`media="${media}"`))
    ) {
      const m = line.match(/content="([^"]+)"/);
      return m ? m[1] : undefined;
    }
  }
  return undefined;
}

describe("index.html entry branding (R2 audit)", () => {
  it("document <title> is LinguaRay", () => {
    expect(HTML).toMatch(/<title>LinguaRay[^<]*<\/title>/);
    expect(HTML).not.toContain("Tauri + Solid + Typescript App");
  });

  it("favicon points at the SVG logo", () => {
    expect(HTML).toContain('rel="icon"');
    expect(HTML).toContain('href="/src/assets/logo.svg"');
  });

  it("light theme-color meta is #F8FAFC", () => {
    const light = meta("theme-color", "(prefers-color-scheme: light)");
    expect(light).toBe("#F8FAFC");
  });

  it("dark theme-color meta is the slate-950 token #020617", () => {
    const dark = meta("theme-color", "(prefers-color-scheme: dark)");
    expect(dark).toBe("#020617");
  });

  it("never uses pure black #000000 as a theme-color", () => {
    const lines = HTML.split("\n");
    for (const line of lines) {
      if (line.includes('name="theme-color"')) {
        const m = line.match(/content="([^"]+)"/);
        const content = m ? m[1] : "";
        expect(content.toLowerCase(), `theme-color ${content} must not be #000000`).not.toBe("#000000");
      }
    }
  });
});
