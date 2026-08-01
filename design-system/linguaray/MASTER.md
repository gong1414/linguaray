# LinguaRay Design System — Master (Single Source of Truth)

> **This is the ONLY design document that production UI must follow.**
>
> - `SKILL-RAW.md` = unmodified skill output, audit evidence only. Do NOT implement from it.
> - `UI-BRIEF.md` = decision rationale (why adopt/override/reject). Does NOT override this file.
> - `pages/<page>.md` = page-specific layout/composition overrides. These may NOT override
>   global tokens, contrast/keyboard rules, component contracts, motion/reduced-motion, or
>   window behavior. Changing those requires editing THIS file first.
>
> All pixel values are **Tauri/CSS logical pixels** (not physical display pixels).

**Project:** LinguaRay
**Date:** 2026-08-01
**Spec:** [Product Baseline (S0 Frozen)](../../docs/superpowers/specs/2026-08-01-linguaray-product-baseline.md)
**Identity:** Native restraint · Beam blue · Compact density · Privacy-first

---

## 1. Color Tokens

### Naming convention

Tokens are role-symmetric across themes. Fill tokens are for backgrounds of filled
elements (buttons, badges). FG tokens are for text/icons on the app background.
A single token never means "fill" in one theme and "text" in the other.

```
--color-{role}-fill       Background of a filled button/badge/banner
--color-on-{role}-fill    Text/icon color on top of that fill
--color-{role}-fg         Text/icon color on the app background (not on a fill)
```

### 1.1 Light Theme

| Token | Hex | On-Token | Contrast | Role |
|---|---|---|---|---|
| `--color-primary-fill` | `#2563EB` | `--color-on-primary-fill: #FFFFFF` | 5.169:1 ✅ | Filled primary buttons |
| `--color-primary-fg` | `#2563EB` | — | on bg: 5.169:1 ✅ | Primary text/icons, active tab |
| `--color-link` | `#1D4ED8` | — | on bg: 6.702:1 ✅ | Hyperlinks |
| `--color-success-fill` | `#15803D` | `--color-on-success-fill: #FFFFFF` | 5.016:1 ✅ | Success badges/banners |
| `--color-success-fg` | `#15803D` | — | on bg: 5.016:1 ✅ | "Key saved ✓", connected icons |
| `--color-destructive-fill` | `#B91C1C` | `--color-on-destructive-fill: #FFFFFF` | 6.470:1 ✅ | Delete buttons |
| `--color-destructive-fg` | `#B91C1C` | — | on bg: 6.470:1 ✅ | Error text/icons |
| `--color-warning-fill` | `#B45309` | `--color-on-warning-fill: #FFFFFF` | 5.022:1 ✅ | Warning banners |
| `--color-warning-fg` | `#B45309` | — | on bg: 5.022:1 ✅ | Warning text/icons |
| `--color-info-fill` | `#1D4ED8` | `--color-on-info-fill: #FFFFFF` | 6.702:1 ✅ | Info banners |
| `--color-info-fg` | `#1D4ED8` | — | on bg: 6.702:1 ✅ | Info text/icons |
| `--color-bg` | `#FFFFFF` | `--color-fg: #0F172A` | 17.853:1 ✅ | App background |
| `--color-bg-elevated` | `#F8FAFC` | `--color-fg-elevated: #0F172A` | 17.201:1 ✅ | Cards, popups |
| `--color-bg-hover` | `#F1F5F9` | — | — | Hover surface |
| `--color-bg-selected` | `#DBEAFE` | `--color-selected-fg: #1D4ED8` | 5.493:1 ✅ | Selected item |
| `--color-bg-overlay` | `rgba(0,0,0,0.4)` | — | — | Modal overlay |
| `--color-fg-muted` | `#475569` | on bg: 7.578:1 ✅ | — | Secondary text |
| `--color-border` | `#E2E8F0` | on bg: 1.233:1 | Decorative only — NOT sufficient for input edge |
| `--color-border-strong` | `#64748B` | on bg: 4.759:1 ✅ (3:1 UI) | Input borders, focusable dividers |
| `--color-ring` | `#2563EB` | — | — | Focus ring (opaque) |
| `--color-disabled-fg` | `#94A3B8` | — | — | Disabled text/icons |
| `--color-disabled-bg` | `#F1F5F9` | — | — | Disabled surface |

### 1.2 Dark Theme

