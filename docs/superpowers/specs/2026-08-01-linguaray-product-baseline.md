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
| **Retention cleanup** | Silent; badge shows "N items older than 30d cleaned" |

### 4.6 Tray / Menu-bar

| State | Display |
|---|---|
| **Normal** | LinguaRay icon; click → menu |
| **Active translation** | Subtle pulse on icon |
| **Error (general)** | Red dot on icon |
| **Update available** | Badge on icon + menu item |

---

## 5. Domain Model

### 5.1 ProviderProfile

```
ProviderProfile {
    uuid: String              // primary key
    template_id: String       // "openai", "anthropic", "google", "custom", etc.
    name: String              // user-editable display name
    protocol: Protocol        // openai_chat | anthropic | gemini | google_translate | custom_http
    endpoint: String          // full URL (base + path)
    model: Option<String>     // model identifier (AI providers)
    enabled: bool             // appears in active selection
    sort_order: i32           // display order
    is_local: bool            // Ollama etc. (no key needed, localhost)
    secret_ref: String        // keystore key (= provider uuid); DB never holds plaintext key
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
    translated_text: String
    elapsed_ms: u64
    error: Option<ErrorClassification>
}

ErrorClassification {
    kind: ErrorKind           // Network | Timeout | RateLimit | ServerError | AuthFailed | InvalidRequest | ParseError
    fallback_eligible: bool   // true for Network/Timeout/RateLimit/ServerError/ParseError
    message: String
}
```

### 5.3 HistoryRecord

```
HistoryRecord {
    id: i64                   // auto-increment
    timestamp: i64            // Unix epoch
    source_text_encrypted: Vec<u8>   // AES-256-GCM
    source_text_nonce: [u8; 12]
    result_text_encrypted: Vec<u8>
    result_text_nonce: [u8; 12]
    provider_uuid: String     // plaintext (metadata, not content)
    engine_id: String
    detected_language: Option<String>
    target_language: String
    is_favorite: bool
}
```

### 5.4 VocabularyItem

```
VocabularyItem {
    id: i64
    timestamp: i64
    word_encrypted: Vec<u8>
    word_nonce: [u8; 12]
    definition_encrypted: Vec<u8>
    definition_nonce: [u8; 12]
    source_language: String
    target_language: String
}
```

---

## 6. Tauri Command Boundary

Frontend ↔ Rust via Tauri IPC. Frontend **cannot** read plaintext API keys or access SQLite directly.

### Provider commands
```
provider_list() → Vec<ProviderProfile>
provider_create(template_id, name, endpoint, model) → ProviderProfile
provider_update(uuid, fields) → ProviderProfile
provider_duplicate(uuid) → ProviderProfile
provider_delete(uuid) → ()
provider_reorder(uuids: Vec<String>) → ()
provider_toggle(uuid, enabled) → ()
provider_set_key(uuid, key) → ()       // writes to keystore only
provider_get_models(uuid) → Vec<ModelInfo>
provider_test_connection(uuid) → ConnectionResult
provider_set_active(uuid) → ()
```

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

### Keystore commands (retained from Phase 4)
```
key_status() → Record<String, bool>
keystore_health() → String
archive_keystore() → Option<String>
reset_keystore() → Option<String>
```

---

## 7. External HTTP API Contract

Default **off**. When enabled: `127.0.0.1:61742` (port configurable). Bearer token auth (32-byte random token generated at enable time). Request body size limit: 1 MB. Rate limit: 60 req/min.

```
GET  /v1/health              → { status: "ok", version: "…" }
GET  /v1/providers           → Vec<{ uuid, name, template_id, enabled }>
POST /v1/translate           → TranslationSession   (body: { text, from?, to, provider_uuid? })
POST /v1/ocr                 → { text, confidence }  (body: { image: base64 })
POST /v1/selection           → ()                    (trigger selection capture)
POST /v1/show-input          → ()                    (show input window)
```

OpenAPI spec served at `/openapi.json`.

---

## 8. Data Architecture

### 8.1 SQLite (bundled, Rust-managed)

