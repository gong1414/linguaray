//! Provider wire contract — spec §Wire.
//!
//! Two option spaces kept distinct:
//! - App translation options (domain/formality/system_prompt_override) shape the
//!   PROMPT (message content), never top-level wire fields.
//! - WireParams (model/temperature/max_tokens/stream) is a strong-typed whitelist
//!   for top-level body fields.

use serde::Serialize;

#[derive(Debug, Clone, Copy)]
pub enum ApiKind {
    OpenAIChat,
    Anthropic,
}

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
                 Output ONLY the translation, no explanations.".to_string(),
    };
    if let Some(d) = &opts.domain {
        system.push_str(&format!(" Domain: {d}."));
    }
    if let Some(f) = &opts.formality {
        system.push_str(&format!(" Register/formality: {f}."));
    }
    let src = if from == "auto" { "the source language (detect it)".to_string() } else { from.to_string() };
    let user = format!("Translate from {src} into {to}:\n\n{text}");
    (system, user)
}

use crate::error::{ConfigKind, Error, FallbackKind};

/// Call a provider. PURE: takes preset + key + params + messages, returns text.
/// Classifies HTTP status into FallbackEligible (429/5xx) vs Config (401/403).
pub async fn call(
    client: &reqwest::Client,
    preset: &crate::providers::ProviderPreset,
    key: &str,
    params: &WireParams,
    system: &str,
    user: &str,
) -> Result<String, Error> {
    let resp = match preset.api_kind {
        ApiKind::OpenAIChat => {
            let body = serde_json::json!({
                "model": params.model,
                "temperature": params.temperature,
                "max_tokens": params.max_tokens,
                "stream": params.stream,
                "messages": [
                    {"role": "system", "content": system},
                    {"role": "user", "content": user},
                ],
            });
            client.post(&preset.endpoint)
                .bearer_auth(key)
                .json(&body)
                .send().await
        }
        ApiKind::Anthropic => {
            let body = serde_json::json!({
                "model": params.model,
                "max_tokens": params.max_tokens.unwrap_or(1024),
                "system": system,
                "messages": [{"role": "user", "content": user}],
            });
            client.post(&preset.endpoint)
                .header("x-api-key", key)
                .header("anthropic-version", "2023-06-01")
                .json(&body)
                .send().await
        }
    };
    let resp = resp.map_err(|e| {
        // §G: distinguish timeout (own variant) from generic network errors so the
        // fallback path + UI can report it precisely. Both stay FallbackEligible.
        if e.is_timeout() {
            Error::FallbackEligible(FallbackKind::Timeout)
        } else {
            Error::FallbackEligible(FallbackKind::Network(e.to_string()))
        }
    })?;
    let status = resp.status().as_u16();
    if status == 401 || status == 403 {
        return Err(Error::Config(ConfigKind::AuthFailed { provider: preset.id.clone(), status }));
    }
    if status == 429 || (500..600).contains(&status) {
        return Err(Error::FallbackEligible(FallbackKind::ProviderStatus { status }));
    }
    if !resp.status().is_success() {
        // §G: a non-2xx that isn't 401/403/429/5xx is a 4xx (400/404/422/...).
        // Treat as Config (InvalidRequest) — NOT fallback-eligible. Retrying with a
        // 2nd provider would needlessly send the text elsewhere for what is almost
        // certainly a bad model/endpoint/request-shape problem.
        return Err(Error::Config(ConfigKind::InvalidRequest { provider: preset.id.clone(), status }));
    }
    let json: serde_json::Value = resp.json().await
        .map_err(|e| Error::FallbackEligible(FallbackKind::Parse(e.to_string())))?;
    let text = match preset.api_kind {
        ApiKind::OpenAIChat => json["choices"][0]["message"]["content"]
            .as_str().ok_or_else(|| Error::FallbackEligible(FallbackKind::Parse("no content".into())))?.to_string(),
        ApiKind::Anthropic => json["content"][0]["text"]
            .as_str().ok_or_else(|| Error::FallbackEligible(FallbackKind::Parse("no text".into())))?.to_string(),
    };
    Ok(text)
}
