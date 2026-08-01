# LinguaRay UI Brief — S1a Design Decisions

**Date:** 2026-08-01
**Branch:** `codex/s1-design-system`
**Spec ref:** [2026-08-01-linguaray-product-baseline.md](../../docs/superpowers/specs/2026-08-01-linguaray-product-baseline.md) (S0 Frozen)
**Skill ref:** `ui-ux-pro-max` v2.11.0 (adapted, SHA `e28f987c`)

---

## 1. Design Identity

**Native restraint + beam blue.** LinguaRay is a menu-bar/tray-resident translation
utility — not a web landing page, not a SaaS dashboard. The UI must feel like a
native macOS/Windows tool: compact, unobtrusive, fast.

| Aspect | Decision | Source |
|---|---|---|
| Overall style | Flat Design + Micro-interactions (skill "Translator App" result) | `product` domain |
| Density | Dense/desktop (8/10) | Design dial `--density 8` |
| Motion | Subtle (3/10): 150–200ms transitions only; no scroll choreography | Design dial `--motion 3` |
| Variance | Centered/minimal (3/10): symmetric, predictable | Design dial `--variance 3` |

---

## 2. Color Tokens

The skill generated a "VPN & Privacy Tool" dark-first palette. LinguaRay needs
**light + dark theme** (system-following). We adopt the skill's accent and semantic
roles, but restructure for dual-theme CSS custom properties.

### Adopted from skill
- Accent role = green (`#22C55E`) — used for "connected", "key saved ✓", success states.
- Destructive = `#DC2626`.
- Border approach: low-opacity rgba on dark; warm grey on light.

### Overridden / rejected
- **Primary = `#1E3A5F` (dark navy) → OVERRIDDEN to beam blue `#2563EB`.**
  The spec calls for "光束蓝" (beam blue). `#1E3A5F` is a muted navy; `#2563EB`
  is a clear, energetic blue that reads as "beam" in both light and dark themes.
- **Skill background `#0F172A` (dark-only) → SPLIT into light/dark tokens.**
- **Skill style "Exaggerated Minimalism" → REJECTED.** That style is for
  fashion/editorial landing pages with `clamp(3rem, 10vw, 12rem)` headings. LinguaRay
  is a compact desktop utility. Adopted style: **Flat Design + Micro-interactions**
  (from the "Translator App" product result, not the generic style match).

### Final token table (CSS custom properties)

| Token | Light | Dark |
|---|---|---|
| `--color-primary` | `#2563EB` | `#3B82F6` |
| `--color-on-primary` | `#FFFFFF` | `#FFFFFF` |
| `--color-accent` | `#16A34A` | `#22C55E` |
| `--color-bg` | `#FFFFFF` | `#0F172A` |
| `--color-bg-elevated` | `#F8FAFC` | `#1E293B` |
| `--color-fg` | `#0F172A` | `#F1F5F9` |
| `--color-fg-muted` | `#64748B` | `#94A3B8` |
| `--color-border` | `#E2E8F0` | `rgba(255,255,255,0.08)` |
| `--color-destructive` | `#DC2626` | `#EF4444` |
| `--color-ring` | `#2563EB20` | `#3B82F620` |

---

## 3. Typography

### Adopted
- Base size: 14px (desktop utility, not 16px web default — the spec calls for
  "紧凑桌面密度" compact density).
- Line-height: 1.5 (skill UX rule, adopted).
- System font stack only.

### Overridden / rejected
- **Skill fonts (Playfair Display SC / Karla) → REJECTED.** Those are a
  restaurant/hospitality pairing with remote Google Fonts. LinguaRay is a
  multilingual desktop tool that must render CJK correctly without loading
  remote fonts (privacy-first principle from the spec).

### Final font stack

```css
--font-sans: -apple-system, BlinkMacSystemFont, "Segoe UI", "Helvetica Neue",
  "PingFang SC", "Microsoft YaHei", "Noto Sans CJK SC", sans-serif;
--font-mono: "SF Mono", "Cascadia Code", "Consolas", "Noto Sans Mono CJK SC",
  monospace;
```

- **No remote Google Fonts.** Privacy-first: no font CDN requests.
- CJK fallbacks: PingFang SC (macOS), Microsoft YaHei (Windows), Noto Sans CJK SC.
- Mono for error codes, API responses, technical details.

---

## 4. Spacing (density 8/10)

Adopted from the skill's `--density 8` output:

