# LinguaRay Design System — Master (Single Source of Truth) (rev-4.2)

> **This is the ONLY design document that production UI must follow.**
>
> - `SKILL-RAW.md` = unmodified skill output, audit evidence only. Do NOT implement from it.
> - `UI-BRIEF.md` = decision rationale (why adopt/override/reject). Does NOT override this file.
> - `token-map.md` = the Penpot → CSS → Legacy alias map. Token names below mirror it verbatim.
> - `pages/<page>.md` = page-specific layout/composition overrides. These may NOT override
>   global tokens, contrast/keyboard rules, component contracts, motion/reduced-motion, or
>   window behavior. Changing those requires editing THIS file first.
>
> All pixel values are **Tauri/CSS logical pixels** (not physical display pixels).

**Project:** LinguaRay
**Date:** 2026-08-08
**Spec:** [Product Baseline (S0 Frozen)](../../docs/superpowers/specs/2026-08-01-linguaray-product-baseline.md) · [R0 Freeze Decision](../../docs/superpowers/specs/2026-08-08-rayline-r0-freeze-decision.md)
**Identity:** Native restraint · Indigo brand · Cyan focus · Privacy-first

---

## 1. Color Tokens

Color is organized in **three sets**: Core primitives (raw scale values), Semantic
Light/Dark (role tokens consumed by components), and engineering extensions (tokens
that Penpot does not define but are required for accessible interaction).

**Core reference rule (§1.6):** Only `tokens.css` may reference `--core-*`. Components
and pages consume Semantic Tokens only.

### 1.0 Core / Primitives — 97 Token

Raw scale values mirror `token-map.md` column 1 verbatim. Naming uses `neutral` (NOT
`slate`); `white`/`black` are independent of the `neutral` scale. **Indigo:
400=`#818CF8`, 500=`#6366F1`, 600=`#4F46E5`, 700=`#4338CA`.**

The full 97-Core Token tables (every value) live in `token-map.md`. This file lists
only the category summary; do NOT duplicate all 97 values here.

| Category | Count | CSS prefix | Notes |
|---|---|---|---|
| colors | 44 | `--core-color-*` | white/black (2) · neutral 50–950 (11) · indigo 50–900 (10) · cyan 50–800 (9) · green/amber/red 50/500/600/700 (12) |
| spacing | 14 | `--core-space-*` | `0/2/4/6/8/10/12/16/20/24/32/40/48/64` |
| radius | 9 | `--core-radius-*` | `0/4/6/8/10/12/16/20/full` |
| border width | 2 | `--core-border-*` | `border.1`/`border.2` |
| opacity | 2 | `--core-opacity-*` | `opacity.disabled`/`opacity.muted` |
| font families | 3 | `--core-font-family-*` | `sans`/`cjk`/`mono` |
| font sizes | 9 | `--core-font-size-*` | `11/12/13/14/16/18/20/24/32` |
| font weights | 4 | `--core-font-weight-*` | `400/500/600/700` |
| typography | 10 | `--core-type-*` | composite tokens (size + weight + line-height) |
| **Total** | **97** | | |

> **Do not fabricate** `color.core.neutral.0` for `color.core.white`. `white`/`black`
> are standalone. For every value see `token-map.md` column 1.

---

### 1.1 Semantic / Light — 28 Token

Mirrors `token-map.md` column 1. The `Contrast` column lists the WCAG ratio for the
token's primary role-pair (text-on-background, fill-on-on-fill, or edge-on-surface).
Values are produced by the automated `contrast.test.ts` script (§1.4).

`shadow.raised` / `shadow.overlay` are top-level shadow tokens (NOT `color.shadow.*`).
`color.overlay` Light = `{color.core.neutral.950}` (a token reference).

| # | Penpot name | CSS Semantic name | Value | Contrast (primary pair) |
|---|---|---|---|---|
| 1 | `color.canvas` | `--color-canvas` | `#F8FAFC` | text.primary 17.063:1 ✅ AA |
| 2 | `color.surface.default` | `--color-surface-default` | `#FFFFFF` | text.primary 17.063:1 ✅ AA |
| 3 | `color.surface.subtle` | `--color-surface-subtle` | `#F1F5F9` | text.primary 14.467:1 ✅ AA |
| 4 | `color.surface.raised` | `--color-surface-raised` | `#FFFFFF` | text.primary 17.063:1 ✅ AA |
| 5 | `color.surface.inverse` | `--color-surface-inverse` | `#0F172A` | text.inverse 17.063:1 ✅ AA |
| 6 | `color.text.primary` | `--color-text-primary` | `#0F172A` | on canvas 17.063:1 ✅ AA |
| 7 | `color.text.secondary` | `--color-text-secondary` | `#475569` | on canvas 7.243:1 ✅ AA |
| 8 | `color.text.tertiary` | `--color-text-tertiary` | `#64748B` | on canvas 4.548:1 ✅ AA |
| 9 | `color.text.disabled` | `--color-text-disabled` | `#94A3B8` | on white 2.564:1 ⚠ decorative only |
| 10 | `color.text.inverse` | `--color-text-inverse` | `#F8FAFC` | on inverse 17.063:1 ✅ AA |
| 11 | `color.brand.default` | `--color-brand-default` | `#4F46E5` | on white 6.288:1 ✅ AA |
| 12 | `color.brand.hover` | `--color-brand-hover` | `#4338CA` | on white 7.903:1 ✅ AA |
| 13 | `color.brand.soft` | `--color-brand-soft` | `#EEF2FF` | text.primary 15.966:1 ✅ AA |
| 14 | `color.accent.default` | `--color-accent-default` | `#06B6D4` | on white 2.428:1 ⚠ decorative only |
| 15 | `color.accent.soft` | `--color-accent-soft` | `#ECFEFF` | text.primary 17.162:1 ✅ AA |
| 16 | `color.border.subtle` | `--color-border-subtle` | `#E2E8F0` | on white 1.233:1 ⚠ decorative |
| 17 | `color.border.default` | `--color-border-default` | `#CBD5E1` | on white 1.485:1 ⚠ decorative |
| 18 | `color.border.strong` | `--color-border-strong` | `#94A3B8` | on white 2.564:1 ⚠ decorative |
| 19 | `color.focus` | `--color-focus` | `#0891B2` | on white 3.682:1 ✅ AA UI (3:1) |
| 20 | `color.success.default` | `--color-status-success` | `#16A34A` | on white 3.296:1 ⚠ see §1.7 |
| 21 | `color.success.soft` | `--color-status-success-soft` | `#F0FDF4` | soft pair 4.791:1 ✅ AA |
| 22 | `color.warning.default` | `--color-status-warning` | `#D97706` | on white 3.186:1 ⚠ see §1.7 |
| 23 | `color.warning.soft` | `--color-status-warning-soft` | `#FFFBEB` | soft pair 4.842:1 ✅ AA |
| 24 | `color.danger.default` | `--color-status-danger` | `#DC2626` | on white 4.829:1 ✅ AA |
| 25 | `color.danger.soft` | `--color-status-danger-soft` | `#FEF2F2` | soft pair 5.915:1 ✅ AA |
| 26 | `color.overlay` | `--color-overlay` | `{color.core.neutral.950}` | scrim (decorative) |
| 27 | `shadow.raised` | `--shadow-raised` | `0 8px 24px -2px #0F172A` | n/a (shadow) |
| 28 | `shadow.overlay` | `--shadow-overlay` | `0 16px 40px -4px #0F172A` | n/a (shadow) |

