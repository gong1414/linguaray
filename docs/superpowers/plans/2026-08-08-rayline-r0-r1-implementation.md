# LinguaRay Rayline R0–R1 实施计划（rev-4.3.2）

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 以 Penpot 实际 Token 为唯一设计源（Core 97 + Semantic Light 28 + Semantic Dark 28 = 153），冻结 Rayline 设计契约（R0），然后实现双层 Token 体系、9 个缺失设计组件、UI Lab 状态画廊和 42 张视觉回归截图基线（R1，20 组件×2 主题 + 2 reduced-motion）。

**Architecture:** R0 以 Penpot 三集合为唯一设计源，区分「Penpot 原生 Token」与「工程扩展 Token」，产出 MASTER.md + Token 映射 + manifest + 16 Surface 页面。R1 重写 tokens.css（唯一可引用 Core 的文件），新增 9 个缺失组件，构建 ComponentGallery + Playwright 视觉回归基线（42 张截图，20 组件×2 主题 + 2 reduced-motion）。

**Tech Stack:** Tauri 2 + Rust + SolidJS + TypeScript；`@linguaray/ui` + `@linguaray/ui-lab`；Kobalte；lucide-solid；axe-core；Vitest；@playwright/test；CSS Custom Properties。

## 前言

### .mimosa/ 核实记录

新 worktree `.mimosa/` 经核实为真实目录（非符号链接），inode 13178074 ≠ 原工作区 13070136，Mimosa 插件自动生成。不阻塞 Git 操作。按授权不读取、不复制、不修改、不删除。

### 基线验证（已通过）

| 命令 | 结果 |
|---|---|
| `pnpm typecheck` | ✅ |
| `pnpm test` | ✅ ui 204 + ui-lab 201 = 405 通过（rev-4.3.2） |
| `pnpm build` | ✅ |
| `pnpm --filter @linguaray/ui-lab build` | ✅ |
| `cd src-tauri && cargo test --features xproc-test-helper` | ✅ 53 通过 |
| `cd src-tauri && cargo clippy --all-targets --features xproc-test-helper -- -D warnings` | ✅ |

### 组件精确盘点（核对源码）

**18 设计组件 = 9 已有 + 9 新增：**

| 已有 9 | 新增 9（R1） |
|---|---|
| Button、IconButton、TextField、Select、Switch、Toast、Confirm、EmptyState、ResultCard | SegmentedControl、ShortcutChip、StatusBadge、InlineError、WindowChrome、SidebarItem、HistoryRow、ProviderRow、TranslationCard |

**6 辅助组件：** Banner、Dialog、Tooltip、Spinner、ProviderCard、VisuallyHidden

**概念契约（backlog，不在 R1）：** TextArea、Checkbox、Card、ListRow

**移出 R1：** AppSidebar、ProgressRail

**packages/ui 最终导出 = 9 + 9 + 6 = 24 个。**

---

## Penpot Token 三集合（唯一设计源）

### Core / Primitives — 97 Token

**Penpot 原生。不得自行改色阶。禁止把 `color.core.white` 虚构为 `color.core.neutral.0`。**

| 分类 | Penpot 原名 | 数量 |
|---|---|---|
| colors | `color.core.white` `#FFFFFF`、`color.core.black` `#000000`、`neutral.50–950`(11)、`indigo.50–900`(10：50/100/200/300/**400=`#818CF8`**/**500=`#6366F1`**/**600=`#4F46E5`**/**700=`#4338CA`**/800/900)、`cyan.50–800`(9)、`green.50/500/600/700`(4)、`amber.50/500/600/700`(4)、`red.50/500/600/700`(4) | 44 |
| spacing | `space.0/2/4/6/8/10/12/16/20/24/32/40/48/64` | 14 |
| radius | `radius.0/4/6/8/10/12/16/20/full` | 9 |
| border width | `border.1` 1px、`border.2` 2px | 2 |
| opacity | `opacity.disabled`、`opacity.muted` | 2 |
| font families | `font.family.sans`、`font.family.cjk`、`font.family.mono` | 3 |
| font sizes | `font.size.11/12/13/14/16/18/20/24/32` | 9 |
| font weights | `font.weight.400/500/600/700` | 4 |
| typography | `type.display`(size.32/weight.700/1.25)、`type.title.lg`(size.24/weight.700/1.33)、`type.title.md`(size.20/weight.600/1.4)、`type.title.sm`(size.16/weight.600/1.5)、`type.body.lg`(size.16/weight.400/1.5)、`type.body.md`(size.14/weight.400/1.43)、`type.body.sm`(size.12/weight.400/1.5)、`type.label.md`(size.13/weight.600/1.23)、`type.label.sm`(size.11/weight.600/1.27)、`type.code`(size.12/weight.500/1.5/mono) —— 每个是 size/weight/lineHeight 复合 Token，不拆分为 `*.line-height` 子 Token | 10 |
| **合计** | | **97** |

### Semantic / Light — 28 Token（Penpot 原生，逐项镜像）

```
color.canvas              #F8FAFC
color.surface.default     #FFFFFF
color.surface.subtle      #F1F5F9
color.surface.raised      #FFFFFF
color.surface.inverse     #0F172A
color.text.primary        #0F172A
color.text.secondary      #475569
color.text.tertiary       #64748B
color.text.disabled       #94A3B8
color.text.inverse        #F8FAFC
color.brand.default       #4F46E5
color.brand.hover         #4338CA
color.brand.soft          #EEF2FF
color.accent.default      #06B6D4
color.accent.soft         #ECFEFF
color.border.subtle       #E2E8F0
color.border.default      #CBD5E1
color.border.strong       #94A3B8
color.focus               #0891B2
color.success.default     #16A34A
color.success.soft        #F0FDF4
color.warning.default     #D97706
color.warning.soft        #FFFBEB
color.danger.default      #DC2626
color.danger.soft         #FEF2F2
color.overlay             {color.core.neutral.950}
shadow.raised             0 8 24 -2 #0F172A
shadow.overlay            0 16 40 -4 #0F172A
```
**（脚本计数 = 28 ✅）**

> **shadow/overlay 原生 Token 值（Penpot 生成源）：**
>
> | Penpot 原生名 | Light 原始值 | Dark 原始值 |
> |---|---|---|
> | `color.overlay` | `{color.core.neutral.950}` | `{color.core.black}` |
> | `shadow.raised` | `0 8 24 -2 #0F172A` | `0 8 24 -2 #000000` |
> | `shadow.overlay` | `0 16 40 -4 #0F172A` | `0 16 40 -4 #000000` |
>
> 以上为 Penpot 原生 Token 值（hex shadow color，非 rgba 半透明）。视觉画板上的半透明 drop-shadow 效果（如 `rgba(0,0,0,0.08)`）是 Penpot 渲染层的 opacity 设置，不是 Token 值，不得冒充原生 Token。token-map.md 必须记录这些 hex 原始值。

### Semantic / Dark — 28 Token（Penpot 原生）

```
color.canvas              #020617
color.surface.default     #0F172A
color.surface.subtle      #1E293B
color.surface.raised      #1E293B
color.surface.inverse     #F8FAFC
color.text.primary        #F8FAFC
color.text.secondary      #CBD5E1
color.text.tertiary       #94A3B8
color.text.disabled       #475569
color.text.inverse        #0F172A
color.brand.default       #818CF8
color.brand.hover         #A5B4FC
color.brand.soft          #312E81
color.accent.default      #22D3EE
color.accent.soft         #164E63
color.border.subtle       #1E293B
color.border.default      #334155
color.border.strong       #475569
color.focus               #22D3EE
color.success.default     #22C55E
color.success.soft        #15803D
color.warning.default     #F59E0B
color.warning.soft        #B45309
color.danger.default      #EF4444
color.danger.soft         #B91C1C
color.overlay             {color.core.black}
shadow.raised             0 8 24 -2 #000000
shadow.overlay            0 16 40 -4 #000000
```
**（脚本计数 = 28 ✅）**

> **Dark shadow/overlay 原生 Token 值：** overlay = `{color.core.black}`，shadow.raised = `0 8 24 -2 #000000`，shadow.overlay = `0 16 40 -4 #000000`。与 Light 相比仅 shadow color 从 `#0F172A` 变为 `#000000`，overlay Token 从 `{neutral.950}` 变为 `{black}`。

### 三层名称映射

每个 Token 有三层名称，严格区分：

| Penpot 原名（设计源） | CSS Semantic 名（代码使用） | 旧兼容名（别名，迁移后删） |
|---|---|---|
| `color.success.default` | `--color-status-success` | `--color-success-fg` |
| `color.success.soft` | `--color-status-success-soft` | （无旧名） |
| `color.brand.default` | `--color-brand-default` | `--color-primary-fill` |
| `shadow.raised` | `--shadow-raised` | `--shadow-md`（旧别名 → `--shadow-raised`） |
| `shadow.overlay` | `--shadow-overlay` | `--shadow-lg`（旧别名 → `--shadow-overlay`） |

> **Penpot 原生栏不得改名。** CSS 可把 `color.success.default` 映射为 `--color-status-success`，但 token-map.md 的 Penpot 栏必须写 `color.success.default`。

### 工程扩展 Token（Penpot 中不存在）

| 工程扩展 Token | Light | Dark | 用途 | WCAG |
|---|---|---|---|---|
| `color.surface.hover` | `#F1F5F9` | `#334155` | hover 背景 | — |
| `color.surface.selected` | `#DBEAFE` | `#1E3A5F` | 选中背景 | — |
| `color.text.selected` | `#1D4ED8` | `#60A5FA` | 选中文字 | — |
| `color.brand.on-fill` | `#FFFFFF` | `#0F172A` | 品牌填充文字 | 6.288 / 5.985 ✅ |
| `color.brand.fg` | `#4F46E5` | `#818CF8` | 画布上的品牌前景 | 6.009 / 6.763 ✅ |
| `color.status.info` | `#2563EB` | `#60A5FA` | 信息状态 | — |
| `color.status.info.soft` | `#EFF6FF` | `#1E3A8A` | 信息软背景 | — |
| `color.disabled.bg` | `#F1F5F9` | `#1E293B` | 禁用背景 | — |
| **`color.border.control`** | **`#64748B`** | **`#64748B`** | **控件边界（TextField/Select 等）** | **4.548 / 4.239 ✅ 3:1** |
| `color.strong-fill.success` | `#15803D` | `#15803D` | Banner/Button 强填充 | 5.016 ✅ |
| `color.strong-fill.warning` | `#B45309` | `#B45309` | 同上 | 5.022 ✅ |
| `color.strong-fill.danger` | `#DC2626` | `#DC2626` | 同上 | 4.829 ✅ |
| `color.strong-fill.info` | `#2563EB` | `#2563EB` | 同上 | 5.169 ✅ |
| `color.strong-on-*` | `#FFFFFF` | `#FFFFFF` | 强填充文字 | 见上 |

