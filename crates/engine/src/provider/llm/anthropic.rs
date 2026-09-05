use std::sync::Arc;

use async_trait::async_trait;
use linguaray_core::{
    ChatMessage, ChatRequest, ChatResponse, ChatRole, LlmError, LlmService, LlmStreamReceiver,
    Provider, ResponseFormat,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::configured_default_model;

// ── Config ────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize, PartialEq, Serialize)]
pub struct AnthropicProviderConfig {
    #[serde(rename = "apiKey", alias = "api_key")]
    pub api_key: String,
    #[serde(rename = "baseUrl", alias = "base_url")]
    pub base_url: Option<String>,
    #[serde(rename = "defaultModel", alias = "default_model", default)]
    pub default_model: String,
    #[serde(rename = "modelsUrl", alias = "models_url", default)]
    pub models_url: Option<String>,
}

// ── Provider ──────────────────────────────────────────────────────────────────

pub struct AnthropicProvider {
    llm_service: Arc<AnthropicLlmService>,
}

impl AnthropicProvider {
    pub fn new(config: AnthropicProviderConfig) -> Result<Self, String> {
        if config.api_key.trim().is_empty() {
            return Err("api_key must not be empty".to_owned());
        }
        let base_url = config
            .base_url
            .clone()
            .unwrap_or_else(|| "https://api.anthropic.com".to_string());

        let http = HttpClient::new(&base_url)?;
        let default_model = configured_default_model(&config.default_model)?;

        let llm_service = Arc::new(AnthropicLlmService {
            api_key: config.api_key.clone(),
            default_model: default_model.clone(),
            http: http.clone(),
            models_url: config.models_url.filter(|url| !url.trim().is_empty()),
        });

        Ok(Self { llm_service })
    }
}

#[async_trait]
impl Provider for AnthropicProvider {
    fn name(&self) -> &'static str {
        "anthropic"
    }

    fn llm(&self) -> Option<&dyn LlmService> {
        Some(self.llm_service.as_ref())
    }

    async fn list_models(&self) -> Result<Vec<String>, LlmError> {
        self.llm_service.list_models().await
    }
}

// ── LLM Service (core) ────────────────────────────────────────────────────────

#[derive(Clone)]
struct HttpClient {
    base_url: String,
    client: reqwest::Client,
}

impl HttpClient {
    fn new(base_url: &str) -> Result<Self, String> {
        Ok(Self {
            base_url: base_url.to_string(),
            client: crate::common::build_http_client()?,
        })
    }

    fn join_url(&self, path: &str) -> String {
        crate::catalog::urls::join_openai_path(&self.base_url, path)
    }
}

pub struct AnthropicLlmService {
    api_key: String,
    default_model: String,
    http: HttpClient,
    models_url: Option<String>,
}

impl AnthropicLlmService {
    fn build_anthropic_body(&self, request: &ChatRequest, stream: bool) -> Value {
        // Anthropic does NOT support system role as a message — it's a top-level field.
        let mut system_prompts: Vec<String> = Vec::new();
        let messages: Vec<Value> = request
            .messages
            .iter()
            .filter_map(|m| match m.role {
                ChatRole::System => {
                    system_prompts.push(m.content.clone());
                    None
                }
                ChatRole::User => Some(serde_json::json!({
                    "role": "user",
                    "content": m.content,
                })),
                ChatRole::Assistant => Some(serde_json::json!({
                    "role": "assistant",
                    "content": m.content,
                })),
            })
            .collect();

        let model = if request.model.is_empty() {
            &self.default_model
        } else {
            &request.model
        };

        let max_tokens = request.max_tokens.unwrap_or(4096);

        let mut body = serde_json::json!({
            "model": model,
            "max_tokens": max_tokens,
            "messages": messages,
            "stream": stream,
        });

        // The Messages API has no `response_format` parameter; fall back to a
        // system-level instruction so JSON modes still take effect.
        match &request.response_format {
            Some(ResponseFormat::JsonObject) => {
                system_prompts.push(
                    "You must respond with a single valid JSON object and nothing else — \
                     no markdown fences, no commentary."
                        .to_string(),
                );
            }
            Some(ResponseFormat::JsonSchema { json_schema }) => {
                let schema = json_schema.get("schema").unwrap_or(json_schema);
                system_prompts.push(format!(
                    "You must respond with a single valid JSON object matching this JSON \
                     schema and nothing else — no markdown fences, no commentary:\n{schema}"
                ));
            }
            Some(ResponseFormat::Text) | None => {}
        }

        if !system_prompts.is_empty() {
            let system = system_prompts.join("\n\n");
            body["system"] = serde_json::json!(system);
        }

        if let Some(temp) = request.temperature {
            body["temperature"] = serde_json::json!(temp);
        }

        body
    }