**Tables:** `providers`, `preferences`, `shortcuts`, `history`, `vocabulary`, `dict_packages`, `_schema_migrations`.

**Access:** Rust only. Frontend goes through Tauri commands. No `tauri-plugin-sql` or frontend SQLite.

**Migration:** Idempotent migration from existing `settings.json` + provider-id keys. Backup before migration. On failure, old data remains recoverable.

### 8.2 Keystore (retained from Phase 4)

- **Protocol:** AES-256-GCM + Argon2id, self-encrypted JSON, machine-bound identity, fail-closed, per-dir fs2 flock.
- **Contents:** API keys (keyed by provider UUID), history encryption key (opt-in, 32 bytes), reserved fields.
- **Database link:** DB stores `secret_ref` (= provider UUID) pointing to the keystore entry. No plaintext key in DB.

### 8.3 History Encryption

- **Consent gate:** First launch → explicit prompt: "Enable history?" — no history written until agreed.
- **Key:** On opt-in, generate 32-byte random key → store in keystore reserved field.
- **Per-record encryption:** AES-256-GCM with fresh nonce per record, record ID as AAD.
- **Encrypted fields:** `source_text`, `result_text` (vocabulary: `word`, `definition`).
- **Plaintext fields:** `timestamp`, `provider_uuid`, `engine_id`, `languages`, `is_favorite` (metadata, not content).
- **Retention:** Default 30 days; favorites never expire. Configurable. Cleanup runs on app start.
- **Keystore reset:** Warn that history/vocabulary become undecryptable. Archive old keystore + DB (`.broken-*`), don't delete.

### 8.4 Migration & Recovery

| Scenario | Behavior |
|---|---|
| Fresh install | Empty DB; no keystore; onboarding flow |
| Upgrade from Phase 4 | Migrate `settings.json` → `preferences` table; provider-id keys → keystore (unchanged); backup first |
| DB corrupt | Archive `.broken-*`; start fresh; keystore intact |
| Keystore corrupt | Archive `.broken-*`; DB intact but `secret_ref` entries dangling; re-enter keys; history undecryptable |
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
| Screenshot permission | Screen Recording prompt | Windows.Graphics.Capture permission flow | Each platform's native permission UX |

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
- **S2a:** Migration tested (fresh + upgrade); ProviderProfile CRUD unit-tested.
- **S2b:** History encryption round-trip tested; tamper/fail-closed tested; consent gate tested.
- **S3:** Selection/input/clipboard translate E2E on macOS; provider CRUD UI; tray quick-switch; multi-engine consent flow.
- **S4:** History search/export; vocabulary CRUD + AnkiConnect export; dictionary lookup + package install.
- **S5:** Region select → OCR → translate on both platforms; permission flows; image/drag/clipboard input.
- **S6:** TTS play/stop; external API endpoints with auth + rate limit; updater manifest generation.
- **S7:** Full Bob/Pot capability matrix closed; both-platform E2E; privacy audit; upgrade test.

---

## 12. UI Skill Provenance

| Field | Value |
|---|---|
| Skill | `ui-ux-pro-max` v2.11.0 |
| Source | [github.com/nextlevelbuilder/ui-ux-pro-max-skill](https://github.com/nextlevelbuilder/ui-ux-pro-max-skill) |
| Pinned commit | `14ddef5c05e52d7c253b8f0129de7bcd1045ae5b` |
| License | MIT (© 2024 Next Level Builder) |
| Install path | `~/.zcode/cli/skills/ui-ux-pro-max/` |
| Install method | Cloned repo at pinned SHA → copied to skills dir (no floating `main`) |
| Smoke test | `python3 scripts/search.py "translation tool" --domain product -n 2` → returned 2 relevant results ✅ |
| Files verified | `skill.json`, `CLAUDE.md`, `scripts/search.py`, `scripts/core.py`, `data/`, `templates/` |

**Constraint:** S0 only installs + audits the skill. No `MASTER.md`, design tokens, mockups, or UI code generated until the state matrix is frozen and S1 begins.

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