| Token | Hex | On-Token | Contrast | Role |
|---|---|---|---|---|
| `--color-primary-fill` | `#2563EB` | `--color-on-primary-fill: #FFFFFF` | 5.169:1 ✅ | Filled primary buttons (same as light) |
| `--color-primary-fg` | `#60A5FA` | — | on bg: 7.022:1 ✅ | Primary text/icons, active tab |
| `--color-link` | `#60A5FA` | — | on bg: 7.022:1 ✅ | Hyperlinks |
| `--color-success-fill` | `#166534` | `--color-on-success-fill: #FFFFFF` | 7.130:1 ✅ | Success badges/banners |
| `--color-success-fg` | `#4ADE80` | — | on bg: 10.245:1 ✅ | Success text/icons |
| `--color-destructive-fill` | `#991B1B` | `--color-on-destructive-fill: #FFFFFF` | 8.310:1 ✅ | Destructive filled buttons |
| `--color-destructive-fg` | `#F87171` | — | on bg: 6.454:1 ✅ | Error text/icons |
| `--color-warning-fill` | `#92400E` | `--color-on-warning-fill: #FFFFFF` | 7.090:1 ✅ | Warning banners |
| `--color-warning-fg` | `#FBBF24` | — | on bg: 10.694:1 ✅ | Warning text/icons |
| `--color-info-fill` | `#1E40AF` | `--color-on-info-fill: #FFFFFF` | 9.130:1 ✅ | Info banners |
| `--color-info-fg` | `#60A5FA` | — | on bg: 7.022:1 ✅ | Info text/icons |
| `--color-bg` | `#0F172A` | `--color-fg: #F1F5F9` | 16.296:1 ✅ | App background |
| `--color-bg-elevated` | `#1E293B` | `--color-fg-elevated: #F1F5F9` | 12.697:1 ✅ | Cards, popups |
| `--color-bg-hover` | `#334155` | — | — | Hover surface |
| `--color-bg-selected` | `#1E3A5F` | `--color-selected-fg: #60A5FA` | 4.524:1 ✅ | Selected item |
| `--color-bg-overlay` | `rgba(0,0,0,0.6)` | — | — | Modal overlay |
| `--color-fg-muted` | `#94A3B8` | on bg: 6.963:1 ✅ | — | Secondary text |
| `--color-border` | `rgba(255,255,255,0.08)` | — | Decorative only |
| `--color-border-strong` | `#64748B` | on bg: 3.751:1 ✅ (3:1 UI) | Input borders, focusable dividers |
| `--color-ring` | `#60A5FA` | — | — | Focus ring (opaque) |
| `--color-disabled-fg` | `#475569` | — | — | Disabled text/icons |
| `--color-disabled-bg` | `#1E293B` | — | — | Disabled surface |

### 1.3 Semantic Rules

- **Filled buttons** use `*-fill` + `on-*-fill`. **Text/icons on app bg** use `*-fg`.
- **Border** (`--color-border`) is decorative only (dividers, card edges). For input
  field edges and focusable separators, use `--color-border-strong` (passes 3:1 non-text UI).
- **Focus ring** is opaque, never `rgba(...,20)`: `outline: 2px solid var(--color-ring); outline-offset: 2px;`
- **Disabled** = `opacity: 0.5` + `cursor: not-allowed` + no hover/pressed.
- **Selected** = `--color-bg-selected` background + `--color-selected-fg` text (both pass AA).

---

## 2. Typography

```css
--font-sans: -apple-system, BlinkMacSystemFont, "Segoe UI", "Helvetica Neue",
  "PingFang SC", "Microsoft YaHei", "Noto Sans CJK SC", sans-serif;
--font-mono: "SF Mono", "Cascadia Code", "Consolas", "Noto Sans Mono CJK SC",
  monospace;
```

No remote fonts. Privacy-first. CJK fallbacks built in.

| Token | Size | Line-height | Weight | Usage |
|---|---|---|---|---|
| `--text-xs` | 12px | 16px | 400 | Timestamps, engine labels, metadata |
| `--text-sm` | 13px | 18px | 400 | Secondary text, helper text |
| `--text-base` | 14px | 20px | 400 | Body text, input text (default) |
| `--text-md` | 16px | 24px | 500 | Section headings in settings |
| `--text-lg` | 20px | 28px | 600 | Window title, onboarding step |
| `--text-xl` | 24px | 32px | 700 | Onboarding hero, result emphasis |

Minimum body: 12px (metadata only). 13px+ for user-readable content.

---

## 3. Spacing, Controls, Radius, Icons

### Spacing (density 8/10 — compact desktop)

