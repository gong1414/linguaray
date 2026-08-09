//! Orchestrates a translation (spec architecture). §G classified fallback:
//! primary (AI) engine first; on `FallbackEligible` retry ONCE with a configured
//! traditional engine. `Config`/`Keystore` errors propagate; LOCAL primaries are
//! sacred (never silently degrade to a remote fallback).

use crate::adapter::profile_to_preset;
use crate::db::providers::ProviderProfile;
use crate::engines::TraditionalEngine;
use crate::error::{ConfigKind, Error};
use crate::keystore::Keystore;
use crate::providers::ProviderPreset;
use crate::wire::{build_prompt, call, AppOptions, WireParams};
use std::sync::Arc;

pub struct TranslateInput<'a> {
    pub text: &'a str,
    pub from: &'a str,
    pub to: &'a str,
    pub options: AppOptions,
}

/// A translation result + the engine id that ACTUALLY produced it (primary on
/// success; fallback id when the fallback fired). Review P2: callers must not tag
/// the result with the primary preset id when the fallback produced it.
#[derive(Debug, serde::Serialize)]
pub struct Translation {
    pub text: String,
    pub engine: String,
}

pub async fn translate(
    client: &reqwest::Client,
    keystore: &Keystore,
    preset: &ProviderPreset,
    input: TranslateInput<'_>,
) -> Result<Translation, Error> {
    // Spec §A "Plaintext-key claims": keep the key in memory only for the shortest
    // window between keystore-read and HTTP-send, and zeroize it after use.
    // Zeroizing<String> wipes the heap buffer on drop.
    //
    // S2a P0: read via the typed `KeystoreData` accessor so the key is found in
    // the nested `provider_keys` map after the v2 migration (the old raw
    // `keys[preset.id]` lookup hit the flat map and returned None for migrated
    // keystores). For the built-in presets the key name is the preset id; for a
    // migrated DB-backed provider the row's `secret_ref` carries the same name
    // (see db::providers), so `preset.id` is the right `secret_ref` here.
    let key = if preset.needs_key {
        let k = keystore
            .get_key(&preset.id)
            .map_err(Error::Keystore)?
            .ok_or_else(|| Error::Config(ConfigKind::MissingKey { provider: preset.id.clone() }))?;
        zeroize::Zeroizing::new(k)
    } else {
        zeroize::Zeroizing::new(String::new())
    };
    let (system, user) = build_prompt(input.text, input.from, input.to, &input.options);
    let params = WireParams {
        model: preset.default_model.clone(),
        temperature: None, max_tokens: None, stream: false,
    };
    call(client, preset, &key, &params, &system, &user).await
        .map(|text| Translation { text, engine: preset.id.clone() })
}

/// Translate with §G classified fallback.
///
/// - Runs the primary (AI) engine first.
/// - On `FallbackEligible` (network/timeout/429/5xx/parse), retries the WHOLE
///   request once with `fallback` (the resolved fallback engine, or `None`).
///   No chunk mixing — Phase 1 doesn't chunk; the fallback translates the full text.
/// - `Config` (missing-key/401/403/invalid-model) and `Keystore` errors PROPAGATE
///   unchanged — these send the user to Settings, never a silent fallback.
/// - LOCAL-primary sacred (§G): if the primary engine is LOCAL (loopback — Ollama
///   etc.), an `FallbackEligible` failure is NOT degraded to a remote fallback
///   engine. Local failure = error.
///
/// `fallback` is taken as an injected `Box<dyn TraditionalEngine>` (rather than
/// resolved from settings inside this fn) so the §G branches are unit-testable
/// with a fake engine instead of the real Google network call. Callers resolve it
/// via `settings.fallback_engine.as_deref().and_then(engines::find)`.
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
    // Primary attempt — clone the options because `translate` takes AppOptions by
    // value and we may still need `input`'s fields for the fallback attempt below.
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
        Ok(t) => Ok(t), // engine == primary preset id
        Err(Error::FallbackEligible(_)) => {
            // §G: local-primary sacred — never silently degrade a LOCAL AI engine
            // to a REMOTE fallback. Local failure = error.
            if is_local(primary_preset) {
                return Err(Error::LocalNoFallback);
            }
            match fallback {
                // No fallback configured (opt-in default) — surface "no fallback".
                None => Err(Error::LocalNoFallback),
                // Single fallback attempt over the whole text; tag the result with
                // the FALLBACK engine id (not the primary).
                Some(eng) => {
                    let fb_id = eng.id().to_string();
                    eng.translate(client, input.text, input.from, input.to)
                        .await
                        .map(|text| Translation { text, engine: fb_id })
                }
            }
        }
        // Config/Auth/Keystore → propagate, do NOT fall back.
        Err(other) => Err(other),
    }
}

// ─── R2a: 并行翻译编排 ────────────────────────────────────────────────────

