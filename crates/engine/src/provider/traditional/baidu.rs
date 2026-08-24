#![cfg_attr(not(feature = "baidu"), allow(dead_code))]

use crate::common::HttpClient;
use async_trait::async_trait;
use base64::Engine as _;
use linguaray_core::{
    DetectLanguageRequest, DetectLanguageResponse, OcrError, OcrService, Provider,
    RecognizeTextRequest, RecognizeTextResponse, RecognizedRect, TextDetection, TextRecognition,
    TextTranslation, TranslateRequest, TranslateResponse, TranslationError, TranslationService,
};
use rand::random;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Deserialize, PartialEq, Serialize)]
pub struct BaiduProviderConfig {
    #[serde(default)]
    #[serde(rename = "appId", alias = "app_id")]
    pub app_id: String,
    #[serde(default)]
    #[serde(rename = "appKey", alias = "app_key")]
    pub app_key: String,
    #[serde(default)]
    #[serde(rename = "apiKey", alias = "api_key")]
    pub api_key: String,
    #[serde(default)]
    #[serde(rename = "secretKey", alias = "secret_key")]
    pub secret_key: String,
    #[serde(rename = "baseUrl", alias = "base_url")]
    pub base_url: Option<String>,
}

pub struct BaiduProvider {
    translation_service: Option<BaiduTranslationService>,
    ocr_service: Option<BaiduOcrService>,
}

struct BaiduTranslationService {
    app_id: String,
    app_key: String,
    http: HttpClient,
}

struct BaiduOcrService {
    api_key: String,
    secret_key: String,
    http: HttpClient,
}

impl BaiduProvider {
    pub fn new(config: BaiduProviderConfig) -> Result<Self, String> {
        let has_translation = !config.app_id.trim().is_empty() && !config.app_key.trim().is_empty();
        let has_ocr = !config.api_key.trim().is_empty() && !config.secret_key.trim().is_empty();
        if !has_translation && !has_ocr {
            return Err(
                "provide appId/appKey for translation or apiKey/secretKey for OCR".to_owned(),
            );
        }
        let base_url = config.base_url.clone();
        let translation_http = if has_translation {
            Some(HttpClient::proxy_aware(base_url.clone().unwrap_or_else(
                || "https://fanyi-api.baidu.com".to_owned(),
            ))?)
        } else {
            None
        };
        let ocr_http = if has_ocr {
            Some(HttpClient::proxy_aware(
                base_url.unwrap_or_else(|| "https://aip.baidubce.com".to_owned()),
            )?)
        } else {
            None
        };
        Ok(Self {
            translation_service: translation_http.map(|http| BaiduTranslationService {
                app_id: config.app_id.clone(),
                app_key: config.app_key.clone(),
                http,
            }),
            ocr_service: ocr_http.map(|http| BaiduOcrService {
                api_key: config.api_key,
                secret_key: config.secret_key,
                http,
            }),
        })
    }
}

#[async_trait(?Send)]
impl TranslationService for BaiduTranslationService {
    async fn detect_language(
        &self,
        request: DetectLanguageRequest,
    ) -> Result<DetectLanguageResponse, TranslationError> {
        let text = request
            .texts
            .into_iter()
            .next()
            .ok_or_else(|| TranslationError::InvalidRequest("texts is required".to_owned()))?;
        let salt = (random::<u32>() % 999_999).to_string();
        let sign = format!(
            "{:x}",
            md5::compute(format!("{}{}{}{}", self.app_id, text, salt, self.app_key))
        );

        let response = self.http.post("/api/trans/vip/language").query(&[
            ("q", text.as_str()),
            ("appid", self.app_id.as_str()),
            ("salt", salt.as_str()),
            ("sign", sign.as_str()),
        ]);
        let response = self
            .http
            .execute(response)
            .await
            .map_err(TranslationError::from_network_error)?;
        let response = TranslationError::from_response("baidu", response).await?;
        let data: Value = response
            .json()
            .await
            .map_err(|error| TranslationError::SerializationError(error.to_string()))?;

        ensure_baidu_success(&data)?;
        let detected = data["data"]["src"].as_str().ok_or_else(|| {
            TranslationError::SerializationError("missing data.src in Baidu response".to_owned())
        })?;

        Ok(DetectLanguageResponse {
            detections: Some(vec![TextDetection {
                detected_language: detected.to_owned(),
                text,
            }]),
        })
    }

