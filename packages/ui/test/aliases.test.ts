import { describe, it, expect } from "vitest";
import { readFileSync, readdirSync, existsSync } from "node:fs";
import { join } from "node:path";

const TOKENS_CSS = readFileSync(join(process.cwd(), "src/styles/tokens.css"), "utf-8");

/** 从 CSS 文本中按 selector 提取声明。selector 传原始字符串。 */
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

/** 合并多 selector 的声明（后覆盖先）。 */
function merge(...maps: Map<string, string>[]): Map<string, string> {
  const out = new Map<string, string>();
  for (const m of maps) for (const [k, v] of m) out.set(k, v);
  return out;
}

/** 递归解析 var() 到最终值。 */
function resolve(name: string, props: Map<string, string>, seen = new Set<string>()): string | null {
  if (seen.has(name)) return null;
  const raw = props.get(name);
  if (raw === undefined) return null;
  const vm = raw.match(/^var\(\s*(--[a-z0-9-]+)\s*(?:,\s*([^)]+))?\s*\)$/);
  if (vm) {
    const r = resolve(vm[1], props, new Set([...seen, name]));
    return r ?? vm[2]?.trim() ?? null;
  }
  return raw;
}

// Light: :root 先，再 [data-theme="light"] 覆盖
const LIGHT = merge(declarationsIn(":root"), declarationsIn('[data-theme="light"]'));
// Dark: :root 先，再 [data-theme="dark"] 覆盖
const DARK = merge(declarationsIn(":root"), declarationsIn('[data-theme="dark"]'));

const ALIASES: Record<string, string> = {
  "--color-primary-fill": "--color-brand-default",
  "--color-on-primary-fill": "--color-brand-on-fill",
  "--color-primary-fg": "--color-brand-fg",
  "--color-bg": "--color-canvas",
  "--color-fg": "--color-text-primary",
  "--color-bg-elevated": "--color-surface-default",
  "--color-fg-elevated": "--color-text-primary",
  "--color-bg-hover": "--color-surface-hover",
  "--color-bg-selected": "--color-surface-selected",
  "--color-bg-overlay": "--color-overlay",
  "--color-fg-muted": "--color-text-secondary",
  "--color-selected-fg": "--color-text-selected",
  "--color-ring": "--color-focus",
  "--color-success-fill": "--color-strong-fill-success",
  "--color-on-success-fill": "--color-strong-on-success",
  "--color-success-fg": "--color-status-success",
  "--color-warning-fill": "--color-strong-fill-warning",
  "--color-on-warning-fill": "--color-strong-on-warning",
  "--color-warning-fg": "--color-status-warning",
  "--color-destructive-fill": "--color-strong-fill-danger",
  "--color-on-destructive-fill": "--color-strong-on-danger",
  "--color-destructive-fg": "--color-status-danger",
  "--color-info-fill": "--color-strong-fill-info",
  "--color-on-info-fill": "--color-strong-on-info",
  "--color-info-fg": "--color-status-info",
  // Shadow legacy aliases (three-tier → two-tier during migration window).
  "--shadow-sm": "--shadow-raised",
  "--shadow-md": "--shadow-raised",
  "--shadow-lg": "--shadow-overlay",
};

