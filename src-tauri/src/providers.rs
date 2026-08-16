//! AI provider preset catalog — LinguaRay's core differentiator (spec §Wire).
//!
//! Each entry is CONFIG DATA (a FULL endpoint URL + dialect + default model + a
//! key-needed flag). The user fills a key and it works. Adding a provider = one
//! catalog row, not a Driver. HTTP encode/decode lives in EngineDrivers.

use linguaray_contracts::{AuthKind, ProtocolKind};

#[derive(Debug, Clone)]
pub struct ProviderPreset {
    pub id: String,
    pub label: String,
    /// FULL endpoint URL, e.g. "https://api.openai.com/v1/chat/completions".
    /// Stored in full (not base_url + route) because Url::join with a leading-'/'
    /// route would drop /v1 or /v1beta/openai. (spec §Wire)
    pub endpoint: String,
    pub protocol: ProtocolKind,
    pub default_model: String,
    pub needs_key: bool,
    pub auth: AuthKind,
}

pub fn presets() -> Vec<ProviderPreset> {
    linguaray_catalog::load()
        .map(|file| {
            file.providers
                .into_iter()
                .map(|p| ProviderPreset {
                    id: p.id,
                    label: p.label,
                    endpoint: p.endpoint,
                    protocol: p.protocol,
                    default_model: p.default_model,
                    needs_key: p.needs_key,
                    auth: p.auth,
                })
                .collect()
        })
        .unwrap_or_default()
}

/// Spec §Privacy: remote endpoints must be HTTPS; HTTP allowed only for loopback
/// (Ollama). Any other scheme is rejected.
pub fn validate_endpoint(url: &str) -> Result<(), String> {
    let parsed = url::Url::parse(url).map_err(|e| format!("bad url: {e}"))?;
    match parsed.scheme() {
        "https" => Ok(()),
        "http" => {
            let h = parsed.host_str().unwrap_or("");
            if h == "localhost" || h == "127.0.0.1" || h == "::1" {
                Ok(())
            } else {
                Err(format!("http only allowed for loopback, got {h}"))
            }
        }
        s => Err(format!("scheme {s} not allowed")),
    }
}