**`color.border.control` 冻结（rev-4）：** Light=`#64748B`，Dark=`#64748B`。TextField、Select 和需要可识别边界的控件迁移到 `border.control`。Penpot `border.strong` 保留为装饰用途（不满足 3:1，不用于控件边界）。

**Strong fill 两主题统一冻结：** success=`#15803D`、warning=`#B45309`、danger=`#DC2626`、info=`#2563EB`，on-fill=`#FFFFFF`（两主题相同）。

**Penpot 回写门：** 工程扩展 Token 标注 `[工程扩展]`，未回写 Penpot 前不得宣称 1:1 一致。

---

## WCAG 对比度表（rev-4 脚本自动计算值）

### Brand

| 组合 | WCAG | AA |
|---|---|---|
| `#4F46E5` / `#FFFFFF` | 6.288:1 | ✅ |
| `#818CF8` / `#0F172A` | 5.985:1 | ✅ |
| `#818CF8` / `#020617` | 6.763:1 | ✅ |
| `#A5B4FC` / `#0F172A` | 8.955:1 | ✅ |

### Focus（UI 3:1）

| `#0891B2` / `#FFFFFF` | 3.682:1 | ✅ |
| `#22D3EE` / `#0F172A` | 9.879:1 | ✅ |

### Strong fill（on-fill=#FFFFFF，两主题统一）

| 组合 | on-fill ≥4.5 | Light surface ≥3 | Dark surface ≥3 |
|---|---|---|---|
| success `#15803D` | 5.016 ✅ | 5.016 ✅ | 3.559 ✅ |
| warning `#B45309` | 5.022 ✅ | 5.022 ✅ | 3.555 ✅ |
| danger `#DC2626` | 4.829 ✅ | 4.829 ✅ | 3.697 ✅ |
| info `#2563EB` | 5.169 ✅ | 5.169 ✅ | 3.454 ✅ |

### StatusBadge（Light: core.700 on core.50 soft；Dark: text.primary on Dark soft）

| variant | Light (core.700/core.50) | WCAG | Dark (text.primary/soft) | WCAG |
|---|---|---|---|---|
| success | `#15803D`/`#F0FDF4` | 4.791 ✅ | `#F8FAFC`/`#15803D` | 4.794 ✅ |
| warning | `#B45309`/`#FFFBEB` | 4.842 ✅ | `#F8FAFC`/`#B45309` | 4.800 ✅ |
| danger | `#B91C1C`/`#FEF2F2` | 5.915 ✅ | `#F8FAFC`/`#B91C1C` | 6.184 ✅ |
| info | `#1D4ED8`/`#EFF6FF` | 6.158 ✅ | `#F8FAFC`/`#1E3A8A` | 9.900 ✅ |

### border.control（UI 3:1）

| `#64748B` / `#F8FAFC` (Light) | 4.548:1 | ✅ |
| `#64748B` / `#020617` (Dark) | 4.239:1 | ✅ |

---

## Global Constraints

- **Penpot 为唯一设计源：** Core 97 + Semantic Light 28 + Semantic Dark 28 = 153。使用 `neutral`（不得 slate）。`color.core.white`/`black` 独立（不是 neutral.0）。Indigo：400=`#818CF8`、500=`#6366F1`、600=`#4F46E5`、700=`#4338CA`。Penpot 原名：`font.family.sans/cjk/mono`、`font.size.11–32`、`border.1/2`、`opacity.disabled/muted`、`type.display/title/body/label/code`。
- **三层名称：** Penpot 原名 → CSS Semantic 名 → 旧兼容名。Penpot 原生栏不得改名。
- **`color.overlay` 是 Penpot 原生**（不是工程扩展）。`color.success.default/soft`、`color.warning.default/soft`、`color.danger.default/soft` 是 Penpot 原生。
- **Dark brand：** default=`#818CF8`，on-fill=`#0F172A`，fg=`#818CF8`，hover=`#A5B4FC`。
- **border.control（工程扩展）：** `#64748B` 两主题。TextField/Select 用此。border.strong 保留装饰。
- **Strong fill（工程扩展，两主题统一）：** success=`#15803D`、warning=`#B45309`、danger=`#DC2626`、info=`#2563EB`，on-fill=`#FFFFFF`。
- **StatusBadge：** Light 用 core.700 前景 + core.50 soft 背景；Dark 用 `--color-text-primary` + Dark soft 背景。不使用 success/warning 实色配白字。
- **字体（方案 B）：** 5 woff2，2,393,380 bytes (2.28MB)，SHA-256 校验。本地打包，OFL，禁远程。
- **Core 引用权：** 只有 tokens.css 可引用 `--core-*`。
- **别名：** `--old: var(--new);` 单向。禁止循环/复制值/it.skip。
- **WCAG 自动化：** contrast.test.ts 脚本验证，禁手工填值。
- **验证命令：** `pnpm --filter @linguaray/ui exec vitest run <file>`、`pnpm --filter @linguaray/ui-lab exec vitest run <file>`、`pnpm --filter @linguaray/ui-lab build`、`cd src-tauri && cargo test --features xproc-test-helper`、`cd src-tauri && cargo clippy --all-targets --features xproc-test-helper -- -D warnings`。

---

## File Structure

### R0 产出（仅文档）
`MASTER.md`（改写）、`token-map.md`（新建）、`handoff-manifest.md`（新建）、`pages/01–16.md`（16 新建）。

### R1 产出（代码 + 测试）
`tokens.css`（改写）、`fonts.css`（新建）、`index.css`（小改）、9 个新组件 `.tsx/.css/.test.tsx`、`providerPresentation.ts`（新建）、`providerTypes.ts`（新建）、`index.ts`（追加）、`aliases.test.ts`、`contrast.test.ts`、`fonts.test.ts`、`assets/fonts/`（5 woff2 + 3 LICENSE）、`ComponentGallery.tsx/.css`、`playwright.config.ts`、`e2e/component-gallery.visual.spec.ts`。

---

## R0 — 调和并冻结契约（仅文档，独立检查点）

### Task R0-1: Token 映射文档

**Files:** Create: `design-system/linguaray/token-map.md`

- [x] **Step 1: 编写 token-map.md** — 三栏（Penpot 原名 → CSS Semantic 名 → 旧兼容名）。Core 97（colors 44 + spacing 14 + radius 9 + border-width 2 + opacity 2 + font-family 3 + font-size 9 + font-weight 4 + typography 10）+ Semantic Light 28 + Semantic Dark 28 + 工程扩展（标注 `[工程扩展]`）+ 旧→新别名表（含 `--color-success-fill → --color-strong-fill-success`、`--color-warning-fill → --color-strong-fill-warning`、`--color-info-fill → --color-strong-fill-info` 及 on-fill/fg 映射）。
- [x] **Step 2:** Run: `node -e "const t=require('fs').readFileSync('design-system/linguaray/token-map.md','utf8'); console.log('neutral='+t.includes('neutral'),'slate='+t.includes('slate'),'eng='+(t.match(/工程扩展/g)||[]).length);"` → Expected: `neutral=true slate=false eng≥14`（2026-08-08 验证通过：neutral=true slate=false eng=17）

### Task R0-2: 改写 MASTER.md

- [x] **Step 1–4:** §1 三集合（Core 97 / Light 28 / Dark 28）+ 工程扩展 + WCAG 脚本值 + border.control + strong-fill。§2 字体方案 B。§3–§10。TextArea/Checkbox/Card/ListRow 标 backlog。
- [x] **Step 5:** Run: `node -e "const t=require('fs').readFileSync('design-system/linguaray/MASTER.md','utf8'); console.log('wcag='+(t.match(/WCAG/g)||[]).length,'old640='+t.includes('6.40'),'borderControl='+t.includes('border.control'));"` → Expected: `wcag≥5 old640=false borderControl=true`（2026-08-08 验证通过：wcag=10 old640=false borderControl=true）

### Task R0-3: Manifest（34 Node ID 强制填充 + 严格解析验证）

**Files:** Create: `design-system/linguaray/handoff-manifest.md`

- [x] **Step 1:** 编写 manifest：Team ID(1) + File ID(1) + 8 页面 ID + Token 97/28/28。
- [x] **Step 2:** **查询填入 34 个真实 Node ID**（16 Surface + 18 Component）。从 Penpot 文件逐个查询。
- [x] **Step 3:** 严格结构验证。保存为 `scripts/verify-manifest.mjs`，用 `node:assert/strict`，任一断言失败必须非零退出：

```javascript
// scripts/verify-manifest.mjs
import { strict as assert } from "node:assert";
import { readFileSync } from "node:fs";

const t = readFileSync("design-system/linguaray/handoff-manifest.md", "utf-8");
const UUID_RE = /[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}/g;
const STRICT_UUID = /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/;

// 精确解析区段：找不到立即失败，禁止回退全文
function extractSection(label) {
  const re = new RegExp(`## ${label}\\n([\\s\\S]*?)(?=\\n## |$)`);
  const m = t.match(re);
  assert.ok(m, `区段 "${label}" 不存在`);
  return m[1];
}

function extractUUIDs(section) {
  return [...section.matchAll(UUID_RE)].map((m) => m[0]);
}

// 1. Team ID + File ID（从区段按行提取，各精确 1 个，严格 UUID 格式）
const teamSection = extractSection("Penpot File & Team");
const teamLineIDs = [...teamSection.matchAll(/Team ID:\s*([0-9a-f-]{36})/g)].map((m) => m[1]);
const fileLineIDs = [...teamSection.matchAll(/File ID:\s*([0-9a-f-]{36})/g)].map((m) => m[1]);
assert.strictEqual(teamLineIDs.length, 1, `Team ID 行应为 1 个，实际 ${teamLineIDs.length}`);
assert.strictEqual(fileLineIDs.length, 1, `File ID 行应为 1 个，实际 ${fileLineIDs.length}`);
assert.match(teamLineIDs[0], STRICT_UUID, "Team ID 格式不合法");
assert.match(fileLineIDs[0], STRICT_UUID, "File ID 格式不合法");
assert.notStrictEqual(fileLineIDs[0], teamLineIDs[0], "File ID 不得等于 Team ID");

// 3. Penpot 页面: 恰好 8，唯一
const pageSection = extractSection("Penpot 页面");
const pageUUIDs = extractUUIDs(pageSection);
assert.strictEqual(pageUUIDs.length, 8, `页面 UUID 应为 8，实际 ${pageUUIDs.length}`);
assert.strictEqual(new Set(pageUUIDs).size, 8, "页面 UUID 有重复");
pageUUIDs.forEach((id) => assert.match(id, STRICT_UUID, `页面 UUID 格式: ${id}`));

// 4. 16 Surface: 恰好 16，唯一
const surfSection = extractSection("16 Surface");
const surfUUIDs = extractUUIDs(surfSection);
assert.strictEqual(surfUUIDs.length, 16, `Surface UUID 应为 16，实际 ${surfUUIDs.length}`);
assert.strictEqual(new Set(surfUUIDs).size, 16, "Surface UUID 有重复");