---

### 1.2 Semantic / Dark — 28 Token

Mirrors `token-map.md` column 1. `color.overlay` Dark = `{color.core.black}`.
`brand.default` = `#818CF8`; `brand.on-fill` (engineering extension, §1.3) = `#0F172A`.

| # | Penpot name | CSS Semantic name | Value | Contrast (primary pair) |
|---|---|---|---|---|
| 1 | `color.canvas` | `--color-canvas` | `#020617` | text.primary 19.281:1 ✅ AA |
| 2 | `color.surface.default` | `--color-surface-default` | `#0F172A` | text.primary 17.063:1 ✅ AA |
| 3 | `color.surface.subtle` | `--color-surface-subtle` | `#1E293B` | text.primary 13.587:1 ✅ AA |
| 4 | `color.surface.raised` | `--color-surface-raised` | `#1E293B` | text.primary 13.587:1 ✅ AA |
| 5 | `color.surface.inverse` | `--color-surface-inverse` | `#F8FAFC` | text.inverse 17.063:1 ✅ AA |
| 6 | `color.text.primary` | `--color-text-primary` | `#F8FAFC` | on canvas 19.281:1 ✅ AA |
| 7 | `color.text.secondary` | `--color-text-secondary` | `#CBD5E1` | on canvas 13.587:1 ✅ AA |
| 8 | `color.text.tertiary` | `--color-text-tertiary` | `#94A3B8` | on canvas 7.868:1 ✅ AA |
| 9 | `color.text.disabled` | `--color-text-disabled` | `#475569` | on surface.default 2.356:1 ⚠ decorative |
| 10 | `color.text.inverse` | `--color-text-inverse` | `#0F172A` | on inverse 17.063:1 ✅ AA |
| 11 | `color.brand.default` | `--color-brand-default` | `#818CF8` | on `#0F172A` on-fill 5.985:1 ✅ AA |
| 12 | `color.brand.hover` | `--color-brand-hover` | `#A5B4FC` | on `#0F172A` 8.955:1 ✅ AA |
| 13 | `color.brand.soft` | `--color-brand-soft` | `#312E81` | text.primary 10.918:1 ✅ AA |
| 14 | `color.accent.default` | `--color-accent-default` | `#22D3EE` | on canvas 11.163:1 ✅ AA |
| 15 | `color.accent.soft` | `--color-accent-soft` | `#164E63` | text.primary 8.711:1 ✅ AA |
| 16 | `color.border.subtle` | `--color-border-subtle` | `#1E293B` | on surface.default 1.220:1 ⚠ decorative |
| 17 | `color.border.default` | `--color-border-default` | `#334155` | on surface.default 1.724:1 ⚠ decorative |
| 18 | `color.border.strong` | `--color-border-strong` | `#475569` | on surface.default 2.356:1 ⚠ decorative |
| 19 | `color.focus` | `--color-focus` | `#22D3EE` | on canvas 11.163:1 ✅ AA |
| 20 | `color.success.default` | `--color-status-success` | `#22C55E` | on `#0F172A` 7.835:1 ✅ AA |
| 21 | `color.success.soft` | `--color-status-success-soft` | `#15803D` | soft pair 4.794:1 ✅ AA |
| 22 | `color.warning.default` | `--color-status-warning` | `#F59E0B` | on `#0F172A` 8.313:1 ✅ AA |
| 23 | `color.warning.soft` | `--color-status-warning-soft` | `#B45309` | soft pair 4.800:1 ✅ AA |
| 24 | `color.danger.default` | `--color-status-danger` | `#EF4444` | on `#0F172A` 4.744:1 ✅ AA |
| 25 | `color.danger.soft` | `--color-status-danger-soft` | `#B91C1C` | soft pair 6.184:1 ✅ AA |
| 26 | `color.overlay` | `--color-overlay` | `{color.core.black}` | scrim (decorative) |
| 27 | `shadow.raised` | `--shadow-raised` | `0 8px 24px -2px #000000` | n/a (shadow) |
| 28 | `shadow.overlay` | `--shadow-overlay` | `0 16px 40px -4px #000000` | n/a (shadow) |

> **shadow/overlay native values:** The table above lists Penpot native token values
> (hex shadow color, NOT rgba semi-transparent). `color.overlay` Light value is
> `{color.core.neutral.950}` (token reference), Dark value is `{color.core.black}`.
> Semi-transparent drop-shadow effects on the visual canvas are render-layer opacity,
> not token values.

---

### 1.3 Engineering Extensions `[工程扩展]`

These tokens do NOT exist in the Penpot file and are required for engineering
implementation. Marked `[工程扩展]`. **Do not claim 1:1 parity until these are
written back to Penpot.** Mirrors `token-map.md` column 2.

