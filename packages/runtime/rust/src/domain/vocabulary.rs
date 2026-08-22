//! Local vocabulary book backed by `<data_dir>/vocabulary.json`.
//!
//! A damaged file must never prevent translation from starting. Entries are
//! kept until the user deletes them.

use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

const VOCABULARY_FILE: &str = "vocabulary.json";
const VOCABULARY_VERSION: u32 = 1;

#[derive(Clone, Copy, Debug, PartialEq, Eq, uniffi::Enum)]
pub enum VocabularyFilter {
    All,
    Favorites,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, uniffi::Record)]
#[serde(rename_all = "camelCase")]
pub struct VocabularyEntry {
    pub id: String,
    pub word: String,
    pub translation: String,
    pub source_language: String,
    pub target_language: String,
    pub source: String,
    pub note: Option<String>,
    pub favorite: bool,
    pub created_at: u64,
    pub updated_at: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, uniffi::Record)]
pub struct VocabularyEntryInput {
    pub id: Option<String>,
    pub word: String,
    pub translation: String,
    pub source_language: String,
    pub target_language: String,
    pub source: String,
    pub note: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct VocabularyFile {
    version: u32,
    entries: Vec<VocabularyEntry>,
}

impl Default for VocabularyFile {
    fn default() -> Self {
        Self {
            version: VOCABULARY_VERSION,
            entries: Vec::new(),
        }
    }
}

pub struct VocabularyStore {
    path: PathBuf,
    entries: Vec<VocabularyEntry>,
    id_seq: u64,
}

impl VocabularyStore {
    pub fn load(data_dir: impl AsRef<Path>) -> Self {
        let path = data_dir.as_ref().join(VOCABULARY_FILE);
        let file = match fs::read_to_string(&path) {
            Ok(content) => match serde_json::from_str::<VocabularyFile>(&content) {
                Ok(file) if file.version == VOCABULARY_VERSION => file,
                Ok(file) => {
                    eprintln!(
                        "[vocabulary] unsupported vocabulary version {} in `{}`",
                        file.version,
                        path.display()
                    );
                    quarantine(&path);
                    VocabularyFile::default()
                }
                Err(error) => {
                    eprintln!(
                        "[vocabulary] failed to decode `{}`: {error}",
                        path.display()
                    );
                    quarantine(&path);
                    VocabularyFile::default()
                }
            },
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => VocabularyFile::default(),
            Err(error) => {
                eprintln!("[vocabulary] failed to read `{}`: {error}", path.display());
                VocabularyFile::default()
            }
        };

        Self {
            path,
            entries: file.entries,
            id_seq: 0,
        }
    }

    pub fn list_entries(
        &self,
        filter: VocabularyFilter,
        query: Option<&str>,
    ) -> Vec<VocabularyEntry> {
        let needle = query.unwrap_or_default().trim().to_lowercase();
        let mut entries = self
            .entries
            .iter()
            .filter(|entry| match filter {
                VocabularyFilter::All => true,
                VocabularyFilter::Favorites => entry.favorite,
            })
            .filter(|entry| {
                needle.is_empty()
                    || [
                        &entry.word,
                        &entry.translation,
                        entry.note.as_deref().unwrap_or(""),
                        &entry.source_language,
                        &entry.target_language,
                    ]
                    .iter()
                    .any(|value| value.to_lowercase().contains(&needle))
            })
            .cloned()
            .collect::<Vec<_>>();
        entries.sort_by(|a, b| {
            b.created_at
                .cmp(&a.created_at)
                .then_with(|| b.updated_at.cmp(&a.updated_at))
                .then_with(|| b.id.cmp(&a.id))
        });
        entries
    }

    pub fn upsert_entry(&mut self, input: VocabularyEntryInput) -> Result<VocabularyEntry, String> {
        let word = required("word", input.word)?;
        let translation = required("translation", input.translation)?;
        let now = now_secs();

        let existing_index = input
            .id
            .as_deref()
            .and_then(|id| self.entries.iter().position(|entry| entry.id == id));
        let (id, created_at, favorite) = if let Some(index) = existing_index {
            let entry = &self.entries[index];
            (entry.id.clone(), entry.created_at, entry.favorite)
        } else {
            let id = input
                .id
                .as_deref()
                .map(str::trim)
                .filter(|id| !id.is_empty())
                .map(str::to_owned)
                .unwrap_or_else(|| self.next_id());
            (id, now, false)
        };

        let note = input
            .note
            .map(|value| value.trim().to_owned())
            .filter(|value| !value.is_empty());

        let entry = VocabularyEntry {
            id,
            word,
            translation,
            source_language: input.source_language.trim().to_owned(),
            target_language: input.target_language.trim().to_owned(),
            source: input.source.trim().to_owned(),
            note,
            favorite,
            created_at,
            updated_at: now,
        };

        if let Some(index) = existing_index {
            self.entries[index] = entry.clone();
        } else {
            self.entries.push(entry.clone());
        }
        self.persist()?;
        Ok(entry)
    }