// 5. 18 Component: 恰好 18，唯一
const compSection = extractSection("18 Component");
const compUUIDs = extractUUIDs(compSection);
assert.strictEqual(compUUIDs.length, 18, `Component UUID 应为 18，实际 ${compUUIDs.length}`);
assert.strictEqual(new Set(compUUIDs).size, 18, "Component UUID 有重复");

// 6. 34 Surface+Component 全局唯一
const all34 = [...surfUUIDs, ...compUUIDs];
assert.strictEqual(all34.length, 34);
assert.strictEqual(new Set(all34).size, 34, "34 Node ID 有全局重复");

// 7. Team + File + Page + Surface + Component 全部 ID 全局唯一
const allIDs = [teamLineIDs[0], fileLineIDs[0], ...pageUUIDs, ...all34];
assert.strictEqual(new Set(allIDs).size, allIDs.length, "全局 ID 有重复");

// 8. 0 TBD
assert.strictEqual((t.match(/TBD-S|TBD-C/gi) || []).length, 0, "存在 TBD 占位符");

// 9. Token 数量精确（解析 Core/Light/Dark 字段）
const coreSection = extractSection("Token 集合");
assert.ok(coreSection.includes("Core: 97"), "Core 数量必须为 97");
assert.ok(coreSection.includes("Light: 28"), "Light 数量必须为 28");
assert.ok(coreSection.includes("Dark: 28"), "Dark 数量必须为 28");
assert.doesNotMatch(coreSection, /~\d+|约\d+/, "Token 数量不得用近似值");

