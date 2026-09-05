// UniFFI emits its encoded UDL metadata as a const byte array. Rust 1.98's
// `large_const_arrays` lint cannot be addressed without patching generated
// scaffolding, so keep the exception scoped to this FFI crate.
#![allow(clippy::large_const_arrays)]

mod api_server;
mod backup;
mod catalog;
pub use catalog::{
    list_catalog_snapshot_models, list_provider_catalog, CatalogCategory, CatalogFieldSpec,
    CatalogModelChoice, CatalogNetworkPolicy, CatalogProviderPreset, CatalogStability,
};
pub mod domain;
mod remote;
pub mod runtime;
pub use api_server::{ApiServerInfo, RuntimeApiServer};
pub use runtime::{
    BackupSummary, RestoreSummary, Runtime, RuntimeBackup, RuntimeDictionary, RuntimeError,
    RuntimeGlossary, RuntimeHistory, RuntimeLlm, RuntimeOcr, RuntimePermission, RuntimeSettings,
    RuntimeTextExtractor, RuntimeTranslation, RuntimeVocabulary, SettingsChange,
    SettingsSubscription, StreamCallback, TranslationEvent, TranslationTask,
};

use linguaray_core::{
    DetectLanguageRequest, DetectLanguageResponse, LanguagePair, LookUpRequest, LookUpResponse,
    RecognizeTextRequest, RecognizeTextResponse, RecognizedRect, TextDetection, TextRecognition,
    TextTranslation, TranslateRequest, TranslateResponse, WordDefinition, WordEtymology, WordImage,
    WordPhrase, WordPronunciation, WordSentence, WordSynonym, WordTag, WordTense,
};

#[uniffi::export]
pub fn echo_detect_language_request(request: DetectLanguageRequest) -> DetectLanguageRequest {
    request
}

#[uniffi::export]
pub fn echo_detect_language_response(response: DetectLanguageResponse) -> DetectLanguageResponse {
    response
}

#[uniffi::export]
pub fn echo_language_pair(language_pair: LanguagePair) -> LanguagePair {
    language_pair
}

#[uniffi::export]
pub fn echo_look_up_request(request: LookUpRequest) -> LookUpRequest {
    request
}

#[uniffi::export]
pub fn echo_look_up_response(response: LookUpResponse) -> LookUpResponse {
    response
}

#[uniffi::export]
pub fn echo_text_detection(text_detection: TextDetection) -> TextDetection {
    text_detection
}

#[uniffi::export]
pub fn echo_text_translation(text_translation: TextTranslation) -> TextTranslation {
    text_translation
}

#[uniffi::export]
pub fn echo_translate_request(request: TranslateRequest) -> TranslateRequest {
    request
}

#[uniffi::export]
pub fn echo_translate_response(response: TranslateResponse) -> TranslateResponse {
    response
}

#[uniffi::export]
pub fn echo_word_definition(word_definition: WordDefinition) -> WordDefinition {
    word_definition
}

#[uniffi::export]
pub fn echo_word_image(word_image: WordImage) -> WordImage {
    word_image
}

#[uniffi::export]
pub fn echo_word_phrase(word_phrase: WordPhrase) -> WordPhrase {
    word_phrase
}

#[uniffi::export]
pub fn echo_word_pronunciation(word_pronunciation: WordPronunciation) -> WordPronunciation {
    word_pronunciation
}

#[uniffi::export]
pub fn echo_word_sentence(word_sentence: WordSentence) -> WordSentence {
    word_sentence
}

#[uniffi::export]
pub fn echo_word_tag(word_tag: WordTag) -> WordTag {
    word_tag
}

#[uniffi::export]
pub fn echo_word_etymology(word_etymology: WordEtymology) -> WordEtymology {
    word_etymology
}

#[uniffi::export]
pub fn echo_word_synonym(word_synonym: WordSynonym) -> WordSynonym {
    word_synonym
}

#[uniffi::export]
pub fn echo_word_tense(word_tense: WordTense) -> WordTense {
    word_tense
}

#[uniffi::export]
pub fn echo_recognize_text_request(request: RecognizeTextRequest) -> RecognizeTextRequest {
    request
}

#[uniffi::export]
pub fn echo_recognize_text_response(response: RecognizeTextResponse) -> RecognizeTextResponse {
    response
}

#[uniffi::export]
pub fn echo_recognized_rect(rect: RecognizedRect) -> RecognizedRect {
    rect
}

#[uniffi::export]
pub fn echo_text_recognition(recognition: TextRecognition) -> TextRecognition {
    recognition
}

uniffi::include_scaffolding!("api");
