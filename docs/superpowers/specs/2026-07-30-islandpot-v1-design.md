# IslandPot v1 — Design Spec

**Status:** Approved (rev 4 — three closing fixes: A atomic-write semantics + pre-decrypt validation, B sentinel-marker state machine, wire option-wording) · **Date:** 2026-07-30

IslandPot is an open-source, cross-platform translation/OCR desktop tool — a
maintained successor to [pot-desktop](https://github.com/pot-app/pot-desktop) and
the open answer to its author's later closed-source paid version ([manggo](https://manggo.pylogmon.cn/)).

This spec resolves the **"how"** — implementation decisions the grilling left open.
**"What"/"why"** (pillars) are locked; see project memory.

## Locked pillars (from grilling — do not re-litigate)

- **Pitch:** AI-native translation (LLM default + translation-tuned), local-first
  second leg, maintained-open-source third. **Not** a chat app.
- **Differentiator:** cc-switch-style **AI provider preset catalog** (pick preset →
  fill key → works). Copy cc-switch's **UX**, NOT its plaintext storage.
- **Stack:** Tauri 2 + Rust + SolidJS. **Platforms:** Windows + macOS only.
- **Contract:** unified `translate(text, from, to, options) -> text`.
- **Engines:** AI presets = config data; traditional engines = built-in Rust (role:
  AI-fallback + system-dict), ported from pot's `.potext` JS (leverage ①).
- **OCR:** PaddleOCR (local). **No WASM / plugin system in v1** (deferred).
- **Constraints:** solo, ~1hr/day (~730hr/~2yr), Rust-fluent, must-ship, no deadline.

---

## Resolved "how" decisions

| Gap | Decision |
|---|---|
| **A — key storage** | Self-encrypted JSON, AES-256-GCM, machine-bound Argon2id key, no master password. See §A. |
| **B — selection capture** | Hybrid via a **vendored** `get-selected-text` (A11y-direct first, copy fallback); precise clipboard-sequence restore algorithm. See §B. |
| **D — trigger** | Global hotkey (configurable), not release-to-translate. |
| **C — popup UX** | Cursor-anchored frameless floating popup. **User-confirmed.** |
| **E — language model** | Reuse each engine's auto-detect; no self-built detector. Target = last-used + configurable. |
| **G — fallback chain** | Classified `FallbackEligible` vs config/auth; per-engine re-chunk; no retry. See §G. |
| **F — settings UI** | Provider catalog list + per-row key/model/default. |
| **H — brand/icon** | Placeholder; post-v1. |
| **I — testing** | Engines via `wiremock`; keystore first-class; clipboard fake-clipboard tests. See §I. |

---

## §A — Keystore protocol (self-encrypted JSON, machine-bound)

**Threat model — explicitly scoped.** Protects against: the keystore file leaked/
transmitted alone (committed, synced, screenshotted), and casual plaintext-on-disk.
**Does NOT protect against:** a same-user malicious process, a local attacker with
full disk access, or full disk/system snapshots. Those are out of scope; stated in-app.

### Crypto — fully pinned, no free parameters

- **KDF = Argon2id (RFC 9106 / v1.3, the `argon2` crate "v19" algorithm id).**
  Parameters pinned: **m = 65536 KiB (64 MiB), t = 3, p = 1, output length = 32 bytes.**
- **KDF inputs (no ambiguity):**
  - `password` = `DOMAIN_SEPARATOR || normalized_identity`, where
    `DOMAIN_SEPARATOR = b"islandpot-keystore-v1\0"` (fixed).
  - `salt` = the envelope's random 16-byte `salt`. **The salt is passed ONLY as the
    Argon2 `salt` parameter — it is NOT concatenated into `password`.** (rev-2 was
    ambiguous; salt appeared in both.)
- **AEAD = AES-256-GCM.** 12-byte random `nonce` per write. **Fixed AAD** =
  `b"islandpot-keystore-envelope-v1"`. **The 16-byte GCM tag is appended to
  `ciphertext` (standard GCM). There is NO separate `tag` field.** (rev-2 left this
  as "either"; pinned.)
- GCM nonce uniqueness is load-bearing (NIST SP 800-38D). Nonce is freshly random
  per full-file write; we do not use counters.

### Envelope (deterministic, single canonical form)

```jsonc
{
  "version": 1,
  "aead": "aes-256-gcm",
  "kdf": "argon2id",
  "kdf_params": { "m_kib": 65536, "t": 3, "p": 1, "output_len": 32 },
  "identity_source": "macos_ioplatformuuid",   // see below; pinned at creation
  "salt": "<base64 16B>",
  "nonce": "<base64 12B>",
  "ciphertext": "<base64 AES-256-GCM(plaintext) || tag>"
}
```

`plaintext` (before encryption) is the JSON object `{"<provider_id>": "<api_key>"}`.

### Identity source — pinned, no weak fallback

- **macOS:** `IOPlatformUUID` via `IOService`/`IOPlatformExpertDevice`. Enum value
  `macos_ioplatformuuid`.
- **Windows:** registry `HKLM\SOFTWARE\Microsoft\Cryptography` → `MachineGuid`
  (read-only). Enum value `windows_machineguid`.
- **No fallback.** The rev-2 `InstallDate` weak fallback is **removed**. If the
  chosen source is unavailable, **fail closed** (do not invent or downgrade to a
  weaker identity — that would silently change the key).
- **`identity_source` is recorded at creation and frozen for the life of the file.**
  Even if a "preferred" source becomes available later, we never auto-switch —
  that would re-derive a different key and look like corruption. To change source,
  the user resets the keystore and re-enters keys.
- Identity is normalized (lowercase, trim, UTF-8) **before** concatenation with the
  domain separator.

### File location & permissions

- **Location = Tauri `appLocalDataDir`** (NOT a roaming dir). Concretely:
  - **Windows:** `%LOCALAPPDATA%\<Identifier>` (non-roaming — explicitly NOT
    `%APPDATA%`, which roams and would sync encrypted-key files across machines the
    machine-bound key can't decrypt).
  - **macOS:** `~/Library/Application Support/<Identifier>`.
- **Permissions are platform-specific, not a blanket "0600":**
  - **macOS:** file `0600`, dir `0700` (chmod after every write).
  - **Windows:** set a DACL granting access to the **current user only** (via
    `windows-acl` / `windows` crate `SetNamedSecurityInfo`); no inheritance.

### Locking & atomic write

- **Cross-process lock:** a **permanent `keystore.lock`** file (sibling to
  `keystore.json`), held (advisory, via `fd-lock`/`fs2`) for the full
  read-modify-write. We do **NOT** lock the `keystore.json` inode — it is replaced
  by the rename, so an inode lock would lock the soon-to-be-orphaned file.
- **In-process:** a `parking_lot::Mutex` serializes writers within our process.
- **Atomic replace is platform-specific and distinguishes first-create vs update:**
  - **macOS/Unix:** same-directory `rename(2)` over the target (atomic on the same
    filesystem; the tmp file is in the same dir to guarantee same-fs). Works for
    both first-create and update.
  - **Windows:** `ReplaceFileW` **requires the target to already exist** — so it is
    used **only for updates**. **First creation** uses same-dir `MoveFileExW`
    (`MOVEFILE_REPLACE_EXISTING` is harmless if absent; it succeeds when the target
    doesn't exist). Plain rename-over-existing is otherwise unreliable; do not use it.
- Sequence: write `keystore.json.tmp` → `fsync` → platform atomic replace (first-create:
  `MoveFileExW`/`rename`; update: `ReplaceFileW`/`rename`) → `fsync` dir (macOS).
- **Stale-tmp handling (honest about crash windows):** a process killed **before**
  the replace can leave `keystore.json.tmp` on disk. We **cannot** guarantee "no tmp
  leak" — that was over-promised in rev-3. What we guarantee: (a) the **canonical
  `keystore.json` stays intact** (the replace is atomic; a pre-replace crash never
  touches it); (b) **on every launch, while holding the cross-process lock, we
  delete any stale `keystore.json.tmp`** before any other operation; (c) **we never
  treat a `.tmp` file as the keystore** under any code path.

### Fail-closed semantics (critical)

On **identity changed**, **GCM auth-tag mismatch** (tamper/wrong identity),
**malformed/truncated JSON**, or **unsupported `version`** → **fail closed**:
- **Do NOT overwrite, do NOT reinitialize, do NOT auto-move the file.** The
  canonical `keystore.json` stays **in place**.
- Only an **explicit user "Reset keystore" action** moves it to
  `keystore.json.broken-<timestamp>` and starts fresh.
- Rationale (rev-2 over-corrected): auto-moving on a transient auth failure makes
  the next launch see "no file" → misjudged as first-run → silently discards the
  user's keys. Keep-in-place is correct; only the user resets.

### Pre-decrypt validation (before any crypto runs)

Before deriving a key or decrypting, validate the envelope against a **fixed
whitelist** — a tampered envelope must never drive memory allocation or KDF cost:

- **`version`, `aead`, `kdf`** must be exactly the supported values (`1`,
  `aes-256-gcm`, `argon2id`); any other value → fail closed.
- **`kdf_params`** must equal the pinned params (`m_kib=65536, t=3, p=1,
  output_len=32`) **exactly** — an attacker-raised `m_kib` is a DoS vector and is
  rejected outright, never honored. We do **not** read KDF cost from the file.
- **`identity_source`** must be a known enum (`macos_ioplatformuuid` /
  `windows_machineguid`) AND must match the source recorded for this machine.
- **Length checks first:** `salt` = 16 bytes, `nonce` = 12 bytes, `ciphertext` ≥ 16
  bytes (must include the appended 16-byte GCM tag). Any mismatch → fail closed,
  before allocation/decryption.

### Plaintext-key claims (corrected)

- The rev-2 claim "frontend never sees plaintext" is **inaccurate and removed**.
  Reality: when the user **inputs** a key it necessarily exists in JS memory and
  traverses the IPC to Rust on submit.
- **Correct promise:**
  - After `set_key` succeeds, Rust **never returns the plaintext key to JS**.
  - On successful submit, the frontend **immediately clears the input field** and
    drops its reference.
- The rev-2 "never paged out" claim is **removed** — we cannot guarantee OS
  behavior. **What we DO promise:** we never write the key to any app file or log;
  we keep it in memory only for the shortest window between keystore-read and
  HTTP-send; we `zeroize` the buffer after use (`zeroize` crate).

### Frontend interface

Only these Tauri commands (no `get_key`):
- `set_key(provider_id, key)` — encrypt + atomic write.
- `delete_key(provider_id)`.
- `key_status()` — returns a map of `provider_id -> bool` (has-key, no value).

---

## §B — Selection capture (hybrid, vendored, precise clipboard algorithm)

**Verified** `yetone/get-selected-text` upstream: **macOS = A11y-direct first,
simulated-copy fallback;** Windows/Linux = simulated `Ctrl+C` + clipboard. The lib
does copy+restore **internally**, and on Windows restores only text/image — so an
unmodified outer wrapper **cannot** interpose a sequence check before restore.

**Decision: vendor (fork into our tree) and modify** `get-selected-text`, so we can
run the precise restore algorithm below. (Alternative — self-implement the clipboard
fallback — was rejected to reuse the macOS A11y logic.)

### Clipboard restore algorithm (pinned state machine, with copy-success detection)

The naive "if sequence advanced, don't restore" is **wrong** — setting our own
marker and the simulated copy itself advance the sequence, and a *failed* copy
(no selection) would otherwise be mistaken for the user's old clipboard. The
correct state machine uses a **sentinel marker** to *detect whether the copy
actually happened*:

1. **Save** the current clipboard content (best-effort: text + image; other formats
   — RTF/HTML/files/private — are NOT capturable on Windows and are lost; accepted,
   documented in §Privacy).
2. **Write a unique sentinel** (e.g. a fixed magic string + uuid) to the clipboard.
3. **`marker_sequence = current sequence number`** (the sequence after our sentinel
   write).
4. **Simulate** `Cmd+C` / `Ctrl+C`, then **poll (bounded timeout)** for
   `current_sequence != marker_sequence` — i.e. wait for *something* to overwrite
   our sentinel, which is what a successful copy does.
   - **Timeout while the clipboard still holds our sentinel** → the copy did not
     happen (no selection / target ignored the keystroke). **Restore the saved
     content, then return `NoSelection`** — we must not return the sentinel or the
     stale clipboard as "selected text."
5. **Copy succeeded:** the clipboard now holds the selection. Read it as the
   selected text. **`owned_sequence = current sequence number`** (re-read now — the
   state *we* left the clipboard in).
6. **Before restoring:** read `current_sequence`. **Restore the saved content ONLY
   IF `current_sequence == owned_sequence`.** If not equal, the user or another app
   wrote the clipboard since our copy → **do NOT restore** (we'd clobber newer
   content); the selection text we already read is still valid.
7. Zeroize the saved content from our buffer.

This is a real state machine; it is **automated-tested with a fake clipboard**
(§I), including: marker present + timeout → `NoSelection`; sentinel overwritten →
success path; another writer advancing the sequence between copy and restore →
no-restore.

**Hard requirement (unchanged):** macOS Accessibility permission is mandatory and
gated with first-launch onboarding (it covers both the A11y read and the simulated
keystroke).

---

## Architecture

### Component map

```
┌─ Frontend (SolidJS, src/) ─────────────────────────────────────┐
│ Popup (C) · Settings/provider-mgmt (F) · onboarding            │
│   invokes: set_key/delete_key/key_status/translate/...         │
│   (NO get_key — Rust never returns plaintext key after store)  │
└────────────────────────────┬────────────────────────────────────┘
                             │ async invoke
┌─ Rust backend ─────────────▼────────────────────────────────────┐
│ AppState: {                                                    │
│   reqwest::Client (shared, redirect-policy=none-for-remote),   │
│   Keystore (in-proc Mutex + keystore.lock cross-proc),         │
│   EngineRegistry, current_generation (latest-wins token) }     │
│                                                                │
│  global-hotkey handler:                                        │
│     gen = next_generation(); current_generation = gen   ← done FIRST │
│     capture under a selection mutex                            │
│     check current_generation==gen before: capture-complete,   │
│         popup-show, submit-result                              │
│     capture active app + cursor + selected text (§B)           │
│                                                                │
│  translate_service(req, gen)  [async]                          │
│     resolve engine → keystore.get_key (Rust-side) → pass key   │
│     build PROMPT from app options (§Wire) → call provider      │
│     classify error (§G) → re-chunked fallback                  │
│                                                                │
│  providers.rs — PURE protocol caller (key passed in; no global │
│                 state access). engines/*.rs likewise.           │
│  keystore.rs — §A protocol.                                     │
└────────────────────────────────────────────────────────────────┘
```

### Units & responsibilities

- **`providers.rs`** — **pure protocol caller.** Signature roughly
  `translate(preset: &ProviderPreset, key: &str, wire: WireParams) -> Result<…>`.
  It receives the key and the strong-typed `WireParams` from the service; it does
  **not** touch the keystore or AppState. Unit-testable in isolation.
- **`engines/*.rs`** — built-in traditional engines, ported from pot JS (Phase 3).
- **`selection.rs`** — vendored capture + §B restore algorithm.
- **`keystore.rs`** — §A protocol; first-class tested.
- **`translate_service`** — orchestrates: token check → resolve → keystore read →
  build prompt from app options → build `WireParams` → call → classify → fallback.

### Data flow — selection translate (race-free)

```
global hotkey fires (tauri-plugin-global-shortcut), handler runs:
  ① gen = atomic_next_generation(); current_generation.store(gen)   ← FIRST, sync
  ② acquire selection_mutex  (captures cannot interleave)
  ③ capture active app + cursor position
  ④ capture_selection() (§B)  → selected text
     · if current_generation != gen  → ABORT (a newer press superseded us)
  ⑤ show frameless popup at cursor anchor (C), state=loading
     · if current_generation != gen  → close, abort
  ⑥ translate_service(req, gen)  [async]
       resolve engine → key (Rust-side) → prompt → WireParams → call
       chunk per the PRIMARY engine's limits BEFORE calling
       on FallbackEligible error → re-chunk per FALLBACK engine's limits, call it
       · if current_generation != gen  → discard result (newer request won)
       else render result/error into popup
```

**Concurrency rules (rev-3 fixes the race):**
- The generation token is allocated **at hotkey entry, synchronously, before any
  async work** — not after capture (rev-2 did it after capture, letting an older
  capture finish later and grab "latest").
- `current_generation` is a single atomic monotonic counter.
- Selection capture is **mutexed** so two captures can't interleave.
- The token is re-checked at every state transition (capture-complete, popup-show,
  result-submit). An older operation that loses the race is aborted/discarded, never
  rendered — results can't arrive out of order.

---

## §G — Fallback & error classification (tightened)

**Class name = `FallbackEligible`** (rev-2's "Retriable" was misleading — we do NOT
retry; the name implied we might).

**`FallbackEligible` → may fall back to a built-in traditional engine (single attempt, no retry):**
- network error, timeout
- 429 Too Many Requests, 5xx
- response parse failure

**Config / auth → go to Settings, do NOT fake a successful fallback:**
- MissingKey (no key set)
- 401 / 403 (bad key)
- invalid model id
- keystore corrupt / unreadable (fail-closed, §A)

**Fallback constraints:**
- **No automatic retry.** One attempt per engine.
- **Chunking happens BEFORE the call, per that engine's limits.** If a request
  falls back, the **whole** request is re-chunked **per the fallback engine's own
  limits** (its budget may differ) and re-sent — never reuse the primary's chunks,
  never mix two engines' outputs.
- **Local mode is sacred:** if the user chose Ollama / a local engine, do NOT
  silently degrade to a remote engine. Local failure = error.
- **Cross-remote fallback requires prior user opt-in** (setting: "if my provider
  fails, fall back to X"). Default off.

---

## §Wire — Provider wire contract (per dialect)

### Endpoint URLs are FULL, stored per preset (no `base_url` + `/route` join)

`Url::join` with a leading-`/` route **replaces** the whole path (dropping `/v1` or
`/v1beta/openai`), so we do NOT store `base_url` + dialect-route. **Each preset
stores its complete endpoint URL.**

| Dialect (`api_kind`) | Example full endpoint stored in preset | Auth header | Required headers | Resp path | Timeout |
|---|---|---|---|---|---|
| `openai_chat` | `https://api.openai.com/v1/chat/completions` | `Authorization: Bearer <key>` | `Content-Type: application/json` | `choices[0].message.content` | 30s |
| `anthropic` | `https://api.anthropic.com/v1/messages` | `x-api-key: <key>` | `anthropic-version: 2023-06-01`, `Content-Type: application/json` | `content[0].text` | 30s |
| `openai_chat` (Gemini) | `https://generativelanguage.googleapis.com/v1beta/openai/chat/completions` | `Authorization: Bearer <key>` | `Content-Type: application/json` | `choices[0].message.content` | 30s |

- **Dialect name unified to `openai_chat`** everywhere. Gemini is just an
  `openai_chat` preset with its full OpenAI-compat endpoint (rev-2's `gemini_openai`
  alias is removed — the table/body mismatch is gone).
- Gemini fix confirmed: official OpenAI-compatible path is
  `/v1beta/openai/chat/completions` (the full URL above), not
  `/v1beta/chat/completions`.

### Two distinct option spaces (do NOT merge)

- **App translation options** (`domain`, `formality`, `system_prompt_override`) are
  **application-layer**. They are folded into the **prompt text** by the
  translate_service's prompt builder. They are **never serialized as top-level /
  provider-specific wire fields** — but note they DO reach the provider indirectly:
  the resulting system/user **message content** (which they shaped) is part of the
  request body. The guarantee is about *where* they appear (prompt content only,
  never as sibling fields of `model`/`temperature`), not that they're absent from
  the body entirely.
- **Wire generation parameters** (the ones that occupy top-level body fields —
  `model`, `temperature`, `max_tokens`, `stream`, `messages`) are a **separate,
  strong-typed `WireParams` struct** (a closed whitelist). Only fields on this struct
  reach the body as wire fields; arbitrary app options cannot be injected as such.

---

## §Privacy — complete (rev-2 gaps closed)

- **No telemetry, ever.**
- **NEVER logged:** selected text, the prompt, the response, HTTP bodies, HTTP
  headers. (The zeroize + no-app-file-write rules in §A apply here too.)
- **Data egress is explicit:** the selected text is sent **only** to the remote
  provider the user chose. A **fallback may send it to a second provider** — but
  only when the user opted into cross-remote fallback (§G). Local mode sends nothing
  off-machine.
- **Transport policy:**
  - Remote endpoints = **HTTPS only**. A preset URL with a non-https scheme is
    rejected at config-load.
  - HTTP is allowed **only for loopback** (Ollama at `127.0.0.1`/`localhost`).
  - The shared `reqwest::Client` is built with **redirect policy = no cross-origin
    redirects** (do not follow redirects to a different host; this prevents a
    provider URL quietly forwarding to a third party).
- **Clipboard side effects (acknowledged, not fully preventable):** the simulated
  copy may push content into **Windows Clipboard History / Cloud Clipboard** or
  **Apple Universal Clipboard** — these are OS-level features we cannot fully
  suppress. We document this; users concerned should disable those OS features.
- **macOS Accessibility** is the one OS-level ask; explicit, scoped, revocable.
- **Production hardening:** enable **CSP** on the webview; ship **minimal Tauri
  capabilities** (only the commands/plugins the app uses — no blanket allow).

---

## §Scope — v1 = translation core; clipboard = user-initiated

**v1 = translation core:**
- selection translate (§B), input translate, **user-initiated clipboard translate**
  (a hotkey/action that translates the *current* clipboard — see below).
- AI provider catalog + keystore + unified pipeline + fallback chain.
- cursor-anchored popup, settings UI, onboarding.

**`clipboard-listener translate` (passive background listening) is REMOVED from v1.**
rev-1/rev-2 listed it but never designed it. Passive listening has real hazards
(opt-in, dedup, anti-self-trigger loop, no auto-remote-send) that are not designed
here. **v1 ships user-initiated clipboard translate only.** If background listening
is ever added (post-v1), it must come with its own spec covering those hazards.

**Deferred to v1.x (before public open-source release):**
- OCR (PaddleOCR) · TTS · external invocation.

README + product description are synced to this scope (committed).

---

## §I — Testing

- **Keystore — first-class unit tests:** tamper (flip byte → tag mismatch →
  fail-closed, file in place), wrong identity (different machine id → fail-closed),
  nonce freshness (two writes differ), corrupt JSON (fail-closed, no auto-move),
  **crash-before-replace (canonical `keystore.json` intact; a stale `keystore.json.tmp`
  may remain and is cleaned at next launch; never used as the keystore)**,
  **first-create path (`MoveFileExW`/`rename`) vs update path (`ReplaceFileW`/`rename`)**,
  **pre-decrypt validation (whitelist-exact version/aead/kdf/kdf_params/identity_source;
  a tampered `kdf_params.m_kib` is rejected, never honored; bad salt/nonce/ciphertext
  lengths fail closed before decryption)**, concurrency (two writers serialize under
  lock), migration/round-trip (v1 envelope), identity-source freeze (source doesn't
  auto-switch), Windows DACL / macOS 0600 set.
- **Selection clipboard algorithm — fake-clipboard automated tests:** a fake
  clipboard implementing sequence-number + get/set; assert (a) restore when
  `current_sequence == owned_sequence`, (b) no-restore when another writer advanced
  it, (c) **sentinel still present at timeout → `NoSelection` + restore saved content,
  never return sentinel/stale clipboard as selected text**, (d) sentinel overwritten
  → success path, (e) another writer advancing the sequence between copy and restore
  → no-restore.
- **Engines** — request construction + response parsing via `wiremock`; assert
  outgoing endpoint/headers/body shape and parsed result; assert `WireParams`
  whitelist (unknown keys never appear as top-level wire fields — they may only be
  present inside message content via the prompt); assert app-options → prompt
  conversion.
- **translate_service** — fake engine implementing the trait; test token latest-wins
  (older result discarded), error classification (`FallbackEligible` vs config),
  whole-request re-chunked fallback (no mixed outputs), no-retry.
- **URL validation** — presets with non-HTTPS (non-loopback) rejected at load;
  redirect-policy enforced.

---

## Out of scope for v1

- Linux, mobile.
- WASM / third-party plugin system + registry.
- OCR (PaddleOCR), TTS, external invocation — v1.x before public release.
- Passive clipboard-listener translate (see §Scope).
- Cross-device key sync. Brand/icon. Resistance to local attacker / disk snapshot (§A).
- Chat, flashcards, grammar explanation, context memory (rejected as scope creep).

## Open questions

None. C (cursor-anchored popup) user-confirmed. **Spec approved (rev-4) — proceeding to `writing-plans`.**
