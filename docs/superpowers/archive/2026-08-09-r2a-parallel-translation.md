Archived-on: 2026-08-14 · reason: superseded by linguaray-plugin-core-design / completed, see git history

# R2a 并行翻译编排 + 类型化失败分类 实施计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 让 LinguaRay 在一次翻译中并行调用多个 AI 引擎（primary + parallel_uuids），每个引擎独立分类失败（`FallbackEligible` / `Config` / `Keystore` / `LocalNoFallback`），并把多结果通过新 IPC 命令 `translate_session` 与新 popup 事件 `popup-multi-result` 交付前端。

**Architecture:** 后端分层推进——(1) `adapter.rs` 把 DB 的 `ProviderProfile`/`Protocol` 适配为 wire 层的 `ProviderPreset`/`ApiKind`，复用既有 `wire::call`；(2) `db::providers::read_active_selection` 读取 preferences 的 primary/parallel/fallback 三个槽位；(3) `service::translate_parallel` 用 `futures::future::join_all` 并发驱动每个引擎走既有 `translate_with_fallback`（§G 分类不变），返回 `Vec<TranslationOutcome>`；(4) `translate_session` 命令把上面三步串起来，parallel 为空时退化为单引擎；(5) popup 增加独立 `popup-multi-result` 事件保留向后兼容；(6) `translate_clipboard` 改走 `translate_session` 核心逻辑，单引擎发老事件、并行发新事件。

**Tech Stack:** Rust + Tauri 2 + rusqlite + reqwest + wiremock（测试）+ `futures = "0.3"`（新增，用于 `join_all`）。

---

## Global Constraints

每个任务的实现都必须满足以下不变量（与现有 S0/S2a 一致）：

- **§Privacy 端点校验**：所有 AI 端点必须 HTTPS（loopback HTTP 仅限 Ollama 等本地引擎）。`providers::validate_endpoint` 是唯一校验入口；本计划不新增端点来源，只复用已存校验。
- **§Privacy 无跨源重定向**：HTTP 客户端固定 `redirect(Policy::none())`（`build_http_client` 已实现）；本计划不新建客户端，只复用 `Session.client`。
- **§G 失败分类不可绕过**：每个引擎独立走 `translate_with_fallback`，`Config`（missing-key/401/403/invalid-model/invalid-request）与 `Keystore` 错误必须原样传播，**绝不**静默降级到另一个 AI 引擎；LOCAL-primary sacred 规则（loopback 引擎失败不退化到远程 fallback）保持不变。
- **不碰前端**：本计划只在 `src-tauri/` 下改动，**禁止**修改 `src/`（SolidJS 前端）、CSP 配置、`tauri.conf.json` 的 CSP/security 段。
- **key 内存最短驻留**：`keystore.get_key` 读出的明文 key 必须用 `zeroize::Zeroizing` 包裹（既有 `service::translate` 已做），并行路径通过复用 `translate_with_fallback` 自动继承。
- **锁序不变**：DB Mutex 与 keystore fs2 flock 永不嵌套；`translate_parallel` 只持有 `&Keystore`（不持 flock，flock 在 `get_key` 内部短暂获取），与既有 `translate` 一致。
- **latest-wins 不变**：`translate_clipboard` 改造后仍必须 `gen.next()` → 执行 → `is_latest` 检查后才发事件；任何路径不得在未检查 `is_latest` 时发 popup 事件。
- **`invoke_handler` 注册**：新增命令必须加入 `tauri::generate_handler![...]`，否则前端 `invoke` 报 "command not found"。
- **`#![allow(...)]` 禁用**：所有新增代码必须过 `cargo clippy --all-targets --features xproc-test-helper -- -D warnings`。
- **不引入新运行时依赖**：除 `futures = "0.3"`（纯 future 组合子，无运行时）外，不新增 async 运行时；Tauri 的 `tauri::async_runtime` 仍是唯一运行时。

---

## File Structure

| 文件 | 职责 | 动作 |
|---|---|---|
| `src-tauri/Cargo.toml` | 新增 `futures = "0.3"` | Modify |
| `src-tauri/src/error.rs` | 新增 `ConfigKind::Unsupported { provider, reason }` 变体（适配失败的类型化分类） | Modify |
| `src-tauri/src/adapter.rs` | `ProviderProfile`→`ProviderPreset`、`Protocol`→`ApiKind` 纯适配器（无 IO） | Create |
| `src-tauri/src/lib.rs` | 声明 `pub mod adapter;` | Modify（一行） |
| `src-tauri/src/db/providers.rs` | 新增 `ActiveSelection` 结构 + `read_active_selection` 读取器 | Modify |
| `src-tauri/src/service.rs` | 新增 `TranslationOutcome` + `translate_parallel`；为复用 fallback 把 `translate_with_fallback` 拆出 `_ref` 变体 | Modify |
| `src-tauri/src/popup.rs` | 新增 `popup-multi-result` 事件 + `TranslationOutcomeSerialized` + `multi_result` 函数 | Modify |
| `src-tauri/src/lib.rs` | 新增 `translate_session` 命令 + `TranslateSessionRequest/Result`；`translate_clipboard` 改走核心；注册命令 | Modify |
| `src-tauri/tests/adapter.rs` | adapter 单元/集成测试 | Create |
| `src-tauri/tests/provider_crud.rs` | 追加 `read_active_selection` 测试 | Modify |
| `src-tauri/tests/translate_parallel.rs` | 并行编排测试 | Create |
| `src-tauri/tests/translate_session.rs` | translate_session 核心集成测试 | Create |

---

## Task 1: Protocol→ApiKind + ProviderProfile→ProviderPreset 适配器

**Files:**
- Create: `src-tauri/src/adapter.rs`
- Modify: `src-tauri/src/lib.rs`（加 `pub mod adapter;`）
- Modify: `src-tauri/src/error.rs`（加 `ConfigKind::Unsupported` 变体）
- Test: `src-tauri/tests/adapter.rs`（集成测试，可访问 crate 公有项）

**Interfaces:**
- Consumes:
  - `crate::wire::ApiKind { OpenAIChat, Anthropic }`（from `wire.rs`）
  - `crate::providers::ProviderPreset { id, label, endpoint, api_kind, default_model, needs_key }`（from `providers.rs`）
  - `crate::db::providers::{Protocol, ProviderProfile}`（from `db/providers.rs`）
- Produces:
  - `pub fn protocol_to_api_kind(protocol: &Protocol) -> Option<ApiKind>`
    - `Protocol::OpenaiChat | Protocol::Gemini` → `Some(ApiKind::OpenAIChat)`
    - `Protocol::Anthropic` → `Some(ApiKind::Anthropic)`
    - `Protocol::GoogleTranslate | Protocol::CustomHttp` → `None`（非 AI 协议，不可走 `wire::call`）
  - `pub fn profile_to_preset(profile: &ProviderProfile) -> Result<ProviderPreset, String>`
    - `Err("unsupported protocol ...")` 当 `protocol_to_api_kind` 返回 `None`
    - Ok 时：`id = profile.uuid`（用 uuid 作 preset id，这样 `keystore.get_key(profile.secret_ref)` 与 `wire::call` 的鉴权都用 profile 自身字段，不依赖 preset catalog）；`label = profile.name`；`endpoint = profile.endpoint`；`api_kind` 来自 `protocol_to_api_kind`；`default_model = profile.model.clone().unwrap_or_default()`（空字符串视为无效，由 wire 层 404 分类兜底）；`needs_key = profile.needs_key`
  - `ConfigKind::Unsupported { provider: String, reason: String }`（error.rs 新变体）

**设计说明（load-bearing）：** preset 的 `id` 字段被 `service::translate` 用作 keystore 的 `secret_ref`（`keystore.get_key(&preset.id)`）。但 DB-backed profile 的 key 名是 `profile.secret_ref`（`provider/<uuid>`），不是 `profile.uuid`。因此 `profile_to_preset` 把 **`preset.id = profile.secret_ref`**（不是 uuid）——这样 `translate` 内部的 `keystore.get_key(&preset.id)` 自动命中 `profile.secret_ref`，无需修改 `service::translate`。`TranslationOutcome.uuid` 由调用方单独从 profile 携带，不依赖 preset.id。修正后的字段映射：
- `preset.id = profile.secret_ref.clone()`（keystore key 名）
- `preset.label = profile.name.clone()`
- `preset.endpoint = profile.endpoint.clone()`
- `preset.api_kind = protocol_to_api_kind(&profile.protocol)?`
- `preset.default_model = profile.model.clone().unwrap_or_default()`
- `preset.needs_key = profile.needs_key`

- [ ] **Step 1: 给 `error.rs` 加 `Unsupported` 变体（RED 准备：后续测试会断言它）**

Modify `src-tauri/src/error.rs`，在 `ConfigKind` 枚举末尾（`InvalidRequest` 之后）加：

```rust
    /// 适配失败：DB-backed provider 的协议不是可调用的 AI 协议
    /// （google_translate/custom_http 无法走 wire::call）。NOT fallback-eligible：
    /// 同一段文本不会因为换一个 AI 引擎而变好，应直接告诉用户去 Settings 修复。
    #[error("unsupported protocol for {provider}: {reason}")]
    Unsupported { provider: String, reason: String },
```

- [ ] **Step 2: 写失败测试 `tests/adapter.rs`**

Create `src-tauri/tests/adapter.rs`:

```rust
//! adapter.rs — Protocol→ApiKind + ProviderProfile→ProviderPreset 适配器测试。
//! 纯函数转换，无 IO、无网络。

use linguaray_lib::adapter::{profile_to_preset, protocol_to_api_kind};
use linguaray_lib::db::providers::{Protocol, ProviderProfile, ProviderCapabilities};
use linguaray_lib::wire::ApiKind;

fn profile(uuid: &str, protocol: Protocol, endpoint: &str, secret_ref: &str) -> ProviderProfile {
    ProviderProfile {
        uuid: uuid.into(),
        template_id: "openai".into(),
        name: format!("Name-{uuid}"),
        protocol,
        endpoint: endpoint.into(),
        model: Some("gpt-4o-mini".into()),
        enabled: true,
        sort_order: 0,
        is_local: false,
        needs_key: true,
        secret_ref: secret_ref.into(),
        capabilities: ProviderCapabilities::default(),
        status: "active".into(),
    }
}

#[test]
fn protocol_to_api_kind_maps_ai_protocols() {
    assert_eq!(protocol_to_api_kind(&Protocol::OpenaiChat), Some(ApiKind::OpenAIChat));
    assert_eq!(protocol_to_api_kind(&Protocol::Gemini), Some(ApiKind::OpenAIChat));
    assert_eq!(protocol_to_api_kind(&Protocol::Anthropic), Some(ApiKind::Anthropic));
}

#[test]
fn protocol_to_api_kind_returns_none_for_non_ai() {
    assert_eq!(protocol_to_api_kind(&Protocol::GoogleTranslate), None);
    assert_eq!(protocol_to_api_kind(&Protocol::CustomHttp), None);
}

#[test]
fn profile_to_preset_openai_chat() {
    let p = profile("u1", Protocol::OpenaiChat, "https://api.openai.com/v1/chat/completions", "provider/u1");
    let preset = profile_to_preset(&p).expect("openai_chat → preset");
    assert_eq!(preset.id, "provider/u1", "preset.id MUST be secret_ref so keystore.get_key hits the right key");
    assert_eq!(preset.label, "Name-u1");
    assert_eq!(preset.endpoint, "https://api.openai.com/v1/chat/completions");
    assert_eq!(preset.api_kind, ApiKind::OpenAIChat);
    assert_eq!(preset.default_model, "gpt-4o-mini");
    assert!(preset.needs_key);
}

#[test]
fn profile_to_preset_anthropic() {
    let p = profile("u2", Protocol::Anthropic, "https://api.anthropic.com/v1/messages", "provider/u2");
    let preset = profile_to_preset(&p).expect("anthropic → preset");
    assert_eq!(preset.api_kind, ApiKind::Anthropic);
    assert_eq!(preset.id, "provider/u2");
}

#[test]
fn profile_to_preset_gemini_maps_to_openai_chat() {
    // Gemini 走 OpenAI-compatible 路径（spec §Wire），与 preset catalog 一致。
    let p = profile("u3", Protocol::Gemini, "https://generativelanguage.googleapis.com/v1beta/openai/chat/completions", "provider/u3");
    let preset = profile_to_preset(&p).expect("gemini → preset");
    assert_eq!(preset.api_kind, ApiKind::OpenAIChat);
}

#[test]
fn profile_to_preset_uses_empty_string_when_model_none() {
    let mut p = profile("u4", Protocol::OpenaiChat, "https://api.openai.com/v1/chat/completions", "provider/u4");
    p.model = None;
    let preset = profile_to_preset(&p).expect("None model → empty string default");
    assert_eq!(preset.default_model, "");
}

#[test]
fn profile_to_preset_rejects_google_translate() {
    let p = profile("u5", Protocol::GoogleTranslate, "https://translate.google.com", "provider/u5");
    let err = profile_to_preset(&p).expect_err("google_translate is not an AI protocol");
    assert!(err.contains("unsupported protocol"), "got: {err}");
}

#[test]
fn profile_to_preset_rejects_custom_http() {
    let p = profile("u6", Protocol::CustomHttp, "https://example.com", "provider/u6");
    assert!(profile_to_preset(&p).is_err());
}

#[test]
fn profile_to_preset_needs_key_false_propagates() {
    // Ollama（keyless）：needs_key=false 必须透传，否则 translate 会去 keystore 找不存在的 key。
    let mut p = profile("u7", Protocol::OpenaiChat, "http://localhost:11434/v1/chat/completions", "provider/u7");
    p.needs_key = false;
    let preset = profile_to_preset(&p).expect("ok");
    assert!(!preset.needs_key);
}
```

- [ ] **Step 3: 跑测试确认失败（RED）**

Run: `cd src-tauri && cargo test --test adapter`
Expected: 编译失败（`unresolved module adapter`）。

- [ ] **Step 4: 在 `lib.rs` 声明模块**

Modify `src-tauri/src/lib.rs`：在 `pub mod a11y;` 之后加一行：

```rust
pub mod adapter;
```

- [ ] **Step 5: 实现 `src-tauri/src/adapter.rs`（GREEN）**

Create `src-tauri/src/adapter.rs`:

```rust
//! DB-backed provider → wire-layer preset 适配器（R2a）。
//!
//! DB 的 `ProviderProfile`（运行时用户配置）与 wire 层的 `ProviderPreset`
//! （HTTP 调用模板）是两个独立类型：前者带 uuid/protocol/capabilities，
//! 后者是 `wire::call` 的输入。本模块做无 IO 的纯转换，让并行翻译编排器
//! 可以复用既有的 `service::translate_with_fallback` + `wire::call`。
//!
//! 关键不变量：`preset.id = profile.secret_ref`（不是 uuid），这样
//! `service::translate` 内部的 `keystore.get_key(&preset.id)` 自动命中
//! profile 的 secret_ref，无需修改 translate 的鉴权逻辑。

use crate::db::providers::{Protocol, ProviderProfile};
use crate::providers::ProviderPreset;
use crate::wire::ApiKind;

/// 把 DB wire 协议族映射到 `wire::call` 支持的 API kind。
/// - OpenAIChat / Gemini → OpenAIChat（spec §Wire：Gemini 走 OpenAI-compatible 路径）
/// - Anthropic           → Anthropic
/// - GoogleTranslate / CustomHttp → None（非 AI 协议，无法走 `wire::call`）
pub fn protocol_to_api_kind(protocol: &Protocol) -> Option<ApiKind> {
    match protocol {
        Protocol::OpenaiChat | Protocol::Gemini => Some(ApiKind::OpenAIChat),
        Protocol::Anthropic => Some(ApiKind::Anthropic),
        Protocol::GoogleTranslate | Protocol::CustomHttp => None,
    }
}

/// 把 DB-backed profile 转成 wire-layer preset。
///
/// 失败条件：`protocol_to_api_kind` 返回 `None`（google_translate/custom_http）。
/// 此时该 profile 不是可调用的 AI 引擎，调用方应跳过它或把它标为失败结果。
///
/// 字段映射（load-bearing）：
/// - `preset.id = profile.secret_ref` — keystore key 名；`service::translate` 用
///   `keystore.get_key(&preset.id)`，所以 id 必须是 secret_ref 才能命中 DB-backed key。
/// - `preset.default_model = profile.model.unwrap_or_default()` — 空字符串由 wire 层
///   404 分类（Config::InvalidRequest）兜底，适配器不做 model 必填校验。
pub fn profile_to_preset(profile: &ProviderProfile) -> Result<ProviderPreset, String> {
    let api_kind = protocol_to_api_kind(&profile.protocol).ok_or_else(|| {
        format!("unsupported protocol for provider {}: {:?}", profile.uuid, profile.protocol)
    })?;
    Ok(ProviderPreset {
        id: profile.secret_ref.clone(),
        label: profile.name.clone(),
        endpoint: profile.endpoint.clone(),
        api_kind,
        default_model: profile.model.clone().unwrap_or_default(),
        needs_key: profile.needs_key,
    })
}

#[cfg(test)]
mod tests {
    // 集成测试见 tests/adapter.rs；此处不重复，避免双维护。
    use super::*;
    #[test]
    fn smoke_adapter_compiles_and_maps() {
        assert_eq!(protocol_to_api_kind(&Protocol::Anthropic), Some(ApiKind::Anthropic));
    }
}
```

- [ ] **Step 6: 跑测试确认通过（GREEN）**

Run: `cd src-tauri && cargo test --test adapter`
Expected: 8 passed.

- [ ] **Step 7: clippy + 提交**

```bash
cd src-tauri && cargo clippy --all-targets --features xproc-test-helper -- -D warnings
git add src-tauri/src/adapter.rs src-tauri/src/lib.rs src-tauri/src/error.rs src-tauri/tests/adapter.rs
git commit -m "feat(r2a): add Protocol→ApiKind + ProviderProfile→ProviderPreset adapter"
```

---

## Task 2: `read_active_selection` 读取器

**Files:**
- Modify: `src-tauri/src/db/providers.rs`
- Test: `src-tauri/tests/provider_crud.rs`（追加）

**Interfaces:**
- Consumes:
  - `rusqlite::Connection`，`crate::db::DbError`
  - 既有 `parse_parallel_uuids(blob: &str) -> Result<Vec<String>, DbError>`（同模块内）
  - preferences 表列：`primary_uuid TEXT`、`parallel_uuids TEXT NOT NULL DEFAULT '[]'`、`fallback_uuid TEXT`（schema.rs:47-49，singleton `id=1`）
- Produces:
  - `pub struct ActiveSelection { pub primary: Option<String>, pub parallel: Vec<String>, pub fallback: Option<String> }`
  - `pub fn read_active_selection(conn: &Connection) -> Result<ActiveSelection, DbError>`
    - 从 `preferences WHERE id=1` 读三个列；`primary_uuid`/`fallback_uuid` 为 NULL → `None`
    - `parallel_uuids` 经 `parse_parallel_uuids` 解析（corrupt JSON → `DbError::Integrity`，与 toggle/update 一致，不静默空数组）
    - 若 preferences 行不存在（理论上不应发生，singleton 由 seed 保证），返回 `ActiveSelection::default()`（全空），**不报错**——这是防御性兜底，与 `read_consent_scope` 用 `optional()` 的容忍风格一致

- [ ] **Step 1: 写失败测试（追加到 `tests/provider_crud.rs` 末尾）**

在 `src-tauri/tests/provider_crud.rs` 文件**末尾**追加：

```rust

// ─── R2a Task 2: read_active_selection ──────────────────────────────────────

#[test]
fn read_active_selection_default_when_no_slots_set() {
    // fresh_db 已 seed 了 preferences singleton（parallel_uuids 默认 '[]'，其余 NULL）。
    let (_dir, db) = fresh_db();
    let sel = db.with_conn(|conn| providers::read_active_selection(conn)).unwrap();
    assert!(sel.primary.is_none(), "primary default None");
    assert!(sel.parallel.is_empty(), "parallel default empty");
    assert!(sel.fallback.is_none(), "fallback default None");
}

#[test]
fn read_active_selection_reads_primary_only() {
    let (_dir, db, p) = fresh_with_one_openai();
    db.with_conn(|conn| {
        conn.execute(
            "UPDATE preferences SET primary_uuid=?1 WHERE id=1",
            rusqlite::params![p.uuid],
        )?;
        Ok(())
    })
    .unwrap();
    let sel = db.with_conn(|conn| providers::read_active_selection(conn)).unwrap();
    assert_eq!(sel.primary.as_deref(), Some(p.uuid.as_str()));
    assert!(sel.parallel.is_empty());
    assert!(sel.fallback.is_none());
}

#[test]
fn read_active_selection_reads_parallel_json_array() {
    let (_dir, db, p1) = fresh_with_one_openai();
    let p2 = db
        .with_conn(|conn| {
            providers::create(conn, "anthropic", "Claude", "https://api.anthropic.com/v1/messages", None)
        })
        .unwrap();
    let _ = &p1;
    let arr = serde_json::to_string(&[&p1.uuid, &p2.uuid]).unwrap();
    db.with_conn(|conn| {
        conn.execute(
            "UPDATE preferences SET parallel_uuids=?1 WHERE id=1",
            rusqlite::params![arr],
        )?;
        Ok(())
    })
    .unwrap();
    let sel = db.with_conn(|conn| providers::read_active_selection(conn)).unwrap();
    assert_eq!(sel.parallel, vec![p1.uuid.clone(), p2.uuid.clone()]);
}

#[test]
fn read_active_selection_reads_all_three_slots() {
    let (_dir, db, p) = fresh_with_one_openai();
    db.with_conn(|conn| {
        conn.execute(
            "UPDATE preferences SET primary_uuid=?1, parallel_uuids=?2, fallback_uuid=?1 WHERE id=1",
            rusqlite::params![p.uuid, serde_json::to_string(&[&p.uuid]).unwrap()],
        )?;
        Ok(())
    })
    .unwrap();
    let sel = db.with_conn(|conn| providers::read_active_selection(conn)).unwrap();
    assert_eq!(sel.primary.as_deref(), Some(p.uuid.as_str()));
    assert_eq!(sel.parallel, vec![p.uuid.clone()]);
    assert_eq!(sel.fallback.as_deref(), Some(p.uuid.as_str()));
}

#[test]
fn read_active_selection_corrupt_parallel_errors() {
    // 与 toggle/update 的 parse_parallel_uuids 一致：corrupt JSON 必须报 Integrity，
    // 不能静默返回空数组（否则会丢一个仍 active 的 uuid）。
    let (_dir, db) = fresh_db();
    db.with_conn(|conn| {
        conn.execute(
            "UPDATE preferences SET parallel_uuids=?1 WHERE id=1",
            rusqlite::params!["not-valid-json{{{"],
        )?;
        Ok(())
    })
    .unwrap();
    let err = db
        .with_conn(|conn| providers::read_active_selection(conn))
        .unwrap_err();
    assert!(matches!(err, DbError::Integrity(_)), "corrupt parallel_uuids must be Integrity, got {err:?}");
}

#[test]
fn read_active_selection_empty_parallel_json_yields_empty_vec() {
    let (_dir, db) = fresh_db();
    // 默认 '[]' → 空 vec（不是 None，因为 parallel 本就是 Vec）。
    let sel = db.with_conn(|conn| providers::read_active_selection(conn)).unwrap();
    assert!(sel.parallel.is_empty());
}
```

