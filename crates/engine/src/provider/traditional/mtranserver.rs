//! HTTP client for a user-hosted MTranServer instance.

use async_trait::async_trait;
use linguaray_core::{
    Provider, TextTranslation, TranslateRequest, TranslateResponse, TranslationError,
    TranslationService,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::common::HttpClient;

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(default)]
pub struct MTranServerProviderConfig {
    #[serde(default)]
    pub token: String,
    #[serde(rename = "baseUrl", alias = "base_url")]
    pub base_url: Option<String>,
}

pub struct MTranServerProvider {
    translation_service: MTranServerTranslationService,
}

struct MTranServerTranslationService {
    token: String,
    http: HttpClient,
}

impl MTranServerProvider {
    pub fn new(config: MTranServerProviderConfig) -> Result<Self, String> {
        let base_url = config
            .base_url
            .filter(|url| !url.trim().is_empty())
            .unwrap_or_else(|| "http://127.0.0.1:8989".to_owned());
        Ok(Self {
            translation_service: MTranServerTranslationService {
                token: config.token.trim().to_owned(),
                http: HttpClient::new(base_url, Default::default()),
            },
        })
    }
}

pub fn parse_mtranserver_response(value: &Value) -> Result<String, TranslationError> {
    value
        .get("result")
        .and_then(Value::as_str)
        .map(str::to_owned)
        .ok_or_else(|| TranslationError::SerializationError("missing result".to_owned()))
}

impl MTranServerTranslationService {
    fn authorized(&self, builder: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        if self.token.is_empty() {
            builder
        } else {
            builder.header("Authorization", format!("Bearer {}", self.token))
        }
    }

    async fn health(&self) -> Result<(), TranslationError> {
        let builder = self.authorized(self.http.get("/health"));
        let response = self
            .http
            .execute(builder)
            .await
            .map_err(TranslationError::from_network_error)?;
        TranslationError::from_response("mtranserver", response)
            .await
            .map(|_| ())
    }
}

#[async_trait(?Send)]
impl TranslationService for MTranServerTranslationService {
    async fn translate(
        &self,
        request: TranslateRequest,
    ) -> Result<TranslateResponse, TranslationError> {
        let target = request.target_language.ok_or_else(|| {
            TranslationError::InvalidRequest("target_language is required".to_owned())
        })?;
        let source = request
            .source_language
            .filter(|code| !code.is_empty())
            .unwrap_or_else(|| "en".to_owned());
        let body = json!({
            "from": source,
            "to": target,
            "text": request.text,
            "html": false,
        });
        let builder = self
            .authorized(self.http.post("/translate"))
            .header("Content-Type", "application/json")
            .json(&body);
        let response = self
            .http
            .execute(builder)
            .await
            .map_err(TranslationError::from_network_error)?;
        let response = TranslationError::from_response_redacting(
            "mtranserver",
            response,
            &[self.token.as_str()],
        )
        .await?;
        let data: Value = response
            .json()
            .await
            .map_err(|error| TranslationError::SerializationError(error.to_string()))?;
        let text = parse_mtranserver_response(&data)?;
        Ok(TranslateResponse {
            translations: vec![TextTranslation {
                detected_source_language: None,
                text,
                audio_url: None,
            }],
        })
    }
}

#[async_trait]
impl Provider for MTranServerProvider {
    fn name(&self) -> &'static str {
        "mtranserver"
    }

    fn translation(&self) -> Option<&dyn TranslationService> {
        Some(&self.translation_service)
    }

    async fn list_models(&self) -> Result<Vec<String>, linguaray_core::LlmError> {
        self.translation_service
            .health()
            .await
            .map(|_| Vec::new())
            .map_err(|error| linguaray_core::LlmError::NetworkError(error.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_result_field() {
        let value = serde_json::json!({ "result": "Hallo" });
        assert_eq!(parse_mtranserver_response(&value).unwrap(), "Hallo");
    }
}
