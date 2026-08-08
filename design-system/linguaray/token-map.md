# LinguaRay Token Map — Penpot → CSS → Legacy Alias

**状态：** R0 冻结 · **日期：** 2026-08-08
**Penpot 源文件：** `3be9e5e1-190f-8090-8008-72a4d9868ce7`
**设计源：** [RAYLINE-REDESIGN.md](RAYLINE-REDESIGN.md) · [MASTER.md](MASTER.md)

> 三层名称：**Penpot 原生名**（设计源）→ **CSS Semantic 名**（代码使用）→ **旧兼容名**（别名，16 Surface 迁移后删除）。
> Penpot 原生栏不得改名。CSS 名是工程映射。旧名仅用于过渡兼容。

---

## 栏 1：Penpot 原生 Token（97 Core + 28 Semantic Light + 28 Semantic Dark = 153）

### Core / Primitives — 97 Token

#### colors — 44

| Penpot 原生名 | CSS 名 | 值 |
|---|---|---|
| `color.core.white` | `--core-color-white` | `#FFFFFF` |
| `color.core.black` | `--core-color-black` | `#000000` |
| `color.core.neutral.50` | `--core-color-neutral-50` | `#F8FAFC` |
| `color.core.neutral.100` | `--core-color-neutral-100` | `#F1F5F9` |
| `color.core.neutral.200` | `--core-color-neutral-200` | `#E2E8F0` |
| `color.core.neutral.300` | `--core-color-neutral-300` | `#CBD5E1` |
| `color.core.neutral.400` | `--core-color-neutral-400` | `#94A3B8` |
| `color.core.neutral.500` | `--core-color-neutral-500` | `#64748B` |
| `color.core.neutral.600` | `--core-color-neutral-600` | `#475569` |
| `color.core.neutral.700` | `--core-color-neutral-700` | `#334155` |
| `color.core.neutral.800` | `--core-color-neutral-800` | `#1E293B` |
| `color.core.neutral.900` | `--core-color-neutral-900` | `#0F172A` |
| `color.core.neutral.950` | `--core-color-neutral-950` | `#020617` |
| `color.core.indigo.50` | `--core-color-indigo-50` | `#EEF2FF` |
| `color.core.indigo.100` | `--core-color-indigo-100` | `#E0E7FF` |
| `color.core.indigo.200` | `--core-color-indigo-200` | `#C7D2FE` |
| `color.core.indigo.300` | `--core-color-indigo-300` | `#A5B4FC` |
| `color.core.indigo.400` | `--core-color-indigo-400` | `#818CF8` |
| `color.core.indigo.500` | `--core-color-indigo-500` | `#6366F1` |
| `color.core.indigo.600` | `--core-color-indigo-600` | `#4F46E5` |
| `color.core.indigo.700` | `--core-color-indigo-700` | `#4338CA` |
| `color.core.indigo.800` | `--core-color-indigo-800` | `#3730A3` |
| `color.core.indigo.900` | `--core-color-indigo-900` | `#312E81` |
| `color.core.cyan.50` | `--core-color-cyan-50` | `#ECFEFF` |
| `color.core.cyan.100` | `--core-color-cyan-100` | `#CFFAFE` |
| `color.core.cyan.200` | `--core-color-cyan-200` | `#A5F3FC` |
| `color.core.cyan.300` | `--core-color-cyan-300` | `#67E8F9` |
| `color.core.cyan.400` | `--core-color-cyan-400` | `#22D3EE` |
| `color.core.cyan.500` | `--core-color-cyan-500` | `#06B6D4` |
| `color.core.cyan.600` | `--core-color-cyan-600` | `#0891B2` |
| `color.core.cyan.700` | `--core-color-cyan-700` | `#0E7490` |
| `color.core.cyan.800` | `--core-color-cyan-800` | `#155E75` |
| `color.core.green.50` | `--core-color-green-50` | `#F0FDF4` |
| `color.core.green.500` | `--core-color-green-500` | `#22C55E` |
| `color.core.green.600` | `--core-color-green-600` | `#16A34A` |
| `color.core.green.700` | `--core-color-green-700` | `#15803D` |
| `color.core.amber.50` | `--core-color-amber-50` | `#FFFBEB` |
| `color.core.amber.500` | `--core-color-amber-500` | `#F59E0B` |
| `color.core.amber.600` | `--core-color-amber-600` | `#D97706` |
| `color.core.amber.700` | `--core-color-amber-700` | `#B45309` |
| `color.core.red.50` | `--core-color-red-50` | `#FEF2F2` |
| `color.core.red.500` | `--core-color-red-500` | `#EF4444` |
| `color.core.red.600` | `--core-color-red-600` | `#DC2626` |
| `color.core.red.700` | `--core-color-red-700` | `#B91C1C` |