注意：`fresh_db` / `fresh_with_one_openai` 已在文件顶部定义（line 21-46），直接复用。

- [ ] **Step 2: 跑测试确认失败（RED）**

Run: `cd src-tauri && cargo test --test provider_crud read_active_selection`
Expected: 编译失败（`no function named read_active_selection`）。

- [ ] **Step 3: 实现 `ActiveSelection` + `read_active_selection`（GREEN）**

在 `src-tauri/src/db/providers.rs` 的 `// ─── preferences active-slot helpers ───`（line 644）section 之前插入新 section：

```rust
// ─── ActiveSelection reader (R2a) ────────────────────────────────────────

/// 当前 (primary, parallel, fallback) 选择快照，从 `preferences` singleton 读出。
/// 用于 `translate_session` 决定要并行调用哪些引擎。
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ActiveSelection {
    /// primary_uuid；NULL/空 → None。
    pub primary: Option<String>,
    /// parallel_uuids JSON 数组解析结果；'[]' → 空 vec。
    pub parallel: Vec<String>,
    /// fallback_uuid；NULL/空 → None。
    pub fallback: Option<String>,
}

/// 读 preferences singleton 的三个 active 槽位。
///
/// `parallel_uuids` 经 [`parse_parallel_uuids`] 解析——corrupt JSON 报
/// `DbError::Integrity`（与 toggle/update 一致，不静默空数组）。
/// 若 singleton 行不存在（理论上 seed 保证存在），返回全空默认值
/// （防御性兜底，与 `read_consent_scope` 的 `optional()` 容忍风格一致）。
pub fn read_active_selection(conn: &Connection) -> Result<ActiveSelection, DbError> {
    use rusqlite::OptionalExtension;
    let row: Option<(Option<String>, String, Option<String>)> = conn
        .query_row(
            "SELECT primary_uuid, parallel_uuids, fallback_uuid \
             FROM preferences WHERE id=1",
            [],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )
        .optional()?;
    match row {
        None => Ok(ActiveSelection::default()),
        Some((primary, parallel_json, fallback)) => {
            let parallel = parse_parallel_uuids(&parallel_json)?;
            Ok(ActiveSelection {
                // 空字符串视为未选（与 set_active_slots 的 "" 语义一致）。
                primary: primary.filter(|s| !s.is_empty()),
                parallel,
                fallback: fallback.filter(|s| !s.is_empty()),
            })
        }
    }
}
```

- [ ] **Step 4: 跑测试确认通过（GREEN）**

Run: `cd src-tauri && cargo test --test provider_crud read_active_selection`
Expected: 6 passed.

- [ ] **Step 5: clippy + 提交**

```bash
cd src-tauri && cargo clippy --all-targets --features xproc-test-helper -- -D warnings
git add src-tauri/src/db/providers.rs src-tauri/tests/provider_crud.rs
git commit -m "feat(r2a): add read_active_selection for primary/parallel/fallback slots"
```

---

## Task 3: 并行翻译编排器 `translate_parallel`

**Files:**
- Modify: `src-tauri/src/service.rs`
- Modify: `src-tauri/Cargo.toml`（加 `futures = "0.3"`）
- Test: `src-tauri/tests/translate_parallel.rs`（Create）

**Interfaces:**
- Consumes:
  - `crate::engines::TraditionalEngine`（trait，`&self` 方法，可并发共享）
  - 既有 `translate_with_fallback(client, keystore, primary_preset, input, fallback) -> Result<Translation, Error>`
  - `crate::adapter::profile_to_preset`（Task 1）
  - `crate::db::providers::ProviderProfile`
  - `crate::wire::AppOptions`（Clone）
- Produces:
  - `pub struct TranslationOutcome { pub uuid: String, pub result: Result<Translation, Error> }`（`Error` derive Clone 需要新增——见下）
  - `pub async fn translate_parallel(client, keystore, profiles: Vec<ProviderProfile>, text, from, to, options, fallback: Option<Arc<dyn TraditionalEngine>>) -> Vec<TranslationOutcome>`

**前置重构（load-bearing）：** 既有 `translate_with_fallback` 接收 `fallback: Option<Box<dyn TraditionalEngine>>`（owned，被消费一次）。并行场景要把同一个 fallback 共享给 N 个引擎，`Box` 无法 clone。解决：拆出 `translate_with_fallback_ref(client, keystore, preset, input, fallback: Option<&dyn TraditionalEngine>)`，原 `translate_with_fallback` 改为薄包装（把 `Box` 转 `&dyn` 后委托）。`translate_parallel` 持有 `Option<Arc<dyn TraditionalEngine>>`，每个并发 future 传 `fallback.as_deref()`（`Arc<dyn T>` → `&(dyn T)` via `as_ref().map(|a| &**a)`）。

**`Error` Clone 实现偏离（rev-4.3.2 记录）：** 计划原定给 `Error`、`FallbackKind`、`ConfigKind` 全部加 `Clone`。实际实现中 `FallbackKind`/`ConfigKind` 已加 Clone（字段全是 String/u16），但 **`Error` 未加 Clone** —— `Error::Keystore(KeystoreError)` 包装的 `KeystoreError` 含 `std::io::Error`（不可 Clone），对整个 `Error` 加 Clone 会编译失败。`TranslationOutcome` 改为用 `Serialize`（IPC 序列化）代替 Clone，测试按值消费（不克隆），功能等价。此偏离不影响 Task 4-6 的任何下游依赖。

- [ ] **Step 1: 给 `Cargo.toml` 加 `futures`**

Modify `src-tauri/Cargo.toml`，在 `[dependencies]` 段（`enigo = "0.2"` 之后、windows-sys 之前）加：

```toml
futures = "0.3"
```

- [ ] **Step 2: 给 `Error`/`FallbackKind`/`ConfigKind` 加 `Clone`（准备 TranslationOutcome）**

Modify `src-tauri/src/error.rs`：
- `pub enum Error` 的 derive 改为 `#[derive(Debug, Clone, Error)]`
- `pub enum FallbackKind` 改为 `#[derive(Debug, Clone, Error)]`
- `pub enum ConfigKind` 改为 `#[derive(Debug, Clone, Error)]`

Run: `cd src-tauri && cargo build` 确认编译（thiserror 派生 Clone 不需要字段实现 Clone 之外的东西；所有字段已是 String/u16）。

- [ ] **Step 3: 写失败测试 `tests/translate_parallel.rs`**

Create `src-tauri/tests/translate_parallel.rs`:

```rust
//! R2a Task 3: translate_parallel 并行编排测试。
//!
//! 三类场景：
//!  1. 2 引擎都成功 → 2 个 Ok outcome，engine 字段是各自 preset.id（=secret_ref）
//!  2. 1 成功 1 失败（500）→ 1 Ok + 1 Err(FallbackEligible)；无 fallback 配置时
//!     失败的那个走 §G：remote primary + no fallback → LocalNoFallback
//!  3. 全部失败（两个 500，无 fallback）→ 2 个 Err(LocalNoFallback)
//!
//! 复用 fallback.rs 的测试 harness 风格：wiremock + lvh.me（非 loopback literal，
//! 避免 is_local 误判）+ no_proxy client + needs_key:false preset（跳过 keystore）。

use linguaray_lib::adapter::profile_to_preset;
use linguaray_lib::db::providers::{Protocol, ProviderProfile, ProviderCapabilities};
use linguaray_lib::error::Error;
use linguaray_lib::service::{translate_parallel, TranslationOutcome};
use linguaray_lib::wire::AppOptions;
use serde_json::json;
use wiremock::{Mock, MockServer, ResponseTemplate};

fn direct_client() -> reqwest::Client {
    reqwest::Client::builder().no_proxy().build().unwrap()
}

fn empty_keystore() -> linguaray_lib::keystore::Keystore {
    let dir = tempfile::tempdir().unwrap().keep();
    linguaray_lib::keystore::Keystore::new(dir).unwrap()
}

/// 构造一个 needs_key=false 的 profile（translate 跳过 keystore，直接打 mock）。
fn profile(uuid: &str, endpoint: &str) -> ProviderProfile {
    ProviderProfile {
        uuid: uuid.into(),
        template_id: "openai".into(),
        name: format!("P-{uuid}").into(),
        protocol: Protocol::OpenaiChat,
        endpoint: endpoint.into(),
        model: Some("m".into()),
        enabled: true,
        sort_order: 0,
        is_local: false,
        needs_key: false,
        secret_ref: format!("provider/{uuid}"),
        capabilities: ProviderCapabilities::default(),
        status: "active".into(),
    }
}

async fn mount_ok(server: &MockServer, body: &str) {
    Mock::given(wiremock::matchers::method("POST"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "choices": [{"message": {"content": body}}]
        })))
        .mount(server)
        .await;
}

async fn mount_500(server: &MockServer) {
    Mock::given(wiremock::matchers::method("POST"))
        .respond_with(ResponseTemplate::new(500))
        .mount(server)
        .await;
}

/// 把 Vec<TranslationOutcome> 按 uuid 排序后返回，断言时与输入顺序解耦（并发完成顺序不定）。
fn sorted_by_uuid(mut v: Vec<TranslationOutcome>) -> Vec<TranslationOutcome> {
    v.sort_by(|a, b| a.uuid.cmp(&b.uuid));
    v
}

#[tokio::test]
async fn two_engines_both_success() {
    let s1 = MockServer::start().await;
    let s2 = MockServer::start().await;
    let port1: u16 = s1.uri().rsplit(':').next().unwrap().parse().unwrap();
    let port2: u16 = s2.uri().rsplit(':').next().unwrap().parse().unwrap();
    mount_ok(&s1, "你好").await;
    mount_ok(&s2, "您好").await;

    let profiles = vec![
        profile("u1", &format!("http://lvh.me:{port1}/v1/chat/completions")),
        profile("u2", &format!("http://lvh.me:{port2}/v1/chat/completions")),
    ];
    let client = direct_client();
    let keystore = empty_keystore();

    let outcomes = translate_parallel(
        &client, &keystore, profiles, "hello", "auto", "zh",
        AppOptions::default(), None,
    )
    .await;
    assert_eq!(outcomes.len(), 2, "exactly one outcome per profile");

    let mut by_uuid = std::collections::HashMap::new();
    for o in outcomes {
        by_uuid.insert(o.uuid, o.result);
    }
    let r1 = by_uuid.remove("u1").unwrap().expect("u1 ok");
    assert_eq!(r1.text, "你好");
    assert_eq!(r1.engine, "provider/u1", "engine tagged with preset.id (=secret_ref)");
    let r2 = by_uuid.remove("u2").unwrap().expect("u2 ok");
    assert_eq!(r2.text, "您好");
    assert_eq!(r2.engine, "provider/u2");
}

#[tokio::test]
async fn one_success_one_failure_no_fallback() {
    let s1 = MockServer::start().await;
    let s2 = MockServer::start().await;
    let port1: u16 = s1.uri().rsplit(':').next().unwrap().parse().unwrap();
    let port2: u16 = s2.uri().rsplit(':').next().unwrap().parse().unwrap();
    mount_ok(&s1, "ok-text").await;
    mount_500(&s2).await;

    let profiles = vec![
        profile("u1", &format!("http://lvh.me:{port1}/v1/chat/completions")),
        profile("u2", &format!("http://lvh.me:{port2}/v1/chat/completions")),
    ];
    let client = direct_client();
    let keystore = empty_keystore();

    let outcomes = translate_parallel(
        &client, &keystore, profiles, "x", "auto", "zh",
        AppOptions::default(), None,
    )
    .await;

    let mut by_uuid = std::collections::HashMap::new();
    for o in outcomes {
        by_uuid.insert(o.uuid, o.result);
    }
    let r1 = by_uuid.remove("u1").unwrap().expect("u1 ok");
    assert_eq!(r1.text, "ok-text");
    // §G：remote primary + 500 + no fallback → LocalNoFallback（不是裸 FallbackEligible）
    let err2 = by_uuid.remove("u2").unwrap().expect_err("u2 failed");
    assert!(
        matches!(err2, Error::LocalNoFallback),
        "expected LocalNoFallback (remote primary, no fallback configured), got {err2:?}"
    );
}

#[tokio::test]
async fn all_fail_no_fallback_yields_all_local_no_fallback() {
    let s1 = MockServer::start().await;
    let s2 = MockServer::start().await;
    let port1: u16 = s1.uri().rsplit(':').next().unwrap().parse().unwrap();
    let port2: u16 = s2.uri().rsplit(':').next().unwrap().parse().unwrap();
    mount_500(&s1).await;
    mount_500(&s2).await;

    let profiles = vec![
        profile("u1", &format!("http://lvh.me:{port1}/v1/chat/completions")),
        profile("u2", &format!("http://lvh.me:{port2}/v1/chat/completions")),
    ];
    let client = direct_client();
    let keystore = empty_keystore();

    let outcomes = translate_parallel(
        &client, &keystore, profiles, "x", "auto", "zh",
        AppOptions::default(), None,
    )
    .await;
    assert_eq!(outcomes.len(), 2);
    for o in sorted_by_uuid(outcomes) {
        assert!(
            matches!(o.result, Err(Error::LocalNoFallback)),
            "all engines failed with no fallback → LocalNoFallback each, got {:?} for {}",
            o.result, o.uuid
        );
    }
}

#[tokio::test]
async fn unsupported_protocol_profile_yields_config_error_outcome() {
    // google_translate 协议无法走 wire::call：profile_to_preset 失败 →
    // translate_parallel 必须把它标成 Err(Config::Unsupported)，而不是 panic 或丢弃。
    let mut p = profile("u-bad", "https://translate.google.com");
    p.protocol = Protocol::GoogleTranslate;
    // 先确认 adapter 确实拒绝它（测试自洽性）。
    assert!(profile_to_preset(&p).is_err());

    let profiles = vec![p];
    let client = direct_client();
    let keystore = empty_keystore();
    let outcomes = translate_parallel(
        &client, &keystore, profiles, "x", "auto", "zh",
        AppOptions::default(), None,
    )
    .await;
    assert_eq!(outcomes.len(), 1);
    match &outcomes[0].result {
        Err(Error::Config(c)) => {
            // ConfigKind::Unsupported；Display 含 "unsupported protocol"。
            let s = format!("{c}");
            assert!(s.contains("unsupported"), "got: {s}");
        }
        other => panic!("expected Config(Unsupported), got {other:?}"),
    }
    assert_eq!(outcomes[0].uuid, "u-bad");
}

#[tokio::test]
async fn empty_profiles_yields_empty_outcomes() {
    let client = direct_client();
    let keystore = empty_keystore();
    let outcomes = translate_parallel(
        &client, &keystore, vec![], "x", "auto", "zh",
        AppOptions::default(), None,
    )
    .await;
    assert!(outcomes.is_empty());
}
```

