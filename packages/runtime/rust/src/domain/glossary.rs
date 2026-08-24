//! Glossary storage and term matching.
//!
//! Terms live in `<data_dir>/glossary/<book_id>.json`, one file per book, so
//! deleting a book is deleting a file and a large book never slows down
//! reading settings. The whole set is held in memory; writes go through
//! [`GlossaryStore`], which rewrites only the affected book.
//!
//! Matching compiles the enabled books into an Aho-Corasick automaton cached
//! per language pair. The cache is keyed on a generation counter that every
//! mutation bumps, so callers never have to invalidate it by hand.

use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

mod compliance;
mod matcher;

pub use compliance::check_compliance;
use matcher::{language_applies, CompiledMatcher, MatcherCache, MatcherKey, PatternSet};

/// Directory under `data_dir` holding one JSON file per book.
const GLOSSARY_DIR: &str = "glossary";

/// Hit counters are a soft statistic, so they accumulate in memory and are
/// written at most this often rather than on every translation. Counts
/// recorded since the last write are lost if the process exits, which is an
/// acceptable trade for not rewriting a book file on every keystroke-driven
/// translation.
const HIT_FLUSH_INTERVAL: Duration = Duration::from_secs(5);

// ── Public types ─────────────────────────────────────────────────────────────

/// A term book as presented to the UI. `entry_count` is derived, so the book
/// list can be rendered without carrying every term across the FFI boundary.
#[derive(Clone, Debug, PartialEq, uniffi::Record)]
pub struct GlossaryBook {
    pub id: String,
    pub name: String,
    pub enabled: bool,
    pub source_language: Option<String>,
    pub target_language: Option<String>,
    pub entry_count: u32,
    pub created_at: u64,
    pub updated_at: u64,
}

/// Book metadata as supplied by a caller. Every field is authoritative: read
/// the book, change what you need, send the whole thing back.
#[derive(Clone, Debug, uniffi::Record)]
pub struct GlossaryBookInput {
    /// `None` creates a new book; `Some(id)` updates an existing one.
    pub id: Option<String>,
    pub name: String,
    pub enabled: bool,
    /// `None` means the book applies to any source language.
    pub source_language: Option<String>,
    /// `None` means the book applies to any target language.
    pub target_language: Option<String>,
}

/// One term and the translation it must receive.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, uniffi::Record)]
pub struct GlossaryEntry {
    pub id: String,
    pub term: String,
    pub translation: String,
    /// Translations that must never appear for this term.
    #[serde(default)]
    pub forbidden: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
    #[serde(default, rename = "caseSensitive")]
    pub case_sensitive: bool,
    /// Requires the match to sit on word boundaries. Only meaningful for
    /// terms that start or end with an ASCII alphanumeric; CJK terms have no
    /// word boundaries to check.
    #[serde(default = "default_true", rename = "wholeWord")]
    pub whole_word: bool,
    #[serde(default)]
    pub hits: u64,
    #[serde(default, rename = "createdAt")]
    pub created_at: u64,
    #[serde(default, rename = "updatedAt")]
    pub updated_at: u64,
}

/// Entry fields as supplied by a caller, with the same full-replace semantics
/// as [`GlossaryBookInput`].
#[derive(Clone, Debug, uniffi::Record)]
pub struct GlossaryEntryInput {
    /// `None` creates a new entry; `Some(id)` updates an existing one.
    pub id: Option<String>,
    pub term: String,
    pub translation: String,
    pub forbidden: Vec<String>,
    pub note: Option<String>,
    pub case_sensitive: bool,
    pub whole_word: bool,
}

/// Summary returned after merging an external glossary into a book.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, uniffi::Record)]
pub struct GlossaryImportReport {
    pub inserted: u32,
    pub updated: u32,
    pub skipped: u32,
}

/// A term found in a source text.
#[derive(Clone, Debug, PartialEq, uniffi::Record)]
pub struct GlossaryMatch {
    pub book_id: String,
    pub entry_id: String,
    pub term: String,
    /// The substring that actually matched, which differs from `term` when
    /// the entry is case-insensitive.
    pub matched_text: String,
    pub translation: String,
    pub forbidden: Vec<String>,
    /// UTF-8 byte offsets into the text that was searched.
    pub start: u32,
    pub end: u32,
}

