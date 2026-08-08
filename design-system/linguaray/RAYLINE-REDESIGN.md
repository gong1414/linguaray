# LinguaRay Rayline Redesign — Design Handoff

**Status:** Design candidate for implementation · **Date:** 2026-08-08

**Design file:** [LinguaRay — Full Product Redesign 2026](https://design.penpot.app/#/workspace?team-id=81f57451-85cc-819d-8008-72a1a7e76bb1&file-id=3be9e5e1-190f-8090-8008-72a4d9868ce7&page-id=3be9e5e1-190f-8090-8008-72a4d9868ce8)

**Implementation plan:** [2026-08-08-rayline-redesign-development-plan.md](../../docs/superpowers/plans/2026-08-08-rayline-redesign-development-plan.md)

This document records the new visual and interaction contract created in Penpot. It does not silently replace the current production tokens in `MASTER.md`: Phase R0 of the implementation plan reconciles both sources, updates `MASTER.md`, and then changes code tokens atomically.

## 1. Direction

**Rayline** is a restrained native-desktop system built around three ideas:

- **Stay in flow:** translation actions are close to the selected text, keyboard-first, and visually quiet.
- **Explain trust boundaries:** local/cloud processing, credentials, permissions, and retention are visible before action.
- **Recover without losing work:** provider failures, permission failures, and keystore recovery preserve the user's input and show a next step.

Visual signature: indigo actions, a narrow cyan “beam” accent, neutral slate surfaces, compact controls, low shadows, Inter + Noto Sans SC + IBM Plex Mono.

## 2. Dual-collection token architecture

Penpot is organized as two layers, with Light and Dark semantic modes:

```text
Core / Primitives
  ├─ color.core.*
  ├─ space.* / radius.* / border.* / opacity.*
  └─ font.* / type.*

Semantic
  ├─ Semantic / Light
  └─ Semantic / Dark
       ├─ color.canvas / surface.* / text.*
       ├─ color.brand.* / accent.* / focus
       ├─ color.success.* / warning.* / danger.*
       └─ shadow.*
```

Components and screens consume semantic tokens only. Primitive values may change without editing component CSS; theme switching changes the active semantic set without changing component names.

### Key primitives

| Role | Value |
|---|---|
| Neutral | Slate `50–950` |
| Brand | Indigo `50–900`; default light `#4F46E5`, dark `#818CF8` |
| Accent / focus | Cyan `50–800`; default light `#06B6D4`, dark `#22D3EE` |
| Success | Green; light `#16A34A`, dark `#22C55E` |
| Warning | Amber; light `#D97706`, dark `#F59E0B` |
| Danger | Red; light `#DC2626`, dark `#EF4444` |
| Spacing | `0, 2, 4, 6, 8, 10, 12, 16, 20, 24, 32, 40, 48, 64` |
| Radius | `0, 4, 6, 8, 10, 12, 16, 20, full` |
| Type | 11–32 px; weights 400/500/600/700; line height 1.23–1.50 |

## 3. Component inventory

The `01 Components` page contains 18 handoff components. Button and Text field use true variant properties; the remaining patterns define reusable composition and state contracts.

| Group | Components |
|---|---|
| Actions | Button (Type × State), Icon button, Segmented control, Shortcut chip |
| Input | Text field (State), Select, Toggle |
| Feedback | Status badges, Inline error, Toast, Confirmation dialog, Empty state |
| Product | Translation card, Result card, Provider row, History row, Sidebar item |
| Shell | Window chrome |

Required states for interactive components: default, hover, pressed, focus-visible, disabled, loading, selected, error, and destructive where applicable. Focus uses the cyan semantic focus token and must remain visible in both themes.

## 4. Product surface inventory

| # | Surface ID | Penpot page | Primary acceptance target |
|---:|---|---|---|
| 01 | `surface.selection-popup` | 10 Core Translation | Cursor-adjacent result, copy/save/TTS, loading/offline/error |
| 02 | `surface.input-window` | 10 Core Translation | Persistent source/result editing with autosave |
| 03 | `surface.multi-result` | 10 Core Translation | Stable provider order, partial failure, pinned result |
| 04 | `surface.tray-menu` | 10 Core Translation | Quick actions, recent status, provider readiness |
| 05 | `surface.provider-center` | 20 Provider & Settings | Provider health, routing, credentials, recovery states |
| 06 | `surface.keystore-recovery` | 20 Provider & Settings | Explicit local-only credential recovery |
| 07 | `surface.shortcuts` | 20 Provider & Settings | Recording, conflict, registration failure |
| 08 | `surface.privacy-data` | 20 Provider & Settings | Local-only mode, retention, export/delete controls |
| 09 | `surface.history` | 30 Knowledge | Searchable encrypted local timeline |
| 10 | `surface.vocabulary` | 30 Knowledge | Save, tag, schedule review, export |
| 11 | `surface.dictionary` | 30 Knowledge | Definition, examples, word forms, offline attribution |
| 12 | `surface.ocr-overlay` | 40 OCR & Media | Region select, local OCR progress, permission recovery |
| 13 | `surface.text-to-speech` | 40 OCR & Media | Voice, playback, speed, queue, offline status |
| 14 | `surface.onboarding` | 50 System | Progressive permissions/provider/privacy/shortcut setup |
| 15 | `surface.external-api` | 50 System | Default-off local endpoint and one-time token disclosure |
| 16 | `surface.updater` | 50 System | Signed update progress, verification, retry/restart |

## 5. Window and layout rules

- Selection popup starts at `460 × 420`; it expands for multi-result instead of opening a competing window.
- Input and main settings use a persistent desktop shell. Settings sidebar collapses to icons before content becomes cramped.
- OCR owns the active display only while selecting; `Esc` and right-click cancel immediately.
- Onboarding remains single-column in the task area, with a step rail at wide desktop sizes.
- Primary actions are right-aligned; destructive actions never sit adjacent without a neutral escape action.
- Window chrome, spacing, state badges, and action placement stay consistent across macOS and Windows even when native frame details differ.

## 6. Handoff gate

Implementation may call a surface complete only when it has:

1. Light and Dark coverage through semantic tokens.
2. Chinese and English strings without clipping at target window size.
3. Keyboard-only operation and visible focus order.
4. Loading, empty, success, partial, offline, permission, and recoverable error states as applicable.
5. Reduced-motion behavior and no color-only status communication.
6. Screenshot comparison against its Penpot surface at the target logical pixel size.
7. Real-machine verification on macOS and Windows for system integrations.