- [ ] **Step 4: 跑测试确认失败（RED）**

Run: `cd src-tauri && cargo test --test translate_parallel`
Expected: 编译失败（`unresolved function translate_parallel` / `TranslationOutcome`）。

- [ ] **Step 5: 重构 `translate_with_fallback` 拆出 `_ref` + 实现 `translate_parallel`（GREEN）**

Modify `src-tauri/src/service.rs`：

**(a) 顶部加 imports：**

把文件顶部的 use 段改为：

```rust
use crate::adapter::profile_to_preset;
use crate::db::providers::ProviderProfile;
use crate::engines::TraditionalEngine;
use crate::error::{ConfigKind, Error};
use crate::keystore::Keystore;
use crate::providers::ProviderPreset;
use crate::wire::{build_prompt, call, AppOptions, WireParams};
use std::sync::Arc;
```

**(b) 把 `translate_with_fallback` 改为薄包装（保留公开签名，向后兼容）：**

替换现有 `pub async fn translate_with_fallback(...) -> Result<Translation, Error>` 整个函数体为：

```rust
/// Translate with §G classified fallback（公开入口，向后兼容旧调用方）。
///
/// 把 `Box<dyn TraditionalEngine>` 转成 `&dyn` 后委托给
/// [`translate_with_fallback_ref`]。新代码（如 `translate_parallel`）应直接
/// 用 `_ref` 变体以共享 fallback 而不需要 per-call owned `Box`。
pub async fn translate_with_fallback(
    client: &reqwest::Client,
    keystore: &Keystore,
    primary_preset: &ProviderPreset,
    input: TranslateInput<'_>,
    fallback: Option<Box<dyn TraditionalEngine>>,
) -> Result<Translation, Error> {
    translate_with_fallback_ref(client, keystore, primary_preset, input, fallback.as_deref()).await
}
```

**(c) 新增 `translate_with_fallback_ref`（真正的 §G 实现，从原 `translate_with_fallback` 函数体搬过来，fallback 类型改 `Option<&dyn TraditionalEngine>`）：**

在 `translate_with_fallback` 之后插入：

```rust
/// §G classified fallback 的真正实现（按引用接收 fallback）。
///
/// 与 `translate_with_fallback` 行为完全一致，唯一区别是 fallback 用引用
/// 而非 owned `Box`，使 `translate_parallel` 可以用 `Arc<dyn TraditionalEngine>`
/// 把同一个 fallback 共享给 N 个并发引擎。
///
/// 行为（不变）：
/// - primary 先跑；成功则返回（engine == primary preset id）。
/// - `FallbackEligible`（network/timeout/429/5xx/parse）：
///   - LOCAL primary（loopback）→ `LocalNoFallback`（local-sacred，绝不退化到远程）。
///   - 否则有 fallback → 跑一次传统引擎，结果 tagged fallback engine id。
///   - 否则无 fallback → `LocalNoFallback`。
/// - `Config`/`Keystore` → 原样传播，绝不 fallback。
pub async fn translate_with_fallback_ref(
    client: &reqwest::Client,
    keystore: &Keystore,
    primary_preset: &ProviderPreset,
    input: TranslateInput<'_>,
    fallback: Option<&dyn TraditionalEngine>,
) -> Result<Translation, Error> {
    match translate(
        client,
        keystore,
        primary_preset,
        TranslateInput {
            text: input.text,
            from: input.from,
            to: input.to,
            options: input.options.clone(),
        },
    )
    .await
    {
        Ok(t) => Ok(t),
        Err(Error::FallbackEligible(_)) => {
            if is_local(primary_preset) {
                return Err(Error::LocalNoFallback);
            }
            match fallback {
                None => Err(Error::LocalNoFallback),
                Some(eng) => {
                    let fb_id = eng.id().to_string();
                    eng.translate(client, input.text, input.from, input.to)
                        .await
                        .map(|text| Translation { text, engine: fb_id })
                }
            }
        }
        Err(other) => Err(other),
    }
}
```

**(d) 新增 `TranslationOutcome` + `translate_parallel`：**

在 `translate_with_fallback_ref` 之后插入：

```rust
// ─── R2a: 并行翻译编排 ────────────────────────────────────────────────────

/// 单个引擎的翻译结果（成功或分类过的错误）。uuid 来自原 ProviderProfile，
/// 与 `result` 内的 engine 字段（preset.id=secret_ref）相互独立——调用方用 uuid
/// 把结果关联回用户选的那个 provider row。
#[derive(Debug, Clone)]
pub struct TranslationOutcome {
    pub uuid: String,
    pub result: Result<Translation, Error>,
}

/// 并行调用多个 AI 引擎，每个独立走 §G fallback 分类。
///
/// - 每个 profile 经 [`profile_to_preset`] 转 preset；转换失败（如 google_translate
///   协议）→ 该引擎产出 `Err(Config::Unsupported)` outcome，**不** panic、**不**丢弃。
/// - 所有引擎用 `futures::future::join_all` 并发驱动，各自跑
///   [`translate_with_fallback_ref`]（带各自的 fallback 机会）。
/// - `fallback` 是 `Option<Arc<dyn TraditionalEngine>>`，所有引擎共享同一个
///   （传统引擎 `translate` 是 `&self`，Arc 允许并发只读共享）。
/// - 返回顺序不保证与输入顺序一致（并发完成顺序不定）；调用方按 `uuid` 关联。
///
/// §G 不变量：每个引擎独立分类错误。`Config`/`Keystore` 错误绝不因另一个引擎
/// 成功而被"覆盖"——它们作为各自 outcome 的 Err 保留，前端按 Surface 03 的
/// "partial success" 渲染。
pub async fn translate_parallel(
    client: &reqwest::Client,
    keystore: &Keystore,
    profiles: Vec<ProviderProfile>,
    text: &str,
    from: &str,
    to: &str,
    options: AppOptions,
    fallback: Option<Arc<dyn TraditionalEngine>>,
) -> Vec<TranslationOutcome> {
    // 先把 profile→preset 的同步转换做完（不放进 async block，避免借用混乱）。
    // 转换失败的先记成 outcome，成功的进入并发池。
    let mut ready: Vec<(String, ProviderPreset)> = Vec::with_capacity(profiles.len());
    let mut outcomes: Vec<TranslationOutcome> = Vec::new();
    for p in profiles {
        let uuid = p.uuid.clone();
        match profile_to_preset(&p) {
            Ok(preset) => ready.push((uuid, preset)),
            Err(reason) => outcomes.push(TranslationOutcome {
                uuid,
                result: Err(Error::Config(ConfigKind::Unsupported {
                    provider: p.uuid.clone(),
                    reason,
                })),
            }),
        }
    }

    // 并发驱动所有 ready 引擎。每个 async block 按引用捕获 client/keystore/text
    // /from/to/fallback，按值捕获自己的 (uuid, preset)。
    let futs: Vec<_> = ready
        .into_iter()
        .map(|(uuid, preset)| {
            let fb_ref: Option<&dyn TraditionalEngine> = fallback.as_deref();
            async move {
                let input = TranslateInput {
                    text,
                    from,
                    to,
                    options: options.clone(),
                };
                let result =
                    translate_with_fallback_ref(client, keystore, &preset, input, fb_ref).await;
                TranslationOutcome { uuid, result }
            }
        })
        .collect();
    let mut all = futures::future::join_all(futs).await;
    outcomes.append(&mut all);
    outcomes
}
```