/// Why a translation failed its glossary check.
#[derive(Clone, Copy, Debug, PartialEq, Eq, uniffi::Enum)]
pub enum GlossaryIssueKind {
    /// The term appeared in the source but its required translation is
    /// missing from the output.
    MissingTranslation,
    /// The output uses a translation the entry forbids.
    ForbiddenUsed,
}

/// One glossary rule the output violated.
#[derive(Clone, Debug, PartialEq, uniffi::Record)]
pub struct GlossaryComplianceIssue {
    pub book_id: String,
    pub entry_id: String,
    pub kind: GlossaryIssueKind,
    pub term: String,
    /// The translation the entry requires.
    pub expected: String,
    /// The forbidden text that was found, for [`GlossaryIssueKind::ForbiddenUsed`].
    pub found: Option<String>,
}

// ── On-disk shape ────────────────────────────────────────────────────────────

#[derive(Clone, Debug, Serialize, Deserialize)]
struct BookFile {
    id: String,
    name: String,
    #[serde(default = "default_true")]
    enabled: bool,
    #[serde(
        default,
        rename = "sourceLanguage",
        skip_serializing_if = "Option::is_none"
    )]
    source_language: Option<String>,
    #[serde(
        default,
        rename = "targetLanguage",
        skip_serializing_if = "Option::is_none"
    )]
    target_language: Option<String>,
    #[serde(default, rename = "createdAt")]
    created_at: u64,
    #[serde(default, rename = "updatedAt")]
    updated_at: u64,
    #[serde(default)]
    entries: Vec<GlossaryEntry>,
}

impl BookFile {
    fn to_book(&self) -> GlossaryBook {
        GlossaryBook {
            id: self.id.clone(),
            name: self.name.clone(),
            enabled: self.enabled,
            source_language: self.source_language.clone(),
            target_language: self.target_language.clone(),
            entry_count: self.entries.len() as u32,
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }
}

fn default_true() -> bool {
    true
}

// ── Store ────────────────────────────────────────────────────────────────────

/// In-memory glossary backed by one JSON file per book.
pub struct GlossaryStore {
    dir: PathBuf,
    books: BTreeMap<String, BookFile>,
    /// Bumped by every mutation that can change matching, so compiled
    /// automatons can be invalidated without diffing the term set.
    generation: u64,
    /// Books whose hit counts have changed but not yet been written.
    pending_hits: BTreeSet<String>,
    last_hit_flush: Instant,
    matchers: Mutex<MatcherCache>,
    id_seq: u64,
}

impl GlossaryStore {
    pub(crate) fn validate_backup(data_dir: impl AsRef<Path>) -> Result<(), String> {
        let dir = data_dir.as_ref().join(GLOSSARY_DIR);
        if !dir.exists() {
            return Ok(());
        }
        let listing = fs::read_dir(&dir).map_err(|error| {
            format!(
                "failed to read glossary directory `{}`: {error}",
                dir.display()
            )
        })?;
        for entry in listing {
            let entry = entry.map_err(|error| {
                format!(
                    "failed to read glossary directory `{}`: {error}",
                    dir.display()
                )
            })?;
            let path = entry.path();
            if path.extension().and_then(|ext| ext.to_str()) == Some("json") {
                load_book_file(&path)
                    .map_err(|error| format!("invalid `{}`: {error}", path.display()))?;
            }
        }
        Ok(())
    }

