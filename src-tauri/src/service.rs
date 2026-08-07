//! Orchestrates a translation (spec architecture). §G classified fallback:
//! primary (AI) engine first; on `FallbackEligible` retry ONCE with a configured
//! traditional engine. `Config`/`Keystore` errors propagate; LOCAL primaries are
//! sacred (never silently degrade to a remote fallback).

use crate::engines::TraditionalEngine;
use crate::error::{ConfigKind, Error};
use crate::keystore::Keystore;
use crate::providers::ProviderPreset;
use crate::wire::{build_prompt, call, AppOptions, WireParams};

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
pub async fn translate_with_fallback(
    client: &reqwest::Client,
    keystore: &Keystore,
    primary_preset: &ProviderPreset,
    input: TranslateInput<'_>,
    fallback: Option<Box<dyn TraditionalEngine>>,
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
