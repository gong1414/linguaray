# Plugin Core PR-0 + PR-1 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Freeze the rev-4 plugin-core spec into the repo (PR-0), then ship a user-visible 21-row provider catalog with Provider Center reading IPC — no Fiber, no kernel hookup (PR-1).

**Architecture:** Catalog and protocol enums live in `src-tauri/crates/` so adding a vendor is JSON + rebuild. `preset_protocol()` dies. Auth and `models_url` are copied onto `ProviderProfile.capabilities` at create. Provider Center deletes the hardcoded 4-row `PRESETS` constant. Support tiers (`ready` / `setup_required` / `unverified`) are first-class. Model fetch never sends a key to a different origin than `profile.endpoint`.

**Tech Stack:** Tauri 2, Rust 2021 workspace under `src-tauri/`, SolidJS + Vitest, Cargo test. Spec law: `docs/superpowers/specs/2026-08-14-linguaray-plugin-core-design.md` rev-4.

**Out of scope:** `linguaray-kernel`, Fiber, Shortcuts hookup, traditional-engine rewrite, worktree deletion (PR-7), remote catalog overlay.

---

## File map

### PR-0 (docs / license only)

| Path | Action |
|---|---|
| `LICENSE` | Create — MIT full text |
| `THIRD_PARTY_NOTICES` | Create — first-party + major deps |
| `.gitignore` | Add `.pnpm-store/` |
| `README.md` | Official plugins in-tree; third-party/WASM still post-v1; catalog tiers |
| `docs/superpowers/specs/2026-08-01-linguaray-product-baseline.md` | Erratum: 21 AI ids + official in-tree plugins |
| `docs/superpowers/specs/2026-08-14-linguaray-plugin-core-design.md` | Already rev-4; include in commit if uncommitted |
| `docs/superpowers/archive/2026-08-14-linguaray-bevy-plugin-core-redesign.md` | Include if untracked |
| `docs/superpowers/plans/2026-08-13-rayline-r4-surfaces-09-10-11.md` | Include if untracked (living product plan) |
| `docs/superpowers/plans/2026-08-14-plugin-core-pr0-pr1.md` | This plan |

### PR-1 (catalog + Provider Center)

| Path | Action |
|---|---|
| `src-tauri/Cargo.toml` | Become workspace; depend on new crates |
| `src-tauri/crates/linguaray-contracts/` | `AuthKind`, `ProtocolKind`, `SupportTier` |
| `src-tauri/crates/linguaray-catalog/` | `providers.json`, `engines.json`, load + schema tests |
| `src-tauri/src/providers.rs` | Stop being the 4-row catalog; keep `validate_endpoint`; re-export catalog types |
| `src-tauri/src/db/providers.rs` | Delete `preset_protocol()`; empty-endpoint law; auth/models_url; Custom protocol patch |
| `src-tauri/src/adapter.rs` | Map `capabilities.auth` onto `ProviderPreset.auth` |
| `src-tauri/src/wire.rs` | Send header from `AuthKind`, not always bearer |
| `src-tauri/src/lib.rs` | `provider_list_presets`; get_models origin check; create/update wire |
| `src-tauri/build.rs` | Register `provider_list_presets` |
| `src-tauri/capabilities/main.json` | `allow-provider-list-presets` |
| `src/features/settings/ProviderCenter.tsx` | Delete `PRESETS`; load IPC; show tiers |
| `src/features/settings/provider-domain.ts` | `validateEndpoint` empty exception; origin helper |
| `apps/ui-lab/src/pages/provider-domain.ts` | Same validator |
| `apps/ui-lab/src/pages/ProviderCenter.tsx` | Consume view `presets` from production types |
| `test/ProviderCenter.test.tsx` | 21 ids + no Google/DeepL as AI presets |
| `src-tauri/tests/wire.rs`, `tests/fallback.rs` | Fill new `auth` field |
| `public/providers/icon-provider.svg` or `src/assets/providers/icon-provider.svg` | Generic icon fallback (one SVG) |

---

## PR-0

### Task 1: MIT LICENSE

**Files:**
- Create: `LICENSE`

- [ ] **Step 1: Write LICENSE**

Use the Expat/MIT text. Copyright holder matches `Cargo.toml` authors (`gong1414`) and year 2026:

```text
MIT License

Copyright (c) 2026 gong1414

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
```

- [ ] **Step 2: Commit**

```bash
git add LICENSE
git commit -m "docs: add root MIT LICENSE"
```

### Task 2: THIRD_PARTY_NOTICES + gitignore

**Files:**
- Create: `THIRD_PARTY_NOTICES`
- Modify: `.gitignore`

- [ ] **Step 1: Write THIRD_PARTY_NOTICES**

```text
LinguaRay third-party notices
=============================

This product includes software developed by third parties.
First-party code is MIT (see LICENSE).

IMPORTANT: src-tauri/src/engines/google.rs currently states it was ported
from pot-desktop (GPL-3.0). That file is isolated pending the clean-room
rewrite required by spec §12.4. It is NOT covered by the MIT grant until
that rewrite lands.

Major runtime / build dependencies (non-exhaustive; see lockfiles for
versions):

- Tauri 2 and official plugins — MIT OR Apache-2.0
  https://github.com/tauri-apps/tauri
- SolidJS — MIT
- reqwest, serde, tokio, rusqlite (bundled SQLite), uuid, aes-gcm,
  argon2, zeroize — MIT OR Apache-2.0 as declared by each crate
- Inter / IBM Plex Mono / Noto Sans SC font files under packages/ui
  — see each font's OFL/license file in packages/ui/src/assets/fonts/

Do not copy pot-app/pot-desktop or Easydict source (GPL-3.0).
```

- [ ] **Step 2: Ignore the pnpm store**

Append to `.gitignore` (file currently has no `.pnpm-store` entry):

```gitignore
# Local pnpm content-addressable store (never commit)
.pnpm-store/
```

- [ ] **Step 3: Commit**

