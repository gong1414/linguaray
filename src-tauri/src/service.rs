//! Orchestrates a translation (spec architecture). v1: single engine, no fallback
//! yet (fallback engines arrive Phase 3 — the error classification is ready).

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

pub async fn translate(
    client: &reqwest::Client,
    keystore: &Keystore,
    preset: &ProviderPreset,
    input: TranslateInput<'_>,
) -> Result<String, Error> {
    // Spec §A "Plaintext-key claims": keep the key in memory only for the shortest
    // window between keystore-read and HTTP-send, and zeroize it after use.
    // Zeroizing<String> wipes the heap buffer on drop.
    let key = if preset.needs_key {
        let keys = keystore.load().map_err(Error::Keystore)?;
        let k = keys
            .get(&preset.id)
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
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
}
