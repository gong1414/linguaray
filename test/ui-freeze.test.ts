import { readdirSync, readFileSync, statSync } from "node:fs";
import { join } from "node:path";
import { describe, expect, it } from "vitest";

/**
 * Hygiene-4 UI freeze. docs/UI-RULES.md 规则 1/2：
 *  - packages/ui 组件清单冻结：新增自制组件必须显式修改本清单并给出理由
 *    （React + Mantine 迁移前的止血措施）。
 *  - 生产代码禁止 WindowChrome（普通窗口一律系统原生标题栏；ui-lab 的
 *    组件画廊是唯一豁免，因为它就是用来展示组件的）。
 */

const ROOT = join(__dirname, "..");
const COMPONENTS_DIR = join(ROOT, "packages", "ui", "src", "components");

/** FROZEN 2026-08-16 — see docs/UI-RULES.md before touching this list. */
const FROZEN_COMPONENTS = [
  "Banner", "Button", "Confirm", "Dialog", "EmptyState", "HistoryRow",
  "IconButton", "InlineError", "ListRow", "ProviderCard", "ProviderRow",
  "ResultCard", "SegmentedControl", "Select", "ShortcutChip", "SidebarItem",
  "Spinner", "StatusBadge", "Switch", "TextArea", "TextField", "Toast",
  "Tooltip", "TranslationCard", "VisuallyHidden", "WindowChrome",
];

function componentNames(): string[] {
  return readdirSync(COMPONENTS_DIR)
    .filter((f) => f.endsWith(".tsx") && !f.endsWith(".test.tsx"))
    .map((f) => f.replace(/\.tsx$/, ""))
    .sort();
}

describe("UI component freeze (hygiene-4)", () => {
  it("packages/ui component list matches the frozen manifest", () => {
    expect(componentNames(), [
      "packages/ui 组件清单已冻结（docs/UI-RULES.md 规则 1）。",
      "新增组件 = 停止扩建迁移前 UI 的例外操作：请先阅读该规则，",
      "确认理由后在 test/ui-freeze.test.ts 的 FROZEN_COMPONENTS 中显式登记。",
    ].join("")).toEqual(FROZEN_COMPONENTS);
  });

  it("production src/ never uses WindowChrome (native title bar only)", () => {
    const offenders: string[] = [];
    const walk = (dir: string) => {
      for (const name of readdirSync(dir)) {
        const p = join(dir, name);
        if (statSync(p).isDirectory()) {
          walk(p);
        } else if (/\.(tsx|ts)$/.test(name)) {
          const text = readFileSync(p, "utf8");
          if (/from\s+["']@linguaray\/ui["']/.test(text) && /WindowChrome/.test(text)) {
            offenders.push(p.slice(ROOT.length + 1));
          }
        }
      }
    };
    walk(join(ROOT, "src"));
    expect(
      offenders,
      "生产代码禁止 WindowChrome（docs/UI-RULES.md 规则 2：普通窗口用系统标题栏）。",
    ).toEqual([]);
  });
});
