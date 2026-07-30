# IslandPot v1 — Design Spec

**Status:** Draft for user review (rev 2 — security, B-fact-fix, ownership, fallback, wire-contract, scope) · **Date:** 2026-07-30

IslandPot is an open-source, cross-platform translation/OCR desktop tool — a
maintained successor to [pot-desktop](https://github.com/pot-app/pot-desktop) and
the open answer to its author's later closed-source paid version ([manggo](https://manggo.pylogmon.cn/)).

This spec resolves the **"how"** — implementation decisions the design grilling
left open. **"What"/"why"** (pillars below) are locked; see project memory.

## Locked pillars (from grilling — do not re-litigate)

- **Pitch:** AI-native translation (LLM default + translation-tuned), local-first
  second leg, maintained-open-source third. **Not** a chat app.
- **Differentiator:** cc-switch-style **AI provider preset catalog** (pick preset →
  fill key → works). We copy cc-switch's **UX**, NOT its storage (cc-switch is
  plaintext; we are not — see A).
- **Stack:** Tauri 2 + Rust + SolidJS. **Platforms:** Windows + macOS only.
- **Contract:** unified `translate(text, from, to, options) -> text`.
- **Engines:** AI presets = config data; traditional engines = built-in Rust
  (role: AI-fallback + system-dict). Ported from pot's `.potext` JS (leverage ①).
- **OCR:** PaddleOCR (local). **No WASM / plugin system in v1** (deferred).
- **Constraints:** solo, ~1hr/day (~730hr/~2yr), Rust-fluent, must-ship, no deadline.

---

## Resolved "how" decisions

| Gap | Decision |
|---|---|
| **A — key storage** | Self-encrypted JSON file, AES-256-GCM, machine-bound key, **no master password**. See §A protocol. |
| **B — selection capture** | **Hybrid** (via `yetone/get-selected-text`): macOS A11y-direct first, simulated-copy fallback; Win simulated-copy. See §B. |
| **D — trigger** | Global hotkey (configurable), not release-to-translate. Default avoids pot/Easydict/STranslate conflicts. |
| **C — popup UX** | Cursor-anchored frameless floating popup (pot/Bob/Easydict standard). **User-confirmed.** |
| **E — language model** | Reuse each engine's auto-detect (mostly LLM `from=auto`); no self-built detector. Target = last-used + configurable. |
| **G — fallback chain** | Classified fallback — not blanket. See §G. |
| **F — settings UI** | Provider catalog list + per-row key/model/default (cc-switch UX mirror). |
| **H — brand/icon** | Placeholder; decide post-v1. |
| **I — testing** | Engines via `wiremock`; **keystore is a tested-first-class unit, not "platform glue"**; commands via unit tests. See §I. |

---

## §A — Keystore protocol (self-encrypted JSON, machine-bound)