console.log("R0-3 manifest: 结构验证通过");
```

- [x] **Step 4:** Run: `node scripts/verify-manifest.mjs`。**任一 assert 失败 → 进程非零退出 → R0 检查点失败 → 禁止进入 R1。不会打印"通过"如果前面有失败。**（2026-08-08：Page 8 / Surface 16 / Component 18 / Node 34，验证通过。）

### Task R0-4: 16 Surface 页面文档

- [x] **Step 1:** 16 个文档。每个含 Penpot 画板尺寸 + 生产窗口尺寸 + 冲突说明 + 状态矩阵 + copy key。Onboarding = 6 状态。
- [x] **Step 2:** ⚠️ R0 检查点（2026-08-08：R0-1/R0-2/R0-3/R0-4 全部验证通过，R0 关闭，进入 R1）。

---

## R1 — 双层 Token、字体、9 组件、画廊、视觉基线

### Task R1-1: tokens.css + aliases.test.ts + contrast.test.ts

**Files:** `tokens.css`（改写）、`aliases.test.ts`（新建）、`contrast.test.ts`（新建）

- [x] **Step 1: aliases.test.ts（RED）** — 纯 CSS 文本解析，selector 传原始字符串不双重转义：

```typescript
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

  it("no --core-* outside tokens.css (recursive scan packages/ui + apps/ui-lab)", () => {
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
```

- [x] **Step 2: contrast.test.ts（RED）** — 读取 tokens.css，解析 Light/Dark 实际值，递归 var() 后计算对比度。**禁止在断言中手写 hex：**

```typescript
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
  // (R1-1 修订：Penpot success/warning/danger.default 值在 soft 背景上不达 AA
  // —— danger #DC2626 on soft #FEF2F2 = 4.41 ❌ —— 故新增 [工程扩展]
  // --color-status-{success,warning,danger,info}-fg 供 StatusBadge 软模式消费。
  // 工程扩展由 17 → 21；冻结的 strong-fill-*/status-*-soft 值不变。)
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

  // StatusBadge Dark: same --color-status-*-fg tokens on Dark soft backgrounds ≥ 4.5
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
```

- [x] **Step 3:** Run: `pnpm --filter @linguaray/ui exec vitest run test/aliases.test.ts test/contrast.test.ts` → FAIL
- [x] **Step 4:** 重写 tokens.css（Core 97 + Semantic 28+28 + 工程扩展 + 别名）。Core 用 `--core-color-white`/`--core-color-black`（独立）。Semantic 用 `--color-status-success`(映射 `color.success.default`)、`--shadow-raised`/`--shadow-overlay`（正式 Semantic 名，直接定义值；`--color-shadow-*` 为兼容别名）。工程扩展含 `--color-border-control`、`--color-strong-fill-*`。别名含 `--color-success-fill` → `var(--color-strong-fill-success)` 等。

  **R1-1 修订（执行时新增）：** 新增 4 个 `[工程扩展]` Semantic Token `--color-status-{success,warning,danger,info}-fg`（Light `#15803D/#B45309/#B91C1C/#1D4ED8`，Dark 统一 `#F8FAFC`），供 StatusBadge 软背景前景使用——Penpot 原生 success/warning/danger.default 值在对应 soft 背景上不达 AA 4.5（danger #DC2626 on soft #FEF2F2 = 4.41 ❌），info.* 无 Penpot 源。冻结值（`--color-strong-fill-*`、`--color-status-*-soft`）保持不变；工程扩展由 17 → 21。已同步 token-map.md / MASTER.md §1.3 + §1.4 / handoff-manifest.md。StatusBadge.css（R1-5）只能消费 `--color-status-*-fg`（前景）与 `--color-status-*-soft`（背景），不得使用 hex / `--core-*` / 旧别名。
- [x] **Step 5:** Run: 同命令 → PASS
- [x] **Step 6:** 回归 `pnpm --filter @linguaray/ui exec vitest run && pnpm --filter @linguaray/ui-lab exec vitest run` → 全绿
- [x] **Step 7:** `pnpm typecheck && pnpm build && pnpm --filter @linguaray/ui-lab build` → 全绿
- [N/A — 未授权 commit] **Step 8:** Commit

---

### Task R1-2: 本地字体打包（方案 B）

**精确制品（实测，SHA-256 校验）：**

| 文件 | 大小 | SHA-256 |
|---|---|---|
| `inter-latin-wght-normal.woff2` | 48,256 | `3100e775e8616cd2611beecfa23a4263d7037586789b43f035236a2e6fbd4c62` |
| `ibm-plex-mono-latin-400-normal.woff2` | 14,708 | `08949f728dc52d528e69b1667d15c89a5686a4ee9a296ff90983985f99c380f7` |
| `ibm-plex-mono-latin-600-normal.woff2` | 15,620 | `0d1f0b8d0722224e32e9f28261bdc86c79115be73444ae5eceb73976a1bcdf83` |
| `noto-sans-sc-chinese-simplified-400-normal.woff2` | 1,142,552 | `95e3633b6a98f764ba3adfb54504a0cd4799328c009adf9081d6c1850f9c4c78` |
| `noto-sans-sc-chinese-simplified-700-normal.woff2` | 1,172,244 | `e1df51edc00bce27b58044e829fb8ec6accc8a5daece475413de90d52818845c` |
| **总计** | **2,393,380 (2.28 MB)** | |

- [x] **Step 1: fonts.test.ts（RED）** — 完整独立代码：

```typescript
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
```

- [x] **Step 2:** Run: `pnpm --filter @linguaray/ui exec vitest run test/fonts.test.ts` → FAIL
- [x] **Step 3:** 安装 devDeps + 复制制品 + 创建 fonts.css + 修改 index.css（按精确文件名 + SHA-256 校验）
- [x] **Step 4:** Run: 同命令 → PASS（7 测试）
- [x] **Step 5:** 回归: `pnpm typecheck && pnpm --filter @linguaray/ui exec vitest run && pnpm build && pnpm --filter @linguaray/ui-lab build` → 全绿
- [N/A — 未授权 commit] **Step 6:** Commit。R1 只验证源码许可证；发行包列入 R7。

---

## R1-3 ~ R1-11：9 个新增组件

> 每个组件：完整可编译测试 → RED → 实现 → GREEN → 回归 → commit。
> 测试从 `../../test/setup` 导入。组件 CSS 只用 Semantic Token，禁止 `--core-*`。

### Task R1-3: SegmentedControl

**Files:** `packages/ui/src/components/SegmentedControl.{tsx,css,test.tsx}`

```typescript
// SegmentedControl.tsx
import { For, createMemo, type Component, type JSX, splitProps } from "solid-js";
import "./SegmentedControl.css";

export type SegmentedOption = { value: string; label: string; icon?: JSX.Element };

export type SegmentedControlProps = {
  options: SegmentedOption[];
  value: string;
  onChange: (value: string) => void;
  ariaLabel: string; // 必填
  disabled?: boolean;
};

export const SegmentedControl: Component<SegmentedControlProps> = (props) => {
  const [, rest] = splitProps(props, ["options", "value", "onChange", "ariaLabel", "disabled"]);
  const currentIndex = createMemo(() =>
    Math.max(0, props.options.findIndex((o) => o.value === props.value)),
  );
  // 局部 ref 数组，存储 tab DOM 节点（禁止 document.querySelector 全局查询）
  let tabRefs: (HTMLButtonElement | undefined)[] = [];

  function activate(index: number) {
    if (props.disabled) return;
    const len = props.options.length;
    const wrapped = ((index % len) + len) % len;
    props.onChange(props.options[wrapped].value);
    // 用局部 ref 移动焦点
    tabRefs[wrapped]?.focus();
  }

  function onKeyDown(e: KeyboardEvent) {
    if (props.disabled) return;
    const i = currentIndex();
    switch (e.key) {
      case "ArrowRight":
      case "ArrowDown":
        e.preventDefault();
        activate(i + 1);
        break;
      case "ArrowLeft":
      case "ArrowUp":
        e.preventDefault();
        activate(i - 1);
        break;
      case "Home":
        e.preventDefault();
        activate(0);
        break;
      case "End":
        e.preventDefault();
        activate(props.options.length - 1);
        break;
    }
  }

  return (
    <div class="seg-control" role="tablist" aria-label={props.ariaLabel} {...rest}>
      <For each={props.options}>
        {(opt, index) => (
          <button
            type="button"
            role="tab"
            class="seg-control__tab"
            aria-selected={opt.value === props.value}
            tabindex={opt.value === props.value ? 0 : -1}
            disabled={props.disabled}
            ref={(el) => (tabRefs[index()] = el)}
            onClick={() => activate(index())}
            onKeyDown={onKeyDown}
          >
            {opt.icon}
            <span>{opt.label}</span>
          </button>
        )}
      </For>
    </div>
  );
};
export default SegmentedControl;
```

```typescript
// SegmentedControl.test.tsx
import { describe, it, expect, vi } from "vitest";
import { render, fireEvent } from "@solidjs/testing-library";
import SegmentedControl from "./SegmentedControl";
import { assertNoAxeViolations } from "../../test/setup";

const opts = [
  { value: "a", label: "Alpha" },
  { value: "b", label: "Beta" },
  { value: "c", label: "Gamma" },
];

describe("SegmentedControl", () => {
  it("renders all options", () => {
    const { getByText } = render(() => <SegmentedControl options={opts} value="a" onChange={() => {}} ariaLabel="View" />);
    expect(getByText("Alpha")).toBeInTheDocument();
    expect(getByText("Gamma")).toBeInTheDocument();
  });

  it("aria-selected on correct tab", () => {
    const { getByText } = render(() => <SegmentedControl options={opts} value="b" onChange={() => {}} ariaLabel="View" />);
    expect(getByText("Beta").closest("[role='tab']")).toHaveAttribute("aria-selected", "true");
    expect(getByText("Alpha").closest("[role='tab']")).toHaveAttribute("aria-selected", "false");
  });

  it("click calls onChange", () => {
    const onChange = vi.fn();
    const { getByText } = render(() => <SegmentedControl options={opts} value="a" onChange={onChange} ariaLabel="View" />);
    fireEvent.click(getByText("Beta"));
    expect(onChange).toHaveBeenCalledWith("b");
  });

  it("disabled prevents onChange", () => {
    const onChange = vi.fn();
    const { getByText } = render(() => <SegmentedControl options={opts} value="a" onChange={onChange} ariaLabel="V" disabled />);
    fireEvent.click(getByText("Beta"));
    expect(onChange).not.toHaveBeenCalled();
  });

  it("ArrowRight activates next and moves DOM focus", () => {
    const onChange = vi.fn();
    const { getByText } = render(() => <SegmentedControl options={opts} value="a" onChange={onChange} ariaLabel="ViewMode" />);
    const tab = getByText("Alpha").closest("[role='tab']") as HTMLElement;
    tab.focus();
    expect(document.activeElement).toBe(tab);
    fireEvent.keyDown(tab, { key: "ArrowRight" });
    expect(onChange).toHaveBeenCalledWith("b");
    // 焦点应移到 Beta tab
    const betaTab = getByText("Beta").closest("[role='tab']") as HTMLElement;
    expect(document.activeElement).toBe(betaTab);
  });

  it("ArrowDown activates next and moves DOM focus", () => {
    const onChange = vi.fn();
    const { getByText } = render(() => <SegmentedControl options={opts} value="a" onChange={onChange} ariaLabel="ViewMode2" />);
    const tab = getByText("Alpha").closest("[role='tab']") as HTMLElement;
    tab.focus();
    fireEvent.keyDown(tab, { key: "ArrowDown" });
    expect(onChange).toHaveBeenCalledWith("b");
    expect(document.activeElement).toBe(getByText("Beta").closest("[role='tab']"));
  });

  it("ArrowLeft wraps to last and moves DOM focus", () => {
    const onChange = vi.fn();
    const { getByText } = render(() => <SegmentedControl options={opts} value="a" onChange={onChange} ariaLabel="ViewMode3" />);
    const tab = getByText("Alpha").closest("[role='tab']") as HTMLElement;
    tab.focus();
    fireEvent.keyDown(tab, { key: "ArrowLeft" });
    expect(onChange).toHaveBeenCalledWith("c");
    expect(document.activeElement).toBe(getByText("Gamma").closest("[role='tab']"));
  });

  it("ArrowUp wraps to last and moves DOM focus", () => {
    const onChange = vi.fn();
    const { getByText } = render(() => <SegmentedControl options={opts} value="a" onChange={onChange} ariaLabel="ViewMode4" />);
    const tab = getByText("Alpha").closest("[role='tab']") as HTMLElement;
    tab.focus();
    fireEvent.keyDown(tab, { key: "ArrowUp" });
    expect(onChange).toHaveBeenCalledWith("c");
    expect(document.activeElement).toBe(getByText("Gamma").closest("[role='tab']"));
  });

  it("Home/End", () => {
    const onChange = vi.fn();
    const { getByText } = render(() => <SegmentedControl options={opts} value="b" onChange={onChange} ariaLabel="V" />);
    const tab = getByText("Beta").closest("[role='tab']")!;
    fireEvent.keyDown(tab, { key: "Home" });
    expect(onChange).toHaveBeenLastCalledWith("a");
    fireEvent.keyDown(tab, { key: "End" });
    expect(onChange).toHaveBeenLastCalledWith("c");
  });

  it("roving tabindex", () => {
    const { getByText } = render(() => <SegmentedControl options={opts} value="b" onChange={() => {}} ariaLabel="V" />);
    expect(getByText("Beta").closest("[role='tab']")).toHaveAttribute("tabindex", "0");
    expect(getByText("Alpha").closest("[role='tab']")).toHaveAttribute("tabindex", "-1");
  });

  it("role=tablist aria-label", () => {
    const { getByRole } = render(() => <SegmentedControl options={opts} value="a" onChange={() => {}} ariaLabel="Mode" />);
    expect(getByRole("tablist")).toHaveAttribute("aria-label", "Mode");
  });

  it("no axe violations", async () => {
    render(() => <SegmentedControl options={opts} value="a" onChange={() => {}} ariaLabel="V" />);
    await assertNoAxeViolations({ disableRules: ["region"] });
  });
});
```

- [x] **Step 1:** 写 test → Run: `pnpm --filter @linguaray/ui exec vitest run src/components/SegmentedControl.test.tsx` → FAIL
- [x] **Step 2:** 实现 .tsx + .css → Run: 同命令 → PASS (12 测试)
- [x] **Step 3:** `pnpm typecheck && pnpm --filter @linguaray/ui exec vitest run` → 全绿
- [N/A — 未授权 commit] **Step 4:** Commit

---

### Task R1-4: ShortcutChip

**Files:** `packages/ui/src/components/ShortcutChip.{tsx,css,test.tsx}`

```typescript
// ShortcutChip.tsx
import { Show, type Component } from "solid-js";
import { X } from "lucide-solid";
import "./ShortcutChip.css";

export type ShortcutChipLabels = {
  recording: string;   // 录制中提示，如 "Recording…"
  conflict: string;    // 冲突提示，如 "Conflict"
  clear: string;       // 清除按钮 aria-label，如 "Clear shortcut"
};
export type ShortcutChipStatus = "recording" | "conflict" | "clear";
export type ShortcutChipProps = {
  shortcut: string;
  status: ShortcutChipStatus;
  labels: ShortcutChipLabels;
  onClear?: () => void;
  disabled?: boolean;
};

const ShortcutChip: Component<ShortcutChipProps> = (props) => {
  return (
    <span
      class="shortcut-chip"
      classList={{
        "shortcut-chip--recording": props.status === "recording",
        "shortcut-chip--conflict": props.status === "conflict",
        "shortcut-chip--disabled": props.disabled,
      }}
      role="status"
      aria-live="polite"
    >
      <kbd class="shortcut-chip__keys">
        {props.status === "recording" ? props.labels.recording : props.shortcut}
      </kbd>
      <Show when={props.status === "conflict"}>
        <span class="shortcut-chip__conflict-text">{props.labels.conflict}</span>
      </Show>
      <Show when={props.onClear && !props.disabled}>
        <button
          type="button"
          class="shortcut-chip__clear"
          aria-label={props.labels.clear}
          onClick={() => props.onClear?.()}
        >
          <X size={14} />
        </button>
      </Show>
    </span>
  );
};
export default ShortcutChip;
```

```typescript
// ShortcutChip.test.tsx
import { describe, it, expect, vi } from "vitest";
import { render, fireEvent } from "@solidjs/testing-library";
import ShortcutChip from "./ShortcutChip";
import { assertNoAxeViolations } from "../../test/setup";

describe("ShortcutChip", () => {
  const labels = { recording: "Recording…", conflict: "Conflict", clear: "Clear shortcut" };

  it("renders shortcut text", () => {
    const { getByText } = render(() => <ShortcutChip shortcut="Ctrl+Shift+T" labels={labels} />);
    expect(getByText("Ctrl+Shift+T")).toBeInTheDocument();
  });

  it("recording shows recording label", () => {
    const { getByText } = render(() => <ShortcutChip shortcut="" recording labels={labels} />);
    expect(getByText("Recording…")).toBeInTheDocument();
  });

  it("conflict shows conflict label", () => {
    const { getByText } = render(() => <ShortcutChip shortcut="Ctrl+X" conflict labels={labels} />);
    expect(getByText("Conflict")).toBeInTheDocument();
  });

  it("onClear fires when clear button clicked", () => {
    const onClear = vi.fn();
    const { getByLabelText } = render(() => <ShortcutChip shortcut="Ctrl+X" onClear={onClear} labels={labels} />);
    fireEvent.click(getByLabelText("Clear shortcut"));
    expect(onClear).toHaveBeenCalledOnce();
  });

  it("disabled hides clear button", () => {
    const onClear = vi.fn();
    const { queryByLabelText } = render(() => <ShortcutChip shortcut="Ctrl+X" onClear={onClear} disabled labels={labels} />);
    expect(queryByLabelText("Clear shortcut")).toBeNull();
  });

  it("no axe violations", async () => {
    render(() => <ShortcutChip shortcut="Ctrl+T" labels={labels} />);
    await assertNoAxeViolations();
  });
});
```

- [x] **Step 1–3:** RED → 实现（CSS 用 `--radius-full` 非 `--core-radius-full`）→ GREEN
- [N/A — 未授权 commit] **Step 4:** commit

**Files:** `packages/ui/src/components/StatusBadge.{tsx,css,test.tsx}`

**状态角色：** Light 用 core.700 前景 + core.50 soft 背景；Dark 用 `--color-text-primary` + Dark soft 背景。

```typescript
// StatusBadge.tsx
import { Show, type Component, type JSX } from "solid-js";
import "./StatusBadge.css";

export type StatusBadgeVariant = "success" | "warning" | "danger" | "info" | "neutral";
export type StatusBadgeProps = {
  variant: StatusBadgeVariant;
  children: JSX.Element;
  icon?: JSX.Element;
  dot?: boolean;
};

const StatusBadge: Component<StatusBadgeProps> = (props) => {
  return (
    <span
      class={`status-badge status-badge--${props.variant}`}
      role="img"
      aria-label={typeof props.children === "string" ? props.children : undefined}
    >
      <Show when={props.dot}>
        <span class="status-badge__dot" aria-hidden="true" />
      </Show>
      <Show when={props.icon}>
        <span class="status-badge__icon" aria-hidden="true">{props.icon}</span>
      </Show>
      <span class="status-badge__label">{props.children}</span>
    </span>
  );
};
export default StatusBadge;
```

```typescript
// StatusBadge.test.tsx
import { describe, it, expect } from "vitest";
import { render } from "@solidjs/testing-library";
import StatusBadge from "./StatusBadge";
import { assertNoAxeViolations } from "../../test/setup";

const variants = ["success", "warning", "danger", "info", "neutral"] as const;

describe("StatusBadge", () => {
  it.each(variants)("renders %s variant with label", (variant) => {
    const { getByText } = render(() => <StatusBadge variant={variant} label="Test" />);
    expect(getByText("Test")).toBeInTheDocument();
  });

  it("dot mode renders dot element", () => {
    const { container } = render(() => <StatusBadge variant="success" label="OK" dot />);
    expect(container.querySelector(".status-badge__dot")).not.toBeNull();
  });

  it("icon renders when provided", () => {
    const { container } = render(() => <StatusBadge variant="info" label="Info" icon={<span data-testid="ic" />} />);
    expect(container.querySelector(".status-badge__icon")).not.toBeNull();
  });

  it.each(variants)("no axe violations for %s", async (variant) => {
    render(() => <StatusBadge variant={variant} label={`${variant} badge`} />);
    await assertNoAxeViolations();
  });
});
```

- [x] **Step 1–3:** RED → 实现（CSS：Light success→`#15803D`/`#F0FDF4`、warning→`#B45309`/`#FFFBEB`、danger→`#B91C1C`/`#FEF2F2`、info→`#1D4ED8`/`#EFF6FF`；Dark 用 `--color-text-primary`/soft）→ GREEN
- [N/A — 未授权 commit] **Step 4:** commit

---

### Task R1-6: InlineError

**Files:** `packages/ui/src/components/InlineError.{tsx,css,test.tsx}`

```typescript
// InlineError.tsx
import { type Component, type JSX } from "solid-js";
import { AlertTriangle } from "lucide-solid";
import "./InlineError.css";

export type InlineErrorProps = {
  children: JSX.Element;
  id?: string;
  icon?: JSX.Element;
};

const InlineError: Component<InlineErrorProps> = (props) => {
  return (
    <p class="inline-error" role="alert" id={props.id}>
      <span class="inline-error__icon" aria-hidden="true">
        {props.icon ?? <AlertTriangle size={14} />}
      </span>
      <span class="inline-error__text">{props.children}</span>
    </p>
  );
};
export default InlineError;
```

```typescript
// InlineError.test.tsx
import { describe, it, expect } from "vitest";
import { render } from "@solidjs/testing-library";
import InlineError from "./InlineError";
import { assertNoAxeViolations } from "../../test/setup";

describe("InlineError", () => {
  it("renders message", () => {
    const { getByText } = render(() => <InlineError message="Something went wrong" />);
    expect(getByText("Something went wrong")).toBeInTheDocument();
  });

  it("has role=alert", () => {
    const { container } = render(() => <InlineError message="Error" />);
    expect(container.querySelector("[role='alert']")).not.toBeNull();
  });

  it("id is applied", () => {
    const { container } = render(() => <InlineError message="Err" id="field-err" />);
    expect(container.querySelector("#field-err")).not.toBeNull();
  });

  it("custom icon", () => {
    const { container } = render(() => <InlineError message="Err" icon={<span data-testid="ci" />} />);
    expect(container.querySelector(".inline-error__icon")).not.toBeNull();
  });

  it("no axe violations", async () => {
    render(() => <InlineError message="Error occurred" />);
    await assertNoAxeViolations();
  });
});
```

- [x] **Step 1–3:** RED → 实现 → GREEN
- [N/A — 未授权 commit] **Step 4:** commit

**Files:** `packages/ui/src/components/WindowChrome.{tsx,css,test.tsx}`

```typescript
// WindowChrome.tsx
import { Show, type Component, type JSX } from "solid-js";
import { Minus, X } from "lucide-solid";
import IconButton from "./IconButton";
import "./WindowChrome.css";

export type WindowChromeLabels = {
  minimize: string; // 如 "Minimize" / "最小化"
  close: string;    // 如 "Close" / "关闭"
};
export type WindowChromeProps = {
  title?: string;
  labels: WindowChromeLabels;
  children: JSX.Element;
  sidebar?: JSX.Element;
  onClose?: () => void;
  onMinimize?: () => void;
};

const WindowChrome: Component<WindowChromeProps> = (props) => {
  return (
    <div class="window-chrome">
      <Show when={props.sidebar}>
        <aside class="window-chrome__sidebar">{props.sidebar}</aside>
      </Show>
      <div class="window-chrome__main">
        <Show when={props.title || props.onClose || props.onMinimize}>
          <header class="window-chrome__header" data-tauri-drag-region>
            <Show when={props.title}>
              <h1 class="window-chrome__title">{props.title}</h1>
            </Show>
            <div class="window-chrome__controls">
              <Show when={props.onMinimize}>
                <IconButton
                  variant="ghost"
                  aria-label={props.labels.minimize}
                  data-tauri-drag-region="false"
                  onClick={props.onMinimize}
                >
                  <Minus aria-hidden="true" size={14} />
                </IconButton>
              </Show>
              <Show when={props.onClose}>
                <IconButton
                  variant="ghost"
                  aria-label={props.labels.close}
                  data-tauri-drag-region="false"
                  onClick={props.onClose}
                >
                  <X aria-hidden="true" size={14} />
                </IconButton>
              </Show>
            </div>
          </header>
        </Show>
        <main class="window-chrome__content">{props.children}</main>
      </div>
    </div>
  );
};
export default WindowChrome;
```

```typescript
// WindowChrome.test.tsx
import { describe, it, expect, vi } from "vitest";
import { render, fireEvent } from "@solidjs/testing-library";
import WindowChrome from "./WindowChrome";
import { assertNoAxeViolations } from "../../test/setup";

describe("WindowChrome", () => {
  const labels = { minimize: "Minimize", close: "Close" };
  it("renders children", () => {
    const { getByText } = render(() => <WindowChrome labels={labels}>Hello</WindowChrome>);
    expect(getByText("Hello")).toBeInTheDocument();
  });

  it("renders title when provided", () => {
    const { getByText } = render(() => <WindowChrome labels={labels} title="Settings">Body</WindowChrome>);
    expect(getByText("Settings")).toBeInTheDocument();
  });

  it("renders sidebar when provided", () => {
    const { getByText } = render(() => <WindowChrome labels={labels} sidebar={<nav>Sidebar</nav>}>Body</WindowChrome>);
    expect(getByText("Sidebar")).toBeInTheDocument();
  });

  it("onClose fires", () => {
    const onClose = vi.fn();
    const { getByLabelText } = render(() => <WindowChrome labels={labels} onClose={onClose}>Body</WindowChrome>);
    fireEvent.click(getByLabelText("Close"));
    expect(onClose).toHaveBeenCalledOnce();
  });

  it("onMinimize fires", () => {
    const onMinimize = vi.fn();
    const { getByLabelText } = render(() => <WindowChrome labels={labels} onMinimize={onMinimize}>Body</WindowChrome>);
    fireEvent.click(getByLabelText("Minimize"));
    expect(onMinimize).toHaveBeenCalledOnce();
  });

  it("no axe violations", async () => {
    render(() => <WindowChrome labels={labels} title="Test"><p>Content</p></WindowChrome>);
    await assertNoAxeViolations();
  });
});
```

- [x] **Step 1–3:** RED → 实现 → GREEN
- [N/A — 未授权 commit] **Step 4:** commit

**Files:** `packages/ui/src/components/SidebarItem.{tsx,css,test.tsx}`

```typescript
// SidebarItem.tsx
import { Show, type Component, type JSX } from "solid-js";
import "./SidebarItem.css";

export type SidebarItemProps = {
  label: string;
  icon: JSX.Element;
  active?: boolean;
  badge?: string;
  onClick?: () => void;
};

const SidebarItem: Component<SidebarItemProps> = (props) => {
  return (
    <button
      type="button"
      class="sidebar-item"
      classList={{ "sidebar-item--active": !!props.active }}
      aria-current={props.active ? "page" : undefined}
      onClick={() => props.onClick?.()}
    >
      <span class="sidebar-item__icon" aria-hidden="true">{props.icon}</span>
      <span class="sidebar-item__label">{props.label}</span>
      <Show when={props.badge}>
        <span class="sidebar-item__badge">{props.badge}</span>
      </Show>
    </button>
  );
};
// 注：原生 <button type="button"> 天然支持 Enter/Space 触发 click，无需人工 onKeyDown。
// tabindex 默认为 0，无需显式设置。
export default SidebarItem;
```

```typescript
// SidebarItem.test.tsx
import { describe, it, expect, vi } from "vitest";
import { render, fireEvent } from "@solidjs/testing-library";
import SidebarItem from "./SidebarItem";
import { assertNoAxeViolations } from "../../test/setup";

describe("SidebarItem", () => {
  it("renders label and icon", () => {
    const { getByText } = render(() => <SidebarItem label="Home" icon={<span data-testid="ic" />} />);
    expect(getByText("Home")).toBeInTheDocument();
  });

  it("active sets aria-current=page", () => {
    const { container } = render(() => <SidebarItem label="Home" icon={<i />} active />);
    expect(container.querySelector("[aria-current='page']")).not.toBeNull();
  });

  it("badge renders", () => {
    const { getByText } = render(() => <SidebarItem label="Home" icon={<i />} badge="3" />);
    expect(getByText("3")).toBeInTheDocument();
  });

  it("onClick fires on click", () => {
    const onClick = vi.fn();
    const { getByRole } = render(() => <SidebarItem label="Home" icon={<i />} onClick={onClick} />);
    fireEvent.click(getByRole("button"));
    expect(onClick).toHaveBeenCalledOnce();
  });

  // Enter/Space 键盘激活由原生 <button> 自动支持，不在 vitest 中人工模拟。
  // 真实键盘交互通过 Playwright e2e 验证（见 sidebar-keyboard.visual.spec.ts）。

  it("no axe violations", async () => {
    render(() => <SidebarItem label="Settings" icon={<i />} />);
    await assertNoAxeViolations();
  });
});
```

- [x] **Step 1–3:** RED → 实现 → GREEN
- [N/A — 未授权 commit] **Step 4:** commit

---

### Task R1-9: HistoryRow

**Files:** `packages/ui/src/components/HistoryRow.{tsx,css,test.tsx}`

**约束：禁止交互控件嵌套。** row 本身非 button；内部有独立 button 元素互为兄弟。

```typescript
// HistoryRow.tsx
import { Show, type Component } from "solid-js";
import { Star } from "lucide-solid";
import "./HistoryRow.css";

export type HistoryRowLabels = {
  addFavorite: string;    // 如 "Add to favorites" / "添加到收藏"
  removeFavorite: string; // 如 "Remove from favorites" / "从收藏移除"
};
export type HistoryRowProps = {
  sourceText: string;
  resultPreview: string;
  timestamp: string;
  engineLabel: string;
  labels?: HistoryRowLabels;
  favorite?: boolean;
  onToggleFavorite?: () => void;
  onClick?: () => void;
};

const HistoryRow: Component<HistoryRowProps> = (props) => {
  const inner = (
    <>
      <span class="history-row__source">{props.sourceText}</span>
      <span class="history-row__preview">{props.resultPreview}</span>
      <span class="history-row__meta">
        <span class="history-row__engine">{props.engineLabel}</span>
        <span class="history-row__time">{props.timestamp}</span>
      </span>
    </>
  );
  // 无 onClick 时渲染非交互 div；有 onClick 时渲染 button（原生 Enter/Space 自动支持）
  return (
    <div class="history-row">
      <Show when={props.onClick} fallback={
        <div class="history-row__content">{inner}</div>
      }>
        <button type="button" class="history-row__content" onClick={() => props.onClick?.()}>
          {inner}
        </button>
      </Show>
      <Show when={props.onToggleFavorite}>
        <button
          type="button"
          class="history-row__fav"
          aria-label={props.favorite ? props.labels?.removeFavorite : props.labels?.addFavorite}
          aria-pressed={props.favorite}
          onClick={() => props.onToggleFavorite?.()}
        >
          <Star size={16} fill={props.favorite ? "currentColor" : "none"} />
        </button>
      </Show>
    </div>
  );
};
export default HistoryRow;
```

```typescript
// HistoryRow.test.tsx
import { describe, it, expect, vi } from "vitest";
import { render, fireEvent } from "@solidjs/testing-library";
import HistoryRow from "./HistoryRow";
import { assertNoAxeViolations } from "../../test/setup";

const baseProps = {
  sourceText: "Hello",
  resultPreview: "你好",
  timestamp: "2026-01-01 12:00",
  engineLabel: "Google",
  labels: { addFavorite: "Add to favorites", removeFavorite: "Remove from favorites" },
};

describe("HistoryRow", () => {
  it("renders texts", () => {
    const { getByText } = render(() => <HistoryRow {...baseProps} />);
    expect(getByText("Hello")).toBeInTheDocument();
    expect(getByText("你好")).toBeInTheDocument();
    expect(getByText("Google")).toBeInTheDocument();
    expect(getByText("2026-01-01 12:00")).toBeInTheDocument();
  });

  it("onClick fires", () => {
    const onClick = vi.fn();
    const { getByText } = render(() => <HistoryRow {...baseProps} onClick={onClick} />);
    fireEvent.click(getByText("Hello"));
    expect(onClick).toHaveBeenCalledOnce();
  });

  it("onToggleFavorite fires", () => {
    const onToggle = vi.fn();
    const { getByLabelText } = render(() => <HistoryRow {...baseProps} onToggleFavorite={onToggle} />);
    fireEvent.click(getByLabelText("Add to favorites"));
    expect(onToggle).toHaveBeenCalledOnce();
  });

  it("favorite button is NOT nested inside onClick button (DOM structure)", () => {
    const { container } = render(() => (
      <HistoryRow {...baseProps} onClick={() => {}} onToggleFavorite={() => {}} />
    ));
    const buttons = container.querySelectorAll("button");
    expect(buttons.length).toBe(2);
    expect(buttons[0].contains(buttons[1])).toBe(false);
    expect(buttons[1].contains(buttons[0])).toBe(false);
  });

  it("no onClick renders non-interactive div (not button)", () => {
    const { container } = render(() => <HistoryRow {...baseProps} />);
    expect(container.querySelector("button.history-row__content")).toBeNull();
    expect(container.querySelector("div.history-row__content")).not.toBeNull();
  });

  it("with onClick renders button", () => {
    const { container } = render(() => <HistoryRow {...baseProps} onClick={() => {}} />);
    expect(container.querySelector("button.history-row__content")).not.toBeNull();
  });

  it("no axe violations", async () => {
    render(() => <HistoryRow {...baseProps} onToggleFavorite={() => {}} />);
    await assertNoAxeViolations();
  });
});
```

- [x] **Step 1–3:** RED → 实现 → GREEN
- [N/A — 未授权 commit] **Step 4:** commit

---

### Task R1-10: ProviderRow（providerTypes.ts + providerPresentation.ts 共享逻辑）

**Files:**
- Create: `packages/ui/src/components/providerTypes.ts`
- Create: `packages/ui/src/components/providerPresentation.ts`
- Create: `packages/ui/src/components/ProviderRow.{tsx,css,test.tsx}`
- Modify: `packages/ui/src/components/ProviderCard.tsx`（改用 providerPresentation + 从 providerTypes 导入类型）

**共享策略（明确）：**
- `providerTypes.ts` 导出 `ProviderRole` 等共享类型（从 ProviderCard 移出，避免反向依赖）。
- `providerPresentation.ts` 导出 `providerStatusBadge(role, hasKey, enabled)` 返回**本地化无关**的 `{ code: string; variant: StatusBadgeVariant }`（不返回硬编码英文）。
- ProviderCard 和 ProviderRow 都导入 providerTypes + providerPresentation。

```typescript
// providerTypes.ts
export type ProviderRole =
  | { kind: "none" }
  | { kind: "primary" }
  | { kind: "parallel"; index: number }
  | { kind: "fallback" };

export type ProviderStatus =
  | "active"
  | "available"
  | "key-missing"
  | "disabled";
```

```typescript
// providerPresentation.ts
import type { ProviderRole, ProviderStatus } from "./providerTypes";
import type { StatusBadgeVariant } from "./StatusBadge";

/** 本地化无关：返回 status code + badge variant，不返回硬编码英文文本。 */
export function providerStatus(
  role: ProviderRole,
  hasKey: boolean,
  enabled: boolean,
): { code: ProviderStatus; variant: StatusBadgeVariant } {
  if (!enabled) return { code: "disabled", variant: "neutral" };
  if (!hasKey) return { code: "key-missing", variant: "warning" };
  if (role.kind === "none") return { code: "available", variant: "neutral" };
  return { code: "active", variant: "success" };
}
```

> **R11 签名更新**（2026-08-10）：上述签名在 R11 中增加了 `needsKey: boolean` 参数：
> - `providerStatus(role, hasKey, enabled, needsKey)` — `!needsKey` 时提前返回
>   `{ code: "available", variant: "neutral" }`（keyless provider 绝不显示 key-missing）。
> - `providerKeyStatus(hasKey, needsKey)` — 新增函数（R1-10 时 ProviderCard 内联使用，
>   R11 抽出到 providerPresentation），返回三态 `"saved" | "missing" | "not-required"`；
>   `!needsKey` 时返回 `"not-required"`。
> - `providerStatusBadge` 最终定名为 `providerStatus`（R1-10 实现）。
> 详见 2026-08-10 计划文档 R11 appendix。

```typescript
// ProviderRow.test.tsx
import { describe, it, expect, vi } from "vitest";
import { render, fireEvent } from "@solidjs/testing-library";
import ProviderRow from "./ProviderRow";
import { providerStatus } from "./providerPresentation";
import { assertNoAxeViolations } from "../../test/setup";

describe("providerPresentation", () => {
  it("disabled → { code: disabled, variant: neutral }", () => {
    expect(providerStatus({ kind: "none" }, true, false)).toEqual({ code: "disabled", variant: "neutral" });
  });
  it("no key → { code: key-missing, variant: warning }", () => {
    expect(providerStatus({ kind: "primary" }, false, true)).toEqual({ code: "key-missing", variant: "warning" });
  });
  it("primary + key → { code: active, variant: success }", () => {
    expect(providerStatus({ kind: "primary" }, true, true)).toEqual({ code: "active", variant: "success" });
  });
  it("none + key → { code: available, variant: neutral }", () => {
    expect(providerStatus({ kind: "none" }, true, true)).toEqual({ code: "available", variant: "neutral" });
  });
});

describe("ProviderRow", () => {
  const labels = {
    edit: "Edit provider", delete: "Delete provider", enabled: "Enabled",
    statusText: { active: "Active", available: "Available", "key-missing": "Key missing", disabled: "Disabled" },
  };
  const baseProps = {
    name: "OpenAI", template: "openai", hasKey: true, enabled: true,
    role: { kind: "primary" } as const,
    labels,
    onToggle: () => {}, onEdit: () => {}, onDelete: () => {},
  };

  it("renders name and template", () => {
    const { getByText } = render(() => <ProviderRow {...baseProps} />);
    expect(getByText("OpenAI")).toBeInTheDocument();
    expect(getByText("openai")).toBeInTheDocument();
  });

  it("onToggle fires with enabled boolean", () => {
    const onToggle = vi.fn();
    const { getByRole } = render(() => <ProviderRow {...baseProps} onToggle={onToggle} />);
    fireEvent.click(getByRole("switch"));
    expect(onToggle).toHaveBeenCalledOnce();
    expect(onToggle).toHaveBeenCalledWith(false);
  });

  it("onEdit fires", () => {
    const onEdit = vi.fn();
    const { getByLabelText } = render(() => <ProviderRow {...baseProps} onEdit={onEdit} />);
    fireEvent.click(getByLabelText("Edit provider"));
    expect(onEdit).toHaveBeenCalledOnce();
  });

  it("onDelete fires", () => {
    const onDelete = vi.fn();
    const { getByLabelText } = render(() => <ProviderRow {...baseProps} onDelete={onDelete} />);
    fireEvent.click(getByLabelText("Delete provider"));
    expect(onDelete).toHaveBeenCalledOnce();
  });

  it("no axe violations", async () => {
    render(() => <ProviderRow {...baseProps} />);
    await assertNoAxeViolations();
  });
});
```

- [x] **Step 1:** 创建 providerTypes.ts + providerPresentation.ts（代码见上）
- [x] **Step 2:** ProviderCard.tsx 改用 providerTypes + providerPresentation（不改变现有行为）。**✅ rev-4.3.2：ProviderCard 已接入 providerPresentation** —— `import { providerKeyStatus } from "./providerPresentation"`、`import type { ProviderRole } from "./providerTypes"`。密钥状态用 `providerKeyStatus(props.hasKey)`（独立于 enabled/disabled，修复了 disabled+missingKey 组合状态 bug）。新增 `disabled + missing key` 回归测试（ProviderCard.regression.test.tsx）。
- [x] **Step 3:** 写 ProviderRow 测试 → Run → FAIL

- [x] **Step 3a: ProviderRow.tsx 实现（完整代码）：**

```typescript
// ProviderRow.tsx
import { Show, type Component } from "solid-js";
import { Pencil, Trash2 } from "lucide-solid";
import Switch from "./Switch";
import StatusBadge from "./StatusBadge";
import { providerStatus } from "./providerPresentation";
import type { ProviderRole } from "./providerTypes";
import "./ProviderRow.css";

export type ProviderRowLabels = {
  edit: string;
  delete: string;
  enabled: string;
  /** 状态码 → 本地化文字映射（status code 不直接作文字）。 */
  statusText: Record<"active" | "available" | "key-missing" | "disabled", string>;
};

export type ProviderRowProps = {
  name: string;
  template: string;
  hasKey: boolean;
  role: ProviderRole;
  enabled: boolean;
  labels: ProviderRowLabels;
  onToggle: (enabled: boolean) => void;
  onEdit: () => void;
  onDelete: () => void;
};

const ProviderRow: Component<ProviderRowProps> = (props) => {
  const status = () => providerStatus(props.role, props.hasKey, props.enabled);
  return (
    <div class="provider-row">
      <div class="provider-row__info">
        <span class="provider-row__name">{props.name}</span>
        <span class="provider-row__template">{props.template}</span>
      </div>
      <StatusBadge variant={status().variant} label={props.labels.statusText[status().code]} dot />
      <Switch checked={props.enabled} onChange={props.onToggle} label={props.labels.enabled} />
      <button type="button" class="provider-row__btn" aria-label={props.labels.edit} onClick={props.onEdit}>
        <Pencil size={16} />
      </button>
      <button type="button" class="provider-row__btn" aria-label={props.labels.delete} onClick={props.onDelete}>
        <Trash2 size={16} />
      </button>
    </div>
  );
};
export default ProviderRow;
```
- [x] **Step 4:** 实现 ProviderRow.tsx + .css → Run → PASS
- [x] **Step 5:** 回归 ProviderCard + 兼容测试。在 `ProviderCard.test.tsx` 末尾追加（或新建 `ProviderCard.regression.test.tsx`），验证 role badge 和 key badge 同时保留：

```typescript
// ProviderCard.regression.test.tsx（追加到现有测试文件）
import { describe, it, expect } from "vitest";
import { render } from "@solidjs/testing-library";
import ProviderCard, { defaultProviderCardLabels } from "./ProviderCard";
import type { ProviderRole } from "./providerTypes";

describe("ProviderCard role/key badge regression", () => {
  const profile = { name: "OpenAI", template: "openai", status: "active" as const };
  const noop = () => {};

  it("primary role renders primary role badge", () => {
    const role: ProviderRole = { kind: "primary" };
    const { container } = render(() => (
      <ProviderCard profile={profile} role={role} hasKey={true} enabled={true}
        onEdit={noop} onDelete={noop} onToggle={noop} />
    ));
    const badge = container.querySelector(".lr-provider-card__role-badge--primary");
    expect(badge).not.toBeNull();
    expect(badge!.textContent).toContain(defaultProviderCardLabels.primary);
  });

  it("parallel role renders parallel role badge with index", () => {
    const role: ProviderRole = { kind: "parallel", index: 2 };
    const { container } = render(() => (
      <ProviderCard profile={profile} role={role} hasKey={true} enabled={true}
        onEdit={noop} onDelete={noop} onToggle={noop} />
    ));
    const badge = container.querySelector(".lr-provider-card__role-badge--parallel");
    expect(badge).not.toBeNull();
    expect(badge!.textContent).toContain("#2");
  });

  it("key saved status renders when hasKey=true", () => {
    const role: ProviderRole = { kind: "none" };
    const { container } = render(() => (
      <ProviderCard profile={profile} role={role} hasKey={true} enabled={true}
        onEdit={noop} onDelete={noop} onToggle={noop} />
    ));
    const keyStatus = container.querySelector(".lr-provider-card__key-status--saved");
    expect(keyStatus).not.toBeNull();
  });

  it("key missing status renders when hasKey=false", () => {
    const role: ProviderRole = { kind: "none" };
    const { container } = render(() => (
      <ProviderCard profile={profile} role={role} hasKey={false} enabled={true}
        onEdit={noop} onDelete={noop} onToggle={noop} />
    ));
    const keyStatus = container.querySelector(".lr-provider-card__key-status--missing");
    expect(keyStatus).not.toBeNull();
  });
});
```

Run: `pnpm --filter @linguaray/ui exec vitest run src/components/ProviderCard` → PASS（现有测试 + 4 回归测试全绿）。
- [N/A — 未授权 commit] **Step 6:** Commit

---

### Task R1-11: TranslationCard（组合 ResultCard，onRetry 为自身 Button）

**Files:** `packages/ui/src/components/TranslationCard.{tsx,css,test.tsx}`

```typescript
// TranslationCard.tsx
import { Show, type Component } from "solid-js";
import ResultCard, { type ResultAction } from "./ResultCard";
import Spinner from "./Spinner";
import Button from "./Button";
import "./TranslationCard.css";

/** MASTER §7 TranslationCard — state is a discriminated union. */
export type TranslationState =
  | { kind: "loading" }
  | { kind: "success"; text: string; elapsedMs: number }
  | { kind: "failure"; errorText: string };

export type TranslationCardLabels = {
  loadingLabel: string;
  failureText: string;
  retryLabel: string;
};

export type TranslationCardProps = {
  engineId: string;
  engineLabel: string;
  state: TranslationState;
  actions?: ResultAction[];
  labels: TranslationCardLabels;
  onRetry?: () => void;
};

const TranslationCard: Component<TranslationCardProps> = (props) => {
  // Solid 的 `<Show when={boolean}>` 不会收窄 JSX 子元素中 `props` 的联合类型，
  // 因此我们需要在 `<Show>` 内部对联合成员使用显式的类型守卫。
  const successState = () =>
    props.state.kind === "success" ? props.state : undefined;
  const failureState = () =>
    props.state.kind === "failure" ? props.state : undefined;

  return (
    <div class="translation-card">
      <div class="translation-card__result">
        <Show when={props.state.kind === "loading"}>
          <Spinner size={16} label={props.labels.loadingLabel} />
        </Show>

        <Show when={successState()}>
          {(s) => (
            <ResultCard
              engineId={props.engineId}
              engineLabel={props.engineLabel}
              outcome="success"
              text={s().text}
              elapsedMs={s().elapsedMs}
              actions={props.actions}
            />
          )}
        </Show>

        <Show when={failureState()}>
          {(s) => (
            <div class="translation-card__retry">
              {/* MASTER §7: labels.failureText introduces the error before the
                  error text itself (no hardcoded English). */}
              <p class="translation-card__failure-text">{props.labels.failureText}</p>
              <ResultCard
                engineId={props.engineId}
                engineLabel={props.engineLabel}
                outcome="failure"
                errorText={s().errorText}
              />
              <Show when={props.onRetry}>
                <Button variant="primary" size="sm" onClick={() => props.onRetry?.()}>
                  {props.labels.retryLabel}
                </Button>
              </Show>
            </div>
          )}
        </Show>
      </div>
    </div>
  );
};
export default TranslationCard;
```

```typescript
// TranslationCard.test.tsx
import { describe, it, expect, vi } from "vitest";
import { render, fireEvent } from "@solidjs/testing-library";
import TranslationCard from "./TranslationCard";
import { assertNoAxeViolations } from "../../test/setup";

const baseProps = {
  sourceText: "Hello",
  loadingLabel: "Loading…",
  engineId: "google",
  engineLabel: "Google",
};

describe("TranslationCard", () => {
  it("renders source text", () => {
    const { getByText } = render(() => <TranslationCard {...baseProps} outcome="success" resultText="你好" />);
    expect(getByText("Hello")).toBeInTheDocument();
  });

  it("renders result text via ResultCard", () => {
    const { getByText } = render(() => <TranslationCard {...baseProps} outcome="success" resultText="你好" />);
    expect(getByText("你好")).toBeInTheDocument();
  });

  it("loading shows Spinner with loadingLabel, not ResultCard", () => {
    const { container, queryByText, getByText } = render(() =>
      <TranslationCard {...baseProps} outcome="success" resultText="你好" loading={true} />,
    );
    expect(container.querySelector(".lr-spinner")).not.toBeNull();
    expect(getByText("Loading…")).toBeInTheDocument();
    expect(queryByText("你好")).toBeNull();
  });

  it("failure with onRetry renders retry button", () => {
    const { getByText } = render(() => (
      <TranslationCard {...baseProps} outcome="failure" failureText="Network error" retryLabel="Retry" onRetry={() => {}} />
    ));
    expect(getByText("Retry")).toBeInTheDocument();
  });

  it("onRetry fires", () => {
    const onRetry = vi.fn();
    const { getByText } = render(() => (
      <TranslationCard {...baseProps} outcome="failure" failureText="Network error" retryLabel="Retry" onRetry={onRetry} />
    ));
    fireEvent.click(getByText("Retry"));
    expect(onRetry).toHaveBeenCalledOnce();
  });

  it("no axe violations", async () => {
    render(() => <TranslationCard {...baseProps} outcome="success" resultText="你好" />);
    await assertNoAxeViolations();
  });
});
```

- [x] **Step 1–3:** RED → 实现 → GREEN
- [N/A — 未授权 commit] **Step 4:** commit

---

### Task R1-12: 导出 9 组件

**Files:** Modify: `packages/ui/src/index.ts`

- [x] **Step 1:** 追加 9 导出（SegmentedControl、ShortcutChip、StatusBadge、InlineError、WindowChrome、SidebarItem、HistoryRow、ProviderRow、TranslationCard + Props 类型）+ `providerStatus` 函数 + `ProviderStatus`/`ProviderRole` 类型（从 providerTypes）
- [x] **Step 2:** Run: `node -e "const t=require('fs').readFileSync('packages/ui/src/index.ts','utf8'); console.log('exports='+(t.match(/export \{ default as/g)||[]).length);"` → Expected: `24`
- [x] **Step 3:** `pnpm typecheck && pnpm --filter @linguaray/ui exec vitest run` → 全绿
- [N/A — 未授权 commit] **Step 4:** Commit

---

### Task R1-13: ComponentGallery + Playwright 视觉回归基线

**Files:**
- Create: `apps/ui-lab/src/pages/ComponentGallery.{tsx,css}`
- Modify: `apps/ui-lab/src/App.tsx`、`apps/ui-lab/src/i18n/index.ts`
- Create: `apps/ui-lab/test/ComponentGallery.test.tsx`
- Create: `apps/ui-lab/playwright.config.ts`
- Create: `apps/ui-lab/e2e/component-gallery.visual.spec.ts`
- Modify: `apps/ui-lab/package.json`（+ `@playwright/test` devDep + `test:visual`/`test:visual:update` 脚本）

- [x] **Step 1: ComponentGallery.test.tsx（RED）** — 完整独立代码：

```typescript
import { describe, it, expect } from "vitest";
import { render } from "@solidjs/testing-library";
import { ComponentGallery } from "../src/pages/ComponentGallery";
import { assertNoAxeViolations } from "../test/setup";

const EXPECTED_COMPONENT_IDS = [
  "button", "icon-button", "segmented-control", "shortcut-chip",
  "text-field", "select", "switch",
  "status-badge", "inline-error", "toast", "confirm", "empty-state",
  "translation-card", "result-card", "provider-row", "history-row",
  "sidebar-item", "window-chrome",
];

describe("ComponentGallery", () => {
  it("renders exactly 18 design components with data-component-id", () => {
    const { container } = render(() => <ComponentGallery locale="en" theme="light" />);
    for (const id of EXPECTED_COMPONENT_IDS) {
      const el = container.querySelector(`[data-component-id="${id}"]`);
      expect(el, `must have [data-component-id="${id}"]`).not.toBeNull();
    }
    const all = container.querySelectorAll("[data-component-id]");
    expect(all.length, "exactly 18 components").toBe(18);
  });

  it("renders zh labels in zh locale", () => {
    const { getByText } = render(() => <ComponentGallery locale="zh" theme="light" />);
    expect(getByText(/按钮|Button/)).toBeInTheDocument();
  });

  it("light theme: no axe violations", async () => {
    document.documentElement.setAttribute("data-theme", "light");
    render(() => <ComponentGallery locale="en" theme="light" />);
    await assertNoAxeViolations({ disableRules: ["color-contrast"] });
  });

  it("dark theme: no axe violations", async () => {
    document.documentElement.setAttribute("data-theme", "dark");
    render(() => <ComponentGallery locale="en" theme="dark" />);
    await assertNoAxeViolations({ disableRules: ["color-contrast"] });
  });
});
```
- [x] **Step 2:** Run: `pnpm --filter @linguaray/ui-lab exec vitest run test/ComponentGallery.test.tsx` → FAIL
- [x] **Step 3:** 实现 ComponentGallery —— 每个 `<section data-component-id="...">` 展示状态矩阵 + 长中文 + reduced-motion 行
- [x] **Step 4:** App.tsx + i18n → Run: 同命令 → PASS

- [x] **Step 5: Playwright 配置**

`apps/ui-lab/playwright.config.ts`:
```typescript
import { defineConfig, devices } from "@playwright/test";

export default defineConfig({
  testDir: "./e2e",
  use: { baseURL: "http://localhost:1421" },
  webServer: {
    command: "pnpm --filter @linguaray/ui-lab exec vite --port 1421",
    port: 1421,
    reuseExistingServer: !process.env.CI,
  },
  projects: [
    { name: "chromium", use: { ...devices["Desktop Chrome"] } },
  ],
});
```

- [x] **Step 6: 视觉回归测试**

`apps/ui-lab/e2e/component-gallery.visual.spec.ts`:
```typescript
import { test, expect } from "@playwright/test";

const COMPONENT_IDS = [
  "button", "icon-button", "segmented-control", "shortcut-chip",
  "text-field", "select", "switch",
  "status-badge", "inline-error", "toast", "confirm", "empty-state",
  "translation-card", "result-card", "provider-row", "history-row",
  "sidebar-item", "window-chrome",
];

for (const theme of ["light", "dark"] as const) {
  for (const id of COMPONENT_IDS) {
    test(`${theme}/${id} visual baseline`, async ({ page }) => {
      await page.goto(`http://localhost:1421/?nav=component-gallery&theme=${theme}`);
      // 等待本地字体加载完成
      await page.evaluate(() => document.fonts.ready);
      // 设置主题
      await page.evaluate((t) => {
        document.documentElement.setAttribute("data-theme", t);
      }, theme);
      const locator = page.locator(`[data-component-id="${id}"]`);
      // 等待组件可见后再截图
      await expect(locator).toBeVisible();
      await expect(locator).toHaveScreenshot(`${theme}-${id}.png`);
    });
  }
}
```

**rev-4.3.2 截图基线 = 42 张（20 组件 × 2 主题 + 2 reduced-motion）。截图前等待 `document.fonts.ready` 和组件 `toBeVisible()`。** 实际 spec 将 `confirm` 移到独立隔离路由（portal 闭合问题），新增 `spinner` 与 `overflow-cjk` 两个 section，并增加 2 个 reduced-motion Spinner 基线（light/dark 各 1）。

- [x] **Step 6b: SidebarItem 键盘交互 e2e** — `apps/ui-lab/e2e/sidebar-keyboard.spec.ts`:

```typescript
import { test, expect } from "@playwright/test";

