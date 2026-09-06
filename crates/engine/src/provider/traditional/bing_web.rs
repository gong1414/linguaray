//! Unofficial Microsoft Edge translator endpoint. Experimental.

use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::common::ClassifyHttpResponse;
use async_trait::async_trait;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine as _;
use linguaray_core::{
    DetectLanguageRequest, DetectLanguageResponse, Provider, TextDetection, TextTranslation,
    TranslateRequest, TranslateResponse, TranslationError, TranslationService,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

const USER_AGENT: &str = "LinguaRay/1.0";
const AUTH_URL: &str = "https://edge.microsoft.com/translate/auth";
const TRANSLATE_URL: &str = "https://api-edge.cognitive.microsofttranslator.com/translate";

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(default)]
pub struct BingWebProviderConfig {}

pub struct BingWebProvider {
    translation_service: BingWebTranslationService,
}

struct BingWebTranslationService {
    client: reqwest::Client,
    token: Mutex<Option<CachedToken>>,
}

#[derive(Clone)]
struct CachedToken {
    value: String,
    exp: u64,
}

impl BingWebProvider {
    pub fn new(_config: BingWebProviderConfig) -> Result<Self, String> {
        Ok(Self {
            translation_service: BingWebTranslationService {
                client: crate::common::build_http_client()?,
                token: Mutex::new(None),
            },
        })
    }
}

pub fn map_bing_web_language(code: &str) -> String {
    match code.trim() {
        "" | "auto" => String::new(),
        "zh-Hans" | "zh-CN" => "zh-Hans".to_owned(),
        "zh-Hant" | "zh-TW" => "zh-Hant".to_owned(),
        other => other.to_owned(),
    }
}

pub fn jwt_expiry_unix(token: &str) -> Option<u64> {
    let payload = token.split('.').nth(1)?;
    let decoded = URL_SAFE_NO_PAD.decode(payload).ok()?;
    let value: Value = serde_json::from_slice(&decoded).ok()?;
    value.get("exp").and_then(Value::as_u64)
}

fn cached_token_still_valid(token: &CachedToken, now_unix: u64) -> bool {
    now_unix + 60 < token.exp
}

pub fn parse_bing_translate_response(
    value: &Value,
) -> Result<(String, Option<String>), TranslationError> {
    let first = value
        .as_array()
        .and_then(|items| items.first())
        .ok_or_else(|| {
            TranslationError::SerializationError("missing Bing translations".to_owned())
        })?;
    let text = first
        .get("translations")
        .and_then(Value::as_array)
        .and_then(|items| items.first())
        .and_then(|item| item.get("text"))
        .and_then(Value::as_str)
        .ok_or_else(|| TranslationError::SerializationError("missing Bing text".to_owned()))?;
    let detected = first
        .get("detectedLanguage")
        .and_then(|item| item.get("language"))
        .and_then(Value::as_str)
        .map(str::to_owned);
    Ok((text.to_owned(), detected))
}

impl BingWebTranslationService {
    fn now_unix() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_secs())
            .unwrap_or(0)
    }

    async fn bearer_token(&self) -> Result<String, TranslationError> {
        {
            let guard = self.token.lock().unwrap_or_else(|error| error.into_inner());
            if let Some(cached) = guard.as_ref() {
                if cached_token_still_valid(cached, Self::now_unix()) {
                    return Ok(cached.value.clone());
                }
            }
        }

        let response = self
            .client
            .get(AUTH_URL)
            .header("User-Agent", USER_AGENT)
            .send()
            .await
            .map_err(TranslationError::from_network_error)?;
        let response = TranslationError::from_response("bing_web", response).await?;
        let token = response
            .text()
            .await
            .map_err(|error| TranslationError::SerializationError(error.to_string()))?
            .trim()
            .to_owned();
        if token.is_empty() {
            return Err(TranslationError::AuthError(
                "empty Bing auth token".to_owned(),
            ));
        }
        if let Some(exp) = jwt_expiry_unix(&token) {
            if let Ok(mut guard) = self.token.lock() {
                *guard = Some(CachedToken {
                    value: token.clone(),
                    exp,
                });
            }
        }
        Ok(token)
    }
}

#[async_trait(?Send)]
impl TranslationService for BingWebTranslationService {
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
        let token = self.bearer_token().await?;
        let from = map_bing_web_language(request.source_language.as_deref().unwrap_or("auto"));
        let to = map_bing_web_language(&target);
        let response = self
            .client
            .post(TRANSLATE_URL)
            .header("User-Agent", USER_AGENT)
            .header("Content-Type", "application/json")
            .header("Authorization", format!("Bearer {token}"))
            .query(&[
                ("api-version", "3.0"),
                ("from", from.as_str()),
                ("to", to.as_str()),
            ])
            .json(&json!([{ "Text": request.text }]))
            .send()
            .await
            .map_err(TranslationError::from_network_error)?;
        let response = TranslationError::from_response("bing_web", response).await?;
        let data: Value = response
            .json()
            .await
            .map_err(|error| TranslationError::SerializationError(error.to_string()))?;
        let (text, detected) = parse_bing_translate_response(&data)?;
        Ok(TranslateResponse {
            translations: vec![TextTranslation {
                detected_source_language: detected,
                text,
                audio_url: None,
            }],
        })
    }
}

impl Provider for BingWebProvider {
    fn name(&self) -> &'static str {
        "bing_web"
    }

    fn translation(&self) -> Option<&dyn TranslationService> {
        Some(&self.translation_service)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_jwt(exp: u64) -> String {
        let payload = URL_SAFE_NO_PAD.encode(format!(r#"{{"exp":{exp}}}"#));
        format!("aaa.{payload}.sig")
    }

    #[test]
    fn reads_jwt_exp_and_caches_until_60s_before() {
        let token = sample_jwt(1_700_000_060);
        assert_eq!(jwt_expiry_unix(&token), Some(1_700_000_060));
        let cached = CachedToken {
            value: token,
            exp: 1_700_000_060,
        };
        assert!(cached_token_still_valid(&cached, 1_699_999_999));
        assert!(!cached_token_still_valid(&cached, 1_700_000_000));
    }

    #[test]
    fn parses_first_translation_text() {
        let value = serde_json::json!([{
            "detectedLanguage": {"language": "en", "score": 1.0},
            "translations": [{"text": "Hallo", "to": "de"}]
        }]);
        let (text, detected) = parse_bing_translate_response(&value).expect("parse");
        assert_eq!(text, "Hallo");
        assert_eq!(detected.as_deref(), Some("en"));
    }
}