| Engineering token | CSS name | Light | Dark | Usage | Reason |
|---|---|---|---|---|---|
| `color.surface.hover` `[工程扩展]` | `--color-surface-hover` | `#F1F5F9` | `#334155` | hover bg | interaction needs its own token |
| `color.surface.selected` `[工程扩展]` | `--color-surface-selected` | `#DBEAFE` | `#1E3A5F` | selected bg | interaction needs its own token |
| `color.text.selected` `[工程扩展]` | `--color-text-selected` | `#1D4ED8` | `#60A5FA` | selected text | paired with surface.selected |
| `color.brand.on-fill` `[工程扩展]` | `--color-brand-on-fill` | `#FFFFFF` | `#0F172A` | brand fill text | WCAG pair (6.288/5.985 ✅) |
| `color.brand.fg` `[工程扩展]` | `--color-brand-fg` | `#4F46E5` | `#818CF8` | canvas brand foreground | distinct from brand.default |
| `color.status.info` `[工程扩展]` | `--color-status-info` | `#2563EB` | `#60A5FA` | info status | Penpot has success/warning/danger but no info |
| `color.status.info.soft` `[工程扩展]` | `--color-status-info-soft` | `#EFF6FF` | `#1E3A8A` | info soft bg | same |
| `color.disabled.bg` `[工程扩展]` | `--color-disabled-bg` | `#F1F5F9` | `#1E293B` | disabled bg | interaction needs its own token |
| `color.border.control` `[工程扩展]` | `--color-border-control` | `#64748B` | `#64748B` | control border | border.strong fails 3:1; controls need a 3:1+ token |
| `color.strong-fill.success` `[工程扩展]` | `--color-strong-fill-success` | `#15803D` | `#15803D` | Banner/Button strong fill | Penpot soft mode not strong enough; success.default #16A34A on white only 3.296:1 ❌ |
| `color.strong-fill.warning` `[工程扩展]` | `--color-strong-fill-warning` | `#B45309` | `#B45309` | same | warning.default #D97706 on white only 3.186:1 ❌ |
| `color.strong-fill.danger` `[工程扩展]` | `--color-strong-fill-danger` | `#DC2626` | `#DC2626` | same | danger.default #EF4444 on white 4.829:1, unified to #DC2626 |
| `color.strong-fill.info` `[工程扩展]` | `--color-strong-fill-info` | `#2563EB` | `#2563EB` | same | info status fill |
| `color.strong-on.success` `[工程扩展]` | `--color-strong-on-success` | `#FFFFFF` | `#FFFFFF` | strong fill text | WCAG pair (5.016:1 ✅) |
| `color.strong-on.warning` `[工程扩展]` | `--color-strong-on-warning` | `#FFFFFF` | `#FFFFFF` | same | WCAG 5.022:1 ✅ |
| `color.strong-on.danger` `[工程扩展]` | `--color-strong-on-danger` | `#FFFFFF` | `#FFFFFF` | same | WCAG 4.829:1 ✅ |
| `color.strong-on.info` `[工程扩展]` | `--color-strong-on-info` | `#FFFFFF` | `#FFFFFF` | same | WCAG 5.169:1 ✅ |
| `color.status.success.fg` `[工程扩展]` | `--color-status-success-fg` | `#15803D` | `#F8FAFC` | StatusBadge soft foreground | success.default #16A34A on success.soft #F0FDF4 only 3.421:1 ❌; Dark softs deep enough → canvas #F8FAFC (4.794:1 ✅) |
| `color.status.warning.fg` `[工程扩展]` | `--color-status-warning-fg` | `#B45309` | `#F8FAFC` | same | warning.default #D97706 on warning.soft #FFFBEB only 3.186:1 ❌; Dark (4.800:1 ✅) |
| `color.status.danger.fg` `[工程扩展]` | `--color-status-danger-fg` | `#B91C1C` | `#F8FAFC` | same | danger #DC2626 on danger.soft #FEF2F2 = 4.415:1 ❌; deeper #B91C1C (Light 5.915:1 / Dark 6.184:1 ✅) |
| `color.status.info.fg` `[工程扩展]` | `--color-status-info-fg` | `#1D4ED8` | `#F8FAFC` | same | info.* absent in Penpot; distinct from strong-fill-info #2563EB (Light 6.158:1 / Dark 9.900:1 ✅) |

**border.control freeze:** `#64748B` in both themes. TextField/Select use this.
`border.strong` (`#94A3B8` Light / `#475569` Dark) stays decorative (fails 3:1).

**Strong-fill unified across themes:** success=`#15803D`, warning=`#B45309`,
danger=`#DC2626`, info=`#2563EB`, on-fill=`#FFFFFF`.

**Penpot write-back gate:** The 21 engineering extension tokens must be written back
to the Penpot file before 1:1 parity can be claimed. Until write-back, this table is
the engineering-extension record.

---

### 1.4 WCAG Contrast Table

Values are computed by an automated WCAG script (NOT hand-filled OFL values). The
script is `contrast.test.ts` and runs in CI.

| Pair | Theme | Ratio | Result |
|---|---|---|---|
| `#4F46E5` brand.default / `#FFFFFF` brand.on-fill | Light | 6.288:1 | ✅ AA |
| `#818CF8` brand.default / `#0F172A` brand.on-fill | Dark | 5.985:1 | ✅ AA |
| `#818CF8` brand.fg / `#020617` canvas | Dark | 6.763:1 | ✅ AA |
| `#0891B2` focus / `#FFFFFF` | Light | 3.682:1 | ✅ AA UI (3:1) |
| `#22D3EE` focus / `#020617` canvas | Dark | 9.879:1 | ✅ AA |
| `#15803D` strong-fill.success / `#FFFFFF` | both | 5.016:1 | ✅ AA |
| `#B45309` strong-fill.warning / `#FFFFFF` | both | 5.022:1 | ✅ AA |
| `#DC2626` strong-fill.danger / `#FFFFFF` | both | 4.829:1 | ✅ AA |
| `#2563EB` strong-fill.info / `#FFFFFF` | both | 5.169:1 | ✅ AA |
| `#64748B` border.control / `#FFFFFF` | Light | 4.759:1 | ✅ AA |
| `#64748B` border.control / `#0F172A` | Dark | 3.751:1 | ✅ AA UI (3:1) |
| `#15803D` status.success.fg / `#F0FDF4` status.success.soft | Light | 4.791:1 | ✅ AA |
| `#B45309` status.warning.fg / `#FFFBEB` status.warning.soft | Light | 4.842:1 | ✅ AA |
| `#B91C1C` status.danger.fg / `#FEF2F2` status.danger.soft | Light | 5.915:1 | ✅ AA |
| `#1D4ED8` status.info.fg / `#EFF6FF` status.info.soft | Light | 6.158:1 | ✅ AA |
| `#F8FAFC` status.*.fg / `#15803D` `#B45309` `#B91C1C` `#1E3A8A` soft | Dark | 4.794 / 4.800 / 6.184 / 9.900 | ✅ AA |

