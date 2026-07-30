# IslandPot v1 — Design Spec

**Status:** Draft for user review · **Date:** 2026-07-30 · **Phase:** 0 done, design before Phase 1

IslandPot is an open-source, cross-platform translation/OCR/TTS desktop tool — a
maintained successor to [pot-desktop](https://github.com/pot-app/pot-desktop)
(which stopped updating) and the open answer to its author's later closed-source
paid version ([manggo](https://manggo.pylogmon.cn/)).

This spec resolves the **"how"** — the implementation decisions that the design
grilling deliberately left open. The **"what" and "why"** (pillars below) are
locked and are not re-opened here; see project memory for their provenance.

## Locked pillars (from grilling — do not re-litigate)

- **Pitch:** AI-native translation (LLM is default + translation-tuned), local-first
  as second leg, maintained-open-source as third. **Not** a chat app.
- **Core differentiator:** cc-switch-style **AI provider preset catalog** — pick a
  preset (OpenAI/Anthropic/Gemini/Ollama/中转站), fill a key, it works. No generic
  OpenAI endpoint form.
- **Stack:** Tauri 2 + Rust backend + SolidJS frontend.
- **Platforms:** Windows + macOS only (Linux out of scope for v1).
- **Contract:** unified `translate(text, from, to, options) -> text`; AI and
  traditional engines share it.
- **Engines:** AI presets = config data; traditional engines = built-in Rust
  modules (role: AI-failure fallback + system-dictionary). Ported from pot's
  `.potext` JS (leverage ①: turn reversing into JS→Rust porting).
- **OCR:** PaddleOCR (local). **No WASM / no plugin system in v1** (deferred).
- **Constraints:** solo, ~1hr/day (~730hr/~2yr budget), Rust-fluent, must-ship,
  no hard deadline.

---

## Resolved "how" decisions (this spec's purpose)

| Gap | Decision | Rationale |
|---|---|---|
| **B — selection capture** | Simulate-copy (`Cmd+C`/`Ctrl+C`) + read clipboard, via `yetone/get-selected-text`. | Same as pot/Easydict; mature, high coverage. |
| **A — key storage** | **Self-encrypted JSON file**; AES key derived from machine/user identity (machine-bound), no master password. | Verified cc-switch actually stores keys **plaintext** in target tools' JSON — but our privacy leg forbids plaintext. Self-encrypted with machine-bound key keeps the cc-switch-style UX (fill-key-and-use, zero friction) while beating plaintext. Accepted trade-off (user-confirmed): key + ciphertext both on-device → stronger than plaintext, but NOT resistant to a local attacker. Deemed acceptable; zero UX friction preserved. |
| **D — trigger** | Global hotkey (configurable). NOT release-to-translate. | Release-translate needs continuous selection polling (battery, false-triggers, heavier permissions); hotkey is the pot/Easydict convention users already know. |
| **C — popup UX** | Cursor-anchored floating, frameless popup (pot/Bob/Easydict style). | De facto standard for selection-translation; context-aware, non-occluding. |
| **E — language model** | `from=auto` relies on each engine's own detection (mostly the LLM). No self-built detector. Target = last-used + configurable. | LLMs already auto-detect well; a custom detector is unneeded solo workload. |
| **G — fallback chain** | On AI error/rate-limit, silently fall back to a built-in traditional engine; UI tags the source. No retry. Long text chunked by token budget. | Retry burns quota; silent fallback matches the "保底" role; chunking prevents single-request failures. |
| **F — settings UI** | Provider list (the catalog) + per-row key/model/default. Direct mirror of cc-switch UX. | The differentiator must be first-class in the UI. |
| **H — brand/icon** | "IslandPot" stays a placeholder; default icon. Decide post-v1. | Not worth 1hr/day now. |
| **I — testing** | Engines via mock-HTTP (`wiremock`) on request construction; Tauri commands via unit tests. | Engine logic is where bugs hide; command layer is thin. |

---

## Architecture

### Component map

```
┌─ Frontend (SolidJS, src/) ────────────────────────┐
│  Popup (C)  ·  Settings/provider mgmt (F)  ·  onboarding │
│         invoke ◄────────────────────────┐         │
└─────────────────────────────────────────┤         │
┌─ Rust backend (src-tauri/src/) ─────────▼───────┐ │
│  Tauri commands: translate · list_engines · capture_selection │
│  ┌─ translate pipeline ────────────────┐        │
│  │  resolve engine → call → (fallback) │        │
│  └──────────┬──────────────┬───────────┘        │
│  ┌──────────▼────┐  ┌──────▼────────┐  ┌────────▼─────┐
│  │ providers.rs  │  │ engines/*.rs  │  │ selection.rs │
│  │ (AI catalog + │  │ (traditional, │  │ get-selected-│
│  │  unified HTTP │  │  ported JS)   │  │ text + clip  │
│  │  caller)      │  │               │  │ backup/restore│
│  └──────┬────────┘  └───────────────┘  └──────────────┘
│         │ key read                                     │
│  ┌──────▼──────────────────────────────────────────┐
│  │ keystore.rs ─► self-encrypted JSON (machine-bound │
│  │                AES key, no master password)       │
│  └─────────────────────────────────────────────────┘
└────────────────────────────────────────────────────┘
```

### Units & responsibilities

- **`providers.rs`** (exists) — the catalog data + the unified OpenAI/Anthropic
  HTTP caller. `translate(req)` currently `bail!`s; Phase 1 implements it.
- **`engines/mod.rs` + `engines/*.rs`** (registry empty) — built-in traditional
  engines, one file each, ported from pot JS. Phase 3.
- **`selection.rs`** (new) — wraps `get-selected-text`; **backs up the clipboard,
  triggers the copy, reads the result, restores the clipboard**. Mandatory restore.
- **`keystore.rs`** (new) — `get_key(provider_id)` / `set_key(...)` over a
  **self-encrypted JSON file**. The AES key is **derived from machine/user
  identity** (machine-bound, no master password) — zero UX friction. Cross-platform
  identical logic (no per-OS keychain binding). File sits in the app data dir.
  Accepted limit: key + ciphertext share the device, so this beats plaintext but
  isn't resistant to a local attacker (user-confirmed trade-off).
- **`translate` command** (exists) — resolves engine by id (provider first, then
  built-in), calls it, applies fallback (G) on error.

### Data flow — selection translate (the primary loop)

```
hotkey pressed (global-shortcut)
  └─► capture_selection()
        ├─ backup clipboard
        ├─ simulate Cmd+C/Ctrl+C  (needs macOS Accessibility perm)
        ├─ read selected text
        └─ restore clipboard
  └─► translate(text, from="auto", to=<last-used>, engine=<default provider>)
        ├─ if AI provider: read key from keystore → HTTP → parse
        ├─ on error: fallback to built-in engine (tagged)
        └─ on long text: chunk by token budget, translate parts, join
  └─► show popup at cursor anchor (C) with result + source tag
```

### Error handling

- **Missing key** → settings deep-link, not a crash.
- **macOS Accessibility not granted** → onboarding step blocks selection features
  with a one-tap "open System Settings" link; other features still work.
- **Engine HTTP error / rate-limit** → silent fallback (G); if no fallback engine,
  show error in popup with a "retry / switch engine" affordance.
- **Clipboard restore failure** → log; never leave the user's clipboard corrupted
  silently (best-effort restore, transparent about the rare failure).

---

## Security & privacy

- **No telemetry, ever.** Stated pillar.
- **Keys in a self-encrypted file** (machine-bound AES, no master password), not
  plaintext on disk. Only read at call time, never logged, never sent anywhere
  except the chosen provider's endpoint. Accepted limit: this beats plaintext but
  is not resistant to a local attacker — the user-confirmed trade-off for zero UX
  friction.
- **Local-first options** (Ollama, PaddleOCR) require no key and work offline.
- The **macOS Accessibility** permission is the one OS-level ask; it's required to
  read selection and is explicitly user-granted, scoped, revocable.

## Testing strategy (I)

- **Engine request construction** — the bug-prone part (signing, params, parsing)
  — tested with `wiremock` mocking the provider/engine HTTP endpoint; assert the
  outgoing request shape and the parsed result. One test file per engine.
- **Tauri commands** — plain Rust unit tests on the translate pipeline with a
  fake engine implementing the `Engine` trait.
- **`selection.rs` / `keystore.rs`** — thin platform glue; integration-tested
  manually per platform (hard to unit-test OS APIs).

---

## Out of scope for v1 (explicitly deferred)

- Linux, mobile.
- WASM / third-party plugin system + registry.
- Pure-Accessibility direct selection read (path B) as a fallback to simulate-copy.
- Cross-device key sync.
- Brand/icon finalization.
- Chat, flashcards, grammar explanation, context memory (rejected as scope creep).

## Open question for the user

This spec was produced after the user declined to answer per-gap choice cards and
signalled "use your engineering judgment, show me the whole thing." Everything
above is decided by that judgment **except one item the user must still confirm**:
the **C popup layout (cursor-anchored float)** — this draft picks it as the de
facto standard, but it's the one decision a user might reasonably want to override.
Confirm or override during spec review.
