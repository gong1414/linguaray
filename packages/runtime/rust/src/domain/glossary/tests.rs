use super::*;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_TEMP_DIR_ID: AtomicU64 = AtomicU64::new(0);

fn temp_data_dir() -> PathBuf {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time went backwards")
        .as_nanos();
    let sequence = NEXT_TEMP_DIR_ID.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!(
        "linguaray-glossary-{}-{timestamp}-{sequence}",
        std::process::id()
    ))
}

fn book_input(name: &str) -> GlossaryBookInput {
    GlossaryBookInput {
        id: None,
        name: name.to_owned(),
        enabled: true,
        source_language: None,
        target_language: None,
    }
}

fn entry_input(term: &str, translation: &str) -> GlossaryEntryInput {
    GlossaryEntryInput {
        id: None,
        term: term.to_owned(),
        translation: translation.to_owned(),
        forbidden: Vec::new(),
        note: None,
        case_sensitive: false,
        whole_word: true,
    }
}

fn store_with_terms(terms: &[(&str, &str)]) -> (GlossaryStore, String) {
    let mut store = GlossaryStore::load(temp_data_dir()).expect("failed to load glossary");
    let book = store.upsert_book(book_input("测试")).expect("upsert book");
    for (term, translation) in terms {
        store
            .upsert_entry(&book.id, entry_input(term, translation))
            .expect("upsert entry");
    }
    (store, book.id)
}

#[test]
fn load_missing_directory_returns_empty_glossary() {
    let store = GlossaryStore::load(temp_data_dir()).expect("failed to load glossary");
    assert!(store.list_books().is_empty());
}

#[test]
fn books_and_entries_round_trip_through_disk() {
    let dir = temp_data_dir();
    let book_id = {
        let mut store = GlossaryStore::load(&dir).expect("failed to load glossary");
        let book = store
            .upsert_book(book_input("机器学习"))
            .expect("upsert book");
        store
            .upsert_entry(&book.id, entry_input("token", "词元"))
            .expect("upsert entry");
        book.id
    };

    let store = GlossaryStore::load(&dir).expect("failed to reload glossary");
    let books = store.list_books();
    assert_eq!(books.len(), 1);
    assert_eq!(books[0].name, "机器学习");
    assert_eq!(books[0].entry_count, 1);

    let entries = store.list_entries(&book_id, None, 0, 0).expect("entries");
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].term, "token");
    assert_eq!(entries[0].translation, "词元");
}

#[test]
fn delete_book_removes_its_file() {
    let dir = temp_data_dir();
    let mut store = GlossaryStore::load(&dir).expect("failed to load glossary");
    let book = store.upsert_book(book_input("临时")).expect("upsert book");
    let path = dir.join(GLOSSARY_DIR).join(format!("{}.json", book.id));
    assert!(path.exists());

    assert!(store.delete_book(&book.id).expect("delete book"));
    assert!(!path.exists());
    assert!(!store.delete_book(&book.id).expect("delete missing book"));
}

#[test]
fn upsert_entry_updates_the_existing_term_instead_of_duplicating() {
    let (mut store, book_id) = store_with_terms(&[("token", "词元")]);
    store
        .upsert_entry(&book_id, entry_input("TOKEN", "标记"))
        .expect("upsert entry");

    let entries = store.list_entries(&book_id, None, 0, 0).expect("entries");
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].term, "TOKEN");
    assert_eq!(entries[0].translation, "标记");
}

#[test]
fn renaming_an_entry_onto_another_term_is_rejected() {
    let (mut store, book_id) = store_with_terms(&[("token", "词元"), ("embedding", "嵌入")]);
    let entries = store.list_entries(&book_id, None, 0, 0).expect("entries");
    let embedding = entries
        .iter()
        .find(|entry| entry.term == "embedding")
        .expect("embedding entry");

    let mut input = entry_input("token", "词元");
    input.id = Some(embedding.id.clone());
    let error = store.upsert_entry(&book_id, input).unwrap_err();
    assert!(
        error.contains("already exists"),
        "unexpected error: {error}"
    );
}