**Threat model — explicitly scoped.** This protects against: the keystore file
being leaked/transmitted alone (e.g. committed, synced, screenshotted), and casual
plaintext-on-disk. **It does NOT protect against:** a same-user malicious process
(malware in the user's session), a local attacker with full disk access, or full
disk/system snapshots. Those are out of scope; for that, users who need it should
run the provider's key through a real secret manager. We will state this in-app.

### File format (versioned envelope)

```
{
  "version": 1,
  "kdf": "argon2id",                 // key-derivation function id
  "kdf_params": { "m_cost": ..., "t_cost": ..., "p_cost": ... },
  "salt": "<base64>",                // random, per-file, 16 bytes
  "nonce": "<base64>",               // 12-byte GCM nonce/IV, FRESH per write
  "aead": "aes-256-gcm",
  "ciphertext": "<base64>",          // AES-256-GCM of the plaintext JSON {provider_id: key}
  "tag": "<base64>"                  // (GCM tag; or embedded in ciphertext per lib)
}
```

- **AEAD = AES-256-GCM.** Nonce/IV uniqueness is load-bearing for GCM security
  (NIST SP 800-38D). **Every full-file rewrite uses a new random nonce** (never
  reused, never counter-derived without a durable monotonic store).
- **KDF = Argon2id** over (salt ∥ machine-identity) → 32-byte AES key. Argon2id
  chosen over PBKDF2 for memory-hardness.
- **Domain separation:** the KDF input is
  `b"islandpot-keystore-v1\0" ∥ normalized_machine_identity ∥ salt`, so the derived
  key can't collide with or be reused by any other use of the same identity.

### Machine / user identity source (normalized)

- **macOS:** `IOPlatformUUID` (`IOService:/.../IOPlatformExpertDevice`, "IOPlatformUUID"
  property) — hardware-stable, survives user changes. Normalized to lowercase,
  trimmed. Fallback if unavailable (e.g. VM): `getuid()`-bound user UUID via
  `dscl . -read /Users/$(id -un) GeneratedUID`. If even that fails: **fail closed**
  (see below), do NOT invent a weak identity.
- **Windows:** `MachineGuid` from registry
  `HKLM\SOFTWARE\Microsoft\Cryptography` (read-only, well-known). Fallback:
  `InstallDate` of the OS via WMI. Normalize identically.

Identity is normalized (lowercase, trim, single encoding) **before** domain
separation, so reformatting can't silently change the key.

### Durability & concurrency

- **File permissions:** `0600` (owner-only), dir `0700`, in the app data dir
  (`~/.islandpot` / `%APPDATA%\islandpot`).
- **Process mutex:** a `fs2`/`fd-lock` advisory lock on the file for the duration
  of a read-modify-write, to serialize concurrent writers within our own process
  (Tauri is single-process; this guards against the rare second-instance case).
- **Atomic write:** write to `keystore.json.tmp` in the same dir, `fsync`, then
  atomic `rename` over `keystore.json`. Never write the target file in place — a
  crash mid-write must not corrupt it.
- **No plaintext key ever paged out / logged.** Keys live only in process memory
  between keystore-read and HTTP-send.

### Fail-closed semantics (critical)

On any of: **identity changed** (machine moved/restored from snapshot),
**ciphertext tampered** (GCM auth tag mismatch), **file corrupted** (malformed
JSON / truncated), **version unsupported** → **fail closed**:

- Do **NOT** silently overwrite or reinitialize the file.
- Preserve the existing file as `keystore.json.broken-<timestamp>`.
- Surface a hard error to the UI: "keystore unreadable (reason); re-enter your
  keys." Never fabricate a working state.
- Only an explicit user action ("reset keystore") may delete it.

### Frontend interface

The frontend (SolidJS) may call **only**:
- `set_key(provider_id, key)` — store (encrypts + writes).
- `delete_key(provider_id)`.
- `key_status()` — returns which provider_ids have a key set (booleans only).

**The frontend can NEVER read a plaintext key.** `get_key` exists only in the
Rust translate service, which reads + decrypts + passes the key into the provider
caller in-memory. There is no `get_key` Tauri command.

---

## §B — Selection capture (hybrid, fact-corrected)

**Verified** against `yetone/get-selected-text` upstream: on **macOS** it does
**A11y-direct read first, simulated-copy fallback** (not pure simulate-copy as the
rev-1 spec claimed); on **Windows/Linux** it simulates `Ctrl+C` + clipboard. So:

- **macOS:** A11y (`AXUIElement` `kAXSelectedTextAttribute`) primary; if the target
  app doesn't expose a selection, fall back to simulate `Cmd+C` + clipboard. This
  is now an **in-v1** strategy (rev-1 wrongly listed A11y-direct as out of scope).
- **Windows:** simulate `Ctrl+C` + clipboard (only path).
- The "Accessibility-direct-read out of scope" line is **removed**.

**Clipboard handling — downgraded to best-effort (rev-1 over-promised):**
- **Windows** clipboard holds multiple simultaneous formats (RTF, HTML, files,
  private types). `get-selected-text`/typical backups capture only **text or
  image**, not the full format set. So we specify **best-effort restore**: we save
  what we can read (text + image if present) and restore exactly that.
- **Sequence guard (mandatory):** before restoring, compare the clipboard's
  **change-count / sequence number** (`GetClipboardSequenceNumber` on Win,
  `NSPasteboard.changeCount` on mac). **If the sequence advanced** (the user or
  another app wrote the clipboard between our capture and restore), **do NOT
  overwrite** — the user has produced new content; our restore would clobber it.
- **Revise error-handling wording:** we no longer promise "never leave corrupted."
  We promise: "best-effort restore, never overwrite a newer clipboard, transparent
  about the rare case where we couldn't restore." This is honest given the format
  limitation.

**Hard requirements (unchanged from rev-1):** macOS Accessibility permission is
mandatory and must be requested + user-guided on first launch (it gates BOTH the
A11y read AND the simulated keystroke).

---

## Architecture

### Component map

```
┌─ Frontend (SolidJS, src/) ─────────────────────────────────┐
│ Popup (C) · Settings/provider-mgmt (F) · onboarding         │
│   invokes: set_key/delete_key/key_status/translate/...      │
│   (NEVER get_key — no plaintext key crosses to JS)          │
└──────────────────────────────┬──────────────────────────────┘
                               │ async invoke
┌─ Rust backend ───────────────▼──────────────────────────────┐
│ AppState: { reqwest::Client (shared), Keystore, EngineRegistry, ... } │
│                                                              │
│  global-hotkey handler ──► selection capture (§B)            │
│         │                       │                            │
│         │ mouse pos / active app / selected text captured    │
│         │                       BEFORE popup steals focus    │
│         ▼                       ▼                            │
│  translate_service ──► resolve engine ──► classify error (§G)│
│         │                                                       │
│         │ reads key from Keystore (§A), passes INTO provider  │
│         ▼                                                       │
│  providers.rs (PURE protocol caller — no global-state access) │
│  engines/*.rs (traditional, ported JS)                         │
│                                                               │
│  Keystore (§A): self-encrypted JSON, machine-bound AES-256-GCM│
└───────────────────────────────────────────────────────────────┘
```

### Units & responsibilities

- **`providers.rs`** — **pure protocol caller.** Given `(provider_preset, key,
  request)`, it builds the HTTP call and parses the response. It does **NOT** touch
  the global keystore or AppState — the translate service reads the key and passes
  it in. This makes providers unit-testable in isolation.
- **`engines/*.rs`** — built-in traditional engines, ported from pot JS (Phase 3).
- **`selection.rs`** — wraps `get-selected-text`; hybrid per §B; best-effort
  clipboard restore with sequence guard.
- **`keystore.rs`** — §A protocol. Tested first-class (§I).
- **`translate_service`** — orchestrates: resolve engine → read key from keystore →
  call provider/engine → classify error → fallback (§G) → request-id/latest-wins.

### Data flow — selection translate (ownership & concurrency)

```
global hotkey fires (tauri-plugin-global-shortcut)
  │  ── ALL of this happens in the hotkey handler BEFORE showing the popup ──
  ├─ capture active app + cursor position
  ├─ capture_selection() ─► selected text (§B hybrid)
  ├─ issue request_id = uuid; this becomes the latest-wins id
  ├─ show frameless popup at cursor anchor (C), state=loading
  │
  └─ translate_service(req, request_id)  [async]
        ├─ resolve engine (provider-first, then built-in)
        ├─ IF engine needs key: keystore.get_key(id) ─► key (or MissingKey error)
        ├─ IF long text: chunk by token budget BEFORE calling
        ├─ call engine (providers/engines) with key passed in
        ├─ classify error (§G):
        │     retriable (net/timeout/429/5xx/parse-fail) → fallback engine
        │     config (MissingKey/401/403/invalid-model/keystore-corrupt) → settings, NOT fallback
        ├─ latest-wins: if a newer request_id arrived, DROP this result
        └─ render result (or error) into popup IF still latest
```

**Concurrency rules:**
- The `reqwest::Client` and `Keystore` live in **Tauri AppState** (shared, cheap
  to clone a `Client` handle / lock the keystore file briefly).
- `translate` command is **async**.
- **Latest-wins:** each translate gets a `request_id`. If a newer request arrives
  (user hit the hotkey again), the older in-flight result is **cancelled or
  discarded** — never rendered — so results can't arrive out of order.
- The popup is shown only after capture completes, so focus-stealing can't corrupt
  the capture.

---

## §G — Fallback & error classification (tightened)

Rev-1's "AI error → silent fallback" was too broad. Errors split into two classes:

**Retriable → may fallback to a built-in traditional engine:**
- network error, timeout
- 429 Too Many Requests, 5xx
- response parse failure

**Config / auth → go to Settings, do NOT fake a successful fallback:**
- MissingKey (no key set for provider)
- 401 / 403 (bad key)
- invalid model id
- keystore corrupt / unreadable (fail-closed, §A)

**Fallback constraints:**
- **Chunking happens BEFORE the call.** If one chunk fails, **discard all partial
  results for this request** and fall back the **whole** request to the fallback
  engine — never mix two engines' outputs across chunks.
- **Local mode is sacred:** if the user selected Ollama / a local engine, do **NOT**
  silently degrade to a remote engine. Local failure surfaces as an error, not a
  silent remote fallback.
- **Cross-remote fallback requires prior user consent** (a setting: "if my provider
  fails, fall back to X"). Default off; never phone home without the user opting in.

---

## §Wire — Provider wire contract (per dialect)

`ApiKind` is insufficient. Each dialect needs a full contract: endpoint path, auth
header, required version headers, response JSON path, timeout. `options` is a
**whitelist** per dialect, never an arbitrary merge into the HTTP body.

| Dialect | Endpoint (joined to base_url) | Auth header | Required headers | Resp path | Timeout |
|---|---|---|---|---|---|
| `openai_chat` | `/chat/completions` | `Authorization: Bearer <key>` | `Content-Type: application/json` | `choices[0].message.content` | 30s |
| `anthropic` | `/messages` | `x-api-key: <key>` | `anthropic-version: 2023-06-01`, `Content-Type: application/json` | `content[0].text` | 30s |
| `gemini_openai` | base_url **must include** `/v1beta/openai/` → `/chat/completions` | `Authorization: Bearer <key>` | `Content-Type: application/json` | `choices[0].message.content` | 30s |

**Gemini fix (rev-1 wrong):** the rev-1 preset set `base_url =
generativelanguage.googleapis.com/v1beta` + `openai_chat` dialect, which would hit
`/v1beta/chat/completions` — **wrong**. The official OpenAI-compatible endpoint is
`/v1beta/openai/chat/completions`. So the Gemini preset's `base_url` is set to
`…/v1beta/openai/` and the openai_chat dialect appends `/chat/completions`.
(Verified against Google's Gemini OpenAI-compat docs.)

**`options` whitelist:** each dialect declares which option keys it accepts
(`domain`, `formality`, `system_prompt_override`). Unknown keys are **dropped**,
never merged into the body — no injection surface.

---

## §Scope — OCR & TTS (scope reconciliation)

Rev-1 inconsistency: README roadmap lists OCR (PaddleOCR) + TTS in v1, but the
design body barely mentioned them. **Resolve by trimming v1 scope** (the lighter
of the two options, consistent with the solo budget):

**v1 = translation core only:**
- selection translate (§B), input translate, clipboard-listener translate
- AI provider catalog (the differentiator) + built-in traditional fallback engines
- cursor-anchored popup, settings UI, keystore

**Deferred to post-v1 (explicitly):**
- screenshot/OCR (PaddleOCR) — model distribution, subprocess orchestration, and
  the OCR→translate data flow are a self-contained chunk; do them as a v1.x unit.
- TTS (语音合成朗读).
- external invocation (外部调用).

**README + product description will be updated to match:** v1 = "selection/input/
clipboard translation with the provider catalog"; OCR/TTS/external-call move to the
roadmap's later phases. (Grilling's "full-feature-before-open-source" still holds —
"full feature" is redefined as the v1 translation core above + the deferred items
landed before the public open-source release, not all crammed into the first shippable cut.)

---

## §I — Testing (keystore promoted to first-class)

- **Keystore — heavily unit-tested (NOT "platform glue, manual only"):**
  - tamper: flip a byte in ciphertext → auth-tag failure → fail-closed, old file preserved.
  - wrong identity: derive key with a different machine id → auth failure → fail-closed.
  - nonce freshness: two writes produce different nonces.
  - corrupt file: truncated/garbled JSON → fail-closed, backup saved.
  - atomic write: simulate crash (kill before rename) → original intact, no `.tmp` leak.
  - concurrency: two writers under the lock serialize.
  - migration: `version: 1` envelope read/round-trips; unknown version → fail-closed.
- **Engines** — request construction + response parsing via `wiremock` mock HTTP;
  assert outgoing request shape (endpoint, headers, body) and parsed result. One
  test file per engine. The `options` whitelist is asserted (unknown keys dropped).
- **translate_service** — with a fake engine implementing the `Engine` trait; test
  latest-wins (older result discarded), error classification (retriable vs config),
  whole-request fallback on chunk failure (no mixed outputs).
- **selection.rs** — platform glue; manually integration-tested per OS (sequence-guard
  behavior verified by hand).

---

## Security & privacy

- **No telemetry, ever.** Stated pillar.
- **Keys** in self-encrypted file (§A), machine-bound, never plaintext on disk,
  never logged, never exposed to the frontend, sent only to the chosen provider's
  endpoint at call time. **Threat-model limits stated in §A and in-app.**
- **Local-first** (Ollama, and later PaddleOCR) needs no key, works offline.
- **macOS Accessibility** permission is the one OS-level ask; explicit, scoped, revocable.
- **Cross-remote fallback is opt-in** (§G) — we never silently send text to a
  remote engine the user didn't choose.

## Out of scope for v1 (explicitly deferred)

- Linux, mobile.
- WASM / third-party plugin system + registry.
- OCR (PaddleOCR), TTS, external invocation — see §Scope (v1.x before public release).
- Cross-device key sync. Brand/icon finalization.
- Chat, flashcards, grammar explanation, context memory (rejected as scope creep).
- Resistance to a same-user malicious process / local attacker / full-disk snapshot (§A).

## Open questions for the user

None outstanding — **C (cursor-anchored popup) is user-confirmed**, rev-1's last
open question is closed. This rev-2 spec awaits user review on the seven amended areas.