/// 单个引擎的翻译结果（成功或分类过的错误）。uuid 来自原 ProviderProfile，
/// 与 `result` 内的 engine 字段（preset.id=secret_ref）相互独立——调用方用 uuid
/// 把结果关联回用户选的那个 provider row。
//
// 未派生 `Clone`：`result` 持有 `Error`，而 `Error::Keystore` 包装 `KeystoreError`
// （含 `std::io::Error`，不可 Clone）。`translate_session` 命令路径只 `Serialize` outcome
// 发给前端，从不 clone 它；`translate_parallel` 测试按值消费 vec（`sorted_by_uuid`）。
// `Serialize`：`Error` 有手写 Serialize impl（Display 字符串），见 error.rs。
#[derive(Debug, serde::Serialize)]
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
/// - 返回顺序严格等于输入顺序（B5）：每个 outcome 携带其输入 index，最终 vec
///   按 index 排序产出——包括 pre-failed 条目也留在原位。`join_all` 的完成顺序
///   不再影响输出顺序；调用方仍可按 `uuid` 关联，但不应再假设顺序无序。
///
/// §G 不变量：每个引擎独立分类错误。`Config`/`Keystore` 错误绝不因另一个引擎
/// 成功而被"覆盖"——它们作为各自 outcome 的 Err 保留，前端按 Surface 03 的
/// "partial success" 渲染。
//
// 8 个参数是计划的既定签名（R2a 测试直接位置调用），收敛成一个 ctx struct
// 会让所有调用点更绕；与 `translate`/`translate_with_fallback` 的扁平签名保持
// 一致更易读。clippy 的 7 参数阈值是经验值，这里故意放宽。
#[allow(clippy::too_many_arguments)]
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
    // Collect (input_index, Option<uuid+preset>, Option<pre-failed outcome>).
    // B5: tag each entry with its input index so the final vec can be rebuilt in
    // STRICT input order. Pre-failed profiles (profile_to_preset rejects) must
    // stay at their input position — NOT float before the ready outcomes, which
    // happened in the old "push pre-failed then append ready" code path.
    type Entry = (usize, Option<(String, ProviderPreset)>, Option<TranslationOutcome>);
    let mut entries: Vec<Entry> = Vec::with_capacity(profiles.len());
    for (idx, p) in profiles.into_iter().enumerate() {
        match profile_to_preset(&p) {
            Ok(preset) => entries.push((idx, Some((p.uuid.clone(), preset)), None)),
            Err(reason) => entries.push((
                idx,
                None,
                Some(TranslationOutcome {
                    uuid: p.uuid.clone(),
                    result: Err(Error::Config(ConfigKind::Unsupported {
                        provider: p.uuid.clone(),
                        reason,
                    })),
                }),
            )),
        }
    }
    // Drive all ready entries concurrently, tagging each result with its index.
    // (B5 only fixes ORDERING. B6 will change the fallback arg to None and add
    // the session-level fallback policy. For now, pass fallback.as_deref() to
    // preserve per-engine behavior until B6 lands.)
    let futs: Vec<_> = entries
        .iter()
        .filter_map(|(idx, ready, _)| ready.as_ref().map(|(uuid, preset)| (*idx, uuid.clone(), preset)))
        .map(|(idx, uuid, preset)| {
            let options = options.clone();
            let fb_ref: Option<&dyn TraditionalEngine> = fallback.as_deref();
            async move {
                let input = TranslateInput { text, from, to, options };
                let result =
                    translate_with_fallback_ref(client, keystore, preset, input, fb_ref).await;
                (idx, TranslationOutcome { uuid, result })
            }
        })
        .collect();
    let mut ready_results = futures::future::join_all(futs).await;
    // Build the final vec in strict input order: walk entries by index.
    ready_results.sort_by_key(|(idx, _)| *idx);
    let mut ready_iter = ready_results.into_iter();
    let mut outcomes: Vec<TranslationOutcome> = Vec::with_capacity(entries.len());
    for (_idx, _ready, pre_failed) in entries {
        if let Some(o) = pre_failed {
            outcomes.push(o);
        } else if let Some((_idx, o)) = ready_iter.next() {
            outcomes.push(o);
        }
    }
    outcomes
}

/// §G: a provider is LOCAL iff its endpoint is loopback. Matches all loopback
/// spellings (localhost, 127.0.0.1, ::1, 0.0.0.0) so the local-sacred rule can't
/// be bypassed by re-spelling the loopback address. Hosts that merely RESOLVE to
/// loopback (e.g. wildcard DNS) are intentionally NOT local — we classify by what
/// the preset literally says, not by network resolution.
fn is_local(p: &ProviderPreset) -> bool {
    let parsed = match url::Url::parse(&p.endpoint) {
        Ok(u) => u,
        Err(_) => return false,
    };
    let host = parsed.host_str().unwrap_or("");
    matches!(host, "localhost" | "127.0.0.1" | "::1" | "0.0.0.0")
}
