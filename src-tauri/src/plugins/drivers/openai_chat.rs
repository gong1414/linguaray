//! One openai-chat Driver. Azure / Xiaomi / Gemini / Ollama are catalog rows.

use linguaray_contracts::{DriverError, DriverInput, EngineDriver, HttpRequestPlan, ProtocolKind};

pub struct OpenaiChatDriver;

impl EngineDriver for OpenaiChatDriver {
    fn id(&self) -> &'static str {
        "openai-chat"
    }

    fn protocol(&self) -> ProtocolKind {
        ProtocolKind::OpenaiChat
    }

    fn build_request(&self, input: &DriverInput<'_>) -> Result<HttpRequestPlan, DriverError> {
        let body = serde_json::json!({
            "model": input.model,
            "temperature": input.temperature,
            "max_tokens": input.max_tokens,
            "stream": input.stream,
            "messages": [
                {"role": "system", "content": input.system},
                {"role": "user", "content": input.user},
            ],
        });
        Ok(HttpRequestPlan {
            url: input.endpoint.to_string(),
            headers: input.auth.http_headers(input.key),
            query: input.auth.query_pairs(input.key),
            body,
        })
    }

    fn parse_response(&self, body: &serde_json::Value) -> Result<String, DriverError> {
        body["choices"][0]["message"]["content"]
            .as_str()
            .map(str::to_string)
            .ok_or_else(|| DriverError("no content".into()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use linguaray_contracts::AuthKind;

    fn input(auth: AuthKind, key: &'static str) -> DriverInput<'static> {
        DriverInput {
            endpoint: "https://example.test/v1/chat/completions",
            model: "m",
            auth,
            key,
            system: "sys",
            user: "hi",
            temperature: None,
            max_tokens: None,
            stream: false,
        }
    }

    #[test]
    fn azure_key_sends_api_key_not_authorization() {
        let plan = OpenaiChatDriver
            .build_request(&input(AuthKind::AzureKey, "sk-az"))
            .unwrap();
        assert_eq!(plan.headers, vec![("api-key".into(), "sk-az".into())]);
        assert!(plan
            .headers
            .iter()
            .all(|(name, _)| name.eq_ignore_ascii_case("api-key")));
    }

    #[test]
    fn none_sends_no_auth() {
        let plan = OpenaiChatDriver
            .build_request(&input(AuthKind::None, "ignored"))
            .unwrap();
        assert!(plan.headers.is_empty());
        assert!(plan.query.is_empty());
    }
}
