//! Provider wire contract — spec §Wire.
//!
//! Two option spaces kept distinct:
//! - App translation options (domain/formality/system_prompt_override) shape the
//!   PROMPT (message content), never top-level wire fields.
//! - WireParams (model/temperature/max_tokens/stream) is a strong-typed whitelist
//!   for top-level body fields.
//!
//! HTTP encode/decode lives in EngineDrivers. This module is the transport
//! executor: Driver plan → reqwest → status classify → Driver parse.

use crate::error::{ConfigKind, Error, FallbackKind};
use crate::plugins::drivers::builtin_registry;
use crate::providers::ProviderPreset;
use linguaray_contracts::{DriverInput, EngineDriverRegistry};
use serde::Serialize;

/// Top-level wire fields. Closed whitelist; nothing else reaches the body as a
/// sibling field of these. App options influence the body only via the prompt
/// (message content), never as top-level wire fields.
#[derive(Debug, Clone, Serialize)]
pub struct WireParams {
    pub model: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
    pub stream: bool,
}

/// App-layer translation options. These shape the prompt, not the wire fields.
#[derive(Debug, Clone, Default)]
pub struct AppOptions {
    pub domain: Option<String>,
    pub formality: Option<String>,
    pub system_prompt_override: Option<String>,
}

/// Build the system + user message content for a translation request.
/// (Spec §Wire: app options influence message content only.)
pub fn build_prompt(text: &str, from: &str, to: &str, opts: &AppOptions) -> (String, String) {
    let mut system = match &opts.system_prompt_override {
        Some(s) => s.clone(),
        None => "You are a professional translator. Translate the user's text. \
                 Output ONLY the translation, no explanations."
            .to_string(),
    };
    if let Some(d) = &opts.domain {
        system.push_str(&format!(" Domain: {d}."));
    }
    if let Some(f) = &opts.formality {
        system.push_str(&format!(" Register/formality: {f}."));
    }
    let src = if from == "auto" {
        "the source language (detect it)".to_string()
    } else {
        from.to_string()
    };
    let user = format!("Translate from {src} into {to}:\n\n{text}");
    (system, user)
}

/// Call a provider via the builtin Driver registry. PURE: takes preset + key +
/// params + messages, returns text. Classifies HTTP status into
/// FallbackEligible (429/5xx) vs Config (401/403).
pub async fn call(
    client: &reqwest::Client,
    preset: &ProviderPreset,
    key: &str,
    params: &WireParams,
    system: &str,
    user: &str,
) -> Result<String, Error> {
    let Some(driver) = builtin_registry().get(preset.protocol) else {
        return Err(Error::Config(ConfigKind::Unsupported {
            provider: preset.id.clone(),
            reason: format!("{:?}", preset.protocol),
        }));
    };
    let plan = driver
        .build_request(&DriverInput {
            endpoint: &preset.endpoint,
            model: &params.model,
            auth: preset.auth,
            key,
            system,
            user,
            temperature: params.temperature,
            max_tokens: params.max_tokens,
            stream: params.stream,
        })
        .map_err(|e| {
            Error::Config(ConfigKind::Unsupported {
                provider: preset.id.clone(),
                reason: e.0,
            })
        })?;
    let mut req = client.post(&plan.url);
    for (name, value) in &plan.headers {
        req = req.header(name, value);
    }
    if !plan.query.is_empty() {
        req = req.query(&plan.query);
    }
    let resp = req.json(&plan.body).send().await.map_err(|e| {
        if e.is_timeout() {
            Error::FallbackEligible(FallbackKind::Timeout)
        } else {
            Error::FallbackEligible(FallbackKind::Network(e.to_string()))
        }
    })?;
    let status = resp.status().as_u16();
    if status == 401 || status == 403 {
        return Err(Error::Config(ConfigKind::AuthFailed {
            provider: preset.id.clone(),
            status,
        }));
    }
    if status == 429 || (500..600).contains(&status) {
        return Err(Error::FallbackEligible(FallbackKind::ProviderStatus {
            status,
        }));
    }
    if !resp.status().is_success() {
        return Err(Error::Config(ConfigKind::InvalidRequest {
            provider: preset.id.clone(),
            status,
        }));
    }
    let json: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| Error::FallbackEligible(FallbackKind::Parse(e.to_string())))?;
    driver
        .parse_response(&json)
        .map_err(|e| Error::FallbackEligible(FallbackKind::Parse(e.0)))
}