    /// Loads every book under `<data_dir>/glossary`. A missing directory is
    /// an empty glossary, not an error; it is created on first write.
    pub fn load(data_dir: impl AsRef<Path>) -> Result<Self, String> {
        let dir = data_dir.as_ref().join(GLOSSARY_DIR);
        let mut books = BTreeMap::new();

        if dir.is_dir() {
            let listing = fs::read_dir(&dir).map_err(|error| {
                format!(
                    "failed to read glossary directory `{}`: {error}",
                    dir.display()
                )
            })?;
            for entry in listing {
                let entry = entry.map_err(|error| {
                    format!(
                        "failed to read glossary directory `{}`: {error}",
                        dir.display()
                    )
                })?;
                let path = entry.path();
                if path.extension().and_then(|ext| ext.to_str()) != Some("json") {
                    continue;
                }
                match load_book_file(&path) {
                    Ok(book) => {
                        books.insert(book.id.clone(), book);
                    }
                    // One corrupt book must not take the whole glossary — and
                    // with it the app's translation path — down with it.
                    Err(error) => eprintln!("[glossary] skipping `{}`: {error}", path.display()),
                }
            }
        }

        Ok(Self {
            dir,
            books,
            generation: 0,
            pending_hits: BTreeSet::new(),
            last_hit_flush: Instant::now(),
            matchers: Mutex::new(MatcherCache::default()),
            id_seq: 0,
        })
    }

    // ── Books ────────────────────────────────────────────────────────────

    pub fn list_books(&self) -> Vec<GlossaryBook> {
        let mut books: Vec<GlossaryBook> = self.books.values().map(BookFile::to_book).collect();
        books.sort_by(|a, b| a.created_at.cmp(&b.created_at).then(a.id.cmp(&b.id)));
        books
    }

    pub fn get_book(&self, book_id: &str) -> Option<GlossaryBook> {
        self.books.get(book_id).map(BookFile::to_book)
    }

    pub fn upsert_book(&mut self, input: GlossaryBookInput) -> Result<GlossaryBook, String> {
        let name = require_non_empty("name", &input.name)?;
        let source_language = normalize_optional(input.source_language);
        let target_language = normalize_optional(input.target_language);
        let now = now_secs();

        let book_id = match input
            .id
            .as_deref()
            .map(str::trim)
            .filter(|id| !id.is_empty())
        {
            Some(id) => {
                validate_book_id(id)?;
                let book = self
                    .books
                    .get_mut(id)
                    .ok_or_else(|| format!("glossary book `{id}` does not exist"))?;
                book.name = name;
                book.enabled = input.enabled;
                book.source_language = source_language;
                book.target_language = target_language;
                book.updated_at = now;
                id.to_owned()
            }
            None => {
                let taken: HashSet<String> = self.books.keys().cloned().collect();
                let id = next_id(&mut self.id_seq, |id| taken.contains(id));
                self.books.insert(
                    id.clone(),
                    BookFile {
                        id: id.clone(),
                        name,
                        enabled: input.enabled,
                        source_language,
                        target_language,
                        created_at: now,
                        updated_at: now,
                        entries: Vec::new(),
                    },
                );
                id
            }
        };

        self.commit(&book_id)?;
        Ok(self.books[&book_id].to_book())
    }

    /// Removes a book and its file. Returns `false` if it did not exist.
    pub fn delete_book(&mut self, book_id: &str) -> Result<bool, String> {
        validate_book_id(book_id)?;
        if self.books.remove(book_id).is_none() {
            return Ok(false);
        }
        self.pending_hits.remove(book_id);
        self.generation += 1;

        let path = self.book_path(book_id);
        if path.exists() {
            fs::remove_file(&path).map_err(|error| {
                format!(
                    "failed to delete glossary book `{}`: {error}",
                    path.display()
                )
            })?;
        }
        Ok(true)
    }

    // ── Entries ──────────────────────────────────────────────────────────

    /// Entries newest-first, optionally filtered by a case-insensitive
    /// substring of the term, translation, forbidden list or note.
    pub fn list_entries(
        &self,
        book_id: &str,
        query: Option<&str>,
        offset: u32,
        limit: u32,
    ) -> Result<Vec<GlossaryEntry>, String> {
        let book = self.require_book(book_id)?;
        let mut entries: Vec<&GlossaryEntry> = book
            .entries
            .iter()
            .filter(|entry| entry_matches_query(entry, query))
            .collect();
        entries.sort_by(|a, b| {
            b.created_at
                .cmp(&a.created_at)
                .then_with(|| a.term.cmp(&b.term))
        });

        let offset = offset as usize;
        if offset >= entries.len() {
            return Ok(Vec::new());
        }
        let end = if limit == 0 {
            entries.len()
        } else {
            entries.len().min(offset + limit as usize)
        };
        Ok(entries[offset..end].iter().map(|e| (*e).clone()).collect())
    }