    pub fn set_favorite(
        &mut self,
        entry_id: &str,
        favorite: bool,
    ) -> Result<Option<VocabularyEntry>, String> {
        let Some(index) = self.entries.iter().position(|entry| entry.id == entry_id) else {
            return Ok(None);
        };
        self.entries[index].favorite = favorite;
        self.entries[index].updated_at = now_secs();
        let entry = self.entries[index].clone();
        self.persist()?;
        Ok(Some(entry))
    }

    pub fn set_note(
        &mut self,
        entry_id: &str,
        note: Option<String>,
    ) -> Result<Option<VocabularyEntry>, String> {
        let Some(index) = self.entries.iter().position(|entry| entry.id == entry_id) else {
            return Ok(None);
        };
        self.entries[index].note = note
            .map(|value| value.trim().to_owned())
            .filter(|value| !value.is_empty());
        self.entries[index].updated_at = now_secs();
        let entry = self.entries[index].clone();
        self.persist()?;
        Ok(Some(entry))
    }

    pub fn delete_entries(&mut self, entry_ids: &[String]) -> Result<u32, String> {
        let before = self.entries.len();
        self.entries
            .retain(|entry| !entry_ids.iter().any(|id| id == &entry.id));
        let removed = before.saturating_sub(self.entries.len());
        if removed > 0 {
            self.persist()?;
        }
        Ok(removed.try_into().unwrap_or(u32::MAX))
    }

    fn next_id(&mut self) -> String {
        loop {
            self.id_seq += 1;
            let id = format!("{:x}{:03x}", now_millis(), self.id_seq & 0xfff);
            if !self.entries.iter().any(|entry| entry.id == id) {
                return id;
            }
        }
    }

    fn persist(&self) -> Result<(), String> {
        let parent = self
            .path
            .parent()
            .ok_or_else(|| "vocabulary path has no parent".to_owned())?;
        fs::create_dir_all(parent).map_err(|error| {
            format!(
                "failed to create vocabulary directory `{}`: {error}",
                parent.display()
            )
        })?;
        let content = serde_json::to_string_pretty(&VocabularyFile {
            version: VOCABULARY_VERSION,
            entries: self.entries.clone(),
        })
        .map_err(|error| format!("failed to encode vocabulary: {error}"))?;
        let temporary = self.path.with_file_name(format!(
            ".{}.tmp",
            self.path
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("vocabulary.json")
        ));
        fs::write(&temporary, content).map_err(|error| {
            format!(
                "failed to write vocabulary `{}`: {error}",
                temporary.display()
            )
        })?;
        if self.path.exists() {
            fs::remove_file(&self.path).map_err(|error| {
                format!(
                    "failed to replace vocabulary `{}`: {error}",
                    self.path.display()
                )
            })?;
        }
        fs::rename(&temporary, &self.path).map_err(|error| {
            format!(
                "failed to commit vocabulary `{}`: {error}",
                self.path.display()
            )
        })
    }
}

fn required(field: &str, value: String) -> Result<String, String> {
    let value = value.trim().to_owned();
    if value.is_empty() {
        Err(format!("vocabulary {field} must not be empty"))
    } else {
        Ok(value)
    }
}

fn quarantine(path: &Path) {
    if !path.exists() {
        return;
    }
    let quarantine = path.with_file_name(format!("vocabulary.json.corrupt-{}", now_secs()));
    if let Err(error) = fs::rename(path, &quarantine) {
        eprintln!(
            "[vocabulary] failed to quarantine `{}` as `{}`: {error}",
            path.display(),
            quarantine.display()
        );
    }
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
mod tests {
    use super::*;

    fn temp_data_dir() -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time went backwards")
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("linguaray-vocabulary-{unique}"));
        fs::create_dir_all(&dir).expect("create vocabulary test dir");
        dir
    }

    fn input(word: &str, translation: &str) -> VocabularyEntryInput {
        VocabularyEntryInput {
            id: None,
            word: word.to_owned(),
            translation: translation.to_owned(),
            source_language: "en".to_owned(),
            target_language: "zh-Hans".to_owned(),
            source: "dictionary".to_owned(),
            note: None,
        }
    }

    #[test]
    fn entries_round_trip_and_are_newest_first() {
        let dir = temp_data_dir();
        {
            let mut store = VocabularyStore::load(&dir);
            store.upsert_entry(input("one", "一")).expect("first");
            store.upsert_entry(input("two", "二")).expect("second");
        }
        let store = VocabularyStore::load(&dir);
        let entries = store.list_entries(VocabularyFilter::All, None);
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].word, "two");
        assert_eq!(entries[1].word, "one");
    }

    #[test]
    fn search_and_favorite_filter() {
        let dir = temp_data_dir();
        let mut store = VocabularyStore::load(&dir);
        let first = store.upsert_entry(input("apple", "苹果")).unwrap();
        store.upsert_entry(input("pear", "梨")).unwrap();
        store.set_favorite(&first.id, true).unwrap();
        let favorites = store.list_entries(VocabularyFilter::Favorites, None);
        assert_eq!(favorites.len(), 1);
        assert_eq!(favorites[0].word, "apple");
        let searched = store.list_entries(VocabularyFilter::All, Some("梨"));
        assert_eq!(searched.len(), 1);
        assert_eq!(searched[0].word, "pear");
    }
}
