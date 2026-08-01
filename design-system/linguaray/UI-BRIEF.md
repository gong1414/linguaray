# LinguaRay UI Brief — Design Decision Record

**Date:** 2026-08-01
**Branch:** `codex/s1-design-system`
**Spec ref:** [2026-08-01-linguaray-product-baseline.md](../../docs/superpowers/specs/2026-08-01-linguaray-product-baseline.md) (S0 Frozen)
**Skill ref:** `ui-ux-pro-max` v2.11.0 (adapted, SHA `e28f987c`)

> **This document records WHY each decision was made. Production UI follows
> ONLY `MASTER.md` (the single source of truth) and `pages/<page>.md` overrides.
> `SKILL-RAW.md` is the unmodified skill output kept for audit. Do NOT implement
> from `SKILL-RAW.md` or this brief — use `MASTER.md`.**

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
**light + dark theme** (system-following). The final token values (with verified
contrast ratios) are in **`MASTER.md` §1** — this section records only the
decision rationale.

### Adopted from skill
- Accent role = green (success/connected).
- Destructive = red.
- Dark border approach: low-opacity rgba.

### Overridden (with contrast corrections)
- **Primary `#1E3A5F` → `#2563EB`.** Spec calls for "光束蓝" (beam blue).
  `#2563EB` passes 5.17:1 with white text in BOTH themes (skill's navy was dark-only).
- **Light success `#16A34A` → `#15803D`** (was 3.30:1; now 5.36:1 with white).
- **Dark success text `#22C55E` → `#4ADE80`** (was 2.28:1 on dark bg; now 9.99:1).
- **Destructive `#DC2626` → `#B91C1C`** light (5.49:1); dark uses `#F87171` text
  (5.48:1) / `#991B1B` filled bg (5.61:1).
- **Focus ring: was `rgba(...,20)` → opaque `#2563EB` light / `#60A5FA` dark**
  with `outline-offset: 2px` (transparent rings are invisible).
- Added `--color-warning`, `--color-info`, `--color-bg-hover`, `--color-bg-selected`,
  `--color-bg-overlay`, `--color-on-success`, `--color-on-destructive`, `--color-disabled-*`.

All ratios are in `MASTER.md` §1.1/§1.2.

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

Final window specs are in **`MASTER.md` §8**. Key decisions recorded here:

- **Multi-engine result:** reuses the SAME popup window in expanded mode. NOT a
  separate panel/window (eliminates the "expand or open separate" ambiguity).
- **Pinned popup:** does NOT hide on blur. Only unpinned popups auto-dismiss.
- **Settings adaptive:** tested at 600×400 min. Sidebar collapses to icon-only
  rail at <700px; hamburger overlay at <500px. No horizontal overflow on the window.
- **Onboarding:** uses 600×400 min (same as settings); single-column flow.

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
