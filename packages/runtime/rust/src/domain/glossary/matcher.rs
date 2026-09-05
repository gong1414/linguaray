use std::collections::HashMap;
use std::sync::Arc;

use aho_corasick::{AhoCorasick, MatchKind};

use super::{GlossaryEntry, GlossaryMatch};

#[derive(Default)]
pub(super) struct MatcherCache {
    pub(super) generation: u64,
    pub(super) entries: HashMap<MatcherKey, Arc<CompiledMatcher>>,
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub(super) struct MatcherKey {
    source: Option<String>,
    target: Option<String>,
}

impl MatcherKey {
    pub(super) fn new(source: Option<&str>, target: Option<&str>) -> Self {
        Self {
            source: normalize_language(source),
            target: normalize_language(target),
        }
    }
}

#[derive(Clone)]
struct PatternInfo {
    book_id: String,
    entry_id: String,
    term: String,
    translation: String,
    forbidden: Vec<String>,
    whole_word: bool,
}

#[derive(Default)]
pub(super) struct PatternSet {
    patterns: Vec<String>,
    infos: Vec<PatternInfo>,
}

impl PatternSet {
    pub(super) fn push(&mut self, book_id: String, entry: &GlossaryEntry) {
        self.patterns.push(entry.term.clone());
        self.infos.push(PatternInfo {
            book_id,
            entry_id: entry.id.clone(),
            term: entry.term.clone(),
            translation: entry.translation.clone(),
            forbidden: entry.forbidden.clone(),
            whole_word: entry.whole_word,
        });
    }

    pub(super) fn build(self, case_insensitive: bool) -> Option<Automaton> {
        if self.patterns.is_empty() {
            return None;
        }
        let automaton = AhoCorasick::builder()
            .match_kind(MatchKind::LeftmostLongest)
            .ascii_case_insensitive(case_insensitive)
            .build(&self.patterns)
            .map_err(|error| eprintln!("[glossary] failed to build matcher: {error}"))
            .ok()?;
        Some(Automaton {
            automaton,
            infos: self.infos,
        })
    }
}

pub(super) struct Automaton {
    automaton: AhoCorasick,
    infos: Vec<PatternInfo>,
}

pub(super) struct CompiledMatcher {
    pub(super) sensitive: Option<Automaton>,
    pub(super) insensitive: Option<Automaton>,
}

impl CompiledMatcher {
    pub(super) fn find(&self, text: &str) -> Vec<GlossaryMatch> {
        let mut raw: Vec<(usize, usize, &PatternInfo)> = Vec::new();
        for automaton in [self.sensitive.as_ref(), self.insensitive.as_ref()]
            .into_iter()
            .flatten()
        {
            for found in automaton.automaton.find_iter(text) {
                let info = &automaton.infos[found.pattern().as_usize()];
                if info.whole_word
                    && !is_word_boundary(text, found.start(), found.end(), &info.term)
                {
                    continue;
                }
                raw.push((found.start(), found.end(), info));
            }
        }

        raw.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| b.1.cmp(&a.1)));
        let mut matches = Vec::new();
        let mut consumed_to = 0usize;
        for (start, end, info) in raw {
            if start < consumed_to {
                continue;
            }
            consumed_to = end;
            matches.push(GlossaryMatch {
                book_id: info.book_id.clone(),
                entry_id: info.entry_id.clone(),
                term: info.term.clone(),
                matched_text: text[start..end].to_owned(),
                translation: info.translation.clone(),
                forbidden: info.forbidden.clone(),
                start: start as u32,
                end: end as u32,
            });
        }
        matches
    }
}

pub(super) fn language_applies(book: Option<&str>, request: Option<&str>) -> bool {
    let Some(book) = book.map(str::trim).filter(|value| !value.is_empty()) else {
        return true;
    };
    let Some(request) = request.map(str::trim).filter(|value| !value.is_empty()) else {
        return true;
    };
    if request.eq_ignore_ascii_case("auto") || book.eq_ignore_ascii_case(request) {
        return true;
    }
    primary_subtag(book).eq_ignore_ascii_case(primary_subtag(request))
}

fn is_word_boundary(text: &str, start: usize, end: usize, term: &str) -> bool {
    let is_word_char = |c: char| c.is_ascii_alphanumeric() || c == '_';
    if term
        .chars()
        .next()
        .is_some_and(|c| c.is_ascii_alphanumeric())
        && text[..start].chars().next_back().is_some_and(is_word_char)
    {
        return false;
    }
    if term
        .chars()
        .next_back()
        .is_some_and(|c| c.is_ascii_alphanumeric())
        && text[end..].chars().next().is_some_and(is_word_char)
    {
        return false;
    }
    true
}

fn primary_subtag(language: &str) -> &str {
    language.split(['-', '_']).next().unwrap_or(language)
}

fn normalize_language(language: Option<&str>) -> Option<String> {
    language
        .map(|value| value.trim().to_lowercase())
        .filter(|value| !value.is_empty())
}