> **注意：** 使用 `neutral`（Penpot 原生名），不得改成 slate。`white`/`black` 独立于 neutral。

#### spacing — 14

| Penpot 原生名 | CSS 名 | 值 |
|---|---|---|
| `space.0` | `--core-space-0` | `0px` |
| `space.2` | `--core-space-2` | `2px` |
| `space.4` | `--core-space-4` | `4px` |
| `space.6` | `--core-space-6` | `6px` |
| `space.8` | `--core-space-8` | `8px` |
| `space.10` | `--core-space-10` | `10px` |
| `space.12` | `--core-space-12` | `12px` |
| `space.16` | `--core-space-16` | `16px` |
| `space.20` | `--core-space-20` | `20px` |
| `space.24` | `--core-space-24` | `24px` |
| `space.32` | `--core-space-32` | `32px` |
| `space.40` | `--core-space-40` | `40px` |
| `space.48` | `--core-space-48` | `48px` |
| `space.64` | `--core-space-64` | `64px` |

#### radius — 9

| Penpot 原生名 | CSS 名 | 值 |
|---|---|---|
| `radius.0` | `--core-radius-0` | `0px` |
| `radius.4` | `--core-radius-4` | `4px` |
| `radius.6` | `--core-radius-6` | `6px` |
| `radius.8` | `--core-radius-8` | `8px` |
| `radius.10` | `--core-radius-10` | `10px` |
| `radius.12` | `--core-radius-12` | `12px` |
| `radius.16` | `--core-radius-16` | `16px` |
| `radius.20` | `--core-radius-20` | `20px` |
| `radius.full` | `--core-radius-full` | `9999px` |

#### border width — 2

| Penpot 原生名 | CSS 名 | 值 |
|---|---|---|
| `border.1` | `--core-border-1` | `1px` |
| `border.2` | `--core-border-2` | `2px` |

#### opacity — 2

| Penpot 原生名 | CSS 名 | 值 |
|---|---|---|
| `opacity.disabled` | `--core-opacity-disabled` | `0.5` |
| `opacity.muted` | `--core-opacity-muted` | `0.7` |

#### font families — 3

| Penpot 原生名 | CSS 名 | 值 |
|---|---|---|
| `font.family.sans` | `--core-font-family-sans` | `"Inter", "Noto Sans SC", -apple-system, ...` |
| `font.family.cjk` | `--core-font-family-cjk` | `"Noto Sans SC", "PingFang SC", "Microsoft YaHei", ...` |
| `font.family.mono` | `--core-font-family-mono` | `"IBM Plex Mono", "SF Mono", "Consolas", ...` |

#### font sizes — 9

| Penpot 原生名 | CSS 名 | 值 |
|---|---|---|
| `font.size.11` | `--core-font-size-11` | `11px` |
| `font.size.12` | `--core-font-size-12` | `12px` |
| `font.size.13` | `--core-font-size-13` | `13px` |
| `font.size.14` | `--core-font-size-14` | `14px` |
| `font.size.16` | `--core-font-size-16` | `16px` |
| `font.size.18` | `--core-font-size-18` | `18px` |
| `font.size.20` | `--core-font-size-20` | `20px` |
| `font.size.24` | `--core-font-size-24` | `24px` |
| `font.size.32` | `--core-font-size-32` | `32px` |