    async fn list_models(&self) -> Result<Vec<String>, LlmError> {
        let url = self
            .models_url
            .clone()
            .unwrap_or_else(|| self.http.join_url("/v1/models"));
        let mut cursor: Option<String> = None;
        let mut seen = std::collections::HashSet::new();
        let mut models = std::collections::BTreeSet::new();
        // Bound the entire traversal, including providers that return bad cursors.
        tokio::time::timeout(crate::catalog::MODELS_FETCH_TIMEOUT, async {
            for _ in 0..100 {
                let mut request = self
                    .http
                    .client
                    .get(&url)
                    .header("x-api-key", &self.api_key)
                    .header("anthropic-version", "2023-06-01")
                    .query(&[("limit", "1000")]);
                if let Some(after) = &cursor {
                    request = request.query(&[("after_id", after)]);
                }
                let response = request
                    .send()
                    .await
                    .map_err(|e| LlmError::NetworkError(e.to_string()))?;
                let status = response.status().as_u16();
                let body = response
                    .text()
                    .await
                    .map_err(|e| LlmError::NetworkError(e.to_string()))?;
                match crate::catalog::interpret_models_response(status, &body, &[&self.api_key]) {
                    crate::catalog::CandidateOutcome::Success(ids) => models.extend(ids),
                    crate::catalog::CandidateOutcome::TryNext => {
                        return Err(LlmError::InvalidRequest(format!(
                            "HTTP {status}: models endpoint unavailable"
                        )))
                    }
                    crate::catalog::CandidateOutcome::Fail(error) => {
                        return Err(LlmError::NetworkError(error))
                    }
                }
                let json: Value = serde_json::from_str(&body)
                    .map_err(|e| LlmError::SerializationError(e.to_string()))?;
                if json["has_more"].as_bool() != Some(true) {
                    return Ok(models.into_iter().collect());
                }
                let next = json["last_id"]
                    .as_str()
                    .filter(|id| !id.is_empty())
                    .ok_or_else(|| {
                        LlmError::SerializationError("missing model pagination cursor".into())
                    })?;
                if !seen.insert(next.to_owned()) {
                    return Err(LlmError::SerializationError(
                        "repeated model pagination cursor".into(),
                    ));
                }
                cursor = Some(next.to_owned());
            }
            Err(LlmError::SerializationError(
                "model pagination exceeds limit".into(),
            ))
        })
        .await
        .map_err(|_| LlmError::NetworkError("model query timed out".into()))?
        .map_err(|error: LlmError| {
            LlmError::NetworkError(crate::catalog::urls::redact_secrets(
                &error.to_string(),
                &[&self.api_key],
            ))
        })
    }
}

#[async_trait]
impl LlmService for AnthropicLlmService {
    fn provider_name(&self) -> &'static str {
        "anthropic"
    }

    fn available_models(&self) -> Vec<String> {
        vec![self.default_model.clone()]
    }

    async fn chat(&self, request: ChatRequest) -> Result<ChatResponse, LlmError> {
        let body = self.build_anthropic_body(&request, false);

        let response = self
            .http
            .client
            .post(self.http.join_url("/v1/messages"))
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| LlmError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let body_text = response.text().await.unwrap_or_default();
            return Err(match status.as_u16() {
                401 | 403 => LlmError::AuthError(body_text),
                429 => LlmError::RateLimitError(body_text),
                400..=499 => LlmError::InvalidRequest(body_text),
                _ => LlmError::NetworkError(format!("HTTP {status}: {body_text}")),
            });
        }

        let raw: Value = response
            .json()
            .await
            .map_err(|e| LlmError::SerializationError(e.to_string()))?;

        let chat_response = parse_anthropic_response(&raw);
        Ok(chat_response)
    }

    async fn chat_stream(&self, request: ChatRequest) -> Result<LlmStreamReceiver, LlmError> {
        let body = self.build_anthropic_body(&request, true);

        let response = self
            .http
            .client
            .post(self.http.join_url("/v1/messages"))
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| LlmError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let body_text = response.text().await.unwrap_or_default();
            return Err(match status.as_u16() {
                401 | 403 => LlmError::AuthError(body_text),
                429 => LlmError::RateLimitError(body_text),
                400..=499 => LlmError::InvalidRequest(body_text),
                _ => LlmError::NetworkError(format!("HTTP {status}: {body_text}")),
            });
        }

        Ok(super::streaming::receive(
            response,
            super::streaming::WireFormat::Anthropic,
            vec![self.api_key.clone()],
        ))
    }
}

/// Parse Anthropic's non-streaming JSON response into a `ChatResponse`.
fn parse_anthropic_response(raw: &Value) -> ChatResponse {
    let id = raw["id"].as_str().map(|s| s.to_string());
    let model = raw["model"].as_str().unwrap_or("unknown").to_string();

    let content_text = raw["content"]
        .as_array()
        .and_then(|blocks| {
            blocks.iter().find_map(|block| {
                if block["type"] == "text" {
                    block["text"].as_str().map(|s| s.to_string())
                } else {
                    None
                }
            })
        })
        .unwrap_or_default();

    let stop_reason = raw["stop_reason"].as_str().map(|s| s.to_string());

    let message = ChatMessage::assistant(content_text);
    let choice = linguaray_core::ChatChoice {
        index: 0,
        message,
        finish_reason: stop_reason,
    };

    let usage = raw.get("usage").map(|u| {
        let input = u["input_tokens"].as_u64().unwrap_or(0) as u32;
        let output = u["output_tokens"].as_u64().unwrap_or(0) as u32;
        linguaray_core::ChatUsage {
            prompt_tokens: input,
            completion_tokens: output,
            total_tokens: input + output,
        }
    });

    ChatResponse {
        id,
        model,
        choices: vec![choice],
        usage,
    }
}
