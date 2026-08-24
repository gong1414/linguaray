mod model;
mod capability;
mod provider_contract;

pub use model::{
    ChatChoice, ChatMessage, ChatRequest, ChatResponse, ChatRole, ChatUsage, DetectLanguageRequest,
    DetectLanguageResponse, LanguageInfo, LanguagePair, LookUpRequest, LookUpResponse,
    RecognizeTextRequest, RecognizeTextResponse, RecognizedRect, ResponseFormat, StreamChunk,
    StreamState, TextDetection, TextRecognition, TextTranslation, TranslateRequest,
    TranslateResponse, TranslationTarget, WordDefinition, WordEtymology, WordImage, WordPhrase,
    WordPronunciation, WordSentence, WordSynonym, WordTag, WordTense,
};
pub use capability::{
    DictionaryError, DictionaryService, LlmError, LlmService, LlmStreamReceiver, OcrError,
    OcrService, TranslationError, TranslationService,
};
pub use provider_contract::Provider;
