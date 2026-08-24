use super::*;
use windows::{
    Graphics::Imaging::BitmapDecoder,
    Media::Ocr::OcrEngine,
    Storage::Streams::{DataWriter, InMemoryRandomAccessStream},
};

pub fn recognize_text(base64_image: &str) -> Result<RecognizeTextResponse, OcrError> {
    let image_bytes = base64::engine::general_purpose::STANDARD
        .decode(base64_image)
        .map_err(|e| OcrError::InvalidRequest(format!("base64 decode failed: {e}")))?;

    // Create an in-memory stream and write image bytes into it
    let stream = InMemoryRandomAccessStream::new()
        .map_err(|e| OcrError::NetworkError(format!("create stream failed: {e}")))?;

    let writer = DataWriter::CreateDataWriter(&stream)
        .map_err(|e| OcrError::NetworkError(format!("create data writer failed: {e}")))?;

    writer
        .WriteBytes(&image_bytes)
        .map_err(|e| OcrError::NetworkError(format!("write bytes failed: {e}")))?;

    writer
        .StoreAsync()
        .map_err(|e| OcrError::NetworkError(format!("store async failed: {e}")))?
        .get()
        .map_err(|e| OcrError::NetworkError(format!("store async wait failed: {e}")))?;

    // Set stream position to beginning
    stream
        .Seek(0)
        .map_err(|e| OcrError::NetworkError(format!("seek failed: {e}")))?;

    // Create a BitmapDecoder from the stream
    let decoder = BitmapDecoder::CreateAsync(&stream)
        .map_err(|e| OcrError::NetworkError(format!("create decoder failed: {e}")))?
        .get()
        .map_err(|e| OcrError::NetworkError(format!("decoder wait failed: {e}")))?;

    // Get the SoftwareBitmap
    let software_bitmap = decoder
        .GetSoftwareBitmapAsync()
        .map_err(|e| OcrError::NetworkError(format!("get software bitmap failed: {e}")))?
        .get()
        .map_err(|e| OcrError::NetworkError(format!("software bitmap wait failed: {e}")))?;

    // Create an OcrEngine from the user's preferred languages
    let ocr_engine = OcrEngine::TryCreateFromUserProfileLanguages()
        .map_err(|e| OcrError::NetworkError(format!("create OCR engine failed: {e}")))?;

    // Recognize the text
    let ocr_result = ocr_engine
        .RecognizeAsync(&software_bitmap)
        .map_err(|e| OcrError::NetworkError(format!("recognize failed: {e}")))?
        .get()
        .map_err(|e| OcrError::NetworkError(format!("recognize wait failed: {e}")))?;

    let text = ocr_result.Text().map(|s| s.to_string()).unwrap_or_default();

    Ok(RecognizeTextResponse {
        text,
        recognitions: None,
    })
}

pub async fn detect_language(
    _request: DetectLanguageRequest,
) -> Result<DetectLanguageResponse, TranslationError> {
    Err(TranslationError::UnsupportedMethod(
        "system language detection is not supported on Windows",
    ))
}

pub async fn translate(_request: &TranslateRequest) -> Result<TranslateResponse, TranslationError> {
    Err(TranslationError::UnsupportedMethod(
        "system translation is not supported on Windows",
    ))
}

pub async fn look_up(_request: &LookUpRequest) -> Result<LookUpResponse, DictionaryError> {
    Err(DictionaryError::UnsupportedMethod(
        "system dictionary is not supported on Windows",
    ))
}