describe("alias dependency graph (CSS text parse)", () => {
  for (const [theme, props] of [["light", LIGHT], ["dark", DARK]] as const) {
    describe(`${theme}`, () => {
      it("every alias declared exactly once", () => {
        for (const old of Object.keys(ALIASES)) {
          const c = (TOKENS_CSS.match(new RegExp(old.replace(/[-]/g, "[-]") + "\\s*:", "g")) || []).length;
          expect(c, `${old} count=${c}`).toBe(1);
        }
      });
      it("every alias target exists and resolves", () => {
        for (const [old, tgt] of Object.entries(ALIASES)) {
          expect(resolve(old, props), `${old} in ${theme}`).not.toBeNull();
          expect(resolve(tgt, props), `${tgt} in ${theme}`).not.toBeNull();
        }
      });
      it("no cycles", () => {
        for (const name of props.keys()) {
          const seen = new Set<string>();
          let cur = name;
          while (cur) {
            if (seen.has(cur)) throw new Error(`Cycle: ${[...seen, cur].join("→")}`);
            seen.add(cur);
            const tgt = props.get(cur)?.match(/^var\(\s*(--[a-z0-9-]+)\s*\)$/)?.[1];
            if (!tgt || !props.has(tgt)) break;
            cur = tgt;
          }
        }
      });
      it("alias final value == target final value", () => {
        for (const [old, tgt] of Object.entries(ALIASES)) {
          expect(resolve(old, props)).toBe(resolve(tgt, props));
        }
      });
    });
  }

  it("required directories exist (no skip on missing)", () => {
    const uiComp = join(process.cwd(), "src/components");
    const labSrc = join(process.cwd(), "../../apps/ui-lab/src");
    expect(existsSync(uiComp), "packages/ui/src/components must exist").toBe(true);
    expect(existsSync(labSrc), "apps/ui-lab/src must exist").toBe(true);
  });

  // R1-12: 启用（9 个组件已全部创建）
  describe("no --core-* outside tokens.css (recursive scan packages/ui + apps/ui-lab)", () => {
    it("no --core-* outside tokens.css", () => {
      const scanDirs = [
        join(process.cwd(), "src/components"),
        join(process.cwd(), "src/styles"),
        join(process.cwd(), "../../apps/ui-lab/src"),
      ];
      for (const dir of scanDirs) {
        expect(existsSync(dir), `${dir} must exist`).toBe(true);
        // 递归扫描 .css/.ts/.tsx
        function walk(d: string): string[] {
          const out: string[] = [];
          for (const entry of readdirSync(d, { withFileTypes: true })) {
            const full = join(d, entry.name);
            if (entry.isDirectory()) out.push(...walk(full));
            else if (/\.(css|tsx?)$/.test(entry.name)) out.push(full);
          }
          return out;
        }
        for (const f of walk(dir)) {
          // tokens.css 本身允许 --core-*；index.css/base.css 可能 import 但不直接引用 --core-*
          if (f.endsWith("tokens.css")) continue;
          const content = readFileSync(f, "utf-8");
          expect(content.match(/--core-/), `${f} must not use --core-*`).toBeNull();
        }
      }
    });
  });

  // TODO R1-12: 启用此测试（组件创建后）。该 guard 检查 9 个新组件文件是否存在且不使用
  // R1-12: 启用（9 个组件已全部创建）
  describe("new files do not use ANY legacy token name (generated from ALIASES keys)", () => {
    it("new files do not use ANY legacy token name (generated from ALIASES keys)", () => {
      // 从别名映射表生成旧 Token 名集合
      const legacyNames = Object.keys(ALIASES);
      const legacyRegex = new RegExp(
        legacyNames.map((n) => n.replace(/[-]/g, "[-]")).join("|"),
      );
      // 新组件文件清单（含 helper：providerPresentation.ts 和 providerTypes.ts 是 helper，豁免 CSS/TSX 配对要求但不豁免 legacy 检查）
      const newFiles = [
        "SegmentedControl.tsx", "SegmentedControl.css", "SegmentedControl.test.tsx",
        "ShortcutChip.tsx", "ShortcutChip.css", "ShortcutChip.test.tsx",
        "StatusBadge.tsx", "StatusBadge.css", "StatusBadge.test.tsx",
        "InlineError.tsx", "InlineError.css", "InlineError.test.tsx",
        "WindowChrome.tsx", "WindowChrome.css", "WindowChrome.test.tsx",
        "SidebarItem.tsx", "SidebarItem.css", "SidebarItem.test.tsx",
        "HistoryRow.tsx", "HistoryRow.css", "HistoryRow.test.tsx",
        "ProviderRow.tsx", "ProviderRow.css", "ProviderRow.test.tsx",
        "TranslationCard.tsx", "TranslationCard.css", "TranslationCard.test.tsx",
        // helper 文件：无 CSS/TSX 配对要求，但禁止 legacy token
        "providerPresentation.ts",
        "providerTypes.ts",
      ];
      const compDir = join(process.cwd(), "src/components");
      for (const fname of newFiles) {
        const p = join(compDir, fname);
        expect(existsSync(p), `${fname} must exist (no skip)`).toBe(true);
        const content = readFileSync(p, "utf-8");
        expect(content.match(legacyRegex), `${fname} must not use legacy tokens`).toBeNull();
      }
    });
  });
});
