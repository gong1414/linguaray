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

/// Run ONLY the primary attempt — no §G fallback conversion. B6 (P1-4):
/// `translate_with_fallback_ref(..., None)` converts a non-local
/// `FallbackEligible` into `LocalNoFallback`, so a session-level eligibility
/// check can NEVER see the original `FallbackEligible`. This fn preserves the
/// RAW `Error` so [`eligible_for_session_fallback`] can detect it. Used by
/// [`translate_parallel`] to drive each primary in parallel, then decide the
/// (single) session-level fallback once all primaries have settled.
///
/// `Config`/`Keystore` errors propagate unchanged; on success the `Translation`
/// is tagged with the primary preset id (engine == preset.id).
pub async fn translate_primary_only(
    client: &reqwest::Client,
    keystore: &Keystore,
    primary_preset: &ProviderPreset,
    input: TranslateInput<'_>,
) -> Result<Translation, Error> {
    translate(
        client,
        keystore,
        primary_preset,
        TranslateInput {
            text: input.text,
            from: input.from,
            to: input.to,
            options: input.options,
        },
    )
    .await
}

/// B6 (P1-4, rev-6-4): PURE eligibility decision for the bounded session-level
/// fallback. Returns `true` iff the session qualifies for EXACTLY ONE fallback
/// call. Rules:
///
/// - `local_primary_failed` → `false` (local-primary sacred: a LOCAL primary
///   that failed blocks any remote fallback for this session).
/// - ANY success → `false` (partial success: surface what we have, don't
///   silently inject a fallback card).
/// - At least one NON-LOCAL engine failed with `FallbackEligible` → `true`
///   (all-remote-transient session: one fallback over the whole text).
/// - Anything else (Config/Keystore errors, only-local failures) → `false`.
///
/// `locality[i]` mirrors `outcomes[i]`: `true` if engine `i` was LOCAL. A
/// missing locality entry (pre-failed profiles that never produced a preset) is
/// treated as `false` (non-local) — a pre-failed primary contributes
/// `local_primary_failed=false` and only blocks via its Config error.
pub fn eligible_for_session_fallback(
    outcomes: &[TranslationOutcome],
    locality: &[bool],
    local_primary_failed: bool,
) -> bool {
    if local_primary_failed {
        return false;
    }
    let any_success = outcomes.iter().any(|o| o.result.is_ok());
    if any_success {
        return false;
    }
    // Any Config/Keystore error blocks the session fallback: these are "user
    // must fix Settings" problems and silently running a fallback would mask
    // them. (Plan deviation: the verbatim plan body only checked `.any()` for a
    // non-local FallbackEligible, which would wrongly return true for a session
    // like [FallbackEligible, Config]. Added this guard so test 5 holds.)
    let any_config_or_keystore = outcomes.iter().any(|o| {
        matches!(o.result, Err(Error::Config(_)) | Err(Error::Keystore(_)))
    });
    if any_config_or_keystore {
        return false;
    }
    outcomes
        .iter()
        .enumerate()
        .any(|(i, o)| {
            let was_local = locality.get(i).copied().unwrap_or(false);
            !was_local && matches!(o.result, Err(Error::FallbackEligible(_)))
        })
}

