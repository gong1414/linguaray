//! Unofficial Tencent Transmart web endpoint. Experimental.

use async_trait::async_trait;
use linguaray_core::{
    Provider, TextTranslation, TranslateRequest, TranslateResponse, TranslationError,
    TranslationService,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::common::{ClassifyHttpResponse, HttpClient};

const USER_AGENT: &str = "LinguaRay/1.0";

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(default)]
pub struct TransmartProviderConfig {
    #[serde(default)]
    pub username: String,
    #[serde(default)]
    pub token: String,
    #[serde(rename = "baseUrl", alias = "base_url")]
    pub base_url: Option<String>,
}

pub struct TransmartProvider {
    translation_service: TransmartTranslationService,
}

struct TransmartTranslationService {
    username: String,
    token: String,
    http: HttpClient,
}

impl TransmartProvider {
    pub fn new(config: TransmartProviderConfig) -> Result<Self, String> {
        let base_url = config
            .base_url
            .filter(|url| !url.trim().is_empty())
            .unwrap_or_else(|| "https://transmart.qq.com".to_owned());
        Ok(Self {
            translation_service: TransmartTranslationService {
                username: config.username.trim().to_owned(),
                token: config.token.trim().to_owned(),
                http: HttpClient::proxy_aware(base_url)?,
            },
        })
    }
}

pub fn parse_transmart_response(value: &Value) -> Result<String, TranslationError> {
    let parts = value
        .get("auto_translation")
        .and_then(Value::as_array)
        .ok_or_else(|| {
            TranslationError::SerializationError("missing auto_translation".to_owned())
        })?;
    let mut text = String::new();
    for part in parts {
        if let Some(chunk) = part.as_str() {
            text.push_str(chunk);
        }
    }
    Ok(text)
}

pub fn transmart_request_body(
    source: &str,
    target: &str,
    text: &str,
    username: &str,
    token: &str,
) -> Value {
    let mut header = json!({ "fn": "auto_translation" });
    if !username.is_empty() {
        header["user"] = json!(username);
    }
    if !token.is_empty() {
        header["token"] = json!(token);
    }
    json!({
        "header": header,
        "type": "plain",
        "source": {
            "lang": source,
            "text_list": [text]
        },
        "target": {
            "lang": target
        }
    })
}

#[async_trait(?Send)]
impl TranslationService for TransmartTranslationService {
    async fn translate(
        &self,
        request: TranslateRequest,
    ) -> Result<TranslateResponse, TranslationError> {
        let target = request.target_language.ok_or_else(|| {
            TranslationError::InvalidRequest("target_language is required".to_owned())
        })?;
        let source = request
            .source_language
            .as_deref()
            .filter(|code| !code.is_empty() && *code != "auto")
            .unwrap_or("auto");
        let body =
            transmart_request_body(source, &target, &request.text, &self.username, &self.token);
        let builder = self
            .http
            .post("/api/imt")
            .header("User-Agent", USER_AGENT)
            .header("Content-Type", "application/json")
            .json(&body);
        let response = self
            .http
            .execute(builder)
            .await
            .map_err(TranslationError::from_network_error)?;
        let response = TranslationError::from_response_redacting(
            "tencent_transmart_web",
            response,
            &[self.token.as_str()],
        )
        .await?;
        let data: Value = response
            .json()
            .await
            .map_err(|error| TranslationError::SerializationError(error.to_string()))?;
        let text = parse_transmart_response(&data)?;
        Ok(TranslateResponse {
            translations: vec![TextTranslation {
                detected_source_language: None,
                text,
                audio_url: None,
            }],
        })
    }
}

impl Provider for TransmartProvider {
    fn name(&self) -> &'static str {
        "tencent_transmart_web"
    }

    fn translation(&self) -> Option<&dyn TranslationService> {
        Some(&self.translation_service)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn concatenates_auto_translation_array() {
        let value = serde_json::json!({ "auto_translation": ["你好", "世界"] });
        assert_eq!(parse_transmart_response(&value).unwrap(), "你好世界");
    }

    #[test]
    fn omits_empty_credentials() {
        let body = transmart_request_body("en", "zh", "Hello", "", "");
        assert_eq!(body["header"]["fn"], "auto_translation");
        assert!(body["header"].get("user").is_none());
        assert!(body["header"].get("token").is_none());
        assert_eq!(body["source"]["text_list"][0], "Hello");
    }

    #[test]
    fn includes_optional_credentials() {
        let body = transmart_request_body("en", "zh", "Hello", "alice", "tok");
        assert_eq!(body["header"]["user"], "alice");
        assert_eq!(body["header"]["token"], "tok");
    }
}