| Token | Value | Usage |
|---|---|---|
| `--space-xs` | 2px | Tight gaps |
| `--space-sm` | 4px | Icon gaps, inline spacing |
| `--space-md` | 8px | Standard padding (cards, inputs) |
| `--space-lg` | 12px | Section padding |
| `--space-xl` | 16px | Large gaps, modal padding |
| `--space-2xl` | 24px | Section margins |
| `--space-3xl` | 32px | Rare: onboarding hero |

---

## 5. Window Sizes & Z-Order

| Window | Size | Z-order | Notes |
|---|---|---|---|
| **Settings (main)** | 800×600 (min 600×400) | Normal | Resizable; not always-on-top |
| **Selection popup** | Auto-sized (min 200×40, max 400×300) | Always-on-top | Transparent background outside card; dismisses on blur |
| **Input window** | 420×280 (min 360×200) | Always-on-top | Resizable; Enter translates, Shift+Enter newlines |
| **OCR overlay** | Full-screen per monitor | Always-on-top (above all) | Transparent; user draws rectangle; Esc cancels |
| **Multi-result panel** | 480×400 (min 400×300) | Always-on-top | Extends from popup or opens separately |

---

## 6. Focus, Keyboard, Contrast, Reduced-Motion

Adopted from skill UX rules (priority 1: Accessibility):

- **Focus rings:** always visible; `--color-ring` 3px box-shadow on `:focus-visible`.
  Never `outline: none` without a replacement.
- **Keyboard navigation:** Tab order follows visual order. All actions reachable
  via keyboard. Input window: Enter = translate, Shift+Enter = newline, Esc = close.
- **Contrast:** WCAG AA minimum (4.5:1 body text, 3:1 large text). Both light and
  dark themes verified.
- **Reduced motion:** `@media (prefers-reduced-motion: reduce)` → all transitions
  reduced to `opacity` only (no `transform`), duration ≤ 100ms. Popup show/hide
  uses fade, not slide.
- **Icon-only buttons:** must have `aria-label` (skill anti-pattern rule).

---

## 7. Icon Set

**Lucide Solid** (per spec §S1 tech stack). The skill recommended Phosphor, but
the spec already fixed Lucide. Key icons:

| Action | Lucide name | Usage |
|---|---|---|
| Translate | `languages` | Primary translate button |
| Copy | `copy` | Copy result to clipboard |
| Pin | `pin` | Pin popup (stays visible) |
| Speaker | `volume-2` | TTS speak |
| History | `history` | Open history |
| Provider | `server` / `cloud` | Provider management |
| Settings | `settings` | Settings |
| Delete | `trash-2` | Destructive actions |
| Check | `check` | Key saved, connection OK |
| Alert | `alert-triangle` | Error, warning |
| Loader | `loader-2` (spin) | Loading states |

---

## 8. Skill Recommendations Summary

### Adopted
- Product match: "Translator App" → Flat Design + AI-Native UI + Micro-interactions
- Color role structure: primary/accent/destructive/muted/border/ring
- Dense spacing scale (density 8/10)
- Subtle motion tier (150–300ms, no scroll choreography)
- Accessibility rules: contrast 4.5:1, focus rings, keyboard nav, icon aria-labels
- Anti-patterns: no emojis as icons, no instant state changes, no invisible focus
- Pre-delivery checklist (adapted for desktop, not responsive web breakpoints)

### Overridden
- Primary color `#1E3A5F` → `#2563EB` (beam blue per spec)
- Fonts (Playfair/Karla) → system font stack (privacy: no remote fonts)
- Style "Exaggerated Minimalism" → "Flat Design + Micro-interactions" (translator, not editorial)
- Page pattern "Enterprise Gateway" → not applicable (desktop utility, no landing page)
- GSAP scroll reveal → not applicable (desktop windows, no scroll)
- Responsive breakpoints 375/768/1024/1440 → not applicable (fixed desktop windows)
- Base font size 16px → 14px (compact desktop density)

### Rejected entirely
- Remote Google Fonts (`@import url(fonts.googleapis.com...)`) — privacy violation
- Hover `transform: translateY(-1px)` on buttons — unnecessary for desktop utility
- Card `transform: translateY(-2px)` on hover — same
- `backdrop-filter: blur(4px)` on modal overlay — keep simple; use `rgba` overlay

---

## 9. Next Steps (S1b)

This brief freezes the design decisions. S1b will:
1. Create `packages/ui` with the CSS token variables, font stack, and base components.
2. Create `apps/ui-lab` with mock-data prototypes for every window/state in the spec's
   state matrix (§4.1–4.16).
3. All prototypes must pass the S1 design gate: clickable, i18n (zh/en), light/dark,
   keyboard-navigable, visually reviewed at target window sizes.