    async fn translate(
        &self,
        request: TranslateRequest,
    ) -> Result<TranslateResponse, TranslationError> {
        let salt = (random::<u32>() % 999_999).to_string();
        let sign = format!(
            "{:x}",
            md5::compute(format!(
                "{}{}{}{}",
                self.app_id, request.text, salt, self.app_key
            ))
        );
        let from = baidu_language_code(request.source_language.as_deref()).unwrap_or("auto");
        let to = baidu_language_code(request.target_language.as_deref()).ok_or_else(|| {
            TranslationError::InvalidRequest("target_language is required".to_owned())
        })?;

        let response = self.http.post("/api/trans/vip/translate").query(&[
            ("q", request.text.as_str()),
            ("from", from),
            ("to", to),
            ("appid", self.app_id.as_str()),
            ("salt", salt.as_str()),
            ("sign", sign.as_str()),
            ("dict", "0"),
        ]);
        let response = self
            .http
            .execute(response)
            .await
            .map_err(TranslationError::from_network_error)?;
        let response = TranslationError::from_response("baidu", response).await?;
        let data: Value = response
            .json()
            .await
            .map_err(|error| TranslationError::SerializationError(error.to_string()))?;

        ensure_baidu_success(&data)?;
        let translations = data["trans_result"]
            .as_array()
            .ok_or_else(|| {
                TranslationError::SerializationError(
                    "missing trans_result in Baidu response".to_owned(),
                )
            })?
            .iter()
            .filter_map(|item| item["dst"].as_str())
            .map(|text| TextTranslation {
                detected_source_language: None,
                text: text.to_owned(),
                audio_url: None,
            })
            .collect();

        Ok(TranslateResponse { translations })
    }
}

impl Provider for BaiduProvider {
    fn name(&self) -> &'static str {
        "baidu"
    }

    fn translation(&self) -> Option<&dyn TranslationService> {
        self.translation_service
            .as_ref()
            .map(|service| service as &dyn TranslationService)
    }

    fn ocr(&self) -> Option<&dyn OcrService> {
        self.ocr_service
            .as_ref()
            .map(|service| service as &dyn OcrService)
    }
}

