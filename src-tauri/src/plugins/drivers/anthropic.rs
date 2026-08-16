//! Anthropic Messages Driver. Protocol wins: always x-api-key + version.

use linguaray_contracts::{DriverError, DriverInput, EngineDriver, HttpRequestPlan, ProtocolKind};

pub struct AnthropicDriver;

impl EngineDriver for AnthropicDriver {
    fn id(&self) -> &'static str {
        "anthropic"
    }

    fn protocol(&self) -> ProtocolKind {
        ProtocolKind::Anthropic
    }

    fn build_request(&self, input: &DriverInput<'_>) -> Result<HttpRequestPlan, DriverError> {
        let body = serde_json::json!({
            "model": input.model,
            "max_tokens": input.max_tokens.unwrap_or(1024),
            "system": input.system,
            "messages": [{"role": "user", "content": input.user}],
        });
        Ok(HttpRequestPlan {
            url: input.endpoint.to_string(),
            headers: vec![
                ("x-api-key".into(), input.key.into()),
                ("anthropic-version".into(), "2023-06-01".into()),
            ],
            query: vec![],
            body,
        })
    }

    fn parse_response(&self, body: &serde_json::Value) -> Result<String, DriverError> {
        body["content"][0]["text"]
            .as_str()
            .map(str::to_string)
            .ok_or_else(|| DriverError("no text".into()))
    }
}