#[test]
fn list_entries_filters_by_query_and_paginates() {
    let (store, book_id) = store_with_terms(&[
        ("token", "词元"),
        ("embedding", "嵌入"),
        ("prompt", "提示词"),
    ]);

    let filtered = store
        .list_entries(&book_id, Some("嵌入"), 0, 0)
        .expect("entries");
    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].term, "embedding");

    assert_eq!(store.count_entries(&book_id, None).expect("count"), 3);
    assert_eq!(
        store
            .list_entries(&book_id, None, 1, 1)
            .expect("page")
            .len(),
        1
    );
    assert!(store
        .list_entries(&book_id, None, 9, 1)
        .expect("page past end")
        .is_empty());
}

#[test]
fn matching_prefers_the_longest_term() {
    let (store, _) = store_with_terms(&[("fine", "美好"), ("fine-tune", "微调")]);
    let matches = store.match_text("we fine-tune the model", None, None);
    assert_eq!(matches.len(), 1);
    assert_eq!(matches[0].term, "fine-tune");
    assert_eq!(matches[0].translation, "微调");
}

#[test]
fn matching_is_case_insensitive_by_default() {
    let (store, _) = store_with_terms(&[("token", "词元")]);
    let matches = store.match_text("A Token and a TOKEN", None, None);
    assert_eq!(matches.len(), 2);
    assert_eq!(matches[0].matched_text, "Token");
    assert_eq!(matches[1].matched_text, "TOKEN");
}

#[test]
fn case_sensitive_entries_only_match_their_own_casing() {
    let mut store = GlossaryStore::load(temp_data_dir()).expect("failed to load glossary");
    let book = store.upsert_book(book_input("品牌")).expect("upsert book");
    let mut input = entry_input("IT", "信息技术");
    input.case_sensitive = true;
    store.upsert_entry(&book.id, input).expect("upsert entry");

    let matches = store.match_text("IT is what it is", None, None);
    assert_eq!(matches.len(), 1);
    assert_eq!(matches[0].start, 0);
}

#[test]
fn whole_word_matching_rejects_terms_inside_longer_words() {
    let (store, _) = store_with_terms(&[("art", "艺术")]);
    assert!(store.match_text("started", None, None).is_empty());
    assert_eq!(store.match_text("modern art.", None, None).len(), 1);
}

#[test]
fn whole_word_matching_does_not_block_cjk_terms() {
    let (store, _) = store_with_terms(&[("词元", "token")]);
    assert_eq!(store.match_text("这个词元很重要", None, None).len(), 1);
}

#[test]
fn books_only_apply_to_their_language_pair() {
    let mut store = GlossaryStore::load(temp_data_dir()).expect("failed to load glossary");
    let book = store
        .upsert_book(GlossaryBookInput {
            id: None,
            name: "英译中".to_owned(),
            enabled: true,
            source_language: Some("en".to_owned()),
            target_language: Some("zh-Hans".to_owned()),
        })
        .expect("upsert book");
    store
        .upsert_entry(&book.id, entry_input("token", "词元"))
        .expect("upsert entry");

    // Exact pair, primary-subtag match, unknown and `auto` all apply.
    assert_eq!(
        store.match_text("token", Some("en"), Some("zh-Hans")).len(),
        1
    );
    assert_eq!(store.match_text("token", Some("en"), Some("zh")).len(), 1);
    assert_eq!(store.match_text("token", None, None).len(), 1);
    assert_eq!(
        store
            .match_text("token", Some("auto"), Some("zh-Hans"))
            .len(),
        1
    );
    // A different target language does not.
    assert!(store.match_text("token", Some("en"), Some("ja")).is_empty());
}

#[test]
fn disabled_books_do_not_match() {
    let (mut store, book_id) = store_with_terms(&[("token", "词元")]);
    assert_eq!(store.match_text("token", None, None).len(), 1);

    let book = store.get_book(&book_id).expect("book");
    store
        .upsert_book(GlossaryBookInput {
            id: Some(book.id),
            name: book.name,
            enabled: false,
            source_language: None,
            target_language: None,
        })
        .expect("disable book");
    assert!(store.match_text("token", None, None).is_empty());
}

