Archived-on: 2026-08-14 · reason: superseded by linguaray-plugin-core-design / completed, see git history

# Phase 3: Built-in Traditional Engine + Fallback Chain + System Dictionary — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Land the first built-in traditional MT engine (Google — keyless, simplest), wire the §G classified fallback chain (AI `FallbackEligible` error → automatically retry a built-in engine), and add macOS system-dictionary lookup. This proves the trad-engine + fallback + dict architecture end-to-end; subsequent engines (DeepL/百度/有道/…) are then a repeatable pattern (one module each).

**Architecture:** A `TraditionalEngine` trait (the engines module from Phase 1's stub) that each built-in engine implements; a `google.rs` impl; a `dict.rs` macOS `DCSCopyTextDefinition` wrapper. The `translate_service` gains a fallback step: on `Error::FallbackEligible`, it retries once with a configured fallback traditional engine (per §G: whole-request re-try, no mixing; local mode never silently degrades to remote; cross-remote fallback opt-in via a setting).

**Tech Stack:** Rust 1.95 · `objc2-core-services` (macOS Dictionary Services) or manual FFI to `DCSCopyTextDefinition` · existing `reqwest`/`service`/`error`/`providers`/`settings`/`popup`. Frontend: a small settings addition (fallback engine selector).

**Spec reference:** `docs/superpowers/specs/2026-07-30-linguaray-v1-design.md` (§G fallback classification — `FallbackEligible` (429/5xx/network/parse) → may fall back; Config (missing-key/401/403/invalid-model/keystore) → Settings, no fallback; local mode sacred; cross-remote opt-in; §E system-dict integration for word lookups).

**Facts verified upfront (2026-07-30):** macOS dict = `DCSCopyTextDefinition` (CoreServices), Rust via `objc2-core-services` crate or manual `extern "C"` FFI; returns plain-text definition. The central pot plugin repo `pot-app/pot-plugins` is GONE (404) but **each engine is a separate repo** (`pot-app/pot-app-translate-plugin-template` + `pot-app-translate-plugin-{google,baidu,youdao,...}`) and most are still accessible — leverage ① (port JS→Rust) still works, source is just distributed.

---

## File Structure

**Create:**
- `src-tauri/src/engines/mod.rs` — REPLACE the Phase-1 stub: define `TraditionalEngine` trait + `registry()` returning concrete engines + `find(id)`.
- `src-tauri/src/engines/google.rs` — first trad engine. Google Translate's free `translate.google.com/translate_a/single` endpoint (no key). Ported from pot's google plugin logic (request construction + response parse).
- `src-tauri/src/dict.rs` — macOS `DCSCopyTextDefinition` FFI (word → plain-text definition); no-op stub on non-macOS.
- `src-tauri/tests/google.rs` — wiremock test of the google engine's request construction + response parsing.
- `src-tauri/tests/fallback.rs` — service fallback test (AI FallbackEligible → trad engine called).

**Modify:**
- `src-tauri/src/service.rs` — add `translate_with_fallback`: try primary (AI), on `FallbackEligible` retry with the configured fallback trad engine (per §G rules); new `dictionary_lookup` path.
- `src-tauri/src/settings.rs` — add `fallback_engine: Option<String>` (default `"google"`).
- `src-tauri/src/lib.rs` — a `lookup_dictionary` command; wire fallback into the selection/input/clipboard paths (or keep translate_default as the single fallback-aware entrypoint).
- `src-tauri/Cargo.toml` — add `objc2-core-services` (macOS) if used; else manual FFI (no dep).

---

## Task 1: TraditionalEngine trait + registry

**Files:** Modify `src-tauri/src/engines/mod.rs` (replace stub); modify `src-tauri/src/lib.rs` (if it references the old `registry() -> Vec<()>`)

- [ ] **Step 1: Replace `src-tauri/src/engines/mod.rs`** with:

```rust
//! Built-in traditional MT engines (spec: AI-failure fallback + system-dict).
//! Each engine is a Rust module ported from the corresponding pot-app plugin's
//! JS (leverage ①: turn reversing into JS→Rust porting). v1 ships Google first;
//! others (DeepL/百度/有道/…) follow the same pattern.

pub mod google;

/// A built-in traditional MT engine. Unlike providers (config-driven), these are
/// compiled-in Rust modules implementing their own request construction.
#[async_trait::async_trait]
pub trait TraditionalEngine: Send + Sync {
    fn id(&self) -> &str;
    fn label(&self) -> &str;
    /// Whether the user must supply credentials (Google free: false).
    fn needs_key(&self) -> bool { false }
    /// Translate. `client` is the shared reqwest client (redirect policy none).
    async fn translate(
        &self,
        client: &reqwest::Client,
        text: &str,
        from: &str,
        to: &str,
    ) -> Result<String, crate::error::Error>;
}

/// The static registry of built-in traditional engines.
pub fn registry() -> Vec<Box<dyn TraditionalEngine>> {
    vec![Box::new(google::Google)]
}

pub fn find(id: &str) -> Option<Box<dyn TraditionalEngine>> {
    registry().into_iter().find(|e| e.id() == id)
}
```

- [ ] **Step 2: Add `async-trait` dep.** In `src-tauri/Cargo.toml` `[dependencies]` add `async-trait = "0.1"`.

- [ ] **Step 3: cargo check (expect: google module missing → that's Task 2).** Run `cargo check`. Expected: an error about `google::Google` not found / module not existing. That's by design — Task 2 creates it. Do NOT commit yet.

## Task 2: Google engine (ported from pot plugin) + wiremock test

**Files:** Create `src-tauri/src/engines/google.rs`, `src-tauri/tests/google.rs`

> **Implementer note (the porting step):** FIRST read the pot google plugin source to port it accurately. Clone/fetch `https://github.com/pot-app/pot-app-translate-plugin-template` for the contract shape, and find the google plugin (search GitHub `pot-app-translate-plugin-google` or `pot-app pot google translate`). Extract: the request URL + query params (the `/translate_a/single` endpoint with `client=gtx`, `dt=t`, sl/tl/q), the response shape (nested array `[[["translation","original",...],...],...]`), and the JSON-path to extract the translated text. Port that logic to Rust below. If the google plugin repo is inaccessible, reconstruct from the well-documented public endpoint (this is widely-known, not a reverse-engineering secret): `GET https://translate.google.com/translate_a/single?client=gtx&sl=<from>&tl=<to>&dt=t&q=<text>`.

- [ ] **Step 1: Create `src-tauri/src/engines/google.rs`:**

```rust
//! Google Translate (free, keyless) — ported from pot's google plugin logic.
//! Endpoint: translate.google.com/translate_a/single (client=gtx, dt=t).
//! Response: nested JSON array; translated segments are at [0][*][0].
use async_trait::async_trait;
use crate::error::{Error, FallbackKind};

pub struct Google;

#[async_trait]
impl super::TraditionalEngine for Google {
    fn id(&self) -> &str { "google" }
    fn label(&self) -> &str { "Google Translate" }

    async fn translate(
        &self,
        client: &reqwest::Client,
        text: &str,
        from: &str,
        to: &str,
    ) -> Result<String, Error> {
        let sl = if from == "auto" { "auto".to_string() } else { from.to_string() };
        let resp = client
            .get("https://translate.google.com/translate_a/single")
            .query(&[
                ("client", "gtx"), ("sl", &sl), ("tl", to),
                ("dt", "t"), ("q", text),
            ])
            .send().await
            .map_err(|e| Error::FallbackEligible(FallbackKind::Network(e.to_string())))?;
        let status = resp.status().as_u16();
        if status == 429 || (500..600).contains(&status) {
            return Err(Error::FallbackEligible(FallbackKind::ProviderStatus { status }));
        }
        if !resp.status().is_success() {
            return Err(Error::FallbackEligible(FallbackKind::ProviderStatus { status }));
        }
        let json: serde_json::Value = resp.json().await
            .map_err(|e| Error::FallbackEligible(FallbackKind::Parse(e.to_string())))?;
        // Response shape: [ [ ["translated","orig",...], ... ], ... ].
        // Concatenate all translated segments at json[0][*][0].
        let segments = json.get(0).and_then(|a| a.as_array()).ok_or_else(|| {
            Error::FallbackEligible(FallbackKind::Parse("no segment array".into()))
        })?;
        let mut out = String::new();
        for seg in segments {
            if let Some(t) = seg.get(0).and_then(|v| v.as_str()) {
                out.push_str(t);
            }
        }
        if out.is_empty() {
            return Err(Error::FallbackEligible(FallbackKind::Parse("empty translation".into())));
        }
        Ok(out)
    }
}
```

- [ ] **Step 2: Create `src-tauri/tests/google.rs`** (wiremock — does NOT hit the real Google endpoint):

```rust
use linguaray_lib::engines::google::Google;
use linguaray_lib::engines::TraditionalEngine;

#[tokio::test]
async fn google_parses_nested_response() {
    use wiremock::{MockServer, Mock, ResponseTemplate};
    use serde_json::json;
    let server = MockServer::start().await;
    // Mimic google's nested-array response: [ [ ["你好","hello",null,null,1], ["世界","world",...] ], null, "en", ... ]
    Mock::given(wiremock::matchers::method("GET"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([
            [ ["你好","hello",null,null,1], ["世界","world",null,null,1] ],
            null, "en", null, null, null, 1.0, []
        ])))
        .mount(&server).await;

    // Build an engine whose endpoint is the mock server. Since Google hardcodes the
    // URL, we test the PARSE logic by pointing a reqwest client at the mock and
    // calling the engine through a thin shim. Simplest: replicate the parse inline
    // OR refactor Google to take a base URL (recommended).
    // For this first cut, refactor Google to hold a `base_url` field defaulting to
    // the real endpoint, overridable in tests — see Task 2 step 3.
    // (Test body completed after the refactor.)
    todo!("complete after Task 2 step 3 refactor")
}
```

- [ ] **Step 3: Refactor Google to be testable.** The hardcoded `translate.google.com` URL blocks wiremock. Give `Google` a `base_url: String` field (default the real URL) and a `pub fn new()` (real) / test can construct with the mock URL:

```rust
pub struct Google { base_url: String }
impl Google {
    pub fn new() -> Self { Self { base_url: "https://translate.google.com".into() } }
    pub fn with_base(base_url: impl Into<String>) -> Self { Self { base_url: base_url.into() } }
}
// in translate(): use format!("{}/translate_a/single", self.base_url) as the URL.
// registry() uses Google::new().
```
Update `registry()` in mod.rs to `Box::new(google::Google::new())`.

- [ ] **Step 4: Complete the wiremock test** against `Google::with_base(server.uri())`:

```rust
let eng = Google::with_base(server.uri());
let client = reqwest::Client::new();
let out = eng.translate(&client, "hello world", "auto", "zh").await.unwrap();
assert_eq!(out, "你好世界");
```

- [ ] **Step 5: Run tests.** `cd /Users/daoyu/Code/projects/linguaray/src-tauri && cargo test --test google` (cargo at `~/.cargo/bin/cargo`). Expected: PASS. Then `cargo test` (all prior + google) green.

- [ ] **Step 6: Commit (Tasks 1+2 together).**
```bash
cd /Users/daoyu/Code/projects/linguaray && git checkout -b phase3 && git add src-tauri/src/engines src-tauri/tests/google.rs src-tauri/Cargo.toml src-tauri/Cargo.lock && git -c user.name=daoyu -c user.email=daoyu@local commit -m "feat(engines): TraditionalEngine trait + Google (keyless) + wiremock test"
```
> Create `phase3` branch in this commit (first task of the phase). Note: `lib.rs` may still reference the old `registry() -> Vec<()>` — if so, fix that reference too (it's likely unused now; grep).

---

## Task 3: Fallback in translate service + fallback setting

**Files:** Modify `src-tauri/src/service.rs`, `src-tauri/src/settings.rs`, `src-tauri/src/lib.rs`

- [ ] **Step 1: Add `fallback_engine` to settings.** In `src-tauri/src/settings.rs`:
```rust
pub struct Settings {
    pub default_provider: String,
    pub target_language: String,
    pub fallback_engine: Option<String>,   // None = no fallback (opt-in per §G)
}
impl Default for Settings {
    fn default() -> Self {
        // §G: cross-remote fallback is OPT-IN. Default None — the user must enable
        // it, which is their consent to send text to a second remote engine.
        Self { default_provider: "openai".into(), target_language: "zh".into(), fallback_engine: None }
    }
}
```
Update `load`/`save` to handle `fallback_engine` (load: `store.get("fallback_engine").and_then(|v| v.as_str().map(String::from))`; if absent → None; save: `store.set("fallback_engine", json!(s.fallback_engine))`).

- [ ] **Step 2: Add `translate_with_fallback` to service.rs.** A new function that tries the primary (AI) engine, and on `FallbackEligible` retries once with the configured fallback trad engine — per §G rules:

```rust
use crate::engines;
use crate::settings::Settings;

/// Translate with §G fallback: primary (AI) first; on FallbackEligible error,
/// retry once with the configured fallback traditional engine. Respects:
/// - local mode sacred: if primary is a local provider (Ollama), NO silent remote fallback.
/// - whole-request retry: the fallback engine translates the FULL text (no chunk mixing).
pub async fn translate_with_fallback(
    client: &reqwest::Client,
    keystore: &crate::keystore::Keystore,
    settings: &Settings,
    primary_preset: &crate::providers::ProviderPreset,
    input: TranslateInput<'_>,
) -> Result<String, Error> {
    match translate(client, keystore, primary_preset, TranslateInput {
        text: input.text, from: input.from, to: input.to, options: input.options.clone(),
    }).await {
        Ok(text) => Ok(text),
        Err(Error::FallbackEligible(_)) => {
            // §G: local-primary sacred — don't silently degrade a local engine.
            if is_local(primary_preset) {
                return Err(Error::Config(crate::error::ConfigKind::MissingKey { /* misuse: signal "local failed, no auto-remote" */ provider: primary_preset.id.clone() }));
                // NOTE: better to add a dedicated error variant; for v1 reuse Config to mean "won't auto-fallback".
            }
            match &settings.fallback_engine {
                None => Err(Error::FallbackEligible(crate::error::FallbackKind::Network("no fallback configured".into()))),
                Some(id) => match engines::find(id) {
                    None => Err(Error::FallbackEligible(crate::error::FallbackKind::Parse(format!("fallback engine {id} not found")))),
                    Some(eng) => eng.translate(client, input.text, input.from, input.to).await,
                }
            }
        }
        Err(other) => Err(other), // Config/Auth/Keystore → propagate (no fallback per §G)
    }
}

fn is_local(p: &crate::providers::ProviderPreset) -> bool {
    // loopback host => local (spec §Privacy loopback definition)
    p.endpoint.starts_with("http://localhost") || p.endpoint.starts_with("http://127.0.0.1")
}
```

> **Implementer note on the local-error variant:** reusing `ConfigKind::MissingKey` to mean "local failed, no auto-remote" is a smell flagged for follow-up. For v1, either add a new `Error::LocalNoFallback` variant (cleaner) OR keep the reuse and add a code comment. Prefer adding the variant if cheap; else comment + move on. Report what you did.

> **Note on `AppOptions: Clone`:** `translate_with_fallback` clones `input.options` to reuse across both attempts. Ensure `wire::AppOptions` derives `Clone` (it already does from Phase 1).

- [ ] **Step 3: Write the fallback test.** `src-tauri/tests/fallback.rs` — primary returns `FallbackEligible` (via a mock AI server returning 429), fallback trad engine (a fake) returns a translation; assert the fallback result. Use a fake `TraditionalEngine` impl in the test. Since `engines::find` uses the real registry (Google, network), the test needs to inject a fake fallback — so either (a) refactor `translate_with_fallback` to take the fallback engine as a parameter (cleanest), or (b) test via the real Google against a wiremock. Prefer (a): change the signature to accept `fallback: Option<Box<dyn TraditionalEngine>>` and have the caller (lib.rs) do `engines::find(&settings.fallback_engine)`.

```rust
// pseudo: primary mocked to return FallbackEligible(429); fallback fake returns "FALLBACK_OK".
// assert result == "FALLBACK_OK" and the fallback engine was called with the full text.
```

- [ ] **Step 4: Wire into lib.rs.** In the selection handler / translate_default / translate_clipboard, replace the direct `service::translate(...)` call with `service::translate_with_fallback(...)` (passing settings + the fallback engine resolved via `engines::find`). 

- [ ] **Step 5: cargo check + cargo test.** All green (prior + google + fallback).

- [ ] **Step 6: Commit.**
```bash
git add src-tauri/src/service.rs src-tauri/src/settings.rs src-tauri/src/lib.rs src-tauri/tests/fallback.rs && git -c user.name=daoyu -c user.email=daoyu@local commit -m "feat(fallback): §G classified fallback (AI FallbackEligible → trad engine) + setting"
```

---

## Task 4: macOS system dictionary lookup

**Files:** Create `src-tauri/src/dict.rs`; modify `src-tauri/src/lib.rs` (command)

- [ ] **Step 1: Create `src-tauri/src/dict.rs`.** macOS Dictionary Services via manual FFI (avoids adding a crate). `DCSCopyTextDefinition(nil, word, range)` returns a plain-text definition:

```rust
//! macOS system dictionary lookup (spec §E: word definitions where LLMs are weak).
//! Uses DCSCopyTextDefinition (CoreServices). Returns plain text or None.
//! Non-macOS: returns None (no system dict).

#[cfg(target_os = "macos")]
pub fn lookup(word: &str) -> Option<String> {
    use core_foundation::base::{CFRelease, TCFType};
    use core_foundation::string::{CFString, CFRange};

    extern "C" {
        fn DCSCopyTextDefinition(
            dict: *const std::ffi::c_void,
            text: core_foundation::string::CFStringRef,
            range: CFRange,
        ) -> core_foundation::string::CFStringRef;
    }

    unsafe {
        let cf_word = CFString::new(word);
        let range = CFRange { location: 0, length: cf_word.char_len() };
        let result = DCSCopyTextDefinition(std::ptr::null(), cf_word.as_concrete_TypeRef(), range);
        if result.is_null() {
            return None;
        }
        let def = CFString::wrap_under_create_rule(result).to_string();
        // wrap_under_create_rule follows Create-rule ownership (no manual CFRelease).
        if def.is_empty() { None } else { Some(def) }
    }
}

#[cfg(not(target_os = "macos"))]
pub fn lookup(_word: &str) -> Option<String> { None }
```

> **Implementer note:** verify the `core-foundation` crate is available (it's a transitive dep of Tauri's `dpi`; may need explicit add). Add `core-foundation = "0.10"` to `[dependencies]` if the import fails. The `DCSCopyTextDefinition` symbol is in the CoreServices framework — on macOS it links automatically; if you get a link error, add `#[link(name = "CoreServices", kind = "framework")]` above the `extern` block. Verify `CFString::char_len()` (UTF-16 length, which DCSCopyTextDefinition expects) exists in the resolved core-foundation version; adjust to the real method name. Match the resolved crate API; report.

- [ ] **Step 2: Add `lookup_dictionary` command + register.** In lib.rs:
```rust
#[tauri::command]
fn lookup_dictionary(word: String) -> Option<String> {
    dict::lookup(&word)
}
```
Register in `invoke_handler!`.

- [ ] **Step 3: cargo check + cargo test.** Green. (dict has no unit test — it's OS FFI, manual-only per §I; the lookup_dictionary command is exercised at runtime.)

- [ ] **Step 4: Commit.**
```bash
git add src-tauri/src/dict.rs src-tauri/src/lib.rs src-tauri/Cargo.toml src-tauri/Cargo.lock && git -c user.name=daoyu -c user.email=daoyu@local commit -m "feat(dict): macOS system dictionary lookup (DCSCopyTextDefinition)"
```

---

## Task 5: Manual end-to-end + final review

**Files:** none (manual + review)

- [ ] **Step 1: `pnpm tauri dev`.** Grant Accessibility (for the selection hotkey).
- [ ] **Step 2: Fallback test:** configure a default AI provider that will FAIL in a FallbackEligible way — easiest: set default to `openai` but DO NOT set a key (→ MissingKey = Config, NO fallback — verify it shows "go to settings", not a silent Google result). To test the fallback path, instead point default at a provider whose endpoint returns 500/429 (e.g. temporarily edit a preset endpoint to a local wiremock-style 500, or use an invalid model). Verify the popup shows a Google fallback result tagged accordingly.
- [ ] **Step 3: Google standalone:** set fallback to `google`; add a way to invoke Google directly (or temporarily set default_provider to `google` — note google is an engine not a provider, so this requires the UI/list_engines to also list trad engines; if not wired, defer to a later UI task and just verify via the fallback path).
- [ ] **Step 4: Dictionary:** (needs a UI affordance — if not added, add a tiny one or call `lookup_dictionary` from the dev console). Verify a word returns its macOS definition.
- [ ] **Step 5: local-sacred:** set default to ollama (local, not running) → trigger → verify it does NOT silently fall back to remote Google (shows an error instead).

> If `list_engines`/UI doesn't surface trad engines yet, that's a known gap — file it. The fallback path (Task 3) is the v1-critical behavior; standalone trad-engine selection is a UX nicety for a follow-up.

- [ ] **Step 6: Final review** (opus code-reviewer) of the phase, then merge to main.

---

## Self-Review (run after writing; fix inline)

- **Spec coverage:** §G classified fallback → Task 3 (FallbackEligible→trad; Config→propagate; local-sacred; no-retry; whole-request). §E system-dict → Task 4. First trad engine → Task 2 (Google, keyless). **Gaps:** cross-remote fallback opt-in setting is NOT yet a separate toggle — currently ANY non-local primary with a fallback_engine set will fall back to the (local? or remote?) trad engine. Google is remote. So "cross-remote fallback requires opt-in" (§G) is partially unenforced: a non-local primary (e.g. openai) failing → falls back to google (also remote) WITHOUT an explicit opt-in. **Fix:** the fallback_engine default should be acceptable because Google is keyless/widely-considered-safe, BUT to honor §G strictly, gate cross-remote fallback behind the existing `fallback_engine` setting being explicitly set by the user (i.e. default `None` until they opt in). Reconsider the default: set `fallback_engine: None` by default (opt-in), document that enabling it = consenting to a second remote engine. Update Task 3 Step 1 default to `None` and Task 5 step 2 accordingly.
- **Placeholder scan:** Task 2 step 2 had a `todo!()` placeholder, explicitly resolved in step 3/4 (the `with_base` refactor) — fine, it's a staged refactor within the task. The local-error-variant smell is flagged with a clear either/or. No lingering TBDs.
- **Type consistency:** `TraditionalEngine` trait used across engines/mod.rs, google.rs, service.rs (fallback), tests. `Settings.fallback_engine` consistent across settings/service/lib. `engines::find(id) -> Option<Box<dyn TraditionalEngine>>` matches service usage.
- **Leverage ① status:** porting source is distributed across per-engine repos (the central `pot-app/pot-plugins` is gone) — Task 2 documents this and falls back to the well-known public Google endpoint if the plugin repo is inaccessible. Honest about the source.

---

## Execution Handoff

Plan complete and saved to `docs/superpowers/plans/2026-07-30-phase3-trad-engine-fallback-dict.md`. Two execution options:

**1. Subagent-Driven (recommended)** — fresh subagent per task, two-stage review.

**2. Inline Execution** — batch with checkpoints.

**Which approach?**
