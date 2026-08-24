use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TranslateRequest {
    pub source_language: Option<String>,
    pub target_language: Option<String>,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TextTranslation {
    pub detected_source_language: Option<String>,
    pub text: String,
    pub audio_url: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TranslateResponse {
    pub translations: Vec<TextTranslation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecognizeTextRequest {
    pub image_path: Option<String>,
    pub base64_image: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecognizedRect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub top: Option<f64>,
    pub right: Option<f64>,
    pub bottom: Option<f64>,
    pub left: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextRecognition {
    pub text: String,
    pub recognized_rect: Option<RecognizedRect>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecognizeTextResponse {
    pub text: String,
    pub recognitions: Option<Vec<TextRecognition>>,
}
