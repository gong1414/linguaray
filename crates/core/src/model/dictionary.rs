use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LookUpRequest {
    pub source_language: String,
    pub target_language: String,
    pub word: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WordPhrase {
    pub text: String,
    pub translations: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WordSentence {
    pub text: String,
    pub translations: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WordDefinition {
    pub r#type: Option<String>,
    pub name: Option<String>,
    pub values: Option<Vec<String>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WordTense {
    pub r#type: Option<String>,
    pub name: Option<String>,
    pub values: Option<Vec<String>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WordEtymology {
    pub origin: Option<String>,
    pub root: Option<Vec<String>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WordImage {
    pub url: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WordPronunciation {
    pub r#type: Option<String>,
    pub phonetic_symbol: Option<String>,
    pub audio_url: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WordSynonym {
    pub r#type: Option<String>,
    pub word: String,
    pub definitions: Option<Vec<String>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WordTag {
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LookUpResponse {
    pub translations: Vec<super::TextTranslation>,
    pub word: Option<String>,
    pub tip: Option<String>,
    pub tags: Option<Vec<WordTag>>,
    pub definitions: Option<Vec<WordDefinition>>,
    pub pronunciations: Option<Vec<WordPronunciation>>,
    pub images: Option<Vec<WordImage>>,
    pub phrases: Option<Vec<WordPhrase>>,
    pub tenses: Option<Vec<WordTense>>,
    pub sentences: Option<Vec<WordSentence>>,
    pub etymology: Option<Vec<WordEtymology>>,
    pub synonyms: Option<Vec<WordSynonym>>,
}