test("SidebarItem: Tab focuses, Enter activates, Space activates", async ({ page }) => {
  // 使用隔离路由：只渲染一个 SidebarItem，无其他可聚焦控件
  await page.goto("http://localhost:1421/?nav=sidebar-isolated&theme=light");
  await page.evaluate(() => document.fonts.ready);
  const item = page.locator("button.sidebar-item");
  await expect(item).toBeVisible();
  // 真实 Tab 聚焦（页面中只有这一个可聚焦元素）
  await page.keyboard.press("Tab");
  await expect(item).toBeFocused();
  // Enter 激活
  let clicked = false;
  await page.exposeFunction("__trackClick", () => { clicked = true; });
  await item.evaluate((el) => el.addEventListener("click", () => (window as any).__trackClick()));
  await page.keyboard.press("Enter");
  expect(clicked).toBe(true);
  // Space 激活
  clicked = false;
  await page.keyboard.press("Space");
  expect(clicked).toBe(true);
});
```

> **隔离路由要求：** ui-lab 必须新增一个 `sidebar-isolated` 路由/页面，只渲染一个 SidebarItem（无 Button/Select/Switch 等其他可聚焦控件），确保首次 Tab 焦点落在 SidebarItem 上。R1-13 Step 3 实现 ComponentGallery 时同步创建此隔离 fixture。

- [x] **Step 7:** `apps/ui-lab/package.json` 新增：
```json
"devDependencies": { "@playwright/test": "1.49.1" },
"scripts": {
  "test:visual": "playwright test",
  "test:visual:update": "playwright test --update-snapshots"
}
```

- [x] **Step 8:** 生成基线: `pnpm --filter @linguaray/ui-lab exec playwright install chromium && pnpm --filter @linguaray/ui-lab test:visual:update`
- [x] **Step 9:** 验证全部用例: `pnpm --filter @linguaray/ui-lab exec playwright test` → **43 passed**（rev-4.3.2：40 visual screenshots + 2 reduced-motion + 1 keyboard test）
- [x] **Step 10:** 回归: `pnpm typecheck && pnpm test && pnpm build && pnpm --filter @linguaray/ui-lab build` → 全绿
- [N/A — 未授权 commit] **Step 11:** Commit

---

### Task R1-14: R1 完成验证

- [x] **Step 1:**
```bash
pnpm typecheck
pnpm test
pnpm build
pnpm --filter @linguaray/ui-lab build
pnpm --filter @linguaray/ui-lab test:visual
cd src-tauri && cargo test --features xproc-test-helper && cargo clippy --all-targets --features xproc-test-helper -- -D warnings && cd ..
```
- [x] **Step 2:** 手动视觉验证（ui-lab dev server）
- [x] **Step 3:** 验收门（rev-4.3.2：24 导出 + 无 --core-* + 别名无环 + WCAG + SHA-256 + 42 截图基线 + 43 Playwright 用例 + ui 204 测试 + ui-lab 201 测试 + 后端 53）

---

## 组件依赖图

```
R1-1 tokens + aliases + contrast
  ├─ R1-2 字体
  ├─ R1-3 SegmentedControl
  ├─ R1-4 ShortcutChip
  ├─ R1-5 StatusBadge → R1-10 providerPresentation 引用 StatusBadgeVariant
  ├─ R1-6 InlineError
  ├─ R1-7 WindowChrome
  ├─ R1-8 SidebarItem
  ├─ R1-9 HistoryRow
  ├─ R1-10 ProviderRow (providerTypes.ts + providerPresentation.ts + R1-5)
  ├─ R1-11 TranslationCard (组合 ResultCard)
  ├─ R1-12 导出 (R1-3~R1-11)
  ├─ R1-13 Gallery + Playwright 42 截图（rev-4.3.2）
  └─ R1-14 验证（manifest 验证已在 R0-3）
