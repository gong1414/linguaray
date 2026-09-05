use async_trait::async_trait;
use base64::Engine;
use linguaray_core::{
    DetectLanguageRequest, DetectLanguageResponse, DictionaryError, DictionaryService,
    LookUpRequest, LookUpResponse, OcrError, OcrService, Provider, RecognizeTextRequest,
    RecognizeTextResponse, TranslateRequest, TranslateResponse, TranslationError,
    TranslationService,
};
#[cfg(target_os = "macos")]
use linguaray_core::{
    RecognizedRect, TextDetection, TextRecognition, TextTranslation, WordDefinition,
    WordPronunciation,
};

#[cfg(target_os = "macos")]
#[path = "system/macos.rs"]
mod platform;

#[cfg(target_os = "windows")]
#[path = "system/windows.rs"]
mod platform;

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
#[path = "system/unsupported.rs"]
mod platform;

// ── System Translation Service ───────────────────────────────────────────

pub struct SystemTranslationService;

#[async_trait(?Send)]
impl TranslationService for SystemTranslationService {
    async fn detect_language(
        &self,
        request: DetectLanguageRequest,
    ) -> Result<DetectLanguageResponse, TranslationError> {
        platform::detect_language(request).await
    }

    async fn translate(
        &self,
        request: TranslateRequest,
    ) -> Result<TranslateResponse, TranslationError> {
        platform::translate(&request).await
    }
}

// ── System Dictionary Service ─────────────────────────────────────────────

pub struct SystemDictionaryService;

#[async_trait(?Send)]
impl DictionaryService for SystemDictionaryService {
    async fn look_up(&self, request: LookUpRequest) -> Result<LookUpResponse, DictionaryError> {
        platform::look_up(&request).await
    }
}

// ── Provider ───────────────────────────────────────────────────────────────

pub struct SystemProvider {
    translation_service: SystemTranslationService,
    dictionary_service: SystemDictionaryService,
}

impl SystemProvider {
    pub fn new() -> Result<Self, String> {
        Ok(Self {
            translation_service: SystemTranslationService,
            dictionary_service: SystemDictionaryService,
        })
    }
}

impl Provider for SystemProvider {
    fn name(&self) -> &'static str {
        "system"
    }

    fn dictionary(&self) -> Option<&dyn DictionaryService> {
        // Windows ships system OCR only. Advertising dictionary look-up here
        // would present a service that always returns UnsupportedMethod.
        #[cfg(target_os = "macos")]
        {
            Some(&self.dictionary_service)
        }
        #[cfg(not(target_os = "macos"))]
        {
            let _ = &self.dictionary_service;
            None
        }
    }

    fn translation(&self) -> Option<&dyn TranslationService> {
        #[cfg(target_os = "macos")]
        {
            Some(&self.translation_service)
        }
        #[cfg(not(target_os = "macos"))]
        {
            let _ = &self.translation_service;
            None
        }
    }

    fn ocr(&self) -> Option<&dyn OcrService> {
        Some(self)
    }
}

#[async_trait(?Send)]
impl OcrService for SystemProvider {
    async fn recognize_text(
        &self,
        request: RecognizeTextRequest,
    ) -> Result<RecognizeTextResponse, OcrError> {
        let base64_image = match (&request.base64_image, &request.image_path) {
            (Some(base64), _) => base64.clone(),
            (None, Some(path)) => {
                let bytes = std::fs::read(path).map_err(|e| {
                    OcrError::InvalidRequest(format!("failed to read image file '{path}': {e}"))
                })?;
                base64::engine::general_purpose::STANDARD.encode(&bytes)
            }
            (None, None) => {
                return Err(OcrError::InvalidRequest(
                    "either base64_image or image_path must be provided".to_owned(),
                ));
            }
        };

        platform::recognize_text(&base64_image)
    }
}
