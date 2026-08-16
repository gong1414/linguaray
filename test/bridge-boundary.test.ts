import { readdirSync, readFileSync, statSync } from "node:fs";
import { join } from "node:path";
import { describe, expect, it } from "vitest";

/**
 * Hygiene-5 bridge boundary. docs/UI-RULES.md 规则 3：`@tauri-apps/*` 只能
 * 出现在 src/bridge/。业务代码（页面、控制器、*-ipc 包装）一律从
 * ../bridge/* 转发模块导入，让 Tauri API 访问只有一个可审计的缝隙。
 * 测试文件豁免：它们 vi.mock 的正是底层模块。
 */

const ROOT = join(__dirname, "..");
const BRIDGE = "src/bridge";

function walk(dir: string, out: string[] = []): string[] {
  for (const name of readdirSync(dir)) {
    const p = join(dir, name);
    if (statSync(p).isDirectory()) walk(p, out);
    else out.push(p);
  }
  return out;
}

const offenders: string[] = [];
for (const abs of walk(join(ROOT, "src"))) {
  const rel = abs.slice(ROOT.length + 1);
  if (rel.startsWith(BRIDGE + "/")) continue;
  if (rel.endsWith(".test.ts") || rel.endsWith(".test.tsx")) continue;
  if (!/\.(ts|tsx)$/.test(rel)) continue;
  const text = readFileSync(abs, "utf8");
  // Static `from "@tauri-apps/…"` and dynamic `import("@tauri-apps/…")`.
  if (text.match(/["']@tauri-apps\//)) offenders.push(rel);
}

describe("bridge boundary (hygiene-5)", () => {
  it("no file outside src/bridge imports @tauri-apps", () => {
    expect(
      offenders,
      [
        "以下文件绕过了 bridge 缝隙直接导入 Tauri API（docs/UI-RULES.md 规则 3）。",
        "请改为从 src/bridge/* 转发模块导入；新插件依赖先在 bridge/ 加转发模块。",
      ].join(""),
    ).toEqual([]);
  });

  it("bridge re-exports stay importable (syntax + symbol sanity)", async () => {
    // Importing each seam in a non-Tauri context must not throw at module
    // load — the @tauri-apps modules are side-effect free outside Tauri.
    for (const mod of ["invoke", "event", "window", "opener", "clipboard", "dialog", "process"]) {
      await import(`../src/bridge/${mod}`);
    }
  });
});
