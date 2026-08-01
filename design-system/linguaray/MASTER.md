# LinguaRay Design System — Master (Single Source of Truth)

> **This is the ONLY design document that production UI must follow.**
> `SKILL-RAW.md` is the unmodified skill output kept for audit only — do NOT
> implement from it. `UI-BRIEF.md` records the rationale for adopt/override/reject
> decisions but does NOT override this file. Page-specific overrides live in
> `pages/<page>.md` and take precedence over this file for that page only.

**Project:** LinguaRay
**Date:** 2026-08-01
**Spec:** [Product Baseline (S0 Frozen)](../../docs/superpowers/specs/2026-08-01-linguaray-product-baseline.md)
**Identity:** Native restraint · Beam blue · Compact density · Privacy-first

---

## 1. Color Tokens

### 1.1 Light Theme

| Token | Hex | On-Token | Contrast | Role |
|---|---|---|---|---|
| `--color-primary` | `#2563EB` | `--color-on-primary: #FFFFFF` | 5.17:1 ✅ | Filled buttons, active links, focus ring |
| `--color-accent` (success) | `#15803D` | `--color-on-success: #FFFFFF` | 5.36:1 ✅ | "Key saved ✓", connected, success banners |
| `--color-destructive` | `#B91C1C` | `--color-on-destructive: #FFFFFF` | 5.49:1 ✅ | Delete buttons, error banners |
| `--color-warning` | `#B45309` | `--color-on-warning: #FFFFFF` | 4.80:1 ✅ | Warnings, rate-limited, partial |
| `--color-info` | `#1D4ED8` | `--color-on-info: #FFFFFF` | 6.21:1 ✅ | Info banners, tips |
| `--color-bg` | `#FFFFFF` | `--color-fg: #0F172A` | 17.9:1 ✅ | App background |
| `--color-bg-elevated` | `#F8FAFC` | `--color-fg-elevated: #0F172A` | 17.2:1 ✅ | Cards, popups |
| `--color-bg-hover` | `#F1F5F9` | — | — | Hover surface |
| `--color-bg-selected` | `#DBEAFE` | — | — | Selected item background |
| `--color-bg-overlay` | `rgba(0,0,0,0.4)` | — | — | Modal overlay |
| `--color-fg-muted` | `#475569` | on `--color-bg`: 7.55:1 ✅ | — | Secondary text |
| `--color-border` | `#E2E8F0` | — | — | Borders, dividers |
| `--color-border-strong` | `#CBD5E1` | — | — | Emphasized borders |
| `--color-ring` | `#2563EB` | — | — | Focus ring (opaque, not transparent) |
| `--color-disabled-fg` | `#94A3B8` | — | — | Disabled text/icons |
| `--color-disabled-bg` | `#F1F5F9` | — | — | Disabled surface |

### 1.2 Dark Theme

| Token | Hex | On-Token | Contrast | Role |
|---|---|---|---|---|
| `--color-primary` | `#2563EB` | `--color-on-primary: #FFFFFF` | 5.17:1 ✅ | Same as light (works on both) |
| `--color-accent` (success text/icon) | `#4ADE80` | — | on `#0F172A`: 9.99:1 ✅ | Success text, check marks, icons |
| `--color-accent-bg` | `#166534` | `--color-on-success: #FFFFFF` | 5.66:1 ✅ | Success filled badges/banners |
| `--color-destructive` (text/icon) | `#F87171` | — | on `#0F172A`: 5.48:1 ✅ | Error text, icons |
| `--color-destructive-bg` | `#991B1B` | `--color-on-destructive: #FFFFFF` | 5.61:1 ✅ | Destructive filled buttons |
| `--color-warning` (text/icon) | `#FBBF24` | — | on `#0F172A`: 9.26:1 ✅ | Warning text, icons |
| `--color-warning-bg` | `#92400E` | `--color-on-warning: #FFFFFF` | 5.50:1 ✅ | Warning filled banners |
| `--color-info` (text/icon) | `#60A5FA` | — | on `#0F172A`: 6.48:1 ✅ | Info text, icons |
| `--color-bg` | `#0F172A` | `--color-fg: #F1F5F9` | 14.6:1 ✅ | App background |
| `--color-bg-elevated` | `#1E293B` | `--color-fg-elevated: #F1F5F9` | 12.7:1 ✅ | Cards, popups |
| `--color-bg-hover` | `#334155` | — | — | Hover surface |
| `--color-bg-selected` | `#1E3A5F` | — | — | Selected item background |
| `--color-bg-overlay` | `rgba(0,0,0,0.6)` | — | — | Modal overlay |
| `--color-fg-muted` | `#94A3B8` | on `--color-bg`: 6.48:1 ✅ | — | Secondary text |
| `--color-border` | `rgba(255,255,255,0.08)` | — | — | Borders, dividers |
| `--color-border-strong` | `rgba(255,255,255,0.15)` | — | — | Emphasized borders |
| `--color-ring` | `#60A5FA` | — | — | Focus ring (opaque) |
| `--color-disabled-fg` | `#475569` | — | — | Disabled text/icons |
| `--color-disabled-bg` | `#1E293B` | — | — | Disabled surface |

