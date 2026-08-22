//! HTTP client for a user-hosted LibreTranslate instance.

use async_trait::async_trait;
use linguaray_core::{
    Provider, TextTranslation, TranslateRequest, TranslateResponse, TranslationError,
    TranslationService,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::common::http_client::HttpClient;

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(default)]
pub struct LibreTranslateProviderConfig {
    #[serde(rename = "apiKey", alias = "api_key")]
    pub api_key: String,
    #[serde(rename = "baseUrl", alias = "base_url")]
    pub base_url: Option<String>,
}

pub struct LibreTranslateProvider {
    translation_service: LibreTranslateTranslationService,
}

struct LibreTranslateTranslationService {
    api_key: String,
    http: HttpClient,
}

impl LibreTranslateProvider {
    pub fn new(config: LibreTranslateProviderConfig) -> Result<Self, String> {
        let base_url = config
            .base_url
            .filter(|url| !url.trim().is_empty())
            .unwrap_or_else(|| "http://127.0.0.1:5000".to_owned());
        Ok(Self {
            translation_service: LibreTranslateTranslationService {
                api_key: config.api_key.trim().to_owned(),
                http: HttpClient::new(base_url, Default::default()),
            },
        })
    }
}

pub fn map_libretranslate_language(code: &str) -> String {
    match code.trim() {
        "" | "auto" => "auto".to_owned(),
        "zh-Hans" | "zh-CN" => "zh".to_owned(),
        "zh-Hant" | "zh-TW" => "zt".to_owned(),
        other => other.to_owned(),
    }
}

pub fn parse_libretranslate_response(value: &Value) -> Result<String, TranslationError> {
    value
        .get("translatedText")
        .and_then(Value::as_str)
        .map(str::to_owned)
        .ok_or_else(|| TranslationError::SerializationError("missing translatedText".to_owned()))
}

impl LibreTranslateTranslationService {
    async fn languages(&self) -> Result<(), TranslationError> {
        let builder = self.http.get("/languages");
        let response = self
            .http
            .execute(builder)
            .await
            .map_err(TranslationError::from_network_error)?;
        TranslationError::from_response("libretranslate", response)
            .await
            .map(|_| ())
    }
}

#[async_trait(?Send)]
impl TranslationService for LibreTranslateTranslationService {
    async fn translate(
        &self,
        request: TranslateRequest,
    ) -> Result<TranslateResponse, TranslationError> {
        let target = request.target_language.ok_or_else(|| {
            TranslationError::InvalidRequest("target_language is required".to_owned())
        })?;
        let mut body = json!({
            "q": request.text,
            "source": map_libretranslate_language(request.source_language.as_deref().unwrap_or("auto")),
            "target": map_libretranslate_language(&target),
            "format": "text",
        });
        if !self.api_key.is_empty() {
            body["api_key"] = json!(self.api_key);
        }
        let builder = self.http.post("/translate").json(&body);
        let response = self
            .http
            .execute(builder)
            .await
            .map_err(TranslationError::from_network_error)?;
        let response = TranslationError::from_response_redacting(
            "libretranslate",
            response,
            &[self.api_key.as_str()],
        )
        .await?;
        let data: Value = response
            .json()
            .await
            .map_err(|error| TranslationError::SerializationError(error.to_string()))?;
        let text = parse_libretranslate_response(&data)?;
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
impl Provider for LibreTranslateProvider {
    fn name(&self) -> &'static str {
        "libretranslate"
    }

    fn translation(&self) -> Option<&dyn TranslationService> {
        Some(&self.translation_service)
    }

    async fn list_models(&self) -> Result<Vec<String>, linguaray_core::LlmError> {
        self.translation_service
            .languages()
            .await
            .map(|_| Vec::new())
            .map_err(|error| linguaray_core::LlmError::NetworkError(error.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_translated_text() {
        let value = serde_json::json!({ "translatedText": "Hallo" });
        assert_eq!(parse_libretranslate_response(&value).unwrap(), "Hallo");
    }

    #[test]
    fn maps_chinese_and_auto() {
        assert_eq!(map_libretranslate_language("zh-Hans"), "zh");
        assert_eq!(map_libretranslate_language("zh-Hant"), "zt");
        assert_eq!(map_libretranslate_language("auto"), "auto");
    }
}