#### font weights — 4

| Penpot 原生名 | CSS 名 | 值 |
|---|---|---|
| `font.weight.400` | `--core-font-weight-400` | `400` |
| `font.weight.500` | `--core-font-weight-500` | `500` |
| `font.weight.600` | `--core-font-weight-600` | `600` |
| `font.weight.700` | `--core-font-weight-700` | `700` |

#### typography — 10 复合 Token

| Penpot 原生名 | CSS 名 | 组成 |
|---|---|---|
| `type.display` | `--core-type-display` | size.32 / weight.700 / lineHeight 1.25 |
| `type.title.lg` | `--core-type-title-lg` | size.24 / weight.700 / lineHeight 1.33 |
| `type.title.md` | `--core-type-title-md` | size.20 / weight.600 / lineHeight 1.4 |
| `type.title.sm` | `--core-type-title-sm` | size.16 / weight.600 / lineHeight 1.5 |
| `type.body.lg` | `--core-type-body-lg` | size.16 / weight.400 / lineHeight 1.5 |
| `type.body.md` | `--core-type-body-md` | size.14 / weight.400 / lineHeight 1.43 |
| `type.body.sm` | `--core-type-body-sm` | size.12 / weight.400 / lineHeight 1.5 |
| `type.label.md` | `--core-type-label-md` | size.13 / weight.600 / lineHeight 1.23 |
| `type.label.sm` | `--core-type-label-sm` | size.11 / weight.600 / lineHeight 1.27 |
| `type.code` | `--core-type-code` | size.12 / weight.500 / lineHeight 1.5 / mono |

**Core 合计 = 44 + 14 + 9 + 2 + 2 + 3 + 9 + 4 + 10 = 97**

---

### Semantic / Light — 28 Token（Penpot 原生）

| # | Penpot 原生名 | CSS Semantic 名 | 值 |
|---|---|---|---|
| 1 | `color.canvas` | `--color-canvas` | `#F8FAFC` |
| 2 | `color.surface.default` | `--color-surface-default` | `#FFFFFF` |
| 3 | `color.surface.subtle` | `--color-surface-subtle` | `#F1F5F9` |
| 4 | `color.surface.raised` | `--color-surface-raised` | `#FFFFFF` |
| 5 | `color.surface.inverse` | `--color-surface-inverse` | `#0F172A` |
| 6 | `color.text.primary` | `--color-text-primary` | `#0F172A` |
| 7 | `color.text.secondary` | `--color-text-secondary` | `#475569` |
| 8 | `color.text.tertiary` | `--color-text-tertiary` | `#64748B` |
| 9 | `color.text.disabled` | `--color-text-disabled` | `#94A3B8` |
| 10 | `color.text.inverse` | `--color-text-inverse` | `#F8FAFC` |
| 11 | `color.brand.default` | `--color-brand-default` | `#4F46E5` |
| 12 | `color.brand.hover` | `--color-brand-hover` | `#4338CA` |
| 13 | `color.brand.soft` | `--color-brand-soft` | `#EEF2FF` |
| 14 | `color.accent.default` | `--color-accent-default` | `#06B6D4` |
| 15 | `color.accent.soft` | `--color-accent-soft` | `#ECFEFF` |
| 16 | `color.border.subtle` | `--color-border-subtle` | `#E2E8F0` |
| 17 | `color.border.default` | `--color-border-default` | `#CBD5E1` |
| 18 | `color.border.strong` | `--color-border-strong` | `#94A3B8` |
| 19 | `color.focus` | `--color-focus` | `#0891B2` |
| 20 | `color.success.default` | `--color-status-success` | `#16A34A` |
| 21 | `color.success.soft` | `--color-status-success-soft` | `#F0FDF4` |
| 22 | `color.warning.default` | `--color-status-warning` | `#D97706` |
| 23 | `color.warning.soft` | `--color-status-warning-soft` | `#FFFBEB` |
| 24 | `color.danger.default` | `--color-status-danger` | `#DC2626` |
| 25 | `color.danger.soft` | `--color-status-danger-soft` | `#FEF2F2` |
| 26 | `color.overlay` | `--color-overlay` | `{color.core.neutral.950}` |
| 27 | `shadow.raised` | `--shadow-raised` | `0 8px 24px -2px #0F172A` |
| 28 | `shadow.overlay` | `--shadow-overlay` | `0 16px 40px -4px #0F172A` |