| Token | Value |
|---|---|
| `--space-xs` | 2px |
| `--space-sm` | 4px |
| `--space-md` | 8px |
| `--space-lg` | 12px |
| `--space-xl` | 16px |
| `--space-2xl` | 24px |
| `--space-3xl` | 32px |

### Control Heights

| Token | Value | Usage |
|---|---|---|
| `--height-sm` | 28px | Compact buttons, badges, small inputs |
| `--height-md` | 32px | Default buttons, icon buttons, inputs |
| `--height-lg` | 36px | Primary action buttons, settings rows |

### Border Radius

| Token | Value | Usage |
|---|---|---|
| `--radius-sm` | 6px | Inputs, badges, tags |
| `--radius-md` | 8px | Buttons, cards |
| `--radius-lg` | 12px | Modals, popup window |

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

### Light

| Token | Value |
|---|---|
| `--shadow-sm` | `0 1px 2px rgba(0,0,0,0.05)` |
| `--shadow-md` | `0 4px 6px rgba(0,0,0,0.08)` |
| `--shadow-lg` | `0 10px 15px rgba(0,0,0,0.1)` |

### Dark

| Token | Value |
|---|---|
| `--shadow-sm` | `0 1px 2px rgba(0,0,0,0.3)` |
| `--shadow-md` | `0 4px 6px rgba(0,0,0,0.4)` |
| `--shadow-lg` | `0 10px 15px rgba(0,0,0,0.5)` |

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
| **Hover** | `--color-bg-hover` surface, or `opacity: 0.9` on filled; 120ms transition |
| **Pressed** | `opacity: 0.8` or slightly darker fill |
| **Focus** | `outline: 2px solid var(--color-ring); outline-offset: 2px;` — `:focus-visible` only; never `outline: none` without replacement |
| **Disabled** | `opacity: 0.5; cursor: not-allowed;` — no hover/pressed/focus effects |
| **Loading** | Replace button content with spinner (12px `Loader2`) + `pointer-events: none`. Under reduced-motion: static icon + "Loading…" text. |
| **Selected** | `--color-bg-selected` bg + `--color-selected-fg` text |
| **Destructive** | `--color-destructive-fill` + `--color-on-destructive-fill`; or `--color-destructive-fg` text variant |

---

## 7. Component Contracts

These contracts define the minimum API for `packages/ui` components in S1b. Each
specifies variants, sizes, props, and behavior — enough to implement without
further design decisions.

### Button

```
variants:  primary | secondary | ghost | destructive
sizes:     sm (28px) | md (32px) | lg (36px)
props:     disabled, loading, leftIcon, rightIcon, fullWidth
```
- `primary`: `--color-primary-fill` bg + `--color-on-primary-fill` text.
- `secondary`: transparent bg + `--color-border-strong` border + `--color-fg` text.
- `ghost`: transparent bg + `--color-fg` text; hover = `--color-bg-hover`.
- `destructive`: `--color-destructive-fill` bg + `--color-on-destructive-fill`.
- `loading`: replace children with 12px spinner (or static icon + "Loading…" under reduced-motion); `pointer-events: none`; keep width to prevent layout shift.

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
- Border: `--color-border-strong` default; `--color-ring` on focus (2px outline + offset).
- `errorText` present → border = `--color-destructive-fg`; helper hidden; error text below in `--color-destructive-fg`.
- `monospace` → `--font-mono`.

### TextArea

Same as TextField but multi-line. `rows` prop. Min height = 3 rows.

### Select

```
props:     label, value, options, disabled, placeholder
```
- Uses Kobalte `Select` (unstyled). Trigger styled like TextField. Dropdown = `--color-bg-elevated` + `--shadow-md`.

### Checkbox

```
props:     checked, disabled, label, indeterminate
```
- Uses Kobalte `Checkbox`. Check icon = `--color-primary-fg` on checked. Box border = `--color-border-strong`.

### Switch

```
props:     checked, disabled, label
```
- On: `--color-primary-fill` track. Off: `--color-border` track.
- Uses Kobalte `Switch`.

### Tabs / SegmentedControl

```
props:     value, tabs: {value, label, icon?}[], onChange
```
- Active tab: `--color-bg-selected` bg + `--color-selected-fg` text.
- Inactive: transparent + `--color-fg-muted`.
- Segmented = pill container with `--color-border` border.

### Dialog / Confirm