### 1.3 Semantic Usage Rules

- **Filled buttons** use primary/accent/destructive with their `--color-on-*` text.
- **Text/icons** in dark theme use the brighter variant (`#4ADE80`, `#F87171`, `#FBBF24`, `#60A5FA`).
- **Focus ring** is always opaque (never `rgba(...20)`) with `outline: 2px solid var(--color-ring); outline-offset: 2px;`.
- **Disabled** state: `opacity: 0.5` + `cursor: not-allowed`; never use `color: var(--color-disabled-fg)` alone.

---

## 2. Typography

### Font Stack

```css
--font-sans: -apple-system, BlinkMacSystemFont, "Segoe UI", "Helvetica Neue",
  "PingFang SC", "Microsoft YaHei", "Noto Sans CJK SC", sans-serif;
--font-mono: "SF Mono", "Cascadia Code", "Consolas", "Noto Sans Mono CJK SC",
  monospace;
```

**No remote fonts.** Privacy-first: zero CDN requests. CJK fallbacks built in.

### Type Scale

| Token | Size | Line-height | Weight | Usage |
|---|---|---|---|---|
| `--text-xs` | 12px | 16px | 400 | Timestamps, engine labels, metadata |
| `--text-sm` | 13px | 18px | 400 | Secondary text, helper text |
| `--text-base` | 14px | 20px | 400 | Body text, input text (default) |
| `--text-md` | 16px | 24px | 500 | Section headings in settings |
| `--text-lg` | 20px | 28px | 600 | Window title, onboarding step |
| `--text-xl` | 24px | 32px | 700 | Onboarding hero, result emphasis |

### Rules

- Minimum body text: 12px (metadata only). 13px+ for anything the user reads as content.
- `font-weight: 700` for emphasis only, not entire paragraphs.
- Monospace for: error codes, API JSON, endpoint URLs, token display.

---

## 3. Spacing & Layout

### Spacing Scale (density 8/10 — compact desktop)

| Token | Value | Usage |
|---|---|---|
| `--space-xs` | 2px | Tight icon-text gaps |
| `--space-sm` | 4px | Inline spacing, icon padding |
| `--space-md` | 8px | Standard padding (cards, inputs, buttons) |
| `--space-lg` | 12px | Section padding |
| `--space-xl` | 16px | Modal padding, large gaps |
| `--space-2xl` | 24px | Section margins, onboarding spacing |
| `--space-3xl` | 32px | Rare: onboarding hero padding |

### Control Heights

| Token | Value | Usage |
|---|---|---|
| `--height-sm` | 28px | Compact buttons, badges, small inputs |
| `--height-md` | 32px | Default buttons, icon buttons, inputs |
| `--height-lg` | 36px | Primary action buttons, settings rows |

### Border Radius

| Token | Value | Usage |
|---|---|---|
| `--radius-sm` | 6px | Inputs, small badges, tags |
| `--radius-md` | 8px | Buttons, cards |
| `--radius-lg` | 12px | Modals, popup window |

### Icon Sizes

| Token | Value | Usage |
|---|---|---|
| `--icon-sm` | 14px | Inline icons in text |
| `--icon-md` | 16px | Default (buttons, list items) |
| `--icon-lg` | 20px | Toolbar, prominent actions |

---

## 4. Shadows & Elevation

### Light Theme

| Token | Value | Usage |
|---|---|---|
| `--shadow-sm` | `0 1px 2px rgba(0,0,0,0.05)` | Cards, inputs |
| `--shadow-md` | `0 4px 6px rgba(0,0,0,0.08)` | Popup, dropdown |
| `--shadow-lg` | `0 10px 15px rgba(0,0,0,0.1)` | Modal |

### Dark Theme

| Token | Value | Usage |
|---|---|---|
| `--shadow-sm` | `0 1px 2px rgba(0,0,0,0.3)` | Cards, inputs |
| `--shadow-md` | `0 4px 6px rgba(0,0,0,0.4)` | Popup, dropdown |
| `--shadow-lg` | `0 10px 15px rgba(0,0,0,0.5)` | Modal |

---

## 5. Motion

### Durations

| Token | Value | Usage |
|---|---|---|
| `--duration-fast` | 120ms | Hover color, toggle, small state |
| `--duration-base` | 180ms | Popup show/hide, panel expand, card transitions |
| `--duration-slow` | 240ms | Modal open/close, onboarding step transition |

### Easing

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

All transitions degrade to instant or opacity-only. No `transform` movements.

---

## 6. Component States

Every interactive component must implement these states:

| State | Light | Dark | Notes |
|---|---|---|---|
| **Default** | Token bg/fg | Token bg/fg | Base appearance |
| **Hover** | `--color-bg-hover` surface or `opacity: 0.9` on filled | Same | 120ms transition |
| **Pressed** | `opacity: 0.8` or darker border | Same | Active pointer-down |
| **Focus** | `outline: 2px solid var(--color-ring); outline-offset: 2px;` | Same | Never `outline: none` without replacement; visible on keyboard nav only (`:focus-visible`) |
| **Disabled** | `opacity: 0.5; cursor: not-allowed;` | Same | No hover/pressed effects |
| **Loading** | Spinner (12px `loader-2` icon, `animation: spin 0.8s linear infinite`); text + inputs `pointer-events: none` | Same | |
| **Selected** | `--color-bg-selected` background; `--color-primary` text/icon | Same | Active tab, chosen provider |
| **Destructive** | `--color-destructive` fill + `--color-on-destructive` text | `--color-destructive-bg` fill or `--color-destructive` text variant | Delete buttons, reset actions |

---

## 7. Icon Set

**Lucide Solid** (fixed by spec). Key mappings:

| Action | Lucide name | Size |
|---|---|---|
| Translate | `languages` | 16px |
| Copy | `copy` | 16px |
| Pin / Unpin | `pin` / `pin-off` | 16px |
| Speak / Stop | `volume-2` / `square` | 16px |
| History | `history` | 16px |
| Provider | `server` | 16px |
| Settings | `settings` | 20px (toolbar) |
| Delete | `trash-2` | 16px |
| Check / Saved | `check` | 14px (inline badge) |
| Error | `alert-triangle` | 16px |
| Loading | `loader-2` | 12px (spinning) |
| Search | `search` | 16px |
| Close | `x` | 16px |
| Star / Unstar | `star` / `star-off` | 16px |
| Globe (language) | `globe` | 14px |

No emoji icons. All icon-only buttons require `aria-label`.

---

## 8. Windows

### 8.1 Window Inventory & Minimum Sizes

| Window | Default | Minimum | Resizable | Z-order |
|---|---|---|---|---|
| Settings (main) | 800×600 | 600×400 | ✅ | Normal |
| Selection popup | Auto (min 200×40, max 400×300) | 200×40 | ❌ | Always-on-top |
| Input window | 420×280 | 360×200 | ✅ | Always-on-top |
| OCR overlay | Full-screen per monitor | — | ❌ | Always-on-top (above all) |
| Multi-result (popup expanded) | Same popup, expanded height (max 480×400) | 200×40 | ❌ | Always-on-top |

### 8.2 Popup Behavior

- **Unpinned:** hides on blur (clicking elsewhere).
- **Pinned:** stays visible until explicitly unpinned or closed; does NOT hide on blur.
- **Multi-engine result:** expands the SAME popup window to show stacked/tabbed results (NOT a separate window). Cards maintain user Provider sort order; completed cards do not jump position.
- **Transparent area:** outside the result card is transparent (macOS private API + Popup.css override). Click-through on transparent area is NOT required.

### 8.3 Settings Adaptive Behavior

Settings is a desktop window, not a web page — but it still adapts at narrow widths:

- **≥ 700px:** full sidebar + content side-by-side.
- **500–699px:** sidebar collapses to icon-only rail; labels appear on hover/tooltip.
- **< 500px (min 400px):** sidebar hidden; hamburger menu opens it as overlay; Provider cards stack vertically; tables scroll horizontally within their container.
- **No horizontal overflow** on the window itself. Internal scroll areas (tables, long lists) are contained.
- Onboarding target: 600×400 minimum (same as settings); uses single-column flow.

### 8.4 OCR Overlay

- Full-screen transparent overlay on each monitor.
- User drags to select a rectangle (same UX as macOS screenshot).
- `Esc` or right-click cancels.
- After selection: overlay hides, capture proceeds.
- No visible window chrome, title bar, or taskbar entry during overlay.

---

## 9. Forbidden Patterns

- ❌ Remote fonts (Google Fonts, CDN)
- ❌ Emoji as icons
- ❌ `outline: none` without replacement focus style
- ❌ `transform: translateY(...)` hover effects on desktop utility UI
- ❌ `backdrop-filter: blur()` on overlays (use solid `rgba` overlay)
- ❌ GSAP scroll-triggered animations (desktop windows, no scroll)
- ❌ Web responsive breakpoints (375/768/1024/1440) — these are desktop windows
- ❌ Layout-shifting hovers (scale, translate that changes element bounds)
- ❌ Instant state changes (0ms) — always use ≥ 120ms transition
- ❌ Placeholder-only labels (always have visible `label` elements for form fields)

---

## 10. Pre-Delivery Checklist

Before any UI is shipped:

- [ ] All color pairs pass WCAG AA (4.5:1 body, 3:1 large text)
- [ ] Light AND dark themes verified
- [ ] Focus rings visible on every interactive element (`:focus-visible`)
- [ ] Full keyboard navigation (Tab order = visual order; Enter/Esc on modals)
- [ ] `prefers-reduced-motion` respected (transitions → 1ms)
- [ ] Icon-only buttons have `aria-label`
- [ ] No remote font requests
- [ ] No horizontal overflow on any window
- [ ] All component states implemented (hover, pressed, focus, disabled, loading, selected, destructive)
- [ ] CJK text renders correctly in both themes
