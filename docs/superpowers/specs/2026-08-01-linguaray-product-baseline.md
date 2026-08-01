# LinguaRay — Product Baseline Spec (S0 Freeze)

**Status:** Draft (S0 — awaiting freeze approval) · **Date:** 2026-08-01
**Supersedes:** [2026-07-30-linguaray-v1-design.md](./2026-07-30-linguaray-v1-design.md) (the v1 translation-core scope; its security/clipboard/CI/signing implementations are retained as the engineering base)

---

## 1. Product Goal & Release Rule

LinguaRay is a **menu-bar/tray-resident cross-platform translation efficiency tool** — an open-source successor to [Bob](https://bobtranslate.com/guide/) and [Pot](https://github.com/pot-app/pot-desktop), with a [CC Switch](https://github.com/farion1231/cc-switch)-style provider configuration experience.

**Alignment benchmark:** Bob and Pot's capability *categories*. Not their brand, pixel design, or every third-party service. Plugin SDK, Bob/Pot plugin compatibility, and 30+ service count are post-baseline.

**Release rule:** **No public release until S7 acceptance passes.** No tags, no GitHub Releases, no preview versions. CI artifacts for internal testing only. The first public tag + Release is created at S7 sign-off.

**Quality principle:** Quality first, no hard deadline. Privacy, security, dual-platform verification, and the UI design gate are never skipped for speed. Full-time estimate: 8–12 weeks.

---

## 2. Capability Matrix

Legend: ✅ Baseline (must ship) · 🔜 Post-baseline (after S7) · ❌ Explicitly excluded

### 2.1 Translation

| Capability | Status | Acceptance result |
|---|---|---|
| Selection translate (highlight → popup) | ✅ | Select text in any app → translation popup at cursor, clipboard restored |
| Input translate (type → translate) | ✅ | Open input window → type → Enter → translation shown |
| Clipboard translate (copy → translate) | ✅ | Copy text → trigger → popup shows translation |
| Screenshot OCR translate (region select → OCR → translate) | ✅ | Select screen region → OCR → translation popup |
| Image OCR translate (file/drag/clipboard image) | ✅ | Provide image → OCR → translation popup |
| Multi-engine parallel translate | ✅ | Explicit opt-in → results from N providers shown side-by-side, with a consent prompt that text goes to multiple services |
| Traditional MT fallback (on AI error) | ✅ | On net/timeout/5xx/parse: fall back to selected traditional engine. On 401/403/missing-key: show error, no fallback |
| Translation history | ✅ | Opt-in encrypted history with search, retention, export |
| Vocabulary / wordbook | ✅ | Save words, export CSV/JSON/AnkiConnect |
| Dictionary lookup | ✅ | Select word → definition (macOS system dict + offline StarDict/MDX packages) |
| Text-to-speech (TTS) | ✅ | Read aloud translation or source (system offline voices) |
| Plugin SDK / third-party plugins | 🔜 | Post-baseline |
| Bob/Pot plugin compatibility | 🔜 | Post-baseline |
| Cloud OCR/TTS provider extensions | 🔜 | Post-baseline |
| Chat / explain / grammar / flashcards | ❌ | Explicitly rejected as scope creep |

### 2.2 Provider Management

| Capability | Status | Acceptance result |
|---|---|---|
| Preset provider catalog (fill-key-and-use) | ✅ | Pick preset → enter key → works |
| Custom OpenAI-compatible provider | ✅ | Enter endpoint + model + key → works |
| Multiple instances per template | ✅ | Two OpenAI accounts with different keys |
| Copy / duplicate provider | ✅ | Clone an existing provider configuration |
| Enable / disable provider | ✅ | Disabled provider doesn't appear in active selection |
| Drag-to-reorder | ✅ | Provider order persists across restarts |
| Quick-switch active provider (tray) | ✅ | Tray menu → switch active provider without opening settings |
| Model discovery (fetch available models) | ✅ | Click "fetch models" → dropdown populates from provider API |
| Connection / latency test | ✅ | Click "test" → shows latency or error |
| Balance / quota display | ✅ | Show when provider API supports it |

**Built-in catalog:**

- **AI:** OpenAI, Anthropic, Gemini, DeepSeek, OpenRouter, Azure OpenAI, Ollama, Custom OpenAI-compatible
- **Traditional:** Google, DeepL, Microsoft Translator, 百度, 有道, 腾讯

### 2.3 System & Platform

| Capability | Status | Acceptance result |
|---|---|---|
| Menu-bar / tray resident (macOS / Windows) | ✅ | App lives in tray; launch does not force-open main window |
| Configurable keyboard shortcuts | ✅ | Settings → change shortcut; conflict detection; registration failure doesn't crash |
| External API (local HTTP server) | ✅ | Default off; `127.0.0.1:61742`; Bearer token auth; rate-limited |
| Auto-update (signed) | ✅ | Tauri updater with separate signing key; manifest only after all platforms pass |
| Full Chinese + English UI | ✅ | Every string localized; `@solid-primitives/i18n` |
| macOS + Windows per-slice parity | ✅ | Each slice ships on both platforms with real-machine testing |

---

## 3. Windows & Entry Points

### 3.1 Window Inventory

| Window | Purpose | Lifecycle |
|---|---|---|
| **Tray/Menu-bar** | Always present; quick actions, provider switch, open settings | Process lifetime |
| **Settings (main)** | Provider CRUD, preferences, shortcuts, privacy, about | On-demand (from tray or dock) |
| **Selection popup** | Floating translation result at cursor | Transient (show on trigger, hide on blur/copy/pin-dismiss) |
| **Input window** | Manual text entry → translate | On-demand (hotkey or tray) |
| **OCR overlay** | Full-screen region selection crosshair | Transient (select → capture → dismiss) |
| **Multi-result panel** | Side-by-side results from parallel engines | Transient (extends popup or opens separate panel) |

### 3.2 Entry Points

| Trigger | Action |
|---|---|
| Global hotkey (selection) | Capture selection → translate → popup |
| Global hotkey (input) | Show input window |
| Global hotkey (OCR) | Show OCR overlay |
| Tray menu → "Translate clipboard" | Read clipboard → translate → popup |
| Tray menu → active provider | Quick-switch |
| Tray menu → Settings | Open settings window |
| Tray menu → History | Open history viewer |
| External API `POST /v1/translate` | Translate without UI; return JSON |
| External API `POST /v1/ocr` | OCR without UI; return text |

---

## 4. State Matrix

Every UI surface must handle these states. "—" means not applicable to that surface.

### 4.1 Selection Popup

| State | Display |
|---|---|
| **Initial** | Not visible (hidden) |
| **Loading** | Small card at cursor: "…" spinner |
| **Success (single)** | Translation text + engine label |
| **Success (multi)** | Tabbed/stacked results, each with engine label |
| **Partial success** | Results from some engines + error badge on failed ones |
| **Error (network)** | Error card: "Network error" |
| **Error (config)** | Error card: "API key missing" or "401 Unauthorized" |
| **Error (no selection)** | Error card: "No text selected" |
| **Error (no permission)** | Error card: "Grant Accessibility permission" |
| **Keystore corrupt** | Error card: "Keystore unreadable" + link to settings recovery |
| **Offline** | If traditional engine available → fallback result; else error "Offline" |
| **Pinned** | Popup stays visible until manually dismissed; supports copy/retry/TTS/favorite |

### 4.2 Input Window

| State | Display |
|---|---|
| **Initial** | Empty textarea, focus ready |
| **Loading** | Translate button → "…", textarea read-only |
| **Success** | Translation shown below input |
| **Error** | Error message shown below input |
| **Offline** | Error "Offline" (or fallback if configured) |

### 4.3 Settings — Provider Center

| State | Display |
|---|---|
| **Initial (no providers)** | Empty state: "Add your first provider" + preset suggestions |
| **Loading models** | Spinner in model dropdown |
| **Model fetch error** | Error tooltip; manual entry still available |
| **Connection testing** | Spinner on test button |
| **Connection OK** | Green checkmark + latency ms |
| **Connection failed** | Red X + error message |
| **Key saved** | "✓" badge on provider card |
| **Key missing** | Warning badge; "Enter key" prompt |
| **Duplicate** | New card with "(copy)" suffix |

### 4.4 OCR Overlay

| State | Display |
|---|---|
| **Initial** | Dimmed screen + crosshair |
| **Selecting** | Bright rectangle follows cursor |
| **Capturing** | Flash + spinner |
| **OCR processing** | Small spinner at selection |
| **Success** | Translation popup at selection |
| **Error (no text)** | "No text recognized" |
| **Error (permission)** | "Grant Screen Recording permission" (macOS) / "Grant capture permission" (Windows) |
| **Cancelled** | Overlay dismissed (Esc / right-click) |

### 4.5 History

| State | Display |
|---|---|
| **Initial (not opted in)** | Privacy gate: "Enable history?" with explanation |
| **Empty** | "No history yet" |
| **Loading** | Skeleton rows |
| **Populated** | List: source snippet → translation snippet + engine + timestamp |
| **Search** | Filtered results; "no matches" if empty |
| **Export** | Format selection → file save dialog |
| **Retention cleanup** | Background cleanup runs; non-intrusive summary badge shows "N items cleaned" |

### 4.6 Tray / Menu-bar

| State | Display |
|---|---|
| **Normal** | LinguaRay icon; click → menu |
| **Active translation** | Subtle pulse on icon |
| **Error (general)** | Red dot on icon |
| **Update available** | Badge on icon + menu item |

### 4.7 Onboarding (first launch)

| State | Display |
|---|---|
| **Welcome** | "Welcome to LinguaRay" + brief intro + "Get started" |
| **Accessibility grant** (macOS) | Explanation + "Open System Settings" button + "Skip" |
| **Add first provider** | Provider preset grid → select → enter key → "Test" |
| **History opt-in** | "Enable translation history?" + privacy explanation + "Enable" / "Skip" |
| **Shortcut setup** | Show default shortcuts; "Customize" or "Use defaults" |
| **Complete** | "You're all set!" + "Open settings" or "Minimize to tray" |

### 4.8 Multi-Result Panel (parallel translation)

| State | Display |
|---|---|
| **Loading** | N spinner cards (one per parallel provider) |
| **Partial success** | Filled results for successful engines + error badges on failed ones |
| **All success** | All cards filled, sorted by elapsed time or user preference |
| **All failed** | All cards show error; fallback (if configured) shown as a separate result |
| **Error (consent revoked)** | "Multi-engine consent required" — link to re-consent |

### 4.9 Settings — Shortcuts

| State | Display |
|---|---|
| **Default** | List of actions + current key combo + "Change" |
| **Recording** | "Press a key combo…" + cancel |
| **Conflict** | Red highlight: "Conflicts with {other action}" + "Override" / "Cancel" |
| **Registration failed** | Warning: "This combo couldn't be registered (system reserved)" + revert |

### 4.10 Settings — Privacy & Data

| State | Display |
|---|---|
| **History disabled** | Toggle off; explanation of what's not stored |
| **History enabled** | Toggle on; retention period selector; "Clear all" button |
| **External API off** | Toggle off; explanation |
| **External API on** | Toggle on; port display; "Regenerate token" (shows token once); "Copy token" |

### 4.11 Settings — Keystore Recovery

| State | Display |
|---|---|
| **Healthy** | No banner; settings normal |
| **Corrupt** | Error banner: "Keystore unreadable: {reason}" + "Archive & re-enter" + "Reset" |
| **Archived** | Banner clears; "Enter your keys again" prompt |
| **Reset (confirm)** | Warning dialog: "History will become undecryptable. Continue?" |

### 4.12 Vocabulary / Wordbook

| State | Display |
|---|---|
| **Empty** | "No saved words yet" + hint |
| **Populated** | List: word + definition snippet + timestamp + delete |
| **Export** | Format selector: CSV / JSON / AnkiConnect → progress → done/error |

### 4.13 Dictionary

| State | Display |
|---|---|
| **No packages** | "No dictionaries installed" + "Browse packages" |
| **Package installing** | Progress bar |
| **Lookup result** | Definition text + source dictionary name |
| **Lookup no result** | "No definition found" |
| **Lookup error** | "Dictionary error: {message}" |

### 4.14 TTS

| State | Display |
|---|---|
| **Idle** | Speaker icon (click to speak) |
| **Speaking** | Animated speaker icon + "Stop" |
| **Error** | "TTS error: {message}" (e.g., no voices available) |
| **No voices** | "No system voices found" (rare) |

### 4.15 External API Token Management

| State | Display |
|---|---|
| **Disabled** | "External API: Off" + "Enable" |
| **Enabling** | Spinner → token shown once → "Copy now — you won't see it again" |
| **Enabled** | "External API: On (port {port})" + "Regenerate token" + "Disable" |
| **Regenerating** | Warning: "Old token will stop working immediately" → new token shown once |

### 4.16 Updater

| State | Display |
|---|---|
| **Checking** | Silent (background) |
| **Up to date** | "LinguaRay is up to date (v{version})" |
| **Update available** | "v{new_version} available" + changelog summary + "Download" |
| **Downloading** | Progress bar |
| **Verifying** | "Verifying signature…" |
| **Verification failed** | Error: "Update signature verification failed — update aborted" |
| **Ready to install** | "Restart to update" + "Restart now" / "Later" |
| **Install failed** | Error: "Update installation failed: {reason}" + "Download manually" link |

---

## 5. Domain Model

### 5.0 Keystore Inner Structure (versioned)

The keystore's encrypted JSON gains a versioned inner structure to separate key
categories. The existing flat `{"provider_id": "key"}` map is migrated to:

```
KeystoreData {
    version: u32,                          // 2 (v1 = legacy flat map)
    provider_keys: Map<String, String>,    // keyed by ProviderProfile UUID
    history_key: Option<[u8; 32]>,         // opt-in history encryption key
    external_api_token: Option<String>,    // opt-in external API bearer token
}
```

**Legacy flat-map migration:** On first load after upgrade, if `version` is absent
(legacy), the flat map is treated as `provider_keys` with template-id keys. The
migration protocol (§8.4) renames them to UUID keys as profiles are created.

### 5.1 ProviderProfile

```
ProviderProfile {
    uuid: String              // deterministic UUID v5 (namespace + template_id + name)
                              // for migrated profiles; UUID v4 for user-created
    template_id: String       // "openai", "anthropic", "google", "custom", etc.
    name: String              // user-editable display name
    protocol: Protocol        // openai_chat | anthropic | gemini | google_translate | custom_http
    endpoint: String          // full URL (base + path); HTTPS-only except loopback
    model: Option<String>     // model identifier (AI providers)
    enabled: bool             // appears in active selection
    sort_order: i32           // display order
    is_local: bool            // Ollama etc. (no key needed, localhost)
    secret_ref: String        // = provider UUID; keystore.provider_keys[uuid] holds the key
    capabilities: ProviderCapabilities  // balance, quota, model_list, etc.
}
```

### 5.2 TranslationSession

```
TranslationSession {
    request_id: String        // unique per request (latest-wins token)
    source_text: String       // (not persisted unless history opt-in)
    detected_language: Option<String>
    results: Vec<ProviderResult>
    total_elapsed_ms: u64
}

ProviderResult {
    provider_uuid: String
    engine_id: String         // actual producing engine
    elapsed_ms: u64
    outcome: ResultOutcome    // tagged enum (not simultaneous text + error)
}

ResultOutcome =
    | Success { translated_text: String }
    | Failure { error: ErrorClassification }

ErrorClassification {
    kind: ErrorKind           // Network | Timeout | RateLimit | ServerError | AuthFailed | InvalidRequest | ParseError
    fallback_eligible: bool   // true for Network/Timeout/RateLimit/ServerError/ParseError
    message: String
}
```

### 5.3 History Model (multi-engine)

History is split into sessions (one per user translation action) and results
(one per provider that responded). This supports multi-engine parallel, partial
success, and per-provider error display.

```
history_sessions {
    session_uuid: String      // app-generated UUID v4, created BEFORE encryption
    timestamp: i64            // Unix epoch
    trigger_source: String    // "selection" | "input" | "clipboard" | "ocr" | "api"
    detected_language: Option<String>
    target_language: String
    is_favorite: bool
    source_text_encrypted: Vec<u8>    // AES-256-GCM (ciphertext || tag concatenated)
    source_text_nonce: [u8; 12]
    source_text_aad: String           // = session_uuid (bound to ciphertext)
    crypto_version: u32               // 1 (AES-256-GCM); allows future algorithm changes
    provider_name_snapshot: String    // display name at time of translation
                              // (survives provider deletion; no dangling blank)
}

history_results {
    session_uuid: String      // FK → history_sessions.session_uuid
    provider_uuid: String     // may dangle if provider deleted (name snapshot in session)
    engine_id: String
    elapsed_ms: u64
    outcome_tag: String       // "success" | "failure"
    result_text_encrypted: Option<Vec<u8>>   // Some on success
    result_text_nonce: Option<[u8; 12]>
    error_kind: Option<String>              // Some on failure
    error_message_encrypted: Option<Vec<u8>> // encrypted error detail
    error_message_nonce: Option<[u8; 12]>
    crypto_version: u32
}
```

**Key design decisions:**
- `session_uuid` is generated **before** encryption and used as AAD — never use
  auto-increment DB IDs for crypto binding (they don't exist until after insert).
- AES-GCM tag is **appended** to ciphertext (`ciphertext || 16-byte tag`), as is
  standard for `aes_gcm::Aes256Gcm::encrypt`.
- `crypto_version` field allows future algorithm migration without schema change.
- `provider_name_snapshot` in the session ensures history never shows a blank or
  dangling name after a provider is deleted.

### 5.4 VocabularyItem

```
vocabulary {
    item_uuid: String         // UUID v4, generated before encryption; used as AAD
    timestamp: i64
    source_language: String
    target_language: String   // plaintext metadata
    word_encrypted: Vec<u8>   // AES-256-GCM (ciphertext || tag)
    word_nonce: [u8; 12]
    word_aad: String           // = item_uuid
    definition_encrypted: Vec<u8>
    definition_nonce: [u8; 12]
    definition_aad: String     // = item_uuid
    crypto_version: u32
}
```

---

## 6. Tauri Command Boundary

Frontend ↔ Rust via Tauri IPC. Frontend **cannot** read plaintext API keys or access SQLite directly.

### Provider commands
```
provider_list() → Vec<ProviderProfile>
provider_create(template_id, name, endpoint, model) → ProviderProfile
provider_update(uuid, patch: ProviderPatch) → ProviderProfile
provider_duplicate(uuid) → ProviderProfile
provider_delete(uuid) → ()
provider_reorder(uuids: Vec<String>) → ()
provider_toggle(uuid, enabled) → ()
provider_set_key(uuid, key) → ()       // writes to keystore.provider_keys[uuid] only
provider_get_models(uuid) → Vec<ModelInfo>
provider_test_connection(uuid) → ConnectionResult
provider_set_active(
    primary: String,                   // primary provider UUID
    parallel: Vec<String>,             // parallel provider UUIDs (empty = single-engine)
    fallback: Option<String>,          // traditional engine UUID or None
) → ()
```

**ProviderPatch** (strong-typed whitelist — unknown fields are rejected, not merged):
```
ProviderPatch {
    name: Option<String>,
    endpoint: Option<String>,           // HTTPS-only except loopback; redirect prohibited
    model: Option<String>,
    enabled: Option<bool>,
    sort_order: Option<i32>,
}
```

**Provider delete cascade:**
- Keystore: `provider_keys[uuid]` is removed (key erased).
- History: `history_sessions.provider_name_snapshot` is retained (already captured
  at translation time); `history_results.provider_uuid` may dangle but the session
  display uses the snapshot, not a live lookup.
- Active settings: if the deleted provider was primary/parallel/fallback, the
  corresponding slot is cleared (primary falls back to the first enabled provider).

**Multi-engine consent:**
- `preferences.parallel_consent_version` tracks the user's consent to send text to
  multiple providers. Adding a new provider to the parallel list requires re-confirming
  the data-sending scope (consent version must match the current provider set hash).
- Default: parallel disabled, no consent stored.

### Translation commands
```
translate_session(request: TranslateRequest) → TranslationSession
translate_clipboard() → ()
translate_ocr(image: ImageData) → TranslationSession
```

### Content commands
```
history_query(filter: HistoryFilter) → Vec<HistoryRecord>
history_delete(ids: Vec<i64>) → ()
history_toggle_favorite(id) → ()
history_clear(before: Option<i64>) → ()
history_export(format: ExportFormat, filter: HistoryFilter) → FilePath
history_set_enabled(enabled: bool) → ()
history_set_retention(days: u32) → ()

vocabulary_add(word, definition, source_lang, target_lang) → ()
vocabulary_list() → Vec<VocabularyItem>
vocabulary_delete(id) → ()
vocabulary_export(format: ExportFormat) → FilePath
```

### System commands
```
ocr_capture_region() → OcrResult
ocr_from_image(image: ImageData) → OcrResult
tts_list_voices() → Vec<SpeechVoice>
tts_speak(text: String, voice_id: Option<String>) → ()
tts_stop() → ()
dict_lookup(word: String) → Option<String>
dict_list_packages() → Vec<DictPackage>
dict_install_package(path: String) → ()
shortcut_check_conflict(combo: String) → Option<String>
shortcut_save(map: ShortcutMap) → ()
shortcut_reset_defaults() → ()
```

### Keystore commands (retained from Phase 4 + extended)
```
key_status() → Record<String, bool>     // keyed by ProviderProfile UUID (not template ID)
keystore_health() → String
archive_keystore() → Option<String>
reset_keystore() → Option<String>
```

### External API token commands
```
external_api_enable() → String          // generates 32-byte token, stores in keystore; returns token ONCE
external_api_status() → { enabled: bool, port: u16 }  // never returns the token
external_api_disable() → ()             // removes token from keystore, stops server
external_api_regenerate_token() → String // invalidates old token, returns new one ONCE
```

**Token lifecycle:** The token is created by `external_api_enable` or
`external_api_regenerate_token` and returned **exactly once**. It is never
readable again — `external_api_status` returns only enabled/port, never the token.
The token is stored in `keystore.external_api_token` and compared in constant time
on every request.

---

## 7. External HTTP API Contract

Default **off**. When enabled: `127.0.0.1:61742` (port configurable). All routes
require Bearer token auth (32-byte random token, constant-time comparison).

### 7.1 Endpoints & Body Limits

| Method | Path | Auth | Body limit | Description |
|---|---|---|---|---|
| GET | `/v1/health` | ✅ | — | `{ status: "ok", version: "…" }` |
| GET | `/v1/providers` | ✅ | — | `Vec<{ uuid, name, template_id, enabled }>` |
| GET | `/openapi.json` | ✅ | — | OpenAPI 3.1 spec |
| POST | `/v1/translate` | ✅ | 1 MiB | `TranslationSession` (body: `{ text, from?, to, provider_uuid? }`) |
| POST | `/v1/ocr` | ✅ | 24 MiB encoded | `{ text, confidence }` (body: `{ image: base64 }`) |
| POST | `/v1/selection` | ✅ | — | Trigger selection capture |
| POST | `/v1/show-input` | ✅ | — | Show input window |

**OCR limits:** encoded body up to 24 MiB (base64 of ~16 MiB raw image); decoded
image max 16 MiB and 32 megapixels. Requests exceeding these return 413.

### 7.2 Security Rules

- **No CORS:** `Access-Control-Allow-Origin` is never sent. Non-allowed `Origin`
  headers are rejected with 403.
- **Loopback Host check:** requests must have `Host: 127.0.0.1:<port>` or
  `Host: localhost:<port>`. DNS-rebinding attacks via a non-loopback Host are
  rejected with 403.
- **Token:** compared in **constant time** (`subtle::ConstantTimeEq`). Token is
  stored in `keystore.external_api_token`; created once, never re-readable.
- **Content type:** all POST endpoints require `Content-Type: application/json`;
  others are rejected with 415.

### 7.3 Rate Limiting

Sliding-window per-instance, keyed by the single API token (one token = one
client identity): **60 requests/minute** across all endpoints. Exceeding returns
429 with `Retry-After` header. The window is process-lifetime (not persisted).

### 7.4 Error Envelope

All error responses use a unified JSON envelope:

```json
{
  "error": {
    "code": "RATE_LIMITED",
    "message": "Rate limit exceeded. Retry after 42 seconds.",
    "details": null
  }
}
```

| HTTP Status | Code | When |
|---|---|---|
| 400 | `BAD_REQUEST` | Malformed JSON, missing required field |
| 401 | `UNAUTHORIZED` | Missing or invalid Bearer token |
| 403 | `FORBIDDEN` | Non-loopback Host, disallowed Origin |
| 413 | `PAYLOAD_TOO_LARGE` | Body exceeds endpoint limit |
| 415 | `UNSUPPORTED_MEDIA_TYPE` | Missing/wrong Content-Type |
| 429 | `RATE_LIMITED` | Rate limit exceeded |
| 500 | `INTERNAL_ERROR` | Unexpected server error (no user content in message) |
| 503 | `SERVICE_UNAVAILABLE` | OCR/translate engine failed |

### 7.5 Token Lifecycle Commands

See §6 "External API token commands" — `external_api_enable`,
`external_api_status`, `external_api_disable`, `external_api_regenerate_token`.

---

## 8. Data Architecture

### 8.1 SQLite (bundled, Rust-managed)

**Tables:** `providers`, `preferences`, `shortcuts`, `history_sessions`, `history_results`, `vocabulary`, `dict_packages`, `_schema_migrations`.

**Access:** Rust only. Frontend goes through Tauri commands. No `tauri-plugin-sql` or frontend SQLite.

### 8.2 Keystore (retained + versioned)

- **Protocol:** AES-256-GCM + Argon2id, self-encrypted JSON, machine-bound identity, fail-closed, per-dir fs2 flock.
- **Inner structure:** versioned (§5.0): `provider_keys` (by UUID), `history_key` (opt-in 32 bytes), `external_api_token` (opt-in).
- **Database link:** DB `providers.secret_ref` = provider UUID → keystore `provider_keys[uuid]`. No plaintext key in DB.

### 8.3 History Encryption

- **Consent gate:** First launch → explicit prompt: "Enable history?" — no history written until agreed.
- **Key:** On opt-in, generate 32-byte random key → store in `keystore.history_key`.
- **Per-record encryption:** AES-256-GCM with fresh nonce per record. Session UUID
  (generated **before** encryption, not auto-increment DB ID) is used as AAD.
  AES-GCM tag is appended to ciphertext (`ciphertext || 16-byte tag`).
  `crypto_version` field (currently 1) allows future algorithm migration.
- **Encrypted fields:** `source_text` (session), `result_text` + `error_message` (results);
  vocabulary: `word`, `definition`.
- **Plaintext fields:** `timestamp`, `trigger_source`, `languages`, `is_favorite`,
  `provider_uuid`, `engine_id`, `provider_name_snapshot`, `elapsed_ms`,
  `outcome_tag`, `error_kind` (metadata, not content).
- **Retention:** Default 30 days; favorites never expire. Configurable. Background
  cleanup runs on app start; shows a non-intrusive summary badge.
- **Keystore reset:** Warn that history/vocabulary become undecryptable. Archive old
  keystore + DB (`.broken-*`), don't delete.

### 8.4 Encrypted History Search

All history content is encrypted at rest — no plaintext index, no blind index, no
searchable ciphertext. Search is performed as follows:

1. Rust reads candidate records from SQLite in **fixed batches** (e.g. 200 rows)
   using cursor-based pagination (ordered by `timestamp DESC`).
2. Each batch is **decrypted in memory** (using `keystore.history_key`).
3. The search query is matched against decrypted `source_text` and `result_text`
   using case-insensitive + Unicode-normalized substring matching.
4. Matching records are returned to the frontend; the next batch is fetched on demand.
5. **Memory is bounded:** only one batch's decrypted content is in memory at a time.
6. **Database work runs on a blocking worker** (`tauri::async_runtime::spawn_blocking`)
   to avoid blocking the async runtime.
7. **Coverage:** all records within the retention period + all favorites are searched.
   No silent omission for performance.
8. **Corrupt-record tolerance:** if a single record fails to decrypt (tampered,
   wrong key, etc.), it is returned with a `corrupt: true` flag and shown as
   "corrupted entry" in the UI. It does **not** abort the entire query.

### 8.5 Crash-Safe Idempotent Migration Protocol

Migration from Phase 4 (`settings.json` + flat-map keystore) to the new schema:

**Phase 1 — Backup:**
- Copy `settings.json` → `settings.json.bak-pre-migration`.
- Copy keystore → `keystore.json.bak-pre-migration`.
- These backups are never deleted by the app; user can manually remove after verifying.

**Phase 2 — DB migration (idempotent):**
- Create tables if not exist (`CREATE TABLE IF NOT EXISTS`).
- If `providers` is empty and `settings.json` exists: parse `default_provider`,
  `target_language` → insert into `preferences`.
- No provider profiles are created here (they are created in Phase 3 which links keys).

**Phase 3 — Keystore migration (crash-safe, idempotent):**
- Load keystore. If `version` field is absent (legacy flat map):
  1. Read `settings.json` to get the list of provider template IDs the user configured.
  2. For each template ID that has a key in the legacy flat map:
     - Generate a **deterministic UUID v5** from `(NAMESPACE, template_id)` —
       same input always produces the same UUID, so re-running after a crash
       produces the same UUID, making the migration idempotent.
     - Create a `ProviderProfile` row in the DB with this UUID.
     - **Add** the key to `provider_keys[uuid]` in the keystore (copy from legacy key).
     - **Commit** the keystore (write + flush).
     - **Commit** the DB row (transaction).
  3. After ALL profiles + keys are committed: remove legacy template-id keys from
     the keystore flat map and set `version = 2`. Commit keystore.
  4. If a crash occurs at any point: on restart, the migration detects `version`
     is still absent (or partially done). It re-runs: existing UUIDs already in
     `provider_keys` are skipped (idempotent check); profiles already in DB are
     skipped (idempotent check). The cleanup step (3) only runs when all profiles
     are confirmed present.
  5. **key_status()** returns results keyed by profile UUID (not template ID).

**Phase 4 — Verify:**
- Assert: every profile in DB has a matching key in `provider_keys`.
- Assert: no legacy template-id keys remain in the keystore.
- If verification fails: do NOT clean up; leave both backups + partial state; show
  an error banner directing the user to manual recovery.

| Scenario | Behavior |
|---|---|
| Fresh install | Empty DB; no keystore; onboarding flow |
| Upgrade from Phase 4 | Run Phase 1–4 migration (above); idempotent; crash-safe |
| DB corrupt | Archive `.broken-*`; start fresh; keystore intact (re-link profiles manually or re-migrate from settings) |
| Keystore corrupt | Archive `.broken-*`; DB intact but keys dangling; re-enter keys; history undecryptable |
| Both corrupt | Archive both; fresh start |

---

## 9. Privacy Data Flow

| Data | Source | Sent where | Consent required | Stored on disk | Logged |
|---|---|---|---|---|---|
| Source text (translation) | User selection/input/clipboard/API | Active provider endpoint(s) | Implicit (user triggered translate) | Only if history opt-in (encrypted) | **Never** |
| Translated text | Provider response | Shown in UI / returned via API | Implicit | Only if history opt-in (encrypted) | **Never** |
| API key | User enters in settings | Provider endpoint (as auth header) | Implicit (user configured) | Keystore (encrypted) | **Never** |
| History content | Translation results | Nowhere (local only) | Explicit opt-in | SQLite (encrypted) | **Never** |
| OCR text | Screen capture / image | OCR engine (local unless cloud configured) | Screen Recording permission (macOS) | Only if history opt-in (encrypted) | **Never** |
| TTS audio | System speech synthesis | Local audio output | Implicit | Nowhere | **Never** |
| External API requests | HTTP client | Local server processes them | Explicit enable + token | Nowhere | **Never** (rate/size only) |
| Provider metadata | User configuration | Nowhere | Implicit | SQLite (plaintext metadata) | OK (uuid, name, template only) |
| Usage analytics | — | — | — | — | **None collected** |

**Logging rule:** Logs may contain: provider uuids, engine ids, error classifications, timing data, request counts. Logs must **never** contain: source text, translated text, prompts, responses, API keys, auth headers, or any user content.

---

## 10. Platform Differences & Per-Slice Parity

| Feature | macOS | Windows | Parity rule |
|---|---|---|---|
| Selection capture | AX-first + sentinel copy-fallback | Sentinel copy-fallback (no AX equivalent) | Both must work; macOS AX is faster but not required on Windows |
| Clipboard compound restore | objc2 NSPasteboardItem | Win32 OpenClipboard + SetClipboardData | Both restore text+image in one write (verified) |
| Keystore atomic replace | `rename()` | `MoveFileExW` / `ReplaceFileW` | Both atomic (verified) |
| Keystore file ACL | `chmod 600` | `SetNamedSecurityInfoW` (protected DACL) | Both owner-only (verified) |
| Tray / menu-bar | `NSStatusItem` | `SystemTray` (Tauri) | Same menu items, same actions |
| OCR | ScreenCaptureKit + Vision | Windows.Graphics.Capture + Windows OCR | Same UX (region select → OCR → translate) |
| TTS | `AVSpeechSynthesizer` | `SpeechSynthesizer` | Same voice list / speak / stop |
| Dictionary | System dict + StarDict/MDX | StarDict/MDX only | Same offline package format |
| Shortcuts | Global hotkey (same engine) | Global hotkey | Same conflict detection |
| Screen capture permission | Screen Recording prompt (TCC) | GraphicsCaptureAccess (programmatic, requires `GraphicsCaptureAccess.requestAccessAsync`) or GraphicsCapturePicker (system picker) | See Windows capture note below |
| Screen capture approach | ScreenCaptureKit (direct frame access) | GraphicsCaptureItem (programmatic) or GraphicsCapturePicker (user picks a window/screen) | Custom overlay draws on top; capture via the chosen API |

**Windows capture note:** Windows.Graphics.Capture has three access paths:
1. **GraphicsCapturePicker** — system UI, user picks a window or display. Most reliable, least intrusive.
2. **GraphicsCaptureAccess.requestAccessAsync** — programmatic borderless access. Requires package identity (MSIX) or a manifest capability; may not be available for a Tauri (non-MSIX) app.
3. **HWND-based direct capture** — requires `SetWindowDisplayAffinity` exemption or undocumented APIs; not recommended.

LinguaRay uses path **1 (GraphicsCapturePicker)** for the first release: the OCR
overlay triggers the picker, the user selects the screen, and the app captures the
selected region. This avoids identity/manifest complexity. Path 2 is a v1.x
improvement if Tauri's MSIX packaging supports it.

**Per-slice acceptance:** each slice (S3–S6) must be verified on **both** macOS
and Windows — each platform's automated tests (CI) **and** real-machine E2E —
before moving to the next slice.

**Per-slice acceptance:** each slice (S3–S6) must be verified on **both** macOS and Windows before moving to the next slice. Windows CI is a mandatory gate; real-machine testing follows.

---

## 11. Slice Dependencies & Test Gates

```
S0 (Spec) ──freeze──▶ S1 (Design) ──design gate──▶ S2a (Data model + migration)
                                                        │
                                                        ▼
                                                   S2b (History encryption + privacy)
                                                        │
                                                        ▼
                                                   S3 (Shell + Provider + Translate)
                                                        │
                                                        ▼
                                                   S4 (History + Vocabulary + Dictionary)
                                                        │
                                                        ▼
                                                   S5 (OCR)
                                                        │
                                                        ▼
                                                   S6 (TTS + External API + Update)
                                                        │
                                                        ▼
                                                   S7 (Acceptance)
```

**Test gates per slice:**
- **S0:** Spec reviewed + frozen (this document).
- **S1:** All UI Lab screens clickable, i18n (zh/en), light/dark, keyboard-navigable, visually reviewed at target sizes.
- **S2a:** Migration tested (fresh + upgrade + crash-replay); ProviderProfile CRUD unit-tested; keystore versioned structure tested.
- **S2b:** History encryption round-trip tested (session + multi-result); tamper/fail-closed tested; consent gate tested; encrypted search tested (batch decrypt, cursor, corrupt tolerance).
- **S3:**
  - macOS automated: selection/input/clipboard translate, provider CRUD, tray quick-switch, multi-engine consent.
  - macOS real-machine: global hotkey trigger, clipboard restore, AX-first vs copy-fallback.
  - Windows CI: cargo check/clippy/test green; compound clipboard path verified.
  - Windows real-machine: global hotkey trigger, clipboard restore, sentinel copy-fallback.
- **S4:**
  - Both platforms automated: history search/export, vocabulary CRUD + AnkiConnect, dictionary lookup + package install.
  - Both platforms real-machine: dictionary lookup UI flow, history search performance with 1000+ records.
- **S5:**
  - macOS automated: ScreenCaptureKit permission flow mocked; Vision OCR pipeline tested.
  - macOS real-machine: region select → OCR → translate; image/drag/clipboard input.
  - Windows CI: cargo check/clippy/test green for OCR module.
  - Windows real-machine: GraphicsCapturePicker → capture → Windows OCR → translate; permission flow.
- **S6:**
  - Both platforms automated: TTS voice list/speak/stop; external API endpoints with auth + rate limit + body limits; updater manifest generation.
  - Both platforms real-machine: TTS playback; external API from another process; signed update install.
- **S7:** Full Bob/Pot capability matrix closed; both-platform real-machine E2E; privacy audit; cross-version upgrade test.

---

## 12. UI Skill Provenance

| Field | Value |
|---|---|
| Skill | `ui-ux-pro-max` v2.11.0 |
| Source | [github.com/nextlevelbuilder/ui-ux-pro-max-skill](https://github.com/nextlevelbuilder/ui-ux-pro-max-skill) |
| Pinned commit | `14ddef5c05e52d7c253b8f0129de7bcd1045ae5b` |
| Root license | MIT (© 2024 Next Level Builder) — `LICENSE` file |
| **Upstream conflict** | `cli/README.md` states CC-BY-NC-4.0 for the CLI tool. The skill data/scripts (used by LinguaRay) are under the root MIT license, but the CC-BY-NC-4.0 clause on the CLI tool means **skill files must NOT be distributed inside LinguaRay release artifacts**. The skill is a development-time tool only, not a runtime dependency. |
| Install path | `~/.zcode/cli/skills/ui-ux-pro-max/` |
| Install source dir | `.claude/skills/ui-ux-pro-max/` (the actual skill directory in the repo, NOT `src/ui-ux-pro-max/` which is the CLI development source) |
| Install method | Cloned repo at pinned SHA → copied `.claude/skills/ui-ux-pro-max/` contents to skills dir (no floating `main`) |
| SKILL.md | Present at install path (`~/.zcode/cli/skills/ui-ux-pro-max/SKILL.md`); ZCode discovers skills via `SKILL.md` in `~/.zcode/cli/skills/<name>/` |
| Smoke test | `python3 ~/.zcode/cli/skills/ui-ux-pro-max/scripts/search.py "translation" --domain product -n 1` → returned "Translator App" result with style recommendation ✅ |
| Files installed | `SKILL.md`, `data/` (CSV databases), `scripts/` (search.py, core.py, design_system.py), `references/` (quick-reference.md, pro-rules.md) |

**Note on `${CLAUDE_PLUGIN_ROOT}`:** The SKILL.md references this variable for script
paths. ZCode does not set this variable. The scripts use relative paths internally
(they locate `data/` relative to the script file via `__file__`), so invoking them
by absolute path from any working directory works. S1 will verify this when
generating the design system.

**Constraint:** S0 only installs + audits the skill. No `MASTER.md`, design tokens,
mockups, or UI code generated until the state matrix is frozen and S1 begins.

---

## 13. Existing Implementations Retained

The following Phase 4 implementations are retained as the engineering base (not rewritten):

- **Keystore:** AES-256-GCM + Argon2id, self-encrypted JSON, machine-bound, fail-closed, per-dir fs2 flock, atomic replace, owner-only ACL.
- **Clipboard:** Compound restore (macOS objc2 NSPasteboardItem + Windows Win32 compound write), §B sentinel state machine, AX-first capture.
- **CSP:** IPC-only `connect-src`, no wildcard; devCsp with Vite HMR.
- **Capabilities:** Per-window least-privilege via AppManifest::commands.
- **CI:** Windows-check gate (check + clippy + test + real clipboard); release workflow (unsigned dry-run + signed tag).
- **Provider wire contract:** WireParams whitelist, §G classified fallback, reqwest 30s timeout, redirect=none.
- **Cross-process lock:** fs2 per-dir flock + single-instance plugin.
