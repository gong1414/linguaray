# Rayline R0 冻结决策记录（rev-4.2）

**状态：已冻结；R0/R1 已合并** · **日期：** 2026-08-08 · **状态同步：** 2026-08-13
**所属流程：** Superpowers brainstorming → writing-plans（rev-4.2）
**上游文档：**
- 产品基线（S0 冻结）：[2026-08-01-linguaray-product-baseline.md](2026-08-01-linguaray-product-baseline.md)
- Rayline 设计交付：[RAYLINE-REDESIGN.md](../../../design-system/linguaray/RAYLINE-REDESIGN.md)
- 实施计划 rev-4.2：[2026-08-08-rayline-r0-r1-implementation.md](../plans/2026-08-08-rayline-r0-r1-implementation.md)

> 本文件记录 R0 阶段裁决。R0/R1 均不修改生产代码。

---

## 1. 审核结论：采用「调和冻结」策略

S0 对颜色完全中立。采用调和冻结：Penpot 三集合为唯一设计源 + 工程扩展分栏 + WCAG 自动验证。

---

## 2. 冻结规则

### 2.1 架构：Penpot 三集合 Token（Core 97 + Semantic 28+28 = 153）

- **Core / Primitives — 97 Token：**
  - colors 44：`color.core.white`/`black`(2)、neutral 50–950(11)、indigo 50–900(10)、cyan 50–800(9)、green/amber/red 各 50/500/600/700(12)
  - spacing 14、radius 9、border width 2（`border.1`/`border.2`）、opacity 2（`opacity.disabled`/`opacity.muted`）
  - font families 3（`font.family.sans`/`cjk`/`mono`）、font sizes 9（`font.size.11–32`）、font weights 4（`font.weight.400/500/600/700`）
  - typography 10 个复合 Token（`type.display`(size.32/weight.700/1.25)、`type.title.lg`(size.24/weight.700/1.33)、`type.title.md`(size.20/weight.600/1.4)、`type.title.sm`(size.16/weight.600/1.5)、`type.body.lg`(size.16/weight.400/1.5)、`type.body.md`(size.14/weight.400/1.43)、`type.body.sm`(size.12/weight.400/1.5)、`type.label.md`(size.13/weight.600/1.23)、`type.label.sm`(size.11/weight.600/1.27)、`type.code`(size.12/weight.500/1.5/mono)）—— 每个 size/weight/lineHeight 复合，不拆 `*.line-height`
  - **禁止把 `color.core.white` 虚构为 `color.core.neutral.0`。使用 neutral（不得 slate）。**
  - **Indigo：400=`#818CF8`、500=`#6366F1`、600=`#4F46E5`、700=`#4338CA`**

- **Semantic / Light — 28 Token**（Penpot 原生，逐项镜像）
- **Semantic / Dark — 28 Token**（Penpot 原生）

**三层名称：** Penpot 原名（设计源）→ CSS Semantic 名（代码）→ 旧兼容名（别名，迁移后删）。Penpot 原生栏不得改名。

**`color.overlay` 是 Penpot 原生**（不是工程扩展）。`color.success.default/soft`、`color.warning.default/soft`、`color.danger.default/soft` 是 Penpot 原生。

**Core 引用权：** 只有 tokens.css 可引用 `--core-*`。

**Surface 映射：** Light canvas=`#F8FAFC`/surface.default=`#FFFFFF`/subtle=`#F1F5F9`/raised=`#FFFFFF`；Dark canvas=`#020617`/surface.default=`#0F172A`/subtle=`#1E293B`/raised=`#1E293B`。

### 2.2 工程扩展 Token

| Token | Light | Dark | 用途 | WCAG |
|---|---|---|---|---|
| `color.surface.hover` | `#F1F5F9` | `#334155` | hover 背景 | — |
| `color.surface.selected` | `#DBEAFE` | `#1E3A5F` | 选中背景 | — |
| `color.text.selected` | `#1D4ED8` | `#60A5FA` | 选中文字 | — |
| `color.brand.on-fill` | `#FFFFFF` | `#0F172A` | 品牌填充文字 | 6.288/5.985 ✅ |
| `color.brand.fg` | `#4F46E5` | `#818CF8` | 品牌前景 | 6.009/6.763 ✅ |
| `color.status.info` | `#2563EB` | `#60A5FA` | 信息状态 | — |
| `color.status.info.soft` | `#EFF6FF` | `#1E3A8A` | 信息软背景 | — |
| `color.disabled.bg` | `#F1F5F9` | `#1E293B` | 禁用背景 | — |
| **`color.border.control`** | **`#64748B`** | **`#64748B`** | **控件边界** | **4.548/4.239 ✅** |
| `color.strong-fill.success` | `#15803D` | `#15803D` | Banner/Button 强填充 | 5.016 ✅ |
| `color.strong-fill.warning` | `#B45309` | `#B45309` | 同上 | 5.022 ✅ |
| `color.strong-fill.danger` | `#DC2626` | `#DC2626` | 同上 | 4.829 ✅ |
| `color.strong-fill.info` | `#2563EB` | `#2563EB` | 同上 | 5.169 ✅ |