**Accent `#06B6D4` is decorative only** (2.428:1) — never used for text or control edges.

---

### 1.5 Legacy Token Aliases (deprecated)

> **DEPRECATED.** These are migration-only aliases. Mirrors `token-map.md` column 3.

Old → new one-way aliases. Form is strictly `--old: var(--new);`. No duplicate value
copies, no cycles. Delete the alias block after all 16 Surfaces migrate and pass
Light/Dark + Chinese/English + visual regression acceptance.

| Legacy alias name (deprecated) | New Semantic target |
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

**Same-name tokens need no alias** (defined directly in the Semantic section):
`--color-border`, `--color-border-strong`, `--color-disabled-fg`, `--color-disabled-bg`
(4 same-name tokens defined directly in the Semantic section, not listed as aliases).

---

### 1.6 Core Reference Rule

**Only `tokens.css` may reference `--core-*`. Components and pages may only consume
Semantic Tokens.** Any CSS in a component or page that references `--core-*` is a
contract violation. The pre-delivery checklist (§10) includes a check that component
CSS contains no `--core-*` references.

---

### 1.7 Status Roles

- **StatusBadge uses soft mode:**
  - **Light:** `core.700` foreground + `core.50` soft background
    (e.g. success `#15803D` / `#F0FDF4` = 4.791:1 ✅).
  - **Dark:** `--color-text-primary` + Dark soft background
    (e.g. `#F8FAFC` / `#15803D` = 4.794:1 ✅).
- **Forbidden:** success/warning solid fills paired with white text. The
  `success.default`/`warning.default` values do not meet AA on white
  (`#16A34A`/white = 3.296:1 ❌).
- **Banner / Button strong fills:** use the `strong-fill.*` engineering-extension
  tokens (§1.3) with `strong-on.*` white text. The unified strong-fill palette
  (success `#15803D`, warning `#B45309`, danger `#DC2626`, info `#2563EB`) is paired
  with `#FFFFFF` text in both themes.

---

## 2. Typography

Local `woff2` bundle (Plan B, 5 files, 2,393,380 bytes = 2.28 MB). OFL license ships
with source. **No remote font requests.** CSP `default-src 'self'`. `@fontsource`
packages live in `devDependencies` and are vendored (their woff2 outputs are copied
into the source tree at build time, not fetched at runtime). If a font fails to load,
the browser falls back to the system stack declared after each family.

```css
--font-sans: "Inter", "Noto Sans SC", -apple-system, BlinkMacSystemFont,
  "Segoe UI", "Helvetica Neue", "PingFang SC", "Microsoft YaHei",
  "Noto Sans CJK SC", sans-serif;
--font-mono: "IBM Plex Mono", "SF Mono", "Cascadia Code", "Consolas",
  "Noto Sans Mono CJK SC", monospace;
```

Three families ship: **Inter** (latin, sans), **Noto Sans SC** (Chinese-simplified,
CJK), **IBM Plex Mono** (latin, mono). The `--core-font-family-cjk` token exists for
explicit CJK targeting; the `--font-sans` stack already interleaves CJK fallback.

### Font bundle

| File | Size | SHA-256 |
|---|---|---|
| `inter-latin-wght-normal.woff2` | 48,256 | `3100e775…` |
| `ibm-plex-mono-latin-400-normal.woff2` | 14,708 | `08949f72…` |
| `ibm-plex-mono-latin-600-normal.woff2` | 15,620 | `0d1f0b8d…` |
| `noto-sans-sc-chinese-simplified-400-normal.woff2` | 1,142,552 | `95e3633b…` |
| `noto-sans-sc-chinese-simplified-700-normal.woff2` | 1,172,244 | `e1df51ed…` |
| **Total** | **2,393,380 (2.28 MB)** | |

### Type scale — 10 composite Core typography tokens

Mirrors `token-map.md` column 1 (`typography`).

| Penpot name | CSS name | Size | Weight | Line-height | Usage |
|---|---|---|---|---|---|
| `type.display` | `--core-type-display` | 32px | 700 | 1.25 | Onboarding hero, result emphasis |
| `type.title.lg` | `--core-type-title-lg` | 24px | 700 | 1.33 | Onboarding step title |
| `type.title.md` | `--core-type-title-md` | 20px | 600 | 1.4 | Window title |
| `type.title.sm` | `--core-type-title-sm` | 16px | 600 | 1.5 | Section headings in settings |
| `type.body.lg` | `--core-type-body-lg` | 16px | 400 | 1.5 | Large body |
| `type.body.md` | `--core-type-body-md` | 14px | 400 | 1.43 | Body text, input text (default) |
| `type.body.sm` | `--core-type-body-sm` | 12px | 400 | 1.5 | Secondary text, metadata |
| `type.label.md` | `--core-type-label-md` | 13px | 600 | 1.23 | Field labels, badges |
| `type.label.sm` | `--core-type-label-sm` | 11px | 600 | 1.27 | Tiny labels |
| `type.code` | `--core-type-code` | 12px | 500 | 1.5 | Inline code (mono) |

### Font weights — 4 Core tokens

| Penpot name | CSS name | Value |
|---|---|---|
| `font.weight.400` | `--core-font-weight-400` | 400 (regular) |
| `font.weight.500` | `--core-font-weight-500` | 500 (medium, code) |
| `font.weight.600` | `--core-font-weight-600` | 600 (semibold, titles/labels) |
| `font.weight.700` | `--core-font-weight-700` | 700 (bold, display) |

Minimum body size: 12px (metadata only). 13px+ for user-readable content.

### Packaging format

