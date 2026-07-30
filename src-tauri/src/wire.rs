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