    pub fn count_entries(&self, book_id: &str, query: Option<&str>) -> Result<u32, String> {
        let book = self.require_book(book_id)?;
        Ok(book
            .entries
            .iter()
            .filter(|entry| entry_matches_query(entry, query))
            .count() as u32)
    }

    pub fn upsert_entry(
        &mut self,
        book_id: &str,
        input: GlossaryEntryInput,
    ) -> Result<GlossaryEntry, String> {
        validate_book_id(book_id)?;
        let entry = self.apply_entry(book_id, input, now_secs())?;
        self.commit(book_id)?;
        Ok(entry)
    }

    /// Merges many entries and writes the book once. Existing terms are
    /// matched case-insensitively, which is the same rule used by interactive
    /// entry editing.
    pub fn import_entries(
        &mut self,
        book_id: &str,
        entries: Vec<GlossaryEntryInput>,
    ) -> Result<GlossaryImportReport, String> {
        validate_book_id(book_id)?;
        self.require_book(book_id)?;
        let mut report = GlossaryImportReport::default();
        let now = now_secs();

        for entry in entries {
            let term = entry.term.trim();
            if term.is_empty() || entry.translation.trim().is_empty() {
                report.skipped += 1;
                continue;
            }
            let existed = self.books[book_id]
                .entries
                .iter()
                .any(|saved| saved.term.eq_ignore_ascii_case(term));
            match self.apply_entry(book_id, entry, now) {
                Ok(_) if existed => report.updated += 1,
                Ok(_) => report.inserted += 1,
                Err(_) => report.skipped += 1,
            }
        }

        if report.inserted > 0 || report.updated > 0 {
            self.commit(book_id)?;
        }
        Ok(report)
    }

    /// Removes an entry. Returns `false` if it did not exist.
    pub fn delete_entry(&mut self, book_id: &str, entry_id: &str) -> Result<bool, String> {
        validate_book_id(book_id)?;
        let book = self
            .books
            .get_mut(book_id)
            .ok_or_else(|| format!("glossary book `{book_id}` does not exist"))?;
        let before = book.entries.len();
        book.entries.retain(|entry| entry.id != entry_id);
        if book.entries.len() == before {
            return Ok(false);
        }
        book.updated_at = now_secs();
        self.commit(book_id)?;
        Ok(true)
    }

    /// Inserts or updates one entry without writing to disk. Shared by
    /// [`Self::upsert_entry`] and bulk import so a multi-row import pays for
    /// a single file write.
    fn apply_entry(
        &mut self,
        book_id: &str,
        input: GlossaryEntryInput,
        now: u64,
    ) -> Result<GlossaryEntry, String> {
        let term = require_non_empty("term", &input.term)?;
        let translation = require_non_empty("translation", &input.translation)?;
        let forbidden: Vec<String> = input
            .forbidden
            .into_iter()
            .map(|value| value.trim().to_owned())
            .filter(|value| !value.is_empty())
            .collect();
        let note = normalize_optional(input.note);

        // A book keys terms case-insensitively: two entries for the same term
        // would give the matcher no way to choose between them.
        let existing_by_term = self
            .books
            .get(book_id)
            .ok_or_else(|| format!("glossary book `{book_id}` does not exist"))?
            .entries
            .iter()
            .find(|entry| entry.term.eq_ignore_ascii_case(&term))
            .map(|entry| entry.id.clone());

        let entry_id = match input
            .id
            .as_deref()
            .map(str::trim)
            .filter(|id| !id.is_empty())
        {
            Some(id) => {
                if existing_by_term.as_deref().is_some_and(|other| other != id) {
                    return Err(format!("term `{term}` already exists in this book"));
                }
                id.to_owned()
            }
            None => match existing_by_term {
                Some(id) => id,
                None => {
                    let taken: HashSet<String> = self.books[book_id]
                        .entries
                        .iter()
                        .map(|entry| entry.id.clone())
                        .collect();
                    next_id(&mut self.id_seq, |id| taken.contains(id))
                }
            },
        };

        let book = self
            .books
            .get_mut(book_id)
            .ok_or_else(|| format!("glossary book `{book_id}` does not exist"))?;

        let entry = match book.entries.iter_mut().find(|entry| entry.id == entry_id) {
            Some(existing) => {
                existing.term = term;
                existing.translation = translation;
                existing.forbidden = forbidden;
                existing.note = note;
                existing.case_sensitive = input.case_sensitive;
                existing.whole_word = input.whole_word;
                existing.updated_at = now;
                existing.clone()
            }
            None => {
                let entry = GlossaryEntry {
                    id: entry_id,
                    term,
                    translation,
                    forbidden,
                    note,
                    case_sensitive: input.case_sensitive,
                    whole_word: input.whole_word,
                    hits: 0,
                    created_at: now,
                    updated_at: now,
                };
                book.entries.push(entry.clone());
                entry
            }
        };
        book.updated_at = now;
        Ok(entry)
    }