`@fontsource/*` packages are declared as `devDependencies`. A build step vendors the
woff2 files into the source tree (`packages/ui/fonts/`) so the runtime serves them
from the app origin. Inter (variable, latin), IBM Plex Mono (400 + 600 latin), and
Noto Sans SC (400 + 700, Chinese-simplified) make up the 5-file bundle. The OFL
license text for each family is committed alongside the woff2 files.

---

## 3. Spacing, Controls, Radius, Icons

### Spacing (density 8/10 — compact desktop)

14-tier `--core-space-*` scale is the source of truth. The legacy `--space-*` names
below are Semantic aliases mapping onto the Core scale.

| Token (Semantic alias) | Core reference | Value |
|---|---|---|
| `--space-xs` | `--core-space-2` | 2px |
| `--space-sm` | `--core-space-4` | 4px |
| `--space-md` | `--core-space-8` | 8px |
| `--space-lg` | `--core-space-12` | 12px |
| `--space-xl` | `--core-space-16` | 16px |
| `--space-2xl` | `--core-space-24` | 24px |
| `--space-3xl` | `--core-space-32` | 32px |

**Full Core spacing scale (14):** `--core-space-0/2/4/6/8/10/12/16/20/24/32/40/48/64`.

### Control Heights

| Token | Value | Usage |
|---|---|---|
| `--height-sm` | 28px | Compact buttons, badges, small inputs |
| `--height-md` | 32px | Default buttons, icon buttons, inputs |
| `--height-lg` | 36px | Primary action buttons, settings rows |

### Border Radius

9-tier `--core-radius-*` scale is the source of truth. The legacy `--radius-*` names
below are Semantic aliases mapping onto the Core scale.

| Token (Semantic alias) | Core reference | Value | Usage |
|---|---|---|---|
| `--radius-sm` | `--core-radius-6` | 6px | Inputs, badges, tags |
| `--radius-md` | `--core-radius-8` | 8px | Buttons, cards |
| `--radius-lg` | `--core-radius-12` | 12px | Modals, popup window |

**Full Core radius scale (9):** `--core-radius-0/4/6/8/10/12/16/20/full`.

### Icon Sizes

| Token | Value | Usage |
|---|---|---|
| `--icon-sm` | 14px | Inline icons |
| `--icon-md` | 16px | Default (buttons, lists) |
| `--icon-lg` | 20px | Toolbar, prominent |

### Icons

Lucide via `lucide-solid` (SolidJS binding; uses standard outline icons). No filled
variants. No emoji. All icon-only buttons require `aria-label`.

| Action | Lucide name |
|---|---|
| Translate | `Languages` |
| Copy | `Copy` |
| Pin / Unpin | `Pin` / `PinOff` |
| Speak / Stop | `Volume2` / `Square` |
| History | `History` |
| Provider | `Server` |
| Settings | `Settings` |
| Delete | `Trash2` |
| Check | `Check` |
| Error | `AlertTriangle` |
| Loading | `Loader2` (spinning) |
| Search | `Search` |
| Close | `X` |
| Star / Unstar | `Star` / `StarOff` |
| Language | `Globe` |

---

## 4. Shadows

Two shadow tokens, top-level (NOT under `color.`, NOT `sm`/`md`/`lg`). Both themes
use the same token names; the shadow color differs.

### Light

| Token | Value |
|---|---|
| `--shadow-raised` | `0 8px 24px -2px #0F172A` |
| `--shadow-overlay` | `0 16px 40px -4px #0F172A` |

### Dark

| Token | Value |
|---|---|
| `--shadow-raised` | `0 8px 24px -2px #000000` |
| `--shadow-overlay` | `0 16px 40px -4px #000000` |

The legacy `--shadow-sm` / `--shadow-md` / `--shadow-lg` tokens are aliases that map
to `--shadow-raised` / `--shadow-overlay` during the migration window and are removed
once all components move to the two-token model. No new component CSS should use the
three-tier names.

---

## 5. Motion

### Durations & Easing

| Token | Value | Usage |
|---|---|---|
| `--duration-fast` | 120ms | Hover color, toggle |
| `--duration-base` | 180ms | Popup show/hide, expand, card transitions |
| `--duration-slow` | 240ms | Modal open/close, onboarding step |

```css
--ease-standard: cubic-bezier(0.4, 0, 0.2, 1);
--ease-in: cubic-bezier(0.4, 0, 1, 1);
--ease-out: cubic-bezier(0, 0, 0.2, 1);
```

### Reduced Motion

```css
@media (prefers-reduced-motion: reduce) {
  *, *::before, *::after {
    transition-duration: 1ms !important;
    animation-duration: 1ms !important;
    animation-iteration-count: 1 !important;
  }
}
```

**Exception — spinners:** Under reduced-motion, a spinning `Loader2` icon must NOT
freeze after 1ms. Replace with a **static icon + visible "Loading…" text** so the
loading state is still communicated without motion. Implementation: CSS class that
hides the spinner animation and shows the text alternative.

---

## 6. Component States

Every interactive component must implement:

| State | Spec |
|---|---|
| **Default** | Token bg/fg per theme |
| **Hover** | `--color-surface-hover` surface, or `opacity: 0.9` on filled; 120ms transition |
| **Pressed** | `opacity: 0.8` or slightly darker fill |
| **Focus** | `outline: 2px solid var(--color-focus); outline-offset: 2px;` — `:focus-visible` only; never `outline: none` without replacement |
| **Disabled** | `opacity: 0.5; cursor: not-allowed;` — no hover/pressed/focus effects |
| **Loading** | Set native `disabled` + `aria-busy="true"`; preserve accessible name. Show spinner (12px `Loader2`) + visually-hidden "Loading…" text for screen readers. Under reduced-motion: static icon + visible "Loading…" text. `tabindex=-1` is redundant on native disabled elements — do not add it. |
| **Selected** | `--color-surface-selected` bg + `--color-text-selected` text |
| **Destructive** | `--color-strong-fill-danger` + `--color-strong-on-danger`; or `--color-status-danger` text variant |

> **Token reference:** the focus ring uses `--color-focus` (the legacy alias
> `--color-ring` points at it via §1.5). Components must consume `--color-focus`
> directly; do not reference `--color-ring` in new CSS.

---

## 7. Component Contracts

These contracts define the minimum API for `packages/ui` components. Each specifies
variants, sizes, props, and behavior — enough to implement without further design
decisions.

### Button