### Semantic / Dark — 28 Token（Penpot 原生）

| # | Penpot 原生名 | CSS Semantic 名 | 值 |
|---|---|---|---|
| 1 | `color.canvas` | `--color-canvas` | `#020617` |
| 2 | `color.surface.default` | `--color-surface-default` | `#0F172A` |
| 3 | `color.surface.subtle` | `--color-surface-subtle` | `#1E293B` |
| 4 | `color.surface.raised` | `--color-surface-raised` | `#1E293B` |
| 5 | `color.surface.inverse` | `--color-surface-inverse` | `#F8FAFC` |
| 6 | `color.text.primary` | `--color-text-primary` | `#F8FAFC` |
| 7 | `color.text.secondary` | `--color-text-secondary` | `#CBD5E1` |
| 8 | `color.text.tertiary` | `--color-text-tertiary` | `#94A3B8` |
| 9 | `color.text.disabled` | `--color-text-disabled` | `#475569` |
| 10 | `color.text.inverse` | `--color-text-inverse` | `#0F172A` |
| 11 | `color.brand.default` | `--color-brand-default` | `#818CF8` |
| 12 | `color.brand.hover` | `--color-brand-hover` | `#A5B4FC` |
| 13 | `color.brand.soft` | `--color-brand-soft` | `#312E81` |
| 14 | `color.accent.default` | `--color-accent-default` | `#22D3EE` |
| 15 | `color.accent.soft` | `--color-accent-soft` | `#164E63` |
| 16 | `color.border.subtle` | `--color-border-subtle` | `#1E293B` |
| 17 | `color.border.default` | `--color-border-default` | `#334155` |
| 18 | `color.border.strong` | `--color-border-strong` | `#475569` |
| 19 | `color.focus` | `--color-focus` | `#22D3EE` |
| 20 | `color.success.default` | `--color-status-success` | `#22C55E` |
| 21 | `color.success.soft` | `--color-status-success-soft` | `#15803D` |
| 22 | `color.warning.default` | `--color-status-warning` | `#F59E0B` |
| 23 | `color.warning.soft` | `--color-status-warning-soft` | `#B45309` |
| 24 | `color.danger.default` | `--color-status-danger` | `#EF4444` |
| 25 | `color.danger.soft` | `--color-status-danger-soft` | `#B91C1C` |
| 26 | `color.overlay` | `--color-overlay` | `{color.core.black}` |
| 27 | `shadow.raised` | `--color-shadow-raised` | `0 8px 24px -2px #000000` |
| 28 | `shadow.overlay` | `--color-shadow-overlay` | `0 16px 40px -4px #000000` |

> **shadow/overlay 原生值：** 上表为 Penpot 原生 Token 值（hex shadow color，非 rgba 半透明）。`color.overlay` 的 Light 值是 `{color.core.neutral.950}`（Token 引用），Dark 值是 `{color.core.black}`。视觉画板上的半透明 drop-shadow 效果是渲染层 opacity，不是 Token 值。

---

## 栏 2：工程扩展 Token（Penpot 中不存在）