    // ── Matching ─────────────────────────────────────────────────────────

    /// Finds every glossary term present in `text`, longest match wins and
    /// overlaps are dropped left to right.
    pub fn match_text(
        &self,
        text: &str,
        source_language: Option<&str>,
        target_language: Option<&str>,
    ) -> Vec<GlossaryMatch> {
        if text.trim().is_empty() {
            return Vec::new();
        }
        let matcher = self.matcher(source_language, target_language);
        matcher.find(text)
    }

    /// Returns the compiled automaton for a language pair, building it on
    /// first use and reusing it until the term set changes.
    fn matcher(
        &self,
        source_language: Option<&str>,
        target_language: Option<&str>,
    ) -> Arc<CompiledMatcher> {
        let key = MatcherKey::new(source_language, target_language);
        let mut cache = match self.matchers.lock() {
            Ok(cache) => cache,
            // A poisoned cache is only ever stale derived data; rebuild it
            // rather than propagating the panic into the translation path.
            Err(poisoned) => poisoned.into_inner(),
        };
        if cache.generation != self.generation {
            cache.entries.clear();
            cache.generation = self.generation;
        }
        if let Some(matcher) = cache.entries.get(&key) {
            return matcher.clone();
        }
        let matcher = Arc::new(self.compile(source_language, target_language));
        cache.entries.insert(key, matcher.clone());
        matcher
    }

    fn compile(
        &self,
        source_language: Option<&str>,
        target_language: Option<&str>,
    ) -> CompiledMatcher {
        let mut sensitive = PatternSet::default();
        let mut insensitive = PatternSet::default();

        for book in self.books.values() {
            if !book.enabled
                || !language_applies(book.source_language.as_deref(), source_language)
                || !language_applies(book.target_language.as_deref(), target_language)
            {
                continue;
            }
            for entry in &book.entries {
                if entry.term.is_empty() {
                    continue;
                }
                let target = if entry.case_sensitive {
                    &mut sensitive
                } else {
                    &mut insensitive
                };
                target.push(book.id.clone(), entry);
            }
        }

        CompiledMatcher {
            sensitive: sensitive.build(false),
            insensitive: insensitive.build(true),
        }
    }

    // ── Hits ─────────────────────────────────────────────────────────────

    /// Increments the hit counter for every matched entry. Counts accumulate
    /// in memory and reach disk on the next flush interval; see
    /// [`HIT_FLUSH_INTERVAL`].
    pub fn record_hits(&mut self, matches: &[GlossaryMatch]) {
        if matches.is_empty() {
            return;
        }
        for hit in matches {
            let Some(book) = self.books.get_mut(&hit.book_id) else {
                continue;
            };
            let Some(entry) = book
                .entries
                .iter_mut()
                .find(|entry| entry.id == hit.entry_id)
            else {
                continue;
            };
            entry.hits = entry.hits.saturating_add(1);
            self.pending_hits.insert(hit.book_id.clone());
        }

        if self.last_hit_flush.elapsed() >= HIT_FLUSH_INTERVAL {
            if let Err(error) = self.flush_hits() {
                eprintln!("[glossary] failed to persist hit counts: {error}");
            }
        }
    }

