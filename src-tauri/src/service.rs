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
#[derive(Debug)]
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
// 注：未派生 `Clone`，因为 `result` 持有 `Error`，而 `Error::Keystore` 包装了
// `KeystoreError`（含 `std::io::Error`，不可 Clone）。计划的测试不需要 clone
// outcome（`sorted_by_uuid` 按值接收并原地排序），所以 `Debug` 足够。
#[derive(Debug)]
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
            // `.map` 的闭包是 `FnMut`，会被调用多次，不能把外层按引用捕获的
            // `options` 直接 move 进 `async move`。每个 future 各 clone 一份。
            // `text`/`from`/`to`/`fb_ref` 是 `&str`/`Option<&dyn>`（Copy），无此问题。
            let options = options.clone();
            let fb_ref: Option<&dyn TraditionalEngine> = fallback.as_deref();
            async move {
                let input = TranslateInput {
                    text,
                    from,
                    to,
                    options,
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