以下 Token 在 Penpot 文件中不存在，是工程实现所需。标注 `[工程扩展]`。**未回写 Penpot 前不得宣称 1:1 一致。**

| 工程扩展 Token | CSS 名 | Light | Dark | 用途 | 需要原因 |
|---|---|---|---|---|---|
| `color.surface.hover` | `--color-surface-hover` | `#F1F5F9` | `#334155` | hover 背景 | 交互态需独立 Token |
| `color.surface.selected` | `--color-surface-selected` | `#DBEAFE` | `#1E3A5F` | 选中背景 | 交互态需独立 Token |
| `color.text.selected` | `--color-text-selected` | `#1D4ED8` | `#60A5FA` | 选中文字 | 与 surface.selected 配对 |
| `color.brand.on-fill` | `--color-brand-on-fill` | `#FFFFFF` | `#0F172A` | 品牌填充文字 | WCAG 配对（6.288/5.985 ✅） |
| `color.brand.fg` | `--color-brand-fg` | `#4F46E5` | `#818CF8` | 画布品牌前景 | 与 brand.default 区分 |
| `color.status.info` | `--color-status-info` | `#2563EB` | `#60A5FA` | 信息状态 | Penpot 有 success/warning/danger 但无 info |
| `color.status.info.soft` | `--color-status-info-soft` | `#EFF6FF` | `#1E3A8A` | 信息软背景 | 同上 |
| `color.disabled.bg` | `--color-disabled-bg` | `#F1F5F9` | `#1E293B` | 禁用背景 | 交互态需独立 Token |
| `color.border.control` | `--color-border-control` | `#64748B` | `#64748B` | 控件边界 | border.strong 不达 3:1，控件需独立 3:1+ Token |
| `color.strong-fill.success` | `--color-strong-fill-success` | `#15803D` | `#15803D` | Banner/Button 强填充 | Penpot soft 模式不够强；success.default #16A34A 配白字仅 3.296:1 ❌ |
| `color.strong-fill.warning` | `--color-strong-fill-warning` | `#B45309` | `#B45309` | 同上 | warning.default #D97706 配白字仅 3.186:1 ❌ |
| `color.strong-fill.danger` | `--color-strong-fill-danger` | `#DC2626` | `#DC2626` | 同上 | danger.default #EF4444 配白字 4.829:1，统一用 #DC2626 |
| `color.strong-fill.info` | `--color-strong-fill-info` | `#2563EB` | `#2563EB` | 同上 | info 状态填充 |
| `color.strong-on.success` | `--color-strong-on-success` | `#FFFFFF` | `#FFFFFF` | 强填充文字 | WCAG 配对（5.016:1 ✅） |
| `color.strong-on.warning` | `--color-strong-on-warning` | `#FFFFFF` | `#FFFFFF` | 同上 | WCAG 5.022:1 ✅ |
| `color.strong-on.danger` | `--color-strong-on-danger` | `#FFFFFF` | `#FFFFFF` | 同上 | WCAG 4.829:1 ✅ |
| `color.strong-on.info` | `--color-strong-on-info` | `#FFFFFF` | `#FFFFFF` | 同上 | WCAG 5.169:1 ✅ |
| `color.status.success.fg` | `--color-status-success-fg` | `#15803D` | `#F8FAFC` | StatusBadge 软背景前景 | success.default #16A34A 配 success.soft #F0FDF4 仅 3.421:1 ❌；Dark 软背景已够深，前景用 canvas #F8FAFC |
| `color.status.warning.fg` | `--color-status-warning-fg` | `#B45309` | `#F8FAFC` | 同上 | warning.default #D97706 配 warning.soft #FFFBEB 仅 3.186:1 ❌ |
| `color.status.danger.fg` | `--color-status-danger-fg` | `#B91C1C` | `#F8FAFC` | 同上 | danger 配 #FFFFFF 文字 4.829:1，软背景模式用更深的 #B91C1C（5.915:1 ✅）；Dark 用 canvas |
| `color.status.info.fg` | `--color-status-info-fg` | `#1D4ED8` | `#F8FAFC` | 同上 | info.* 在 Penpot 不存在；与 strong-fill-info #2563EB 区分（soft 模式前景需更深） |