    /// Writes any books with unsaved hit counts.
    pub fn flush_hits(&mut self) -> Result<(), String> {
        self.last_hit_flush = Instant::now();
        let pending = std::mem::take(&mut self.pending_hits);
        for book_id in pending {
            let Some(book) = self.books.get(&book_id) else {
                continue;
            };
            self.write_book(book)?;
        }
        Ok(())
    }

    // ── Persistence helpers ──────────────────────────────────────────────

    /// Persists one book and invalidates compiled automatons.
    fn commit(&mut self, book_id: &str) -> Result<(), String> {
        self.generation += 1;
        // The book is about to be written in full, so any hits it was
        // holding go out with it.
        self.pending_hits.remove(book_id);
        let book = self
            .books
            .get(book_id)
            .ok_or_else(|| format!("glossary book `{book_id}` does not exist"))?;
        self.write_book(book)
    }

    fn write_book(&self, book: &BookFile) -> Result<(), String> {
        fs::create_dir_all(&self.dir).map_err(|error| {
            format!(
                "failed to create glossary directory `{}`: {error}",
                self.dir.display()
            )
        })?;
        let path = self.book_path(&book.id);
        let content = serde_json::to_string_pretty(book)
            .map_err(|error| format!("failed to encode glossary book `{}`: {error}", book.id))?;
        fs::write(&path, content).map_err(|error| {
            format!(
                "failed to write glossary book `{}`: {error}",
                path.display()
            )
        })
    }

    fn book_path(&self, book_id: &str) -> PathBuf {
        self.dir.join(format!("{book_id}.json"))
    }

    fn require_book(&self, book_id: &str) -> Result<&BookFile, String> {
        self.books
            .get(book_id)
            .ok_or_else(|| format!("glossary book `{book_id}` does not exist"))
    }
}

/// Time-ordered id, retried until `taken` reports it free. Ids restart their
/// sequence each launch, so the collision check is what actually guarantees
/// uniqueness.
fn next_id(id_seq: &mut u64, taken: impl Fn(&str) -> bool) -> String {
    loop {
        *id_seq += 1;
        let id = format!("{:x}{:03x}", now_millis(), *id_seq & 0xfff);
        if !taken(&id) {
            return id;
        }
    }
}

fn normalize_optional(value: Option<String>) -> Option<String> {
    value
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty())
}

fn require_non_empty(name: &str, value: &str) -> Result<String, String> {
    let value = value.trim();
    if value.is_empty() {
        return Err(format!("{name} is required"));
    }
    Ok(value.to_owned())
}

fn entry_matches_query(entry: &GlossaryEntry, query: Option<&str>) -> bool {
    let Some(query) = query.map(str::trim).filter(|value| !value.is_empty()) else {
        return true;
    };
    let needle = query.to_lowercase();
    entry.term.to_lowercase().contains(&needle)
        || entry.translation.to_lowercase().contains(&needle)
        || entry
            .forbidden
            .iter()
            .any(|value| value.to_lowercase().contains(&needle))
        || entry
            .note
            .as_deref()
            .is_some_and(|note| note.to_lowercase().contains(&needle))
}

/// Book ids become file names, so they may not carry path separators or
/// anything else that would let a book escape the glossary directory.
fn validate_book_id(book_id: &str) -> Result<(), String> {
    if book_id.is_empty() {
        return Err("book_id is required".to_owned());
    }
    if !book_id
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    {
        return Err(format!(
            "invalid glossary book id `{book_id}`: only letters, digits, `-` and `_` are allowed"
        ));
    }
    Ok(())
}

fn load_book_file(path: &Path) -> Result<BookFile, String> {
    let content = fs::read_to_string(path).map_err(|error| format!("failed to read: {error}"))?;
    let mut book: BookFile =
        serde_json::from_str(&content).map_err(|error| format!("failed to parse: {error}"))?;
    if book.id.trim().is_empty() {
        book.id = path
            .file_stem()
            .and_then(|stem| stem.to_str())
            .unwrap_or_default()
            .to_owned();
    }
    validate_book_id(&book.id)?;
    Ok(book)
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or_default()
}

fn now_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis() as u64)
        .unwrap_or_default()
}

#[cfg(test)]
#[path = "glossary/tests.rs"]
mod tests;