/// 并行调用多个 AI 引擎，再用 B6 bounded session-level fallback（P1-4, rev-6-4）。
///
/// - 每个 profile 经 [`profile_to_preset`] 转 preset；转换失败（如 google_translate
///   协议）→ 该引擎产出 `Err(Config::Unsupported)` outcome，**不** panic、**不**丢弃。
/// - 所有 ready 引擎用 `futures::future::join_all` 并发驱动，各自跑
///   [`translate_primary_only`]（**不**走 per-engine fallback；保留 RAW `Error`，
///   这样 [`eligible_for_session_fallback`] 才能看到 `FallbackEligible`）。
/// - `fallback` 是 `Option<Arc<dyn TraditionalEngine>>`，当整个 session 满足
///   eligibility 时被调用 **至多一次**：所有非 local primary 全部 transient 失败、
///   无任何成功、无 Config/Keystore 错误、且 primary 不是 local 失败（local-sacred）。
///   fallback 结果作为 **新 outcome 卡片** 追加（uuid = fallback engine id）。
/// - 返回顺序：N 个 primary outcome 严格按输入顺序（B5），随后至多 1 个 fallback
///   outcome（仅在 eligible 且 fallback Some 时）。pre-failed 条目留在原输入位置。
///
/// §G 不变量：`Config`/`Keystore` 错误绝不因另一个引擎成功而被"覆盖"——它们作为
/// 各自 outcome 的 Err 保留，前端按 Surface 03 的 "partial success" 渲染。Local
/// primary 的 `FallbackEligible` **不计入** session eligibility（local-sacred），且
/// local primary 失败会 **完全阻断** session fallback（rev-6-4）。
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
    // Drive all ready entries concurrently via translate_primary_only (B6,
    // P1-4). Unlike translate_with_fallback_ref(..., None) — which converts a
    // non-local FallbackEligible into LocalNoFallback and so hides it from the
    // session-level check — translate_primary_only preserves the RAW Error so
    // eligible_for_session_fallback can detect it.
    //
    // Capture was_local = is_local(&preset) HERE (before the await) so the
    // locality classification is stable and not affected by the future's move.
    // text/from/to are &str (borrowed fn params) — the futures borrow them and
    // they remain in scope for the session-fallback call after join_all.
    let futs: Vec<_> = entries
        .iter()
        .filter_map(|(idx, ready, _)| {
            ready.as_ref().map(|(uuid, preset)| {
                (*idx, uuid.clone(), preset, is_local(preset))
            })
        })
        .map(|(idx, uuid, preset, was_local)| {
            let options = options.clone();
            async move {
                let input = TranslateInput { text, from, to, options };
                let result =
                    translate_primary_only(client, keystore, preset, input).await;
                (idx, was_local, TranslationOutcome { uuid, result })
            }
        })
        .collect();
    let ready_results = futures::future::join_all(futs).await;
    // Index the ready results by input index so we can rebuild strict input
    // order AND carry each outcome's locality in parallel. The (was_local,
    // outcome) pair is consumed by the ordered walk below.
    let mut by_idx: std::collections::HashMap<usize, (bool, TranslationOutcome)> =
        std::collections::HashMap::with_capacity(ready_results.len());
    for (idx, was_local, o) in ready_results {
        by_idx.insert(idx, (was_local, o));
    }
    // Read the PRIMARY's locality BEFORE the consuming walk. The primary is the
    // entry at input index 0 (a pre-failed primary contributes locality=false —
    // it never produced a preset, so it can't be local). This drives the
    // local_primary_failed rule (rev-6-4): a LOCAL primary that failed blocks
    // the session fallback entirely.
    let primary_was_local = by_idx.get(&0).map(|(wl, _)| *wl).unwrap_or(false);
    // Build outcomes + a parallel locality Vec in STRICT input order. Pre-failed
    // entries (no preset → no is_local call) are inserted in place with
    // locality=false. This preserves B5's ordering invariant while giving the
    // eligibility check a locality entry for every outcome.
    let mut outcomes: Vec<TranslationOutcome> = Vec::with_capacity(entries.len());
    let mut locality: Vec<bool> = Vec::with_capacity(entries.len());
    for (idx, _ready, pre_failed) in entries {
        if let Some(o) = pre_failed {
            outcomes.push(o); // pre-failed → locality false (never had a preset)
            locality.push(false);
        } else if let Some((was_local, o)) = by_idx.remove(&idx) {
            outcomes.push(o);
            locality.push(was_local);
        }
    }
    // B6 bounded session-level fallback decision (P1-4, rev-6-4). A LOCAL
    // primary that failed blocks the session fallback; otherwise the pure
    // eligibility fn decides (all-remote-transient → one fallback).
    let local_primary_failed =
        primary_was_local && outcomes.first().map(|o| o.result.is_err()).unwrap_or(false);
    let eligible = eligible_for_session_fallback(&outcomes, &locality, local_primary_failed);
    // Fire the fallback AT MOST ONCE per session. The fallback translates the
    // WHOLE text once and is appended as a NEW outcome card (uuid = fallback
    // engine id), so callers see N primary cards + at most 1 fallback card.
    // text/from/to are still borrowed here (they're &str fn params), and
    // options was cloned into the futures — re-clone once for this call.
    if eligible {
        if let Some(eng) = fallback.as_deref() {
            let fb_id = eng.id().to_string();
            let fb_result = eng
                .translate(client, text, from, to)
                .await
                .map(|text| Translation { text, engine: fb_id.clone() });
            outcomes.push(TranslationOutcome { uuid: fb_id, result: fb_result });
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
