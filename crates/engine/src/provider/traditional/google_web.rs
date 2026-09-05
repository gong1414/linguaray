//! Unofficial Google Translate web endpoint. Experimental; not an official API.

use async_trait::async_trait;
use linguaray_core::{
    DetectLanguageRequest, DetectLanguageResponse, Provider, TextDetection, TextTranslation,
    TranslateRequest, TranslateResponse, TranslationError, TranslationService,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::common::{ClassifyHttpResponse, HttpClient};

const USER_AGENT: &str = "LinguaRay/1.0";

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(default)]
pub struct GoogleWebProviderConfig {
    #[serde(rename = "baseUrl", alias = "base_url")]
    pub base_url: Option<String>,
}

pub struct GoogleWebProvider {
    translation_service: GoogleWebTranslationService,
}

struct GoogleWebTranslationService {
    http: HttpClient,
}

impl GoogleWebProvider {
    pub fn new(config: GoogleWebProviderConfig) -> Result<Self, String> {
        let base_url = config
            .base_url
            .filter(|url| !url.trim().is_empty())
            .unwrap_or_else(|| "https://translate.google.com".to_owned());
        Ok(Self {
            translation_service: GoogleWebTranslationService {
                http: HttpClient::proxy_aware(base_url)?,
            },
        })
    }
}

pub fn map_google_web_language(code: &str) -> String {
    match code.trim() {
        "" | "auto" => "auto".to_owned(),
        "zh-Hans" | "zh-CN" => "zh-CN".to_owned(),
        "zh-Hant" | "zh-TW" => "zh-TW".to_owned(),
        other => other.to_owned(),
    }
}

pub fn parse_google_web_response(
    value: &Value,
) -> Result<(String, Option<String>), TranslationError> {
    let segments = value.get(0).and_then(Value::as_array).ok_or_else(|| {
        TranslationError::SerializationError("missing translation segments".to_owned())
    })?;
    let mut text = String::new();
    for segment in segments {
        if let Some(part) = segment.get(0).and_then(Value::as_str) {
            text.push_str(part);
        }
    }
    let detected = value.get(2).and_then(Value::as_str).map(str::to_owned);
    Ok((text, detected))
}

#[async_trait(?Send)]
impl TranslationService for GoogleWebTranslationService {
    async fn detect_language(
        &self,
        request: DetectLanguageRequest,
    ) -> Result<DetectLanguageResponse, TranslationError> {
        let text = request
            .texts
            .into_iter()
            .next()
            .ok_or_else(|| TranslationError::InvalidRequest("texts is required".to_owned()))?;
        let translated = self
            .translate(TranslateRequest {
                source_language: Some("auto".to_owned()),
                target_language: Some("en".to_owned()),
                text: text.clone(),
            })
            .await?;
        let detected = translated
            .translations
            .first()
            .and_then(|item| item.detected_source_language.clone())
            .unwrap_or_default();
        Ok(DetectLanguageResponse {
            detections: Some(vec![TextDetection {
                detected_language: detected,
                text,
            }]),
        })
    }

    async fn translate(
        &self,
        request: TranslateRequest,
    ) -> Result<TranslateResponse, TranslationError> {
        let target = request.target_language.ok_or_else(|| {
            TranslationError::InvalidRequest("target_language is required".to_owned())
        })?;
        let source = map_google_web_language(request.source_language.as_deref().unwrap_or("auto"));
        let target = map_google_web_language(&target);
        let builder = self
            .http
            .get("/translate_a/single")
            .header("User-Agent", USER_AGENT)
            .query(&[
                ("client", "gtx"),
                ("sl", source.as_str()),
                ("tl", target.as_str()),
                ("hl", target.as_str()),
                ("dt", "t"),
                ("ie", "UTF-8"),
                ("oe", "UTF-8"),
                ("q", request.text.as_str()),
            ]);
        let response = self
            .http
            .execute(builder)
            .await
            .map_err(TranslationError::from_network_error)?;
        let response = TranslationError::from_response("google_web", response).await?;
        let data: Value = response
            .json()
            .await
            .map_err(|error| TranslationError::SerializationError(error.to_string()))?;
        let (text, detected) = parse_google_web_response(&data)?;
        Ok(TranslateResponse {
            translations: vec![TextTranslation {
                detected_source_language: detected,
                text,
                audio_url: None,
            }],
        })
    }
}

impl Provider for GoogleWebProvider {
    fn name(&self) -> &'static str {
        "google_web"
    }

    fn translation(&self) -> Option<&dyn TranslationService> {
        Some(&self.translation_service)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_chinese_variants() {
        assert_eq!(map_google_web_language("zh-Hans"), "zh-CN");
        assert_eq!(map_google_web_language("zh-Hant"), "zh-TW");
        assert_eq!(map_google_web_language("auto"), "auto");
        assert_eq!(map_google_web_language("en"), "en");
    }

    #[test]
    fn concatenates_segments_and_reads_detected_language() {
        let value = serde_json::json!([
            [
                ["Hello ", "Hallo", null, null, 1],
                ["world", "Welt", null, null, 1]
            ],
            null,
            "de"
        ]);
        let (text, detected) = parse_google_web_response(&value).expect("parse");
        assert_eq!(text, "Hello world");
        assert_eq!(detected.as_deref(), Some("de"));
    }

    #[test]
    fn missing_segments_is_an_error() {
        let value = serde_json::json!({"error": "nope"});
        assert!(parse_google_web_response(&value).is_err());
    }
}