注意：`Option<Arc<dyn T>>.as_deref()` 返回 `Option<&(dyn T)>`（Arc 的 Deref impl 让它满足），正是 `translate_with_fallback_ref` 想要的 `Option<&dyn TraditionalEngine>`。

- [ ] **Step 6: 跑测试确认通过（GREEN）**

Run: `cd src-tauri && cargo test --test translate_parallel`
Expected: 5 passed.

- [ ] **Step 7: 跑既有 fallback 测试确认未回归**

Run: `cd src-tauri && cargo test --test fallback`
Expected: 5 passed（`translate_with_fallback` 公开签名未变，行为委托给 `_ref`）。

- [ ] **Step 8: clippy + 提交**

```bash
cd src-tauri && cargo clippy --all-targets --features xproc-test-helper -- -D warnings
git add src-tauri/Cargo.toml src-tauri/src/error.rs src-tauri/src/service.rs src-tauri/tests/translate_parallel.rs
git commit -m "feat(r2a): add translate_parallel orchestrator + Clone Error + fallback_ref split"
```

---

## Task 4: 新 IPC 命令 `translate_session`

**Files:**
- Modify: `src-tauri/src/lib.rs`
- Test: `src-tauri/tests/translate_session.rs`（Create）

**Interfaces:**
- Consumes:
  - `crate::db::providers::{read_active_selection, list, ProviderProfile, ProviderStatus}`（Task 2 + 既有）
  - `crate::service::{translate_parallel, TranslationOutcome, translate_with_fallback}`（Task 3）
  - `crate::adapter::profile_to_preset`（Task 1，单引擎路径用）
  - `crate::AppState`、`require_ready_gated`、`Session`
  - `crate::engines::find`（fallback 解析）
  - `crate::settings::load`（读 fallback_engine 设置）
  - `crate::wire::AppOptions`
- Produces:
  - `pub struct TranslateSessionRequest { pub text: String, pub from: String, pub to: String }`
  - `pub struct TranslateSessionResult { pub outcomes: Vec<TranslationOutcome>, pub actual_engine: Option<String> }`
    - `actual_engine`：单引擎成功时 = 该引擎的 `Translation.engine`（preset.id=secret_ref）；并行或全部失败时 = `None`
  - `async fn translate_session(app: tauri::AppHandle, state: State<Arc<Session>>, app_state: State<Arc<AppState>>, req: TranslateSessionRequest) -> Result<TranslateSessionResult, String>`（`#[tauri::command]`）
  - 内部纯逻辑函数 `fn run_translate_session(db: &Arc<Database>, client: &reqwest::Client, keystore: &Keystore, app: &tauri::AppHandle, text, from, to) -> Result<TranslateSessionResult, String>`（被命令与 translate_clipboard 共用，可测）

**流程（命令）：**
1. `session_client(&state)?` + `session_keystore(&state)?` 解析 client/keystore。
2. `spawn_blocking` 内 `data_gate.read()` → `require_ready_gated` → `db`。在 blocking 内只读 `read_active_selection` + `list`，把 `ActiveSelection` + `Vec<ProviderProfile>` clone 出来（离开 blocking 线程）。
3. 回到 async：从 settings 读 `fallback_engine`，`engines::find` 解析传统引擎 → `Option<Arc<dyn TraditionalEngine>>`（包 Arc 以共享）。
4. 过滤 active+enabled 的 profile：取 `selection.primary` + `selection.parallel` 里的 uuid，在 `list` 结果里找 enabled+active 的 profile。顺序：primary 先，parallel 按 selection.parallel 顺序。
5. 若过滤后 profiles 为空 → `Err("no active provider selected")`。
6. 若 `selection.parallel` 为空（单引擎）→ 用 primary 的 profile 经 `profile_to_preset` + `translate_with_fallback`（带 fallback）跑一次；`actual_engine = Some(out.engine)`。
7. 否则（并行）→ `translate_parallel(...)`；`actual_engine = None`（前端按 outcomes 自行处理多结果）。
8. 返回 `TranslateSessionResult { outcomes, actual_engine }`。

**fallback 解析注意：** 既有 `engines::find(&str) -> Option<Box<dyn TraditionalEngine>>`。并行场景需要 `Arc`。包一层：`find(...).map(|b| Arc::<dyn TraditionalEngine>::from(b))`（`Box<dyn T>` → `Arc<dyn T>` via `From`）。

- [ ] **Step 1: 写失败测试 `tests/translate_session.rs`**

Create `src-tauri/tests/translate_session.rs`:

```rust
//! R2a Task 4: translate_session 核心（run_translate_session）集成测试。
//!
//! 直接测纯逻辑函数 `run_translate_session(db, client, keystore, app, ...)`，
//! 跳过 Tauri State/AppHandle 装配（那部分由类型签名保证）。
//! 复用 provider_crud 的 fresh_db 模式 + translate_parallel 的 wiremock 模式。

use linguaray_lib::db::providers as db_providers;
use linguaray_lib::db::schema;
use linguaray_lib::db::Database;
use linguaray_lib::run_translate_session;
use serde_json::json;
use tempfile::tempdir;
use wiremock::{Mock, MockServer, ResponseTemplate};

fn fresh_db() -> (tempfile::TempDir, Database) {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("test.db");
    let db = Database::open(&db_path).unwrap();
    db.with_conn(|conn| {
        let tx = conn.transaction()?;
        schema::create_all_tables(&tx)?;
        schema::seed_singletons(&tx)?;
        tx.commit()?;
        Ok(())
    })
    .unwrap();
    (dir, db)
}

fn direct_client() -> reqwest::Client {
    reqwest::Client::builder().no_proxy().build().unwrap()
}

fn empty_keystore() -> linguaray_lib::keystore::Keystore {
    let dir = tempfile::tempdir().unwrap().keep();
    linguaray_lib::keystore::Keystore::new(dir).unwrap()
}

async fn mount_ok(server: &MockServer, body: &str) {
    Mock::given(wiremock::matchers::method("POST"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "choices": [{"message": {"content": body}}]
        })))
        .mount(server)
        .await;
}

/// 最小 AppHandle 替身：run_translate_session 只用它做 settings::load，
/// 而 settings 在测试环境下会回退到默认值（fallback_engine=None）。
/// 这里传一个未初始化 plugin-store 的 handle 也能让 settings::load 返回默认。
/// 若 settings::load 强依赖 plugin-store，则改用一个不读 settings 的入口
/// （见下方 run_translate_session_with_fallback 测试变体）。
fn test_app_handle() -> tauri::AppHandle {
    // tauri::test 不可用；run_translate_session 接收 &AppHandle 仅用于 settings::load。
    // 我们用一个 helper 入口避开 AppHandle（见 lib.rs 的 pub fn run_translate_session_no_settings）。
    panic!("use run_translate_session_no_settings in tests")
}

#[tokio::test]
async fn single_engine_path_primary_only() {
    // preferences 只设 primary（parallel 为空）→ 单引擎路径，actual_engine=Some。
    let (_dir, db) = fresh_db();
    let server = MockServer::start().await;
    let port: u16 = server.uri().rsplit(':').next().unwrap().parse().unwrap();
    mount_ok(&server, "单引擎结果").await;

    // 建一个 needs_key=false 的 openai profile 指向 mock，设为 primary。
    let p = db
        .with_conn(|conn| {
            let p = db_providers::create(
                conn,
                "openai",
                "OpenAI",
                &format!("http://lvh.me:{port}/v1/chat/completions"),
                None,
            )?;
            // needs_key=false 才能 translate 跳过 keystore；但 create 默认 needs_key=true。
            // 直接改库把 needs_key 关掉（与 fallback.rs 测试哲学一致：聚焦路径）。
            conn.execute(
                "UPDATE providers SET needs_key=0 WHERE uuid=?1",
                rusqlite::params![p.uuid],
            )?;
            // 设为 primary。
            conn.execute(
                "UPDATE preferences SET primary_uuid=?1 WHERE id=1",
                rusqlite::params![p.uuid],
            )?;
            Ok(p)
        })
        .unwrap();

    let client = direct_client();
    let keystore = empty_keystore();
    let db_arc = std::sync::Arc::new(db);

    let result = linguaray_lib::run_translate_session_no_settings(
        &db_arc, &client, &keystore, "hello", "auto", "zh",
    )
    .await
    .expect("single engine ok");

    assert_eq!(result.outcomes.len(), 1);
    let o = &result.outcomes[0];
    assert_eq!(o.uuid, p.uuid);
    let t = o.result.as_ref().expect("ok");
    assert_eq!(t.text, "单引擎结果");
    assert_eq!(result.actual_engine.as_deref(), Some(t.engine.as_str()));
}

#[tokio::test]
async fn parallel_path_two_engines() {
    // primary + 1 parallel → 并行路径，actual_engine=None。
    let (_dir, db) = fresh_db();
    let s1 = MockServer::start().await;
    let s2 = MockServer::start().await;
    let port1: u16 = s1.uri().rsplit(':').next().unwrap().parse().unwrap();
    let port2: u16 = s2.uri().rsplit(':').next().unwrap().parse().unwrap();
    mount_ok(&s1, "结果1").await;
    mount_ok(&s2, "结果2").await;

    let (uuid1, uuid2) = db
        .with_conn(|conn| {
            let p1 = db_providers::create(
                conn, "openai", "A",
                &format!("http://lvh.me:{port1}/v1/chat/completions"), None,
            )?;
            let p2 = db_providers::create(
                conn, "gemini", "B",
                &format!("http://lvh.me:{port2}/v1/chat/completions"), None,
            )?;
            // 都关 needs_key。
            conn.execute(
                "UPDATE providers SET needs_key=0 WHERE uuid IN (?1, ?2)",
                rusqlite::params![p1.uuid, p2.uuid],
            )?;
            // primary=p1, parallel=[p2]。
            conn.execute(
                "UPDATE preferences SET primary_uuid=?1, parallel_uuids=?2 WHERE id=1",
                rusqlite::params![p1.uuid, serde_json::to_string(&[&p2.uuid]).unwrap()],
            )?;
            Ok((p1.uuid, p2.uuid))
        })
        .unwrap();

    let client = direct_client();
    let keystore = empty_keystore();
    let db_arc = std::sync::Arc::new(db);

    let result = linguaray_lib::run_translate_session_no_settings(
        &db_arc, &client, &keystore, "hi", "auto", "zh",
    )
    .await
    .expect("parallel ok");

    assert_eq!(result.outcomes.len(), 2);
    assert!(result.actual_engine.is_none(), "parallel → actual_engine None");
    let mut by_uuid = std::collections::HashMap::new();
    for o in result.outcomes {
        by_uuid.insert(o.uuid, o.result.unwrap().text);
    }
    assert_eq!(by_uuid.get(&uuid1).map(String::as_str), Some("结果1"));
    assert_eq!(by_uuid.get(&uuid2).map(String::as_str), Some("结果2"));
}

#[tokio::test]
async fn no_active_selection_errors() {
    let (_dir, db) = fresh_db();
    let client = direct_client();
    let keystore = empty_keystore();
    let db_arc = std::sync::Arc::new(db);
    let err = linguaray_lib::run_translate_session_no_settings(
        &db_arc, &client, &keystore, "x", "auto", "zh",
    )
    .await
    .unwrap_err();
    assert!(err.contains("no active provider"), "got: {err}");
}

#[tokio::test]
async fn disabled_primary_is_skipped_even_if_selected() {
    // primary 被选中但 enabled=false：过滤后无可用 profile → 错误
    // （validate_active_selection 在 set_active 时拦截，但运行时也兜底）。
    let (_dir, db) = fresh_db();
    let server = MockServer::start().await;
    let port: u16 = server.uri().rsplit(':').next().unwrap().parse().unwrap();
    mount_ok(&server, "x").await;

    let p = db
        .with_conn(|conn| {
            let p = db_providers::create(
                conn, "openai", "A",
                &format!("http://lvh.me:{port}/v1/chat/completions"), None,
            )?;
            conn.execute("UPDATE providers SET needs_key=0, enabled=0 WHERE uuid=?1", rusqlite::params![p.uuid])?;
            conn.execute("UPDATE preferences SET primary_uuid=?1 WHERE id=1", rusqlite::params![p.uuid])?;
            Ok(p.uuid)
        })
        .unwrap();
    let _ = p;

    let client = direct_client();
    let keystore = empty_keystore();
    let db_arc = std::sync::Arc::new(db);
    let err = linguaray_lib::run_translate_session_no_settings(
        &db_arc, &client, &keystore, "x", "auto", "zh",
    )
    .await
    .unwrap_err();
    assert!(err.contains("no active provider"), "disabled primary must be filtered out: {err}");
}
```