```

---

## 尚未解决的决策

**全部已清零。**

| 项 | 方案 |
|---|---|
| Core 数量 | 97（colors 44 + spacing 14 + radius 9 + border-width 2 + opacity 2 + font-family 3 + font-size 9 + font-weight 4 + typography 10） |
| Semantic 数量 | Light 28 + Dark 28（逐项脚本计数） |
| border.strong | 保留装饰；border.control=#64748B 工程扩展用于控件边界 |
| strong-fill | 两主题统一 #15803D/#B45309/#DC2626/#2563EB + #FFFFFF |
| StatusBadge | Light core.700/core.50；Dark text.primary/soft |
| ProviderRow 共享 | providerTypes.ts + providerPresentation.ts（本地化无关 code/variant） |
| TranslationCard retry | 自身 Button（不假装传给 ResultCard） |
| 视觉回归 | rev-4.3.2：R1 建 42 张截图基线（20×2 + 2 reduced-motion）；Playwright 43 用例；Surface 截图随 R2–R6 |
| 字体 | 方案 B，5 woff2，2,393,380 bytes，SHA-256 |
| AppSidebar/ProgressRail/TextArea/Checkbox/Card/ListRow | 移出 R1 / backlog |

---

## 对 rev-4 的修正（rev-4.1 定点修复）

1. Penpot 原生名称精确化：`font.weight.400/500/600/700`、10 个 typography Token 完整列出（type.display/title/body/label/code + 各自 line-height）、overlay/shadow 的 Penpot 原始效果对象与 CSS 归一化值分开记录（标注 `[CSS 归一化]`，不得标为 Penpot 原生 1:1）
2. 删除所有 rev-3 引用（"见 rev-3 代码"/"同 rev-3"/"代码同 rev-3"），补全 contrast.test.ts、fonts.test.ts、ComponentGallery.test.tsx 完整独立代码
3. TypeScript typecheck 修复：ShortcutChip 删除 ariaLabel prop（chip 有可见文本）；ResultCard 改为 default import；Spinner 测试选择器 `.spinner` → `.lr-spinner`；所有组件 label 使用可见本地化文本
4. 别名纪律测试重写：递归扫描 .ts/.tsx/.css；必需目录不存在必须 fail；helper 文件（providerPresentation.ts/providerTypes.ts）豁免 CSS/TSX 配对但禁止 legacy token；从 ALIASES 映射表生成旧 Token 名集合；新文件禁止任何旧 Token
5. 交互修复：SegmentedControl 方向键改变选中项并移动真实 DOM 焦点（document.activeElement 断言）；删除原生 button 人工 Enter/Space 逻辑（SidebarItem/HistoryRow）；HistoryRow 无 onClick 时渲染非交互 div；ProviderCard 增加 role/key badge 兼容回归测试
6. Manifest 严格验证移到 R0-3：Team 1 + File 1 + Page 8 + Surface 16 + Component 18、UUID 格式合法且全局唯一、0 TBD、R0 检查点失败禁止进入 R1
7. Playwright：@playwright/test 固定 1.49.1；截图前等待 document.fonts.ready + 组件 toBeVisible；rev-4.3.2 实际基线 = 42 张（20 组件×2 主题 + 2 reduced-motion），用例 43（40 visual + 2 reduced-motion + 1 keyboard）；`confirm` 走隔离路由、`spinner`/`overflow-cjk` 新增 section
