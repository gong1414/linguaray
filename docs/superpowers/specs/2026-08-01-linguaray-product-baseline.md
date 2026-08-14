# LinguaRay — Product Baseline Spec (S0 Freeze)

**Status:** Approved — S0 Frozen · **Date:** 2026-08-01
**Supersedes:** [2026-07-30-linguaray-v1-design.md](./2026-07-30-linguaray-v1-design.md) (the v1 translation-core scope; its security/clipboard/CI/signing implementations are retained as the engineering base)

**Erratum (2026-08-14, plugin-core rev-4):**

1. Official in-tree Capability/Driver plugins supersede the sentence
   “v1 has no plugin system” for *first-party* code. Third-party SDK,
   Bob/Pot plugin compatibility, and WASM remain 🔜.
2. Official AI catalog is the 21 ids in
   `docs/superpowers/specs/2026-08-14-linguaray-plugin-core-design.md` §7.3,
   with support tiers `ready` / `setup_required` / `unverified`.
   The 8-name list below is historical. 30+ relay long-tail remains 🔜.
   Custom still covers user-supplied OpenAI-compatible / Anthropic endpoints.

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
| **Saving** | Spinner on save button; inputs disabled |
| **Save failed** | Error toast: "Failed to save: {reason}" |
| **Save conflict** | Error: "This provider was modified elsewhere. Reload?" |
| **Delete confirm** | Dialog: "Delete {name}? History references are preserved." |
| **Deleting** | Card greyed out + spinner; disabled |
| **Delete retry** | Card shows "Delete failed — retry?" |
| **Drag-to-reorder** | Drag handle; visual indicator on hover/drag |
| **Reorder persist failed** | Toast: "Failed to save order — reverted" |
| **Balance loading** | Spinner where balance would show |
| **Balance unsupported** | "—" (no balance for this provider) |
| **Balance rate-limited** | "Rate limited — try later" |
| **Balance error** | "Error fetching balance" |
| **Endpoint invalid** | Red border on endpoint field: "Must be HTTPS (or localhost)" |
| **Model manual entry** | Text input visible when fetch fails or is unsupported |

### 4.4 OCR Overlay

| State | Display |
|---|---|
| **Initial** | Dimmed screen + crosshair |
| **Selecting** | Bright rectangle follows cursor |
| **Capturing** | Flash + spinner |
| **OCR processing** | Small spinner at selection |
| **Success** | Translation popup at selection |
| **Error (no text)** | "No text recognized" |
| **Error (permission)** | macOS: "Grant Screen Recording permission". Windows: "Capture unavailable" (protected content / unsupported session / remote desktop) |
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
| **All success** | All cards filled, displayed in the user's Provider sort order (NOT by elapsed time); completed cards do not jump position as others arrive |
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
| **External API on** | Toggle on; port display; "Regenerate token" (shows new token once in a modal); "Disable" (token NOT copyable from this view) |

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
| **Enabled** | "External API: On (port {port})" + "Regenerate token" + "Disable" (token NOT shown or copyable from this state) |
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
    provider_keys: Map<String, String>,    // keyed by secret_ref (legacy_id for migrated, "provider/<uuid>" for new)
    history_key: Option<[u8; 32]>,         // opt-in history encryption key
    external_api_token: Option<String>,    // opt-in; base64url (no padding) of 32 random bytes
}
```

**Legacy flat-map migration:** On first load after upgrade, if `version` is absent
(legacy), the flat map is treated as `provider_keys` with template-id keys. The
migration protocol (§8.5) converts the flat map to `provider_keys` without renaming keys.

### 5.1 ProviderProfile

```
ProviderProfile {
    uuid: String              // deterministic UUID v5 for migrated: UUIDv5(NAMESPACE_LINGUARAY,
                              //   "linguaray:legacy-provider:" + legacy_id); UUID v4 for user-created
    template_id: String       // "openai", "anthropic", "google", "custom", etc.
    name: String              // user-editable display name
    protocol: Protocol        // openai_chat | anthropic | gemini | google_translate | custom_http
    endpoint: String          // full URL (base + path); HTTPS-only except loopback
    model: Option<String>     // model identifier (AI providers)
    enabled: bool             // appears in active selection
    sort_order: i32           // display order
    is_local: bool            // Ollama etc. (no key needed, localhost)
    secret_ref: String        // keystore key name. Migrated profiles: = legacy_id (e.g. "openai").
                              // New profiles: "provider/<uuid>". NOT necessarily == uuid.
    capabilities: ProviderCapabilities  // balance, quota, model_list, etc.
    status: ProviderStatus    // Active | Deleting | Deleted (tombstone)
                              // provider_list() returns only Active by default;
                              // Deleted profiles retained for history name_snapshot.
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
    message: String
    // fallback_eligible is DERIVED from kind via ErrorKind::is_fallback_eligible(),
    // not an independent field. Network/Timeout/RateLimit/ServerError/ParseError → true;
    // AuthFailed/InvalidRequest → false. This prevents a contradiction between
    // kind and fallback_eligible.
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
    crypto_version: u32               // 1 (AES-256-GCM); allows future algorithm changes
}