```
props:     open, title, description, children, footer, onClose
Confirm:   title, message, confirmLabel, cancelLabel, variant (primary|destructive), onConfirm, onCancel
```
- Overlay: `--color-bg-overlay`. Dialog: `--color-bg-elevated` + `--radius-lg` + `--shadow-lg`.
- Close on `Esc`. Focus trap inside dialog. First focusable = initial focus.

### Banner / Toast

```
Banner:    variant (info|success|warning|destructive), title, description, action?, onDismiss?
Toast:     variant, message, duration (default 3s), onDismiss
```
- Banner = full-width, top of content area. Toast = bottom-right, auto-dismiss.
- Colors: `*-fill` bg + `on-*-fill` text.

### Tooltip

```
props:     content, children, side (top|bottom|left|right)
```
- Uses Kobante `Tooltip`. `--color-bg-elevated` bg + `--color-fg` text. `--text-sm`. Max width 240px.

### Card

```
props:     children, padding (default md), interactive, onClick
```
- `--color-bg-elevated` + `--radius-md` + `--shadow-sm`.
- Interactive: hover = `--shadow-md`; cursor: pointer.

### ListRow

```
props:     leading (icon|avatar), title, subtitle, trailing (badge|action), onClick
```
- Height: `--height-lg` (36px). Padding: `--space-md` horizontal.
- Selected: `--color-bg-selected` bg.

### ProviderCard

```
props:     profile (name, template, status), hasKey, isActive, onEdit, onDelete, onToggle
```
- Card layout: name + template badge + key status indicator + enabled switch.
- Active provider: `--color-bg-selected` accent border-left (3px `--color-primary-fill`).

### ResultCard

```
props:     engineId, engineLabel, text, elapsedMs, outcome (success|failure), errorText, actions (copy, speak, pin, favorite)
```
- Success: text in `--text-base`; engine label in `--text-xs` `--color-fg-muted`.
- Failure: `--color-destructive-fg` error text.
- Actions row: IconButton row at bottom (ghost variant).

### Spinner / EmptyState

```
Spinner:   size (12|16|20)
EmptyState: icon, title, description, action?
```
- Spinner: `Loader2` icon + CSS spin animation. Under reduced-motion: hide spinner, show "Loading…" text.
- EmptyState: centered icon (32px, `--color-fg-muted`) + title (`--text-md`) + description (`--text-sm`, `--color-fg-muted`).

---

## 8. Windows

All sizes are Tauri/CSS logical pixels.

### 8.1 Window Inventory & Constraints

| Window | Default | Minimum | Max | Resizable | Z-order |
|---|---|---|---|---|---|
| Settings (main) | 800×600 | 600×400 | — | ✅ | Normal |
| Selection popup | Auto | 200×40 | 400×300 (single) / 480×400 (expanded) | ❌ | Always-on-top |
| Input window | 420×280 | 360×200 | — | ✅ | Always-on-top |
| OCR overlay | Full-screen per monitor | — | — | ❌ | Above all |

### 8.2 Popup Mode Switching

- **Single result:** popup max = 400×300.
- **Multi-engine (expanded):** popup max changes to 480×400 via Tauri `setSize`/
  `setMaxSize` when entering expanded mode, and reverts when leaving.
- **Multi-engine layout:** stacked `ResultCard`s in provider sort order, internal
  vertical scroll. NOT tabs. Cards do not jump position as results arrive.
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
- ✅ Density adjustments for that page (tighter/looser spacing)

`pages/<page>.md` files may NOT override:

- ❌ Global color tokens (§1)
- ❌ Contrast and keyboard accessibility rules (§6 states, §10 checklist)
- ❌ Component contracts (§7)
- ❌ Motion / reduced-motion rules (§5)
- ❌ Window base behavior (§8)

Changing any of these requires editing MASTER.md first.

---

## 10. Pre-Delivery Checklist

- [ ] All color fill+text pairs pass WCAG AA (≥4.5:1) — see §1 contrast column
- [ ] Non-text UI (borders, focus rings) pass 3:1 minimum
- [ ] Light AND dark themes verified
- [ ] Focus rings visible on every interactive element (`:focus-visible`)
- [ ] Full keyboard navigation (Tab order = visual order; Enter/Esc on modals/inputs)
- [ ] `prefers-reduced-motion` respected (transitions → 1ms; spinners → static + text)
- [ ] Icon-only buttons have `aria-label`
- [ ] No remote font requests
- [ ] No horizontal overflow on any window
- [ ] All component states implemented (hover, pressed, focus, disabled, loading, selected, destructive)
- [ ] CJK text renders correctly in both themes
