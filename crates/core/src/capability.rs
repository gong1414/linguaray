use std::sync::mpsc::Receiver;

use async_trait::async_trait;
use reqwest::{Response, StatusCode};
use thiserror::Error;

use crate::{
    ChatRequest, ChatResponse, DetectLanguageRequest, DetectLanguageResponse, LanguagePair,
    LookUpRequest, LookUpResponse, RecognizeTextRequest, RecognizeTextResponse, StreamChunk,
    TranslateRequest, TranslateResponse,
};

async fn failed_response(response: Response, secrets: &[&str]) -> (StatusCode, String) {
    let status = response.status();
    let mut body = response.text().await.unwrap_or_default();
    for secret in secrets.iter().copied().filter(|value| !value.is_empty()) {
        body = body.replace(secret, "[redacted]");
    }
    let message = if body.is_empty() {
        status.to_string()
    } else {
        body.chars().take(512).collect()
    };
    (status, message)
}

macro_rules! http_error {
    ($name:ident) => {
        #[derive(Debug, Error, Clone)]
        pub enum $name {
            #[error("unsupported method: {0}")]
            UnsupportedMethod(&'static str),
            #[error("configuration error: {0}")]
            ConfigError(String),
            #[error("authentication failed: {0}")]
            AuthError(String),
            #[error("rate limited: {0}")]
            RateLimitError(String),
            #[error("invalid request: {0}")]
            InvalidRequest(String),
            #[error("network error: {0}")]
            NetworkError(String),
            #[error("serialization error: {0}")]
            SerializationError(String),
        }

        impl $name {
            pub fn from_network_error(error: reqwest::Error) -> Self {
                Self::NetworkError(error.to_string())
            }

            async fn classify_response(
                provider: &'static str,
                response: Response,
                secrets: &[&str],
            ) -> Result<Response, Self> {
                if response.status().is_success() {
                    return Ok(response);
                }
                let (status, message) = failed_response(response, secrets).await;
                Err(match status {
                    StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN => Self::AuthError(message),
                    StatusCode::TOO_MANY_REQUESTS => Self::RateLimitError(message),
                    value if value.is_client_error() => Self::InvalidRequest(message),
                    _ => Self::NetworkError(format!("{provider}: {message}")),
                })
            }

            pub async fn from_response(
                provider: &'static str,
                response: Response,
            ) -> Result<Response, Self> {
                Self::classify_response(provider, response, &[]).await
            }
        }
    };
}

http_error!(DictionaryError);
http_error!(OcrError);
http_error!(TranslationError);

impl TranslationError {
    pub async fn from_response_redacting(
        provider: &'static str,
        response: Response,
        secrets: &[&str],
    ) -> Result<Response, Self> {
        Self::classify_response(provider, response, secrets).await
    }
}

#[derive(Debug, Error, Clone)]
pub enum LlmError {
    #[error("configuration error: {0}")]
    ConfigError(String),
    #[error("authentication failed: {0}")]
    AuthError(String),
    #[error("rate limited: {0}")]
    RateLimitError(String),
    #[error("invalid request: {0}")]
    InvalidRequest(String),
    #[error("network error: {0}")]
    NetworkError(String),
    #[error("serialization error: {0}")]
    SerializationError(String),
    #[error("stream error: {0}")]
    StreamError(String),
    #[error("unsupported operation: {0}")]
    UnsupportedOperation(String),
}

pub struct LlmStreamReceiver {
    pub rx: Receiver<StreamChunk>,
}

impl LlmStreamReceiver {
    pub fn recv(&self) -> Result<StreamChunk, LlmError> {
        self.rx.recv().map_err(|error| LlmError::StreamError(error.to_string()))
    }

    pub fn try_recv(&self) -> Option<StreamChunk> {
        self.rx.try_recv().ok()
    }
}

#[async_trait(?Send)]
pub trait TranslationService: Send + Sync {
    async fn get_supported_language_pairs(&self) -> Result<Vec<LanguagePair>, TranslationError> {
        Err(TranslationError::UnsupportedMethod("get_supported_language_pairs"))
    }

    async fn detect_language(
        &self,
        _request: DetectLanguageRequest,
    ) -> Result<DetectLanguageResponse, TranslationError> {
        Err(TranslationError::UnsupportedMethod("detect_language"))
    }

    async fn translate(
        &self,
        request: TranslateRequest,
    ) -> Result<TranslateResponse, TranslationError>;
}

#[async_trait(?Send)]
pub trait DictionaryService: Send + Sync {
    async fn look_up(&self, request: LookUpRequest) -> Result<LookUpResponse, DictionaryError>;
}

#[async_trait(?Send)]
pub trait OcrService: Send + Sync {
    async fn recognize_text(
        &self,
        request: RecognizeTextRequest,
    ) -> Result<RecognizeTextResponse, OcrError>;
}

#[async_trait]
pub trait LlmService: Send + Sync {
    fn provider_name(&self) -> &'static str;
    fn available_models(&self) -> Vec<String>;
    async fn chat(&self, request: ChatRequest) -> Result<ChatResponse, LlmError>;
    async fn chat_stream(&self, request: ChatRequest) -> Result<LlmStreamReceiver, LlmError>;
}

#[cfg(test)]
mod tests {
    use super::failed_response;

    #[test]
    fn redaction_is_bounded_and_ignores_empty_values() {
        let mut message = "Bearer secret".to_owned();
        for _ in 0..600 {
            message.push('x');
        }
        let sanitized = ["secret", ""].into_iter().fold(message, |value, secret| {
            if secret.is_empty() { value } else { value.replace(secret, "[redacted]") }
        });
        assert!(sanitized.contains("[redacted]"));
        assert!(!sanitized.contains("Bearer secret"));
        let _ = failed_response;
    }
}