```
variants:  primary | secondary | ghost | destructive
sizes:     sm (28px) | md (32px) | lg (36px)
props:     disabled, loading, leftIcon, rightIcon, fullWidth
```
- `primary`: `--color-brand-default` bg + `--color-brand-on-fill` text.
- `secondary`: transparent bg + `--color-border-control` border + `--color-text-primary` text.
- `ghost`: transparent bg + `--color-text-primary` text; hover = `--color-surface-hover`.
- `destructive`: `--color-strong-fill-danger` bg + `--color-strong-on-danger`.
- `loading`: per §6 Loading state — native `disabled` + `aria-busy="true"`,
  spinner + visually-hidden "Loading…". Keep width to prevent layout shift.

### IconButton

```
sizes:     sm (28×28) | md (32×32) | lg (36×36)
props:     disabled, loading, aria-label (required), variant
variants:  ghost | primary | destructive
```
- Always 1:1 square. Icon centered. `aria-label` is mandatory (no visible text).
- Same state model as Button.

### TextField

```
sizes:     md (32px) | lg (36px)
props:     label (always visible, not placeholder-only), value, placeholder,
           helperText, errorText, disabled, type, monospace
```
- Border: `--color-border-control` default; `--color-focus` on focus (2px outline + offset).
- `label` is associated via `<label for=id>`; input has matching `id`.
- `errorText` present → `aria-invalid="true"`; border = `--color-status-danger`;
  helper hidden; error text below in `--color-status-danger`; `aria-describedby`
  points to the error text element ID.
- `helperText` present (no error) → `aria-describedby` points to helper text element ID.
- `monospace` → `--font-mono`.

### TextArea `[backlog — 未实现]`

Same as TextField but multi-line. `rows` prop. Min height = 3 rows.

### Select

```
props:     label, value, options, disabled, placeholder
```
- Uses Kobalte `Select` (unstyled). Trigger styled like TextField (border
  `--color-border-control`). Dropdown = `--color-surface-default` + `--shadow-overlay`.

### Checkbox `[backlog — 未实现]`

```
props:     checked, disabled, label, indeterminate
```
- Checked: background = `--color-brand-default`, check icon = `--color-brand-on-fill`.
- Unchecked: border = `--color-border-control`, background = transparent.
- Uses Kobalte `Checkbox`.

### Switch

```
props:     checked, disabled, label
```
- On: `--color-brand-default` track + `--color-brand-on-fill` thumb.
- Off: `--color-border-control` track + `--color-brand-on-fill` thumb
  (same thumb color in both states; Light on-fill white passes 4.548:1).
- Uses Kobalte `Switch`.

### Tabs / SegmentedControl

```
props:     value, tabs: {value, label, icon?}[], onChange
```
- Active tab: `--color-surface-selected` bg + `--color-text-selected` text.
- Inactive: transparent + `--color-text-secondary`.
- Segmented = pill container with `--color-border-subtle` border.

### Dialog / Confirm

```
props:     open, title, description, children, footer, onClose
Confirm:   title, message, confirmLabel, cancelLabel, variant (primary|destructive), onConfirm, onCancel
```
- Overlay: `--color-overlay`. Dialog: `--color-surface-default` + `--radius-lg` + `--shadow-overlay`.
- Close on `Esc`. Focus trap inside dialog.
- **Focus management:** on open, focus moves into dialog. On close, focus **restores
  to the trigger element** that opened the dialog.
- **Destructive Confirm:** initial focus lands on **Cancel** (not Confirm) to
  prevent accidental destructive action via Enter key.
- Uses Kobalte `Dialog`.

### Banner / Toast

```
Banner:    variant (info|success|warning|destructive), title, description, action?, onDismiss?
Toast:     variant, message, duration?, onDismiss
```
- Banner = full-width, top of content area. Toast = bottom-right.
- **Banner/Button strong fills** use the `strong-fill.*` engineering-extension
  tokens (§1.3) with `strong-on.*` white text (success `#15803D` / warning `#B45309`
  / danger `#DC2626` / info `#2563EB`). StatusBadge uses soft mode (§1.7), NOT strong fill.
- **Toast roles:** info/success → `role="status"`; warning/destructive → `role="alert"`.
- **Toast auto-dismiss:** info/success/warning auto-dismiss after 3s (default).
  **Destructive toasts do NOT auto-dismiss** — require explicit user dismissal.

### Tooltip

```
props:     content, children, side (top|bottom|left|right)
```
- Uses Kobalte `Tooltip`. `--color-surface-default` bg + `--color-text-primary` text. `--text-sm`. Max width 240px.

### Card `[backlog — 未实现]`

```
props:     children, padding (default md), interactive, onClick
```
- `--color-surface-default` + `--radius-md` + `--shadow-raised`.
- Interactive (has `onClick`, no nested buttons/links): render as `<button>` (or
  `<a>` if navigates). Tabbable. Enter/Space triggers `onClick`. Hover =
  `--shadow-overlay`; `cursor: pointer`. `:focus-visible` ring per §6. NOT a bare
  `<div onClick>`.
- Non-interactive (has nested buttons/links): render as `<div>`; inner elements
  handle their own interaction.

### ListRow

```
props:     leading (icon|avatar), title, subtitle, trailing (badge|action), onClick?
```
- **Single-line:** height 36px. Title only.
- **Two-line:** height 52px minimum. Title (14px) + subtitle (12px `--color-text-secondary`).
- Padding: `--space-md` horizontal.
- **No trailing action + has `onClick`:** entire row renders as `<button>`. Full-row
  click target. Tab + Enter supported.
- **Has trailing action:** row renders as non-interactive `<div>`. The title/leading
  area is a separate `<button>` (the primary action); the trailing action is its own
  `<button>`. No nested interactive elements inside a button.
- Selected: `--color-surface-selected` bg + `--color-text-selected` text.
- R3b promotion: exported by `@linguaray/ui`; Surface 07 uses the shared
  implementation rather than a page-local row.

### ProviderCard

```
props:     profile (name, template, status), hasKey, isActive, onEdit, onDelete, onToggle
```
- Card layout: name + template badge + key status indicator + enabled switch.
- Active provider: `--color-surface-selected` accent border-left (3px `--color-text-selected`).

### ResultCard

