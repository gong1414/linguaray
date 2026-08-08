import { describe, it, expect } from "vitest";
import { readFileSync } from "node:fs";
import { join } from "node:path";

const TOKENS_CSS = readFileSync(join(process.cwd(), "src/styles/tokens.css"), "utf-8");

/** 按 selector 提取声明（与 aliases.test.ts 相同的纯文本解析方式）。 */
function declarationsIn(selector: string): Map<string, string> {
  const map = new Map<string, string>();
  const escaped = selector.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
  const re = new RegExp(escaped + "\\s*\\{([^}]*)\\}", "g");
  let m: RegExpExecArray | null;
  while ((m = re.exec(TOKENS_CSS)) !== null) {
    const dr = /(--[a-z0-9-]+)\s*:\s*([^;]+);/g;
    let dm: RegExpExecArray | null;
    while ((dm = dr.exec(m[1])) !== null) map.set(dm[1], dm[2].trim());
  }
  return map;
}
function merge(...maps: Map<string, string>[]): Map<string, string> {
  const out = new Map<string, string>();
  for (const m of maps) for (const [k, v] of m) out.set(k, v);
  return out;
}
function resolve(name: string, props: Map<string, string>, seen = new Set<string>()): string | null {
  if (seen.has(name)) return null;
  const raw = props.get(name);
  if (raw === undefined) return null;
  const vm = raw.match(/^var\(\s*(--[a-z0-9-]+)\s*(?:,\s*([^)]+))?\s*\)$/);
  if (vm) return resolve(vm[1], props, new Set([...seen, name])) ?? vm[2]?.trim() ?? null;
  return raw;
}

const LIGHT = merge(declarationsIn(":root"), declarationsIn('[data-theme="light"]'));
const DARK = merge(declarationsIn(":root"), declarationsIn('[data-theme="dark"]'));

/** 从 Token 名获取 hex 值（不手写 hex，全部从 tokens.css 解析）。 */
function tokenHex(name: string, theme: Map<string, string>): string {
  const v = resolve(name, theme);
  expect(v, `${name} must resolve in tokens.css`).not.toBeNull();
  expect(v!.startsWith("#"), `${name} must be a hex value`).toBe(true);
  return v!;
}

function hexToRgb(h: string): [number, number, number] {
  const c = h.replace("#", "");
  return [parseInt(c.slice(0, 2), 16), parseInt(c.slice(2, 4), 16), parseInt(c.slice(4, 6), 16)];
}
function lum([r, g, b]: [number, number, number]): number {
  const f = (v: number) => { v /= 255; return v <= 0.03928 ? v / 12.92 : ((v + 0.055) / 1.055) ** 2.4; };
  return 0.2126 * f(r) + 0.7152 * f(g) + 0.0722 * f(b);
}
function ratio(fgHex: string, bgHex: string): number {
  const [l1, l2] = [lum(hexToRgb(fgHex)), lum(hexToRgb(bgHex))];
  return (Math.max(l1, l2) + 0.05) / (Math.min(l1, l2) + 0.05);
}

/** 从 Token 名计算对比度（不手写 hex）。 */
function tokenRatio(fgToken: string, bgToken: string, theme: Map<string, string>): number {
  return ratio(tokenHex(fgToken, theme), tokenHex(bgToken, theme));
}

const AA = 4.5;
const UI = 3.0;