history_results {
    result_uuid: String       // app-generated UUID v4, created BEFORE encryption
    session_uuid: String      // FK → history_sessions.session_uuid
    provider_uuid: String     // may dangle if provider deleted
    provider_name_snapshot: String  // display name captured at translation time
                                    // (survives provider deletion; no dangling blank)
    engine_id: String
    elapsed_ms: u64
    outcome_tag: String       // "success" | "failure"
    result_text_encrypted: Option<Vec<u8>>   // Some on success
    result_text_nonce: Option<[u8; 12]>
    error_kind: Option<String>              // Some on failure (plaintext; derived from ErrorKind)
    error_message_encrypted: Option<Vec<u8>> // encrypted error detail
    error_message_nonce: Option<[u8; 12]>
    crypto_version: u32
}
```

**AAD domain separation (prevents ciphertext swap within or across sessions):**
Each encrypted field uses a domain-specific AAD string:
- Session source text: `"linguaray-history-v1|session|<session_uuid>|source"`
- Result success text: `"linguaray-history-v1|result|<result_uuid>|text"`
- Result error message: `"linguaray-history-v1|result|<result_uuid>|error"`
- Vocabulary word: `"linguaray-vocab-v1|item|<item_uuid>|word"`
- Vocabulary definition: `"linguaray-vocab-v1|item|<item_uuid>|definition"`

This ensures a ciphertext from one field/context cannot be swapped into another
and still pass GCM authentication.

**Key design decisions:**
- UUIDs are generated **before** encryption and used as AAD — never use
  auto-increment DB IDs for crypto binding (they don't exist until after insert).
- AES-GCM tag is **appended** to ciphertext (`ciphertext || 16-byte tag`).
- `crypto_version` field allows future algorithm migration without schema change.
- `provider_name_snapshot` is on **each result** (not the session), because a
  multi-engine session has results from different providers.
- `fallback_eligible` is **derived from `ErrorKind`** at the wire layer
  (`ErrorKind::is_fallback_eligible()`), never an independent input field that
  could contradict `kind`.

### 5.4 VocabularyItem

```
vocabulary {
    item_uuid: String         // UUID v4, generated before encryption; used as AAD
    timestamp: i64
    source_language: String
    target_language: String   // plaintext metadata
    word_encrypted: Vec<u8>   // AES-256-GCM (ciphertext || tag)
    word_nonce: [u8; 12]
    // AAD = "linguaray-vocab-v1|item|<item_uuid>|word" (domain-separated, see §5.3)
    definition_encrypted: Vec<u8>
    definition_nonce: [u8; 12]
    // AAD = "linguaray-vocab-v1|item|<item_uuid>|definition" (domain-separated, see §5.3)
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
provider_set_key(uuid, key) → ()       // writes to keystore.provider_keys[<profile.secret_ref>] only
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

**Provider delete state machine (crash-safe):**
1. DB: mark profile `status = 'deleting'`, set `enabled = false`. Immediately remove
   from `primary`, `parallel`, `fallback` in preferences (so it can't be invoked).
   Commit DB.
2. Keystore: remove `provider_keys[secret_ref]`. Commit keystore.
3. DB: convert profile to **tombstone** (`status = 'deleted'`, name retained as
   `"deleted: <original_name>"` for any UI reference). Commit DB.
4. History: no action needed — `history_results.provider_name_snapshot` already
   captured the display name at translation time.
5. **Crash recovery:** on startup, any profile with `status = 'deleting'` resumes
   at step 2 (re-attempt keystore removal). A profile in `deleting` state is never
   callable (enabled=false, removed from active slots). It cannot revert to a
   usable state — the state machine only progresses forward.

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
history_search(query: String, cursor: Option<String>) → HistoryPage
history_list(cursor: Option<String>, favorites_only: bool) → HistoryPage
history_delete(session_uuids: Vec<String>) → ()
history_toggle_favorite(session_uuid: String) → ()
history_clear(before_timestamp: Option<i64>) → ()
history_export(format: ExportFormat, filter: HistoryFilter) → FilePath
history_set_enabled(enabled: bool) → ()
history_set_retention(days: u32) → ()

// HistoryPage: cursor-based pagination for both search and list.
// `next_cursor` = None when all matching records have been scanned.
// `scan_complete` = false means more batches MAY contain matches (search only).
HistoryPage {
    items: Vec<HistorySessionSummary>,
    next_cursor: Option<String>,
    scan_complete: bool,       // true when all retained records + favorites scanned
}

vocabulary_add(word, definition, source_lang, target_lang) → ()
vocabulary_list(cursor: Option<String>) → VocabularyPage
vocabulary_delete(item_uuid: String) → ()
vocabulary_export(format: ExportFormat) → FilePath
```

**History search normalization:** Unicode NFKC + case folding (NFKC_Casefold
semantics; the specific Rust crate is decided at S2a implementation). Batch size
is fixed at **200** (not configurable; not a parameter). Search covers all records
within the retention period + all favorites regardless of age.

**`HistoryPage.next_cursor`:** opaque cursor encoding the last scanned
`(timestamp, session_uuid)` pair. The frontend must not parse or construct it;
it passes it back verbatim to fetch the next batch.

**AnkiConnect export:** Sends decrypted vocabulary items to `127.0.0.1:8765` via
the AnkiConnect API. Decrypted content exists only in memory during the export
request; no temporary plaintext file is written. If the request fails, the error
is surfaced to the user; no plaintext persists.

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
external_api_enable(port: Option<u16>) → String  // bind socket → write token+prefs → start server; returns token ONCE
external_api_status() → ExternalApiStatus        // never returns the token
external_api_disable() → ()                      // removes token from keystore, stops server
external_api_regenerate_token() → String         // invalidates old token, returns new one ONCE
```

```
ExternalApiStatus =
    | Disabled
    | Enabled { port: u16 }
    | PortInUse { configured_port: u16 }  // token retained; user must change port
```

**Token format:** 32 random bytes, base64url-encoded (no padding).

**Enable sequence (crash-safe ordering):**
1. Validate and bind the TCP socket to `127.0.0.1:<port>` (default 61742).
   If bind fails → return error; do NOT write to keystore or preferences.
2. Write token to `keystore.external_api_token` + port + `enabled = true` to `preferences`.
3. Start the HTTP server on the bound socket.
4. If keystore write fails → close the socket; return error; no "enabled" state persists.
5. If server start fails → close the socket; remove token from keystore; set
   `enabled = false` in preferences; return error. State reverts to Disabled.

**Port-in-use recovery:** on restart, if the configured port is occupied, the
server does NOT start. `external_api_status()` returns `PortInUse`. The token is
retained in the keystore. The user calls `external_api_enable(new_port)` to rebind.

**Token lifecycle:** Created by `external_api_enable` or `external_api_regenerate_token`,
returned **exactly once**. Never readable again — `external_api_status` returns only
status + port, never the token. Compared in constant time on every request.

**Regenerate sequence:** generate new 32-byte token → write to keystore (atomic) →
atomically replace the in-memory token used by the running server. If keystore
write fails, the old token continues to be accepted (no disruption). The new token
is returned to the caller only on success.

**Token format in keystore:** stored as base64url-encoded string (no padding) in
`keystore.external_api_token`.

**Origin policy:** **Reject any request with an `Origin` header.** No CORS support.
LinguaRay's external API is for local scripts/tools, not browser clients.

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
- **Database link:** DB `providers.secret_ref` is the stable lookup key into `keystore.provider_keys[secret_ref]`. No plaintext key in DB.

### 8.3 History Encryption

- **Consent gate:** First launch → explicit prompt: "Enable history?" — no history written until agreed.
- **Key:** On opt-in, generate 32-byte random key → store in `keystore.history_key`.
- **Per-record encryption:** AES-256-GCM with fresh nonce per record. Each encrypted
  field uses a domain-separated AAD string (see §5.3 for the exact AAD formats).
  UUIDs are generated **before** encryption (not auto-increment DB IDs) and embedded
  in the AAD. AES-GCM tag is appended to ciphertext (`ciphertext || 16-byte tag`).
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

1. Rust reads candidate records from SQLite in **fixed batches** (200 rows)
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

Migration from Phase 4 (`settings.json` + flat-map keystore) to the new schema.
The key principle: **`secret_ref` is stable across the migration** — the legacy
key name IS the `secret_ref`, so the DB and keystore never need a coordinated
rename. The keystore's atomic rewrite changes the envelope format, not the key names.

**Phase 1 — Backup:**
- Copy `settings.json` → `settings.json.bak-pre-migration`.
- Copy keystore → `keystore.json.bak-pre-migration`.
- These backups are never deleted by the app.

**Phase 2 — DB schema (idempotent):**
- `CREATE TABLE IF NOT EXISTS` for all tables.
- If `preferences` is empty and `settings.json` exists: parse `default_provider`,
  `target_language`, `fallback_engine` → insert into `preferences`.

**Phase 3 — Profile + key migration (crash-safe, idempotent):**

The key insight: enumerate **all** keys in the legacy flat map (not just
`settings.json` defaults — a user may have saved a key for a provider they
haven't set as default). For each legacy key:

1. `legacy_id` = the key name in the flat map (e.g. `"openai"`, `"anthropic"`).
2. Generate a **deterministic UUID v5**: `UUIDv5(NAMESPACE_LINGUARAY, "linguaray:legacy-provider:" + legacy_id)`.
   - Re-running after a crash produces the same UUID → idempotent.
3. `secret_ref = legacy_id` (unchanged — the keystore key name stays the same).
   - New user-created profiles use `secret_ref = "provider/<uuid>"`.
4. Insert the `ProviderProfile` row in a single DB transaction (with
   `secret_ref = legacy_id`). If the row already exists (crash recovery),
   the insert is skipped (`INSERT OR IGNORE`).
5. **Crash safety:** at this point the DB has the profile, and the keystore still
   has the key under the legacy flat map under the same name. A reader can find
   the key via `secret_ref` whether the keystore is v1 (flat map) or v2
   (`provider_keys`). There is no half-state window.

**Phase 4 — Keystore atomic rewrite (single operation):**
- After ALL profiles are committed to the DB:
- Load the keystore. If `version` is absent (legacy flat map):
  1. Copy every `{key: value}` from the flat map into `provider_keys` under the
     **same key name** (no rename). `secret_ref` in the DB already matches.
  2. Set `version = 2`. Clear the legacy flat map fields.
  3. **Atomic write** the keystore (the existing atomic-replace + fs2 flock from
     Phase 4 applies). This is a single indivisible operation — the keystore is
     either fully v1 or fully v2, never mixed.
  4. A crash before this write: keystore is still v1 flat map; DB profiles have
     `secret_ref = legacy_id`; reader finds keys in the flat map. Re-run detects
     `version` absent → re-runs Phase 3 (idempotent) → re-attempts Phase 4.
  5. A crash after this write: keystore is v2; `provider_keys[legacy_id]` has
     the key; DB `secret_ref = legacy_id` matches. Migration is structurally
     complete.

**Phase 5 — Verify + complete flag:**
- Read the v2 keystore back. Assert every DB profile's `secret_ref` exists in
  `provider_keys`.
- Write `_schema_migrations.migration_complete = true`.
- If verification fails: leave both backups + partial state; show error banner.

| Scenario | Behavior |
|---|---|
| Fresh install | Empty DB; no keystore; onboarding flow |
| Upgrade from Phase 4 | Run Phase 1–5 (above); idempotent; crash-safe; no mixed-state window |
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
| External API token | Generated by app (32 random bytes) | Compared in constant time on each API request | Explicit enable | Keystore (encrypted) | **Never** |
| History content | Translation results | Nowhere (local only) | Explicit opt-in | SQLite (encrypted) | **Never** |
| OCR text | Screen capture / image | OCR engine (local unless cloud configured) | Screen Recording permission (macOS) | Only if history opt-in (encrypted) | **Never** |
| TTS audio | System speech synthesis | Local audio output | Implicit | Nowhere | **Never** |
| External API requests | HTTP client | Local server processes them | Explicit enable + token | Nowhere | **Never** (rate/size only) |
| AnkiConnect export | Vocabulary items | `127.0.0.1:8765` (user-initiated only) | Implicit (user clicked export) | Nowhere (in-memory only during request) | **Never** |
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
| OCR | ScreenCaptureKit + Vision | DXGI Desktop Duplication + Windows.Media.Ocr | Same UX (region select → OCR → translate) |
| TTS | `AVSpeechSynthesizer` | `SpeechSynthesizer` | Same voice list / speak / stop |
| Dictionary | System dict + StarDict/MDX | StarDict/MDX only | Same offline package format |
| Shortcuts | Global hotkey (same engine) | Global hotkey | Same conflict detection |
| Screen capture permission | Screen Recording prompt (TCC) | DXGI Desktop Duplication (no picker; overlay-based region select) | See Windows capture note below |
| Screen capture approach | ScreenCaptureKit (direct frame access) | DXGI Desktop Duplication API + transparent overlay per monitor | Custom overlay draws on top; capture via DXGI |

**Windows capture note:** GraphicsCapturePicker only lets the user select a whole
window or display — it cannot provide Bob/Pot-style arbitrary rectangular region
selection. For the baseline:

1. **Tauri creates a transparent overlay window on each monitor.** The user draws
   a rectangle directly on the overlay (same UX as macOS).
2. **Overlay hides after selection.** The app captures the selected monitor(s)
   using **DXGI Desktop Duplication API** (`IDXGIOutputDuplication`), which provides
   per-monitor frame buffers without requiring GraphicsCapturePicker or MSIX identity.
3. **Cross-monitor selection:** if the rectangle spans multiple monitors, each
   monitor's frame is captured separately and cropped to the intersection, then
   stitched by physical-pixel coordinates.
4. **Output:** all frames are converted to BGRA8/sRGB before being passed to
   Windows OCR (`Windows.Media.Ocr`).
5. **Error cases:** DRM-protected content, remote desktop sessions, or capture-unavailable
   drivers return a clear error message to the user. No fabricated persistent permission
   prompt is shown (DXGI Desktop Duplication does not require a user-grant dialog).
6. **GraphicsCapturePicker** may be added in a future version as a separate
   "capture window/screen" mode, but it is NOT used for region OCR in the baseline.

**Per-slice acceptance:** each slice (S3–S6) must be verified on **both** macOS
and Windows — each platform's automated tests (CI) **and** real-machine E2E —
before moving to the next slice.

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
  - Windows real-machine: overlay → DXGI capture/crop/stitch → Windows.Media.Ocr → translate; protected-content/remote-desktop error handling.
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
| **Upstream license conflict** | `cli/README.md` states CC-BY-NC-4.0 for the CLI tool. Upstream metadata is inconsistent; the legal status is not self-adjudicated here. The skill is **development-time only, installed on the developer's machine, and never distributed inside LinguaRay release artifacts**. |
| Install path | `~/.zcode/cli/skills/ui-ux-pro-max/` |
| Install source dir | `.claude/skills/ui-ux-pro-max/` (the actual skill directory in the repo) |
| Install method | Cloned repo at pinned SHA → copied `.claude/skills/ui-ux-pro-max/` contents to skills dir |
| Platform adaptation | The original `SKILL.md` uses `${CLAUDE_PLUGIN_ROOT}` (a Claude Code variable ZCode does not set). A **path-adapted copy** was generated: all `${CLAUDE_PLUGIN_ROOT}/.claude/skills/ui-ux-pro-max` references replaced with the absolute install path `/Users/daoyu/.zcode/cli/skills/ui-ux-pro-max`. Original preserved as `SKILL.md.orig`. |
| Adapted SKILL.md SHA | `e28f987cf4230eb3ade0a339bf8530bebcfd4fc8` |
| Original SKILL.md SHA | `1358d9cf81e9a7ee973508b1744bd0938a009a93` |
| Adaptation scope | Path substitution only (11 occurrences of `${CLAUDE_PLUGIN_ROOT}` → `/Users/daoyu/.zcode/cli/skills/ui-ux-pro-max`); no logic, data, or structural changes |
| Smoke test (repo root) | `python3 ~/.zcode/cli/skills/ui-ux-pro-max/scripts/search.py "desktop productivity" --domain product -n 1` → "Productivity Tool" result ✅ |
| Smoke test (/tmp) | `python3 ~/.zcode/cli/skills/ui-ux-pro-max/scripts/search.py "minimal" --domain style -n 1` → "Minimalism" result ✅ |
| Files installed | `SKILL.md` (adapted), `SKILL.md.orig` (upstream), `data/` (CSV databases), `scripts/` (search.py, core.py, design_system.py), `references/` (quick-reference.md, pro-rules.md) |

**Skill discovery:** ZCode discovers skills via `SKILL.md` in `~/.zcode/cli/skills/<name>/`.
The adapted `SKILL.md` has valid frontmatter (`name: ui-ux-pro-max`, `description: ...`).

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