> **status-*-fg 用途说明：** StatusBadge 软背景模式（前景 / 软背景）需要独立的 WCAG AA 合规前景 Token。Penpot 原生 status.default 值（success #16A34A / warning #D97706 / danger #DC2626）在对应的 soft 背景上均不达 4.5:1（success 3.421 / warning 3.186 / danger 4.415 ❌），所以新增此 4 个工程扩展 Token。Light 取深色（core-green-700 / amber-700 / red-700 / 独立蓝），Dark 因软背景本身已深（success.soft #15803D / warning.soft #B45309 / danger.soft #B91C1C / info.soft #1E3A8A），前景统一用 canvas 文字 #F8FAFC（4.794/4.800/6.184/9.900 ✅）。

**Penpot 回写门：** 以上 21 个工程扩展 Token 需回写到 Penpot 文件后才能宣称 1:1 一致。回写前此表作为「工程扩展」记录。

---

## 栏 3：旧→新别名表（仅旧名 ≠ 新名）

| 旧兼容名 | 新 Semantic 名（别名指向） |
|---|---|
| `--color-primary-fill` | `var(--color-brand-default)` |
| `--color-on-primary-fill` | `var(--color-brand-on-fill)` |
| `--color-primary-fg` | `var(--color-brand-fg)` |
| `--color-bg` | `var(--color-canvas)` |
| `--color-fg` | `var(--color-text-primary)` |
| `--color-bg-elevated` | `var(--color-surface-default)` |
| `--color-fg-elevated` | `var(--color-text-primary)` |
| `--color-bg-hover` | `var(--color-surface-hover)` `[工程扩展]` |
| `--color-bg-selected` | `var(--color-surface-selected)` `[工程扩展]` |
| `--color-bg-overlay` | `var(--color-overlay)` |
| `--color-fg-muted` | `var(--color-text-secondary)` |
| `--color-selected-fg` | `var(--color-text-selected)` `[工程扩展]` |
| `--color-ring` | `var(--color-focus)` |
| `--color-success-fill` | `var(--color-strong-fill-success)` `[工程扩展]` |
| `--color-on-success-fill` | `var(--color-strong-on-success)` `[工程扩展]` |
| `--color-success-fg` | `var(--color-status-success)` |
| `--color-warning-fill` | `var(--color-strong-fill-warning)` `[工程扩展]` |
| `--color-on-warning-fill` | `var(--color-strong-on-warning)` `[工程扩展]` |
| `--color-warning-fg` | `var(--color-status-warning)` |
| `--color-destructive-fill` | `var(--color-strong-fill-danger)` `[工程扩展]` |
| `--color-on-destructive-fill` | `var(--color-strong-on-danger)` `[工程扩展]` |
| `--color-destructive-fg` | `var(--color-status-danger)` |
| `--color-info-fill` | `var(--color-strong-fill-info)` `[工程扩展]` |
| `--color-on-info-fill` | `var(--color-strong-on-info)` `[工程扩展]` |
| `--color-info-fg` | `var(--color-status-info)` `[工程扩展]` |

**无需别名的同名 Token（Semantic 区直接用新名定义）：** `--color-border`、`--color-border-strong`、`--color-disabled-fg`、`--color-disabled-bg`（共 4 个同名 Token 在 Semantic 区直接定义，不列入别名）。

> **别名删除条件：** 16 个 Surface 全部迁移并通过 Light/Dark + 中英文 + 视觉回归验收后，删除别名区。形式严格 `--old: var(--new);`，禁止复制第二份值，禁止循环。
