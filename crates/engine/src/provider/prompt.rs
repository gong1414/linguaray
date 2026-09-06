use std::collections::HashSet;

/// One user glossary rule inserted into a translation request.
pub struct GlossaryTerm {
    pub term: String,
    pub translation: String,
    pub forbidden: Vec<String>,
}

fn json_only(contract: &str) -> String {
    format!("Return only valid JSON matching this contract: {contract}")
}

pub fn translate_word_system_prompt(source: &str, target: &str) -> String {
    format!(
        "Translate one word from {source} to {target}. Include concise lexical information. {}",
        json_only(
            r#"{"translations":[{"text":"...","part_of_speech":null,"explanation":null}],"pronunciation":null}"#,
        )
    )
}

pub fn translate_word_user_prompt(word: &str) -> String {
    format!("Word:\n{word}")
}

pub fn glossary_constraints(terms: &[GlossaryTerm]) -> Option<String> {
    let mut unique = HashSet::new();
    let rules = terms
        .iter()
        .filter(|item| unique.insert(item.term.as_str()))
        .map(|item| {
            let prohibited = if item.forbidden.is_empty() {
                String::new()
            } else {
                format!("; prohibited alternatives: {}", item.forbidden.join(", "))
            };
            format!("- {} => {}{prohibited}", item.term, item.translation)
        })
        .collect::<Vec<_>>();

    (!rules.is_empty()).then(|| {
        format!(
            "Required terminology (override general style choices):\n{}",
            rules.join("\n")
        )
    })
}

pub fn translate_text_system_prompt(
    source: &str,
    target: &str,
    style: Option<&str>,
    glossary: &[GlossaryTerm],
) -> String {
    let mut instructions = translation_instructions(source, target, style, glossary);
    instructions.push(json_only(r#"{"translations":[{"text":"..."}]}"#));
    instructions.join("\n\n")
}

/// Streaming output is displayed as it arrives, so it must be plain text.
pub fn translate_text_stream_system_prompt(
    source: &str,
    target: &str,
    glossary: &[GlossaryTerm],
) -> String {
    let mut instructions = translation_instructions(source, target, None, glossary);
    instructions.push(
        "Return only the translated text. Do not add JSON wrappers, Markdown fences, labels, or explanations."
            .to_owned(),
    );
    instructions.join("\n\n")
}

fn translation_instructions(
    source: &str,
    target: &str,
    style: Option<&str>,
    glossary: &[GlossaryTerm],
) -> Vec<String> {
    let mut instructions = vec![
        format!("Translate from {source} to {target}."),
        "Preserve meaning, tone, paragraph breaks, code, and established names.".to_owned(),
    ];
    if let Some(style) = style.filter(|value| !value.trim().is_empty()) {
        instructions.push(format!("Requested writing style: {style}."));
    }
    if let Some(rules) = glossary_constraints(glossary) {
        instructions.push(rules);
    }
    instructions
}

pub fn translate_text_user_prompt(text: &str) -> String {
    format!("Source text:\n{text}")
}

pub fn dictionary_lookup_system_prompt(source: &str, target: &str) -> String {
    format!(
        "Create a learner-friendly dictionary entry for a {source} word in {target}. Include pronunciation, definitions, examples, related words, phrases, and etymology when known. {}",
        json_only(r#"{"word":"...","pronunciations":[],"definitions":[],"synonyms":[],"antonyms":[],"phrases":[],"etymology":null,"usage_notes":null}"#)
    )
}

pub fn dictionary_lookup_user_prompt(word: &str) -> String {
    format!("Dictionary headword:\n{word}")
}

pub fn polish_translation_system_prompt(style: &str) -> String {
    format!(
        "Edit the supplied translation into a {style} style without changing facts or intent. Return only the revised text."
    )
}

pub fn explain_translation_system_prompt() -> String {
    "Explain the translation choices for a language learner. Cover wording, grammar, localization, and important trade-offs in no more than three short paragraphs.".to_owned()
}

pub fn alternative_translations_system_prompt(count: u32, style: Option<&str>) -> String {
    let style = style
        .filter(|value| !value.trim().is_empty())
        .map(|value| format!(" Use a {value} style."))
        .unwrap_or_default();
    format!(
        "Produce {count} meaningfully different translations of the supplied source.{style} {}",
        json_only(r#"{"alternatives":[{"text":"...","why":"..."}]}"#)
    )
}

#[cfg(test)]
mod tests {
    use super::{glossary_constraints, translate_text_system_prompt, GlossaryTerm};

    #[test]
    fn glossary_terms_are_unique_and_explicit() {
        let terms = [
            GlossaryTerm {
                term: "token".to_owned(),
                translation: "词元".to_owned(),
                forbidden: vec!["令牌".to_owned()],
            },
            GlossaryTerm {
                term: "token".to_owned(),
                translation: "词元".to_owned(),
                forbidden: vec![],
            },
        ];
        let rules = glossary_constraints(&terms).expect("glossary rules");
        assert_eq!(rules.matches("token =>").count(), 1);
        assert!(rules.contains("prohibited alternatives: 令牌"));
    }

    #[test]
    fn translation_prompt_keeps_the_json_contract() {
        let prompt = translate_text_system_prompt("en", "zh-Hans", None, &[]);
        assert!(prompt.contains("translations"));
        assert!(prompt.contains("Return only valid JSON"));
    }
}