- [ ] **Step 2: 跑测试确认失败（RED）**

Run: `cd src-tauri && cargo test --test translate_session`
Expected: 编译失败（`unresolved function run_translate_session_no_settings`）。

- [ ] **Step 3: 实现 `run_translate_session` + `_no_settings` + 命令（GREEN）**

Modify `src-tauri/src/lib.rs`：

**(a) 顶部 imports 加几行（找现有 use 段，在 `use crate::db::providers::{self as db_providers, ProviderPatch, ProviderProfile};` 这行附近补）：**

```rust
use crate::db::providers::{ActiveSelection, ProviderStatus};
use crate::service::{translate_parallel, translate_with_fallback, TranslationOutcome};
use std::sync::Arc;
use crate::adapter::profile_to_preset;
```

（`Arc` 可能已在 lib.rs 导入；若重复则只保留一处。`ProviderProfile` 已导入，无需重复。）

**(b) 在 `translate_clipboard` 命令之后（line ~384 附近）插入新类型 + 核心函数：**

```rust
// ─── R2a: translate_session ──────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize)]
pub struct TranslateSessionRequest {
    pub text: String,
    pub from: String,
    pub to: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct TranslateSessionResult {
    /// 每个引擎的结果（成功或分类过的错误）。单引擎路径长度=1，并行=primary+parallel 数。
    pub outcomes: Vec<TranslationOutcome>,
    /// 单引擎成功时的实际 engine id（preset.id=secret_ref）；并行或全失败时 None。
    /// 老前端可只读这个字段保持单结果 UI 工作；新前端读 outcomes 渲染多结果。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub actual_engine: Option<String>,
}

/// 翻译会话核心逻辑（纯函数，无 Tauri State 依赖）。
///
/// 被两个入口共享：
/// - `translate_session` 命令（带 AppHandle，从 settings 读 fallback_engine）
/// - `translate_clipboard`（同上）
/// - 测试用 `run_translate_session_no_settings`（fallback=None，避开 settings）
///
/// 流程见 plan Task 4：read_active_selection → list → 过滤 active+enabled →
/// 单引擎 or translate_parallel。
async fn run_translate_session(
    db: &Arc<Database>,
    client: &reqwest::Client,
    keystore: &keystore::Keystore,
    app: &tauri::AppHandle,
    text: &str,
    from: &str,
    to: &str,
) -> Result<TranslateSessionResult, String> {
    // 读 fallback_engine（§G opt-in，默认 None）。
    let fallback_box = settings::load(app).fallback_engine.as_deref().and_then(engines::find);
    let fallback: Option<Arc<dyn engines::TraditionalEngine>> =
        fallback_box.map(|b| Arc::<dyn engines::TraditionalEngine>::from(b));
    run_translate_session_with_fallback(db, client, keystore, text, from, to, fallback).await
}

/// 测试入口：不读 settings，fallback 直接传 None（聚焦核心路径）。
pub async fn run_translate_session_no_settings(
    db: &Arc<Database>,
    client: &reqwest::Client,
    keystore: &keystore::Keystore,
    text: &str,
    from: &str,
    to: &str,
) -> Result<TranslateSessionResult, String> {
    run_translate_session_with_fallback(db, client, keystore, text, from, to, None).await
}

async fn run_translate_session_with_fallback(
    db: &Arc<Database>,
    client: &reqwest::Client,
    keystore: &keystore::Keystore,
    text: &str,
    from: &str,
    to: &str,
    fallback: Option<Arc<dyn engines::TraditionalEngine>>,
) -> Result<TranslateSessionResult, String> {
    // 读 active selection + 全量 list（一个 blocking 块内，gate 保护）。
    let (selection, all_profiles) = {
        // 注意：调用方（命令）已持有 data_gate + require_ready_gated；这里为了
        // 让纯函数可测，直接用 db（测试里 db 是健康的）。命令路径会在外层先 gate。
        let sel = db
            .with_conn(|conn| db_providers::read_active_selection(conn))
            .map_err(|e| e.to_string())?;
        let list = db
            .with_conn(|conn| db_providers::list(conn))
            .map_err(|e| e.to_string())?;
        (sel, list)
    };

    // 过滤出 active+enabled 的 profile，按 selection 顺序（primary 先，parallel 次）。
    // 与 validate_active_selection 的 active+enabled 判定一致。
    let is_callable = |p: &ProviderProfile| {
        p.status == ProviderStatus::Active.as_str() && p.enabled
    };
    let mut profiles: Vec<ProviderProfile> = Vec::new();
    if let Some(primary_uuid) = &selection.primary {
        if let Some(p) = all_profiles.iter().find(|p| &p.uuid == primary_uuid) {
            if is_callable(p) {
                profiles.push(p.clone());
            }
        }
    }
    for uuid in &selection.parallel {
        if let Some(p) = all_profiles.iter().find(|p| &p.uuid == uuid) {
            if is_callable(p) && !profiles.iter().any(|q| q.uuid == p.uuid) {
                profiles.push(p.clone());
            }
        }
    }
    if profiles.is_empty() {
        return Err("no active provider selected".into());
    }

    // 单引擎 vs 并行。
    if selection.parallel.is_empty() {
        // 单引擎：用 primary profile + translate_with_fallback。
        let preset = profile_to_preset(&profiles[0])
            .map_err(|e| format!("adapter error: {e}"))?;
        let input = service::TranslateInput {
            text,
            from,
            to,
            options: wire::AppOptions::default(),
        };
        let fb_ref: Option<&dyn engines::TraditionalEngine> = fallback.as_deref();
        let result = service::translate_with_fallback_ref(
            client, keystore, &preset, input, fb_ref,
        )
        .await;
        let actual_engine = match &result {
            Ok(t) => Some(t.engine.clone()),
            Err(_) => None,
        };
        Ok(TranslateSessionResult {
            outcomes: vec![TranslationOutcome {
                uuid: profiles[0].uuid.clone(),
                result,
            }],
            actual_engine,
        })
    } else {
        // 并行。
        let outcomes = translate_parallel(
            client,
            keystore,
            profiles,
            text,
            from,
            to,
            wire::AppOptions::default(),
            fallback,
        )
        .await;
        Ok(TranslateSessionResult {
            outcomes,
            actual_engine: None,
        })
    }
}

/// 并行/单引擎翻译命令（R2a）。前端用 `invoke('translate_session', { req })` 调用。
///
/// 从 AppState 读 active selection + providers，从 Session 读 client/keystore，
/// 从 settings 读 fallback_engine。parallel 为空时退化为单引擎（actual_engine=Some）。
#[tauri::command]
async fn translate_session(
    app: tauri::AppHandle,
    state: tauri::State<'_, Arc<Session>>,
    app_state: tauri::State<'_, Arc<AppState>>,
    req: TranslateSessionRequest,
) -> Result<TranslateSessionResult, String> {
    let client = session_client(&state)?.clone();
    let keystore = session_keystore(&state)?;
    let app_arc = app_state.inner().clone();
    // 在 blocking 内 gate + 读 DB 快照（gate 必须在 clone Arc 之前，见 provider_list 注释）。
    // 这里我们让 run_translate_session 自己用 db.with_conn 读；但 gate 要由命令持有。
    // 所以：spawn_blocking 里 gate + require_ready_gated 拿到 db Arc，直接交给核心。
    let db = tauri::async_runtime::spawn_blocking(move || -> Result<Arc<Database>, String> {
        let _gate = app_arc.data_gate.read();
        let db = require_ready_gated(&app_arc, &_gate)?;
        Ok(db)
    })
    .await
    .map_err(|e| e.to_string())??;
    run_translate_session(&db, &client, keystore, &app, &req.text, &req.from, &req.to).await
}
```

注意：`session_client` 返回 `&reqwest::Client`，但 `run_translate_session` 在 `spawn_blocking` 之后跨 await 点用，所以这里 `.clone()` 一个 owned `reqwest::Client`（reqwest::Client 是 Arc 内部，clone 廉价）。`session_keystore` 返回 `&Keystore`，引用只在 `run_translate_session` 的 await 期间存活——`state` 由 Tauri 持有，生命周期覆盖整个命令，安全。

**(c) 在 `invoke_handler` 注册命令（line ~2179 附近，`generate_handler![...]` 内加 `translate_session,`）：**

在 `translate_clipboard,` 这行之后加：

```rust
            translate_clipboard,
            translate_session,
```

- [ ] **Step 4: 跑测试确认通过（GREEN）**

Run: `cd src-tauri && cargo test --test translate_session`
Expected: 4 passed.

- [ ] **Step 5: clippy + 提交**

```bash
cd src-tauri && cargo clippy --all-targets --features xproc-test-helper -- -D warnings
git add src-tauri/src/lib.rs src-tauri/tests/translate_session.rs
git commit -m "feat(r2a): add translate_session IPC command (single + parallel paths)"
```

---

## Task 5: popup 事件重构（`popup-multi-result`）

**Files:**
- Modify: `src-tauri/src/popup.rs`
- Test: `src-tauri/src/popup.rs`（模块内 `#[cfg(test)] mod tests`）

**Interfaces:**
- Consumes:
  - `crate::service::TranslationOutcome`（Task 3，含 `uuid` + `Result<Translation, Error>`）
  - `crate::error::Error`（Task 3 已 Clone）
- Produces:
  - `pub struct TranslationOutcomeSerialized { pub uuid: String, pub ok: bool, pub text: Option<String>, pub engine: Option<String>, pub error: Option<String> }`
  - `impl From<&TranslationOutcome> for TranslationOutcomeSerialized`
  - `pub fn multi_result(app: &tauri::AppHandle, outcomes: &[TranslationOutcome]) -> Result<(), String>` — emit 事件 `"popup-multi-result"`，payload = `{ "outcomes": [...] }`
  - 既有 `popup-state` 事件**完全不变**（向后兼容老前端）

**设计决策（load-bearing）：** 选独立事件 `popup-multi-result` 而非扩 `Payload`，因为：
1. 既有 `Payload<'a>` 用 `&'a str` 借用，多结果数据是 owned `Vec`，混在一个 struct 里生命周期不可调和。
2. 独立事件让老前端（只听 `popup-state`）零改动；新前端按需监听 `popup-multi-result`。
3. 符合 Surface 03（多结果页）是独立 surface 的事实。

