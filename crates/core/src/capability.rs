use std::sync::mpsc::Receiver;

use async_trait::async_trait;
use thiserror::Error;

use crate::{
    ChatRequest, ChatResponse, DetectLanguageRequest, DetectLanguageResponse, LanguagePair,
    LookUpRequest, LookUpResponse, RecognizeTextRequest, RecognizeTextResponse, StreamChunk,
    TranslateRequest, TranslateResponse,
};

macro_rules! capability_error {
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
            pub fn from_network_error(error: impl ToString) -> Self {
                Self::NetworkError(error.to_string())
            }

            pub fn from_http_status(provider: &'static str, status: u16, message: String) -> Self {
                match status {
                    401 | 403 => Self::AuthError(message),
                    429 => Self::RateLimitError(message),
                    400..=499 => Self::InvalidRequest(message),
                    _ => Self::NetworkError(format!("{provider}: {message}")),
                }
            }
        }
    };
}

capability_error!(DictionaryError);
capability_error!(OcrError);
capability_error!(TranslationError);

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
        self.rx
            .recv()
            .map_err(|error| LlmError::StreamError(error.to_string()))
    }

    pub fn try_recv(&self) -> Option<StreamChunk> {
        self.rx.try_recv().ok()
    }
}

#[async_trait(?Send)]
pub trait TranslationService: Send + Sync {
    async fn get_supported_language_pairs(&self) -> Result<Vec<LanguagePair>, TranslationError> {
        Err(TranslationError::UnsupportedMethod(
            "get_supported_language_pairs",
        ))
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
    use super::TranslationError;

    #[test]
    fn http_status_maps_to_capability_errors() {
        assert!(matches!(
            TranslationError::from_http_status("demo", 401, "nope".into()),
            TranslationError::AuthError(_)
        ));
        assert!(matches!(
            TranslationError::from_http_status("demo", 429, "slow".into()),
            TranslationError::RateLimitError(_)
        ));
        assert!(matches!(
            TranslationError::from_http_status("demo", 400, "bad".into()),
            TranslationError::InvalidRequest(_)
        ));
        match TranslationError::from_http_status("demo", 500, "boom".into()) {
            TranslationError::NetworkError(message) => {
                assert!(message.contains("demo"));
                assert!(message.contains("boom"));
            }
            other => panic!("{other:?}"),
        }
    }
}