```bash
git add THIRD_PARTY_NOTICES .gitignore
git commit -m "docs: add THIRD_PARTY_NOTICES and ignore .pnpm-store"
```

### Task 3: S0 erratum + README

**Files:**
- Modify: `docs/superpowers/specs/2026-08-01-linguaray-product-baseline.md`
- Modify: `README.md`

- [ ] **Step 1: Add erratum after the S0 header block** (after line 6, before `## 1.`)

```markdown
**Erratum (2026-08-14, plugin-core rev-4):**

1. Official in-tree Capability/Driver plugins supersede the sentence
   “v1 has no plugin system” for *first-party* code. Third-party SDK,
   Bob/Pot plugin compatibility, and WASM remain 🔜.
2. Official AI catalog is the 21 ids in
   `docs/superpowers/specs/2026-08-14-linguaray-plugin-core-design.md` §7.3,
   with support tiers `ready` / `setup_required` / `unverified`.
   The 8-name list below is historical. 30+ relay long-tail remains 🔜.
   Custom still covers user-supplied OpenAI-compatible / Anthropic endpoints.
```

Do **not** silently rewrite the historical “Built-in catalog” bullets; the erratum is the law.

- [ ] **Step 2: README**

Replace lines 52–53:

```markdown
- v1 official capabilities and protocol drivers are **in-tree plugins**
  (see `docs/superpowers/specs/2026-08-14-linguaray-plugin-core-design.md`).
  Third-party / WASM loading remains post-v1.
```

In “Current capabilities”, change “fill-key-and-use” to:

```markdown
- AI provider catalog (21 official presets; only `ready` rows are
  fill-key-and-use — Azure / Custom / Doubao require extra setup)
```

- [ ] **Step 3: Commit living docs that are still untracked**

```bash
git add \
  docs/superpowers/specs/2026-08-01-linguaray-product-baseline.md \
  README.md \
  docs/superpowers/specs/2026-08-14-linguaray-plugin-core-design.md \
  docs/superpowers/archive/2026-08-14-linguaray-bevy-plugin-core-redesign.md \
  docs/superpowers/plans/2026-08-13-rayline-r4-surfaces-09-10-11.md \
  docs/superpowers/plans/2026-08-14-plugin-core-pr0-pr1.md
git status
git commit -m "docs: freeze plugin-core rev-4, S0 erratum, archive Bevy draft"
```

Expected: Bevy file is only under `archive/`, never under `specs/` again.

---

## PR-1

TDD order: contracts → catalog tests/JSON → workspace → DB/create/update → wire/auth → IPC → frontend.

### Task 4: `linguaray-contracts` crate (types only)

**Files:**
- Create: `src-tauri/crates/linguaray-contracts/Cargo.toml`
- Create: `src-tauri/crates/linguaray-contracts/src/lib.rs`
- Test: same crate `#[cfg(test)]`

- [ ] **Step 1: Write failing crate (types + tests first)**

`src-tauri/crates/linguaray-contracts/Cargo.toml`:

```toml
[package]
name = "linguaray-contracts"
version = "0.1.0"
edition = "2021"
license = "MIT"
description = "Shared protocol/auth/tier enums for LinguaRay catalog and wire"
publish = false

[dependencies]
serde = { version = "1", features = ["derive"] }
```