序列化 outcome 时把 `Result` 拆成 `ok` + `text`/`error` 扁平字段，前端不用 match Result（serde 对 Result 的序列化是 `{"Ok":...}/{"Err":...}` 嵌套，前端处理麻烦）。

- [ ] **Step 1: 写失败测试（模块内）**

在 `src-tauri/src/popup.rs` 文件末尾追加：

```rust

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::{ConfigKind, Error, FallbackKind};
    use crate::service::{Translation, TranslationOutcome};

    fn ok_outcome(uuid: &str, text: &str, engine: &str) -> TranslationOutcome {
        TranslationOutcome {
            uuid: uuid.into(),
            result: Ok(Translation { text: text.into(), engine: engine.into() }),
        }
    }

    fn err_outcome(uuid: &str, err: Error) -> TranslationOutcome {
        TranslationOutcome { uuid: uuid.into(), result: Err(err) }
    }

    #[test]
    fn serialize_ok_outcome() {
        let o = ok_outcome("u1", "你好", "provider/u1");
        let s = TranslationOutcomeSerialized::from(&o);
        assert!(s.ok);
        assert_eq!(s.text.as_deref(), Some("你好"));
        assert_eq!(s.engine.as_deref(), Some("provider/u1"));
        assert!(s.error.is_none());
        assert_eq!(s.uuid, "u1");
    }

    #[test]
    fn serialize_err_outcome_carries_message() {
        let o = err_outcome("u2", Error::LocalNoFallback);
        let s = TranslationOutcomeSerialized::from(&o);
        assert!(!s.ok);
        assert!(s.text.is_none());
        assert!(s.engine.is_none());
        let err = s.error.expect("error message present");
        assert!(err.contains("no fallback"), "got: {err}");
        assert_eq!(s.uuid, "u2");
    }

    #[test]
    fn serialize_config_error_outcome() {
        let o = err_outcome(
            "u3",
            Error::Config(ConfigKind::AuthFailed { provider: "p".into(), status: 401 }),
        );
        let s = TranslationOutcomeSerialized::from(&o);
        assert!(!s.ok);
        assert!(s.error.as_deref().unwrap().contains("401"));
    }

    #[test]
    fn serialize_fallback_eligible_outcome() {
        let o = err_outcome(
            "u4",
            Error::FallbackEligible(FallbackKind::ProviderStatus { status: 500 }),
        );
        let s = TranslationOutcomeSerialized::from(&o);
        assert!(!s.ok);
        assert!(s.error.as_deref().unwrap().contains("500"));
    }

    #[test]
    fn multi_result_payload_shape_is_outcomes_array() {
        // 序列化 shape 校验（不发真实事件）：payload 必须是 { "outcomes": [...] }。
        let outcomes = vec![
            ok_outcome("u1", "a", "provider/u1"),
            err_outcome("u2", Error::LocalNoFallback),
        ];
        let payload = PopupMultiPayload {
            outcomes: outcomes.iter().map(TranslationOutcomeSerialized::from).collect(),
        };
        let json = serde_json::to_string(&payload).unwrap();
        assert!(json.contains("\"outcomes\""), "{json}");
        assert!(json.contains("\"u1\""), "{json}");
        assert!(json.contains("\"u2\""), "{json}");
        // ok outcome 带 text，err outcome 带 error。
        assert!(json.contains("\"text\":\"a\""), "{json}");
        assert!(json.contains("\"error\""), "{json}");
    }

    #[test]
    fn multi_result_emits_named_event() {
        // 直接验证事件名常量，避免依赖 Tauri runtime。
        assert_eq!(POPUP_MULTI_EVENT, "popup-multi-result");
    }
}
```

- [ ] **Step 2: 跑测试确认失败（RED）**

Run: `cd src-tauri && cargo test --lib popup`
Expected: 编译失败（`unresolved TranslationOutcomeSerialized / PopupMultiPayload / POPUP_MULTI_EVENT`）。

- [ ] **Step 3: 实现多结果事件（GREEN）**

Modify `src-tauri/src/popup.rs`。在文件顶部 imports 之后、`const POPUP` 附近加事件名常量与序列化类型；在文件底部（tests 之前）加 `multi_result` 函数。

把整个文件顶部的常量区改为：

```rust
//! Show/move/hide the frameless popup window; push a payload (loading / result).
use tauri::{Emitter, Manager, WebviewWindow};

const POPUP: &str = "popup";
/// R2a: 多结果事件名（独立于 popup-state，向后兼容老前端）。
const POPUP_MULTI_EVENT: &str = "popup-multi-result";
```

在 `Payload` struct 定义之后（line 38 之后）追加：

```rust

// ─── R2a: 多结果事件 ──────────────────────────────────────────────────────

/// 单个引擎结果的序列化形态（前端友好：Result 拆成 ok + text/error 扁平字段）。
#[derive(Clone, serde::Serialize)]
pub struct TranslationOutcomeSerialized {
    pub uuid: String,
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub engine: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl From<&crate::service::TranslationOutcome> for TranslationOutcomeSerialized {
    fn from(o: &crate::service::TranslationOutcome) -> Self {
        match &o.result {
            Ok(t) => Self {
                uuid: o.uuid.clone(),
                ok: true,
                text: Some(t.text.clone()),
                engine: Some(t.engine.clone()),
                error: None,
            },
            Err(e) => Self {
                uuid: o.uuid.clone(),
                ok: false,
                text: None,
                engine: None,
                error: Some(e.to_string()),
            },
        }
    }
}

#[derive(Clone, serde::Serialize)]
struct PopupMultiPayload {
    outcomes: Vec<TranslationOutcomeSerialized>,
}

/// 推送多引擎翻译结果（R2a）。emit `popup-multi-result` 事件，
/// payload = `{ "outcomes": [ { uuid, ok, text?, engine?, error? }, ... ] }`。
/// 老前端只听 `popup-state`，不受影响；新前端监听本事件渲染 Surface 03。
pub fn multi_result(
    app: &tauri::AppHandle,
    outcomes: &[crate::service::TranslationOutcome],
) -> Result<(), String> {
    let win = window(app)?;
    let payload = PopupMultiPayload {
        outcomes: outcomes.iter().map(TranslationOutcomeSerialized::from).collect(),
    };
    win.emit(POPUP_MULTI_EVENT, payload).map_err(|e| e.to_string())?;
    Ok(())
}
```

- [ ] **Step 4: 跑测试确认通过（GREEN）**

Run: `cd src-tauri && cargo test --lib popup`
Expected: 6 passed.

- [ ] **Step 5: clippy + 提交**

```bash
cd src-tauri && cargo clippy --all-targets --features xproc-test-helper -- -D warnings
git add src-tauri/src/popup.rs
git commit -m "feat(r2a): add popup-multi-result event for parallel translation outcomes"
```

---

## Task 6: `translate_clipboard` 接入 `translate_session`

**Files:**
- Modify: `src-tauri/src/lib.rs`（改 `translate_clipboard` 命令）

**Interfaces:**
- Consumes:
  - `run_translate_session`（Task 4）
  - `popup::{show_at, result, error, multi_result}`（Task 5）
  - `concurrency::GenerationToken`（既有）
  - `AppState`、`require_ready_gated`（既有）
  - `Session`（既有）
- Produces:
  - 改造后的 `translate_clipboard(app, state: Arc<Session>, app_state: Arc<AppState>) -> Result<(), String>`

**流程（改造后）：**
1. `gen.next()` 分配令牌（不变）。
2. selection_lock 下读 clipboard（不变）；空 → `Err("clipboard empty")`（不变）。
3. `cursor::position()` + `popup::show_at(loading)`（不变）。
4. `session_client` + `session_keystore`（不变；失败 → gen 检查后 error popup）。
5. **新**：`spawn_blocking` 内 `app_state.data_gate.read()` + `require_ready_gated` 拿 db Arc。
6. **新**：`run_translate_session(&db, client, keystore, &app, &text, "auto", &settings.target_language).await`。
   - 注意：target_language 现从 settings 读（既有逻辑），from 固定 "auto"。
7. **新**：结果处理：
   - Ok：若 `actual_engine.is_some()`（单引擎成功）→ `popup::result(text, engine)`（老事件，向后兼容）。
   - Ok：若 `actual_engine.is_none()`（并行）→ `popup::multi_result(outcomes)`（新事件）。
   - 单引擎但失败（actual_engine=None 且 outcomes.len()==1 且 outcome 失败）→ `popup::error(outcome.error)`。
   - 并行且**全部**失败 → `popup::error("all engines failed")`（简化；前端可后续细化按 outcome 渲染）。
   - Err（核心返回错误，如 no active provider）→ `popup::error(msg)`。
8. 所有 popup 发送前都要 `gen.is_latest` 检查（latest-wins 不变）。

**latest-wins 关键不变量：** `gen.next()` 必须在 spawn_blocking 之前（既有代码已如此）；`run_translate_session` 完成后必须 `is_latest` 检查才能发事件，否则旧请求会覆盖新 popup。

- [ ] **Step 1: 写失败测试**

`translate_clipboard` 深度依赖 Tauri runtime（AppHandle、webview window），单元测试不现实。改为：**测它共享的核心逻辑 `run_translate_session` 已被 Task 4 覆盖**，本任务只新增一个「集成契约」测试——验证 `translate_clipboard` 在「无 active selection」时不 panic 且走 error 路径。

但更实际的是：本任务的「测试」是 **Task 4 的 `run_translate_session_no_settings` 测试已覆盖核心**。本任务额外加一个 lib.rs 模块内测试，验证「单引擎结果 → 触发老事件、并行结果 → 触发新事件」的**分支判定函数**（抽出纯函数避免 Tauri 依赖）。

先抽出纯函数 `decide_clipboard_popup(result) -> ClipboardPopupDecision`，再测它。

在 `src-tauri/src/lib.rs` 的 `#[cfg(test)] mod tests` 内（文件末尾既有 tests mod 里）追加：

```rust

    // ─── R2a Task 6: translate_clipboard 分支决策 ──────────────────────────────

    #[test]
    fn clipboard_decision_single_success_uses_legacy_event() {
        let result = TranslateSessionResult {
            outcomes: vec![TranslationOutcome {
                uuid: "u1".into(),
                result: Ok(service::Translation { text: "你好".into(), engine: "provider/u1".into() }),
            }],
            actual_engine: Some("provider/u1".into()),
        };
        let d = decide_clipboard_popup(&result);
        assert!(matches!(d, ClipboardPopupDecision::SingleSuccess { .. }));
        if let ClipboardPopupDecision::SingleSuccess { text, engine } = d {
            assert_eq!(text, "你好");
            assert_eq!(engine, "provider/u1");
        }
    }

    #[test]
    fn clipboard_decision_parallel_uses_multi_event() {
        let result = TranslateSessionResult {
            outcomes: vec![
                TranslationOutcome {
                    uuid: "u1".into(),
                    result: Ok(service::Translation { text: "a".into(), engine: "p/u1".into() }),
                },
                TranslationOutcome {
                    uuid: "u2".into(),
                    result: Err(crate::error::Error::LocalNoFallback),
                },
            ],
            actual_engine: None,
        };
        let d = decide_clipboard_popup(&result);
        assert!(matches!(d, ClipboardPopupDecision::Multi { .. }));
    }

    #[test]
    fn clipboard_decision_single_failure_is_error() {
        let result = TranslateSessionResult {
            outcomes: vec![TranslationOutcome {
                uuid: "u1".into(),
                result: Err(crate::error::Error::LocalNoFallback),
            }],
            actual_engine: None,
        };
        let d = decide_clipboard_popup(&result);
        match d {
            ClipboardPopupDecision::Error(msg) => assert!(msg.contains("no fallback"), "{msg}"),
            other => panic!("expected Error, got {other:?}"),
        }
    }

    #[test]
    fn clipboard_decision_all_parallel_failed_is_error() {
        let result = TranslateSessionResult {
            outcomes: vec![
                TranslationOutcome { uuid: "u1".into(), result: Err(crate::error::Error::LocalNoFallback) },
                TranslationOutcome { uuid: "u2".into(), result: Err(crate::error::Error::LocalNoFallback) },
            ],
            actual_engine: None,
        };
        let d = decide_clipboard_popup(&result);
        assert!(matches!(d, ClipboardPopupDecision::Error(_)));
    }
```

