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

The skill generated a dark-first "VPN & Privacy Tool" palette. LinguaRay needs
dual-theme (light + dark). All final token values and independently-verified
contrast ratios are in **`MASTER.md` §1**. This section records only the rationale.

### Decisions
- **Primary:** skill gave dark navy. Overridden to beam blue (S1 design decision —
  the product identity calls for "光束蓝"). Final values in MASTER.md §1.
- **Success/destructive/warning/info:** skill's values had AA failures in several
  pairs. All corrected — final values and ratios in MASTER.md §1.
- **Token naming:** restructured to symmetric `*-fill` / `on-*-fill` / `*-fg`
  pattern so a token never means fill in one theme and text in the other.
- **Decorative vs strong borders:** skill had one border token at ~1.2:1 on white.
  Split into `--color-border` (decorative) and `--color-border-strong` (3:1 for
  input edges and switch tracks).
- **Selected foreground:** added `--color-selected-fg` (separate from primary) to
  pass AA on selected backgrounds.

---

## 3. Typography

### Decisions
- **Base size 14px** (S1 design decision — compact desktop density; skill default
  was 16px web).
- **System font stack only** (privacy-first: no remote fonts). Skill recommended
  Playfair Display SC / Karla (a restaurant/hospitality pairing with Google Fonts).
  Rejected: wrong mood, wrong domain, privacy violation.
- **CJK fallbacks** built into the system stack.
- Final font stack, type scale, and weights in **MASTER.md §2**.

---

## 4. Spacing, Controls, Icons — See MASTER.md §3

Adopted skill's density 8/10 output. All final token values, control heights,
border radius, icon sizes, and icon mappings in **MASTER.md §3**.

---

## 5. Windows — See MASTER.md §8

Key decisions (final specs in MASTER.md):

- **Multi-engine result:** reuses the SAME popup in expanded mode. Results shown
  side-by-side (per S0 spec). NOT a separate window.
- **Pinned popup:** does NOT hide on blur.
- **Settings adaptive:** min 600px. Sidebar collapses to icon-only at 600–699px.
  No hamburger path.
- **Onboarding:** 600×400 min, single-column.

---

## 6. Focus, Keyboard, Motion, Contrast — See MASTER.md

All final values in **MASTER.md §1, §5, §6, §10**. Rationale only here:

- **Focus:** opaque `--color-ring` + `outline` (not box-shadow).
- **Motion:** subtle tier (S1 design decision). Reduced-motion: 1ms transitions +
  spinner static/text fallback.
- **Contrast:** all ratios independently computed via WCAG formula.

---

## 7. Icon Set

**Lucide via `lucide-solid`** (SolidJS binding; standard outline icons — no filled
variants). S1 design decision (skill recommended Phosphor; overridden). Key icons:

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
- Color role structure (adapted to symmetric fill/fg naming)
- Dense spacing scale (density 8/10)
- Subtle motion tier (no scroll choreography; final durations in MASTER.md §5)
- Accessibility rules: contrast 4.5:1, focus rings, keyboard nav, icon aria-labels
- Anti-patterns: no emojis as icons, no instant state changes, no invisible focus
- Pre-delivery checklist (adapted for desktop, not responsive web breakpoints)

### Overridden
- Primary color → beam blue (S1 design decision; final values in MASTER.md §1)
- Fonts → system font stack (privacy: no remote fonts)
- Style → Flat Design + Micro-interactions (translator, not editorial)
- Page pattern → not applicable (desktop utility, no landing page)
- GSAP scroll reveal → not applicable (desktop windows, no scroll)
- Responsive breakpoints → not applicable (fixed desktop windows)
- Base font size → 14px (compact desktop density; S1 design decision)

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