**border.control 冻结：** `#64748B` 两主题。TextField/Select 用此。border.strong(`#94A3B8` Light / `#475569` Dark) 保留装饰（不达 3:1）。

**Strong fill 两主题统一：** success=`#15803D`、warning=`#B45309`、danger=`#DC2626`、info=`#2563EB`，on-fill=`#FFFFFF`。

### 2.3 品牌色

| Token | Light | Dark |
|---|---|---|
| `color.brand.default` | `#4F46E5` (indigo.600) | `#818CF8` (indigo.400) |
| `color.brand.on-fill` | `#FFFFFF` | `#0F172A` |
| `color.brand.fg` | `#4F46E5` | `#818CF8` |
| `color.brand.hover` | `#4338CA` | `#A5B4FC` |

WCAG（脚本值）：`#4F46E5`/`#FFFFFF`=6.288 ✅、`#818CF8`/`#0F172A`=5.985 ✅、`#818CF8`/`#020617`=6.763 ✅。

### 2.4 焦点色

Light `#0891B2`（3.682:1 ✅）、Dark `#22D3EE`（9.879:1 ✅）。Accent `#06B6D4` 仅装饰（2.428:1）。

### 2.5 StatusBadge 状态角色

- **Light：** core.700 前景 + core.50 soft 背景（success `#15803D`/`#F0FDF4` 4.791:1 ✅ 等）
- **Dark：** `--color-text-primary` + Dark soft 背景（`#F8FAFC`/`#15803D` 4.794:1 ✅ 等）
- 禁止 success/warning 实色配白字

### 2.6 无障碍约束

焦点 3:1、WCAG 对比度脚本自动验证（禁手工填）、键盘、reduced-motion。

### 2.7 字体（方案 B，实测冻结）

| 文件 | 大小 | SHA-256 |
|---|---|---|
| inter-latin-wght-normal.woff2 | 48,256 | `3100e775...` |
| ibm-plex-mono-latin-400-normal.woff2 | 14,708 | `08949f72...` |
| ibm-plex-mono-latin-600-normal.woff2 | 15,620 | `0d1f0b8d...` |
| noto-sans-sc-chinese-simplified-400-normal.woff2 | 1,142,552 | `95e3633b...` |
| noto-sans-sc-chinese-simplified-700-normal.woff2 | 1,172,244 | `e1df51ed...` |
| **总计** | **2,393,380 (2.28 MB)** | |

本地打包，OFL，禁远程。@fontsource 放 devDependencies。R1 只验证源码许可证；发行包列入 R7。

### 2.8 Token 迁移：别名桥接

旧→新单向别名。形式 `--old: var(--new);`。目标统一 `--color-brand-default`。含 `--color-success-fill → --color-strong-fill-success` 等。aliases.test.ts：CSS 文本解析（非 JSDOM）+ 两主题 + 循环 + 新文件无旧名 + 无 skip。

---

## 3. R0 / R1 边界

**R0（仅文档）：** token-map.md、MASTER.md、handoff-manifest.md（34 Node ID 强制真实 UUID）、pages/01–16.md。不改 tokens.css/组件/src/后端。

**R1（代码）：** tokens.css、fonts、9 组件、providerTypes/providerPresentation、aliases/contrast/fonts 测试、ComponentGallery + Playwright 36 截图、manifest 解析测试。不改 src/ 和 src-tauri/。

---

## 4. 组件清单

**18 设计组件 = 9 已有 + 9 新增。**
**6 辅助组件：** Banner、Dialog、Tooltip、Spinner、ProviderCard、VisuallyHidden。
**概念契约（R1 时 backlog）：** TextArea、Checkbox、Card、ListRow。TextArea
已在 R2 提升为生产组件；ListRow 在 R3b Surface 07 提升为生产组件。
**移出 R1：** AppSidebar、ProgressRail。
**packages/ui 最终 = 24 导出。**

---

## 5. 审核状态

**已冻结。** 用户完成 rev-1 → rev-4.2 六轮审核，R0/R1 已实现、验证并合并。
本状态同步只纠正历史治理信息，不改变上述冻结 Token 或组件契约。