- [ ] **Step 2: 跑测试确认失败（RED）**

Run: `cd src-tauri && cargo test --lib clipboard_decision`
Expected: 编译失败（`unresolved decide_clipboard_popup / ClipboardPopupDecision`）。

- [ ] **Step 3: 实现 `ClipboardPopupDecision` + `decide_clipboard_popup` + 改造 `translate_clipboard`（GREEN）**

Modify `src-tauri/src/lib.rs`：

**(a) 在 `TranslateSessionResult` 定义之后插入决策类型 + 函数：**

```rust
/// translate_clipboard 根据翻译结果决定发哪种 popup 事件的纯函数决策。
/// 抽出来便于测试（translate_clipboard 本身依赖 Tauri runtime 不可单测）。
#[derive(Debug)]
enum ClipboardPopupDecision {
    /// 单引擎成功 → 走老 popup-state 事件（向后兼容）。
    SingleSuccess { text: String, engine: String },
    /// 并行（含部分成功）→ 走 popup-multi-result 事件。
    Multi,
    /// 单引擎失败 / 并行全失败 / 核心错误 → 走 popup-state error。
    Error(String),
}

fn decide_clipboard_popup(result: &TranslateSessionResult) -> ClipboardPopupDecision {
    if result.outcomes.is_empty() {
        return ClipboardPopupDecision::Error("translation produced no outcomes".into());
    }
    // 单引擎路径：actual_engine=Some 表示成功。
    if let Some(engine) = &result.actual_engine {
        // 长度必为 1（run_translate_session 单引擎路径契约）。
        if let Some(o) = result.outcomes.first() {
            if let Ok(t) = &o.result {
                return ClipboardPopupDecision::SingleSuccess {
                    text: t.text.clone(),
                    engine: engine.clone(),
                };
            }
        }
        // actual_engine=Some 但 outcome 失败（理论不应发生）→ 当错误处理。
        return ClipboardPopupDecision::Error("single engine failed unexpectedly".into());
    }
    // actual_engine=None：并行路径。
    if result.outcomes.len() == 1 {
        // 退化单引擎但失败。
        if let Some(err) = result.outcomes.first().and_then(|o| o.result.as_ref().err()) {
            return ClipboardPopupDecision::Error(err.to_string());
        }
    }
    // 并行全失败？
    let all_failed = result.outcomes.iter().all(|o| o.result.is_err());
    if all_failed {
        return ClipboardPopupDecision::Error("all engines failed".into());
    }
    // 并行（含部分成功）。
    ClipboardPopupDecision::Multi
}
```

**(b) 改造 `translate_clipboard` 命令（替换 line 322-384 整个函数体）：**

```rust
#[tauri::command]
async fn translate_clipboard(
    app: tauri::AppHandle,
    state: tauri::State<'_, Arc<Session>>,
    app_state: tauri::State<'_, Arc<AppState>>,
) -> Result<(), String> {
    // latest-wins：先分配 gen（同步，保证 press 顺序）。
    let gen = state.gen.next();
    let text = {
        let _g = state.gen.selection_lock();
        clipboard::get_text()?
    };
    if text.trim().is_empty() {
        return Err("clipboard empty".into());
    }
    let (x, y) = cursor::position();
    let _ = popup::show_at(&app, x, y);
    let s = settings::load(&app);

    let client = match session_client(&state) {
        Ok(c) => c.clone(),
        Err(msg) => {
            if state.gen.is_latest(gen) {
                let _ = popup::error(&app, &msg);
            }
            return Ok(());
        }
    };
    let keystore = match session_keystore(&state) {
        Ok(k) => k,
        Err(msg) => {
            if state.gen.is_latest(gen) {
                let _ = popup::error(&app, &msg);
            }
            return Ok(());
        }
    };

    // gate + require_ready_gated 拿 db（spawn_blocking 内，与 translate_session 一致）。
    let app_arc = app_state.inner().clone();
    let db = match tauri::async_runtime::spawn_blocking(move || -> Result<Arc<Database>, String> {
        let _gate = app_arc.data_gate.read();
        require_ready_gated(&app_arc, &_gate)
    })
    .await
    {
        Ok(Ok(db)) => db,
        Ok(Err(msg)) => {
            if state.gen.is_latest(gen) {
                let _ = popup::error(&app, &msg);
            }
            return Ok(());
        }
        Err(e) => {
            if state.gen.is_latest(gen) {
                let _ = popup::error(&app, &format!("join error: {e}"));
            }
            return Ok(());
        }
    };

    // 走统一核心（从 settings 读 fallback_engine；target_language 来自 settings）。
    let session_result = run_translate_session(
        &db, &client, keystore, &app, &text, "auto", &s.target_language,
    )
    .await;

    // latest-wins：完成后检查 gen 才发事件。
    if !state.gen.is_latest(gen) {
        return Ok(());
    }
    match session_result {
        Ok(r) => match decide_clipboard_popup(&r) {
            ClipboardPopupDecision::SingleSuccess { text, engine } => {
                let _ = popup::result(&app, &text, &engine);
            }
            ClipboardPopupDecision::Multi => {
                let _ = popup::multi_result(&app, &r.outcomes);
            }
            ClipboardPopupDecision::Error(msg) => {
                let _ = popup::error(&app, &msg);
            }
        },
        Err(msg) => {
            let _ = popup::error(&app, &msg);
        }
    }
    Ok(())
}
```

- [ ] **Step 4: 跑测试确认通过（GREEN）**

Run: `cd src-tauri && cargo test --lib clipboard_decision`
Expected: 4 passed.

- [ ] **Step 5: 跑全量测试确认无回归**

Run: `cd src-tauri && cargo test --features xproc-test-helper`
Expected: 全部通过（含既有 fallback/wire/provider_crud/keystore 等）。

- [ ] **Step 6: clippy + 提交**

```bash
cd src-tauri && cargo clippy --all-targets --features xproc-test-helper -- -D warnings
git add src-tauri/src/lib.rs
git commit -m "feat(r2a): route translate_clipboard through translate_session (single+parallel)"
```

---

## 最终验证

完成全部 6 个任务后，跑以下命令确认整体一致：

```bash
cd src-tauri
cargo test --features xproc-test-helper
cargo clippy --all-targets --features xproc-test-helper -- -D warnings
cargo build
```

预期：
- `cargo test`：所有测试通过（既有 + R2a 新增 adapter/active_selection/translate_parallel/translate_session/popup/clipboard_decision）。
- `cargo clippy`：零 warning。
- `cargo build`：成功（新命令 `translate_session` 已注册进 `invoke_handler`；`translate_clipboard` 签名加了 `app_state` 参数——前端调用 `invoke('translate_clipboard')` 无需改参数，Tauri 自动注入 State）。

**前端契约（供 R2b 前端任务参考，不在 R2a 实现）：**
- `invoke('translate_session', { req: { text, from, to } })` → `{ outcomes: [{ uuid, ok, text?, engine?, error? }], actual_engine? }`
- `invoke('translate_clipboard')` 仍无参数；监听 `popup-state`（单结果，向后兼容）或 `popup-multi-result`（多结果，payload `{ outcomes: [...] }`）。

---

## Self-Review 记录

**1. Spec coverage（6 个组件 → 任务映射）：**
- Task 1（Protocol→ApiKind + Profile→Preset 适配器）→ ✅ Task 1
- Task 2（active-selection 读取器）→ ✅ Task 2
- Task 3（并行翻译编排器）→ ✅ Task 3（含 `TranslationOutcome`、`translate_parallel`、独立错误分类复用 §G）
- Task 4（translate_session 命令）→ ✅ Task 4（含单引擎退化、active+enabled 过滤）
- Task 5（popup 事件重构）→ ✅ Task 5（独立 `popup-multi-result` 事件，向后兼容）
- Task 6（translate_clipboard 接入）→ ✅ Task 6（含 latest-wins 不变量、分支决策）

**2. Placeholder 扫描：** 无 TBD/TODO/「类似 Task N」；每个 code step 都有完整可编译代码。

**3. Type consistency：**
- `TranslationOutcome { uuid, result: Result<Translation, Error> }` — Task 3 定义，Task 4/5/6 一致使用。
- `profile_to_preset(&ProviderProfile) -> Result<ProviderPreset, String>` — Task 1 定义，Task 3/4 一致。
- `translate_parallel(client, keystore, profiles: Vec<ProviderProfile>, text, from, to, options, fallback: Option<Arc<dyn TraditionalEngine>>)` — Task 3 定义，Task 4 一致调用。
- `run_translate_session(db, client, keystore, app, text, from, to)` — Task 4 定义，Task 6 一致调用。
- `popup::multi_result(app, &[TranslationOutcome])` — Task 5 定义，Task 6 一致调用。
- `preset.id = profile.secret_ref`（非 uuid）— Task 1 文档化，Task 3 测试断言（`engine == "provider/u1"`）。

---

## Rev-4 Retroactive Status (2026-08-09)

Appended by the R2/R3a contract audit (docs/superpowers/plans/2026-08-09-r2-r3-contract-audit-fixes.md).
Historical RED states are preserved as-written; this table records the actual
shipped state and where gaps are closed. Each "Shipped?" claim was verified
against the current source tree (file/function grep) at append time.

| Original task | Shipped? | Gap closed in (audit task) |
|---|---|---|
| Task 1: Protocol→ApiKind + ProviderProfile→ProviderPreset adapter (`adapter.rs`, `protocol_to_api_kind`) | yes — `src-tauri/src/adapter.rs` defines `protocol_to_api_kind` + `ApiKind::{OpenAIChat,Anthropic}`; `providers.rs` defines `ProviderPreset` presets | — |
| Task 2: `read_active_selection` reader (`db/providers.rs`) | yes — `read_active_selection` called from `lib.rs` (`provider_get_active_selection`, `run_translate_session`, cold-start) | — |
| Task 3: `translate_parallel` orchestrator (`service.rs`) | yes — `pub async fn translate_parallel` in `service.rs:273`; stable input order + bounded fallback | B5 (stable order), B6 (bounded fallback) |
| Task 4: `translate_session` IPC command | yes — `run_translate_session` in `lib.rs` + registered in `invoke_handler`; generation-token staleness guard wired | A2 (gen-token via `capture_and_translate`), D4 Step 4 (capability auth) |
| Task 5: popup event refactor (`popup-multi-result`) | yes — `POPUP_MULTI_EVENT = "popup-multi-result"` in `popup.rs`; frontend decoder `decodePopupMultiResult` in `decode.ts` | B3 (friendly engine labels / no `secret_ref`) |
| Task 6: `translate_clipboard` routes through `translate_session` core | yes — `translate_clipboard` in `lib.rs:337` calls `run_translate_session`; `decide_popup_event` pure helper tested | A4 (tray Active/Error pulse on this path), A2 (gen-token) |
