//! Compact offline English-Chinese dictionary backed by ECDICT.
//!
//! The bundled data is a reproducible subset of skywind3000/ECDICT. See
//! `THIRD_PARTY_NOTICES.md` and `scripts/generate_ecdict.py`.

use std::collections::HashMap;
use std::io::Read;
use std::sync::OnceLock;

use async_trait::async_trait;
use flate2::read::GzDecoder;
use linguaray_core::{
    DictionaryError, DictionaryService, LookUpRequest, LookUpResponse, Provider, TextTranslation,
    WordDefinition, WordPronunciation,
};
use serde::Deserialize;

const ECDICT_DATA: &[u8] = include_bytes!("../../../assets/ecdict-compact.json.gz");

#[derive(Clone, Debug, Deserialize)]
struct EcdictEntry {
    #[serde(rename = "w")]
    word: String,
    #[serde(rename = "p")]
    phonetic: String,
    #[serde(rename = "t")]
    translation: String,
    #[serde(rename = "d")]
    definition: String,
    #[serde(rename = "x")]
    exchange: String,
}

struct EcdictIndex {
    entries: HashMap<String, EcdictEntry>,
    aliases: HashMap<String, String>,
}

static INDEX: OnceLock<Result<EcdictIndex, String>> = OnceLock::new();

pub struct EcdictProvider;

impl Provider for EcdictProvider {
    fn name(&self) -> &'static str {
        "ECDICT"
    }

    fn dictionary(&self) -> Option<&dyn DictionaryService> {
        Some(self)
    }
}

#[async_trait(?Send)]
impl DictionaryService for EcdictProvider {
    async fn look_up(&self, request: LookUpRequest) -> Result<LookUpResponse, DictionaryError> {
        let query = normalize(&request.word);
        if query.is_empty() {
            return Err(DictionaryError::InvalidRequest(
                "dictionary word must not be empty".to_owned(),
            ));
        }

        let index = index()?;
        let key = if index.entries.contains_key(&query) {
            &query
        } else if let Some(lemma) = index.aliases.get(&query) {
            lemma
        } else {
            return Ok(empty_response(request.word));
        };
        let entry = &index.entries[key];

        let translations = entry
            .translation
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .map(|text| TextTranslation {
                detected_source_language: Some("en".to_owned()),
                text: text.to_owned(),
                audio_url: None,
            })
            .collect::<Vec<_>>();
        let definition_values = entry
            .definition
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .map(ToOwned::to_owned)
            .collect::<Vec<_>>();

        Ok(LookUpResponse {
            translations,
            word: Some(entry.word.clone()),
            tip: None,
            tags: None,
            definitions: (!definition_values.is_empty()).then(|| {
                vec![WordDefinition {
                    r#type: Some("english".to_owned()),
                    name: Some("English".to_owned()),
                    values: Some(definition_values),
                }]
            }),
            pronunciations: (!entry.phonetic.is_empty()).then(|| {
                vec![WordPronunciation {
                    r#type: None,
                    phonetic_symbol: Some(entry.phonetic.clone()),
                    audio_url: None,
                }]
            }),
            images: None,
            phrases: None,
            tenses: None,
            sentences: None,
            etymology: None,
            synonyms: None,
        })
    }
}

fn index() -> Result<&'static EcdictIndex, DictionaryError> {
    match INDEX.get_or_init(load_index) {
        Ok(index) => Ok(index),
        Err(message) => Err(DictionaryError::SerializationError(message.clone())),
    }
}

fn load_index() -> Result<EcdictIndex, String> {
    let mut decoder = GzDecoder::new(ECDICT_DATA);
    let mut json = Vec::new();
    decoder
        .read_to_end(&mut json)
        .map_err(|error| format!("failed to decompress ECDICT: {error}"))?;
    let records: Vec<EcdictEntry> = serde_json::from_slice(&json)
        .map_err(|error| format!("failed to parse ECDICT: {error}"))?;

    let mut entries = HashMap::with_capacity(records.len());
    for entry in records {
        entries.insert(normalize(&entry.word), entry);
    }

    let mut aliases = HashMap::new();
    for (word, entry) in &entries {
        for item in entry.exchange.split('/') {
            let Some((_, inflection)) = item.split_once(':') else {
                continue;
            };
            let alias = normalize(inflection);
            if !alias.is_empty() && !entries.contains_key(&alias) {
                aliases.entry(alias).or_insert_with(|| word.clone());
            }
        }
    }

    Ok(EcdictIndex { entries, aliases })
}

fn normalize(word: &str) -> String {
    word.trim()
        .trim_matches(|character: char| {
            !character.is_alphanumeric() && character != '\'' && character != '-'
        })
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

fn empty_response(word: String) -> LookUpResponse {
    LookUpResponse {
        translations: Vec::new(),
        word: Some(word),
        tip: None,
        tags: None,
        definitions: None,
        pronunciations: None,
        images: None,
        phrases: None,
        tenses: None,
        sentences: None,
        etymology: None,
        synonyms: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn looks_up_a_common_word_offline() {
        let response = EcdictProvider
            .look_up(LookUpRequest {
                source_language: "en".to_owned(),
                target_language: "zh-Hans".to_owned(),
                word: "apple".to_owned(),
            })
            .await
            .expect("lookup");

        assert_eq!(response.word.as_deref(), Some("apple"));
        assert!(response
            .translations
            .iter()
            .any(|translation| translation.text.contains("苹果")));
        assert!(response.pronunciations.is_some());
    }

    #[tokio::test]
    async fn resolves_common_inflections_to_their_lemma() {
        let response = EcdictProvider
            .look_up(LookUpRequest {
                source_language: "en".to_owned(),
                target_language: "zh-Hans".to_owned(),
                word: "apples".to_owned(),
            })
            .await
            .expect("lookup");

        assert_eq!(response.word.as_deref(), Some("apple"));
    }
}