```
props:     engineId, engineLabel, text, elapsedMs, outcome (success|failure), errorText, actions (copy, speak, pin, favorite)
```
- Success: text in `--text-base`; engine label in `--text-xs` `--color-text-secondary`.
- Failure: `--color-status-danger` error text.
- Actions row: IconButton row at bottom (ghost variant).

### Spinner / EmptyState

```
Spinner:   size (12|16|20), label (required, defaults "Loading…")
EmptyState: icon, title, description, action?
```
- Spinner: `Loader2` icon + CSS spin animation. `label` provides the accessible name
  and the visible reduced-motion text. Under reduced-motion: hide spinner, show the
  `label` text.
- EmptyState: centered icon (32px, `--color-text-secondary`) + title (`--text-md`) +
  description (`--text-sm`, `--color-text-secondary`).

### SegmentedControl

```
props:     value, segments: {value, label, icon?}[], onChange, ariaLabel (required), disabled
```
- Roving tabindex model: exactly one segment holds `tabindex="0"` (the active one),
  all others hold `tabindex="-1"`.
- Keyboard: `ArrowLeft` / `ArrowRight` (horizontal) and `ArrowUp` / `ArrowDown`
  (vertical) move selection; `Home` selects the first segment; `End` selects the last.
  Activation updates both the value and the roving tabindex position in one step.
- Active segment: `--color-surface-selected` bg + `--color-text-selected` text.
  Inactive: transparent + `--color-text-primary`.
- Container: pill, `--color-border-subtle` border, `--radius-full`.
- `ariaLabel` is required — the group needs an accessible name (`role="radiogroup"` +
  `aria-label`, each segment `role="radio"` + `aria-checked`).

### ShortcutChip

```
props:     shortcut: string, status: "recording" | "conflict" | "clear", labels: { recording: string, conflict: string, clear: string }
```
- Displays a keyboard shortcut as a chip. `status` drives the appearance:
  - `clear` — default neutral chip.
  - `recording` — focus-ring style (`--color-focus` border) while waiting for input.
  - `conflict` — `--color-status-danger` text/border, plus the conflict reason text
    from `labels.conflict`.
- `labels` is required and provides the localized strings for each status (e.g.
  `labels.recording` = "Press a key combination…", `labels.conflict` =
  "Conflicts with {x}", `labels.clear` = a screen-reader-friendly description of the
  current shortcut).

### StatusBadge

```
props:     variant: "success" | "warning" | "danger" | "info" | "neutral",
           children, dot?: boolean, icon?: LucideIcon
```
- Always soft mode (§1.7): never strong fill.
  - **Light:** `core.700` foreground + `core.50` soft background.
  - **Dark:** `--color-text-primary` foreground + Dark soft background.
- Variant → soft-pair source:
  - success → `color.core.green.700` / `color.core.green.50`
  - warning → `color.core.amber.700` / `color.core.amber.50`
  - danger → `color.core.red.700` / `color.core.red.50`
  - info → `color.core.indigo.700` / `color.core.indigo.50`
  - neutral → `color.core.neutral.700` / `color.core.neutral.100`
- `dot` renders a leading colored dot (8px, filled with the variant's `*.700` core
  color) to reinforce status at a glance.
- Success/warning **must not** use solid fills with white text (fails per §1.7).

### InlineError

```
props:     children, id?: string
```
- Renders with `role="alert"`. When used as a field error message, the field sets
  `aria-invalid="true"` and `aria-describedby` pointing at the InlineError `id`.
- `id` is optional but should be supplied whenever the message must be associated
  with a control via `aria-describedby`.
- `variant="danger"` (default) uses `--color-status-danger`;
  `variant="warning"` uses the semantic warning foreground/soft pair. Both use
  `--text-sm`; warning is used for recoverable OS shortcut-registration failure.

### WindowChrome

```
props:     title: string, sidebar: Component, children: Component,
           labels: { minimize: string, close: string }
```
- Renders the OS-window frame: a draggable title bar (with the window `title` and the
  minimize/close buttons), a `sidebar` slot, and a `children` content slot.
- The title bar is the drag region (`data-tauri-drag-region`); the minimize/close
  buttons opt out of the drag region so they receive clicks.
- `labels.minimize` and `labels.close` are required accessible names for the
  window-control IconButtons (no visible text).
- Close = `X` icon; minimize = `Minus` icon. Both use the ghost IconButton variant.

### SidebarItem

```
props:     icon: LucideIcon, label: string, active?: boolean, onClick?: () => void, disabled?: boolean
```
- Renders as a native `<button>`. Enter/Space activate it natively (do not wire a
  custom key handler that shadows the native behavior).
- `active` sets `aria-current="page"` (not `aria-pressed`) and applies the selected
  treatment: `--color-surface-selected` bg + `--color-text-selected` text + a leading
  accent bar.
- Inactive item: transparent bg + `--color-text-primary`; hover = `--color-surface-hover`.
- `label` is both the visible text and the accessible name.

### HistoryRow

```
props:     preview: string, engineLabel?: string, elapsedMs?: number, favorite?: boolean,
           onToggleFavorite?: () => void, onClick?: () => void,
           labels: { addFavorite: string, removeFavorite: string }
```
- **No `onClick` →** render as a non-interactive `<div>` (no role, no tabindex). It is
  a layout row, not a button.
- **Has `onClick` →** the preview area renders as a `<button>` covering the row;
  Tab + Enter/Space are supported.
- **Favorite action:** always a separate `<button>` at the trailing edge, using the
  Star/StarOff icon. Its accessible name comes from `labels.addFavorite` /
  `labels.removeFavorite` depending on the current `favorite` state.
- **No nested interactive controls inside the row button.** When the row itself is a
  button (has `onClick`), the favorite toggle must sit outside the row button in the
  DOM — the row splits into a primary-action button and a trailing action button
  inside a non-interactive container. Never nest a `<button>` inside another `<button>`.

### ProviderRow

```
props:     profile: ProviderProfile, enabled: boolean,
           onToggle: (enabled: boolean) => void, onEdit: () => void, onDelete: () => void,
           labels: { edit: string, delete: string, enabled: string, statusText: (s: ProviderStatus) => string }
```
- Shared visual model lives in `providerPresentation` (icon + name + template badge +
  key-status indicator) so ProviderCard and ProviderRow stay in sync.