`src-tauri/crates/linguaray-contracts/src/lib.rs` — start with the tests and empty enums so `cargo test -p linguaray-contracts` fails to compile until you fill them, or write enums + tests together as below (this crate is types-only; the “failing test” is the serde round-trip and reject-unknown).

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ProtocolKind {
    OpenaiChat,
    Anthropic,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AuthKind {
    #[default]
    Bearer,
    XApiKey,
    AzureKey,
    Query,
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SupportTier {
    Ready,
    SetupRequired,
    Unverified,
}

impl AuthKind {
    pub fn header_name(self) -> Option<&'static str> {
        match self {
            AuthKind::Bearer => Some("Authorization"),
            AuthKind::XApiKey => Some("x-api-key"),
            AuthKind::AzureKey => Some("api-key"),
            AuthKind::Query | AuthKind::None => None,
        }
    }
}

impl ProtocolKind {
    pub fn to_db(self) -> &'static str {
        match self {
            ProtocolKind::OpenaiChat => "openai_chat",
            ProtocolKind::Anthropic => "anthropic",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serde_kebab() {
        assert_eq!(serde_json::to_string(&AuthKind::AzureKey).unwrap(), "\"azure-key\"");
        assert_eq!(serde_json::to_string(&SupportTier::SetupRequired).unwrap(), "\"setup_required\"");
        assert_eq!(serde_json::from_str::<ProtocolKind>("\"openai-chat\"").unwrap(), ProtocolKind::OpenaiChat);
    }

    #[test]
    fn unknown_auth_rejected() {
        assert!(serde_json::from_str::<AuthKind>("\"magic\"").is_err());
    }
}
```

- [ ] **Step 2: Run tests**

```bash
cd src-tauri && cargo test -p linguaray-contracts
```

This fails until the workspace exists (Task 5). If you run it after Task 5, expected: PASS (2 tests).

- [ ] **Step 3: Commit after Task 5 workspace lands** (or commit crate files now and workspace in the next commit). Prefer one commit: `feat(contracts): AuthKind ProtocolKind SupportTier`.

### Task 5: Workspace prelude

**Files:**
- Modify: `src-tauri/Cargo.toml`

- [ ] **Step 1: Add workspace + path deps at the top of `src-tauri/Cargo.toml` after `[package]` is fine; put workspace at file top as Cargo prefers:**

```toml
[workspace]
members = [".", "crates/linguaray-contracts", "crates/linguaray-catalog"]
resolver = "2"
```

Under `[dependencies]` of the `linguaray` package add:

```toml
linguaray-contracts = { path = "crates/linguaray-contracts" }
linguaray-catalog = { path = "crates/linguaray-catalog" }
```

(`linguaray-catalog` crate is created in Task 6; add the dep in Task 6 if you want the workspace to build after Task 5 with only contracts.)

- [ ] **Step 2: Prove the app still builds**

```bash
cd src-tauri && cargo check -p linguaray
```

Expected: success (after catalog crate exists, or only add catalog member in Task 6).

- [ ] **Step 3: Commit**

```bash
git add src-tauri/Cargo.toml src-tauri/Cargo.lock src-tauri/crates/linguaray-contracts
git commit -m "build: src-tauri workspace + linguaray-contracts"
```

### Task 6: Catalog crate — failing schema tests first

**Files:**
- Create: `src-tauri/crates/linguaray-catalog/Cargo.toml`
- Create: `src-tauri/crates/linguaray-catalog/src/lib.rs`
- Create: `src-tauri/crates/linguaray-catalog/src/validate.rs`
- Create: `src-tauri/crates/linguaray-catalog/providers.json` (Task 7)
- Create: `src-tauri/crates/linguaray-catalog/engines.json`

- [ ] **Step 1: Write tests that encode spec §7.5**

`Cargo.toml`:

```toml
[package]
name = "linguaray-catalog"
version = "0.1.0"
edition = "2021"
license = "MIT"
publish = false

[dependencies]
linguaray-contracts = { path = "../linguaray-contracts" }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
url = "2"
```

`src/lib.rs` (minimal types; `load()` can return empty until JSON exists — tests will fail):

```rust
mod validate;

pub use linguaray_contracts::{AuthKind, ProtocolKind, SupportTier};
pub use validate::{validate_catalog, CatalogError};

use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct CatalogFile {
    pub schema_version: u32,
    pub catalog_revision: u32,
    pub providers: Vec<CatalogProvider>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CatalogProvider {
    pub id: String,
    pub label: String,
    pub protocol: ProtocolKind,
    pub auth: AuthKind,
    pub endpoint: String,
    pub default_model: String,
    pub needs_key: bool,
    pub support_tier: SupportTier,
    #[serde(default)]
    pub requires_user_endpoint: bool,
    #[serde(default)]
    pub models_url: Option<String>,
    #[serde(default)]
    pub website: Option<String>,
    #[serde(default)]
    pub console_url: Option<String>,
    #[serde(default)]
    pub docs: Option<String>,
    #[serde(default)]
    pub notes: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub icon: Option<String>,
}

pub fn load() -> Result<CatalogFile, CatalogError> {
    let raw = include_str!("../providers.json");
    let file: CatalogFile = serde_json::from_str(raw)?;
    validate_catalog(&file)?;
    Ok(file)
}

pub fn get(id: &str) -> Option<CatalogProvider> {
    load().ok()?.providers.into_iter().find(|p| p.id == id)
}

pub const REQUIRED_IDS: &[&str] = &[
    "openai", "anthropic", "gemini", "deepseek", "openrouter",
    "azure-openai", "ollama", "custom", "zhipu-glm", "kimi",
    "minimax", "bailian", "doubao", "siliconflow", "modelscope",
    "stepfun", "xiaomi-mimo", "nvidia-nim", "groq", "mistral", "together",
];

pub const RELAY_HOST_DENY: &[&str] = &[
    "packycode.com", "cubence.com", "aigocode.com", "right.codes",
    "aicodemirror.com",
];
```

`src/validate.rs` must implement (fail the test if not):

- `schema_version == 1`
- unique kebab-case ids
- all `REQUIRED_IDS` present (exactly those 21, extras fail)
- `ready` ⇒ `!requires_user_endpoint` and non-empty endpoint + default_model
- `setup_required` may have empty endpoint
- non-empty endpoints pass HTTPS-or-loopback (`localhost` / `127.0.0.1` / `::1` only; **do not** accept `0.0.0.0` — preserve today’s split)
- `needs_key == false` only if loopback or `auth == none`
- host not in `RELAY_HOST_DENY`
- `ready` set is exactly `{openai, anthropic, gemini, ollama}` until a later smoke-promotion PR

Include tests:

```rust
#[test]
fn shipped_catalog_validates() {
    let c = crate::load().expect("catalog");
    assert_eq!(c.providers.len(), 21);
    assert_eq!(c.schema_version, 1);
}

#[test]
fn ready_cannot_be_empty_endpoint() {
    // construct a CatalogFile in memory and expect validate_catalog Err
}

#[test]
fn unverified_cannot_be_silently_ready() {
    // if support_tier=ready for deepseek → Err
}

#[test]
fn relay_host_rejected() {
    // endpoint https://api.packycode.com/v1/chat/completions → Err
}
```

- [ ] **Step 2: Run tests — expect FAIL** (no JSON or empty JSON)

```bash
cd src-tauri && cargo test -p linguaray-catalog
```

Expected: compile error (`providers.json` missing) or `load()` validation error.

- [ ] **Step 3: Add `engines.json`** (metadata only; no drivers this PR)

```json
{
  "schema_version": 1,
  "engines": [
    {"id":"google","label":"Google Translate","unofficial_gtx":true,"needs_key":false},
    {"id":"deepl","label":"DeepL","needs_key":true},
    {"id":"microsoft","label":"Microsoft Translator","needs_key":true},
    {"id":"baidu","label":"百度翻译","needs_key":true},
    {"id":"youdao","label":"有道","needs_key":true},
    {"id":"tencent","label":"腾讯","needs_key":true}
  ]
}
```

Do not implement traditional drivers here.

### Task 7: `providers.json` — 21 rows from spec appendix A

**Files:**
- Create: `src-tauri/crates/linguaray-catalog/providers.json`

- [ ] **Step 1: Write the file** (values must match spec appendix A; `support_tier` as frozen)

Use this document (do not invent Azure’s host):

```json
{
  "schema_version": 1,
  "catalog_revision": 1,
  "providers": [
    {"id":"openai","label":"OpenAI","protocol":"openai-chat","auth":"bearer","endpoint":"https://api.openai.com/v1/chat/completions","default_model":"gpt-4o-mini","needs_key":true,"support_tier":"ready","models_url":"https://api.openai.com/v1/models","icon":"openai"},
    {"id":"anthropic","label":"Anthropic Claude","protocol":"anthropic","auth":"x-api-key","endpoint":"https://api.anthropic.com/v1/messages","default_model":"claude-sonnet-4-5","needs_key":true,"support_tier":"ready","models_url":"https://api.anthropic.com/v1/models","icon":"anthropic"},
    {"id":"gemini","label":"Google Gemini","protocol":"openai-chat","auth":"bearer","endpoint":"https://generativelanguage.googleapis.com/v1beta/openai/chat/completions","default_model":"gemini-3.6-flash","needs_key":true,"support_tier":"ready","models_url":"https://generativelanguage.googleapis.com/v1beta/openai/models","icon":"gemini"},
    {"id":"deepseek","label":"DeepSeek","protocol":"openai-chat","auth":"bearer","endpoint":"https://api.deepseek.com/chat/completions","default_model":"deepseek-v4-flash","needs_key":true,"support_tier":"unverified","models_url":"https://api.deepseek.com/models","docs":"https://api-docs.deepseek.com/"},
    {"id":"openrouter","label":"OpenRouter","protocol":"openai-chat","auth":"bearer","endpoint":"https://openrouter.ai/api/v1/chat/completions","default_model":"openai/gpt-4o-mini","needs_key":true,"support_tier":"unverified","models_url":"https://openrouter.ai/api/v1/models"},
    {"id":"azure-openai","label":"Azure OpenAI","protocol":"openai-chat","auth":"azure-key","endpoint":"","default_model":"","needs_key":true,"support_tier":"setup_required","requires_user_endpoint":true,"notes":"Paste full URL. Templates: https://{resource}.openai.azure.com/openai/v1/chat/completions or classic deployments/{deployment}/chat/completions?api-version=2024-10-21"},
    {"id":"ollama","label":"Ollama (local)","protocol":"openai-chat","auth":"none","endpoint":"http://localhost:11434/v1/chat/completions","default_model":"qwen2.5:7b","needs_key":false,"support_tier":"ready","models_url":"http://localhost:11434/v1/models"},
    {"id":"custom","label":"Custom","protocol":"openai-chat","auth":"bearer","endpoint":"","default_model":"","needs_key":true,"support_tier":"setup_required","requires_user_endpoint":true,"notes":"User supplies full endpoint. Toggle Anthropic to switch protocol; auth is derived."},
    {"id":"zhipu-glm","label":"智谱 GLM","protocol":"openai-chat","auth":"bearer","endpoint":"https://open.bigmodel.cn/api/paas/v4/chat/completions","default_model":"glm-4-flash","needs_key":true,"support_tier":"unverified","models_url":"https://open.bigmodel.cn/api/paas/v4/models"},
    {"id":"kimi","label":"Kimi","protocol":"openai-chat","auth":"bearer","endpoint":"https://api.moonshot.cn/v1/chat/completions","default_model":"kimi-k3","needs_key":true,"support_tier":"unverified","models_url":"https://api.moonshot.cn/v1/models","notes":"Domestic default api.moonshot.cn. Global: https://api.moonshot.ai/v1/chat/completions. Switching origin discards models_url and re-derives; never send the key to the other region."},
    {"id":"minimax","label":"MiniMax","protocol":"openai-chat","auth":"bearer","endpoint":"https://api.minimax.io/v1/chat/completions","default_model":"MiniMax-M3","needs_key":true,"support_tier":"unverified","models_url":"https://api.minimax.io/v1/models"},
    {"id":"bailian","label":"通义 / 百炼","protocol":"openai-chat","auth":"bearer","endpoint":"https://dashscope.aliyuncs.com/compatible-mode/v1/chat/completions","default_model":"qwen-plus","needs_key":true,"support_tier":"unverified","models_url":"https://dashscope.aliyuncs.com/compatible-mode/v1/models"},
    {"id":"doubao","label":"豆包","protocol":"openai-chat","auth":"bearer","endpoint":"https://ark.cn-beijing.volces.com/api/v3/chat/completions","default_model":"","needs_key":true,"support_tier":"setup_required","requires_user_endpoint":false,"notes":"Fill the Ark endpoint id (ep-xxxxxxxx) or a public model name before Translate."},
    {"id":"siliconflow","label":"SiliconFlow","protocol":"openai-chat","auth":"bearer","endpoint":"https://api.siliconflow.cn/v1/chat/completions","default_model":"Qwen/Qwen2.5-7B-Instruct","needs_key":true,"support_tier":"unverified","models_url":"https://api.siliconflow.cn/v1/models"},
    {"id":"modelscope","label":"ModelScope","protocol":"openai-chat","auth":"bearer","endpoint":"https://api-inference.modelscope.cn/v1/chat/completions","default_model":"Qwen/Qwen2.5-7B-Instruct","needs_key":true,"support_tier":"unverified","models_url":"https://api-inference.modelscope.cn/v1/models"},
    {"id":"stepfun","label":"StepFun","protocol":"openai-chat","auth":"bearer","endpoint":"https://api.stepfun.com/v1/chat/completions","default_model":"step-3.7-flash","needs_key":true,"support_tier":"unverified","models_url":"https://api.stepfun.com/v1/models"},
    {"id":"xiaomi-mimo","label":"小米 MiMo","protocol":"openai-chat","auth":"azure-key","endpoint":"https://api.xiaomimimo.com/v1/chat/completions","default_model":"mimo-v2.5-pro","needs_key":true,"support_tier":"unverified","models_url":"https://api.xiaomimimo.com/v1/models"},
    {"id":"nvidia-nim","label":"NVIDIA NIM","protocol":"openai-chat","auth":"bearer","endpoint":"https://integrate.api.nvidia.com/v1/chat/completions","default_model":"meta/llama-3.1-8b-instruct","needs_key":true,"support_tier":"unverified","models_url":"https://integrate.api.nvidia.com/v1/models"},
    {"id":"groq","label":"Groq","protocol":"openai-chat","auth":"bearer","endpoint":"https://api.groq.com/openai/v1/chat/completions","default_model":"llama-3.3-70b-versatile","needs_key":true,"support_tier":"unverified","models_url":"https://api.groq.com/openai/v1/models"},
    {"id":"mistral","label":"Mistral","protocol":"openai-chat","auth":"bearer","endpoint":"https://api.mistral.ai/v1/chat/completions","default_model":"mistral-small-latest","needs_key":true,"support_tier":"unverified","models_url":"https://api.mistral.ai/v1/models"},
    {"id":"together","label":"Together","protocol":"openai-chat","auth":"bearer","endpoint":"https://api.together.ai/v1/chat/completions","default_model":"Qwen/Qwen2.5-7B-Instruct-Turbo","needs_key":true,"support_tier":"unverified","models_url":"https://api.together.ai/v1/models"}
  ]
}
```

Doubao has a non-empty official chat URL but empty model → `setup_required`. Validator: `setup_required` may have empty `default_model`. Empty endpoint only when `requires_user_endpoint`.

- [ ] **Step 2: Run catalog tests**

```bash
cd src-tauri && cargo test -p linguaray-catalog
```

Expected: PASS.

- [ ] **Step 3: Commit**

```bash
git add src-tauri/crates/linguaray-catalog src-tauri/Cargo.toml src-tauri/Cargo.lock
git commit -m "feat(catalog): 21 official presets with support tiers"
```

### Task 8: Delete `preset_protocol()`; create() copies auth + models_url; empty-endpoint law

**Files:**
- Modify: `src-tauri/src/db/providers.rs` (`preset_lookup` ~324, `preset_protocol` ~352, `create` ~369, `ProviderCapabilities` ~102, `ProviderPatch` ~147, `update` ~476)
- Modify: `src-tauri/src/providers.rs` — `presets()` becomes a thin wrapper over catalog for any leftover callers, or deleted if you update all call sites
- Test: add tests in `src-tauri/src/db/providers.rs` `#[cfg(test)]` or `src-tauri/tests/` if that is where create tests live

Search first:

```bash
rg -n "preset_protocol|fn presets\(|providers::presets" src-tauri
```

- [ ] **Step 1: Write failing tests** (in the existing providers test module — extend it)

```rust
#[test]
fn create_custom_allows_empty_endpoint() {
    // open temp db via existing test helper
    let p = create(&mut conn, "custom", "My custom", "", None).unwrap();
    assert_eq!(p.endpoint, "");
    assert_eq!(p.protocol, Protocol::OpenaiChat); // catalog default, NOT CustomHttp
    assert_eq!(p.capabilities.auth, Some(AuthKind::Bearer));
    assert!(p.capabilities.models_url.is_none());
}

#[test]
fn create_azure_allows_empty_endpoint() {
    let p = create(&mut conn, "azure-openai", "Az", "", None).unwrap();
    assert_eq!(p.endpoint, "");
    assert_eq!(p.capabilities.auth, Some(AuthKind::AzureKey));
}

#[test]
fn create_openai_empty_endpoint_rejected() {
    let err = create(&mut conn, "openai", "O", "", None).unwrap_err();
    // Integrity — validate_endpoint("") fails because ready rows are not exempt
}

#[test]
fn create_xiaomi_copies_azure_key_auth() {
    let p = create(&mut conn, "xiaomi-mimo", "Mi", "", None).unwrap();
    // xiaomi has a non-empty catalog endpoint, so empty caller uses catalog URL
    assert_eq!(p.capabilities.auth, Some(AuthKind::AzureKey));
    assert!(p.endpoint.starts_with("https://api.xiaomimimo.com"));
}

#[test]
fn unknown_template_still_custom_http_repair() {
    let p = create(&mut conn, "not-a-real-id", "X", "", None).unwrap();
    assert_eq!(p.protocol, Protocol::CustomHttp);
}
```

Run: `cargo test -p linguaray --lib create_custom_allows_empty` — FAIL (custom currently becomes CustomHttp only when unknown; once `custom` is a catalog id, today’s create will `validate_endpoint("")` and error).

- [ ] **Step 2: Implement**

`ProviderCapabilities`:

```rust
use linguaray_contracts::AuthKind;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProviderCapabilities {
    pub balance: bool,
    pub quota: bool,
    pub model_list: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub auth: Option<AuthKind>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub models_url: Option<String>,
}
```

Replace `preset_lookup` / delete `preset_protocol`:

```rust
struct PresetDerived {
    protocol: Protocol,
    auth: AuthKind,
    models_url: Option<String>,
    endpoint: String,
    default_model: Option<String>,
    needs_key: bool,
    requires_user_endpoint: bool,
}

fn preset_lookup(template_id: &str) -> Option<PresetDerived> {
    linguaray_catalog::get(template_id).map(|row| PresetDerived {
        protocol: match row.protocol {
            ProtocolKind::OpenaiChat => Protocol::OpenaiChat,
            ProtocolKind::Anthropic => Protocol::Anthropic,
        },
        auth: row.auth,
        models_url: row.models_url,
        endpoint: row.endpoint,
        default_model: if row.default_model.is_empty() { None } else { Some(row.default_model) },
        needs_key: row.needs_key,
        requires_user_endpoint: row.requires_user_endpoint,
    })
}
```

In `create`, after computing `ep`:

```rust
let skip_validate = d.requires_user_endpoint && ep.is_empty();
if !skip_validate {
    crate::providers::validate_endpoint(&ep).map_err(DbError::Integrity)?;
}
```

Set

```rust
capabilities: ProviderCapabilities {
    auth: Some(d.auth),
    models_url: d.models_url,
    ..Default::default()
},
```

Doubao: `requires_user_endpoint=false` but empty model — create succeeds; Translation later treats empty model as `Config::InvalidRequest` (already does via 404/empty). UI disables Translate when `setup_required && !model`.

- [ ] **Step 3: Run the new tests + existing db provider tests**

```bash
cd src-tauri && cargo test --lib db::providers
```

Expected: PASS. Existing migration tests that assume only 4 preset ids must still pass (they look up openai/anthropic/gemini/ollama).

- [ ] **Step 4: Commit**

```bash
git commit -am "feat(providers): catalog-backed create, empty-endpoint law, persist auth"
```

### Task 9: Custom protocol patch (spec §9.1.1)

**Files:**
- Modify: `ProviderPatch` and `update` in `src-tauri/src/db/providers.rs`

- [ ] **Step 1: Failing tests**

```rust
#[test]
fn custom_protocol_patch_derives_auth() {
    let p = create(&mut conn, "custom", "C", "https://example.com/v1/messages", None).unwrap();
    let out = update(&mut conn, &p.uuid, &ProviderPatch {
        protocol: Some(Protocol::Anthropic),
        expected_version: p.version,
        ..empty_patch()
    }).unwrap();
    let written = match out { UpdateOutcome::Written(w) => w, _ => panic!() };
    assert_eq!(written.protocol, Protocol::Anthropic);
    assert_eq!(written.capabilities.auth, Some(AuthKind::XApiKey));
}

#[test]
fn non_custom_protocol_patch_rejected() {
    let p = create(&mut conn, "openai", "O", "", None).unwrap();
    let err = update(&mut conn, &p.uuid, &ProviderPatch {
        protocol: Some(Protocol::Anthropic),
        expected_version: p.version,
        ..empty_patch()
    }).unwrap_err();
    // Integrity("protocol patch only allowed on template_id=custom")
}
```

`ProviderPatch` currently has `deny_unknown_fields` and no `protocol`. Frontend will start sending `protocol` only for custom.

- [ ] **Step 2: Implement**

```rust
pub struct ProviderPatch {
    pub name: Option<String>,
    pub endpoint: Option<String>,
    pub model: Option<String>,
    pub enabled: Option<bool>,
    pub sort_order: Option<i32>,
    pub expected_version: i64,
    #[serde(default)]
    pub protocol: Option<Protocol>,
}
```

In `update`, after loading `existing`:

```rust
if let Some(new_proto) = patch.protocol {
    if existing.template_id != "custom" {
        return Err(DbError::Integrity(
            "protocol patch only allowed on template_id=custom".into(),
        ));
    }
    // derive auth in the same transaction
    existing.capabilities.auth = Some(match new_proto {
        Protocol::Anthropic => AuthKind::XApiKey,
        Protocol::OpenaiChat | Protocol::Gemini => AuthKind::Bearer,
        other => {
            return Err(DbError::Integrity(format!("custom cannot use {other:?}")));
        }
    });
    existing.protocol = new_proto;
}
```

Include `protocol` and `capabilities` in the UPDATE SQL (capabilities was not updated today — **must** write it when protocol/auth/models_url change).

When `patch.endpoint` changes origin, also:

```rust
if origin_changed {
    existing.capabilities.models_url = derive_models_url(&endpoint, existing.protocol);
}
```

`derive_models_url`:

```rust
fn derive_models_url(endpoint: &str, protocol: Protocol) -> Option<String> {
    let u = url::Url::parse(endpoint).ok()?;
    // same origin; replace last path segment with "models"
    let mut out = u.clone();
    {
        let mut segs: Vec<String> = out.path_segments()?.map(|s| s.to_string()).collect();
        if segs.last().map(|s| s == "completions" || s == "messages").unwrap_or(false) {
            segs.pop();
        }
        if segs.last().map(|s| s != "models").unwrap_or(true) {
            segs.push("models".into());
        }
        out.set_path(&segs.join("/"));
    }
    let _ = protocol; // Anthropic also /v1/models on same origin
    Some(out.to_string())
}
```

- [ ] **Step 3: Tests pass + commit**

```bash
git commit -am "feat(providers): custom protocol patch derives auth; models_url follows origin"
```

### Task 10: Wire + adapter honor `AuthKind`

**Files:**
- Modify: `src-tauri/src/providers.rs` (`ProviderPreset` add `auth: AuthKind`)
- Modify: `src-tauri/src/adapter.rs`
- Modify: `src-tauri/src/wire.rs`
- Modify: `src-tauri/tests/wire.rs`, `src-tauri/tests/fallback.rs`, `src-tauri/src/lib.rs` test fixtures (~3993)

- [ ] **Step 1: Add field; fix compile with `auth: AuthKind::Bearer` (or `None` for ollama) on every struct literal.** `cargo test` will fail to compile until all literals are updated — that is the red signal.

```rust
// providers.rs
pub struct ProviderPreset {
    pub id: String,
    pub label: String,
    pub endpoint: String,
    pub api_kind: ApiKind,
    pub default_model: String,
    pub needs_key: bool,
    pub auth: linguaray_contracts::AuthKind,
}
```

Delete or replace `presets()` so it maps `linguaray_catalog::load().providers` into `ProviderPreset` (needed by `list_engines` and any startup validation). Gemini stays `ApiKind::OpenAIChat`.

- [ ] **Step 2: adapter**

```rust
let auth = profile.capabilities.auth.unwrap_or(AuthKind::Bearer);
Ok(ProviderPreset { /* existing fields */, auth })
```

- [ ] **Step 3: wire.rs — OpenAIChat arm uses auth, not always bearer**

```rust
let mut req = client.post(&preset.endpoint);
req = match preset.auth {
    AuthKind::Bearer => req.bearer_auth(key),
    AuthKind::AzureKey => req.header("api-key", key),
    AuthKind::XApiKey => req.header("x-api-key", key),
    AuthKind::None => req,
    AuthKind::Query => req.query(&[("key", key)]),
};
req = req.json(&body);
req.send().await
```

Anthropic arm stays `x-api-key` + `anthropic-version` (ignore Bearer even if a bad row says so — protocol wins for Anthropic). Add a unit/integration test with `wiremock` that Xiaomi/Azure (`AuthKind::AzureKey` + OpenAIChat) sends `api-key` and does **not** send `Authorization`.

- [ ] **Step 4: Run**

```bash
cd src-tauri && cargo test --test wire --test fallback
cd src-tauri && cargo test --lib adapter
```

Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git commit -am "feat(wire): send api-key / bearer / none from profile auth"
```

### Task 11: `provider_list_presets` IPC + get_models origin rule

**Files:**
- Modify: `src-tauri/src/lib.rs` (new command + `generate_handler!` ~3738 + `provider_get_models` ~1994)
- Modify: `src-tauri/build.rs` commands array
- Modify: `src-tauri/capabilities/main.json`
- After `cargo build -p linguaray`, commit generated `src-tauri/permissions/autogenerated/provider_list_presets.toml` if the build writes it

- [ ] **Step 1: Command**

```rust
#[derive(Serialize)]
struct CatalogPresetDto {
    id: String,
    label: String,
    endpoint: String,
    default_model: String,
    needs_key: bool,
    auth: AuthKind,
    requires_user_endpoint: bool,
    notes: Option<String>,
    console_url: Option<String>,
    support_tier: SupportTier,
    icon: Option<String>,
}

#[tauri::command]
fn provider_list_presets() -> Result<Vec<CatalogPresetDto>, String> {
    let file = linguaray_catalog::load().map_err(|e| e.to_string())?;
    Ok(file.providers.into_iter().map(|p| CatalogPresetDto {
        id: p.id,
        label: p.label,
        endpoint: p.endpoint,
        default_model: p.default_model,
        needs_key: p.needs_key,
        auth: p.auth,
        requires_user_endpoint: p.requires_user_endpoint,
        notes: p.notes,
        console_url: p.console_url,
        support_tier: p.support_tier,
        icon: p.icon,
    }).collect())
}
```

This command does **not** need DB/keystore. Default deny: only `main` window capability allows it. Popup/input must **not** get the allow.

- [ ] **Step 2: `provider_get_models` origin law (spec §7.4)**

When implementing HTTP fetch (this PR may still return profile.model + catalog default — allowed). If you add HTTP:

```rust
fn models_request_url(profile: &ProviderProfile) -> Result<String, String> {
    let ep = url::Url::parse(&profile.endpoint).map_err(|e| e.to_string())?;
    let stored = profile.capabilities.models_url.as_deref()
        .and_then(|s| url::Url::parse(s).ok());
    match stored {
        Some(u) if u.origin() == ep.origin() => Ok(u.to_string()),
        Some(_) => Err("models_url origin mismatch".into()), // do NOT attach key
        None => derive_models_url(&profile.endpoint, profile.protocol)
            .ok_or_else(|| "cannot derive models url".into()),
    }
}
```

If origin mismatch: return `Err` / empty list, **never** send the key. Add a unit test with a Kimi-shaped profile (`endpoint` on `.ai`, `capabilities.models_url` still `.cn`) asserting the helper errors before any HTTP.

This PR’s minimum: implement the helper + test; keep current “return profile.model + default” behavior **after** the helper would accept, so we don’t silently keep the cross-origin URL for a future fetch.

- [ ] **Step 3: Build to regenerate permissions, add `allow-provider-list-presets` to `capabilities/main.json` only**

```bash
cd src-tauri && cargo build -p linguaray
```

- [ ] **Step 4: Commit**

```bash
git commit -am "feat(ipc): provider_list_presets; models_url same-origin check"
```

### Task 12: Frontend validator + Provider Center grid

**Files:**
- Modify: `src/features/settings/provider-domain.ts` (`validateEndpoint` ~271)
- Modify: `src/features/settings/provider-domain.test.ts` (~199)
- Modify: `apps/ui-lab/src/pages/provider-domain.ts` (keep in sync)
- Modify: `src/features/settings/ProviderCenter.tsx` (`PRESETS` ~97, `presets={PRESETS}` ~1608, `handleAddPreset`, Translate/Test disable)
- Modify: `test/ProviderCenter.test.tsx` (~1360)
- Modify: `apps/ui-lab/src/pages/ProviderCenter.tsx` (stop importing `PRESETS` if unused)
- Create: `src/assets/providers/icon-provider.svg` (simple 24×24 monochrome mark)

- [ ] **Step 1: Failing TS tests**

```ts
it("allows empty endpoint when allowEmpty is set", () => {
  expect(validateEndpoint("", { allowEmpty: true }).ok).toBe(true);
});

it("still requires endpoint by default", () => {
  expect(validateEndpoint("").ok).toBe(false);
  expect((validateEndpoint("") as { code: string }).code).toBe("endpoint-required");
});

it("does not treat 0.0.0.0 as loopback", () => {
  const r = validateEndpoint("http://0.0.0.0:11434");
  expect(r.ok).toBe(false);
});
```

Change signature:

```ts
export function validateEndpoint(
  endpoint: string,
  opts?: { allowEmpty?: boolean },
): EndpointValidationResult {
  const trimmed = endpoint.trim();
  if (!trimmed) {
    return opts?.allowEmpty ? { ok: true } : { ok: false, code: "endpoint-required" };
  }
  // existing logic unchanged
}
```

Mirror in ui-lab `provider-domain.ts`.

- [ ] **Step 2: Expand `Preset` and load from IPC**

```ts
export type SupportTier = "ready" | "setup_required" | "unverified";
export type Preset = {
  templateId: string;
  name: string | null;
  endpoint: string;
  model: string | null;
  needsKey: boolean;
  auth: string;
  requiresUserEndpoint: boolean;
  notes: string | null;
  supportTier: SupportTier;
  icon: string | null;
};
```

In the production controller (not the View), on mount:

```ts
const [presets, setPresets] = createSignal<Preset[]>([]);
onMount(async () => {
  const rows = await invoke<CatalogPresetDto[]>("provider_list_presets");
  setPresets(rows.map(dtoToPreset));
});
```

Pass `presets={presets()}` into `ProviderCenterView`. **Delete `export const PRESETS`.** Lab fixtures pass an explicit array (copy the 4 ready rows plus any visual cases). Tests’ `routeInvoke` must stub `provider_list_presets` returning 21 ids.

- [ ] **Step 3: Replace the 4-preset test**

```ts
it("preset grid lists all official catalog ids and no traditional engines", async () => {
  routeInvoke({
    ...DEFAULT_ROUTES,
    provider_list_presets: () => OFFICIAL_PRESET_DTOS, // 21 rows
  });
  const { findByText } = render(() => <ProviderCenter />);
  expect(await findByText("OpenAI")).toBeTruthy();
  expect(await findByText("DeepSeek")).toBeTruthy();
  expect(await findByText("Azure OpenAI")).toBeTruthy();
  expect(screen.queryByText(/Google Translate/)).toBeNull();
  expect(screen.queryByText(/^DeepL$/)).toBeNull();
});
```

- [ ] **Step 4: UI rules**

- Show a chip: Ready / Setup required / Unverified (i18n keys under `provider.tier.*`).
- Disable Translate / Test / Fetch models when `(requiresUserEndpoint && !endpoint) || (supportTier === "setup_required" && !model)` (covers Doubao empty model).
- Azure notes: button “Insert URL template” writes the v1 template into the endpoint draft (user still replaces `{resource}`).
- Kimi: button “Use global endpoint” sets draft to `https://api.moonshot.ai/v1/chat/completions` (profile update triggers backend origin re-derive).
- Custom: Anthropic switch calls `provider_update` with `{ protocol: "anthropic", expected_version }`.
- Icon: `<img src={iconUrl(p.icon)} />` falling back to `icon-provider.svg`.

- [ ] **Step 5: Run frontend tests**

```bash
pnpm test -- test/ProviderCenter.test.tsx src/features/settings/provider-domain.test.ts
pnpm --filter @linguaray/ui-lab test -- src/pages/provider-domain.ts
```

Expected: PASS. Then `pnpm typecheck`.

- [ ] **Step 6: Commit**

```bash
git commit -am "feat(ui): Provider Center reads catalog IPC with support tiers"
```

### Task 13: Opt-in authenticated smoke (merge gate, not default CI)

**Files:**
- Create: `src-tauri/tests/catalog_smoke.rs`

- [ ] **Step 1: Write ignored-by-default smoke**

```rust
/// Run with:
///   LINGUARAY_SMOKE=1 LINGUARAY_SMOKE_KEY_OPENAI=sk-... cargo test -p linguaray --test catalog_smoke -- --ignored --nocapture
#[tokio::test]
#[ignore]
async fn smoke_ready_or_promoted_rows() {
    if std::env::var("LINGUARAY_SMOKE").ok().as_deref() != Some("1") {
        return;
    }
    // For each catalog row whose env LINGUARAY_SMOKE_KEY_<ID> is set
    // (ID uppercased, '-' → '_'):
    //   build reqwest client (no redirect, 15s)
    //   call wire::call with catalog endpoint/auth/default_model
    //   text "ping" → "zh"
    //   assert result is Ok or Config::AuthFailed (key rejected) — never panic on 401;
    //   record outcome. Promotion to ready still requires a human + spec change.
}
```

Default `cargo test` must **not** hit the network.

- [ ] **Step 2: Document in PR description** that schema tests are the required CI; smoke is maintainer-only; no unverified row is marked ready in this PR.

- [ ] **Step 3: Commit**

```bash
git add src-tauri/tests/catalog_smoke.rs
git commit -m "test(catalog): opt-in authenticated smoke matrix"
```

### Task 14: PR-1 verification sweep

- [ ] **Step 1: Commands**

```bash
cd src-tauri && cargo test
pnpm test
pnpm typecheck
# optional: pnpm tauri build is not required if cargo check + frontend tests pass
```

Expected: all green. No new `ready` rows beyond openai/anthropic/gemini/ollama.

- [ ] **Step 2: Manual checklist (do not claim user-visible done without this)**

1. Settings → Provider Center shows 21 presets with tier chips.
2. Add OpenAI (ready): key + test still works as today.
3. Add Custom: empty endpoint allowed; Translate disabled until HTTPS URL saved.
4. Add Azure: template insert; empty endpoint allowed.
5. Add Doubao: endpoint prefilled, model empty, Translate disabled until model filled.
6. Add Kimi, click global endpoint, save: `capabilities.models_url` origin is `.ai` (inspect via rust test or log).
7. Popup/input windows cannot invoke `provider_list_presets` (permission deny).

- [ ] **Step 3: No extra commit unless fixes are needed.**

---

## Spec coverage

| Spec law | Task |
|---|---|
| LICENSE + NOTICE + `.pnpm-store/` | 1–2 |
| S0 erratum, README, archive Bevy, commit R4 | 3 |
| contracts enums | 4 |
| `src-tauri/crates/` workspace, no root virtual workspace | 5 |
| 21-row JSON, tiers, no relays | 6–7 |
| delete `preset_protocol()`, empty-endpoint law, persist auth/models_url | 8 |
| Custom protocol patch derives auth | 9 |
| one openai-chat driver path + azure-key header | 10 |
| `provider_list_presets`, same-origin models_url | 11 |
| Provider Center IPC, 21 grid, icon fallback, 0.0.0.0 split kept | 12 |
| smoke opt-in, JSON cannot promote ready | 13 |
| GTX / kernel / Fiber | **not this plan** |
| PR-7 worktrees | **not this plan** |

## Placeholder scan

No TBD steps. Catalog JSON is fully specified. Commands are copy-pasteable.

## Type consistency

- `AuthKind` / `ProtocolKind` / `SupportTier` kebab-case serde in contracts == JSON == TS `support_tier` wire (`setup_required`).
- DB `Protocol` unchanged (no CHECK migration).
- `ProviderPreset.auth` is the only new required Rust field on the wire preset; test fixtures must set it.
- IPC DTO field `support_tier` matches catalog serde.

---

**Plan complete and saved to `docs/superpowers/plans/2026-08-14-plugin-core-pr0-pr1.md`. Two execution options:**

**1. Subagent-Driven (recommended)** — I dispatch a fresh subagent per task, review between tasks.

**2. Inline Execution** — I execute tasks in this session with checkpoints.

Which approach?
