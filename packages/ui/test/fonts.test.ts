import { describe, it, expect } from "vitest";
import { existsSync, readFileSync, readdirSync, statSync } from "node:fs";
import { join } from "node:path";
import { createHash } from "node:crypto";

const FONTS_DIR = join(process.cwd(), "src/assets/fonts");

const EXPECTED: Record<string, { sha256: string; minBytes: number }> = {
  "inter-latin-wght-normal.woff2":
    { sha256: "3100e775e8616cd2611beecfa23a4263d7037586789b43f035236a2e6fbd4c62", minBytes: 48000 },
  "ibm-plex-mono-latin-400-normal.woff2":
    { sha256: "08949f728dc52d528e69b1667d15c89a5686a4ee9a296ff90983985f99c380f7", minBytes: 14000 },
  "ibm-plex-mono-latin-600-normal.woff2":
    { sha256: "0d1f0b8d0722224e32e9f28261bdc86c79115be73444ae5eceb73976a1bcdf83", minBytes: 15000 },
  "noto-sans-sc-chinese-simplified-400-normal.woff2":
    { sha256: "95e3633b6a98f764ba3adfb54504a0cd4799328c009adf9081d6c1850f9c4c78", minBytes: 1100000 },
  "noto-sans-sc-chinese-simplified-700-normal.woff2":
    { sha256: "e1df51edc00bce27b58044e829fb8ec6accc8a5daece475413de90d52818845c", minBytes: 1100000 },
};

const LICENSES = ["LICENSE-Inter.txt", "LICENSE-NotoSansSC.txt", "LICENSE-IBMPlexMono.txt"];

function sha256(path: string): string {
  return createHash("sha256").update(readFileSync(path)).digest("hex");
}

describe("local font packaging", () => {
  it("every woff2 exists with correct SHA-256 and min size", () => {
    for (const [filename, { sha256: expected, minBytes }] of Object.entries(EXPECTED)) {
      const path = join(FONTS_DIR, filename);
      expect(existsSync(path), `${filename} must exist`).toBe(true);
      expect(statSync(path).size, `${filename} ≥ ${minBytes}`).toBeGreaterThanOrEqual(minBytes);
      expect(sha256(path), `${filename} SHA-256`).toBe(expected);
    }
  });

  it("three OFL license files contain SIL Open Font License", () => {
    for (const filename of LICENSES) {
      const path = join(FONTS_DIR, filename);
      expect(existsSync(path), `${filename} must exist`).toBe(true);
      expect(readFileSync(path, "utf-8")).toContain("SIL Open Font License");
    }
  });

  it("total woff2 size is 2393380 bytes (< 3MB, measured)", () => {
    const woff2 = readdirSync(FONTS_DIR).filter((f) => f.endsWith(".woff2"));
    const total = woff2.reduce((s, f) => s + statSync(join(FONTS_DIR, f)).size, 0);
    expect(total).toBe(2393380);
    expect(total).toBeLessThan(3 * 1024 * 1024);
  });

  it("fonts.css has local src only, references all 5 files", () => {
    const css = readFileSync(join(process.cwd(), "src/styles/fonts.css"), "utf-8");
    expect(css.match(/url\(\s*['"]?https?:\/\//), "no remote URLs").toBeNull();
    for (const filename of Object.keys(EXPECTED)) {
      expect(css, `must reference ${filename}`).toContain(filename);
    }
  });

  it("index.css imports fonts.css", () => {
    expect(readFileSync(join(process.cwd(), "src/styles/index.css"), "utf-8")).toContain("fonts.css");
  });

  it("package.json has @fontsource in devDependencies (vendored)", () => {
    const pkg = JSON.parse(readFileSync(join(process.cwd(), "package.json"), "utf-8"));
    const dd = pkg.devDependencies ?? {};
    expect(dd["@fontsource-variable/inter"]).toBe("5.3.0");
    expect(dd["@fontsource/noto-sans-sc"]).toBe("5.3.0");
    expect(dd["@fontsource/ibm-plex-mono"]).toBe("5.3.0");
  });
});
