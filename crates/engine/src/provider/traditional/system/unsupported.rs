use super::*;

pub fn recognize_text(_base64_image: &str) -> Result<RecognizeTextResponse, OcrError> {
    Err(OcrError::UnsupportedMethod(
        "system OCR is not supported on this platform",
    ))
}

pub async fn detect_language(
    _request: DetectLanguageRequest,
) -> Result<DetectLanguageResponse, TranslationError> {
    Err(TranslationError::UnsupportedMethod(
        "system language detection is not supported on this platform",
    ))
}

pub async fn translate(_request: &TranslateRequest) -> Result<TranslateResponse, TranslationError> {
    Err(TranslationError::UnsupportedMethod(
        "system translation is not supported on this platform",
    ))
}

pub async fn look_up(_request: &LookUpRequest) -> Result<LookUpResponse, DictionaryError> {
    Err(DictionaryError::UnsupportedMethod(
        "system dictionary is not supported on this platform",
    ))
}