#[test]
fn edits_invalidate_the_compiled_matcher() {
    let (mut store, book_id) = store_with_terms(&[("token", "词元")]);
    assert_eq!(store.match_text("token", None, None).len(), 1);

    let entries = store.list_entries(&book_id, None, 0, 0).expect("entries");
    store
        .delete_entry(&book_id, &entries[0].id)
        .expect("delete entry");
    assert!(store.match_text("token", None, None).is_empty());
}

#[test]
fn compliance_reports_missing_and_forbidden_translations() {
    let mut store = GlossaryStore::load(temp_data_dir()).expect("failed to load glossary");
    let book = store
        .upsert_book(book_input("机器学习"))
        .expect("upsert book");
    let mut input = entry_input("token", "词元");
    input.forbidden = vec!["标记".to_owned(), "令牌".to_owned()];
    store.upsert_entry(&book.id, input).expect("upsert entry");

    let matches = store.match_text("a token here", None, None);
    assert!(check_compliance(&matches, "这里有一个词元").is_empty());

    let issues = check_compliance(&matches, "这里有一个标记");
    assert_eq!(issues.len(), 2);
    assert_eq!(issues[0].kind, GlossaryIssueKind::MissingTranslation);
    assert_eq!(issues[1].kind, GlossaryIssueKind::ForbiddenUsed);
    assert_eq!(issues[1].found.as_deref(), Some("标记"));
}

#[test]
fn compliance_reports_each_entry_once_however_often_it_matched() {
    let (store, _) = store_with_terms(&[("token", "词元")]);
    let matches = store.match_text("token token token", None, None);
    assert_eq!(matches.len(), 3);
    assert_eq!(check_compliance(&matches, "空译文").len(), 1);
}

#[test]
fn record_hits_counts_every_occurrence() {
    let (mut store, book_id) = store_with_terms(&[("token", "词元")]);
    let matches = store.match_text("token and token", None, None);
    store.record_hits(&matches);
    store.flush_hits().expect("flush hits");

    let entries = store.list_entries(&book_id, None, 0, 0).expect("entries");
    assert_eq!(entries[0].hits, 2);
}

#[test]
fn hit_counts_survive_a_reload() {
    let dir = temp_data_dir();
    let book_id = {
        let mut store = GlossaryStore::load(&dir).expect("failed to load glossary");
        let book = store
            .upsert_book(book_input("机器学习"))
            .expect("upsert book");
        store
            .upsert_entry(&book.id, entry_input("token", "词元"))
            .expect("upsert entry");
        let matches = store.match_text("token", None, None);
        store.record_hits(&matches);
        store.flush_hits().expect("flush hits");
        book.id
    };

    let store = GlossaryStore::load(&dir).expect("failed to reload glossary");
    let entries = store.list_entries(&book_id, None, 0, 0).expect("entries");
    assert_eq!(entries[0].hits, 1);
}

#[test]
fn book_ids_that_could_escape_the_directory_are_rejected() {
    let mut store = GlossaryStore::load(temp_data_dir()).expect("failed to load glossary");
    assert!(store.delete_book("../settings").is_err());
    assert!(store
        .upsert_entry("../settings", entry_input("token", "词元"))
        .is_err());
}

#[test]
fn a_corrupt_book_does_not_break_the_rest_of_the_glossary() {
    let dir = temp_data_dir();
    let mut store = GlossaryStore::load(&dir).expect("failed to load glossary");
    store.upsert_book(book_input("好的")).expect("upsert book");
    fs::write(dir.join(GLOSSARY_DIR).join("broken.json"), "{not json")
        .expect("failed to write corrupt book");

    let store = GlossaryStore::load(&dir).expect("failed to reload glossary");
    assert_eq!(store.list_books().len(), 1);
}