#[async_trait(?Send)]
impl OcrService for BaiduOcrService {
    async fn recognize_text(
        &self,
        request: RecognizeTextRequest,
    ) -> Result<RecognizeTextResponse, OcrError> {
        let image = image_base64(request)?;
        let token_response = self
            .http
            .client()
            .post(format!("{}/oauth/2.0/token", self.http.base_url()))
            .query(&[
                ("grant_type", "client_credentials"),
                ("client_id", self.api_key.as_str()),
                ("client_secret", self.secret_key.as_str()),
            ]);
        let token_response = self
            .http
            .execute(token_response)
            .await
            .map_err(OcrError::from_network_error)?;
        let token_response = OcrError::from_response("baidu-ocr-token", token_response).await?;
        let token: Value = token_response
            .json()
            .await
            .map_err(|error| OcrError::SerializationError(error.to_string()))?;
        let access_token = token["access_token"].as_str().ok_or_else(|| {
            OcrError::NetworkError(baidu_ocr_error(&token, "missing access_token"))
        })?;

        let response = self
            .http
            .client()
            .post(format!("{}/rest/2.0/ocr/v1/general", self.http.base_url()))
            .query(&[("access_token", access_token)])
            .form(&[("image", image.as_str()), ("detect_direction", "true")]);
        let response = self
            .http
            .execute(response)
            .await
            .map_err(OcrError::from_network_error)?;
        let response = OcrError::from_response("baidu-ocr", response).await?;
        let data: Value = response
            .json()
            .await
            .map_err(|error| OcrError::SerializationError(error.to_string()))?;
        if data.get("error_code").is_some() {
            return Err(OcrError::NetworkError(baidu_ocr_error(
                &data,
                "recognition failed",
            )));
        }

        let recognitions = data["words_result"]
            .as_array()
            .ok_or_else(|| {
                OcrError::SerializationError(
                    "missing words_result in Baidu OCR response".to_owned(),
                )
            })?
            .iter()
            .filter_map(|item| {
                let text = item["words"].as_str()?.trim();
                if text.is_empty() {
                    return None;
                }
                let location = item["location"].as_object();
                Some(TextRecognition {
                    text: text.to_owned(),
                    recognized_rect: location.map(|value| RecognizedRect {
                        x: value
                            .get("left")
                            .and_then(Value::as_f64)
                            .unwrap_or_default(),
                        y: value.get("top").and_then(Value::as_f64).unwrap_or_default(),
                        width: value
                            .get("width")
                            .and_then(Value::as_f64)
                            .unwrap_or_default(),
                        height: value
                            .get("height")
                            .and_then(Value::as_f64)
                            .unwrap_or_default(),
                        top: value.get("top").and_then(Value::as_f64),
                        right: None,
                        bottom: None,
                        left: value.get("left").and_then(Value::as_f64),
                    }),
                })
            })
            .collect::<Vec<_>>();
        let text = recognitions
            .iter()
            .map(|item| item.text.as_str())
            .collect::<Vec<_>>()
            .join("\n");
        Ok(RecognizeTextResponse {
            text,
            recognitions: Some(recognitions),
        })
    }
}

fn image_base64(request: RecognizeTextRequest) -> Result<String, OcrError> {
    match (request.base64_image, request.image_path) {
        (Some(image), _) => Ok(image),
        (None, Some(path)) => std::fs::read(&path)
            .map(|bytes| base64::engine::general_purpose::STANDARD.encode(bytes))
            .map_err(|error| {
                OcrError::InvalidRequest(format!("failed to read image file '{path}': {error}"))
            }),
        (None, None) => Err(OcrError::InvalidRequest(
            "either base64_image or image_path must be provided".to_owned(),
        )),
    }
}

fn baidu_ocr_error(data: &Value, fallback: &str) -> String {
    let code = data["error_code"]
        .as_i64()
        .map(|value| value.to_string())
        .or_else(|| data["error"].as_str().map(ToOwned::to_owned))
        .unwrap_or_else(|| "unknown".to_owned());
    let message = data["error_msg"]
        .as_str()
        .or_else(|| data["error_description"].as_str())
        .unwrap_or(fallback);
    format!("baidu OCR: {code}: {message}")
}

fn baidu_language_code(language: Option<&str>) -> Option<&str> {
    match language {
        Some("es") => Some("spa"),
        Some("fr") => Some("fra"),
        Some("ja") => Some("jp"),
        Some("ko") => Some("kor"),
        Some(other) => Some(other),
        None => None,
    }
}

fn ensure_baidu_success(data: &Value) -> Result<(), TranslationError> {
    if let Some(code) = data["error_code"].as_i64() {
        if code != 0 {
            let message = data["error_msg"].as_str().unwrap_or("unknown error");
            return Err(TranslationError::NetworkError(format!(
                "baidu: {code}: {message}"
            )));
        }
    }

    if let Some(code) = data["error_code"].as_str() {
        if code != "0" {
            let message = data["error_msg"].as_str().unwrap_or("unknown error");
            return Err(TranslationError::NetworkError(format!(
                "baidu: {code}: {message}"
            )));
        }
    }

    Ok(())
}
