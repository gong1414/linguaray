mod chat;
mod dictionary;
mod language;
mod translation;

pub use chat::{
    ChatChoice, ChatMessage, ChatRequest, ChatResponse, ChatRole, ChatUsage, ResponseFormat,
    StreamChunk, StreamState,
};
pub use dictionary::{
    LookUpRequest, LookUpResponse, WordDefinition, WordEtymology, WordImage, WordPhrase,
    WordPronunciation, WordSentence, WordSynonym, WordTag, WordTense,
};
pub use language::{
    DetectLanguageRequest, DetectLanguageResponse, LanguageInfo, LanguagePair, TextDetection,
    TranslationTarget,
};
pub use translation::{
    RecognizeTextRequest, RecognizeTextResponse, RecognizedRect, TextRecognition, TextTranslation,
    TranslateRequest, TranslateResponse,
};