describe("WCAG contrast from tokens.css (no hand-written hex)", () => {
  // Brand: on-fill vs brand.default
  it("Light: brand-on-fill on brand-default ≥ 4.5", () =>
    expect(tokenRatio("--color-brand-on-fill", "--color-brand-default", LIGHT)).toBeGreaterThanOrEqual(AA));
  it("Dark: brand-on-fill on brand-default ≥ 4.5", () =>
    expect(tokenRatio("--color-brand-on-fill", "--color-brand-default", DARK)).toBeGreaterThanOrEqual(AA));

  // Brand: brand.default vs surface.default (可见性)
  it("Light: brand-default on surface-default ≥ 4.5", () =>
    expect(tokenRatio("--color-brand-default", "--color-surface-default", LIGHT)).toBeGreaterThanOrEqual(AA));
  it("Dark: brand-default on surface-default ≥ 4.5", () =>
    expect(tokenRatio("--color-brand-default", "--color-surface-default", DARK)).toBeGreaterThanOrEqual(AA));

  // Focus vs surface (UI 3:1)
  it("Light: focus on surface-default ≥ 3", () =>
    expect(tokenRatio("--color-focus", "--color-surface-default", LIGHT)).toBeGreaterThanOrEqual(UI));
  it("Dark: focus on surface-default ≥ 3", () =>
    expect(tokenRatio("--color-focus", "--color-surface-default", DARK)).toBeGreaterThanOrEqual(UI));

  // Strong fill on-fill ≥ 4.5（两主题，从工程扩展 Token 读取）
  for (const s of ["success", "warning", "danger", "info"] as const) {
    it(`strong-fill ${s}: on-${s} on strong-fill-${s} ≥ 4.5 (light)`, () =>
      expect(tokenRatio(`--color-strong-on-${s}`, `--color-strong-fill-${s}`, LIGHT)).toBeGreaterThanOrEqual(AA));
    it(`strong-fill ${s}: on-${s} on strong-fill-${s} ≥ 4.5 (dark)`, () =>
      expect(tokenRatio(`--color-strong-on-${s}`, `--color-strong-fill-${s}`, DARK)).toBeGreaterThanOrEqual(AA));
  }

  // Strong fill vs surface ≥ 3
  for (const s of ["success", "warning", "danger", "info"] as const) {
    it(`strong-fill ${s} on Light surface ≥ 3`, () =>
      expect(tokenRatio(`--color-strong-fill-${s}`, "--color-surface-default", LIGHT)).toBeGreaterThanOrEqual(UI));
    it(`strong-fill ${s} on Dark surface ≥ 3`, () =>
      expect(tokenRatio(`--color-strong-fill-${s}`, "--color-surface-default", DARK)).toBeGreaterThanOrEqual(UI));
  }

  // StatusBadge Light: dedicated status-*-fg foregrounds on status-*-soft ≥ 4.5.
  // (Penpot success/warning/danger.default values — e.g. danger #DC2626 on soft
  // #FEF2F2 = 4.41 — miss AA, so StatusBadge soft mode consumes the dedicated
  // --color-status-*-fg [工程扩展] tokens defined in tokens.css.)
  const lightBadgeForegrounds = {
    success: "--color-status-success-fg",
    warning: "--color-status-warning-fg",
    danger: "--color-status-danger-fg",
    info: "--color-status-info-fg",
  } as const;
  for (const [status, foreground] of Object.entries(lightBadgeForegrounds)) {
    it(`Badge Light: ${status} foreground on soft background ≥ 4.5`, () => {
      expect(
        tokenRatio(
          foreground,
          `--color-status-${status}-soft`,
          LIGHT,
        ),
      ).toBeGreaterThanOrEqual(AA);
    });
  }

  // StatusBadge Dark: same --color-status-*-fg tokens on Dark soft backgrounds ≥ 4.5.
  for (const status of ["success", "warning", "danger", "info"] as const) {
    it(`Badge Dark: ${status} foreground on soft background ≥ 4.5`, () => {
      expect(
        tokenRatio(
          `--color-status-${status}-fg`,
          `--color-status-${status}-soft`,
          DARK,
        ),
      ).toBeGreaterThanOrEqual(AA);
    });
  }

  // border.control vs canvas (UI 3:1)
  it("Light: border-control on canvas ≥ 3", () =>
    expect(tokenRatio("--color-border-control", "--color-canvas", LIGHT)).toBeGreaterThanOrEqual(UI));
  it("Dark: border-control on canvas ≥ 3", () =>
    expect(tokenRatio("--color-border-control", "--color-canvas", DARK)).toBeGreaterThanOrEqual(UI));
});