- The toggle emits `onToggle(enabled: boolean)` — the new state, not a raw event — so
  the parent does not need to read the Switch to know the intent.
- `onEdit` / `onDelete` are separate IconButtons; their accessible names come from
  `labels.edit` / `labels.delete`.
- `labels.statusText` maps a `ProviderStatus` to a localized status string used as
  the StatusBadge label and the row's accessible description.
- Active provider accent: `--color-surface-selected` border-left (3px).

### TranslationCard

```
type TranslationState =
  | { kind: "loading" }
  | { kind: "success"; text: string; elapsedMs: number }
  | { kind: "failure"; errorText: string }

props:     engineId, engineLabel, state: TranslationState,
           actions: { copy, speak, pin, favorite },
           labels: { loadingLabel: string, failureText: string, retryLabel: string }
```
- The `state` prop is a discriminated union — the component renders differently per
  `kind`. The success/failure action row only renders on `success`; the failure retry
  control only renders on `failure`; the loading treatment only renders on `loading`.
- `labels.loadingLabel`, `labels.failureText`, and `labels.retryLabel` are all
  required (no hardcoded English).
- **Loading:** Spinner (with `labels.loadingLabel` as its accessible name) replaces
  the result text area. No action row while loading.
- **Failure:** `labels.failureText` introduces the error; the error itself renders in
  `--color-status-danger`. The retry control is a Button (primary) with
  `labels.retryLabel`; its `onRetry` is the Button's own press handler (the Button is
  the retry affordance — do not wrap it in an extra clickable `<div>`).
- **Success:** `text` in `--text-base`; engine label in `--text-xs`
  `--color-text-secondary`; elapsed time as metadata. Actions row (IconButton row,
  ghost variant) at the bottom.

---

## 8. Windows

All sizes are Tauri/CSS logical pixels.

### 8.1 Window Inventory & Constraints

| Window | Default | Minimum | Max | Resizable | Z-order |
|---|---|---|---|---|---|
| Settings (main) | 800×600 | 600×400 | — | ✅ | Normal |
| Selection popup | Auto | 200×40 | 400×300 (single) / 600×400 (expanded) | ❌ | Always-on-top |
| Input window | 420×280 | 360×200 | — | ✅ | Always-on-top |
| OCR overlay | Full-screen per monitor | — | — | ❌ | Above all |

### 8.2 Popup Mode Switching

- **Single result:** popup max = 400×300.
- **Multi-engine (expanded):** popup max changes to 600×400 via Tauri `setSize`/
  `setMaxSize` when entering expanded mode, and reverts when leaving.
- **Multi-engine layout:** `ResultCard`s shown **side-by-side** (per S0 spec
  §2.1 "results from N providers shown side-by-side"), in provider sort order.
  Cards do not jump position as results arrive.
  - 2 providers: 2 columns, each min 200px wide.
  - 3+ providers: horizontal scroll if total width exceeds popup max.
  - Each card is independently scrollable vertically if its text overflows.
- **Work-area clamping (both single and expanded):**
  - `width = min(maxWidth, workArea.width - margin×2)` where margin = 8px.
  - `height = min(maxHeight, workArea.height - margin×2)`.
  - `x = clamp(x, workArea.x + margin, workArea.x + workArea.width - width - margin)`.
  - `y = clamp(y, workArea.y + margin, workArea.y + workArea.height - height - margin)`.
  - All four edges clamped, not just the right. The native window never extends
    beyond the screen work area. If available width is less than two 200px cards,
    horizontal internal scroll handles overflow within the clamped window.
- **Pinned:** popup stays visible regardless of blur. Only unpinned popups hide on blur.

### 8.3 Settings Adaptive Behavior

Min width = 600px (enforced by Tauri). Within 600–800px:

- **≥ 700px:** full sidebar (labels + icons) + content side-by-side.
- **600–699px:** sidebar collapses to icon-only rail; labels appear as tooltips on hover.
- **No hamburger menu path.** Min is 600px; if user needs more, they resize.
- No horizontal overflow on the window. Internal scroll areas (tables, long lists) are contained with `overflow-x: auto`.

### 8.4 OCR Overlay

- Full-screen transparent overlay on each monitor.
- User drags to select a rectangle.
- `Esc` or right-click cancels.
- After selection: overlay hides, capture proceeds.
- No visible window chrome, title bar, or taskbar entry.

### 8.5 Onboarding

- Target window: 600×400 (same as Settings min).
- Single-column flow with steps. Not resizable below 600×400.

---

## 9. Page Overrides Boundary

`pages/<page>.md` files may override:

- ✅ Layout structure (grid, flex direction, sidebar arrangement)
- ✅ Component composition (which components, in what order)
- ✅ Content guidance (copy, labels, empty-state text)
- ✅ Density adjustments for that page (using existing spacing tokens to adjust layout gaps; does NOT change component sizes or control heights)

`pages/<page>.md` files may NOT override:

- ❌ Global color tokens (§1)
- ❌ Contrast and keyboard accessibility rules (§1, §6, §10)
- ❌ Component contracts (§7)
- ❌ Motion / reduced-motion rules (§5)
- ❌ Window base behavior (§8)

Changing any of these requires editing MASTER.md first.

---

## 10. Pre-Delivery Checklist

- [ ] All color fill+text pairs pass WCAG AA (≥4.5:1) — see §1.4 contrast table
- [ ] All non-decorative UI elements carrying state/boundary identification (borders on inputs, switch tracks, focus rings) pass 3:1 minimum
- [ ] Light AND dark themes verified
- [ ] Focus rings visible on every interactive element (`:focus-visible`)
- [ ] Full keyboard navigation (Tab order = visual order; Enter/Esc on modals/inputs)
- [ ] `prefers-reduced-motion` respected (transitions → 1ms; spinners → static + text)
- [ ] Icon-only buttons have `aria-label`
- [ ] No runtime remote font requests
- [ ] WCAG contrast verified by an automated script (`contrast.test.ts`)
- [ ] Component CSS contains no `--core-*` references
- [ ] No horizontal overflow on any window
- [ ] All component states implemented (hover, pressed, focus, disabled, loading, selected, destructive)
- [ ] CJK text renders correctly in both themes
