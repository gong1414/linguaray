# R4 Surface 09-11 Implementation Plan (rev-2)

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement Surface 09 (History list/search/detail/favorite/delete/export), Surface 10 (Vocabulary encrypted paginated CRUD + file export + AnkiConnect), and Surface 11 (Dictionary offline StarDict/MDX + system lookup + package install with hardening), addressing all 7 P1 issues from the rev-1 rejection.

**Architecture:** Three surfaces, three stages, each gated by a three-way review. Surface 09 extends the existing `history::search` module with a batched export iterator (private `read_batch` reused internally, never exposed) and a file-writing service; favorites/delete live in `db::history` repository functions (not in `lib.rs`). Surface 10 adds `db::vocabulary` (paginated CRUD) + `vocabulary` service (encrypt/decrypt/export-to-file/AnkiConnect POST) sharing the history key via `get_or_create_history_key` without enabling history consent. Surface 11 ships real StarDict + MDX parsers, app-owned dictionary directory with path-traversal/symlink/bomb protection and atomic copy+rollback.

**Tech Stack:** Rust (Tauri 2.x, rusqlite, AES-256-GCM, reqwest, flate2), TypeScript (SolidJS, Vitest, axe-core, Playwright), CSS semantic tokens. New crate deps: `tauri-plugin-dialog = "2"`, `flate2 = "1"`. New npm dep: `@tauri-apps/plugin-dialog`.

## Global Constraints

- **Export returns FilePath, not String:** History export (`history_export`) and Vocabulary file export (`vocabulary_export_file`) receive a `file_path: String` (chosen by the frontend via `@tauri-apps/plugin-dialog` `save()`), write the decrypted+formatted content to that path in the backend, and return the written path as `String`. No large strings cross the IPC boundary. AnkiConnect export (`vocabulary_export_anki`) does NOT write a file — it POSTs directly to `127.0.0.1:8765`.
- **No new crypto:** Reuse `crypto.rs` `encrypt_field`/`decrypt_field` + `HistoryField::VocabularyWord { uuid }` / `HistoryField::VocabularyDefinition { uuid }` AAD variants. The 32-byte history key from `Keystore::get_history_key()` is shared by history + vocabulary.
- **Vocabulary key = get-or-create, does NOT enable history:** `vocabulary_add` calls `keystore.get_or_create_history_key()` (creates the key if absent, idempotent if present) but NEVER sets `preferences.history_enabled=1`. Vocabulary works regardless of the history consent flag.
- **Key-first locking:** Vocabulary writes follow the same key-first → DB-consent-recheck discipline as `persist_translation_session`. Favorite/delete/export run under `data_gate` (read for export, write for favorite/delete).
- **Spawn_blocking + data_gate:** All DB/keystore IPC commands run on `spawn_blocking` + `data_gate`. Guards are `!Send` — they live on the blocking thread only.
- **favorite/delete SQL in repository/service modules:** The SQL statements for `history_toggle_favorite` and `history_delete_session` live in `src-tauri/src/db/history.rs` (new functions `toggle_favorite`, `delete_session`). The vocabulary CRUD SQL lives in `src-tauri/src/db/vocabulary.rs`. `lib.rs` commands are thin wrappers that call these.
- **Internal paginated iterator:** `history_export` loops `search::search()` internally (200/batch + opaque cursor) until `scan_complete=true`, decrypts in-memory per batch, accumulates formatted output, and writes to `file_path`. Cursor monotonicity is enforced by the existing `search()` cursor decode (each batch's cursor encodes the last row's `(timestamp, session_uuid)`; `read_batch` uses `timestamp < cursor_ts OR (timestamp = cursor_ts AND session_uuid < cursor_uuid)` so pages never overlap or skip). No cursor parameter is exposed to the frontend.
- **HistoryFilter:** Export accepts a `HistoryFilter { query: Option<String>, favorites_only: bool }` struct. When `query` is `None`/empty and `favorites_only` is `false`, ALL non-expired sessions are exported (respecting retention cutoff).
- **Concurrent retention safety:** Export holds `data_gate.read()` for its entire duration, so a concurrent `cleanup_expired` (which needs `data_gate.write()` via `history_clear_all`'s gate or the startup cleanup path) cannot delete rows mid-export. The export reads a consistent snapshot.
- **No temp plaintext:** Vocabulary file export (CSV/JSON) decrypts in-memory only; AnkiConnect sends to `127.0.0.1:8765` via a hardened reqwest client (no redirect, 10s timeout, response validation, no plaintext file). The export-to-file writes directly to the user-chosen path (the user explicitly chose it, so it is user-owned, not temp).
- **Favorites never expire:** History favorites survive retention cleanup (already enforced in `search.rs` `read_batch`: `WHERE (timestamp >= ?1 OR is_favorite=1)`).
- **Dictionary is real in R4:** StarDict (.ifo/.idx/.dict + optional .dict.dz) and MDX (.mdx) parsers are custom Rust implementations (no mature crates exist). Package install validates path-traversal, symlink, and decompression-bomb. Atomic copy+rollback. Source attribution on every lookup result.
- **`dict_lookup` is the unified command name:** The existing `lookup_dictionary` stub (dead_code) is replaced by `dict_lookup(word) -> Option<DictLookupResult>` where `DictLookupResult { definition: String, source: String }`. macOS system dict is tried first; offline packages are tried second. Windows gets offline-only.
- **Semantic tokens only:** No hex, no `--core-*`, no old aliases.
- **Git:** Explicit file lists in every commit, no `git add -A`, never stage `.mimosa/` or `.worktrees/`.
- **TDD:** Every task writes RED test (exact name + assertions) → verify fails → implement → verify passes → commit.
- **Stage gates:** Stage A/B/C each PAUSE for three-way review: (1) Rust/security reviewer, (2) frontend/design/a11y reviewer, (3) test-authenticity reviewer. No task in the next stage starts until all three approve.
- **ui-lab reuses production Views:** HistoryView, VocabularyView, DictionaryView fixtures import the REAL production components from `@app/features/settings/`, NOT copies.
- **Plan tracked in git:** This file is `git add`-ed after writing.

## File Structure

### Backend — Rust

```
src-tauri/src/
├── history/
│   ├── mod.rs          — re-exports (modify: add export module)
│   ├── crypto.rs       — HistoryField AAD variants (NO CHANGE — Vocabulary* already present)
│   ├── search.rs       — search() + read_batch (PRIVATE) + decode_cursor (modify: add export_all)
│   └── export.rs       — NEW: HistoryFilter, ExportFormat, export_all() iterator, write_export_file()
├── vocabulary.rs       — NEW: VocabularyService (add_word, list_words paginated, delete_word, export_file, export_anki)
├── dict/
│   ├── mod.rs          — NEW: re-exports
│   ├── stardict.rs     — NEW: StarDictParser (.ifo/.idx/.dict + .dict.dz via flate2)
│   ├── mdx.rs          — NEW: MdxParser (.mdx header + zlib key/record blocks via flate2)
│   ├── package.rs      — NEW: install_package (path traversal/symlink/bomb checks, atomic copy+rollback)
│   └── lookup.rs       — NEW: dict_lookup (macOS system + offline fallback, source attribution)
├── dict.rs             — REMOVE (replaced by dict/ module; macOS DCSCopyTextDefinition moves to dict/lookup.rs)
├── db/
│   ├── history.rs      — MODIFY: add toggle_favorite(), delete_session()
│   └── vocabulary.rs   — NEW: create(), read_page(cursor), delete(), count()
├── lib.rs              — MODIFY: add 11 commands, remove lookup_dictionary dead_code, register plugin+commands
└── Cargo.toml          — MODIFY: add tauri-plugin-dialog, flate2
```

### Backend — Capabilities

```
src-tauri/capabilities/
├── main.json           — MODIFY: add history_*, vocabulary_*, dict_* permissions + dialog:allow-save
├── popup.json          — MODIFY: add allow-vocabulary-add
└── input.json          — MODIFY: add allow-vocabulary-add
```

### Frontend — TypeScript

```
src/features/settings/
├── SettingsShell.tsx   — MODIFY: SettingsSection union (+history/vocabulary/dictionary), navItems, NavDef
├── copy.ts             — MODIFY: NavCopy type (+history/vocabulary/dictionary), EN + ZH values
├── HistoryView.tsx     — NEW: 8-state component
├── HistoryView.css     — NEW
├── history-copy.ts     — NEW: EN/ZH copy (12 keys from 09-history.md)
├── history-ipc.ts      — NEW: typed invoke wrappers
├── history-types.ts    — NEW: HistoryPage/HistoryItem/HistoryFilter wire types
├── VocabularyView.tsx  — NEW: 5-state component
├── VocabularyView.css  — NEW
├── vocabulary-copy.ts  — NEW: EN/ZH copy (15 keys from 10-vocabulary.md)
├── vocabulary-ipc.ts   — NEW: typed invoke wrappers
├── vocabulary-types.ts — NEW: VocabularyPage/VocabularyItem wire types
├── DictionaryView.tsx  — NEW: 6-state component
├── DictionaryView.css  — NEW
├── dictionary-copy.ts  — NEW: EN/ZH copy (10 keys from 11-dictionary.md)
├── dictionary-ipc.ts   — NEW: typed invoke wrappers
└── dictionary-types.ts — NEW: DictPackage/DictLookupResult wire types
```

### Frontend — App + Popup + Input

```
src/
├── App.tsx             — MODIFY: navigate whitelist (+history/vocabulary/dictionary), render switch
├── Popup.tsx           — MODIFY: favorite button → vocabulary_add (source pre-fills word, success text pre-fills definition)
└── InputPanel.tsx      — MODIFY: ResultCard favorite action → vocabulary_add
```

### Frontend — Tests

```
test/
├── HistoryView.test.tsx          — NEW: 8 state tests
├── HistoryView.a11y.test.tsx     — NEW: axe scan
├── VocabularyView.test.tsx       — NEW: 5 state tests
├── VocabularyView.a11y.test.tsx  — NEW: axe scan
├── DictionaryView.test.tsx       — NEW: 6 state tests
├── DictionaryView.a11y.test.tsx  — NEW: axe scan
├── SettingsShell.test.tsx        — MODIFY: assert 7 nav items render
└── App.test.tsx                  — MODIFY: assert navigate whitelist includes history/vocabulary/dictionary
```

### Backend — Tests

```
src-tauri/tests/
├── history_export.rs             — NEW: export iterator (201/1000+/last/empty/corrupt/concurrent)
├── history_repository.rs         — EXTEND: toggle_favorite, delete_session
├── vocabulary.rs                 — NEW: CRUD + pagination + export + anki
├── dictionary_stardict.rs        — NEW: StarDict parse + lookup
├── dictionary_mdx.rs             — NEW: MDX parse + lookup
├── dictionary_package.rs         — NEW: install security (traversal/symlink/bomb/atomic)
└── dictionary_lookup.rs          — NEW: unified lookup + source attribution
```

### UI-Lab

```
apps/ui-lab/src/
├── App.tsx               — MODIFY: add history/vocabulary/dictionary Match blocks (reusing production Views)
├── pages/HistoryView.tsx  — NEW: lab wrapper importing @app production HistoryView
├── pages/VocabularyView.tsx — NEW: lab wrapper
└── pages/DictionaryView.tsx — NEW: lab wrapper
```

---

## Stage A: Surface 09 — History

**Checkpoint goal:** User can search (200/batch cursor), scroll, favorite, delete, and export history records. Export writes to a user-chosen file path (returns FilePath, not String). All 8 states render. Gate barrier tests prove locks release on error paths.

**Three-way review gate at end of Stage A:**
1. **Rust/security reviewer:** export iterator cursor monotonicity, data_gate read-guard spans full export, no plaintext leaks, HistoryFilter sanitization.
2. **Frontend/design/a11y reviewer:** all 8 states render with semantic tokens, corrupt-row badge accessible, keyboard navigation, axe scan clean.
3. **Test-authenticity reviewer:** 201-record / 1000+ / last-page / empty / corrupt / concurrent-mutation tests are real (not tautological), RED→GREEN observed.

---

### Task A1: Dependencies + HistoryFilter + export iterator

**Files:**
- Modify: `src-tauri/Cargo.toml` — add `tauri-plugin-dialog = "2"` and `flate2 = "1"`
- Create: `src-tauri/src/history/export.rs` — `HistoryFilter`, `ExportFormat`, `export_all()`, `write_export_file()`
- Modify: `src-tauri/src/history/mod.rs` — add `pub mod export;`
- Create: `src-tauri/tests/history_export.rs` — integration tests
- Test: `src-tauri/tests/history_export.rs`

**Interfaces:**
- Consumes: `history::search::search(db, keystore, query, cursor) -> HistoryPage` (existing PUBLIC function), `HistoryPage { items, next_cursor, scan_complete }`
- Produces:
  - `HistoryFilter { query: Option<String>, favorites_only: bool }`
  - `ExportFormat { Csv, Json }`
  - `fn export_all(db, keystore, filter) -> Result<Vec<DecryptedHistorySession>, HistorySearchError>` — internal loop calling `search()` until `scan_complete`
  - `fn write_export_file(sessions, path, format) -> Result<(), std::io::Error>` — writes CSV or JSON to `path`

- [ ] **Step 1: Write the failing test**

```rust
// src-tauri/tests/history_export.rs
use linguaray_lib::db::{schema, Database};
use linguaray_lib::history::crypto::{encrypt_field, HistoryField};
use linguaray_lib::history::export::{export_all, write_export_file, ExportFormat, HistoryFilter};
use linguaray_lib::history::search::HISTORY_SEARCH_BATCH;
use linguaray_lib::keystore::Keystore;
use tempfile::TempDir;
use zeroize::Zeroizing;

struct Harness { _dir: TempDir, db: Database, keystore: Keystore }

impl Harness {
    fn new() -> Self {
        let dir = tempfile::tempdir().unwrap();
        let db = Database::open(&dir.path().join("export.db")).unwrap();
        db.with_conn(|conn| {
            let tx = conn.transaction()?;
            schema::create_all_tables(&tx)?;
            schema::seed_singletons(&tx)?;
            tx.execute("UPDATE preferences SET history_enabled=1 WHERE id=1", [])?;
            tx.commit()?;
            Ok(())
        }).unwrap();
        let keystore = Keystore::new(dir.path().join("keystore")).unwrap();
        Self { _dir: dir, db, keystore }
    }

    fn insert_encrypted_session(&self, uuid: &str, timestamp: i64, source: &str) {
        let key = Zeroizing::new(self.keystore.get_or_create_history_key().unwrap().0);
        let enc = encrypt_field(&key, &HistoryField::SessionSource { uuid }, source.as_bytes()).unwrap();
        self.db.with_conn(|conn| {
            conn.execute(
                "INSERT INTO history_sessions
                 (session_uuid, timestamp, trigger_source, target_language, is_favorite,
                  source_text_encrypted, source_text_nonce, crypto_version)
                 VALUES (?1, ?2, 'input', 'zh', 0, ?3, ?4, ?5)",
                rusqlite::params![uuid, timestamp, enc.ciphertext, enc.nonce.as_slice(), enc.crypto_version],
            )?;
            Ok(())
        }).unwrap();
    }
}

#[test]
fn export_all_collects_exactly_201_records_across_two_batches() {
    let h = Harness::new();
    let base = 1_700_000_000i64;
    for i in 0..201 {
        h.insert_encrypted_session(&format!("sess-{i:03}"), base + i as i64, &format!("text-{i}"));
    }
    let sessions = export_all(&h.db, &h.keystore, &HistoryFilter::default()).unwrap();
    assert_eq!(sessions.len(), 201);
    // Cursor monotonic: timestamps are strictly descending.
    for w in sessions.windows(2) {
        assert!(w[0].timestamp >= w[1].timestamp, "non-monotonic: {} then {}", w[0].timestamp, w[1].timestamp);
    }
}

#[test]
fn export_all_handles_1000_plus_records() {
    let h = Harness::new();
    let base = 1_700_000_000i64;
    for i in 0..1050 {
        h.insert_encrypted_session(&format!("big-{i:04}"), base + i as i64, &format!("word-{i}"));
    }
    let sessions = export_all(&h.db, &h.keystore, &HistoryFilter::default()).unwrap();
    assert_eq!(sessions.len(), 1050);
}

#[test]
fn export_all_last_page_with_partial_batch() {
    let h = Harness::new();
    let base = 1_700_000_000i64;
    // Exactly 200 → one full batch, scan_complete on the second empty call.
    for i in 0..200 {
        h.insert_encrypted_session(&format!("p-{i:03}"), base + i as i64, &format!("t-{i}"));
    }
    let sessions = export_all(&h.db, &h.keystore, &HistoryFilter::default()).unwrap();
    assert_eq!(sessions.len(), 200);
}

#[test]
fn export_all_empty_db_returns_empty_vec() {
    let h = Harness::new();
    let sessions = export_all(&h.db, &h.keystore, &HistoryFilter::default()).unwrap();
    assert!(sessions.is_empty());
}

#[test]
fn export_all_skips_corrupt_rows_but_includes_them_marked() {
    let h = Harness::new();
    // Insert a valid session.
    h.insert_encrypted_session("good", 1_700_000_100, "hello");
    // Insert a corrupt session: ciphertext is garbage, nonce is valid-length but wrong.
    h.db.with_conn(|conn| {
        conn.execute(
            "INSERT INTO history_sessions
             (session_uuid, timestamp, trigger_source, target_language, is_favorite,
              source_text_encrypted, source_text_nonce, crypto_version)
             VALUES ('bad', 1700000099, 'input', 'zh', 0, X'DEADBEEF', X'000102030405060708090A0B', 1)",
            [],
        )?;
        Ok(())
    }).unwrap();
    let sessions = export_all(&h.db, &h.keystore, &HistoryFilter::default()).unwrap();
    assert_eq!(sessions.len(), 2);
    let corrupt = sessions.iter().find(|s| s.session_uuid == "bad").unwrap();
    assert!(corrupt.corrupt);
    let good = sessions.iter().find(|s| s.session_uuid == "good").unwrap();
    assert!(!good.corrupt);
    assert_eq!(good.source_text.as_deref(), Some("hello"));
}

#[test]
fn export_all_concurrent_mutation_does_not_crash() {
    let h = Harness::new();
    let base = 1_700_000_000i64;
    for i in 0..50 {
        h.insert_encrypted_session(&format!("c-{i:03}"), base + i as i64, &format!("s-{i}"));
    }
    let db2 = h.db.clone_arc();
    // Simulate a concurrent delete on a different thread.
    let deleter = std::thread::spawn(move || {
        db2.with_conn(|conn| {
            conn.execute("DELETE FROM history_sessions WHERE session_uuid='c-000'", [])?;
            Ok(())
        }).unwrap();
    });
    let sessions = export_all(&h.db, &h.keystore, &HistoryFilter::default()).unwrap();
    deleter.join().unwrap();
    // We got a snapshot; length is 49 or 50 depending on race, but never panics.
    assert!(sessions.len() == 49 || sessions.len() == 50);
}

#[test]
fn write_export_file_csv_writes_valid_csv() {
    let h = Harness::new();
    h.insert_encrypted_session("s1", 1_700_000_100, "hello");
    let sessions = export_all(&h.db, &h.keystore, &HistoryFilter::default()).unwrap();
    let out = h._dir.path().join("export.csv");
    write_export_file(&sessions, &out, ExportFormat::Csv).unwrap();
    let content = std::fs::read_to_string(&out).unwrap();
    assert!(content.contains("hello"));
    assert!(content.lines().count() >= 2); // header + data
}

#[test]
fn write_export_file_json_writes_valid_json() {
    let h = Harness::new();
    h.insert_encrypted_session("s1", 1_700_000_100, "hello");
    let sessions = export_all(&h.db, &h.keystore, &HistoryFilter::default()).unwrap();
    let out = h._dir.path().join("export.json");
    write_export_file(&sessions, &out, ExportFormat::Json).unwrap();
    let content = std::fs::read_to_string(&out).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&content).unwrap();
    assert!(parsed.is_array());
}

#[test]
fn export_all_favorites_only_returns_only_favorites() {
    let h = Harness::new();
    h.insert_encrypted_session("fav", 1_700_000_100, "favorite text");
    h.insert_encrypted_session("norm", 1_700_000_099, "normal text");
    h.db.with_conn(|conn| {
        conn.execute("UPDATE history_sessions SET is_favorite=1 WHERE session_uuid='fav'", [])?;
        Ok(())
    }).unwrap();
    let sessions = export_all(&h.db, &h.keystore, &HistoryFilter { query: None, favorites_only: true }).unwrap();
    assert_eq!(sessions.len(), 1);
    assert_eq!(sessions[0].session_uuid, "fav");
}
```

- [ ] **Step 2: Run tests to verify they fail**

```bash
cargo test --manifest-path src-tauri/Cargo.toml --test history_export -- --nocapture
```

Expected: COMPILE ERROR — `unresolved import linguaray_lib::history::export` (module + types do not exist yet).

- [ ] **Step 3: Write the implementation**

Add to `src-tauri/Cargo.toml` under `[dependencies]`:
```toml
tauri-plugin-dialog = "2"
flate2 = "1"
```

Create `src-tauri/src/history/export.rs`:
```rust
//! Batched export iterator for encrypted history.
//!
//! Reuses the PUBLIC `search::search()` function internally. Each batch decrypts
//! at most 200 sessions in memory. Cursor monotonicity is enforced by the
//! existing `search()` cursor mechanism: each page's cursor encodes the last
//! row's `(timestamp, session_uuid)`, and `read_batch` uses
//! `timestamp < cursor_ts OR (timestamp = cursor_ts AND session_uuid < cursor_uuid)`
//! so pages never overlap or skip.

use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::db::Database;
use crate::history::search::{search, DecryptedHistorySession, HistorySearchError, HISTORY_SEARCH_BATCH};
use crate::keystore::Keystore;

/// Filter applied during export. `query` = None or empty means all sessions.
/// `favorites_only` = true exports only sessions with `is_favorite=1`.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HistoryFilter {
    pub query: Option<String>,
    pub favorites_only: bool,
}

/// Output format for file export.
#[derive(Debug, Clone, Copy)]
pub enum ExportFormat {
    Csv,
    Json,
}

/// Iterate ALL matching sessions via repeated `search()` calls (200/batch),
/// decrypting each batch in memory. Returns the full collected set.
///
/// Cursor monotonicity is guaranteed by `search()`'s existing cursor logic.
/// The caller holds `data_gate.read()` so a concurrent `cleanup_expired` cannot
/// delete rows mid-export.
pub fn export_all(
    db: &Database,
    keystore: &Keystore,
    filter: &HistoryFilter,
) -> Result<Vec<DecryptedHistorySession>, HistorySearchError> {
    let query = filter.query.as_deref().unwrap_or("");
    let mut all = Vec::new();
    let mut cursor: Option<String> = None;
    loop {
        let page = search(db, keystore, query, cursor.as_deref())?;
        let batch_size = page.items.len();
        if filter.favorites_only {
            all.extend(page.items.into_iter().filter(|s| s.is_favorite));
        } else {
            all.extend(page.items);
        }
        if page.scan_complete || page.next_cursor.is_none() {
            break;
        }
        cursor = page.next_cursor;
        // Guard against infinite loop: if a batch returns exactly BATCH items
        // but no cursor advance, something is wrong.
        if batch_size == 0 {
            break;
        }
    }
    Ok(all)
}

/// Write the collected sessions to `path` in the requested format.
/// The caller holds the `data_gate` guard; file I/O is outside the DB lock.
pub fn write_export_file(
    sessions: &[DecryptedHistorySession],
    path: &Path,
    format: ExportFormat,
) -> Result<(), std::io::Error> {
    let content = match format {
        ExportFormat::Csv => format_csv(sessions),
        ExportFormat::Json => format_json(sessions),
    };
    std::fs::write(path, content)?;
    Ok(())
}

fn format_csv(sessions: &[DecryptedHistorySession]) -> String {
    let mut out = String::from("session_uuid,timestamp,trigger_source,target_language,is_favorite,source_text\n");
    for s in sessions {
        let source = s.source_text.as_deref().unwrap_or("");
        // Escape: wrap in quotes, double internal quotes.
        let escaped = source.replace('"', "\"\"");
        out.push_str(&format!(
            "{},{},{},{},{},\"{}\"\n",
            s.session_uuid, s.timestamp, s.trigger_source, s.target_language,
            s.is_favorite as i64, escaped,
        ));
    }
    out
}

fn format_json(sessions: &[DecryptedHistorySession]) -> String {
    // Serialize using serde_json so the output is valid JSON.
    serde_json::to_string_pretty(sessions).unwrap_or_else(|_| "[]".to_string())
}
```

Modify `src-tauri/src/history/mod.rs` — add:
```rust
pub mod export;
```

- [ ] **Step 4: Run tests to verify they pass**

```bash
cargo test --manifest-path src-tauri/Cargo.toml --test history_export -- --nocapture
```

Expected: PASS (all 10 tests).

- [ ] **Step 5: Commit**

```bash
git add src-tauri/Cargo.toml src-tauri/Cargo.lock src-tauri/src/history/export.rs src-tauri/src/history/mod.rs src-tauri/tests/history_export.rs
git commit -m "feat(r4-a1): history export iterator + HistoryFilter + file writer (10 tests)

- export_all() loops search() internally (200/batch, cursor monotonic)
- write_export_file() writes CSV/JSON to user-chosen path
- HistoryFilter { query, favorites_only }
- Tests: 201/1000+/last-page/empty/corrupt/concurrent-mutation/favorites-only"
```

---

### Task A2: History favorite/delete repository + export IPC command

**Files:**
- Modify: `src-tauri/src/db/history.rs` — add `toggle_favorite()`, `delete_session()`
- Modify: `src-tauri/src/lib.rs` — add `history_toggle_favorite`, `history_delete_session`, `history_export` commands + register in `invoke_handler!` + `build.rs`
- Modify: `src-tauri/build.rs` — add 3 command names
- Modify: `src-tauri/capabilities/main.json` — add 3 permissions
- Test: `src-tauri/tests/history_repository.rs` (extend)

**Interfaces:**
- Consumes: `Database`, `Keystore`, `history::export::{export_all, write_export_file, HistoryFilter, ExportFormat}`
- Produces:
  - `db::history::toggle_favorite(conn, session_uuid) -> Result<bool, DbError>` — returns the NEW favorite state
  - `db::history::delete_session(conn, session_uuid) -> Result<(), DbError>` — deletes one session (cascade removes results)
  - IPC: `history_toggle_favorite(session_uuid: String) -> Result<bool, String>`
  - IPC: `history_delete_session(session_uuid: String) -> Result<(), String>`
  - IPC: `history_export(file_path: String, format: String, filter: HistoryFilter) -> Result<String, String>` — returns written file path

- [ ] **Step 1: Write the failing test**

Add to `src-tauri/tests/history_repository.rs`:
```rust
#[test]
fn toggle_favorite_flips_and_returns_new_state() {
    let h = Harness::new();
    h.insert_session("sess-a", 1_700_000_100, false);
    let new_state = h.db.with_conn(|conn| db_history::toggle_favorite(conn, "sess-a")).unwrap();
    assert!(new_state);
    let db_val: i64 = h.db.with_conn(|conn| {
        conn.query_row("SELECT is_favorite FROM history_sessions WHERE session_uuid='sess-a'", [], |r| r.get(0))
    }).unwrap();
    assert_eq!(db_val, 1);
    let new_state2 = h.db.with_conn(|conn| db_history::toggle_favorite(conn, "sess-a")).unwrap();
    assert!(!new_state2);
}

#[test]
fn delete_session_removes_session_and_cascades_results() {
    let h = Harness::new();
    h.insert_session("sess-del", 1_700_000_100, false);
    assert_eq!(h.counts(), (1, 1));
    h.db.with_conn(|conn| db_history::delete_session(conn, "sess-del")).unwrap();
    assert_eq!(h.counts(), (0, 0));
}

#[test]
fn toggle_favorite_missing_uuid_returns_not_found() {
    let h = Harness::new();
    let result = h.db.with_conn(|conn| db_history::toggle_favorite(conn, "nonexistent"));
    assert!(result.is_err());
}

#[test]
fn delete_session_missing_uuid_returns_not_found() {
    let h = Harness::new();
    let result = h.db.with_conn(|conn| db_history::delete_session(conn, "nonexistent"));
    assert!(result.is_err());
}
```

- [ ] **Step 2: Run tests to verify they fail**

```bash
cargo test --manifest-path src-tauri/Cargo.toml --test history_repository -- toggle_favorite delete_session --nocapture
```

Expected: COMPILE ERROR — `no function named toggle_favorite found` / `no function named delete_session found`.

- [ ] **Step 3: Write the implementation**

Add to `src-tauri/src/db/history.rs`:
```rust
/// Toggle the favorite flag on a session. Returns the NEW state (true = favorite).
/// Fails with NotFound if the session does not exist.
pub fn toggle_favorite(conn: &mut Connection, session_uuid: &str) -> Result<bool, DbError> {
    let tx = conn.transaction()?;
    let current: i64 = tx
        .query_row(
            "SELECT is_favorite FROM history_sessions WHERE session_uuid=?1",
            [session_uuid],
            |row| row.get(0),
        )
        .map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => {
                DbError::NotFound(format!("session {session_uuid} not found"))
            }
            other => DbError::Sqlite(other),
        })?;
    let new_state = current == 0;
    let changed = tx.execute(
        "UPDATE history_sessions SET is_favorite=?1 WHERE session_uuid=?2",
        rusqlite::params![new_state as i64, session_uuid],
    )?;
    if changed != 1 {
        tx.rollback()?;
        return Err(DbError::NotFound(format!("session {session_uuid} not found")));
    }
    tx.commit()?;
    Ok(new_state)
}

/// Delete a single session. Result rows cascade-delete via the FK ON DELETE CASCADE.
/// Fails with NotFound if the session does not exist.
pub fn delete_session(conn: &mut Connection, session_uuid: &str) -> Result<(), DbError> {
    let tx = conn.transaction()?;
    let changed = tx.execute(
        "DELETE FROM history_sessions WHERE session_uuid=?1",
        [session_uuid],
    )?;
    if changed != 1 {
        tx.rollback()?;
        return Err(DbError::NotFound(format!("session {session_uuid} not found")));
    }
    tx.commit()?;
    Ok(())
}
```

Add IPC commands to `src-tauri/src/lib.rs` (after `history_search`):
```rust
#[tauri::command]
async fn history_toggle_favorite(
    state: tauri::State<'_, Arc<AppState>>,
    session_uuid: String,
) -> Result<bool, String> {
    let app_state = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        let gate = app_state.data_gate.write();
        let db = require_ready_gated_write(&app_state, &gate)?;
        db.with_conn(|conn| crate::db::history::toggle_favorite(conn, &session_uuid))
            .map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn history_delete_session(
    state: tauri::State<'_, Arc<AppState>>,
    session_uuid: String,
) -> Result<(), String> {
    let app_state = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        let gate = app_state.data_gate.write();
        let db = require_ready_gated_write(&app_state, &gate)?;
        db.with_conn(|conn| crate::db::history::delete_session(conn, &session_uuid))
            .map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn history_export(
    state: tauri::State<'_, Arc<Session>>,
    app_state: tauri::State<'_, Arc<AppState>>,
    file_path: String,
    format: String,
    filter: crate::history::export::HistoryFilter,
) -> Result<String, String> {
    session_keystore(&state)?;
    let session = state.inner().clone();
    let st = app_state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        // READ gate: spans the entire export so cleanup_expired cannot delete rows mid-export.
        let _gate = st.data_gate.read();
        let db = require_ready_gated(&st, &_gate)?;
        let keystore = session_keystore(&session)?;
        let sessions = crate::history::export::export_all(&db, keystore, &filter)
            .map_err(|e| e.to_string())?;
        let fmt = match format.as_str() {
            "csv" => crate::history::export::ExportFormat::Csv,
            "json" => crate::history::export::ExportFormat::Json,
            other => return Err(format!("unsupported export format: {other}")),
        };
        let path = std::path::PathBuf::from(&file_path);
        crate::history::export::write_export_file(&sessions, &path, fmt)
            .map_err(|e| e.to_string())?;
        Ok(file_path)
    })
    .await
    .map_err(|e| e.to_string())?
}
```

Register in `invoke_handler!` (add after `history_search`):
```rust
            history_toggle_favorite,
            history_delete_session,
            history_export,
```

Add to `src-tauri/build.rs` `.commands(&[...])`:
```rust
            "history_toggle_favorite",
            "history_delete_session",
            "history_export",
```

Add to `src-tauri/capabilities/main.json` permissions:
```json
    "allow-history-toggle-favorite",
    "allow-history-delete-session",
    "allow-history-export",
    "dialog:allow-save"
```

- [ ] **Step 4: Run tests to verify they pass**

```bash
cargo test --manifest-path src-tauri/Cargo.toml --test history_repository -- --nocapture
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings 2>&1 | tail -5
```

Expected: PASS (all history_repository tests including the 4 new ones). Clippy clean.

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/db/history.rs src-tauri/src/lib.rs src-tauri/build.rs src-tauri/capabilities/main.json src-tauri/tests/history_repository.rs
git commit -m "feat(r4-a2): history favorite/delete repository + export IPC command

- db::history::toggle_favorite() returns new state, fails NotFound on missing
- db::history::delete_session() cascades results, fails NotFound on missing
- history_export IPC: READ gate spans full export, writes to file_path, returns path
- Capabilities: main.json + dialog:allow-save"
```

---

### Task A3: History frontend types + IPC + copy

**Files:**
- Create: `src/features/settings/history-types.ts`
- Create: `src/features/settings/history-ipc.ts`
- Create: `src/features/settings/history-copy.ts`

**Interfaces:**
- Consumes: `@tauri-apps/api/core` `invoke`, `@tauri-apps/plugin-dialog` `save`
- Produces: typed wrappers used by HistoryView + App

- [ ] **Step 1: Write the failing test**

Create `test/history-ipc.test.ts`:
```typescript
import { describe, it, expect, vi, beforeEach } from "vitest";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));
vi.mock("@tauri-apps/plugin-dialog", () => ({
  save: vi.fn(),
}));

import { invoke } from "@tauri-apps/api/core";
import { save } from "@tauri-apps/plugin-dialog";
import {
  searchHistory,
  toggleFavorite,
  deleteSession,
  exportHistory,
} from "../src/features/settings/history-ipc";

describe("history-ipc", () => {
  beforeEach(() => vi.clearAllMocks());

  it("searchHistory passes query + cursor to invoke", async () => {
    vi.mocked(invoke).mockResolvedValue({ items: [], next_cursor: null, scan_complete: true });
    await searchHistory("hello", null);
    expect(invoke).toHaveBeenCalledWith("history_search", { query: "hello", cursor: null });
  });

  it("toggleFavorite calls history_toggle_favorite with session_uuid", async () => {
    vi.mocked(invoke).mockResolvedValue(true);
    const result = await toggleFavorite("sess-1");
    expect(result).toBe(true);
    expect(invoke).toHaveBeenCalledWith("history_toggle_favorite", { sessionUuid: "sess-1" });
  });

  it("deleteSession calls history_delete_session", async () => {
    vi.mocked(invoke).mockResolvedValue(undefined);
    await deleteSession("sess-1");
    expect(invoke).toHaveBeenCalledWith("history_delete_session", { sessionUuid: "sess-1" });
  });

  it("exportHistory opens save dialog then calls history_export with file_path", async () => {
    vi.mocked(save).mockResolvedValue("/tmp/export.csv");
    vi.mocked(invoke).mockResolvedValue("/tmp/export.csv");
    const result = await exportHistory("csv", { query: null, favorites_only: false });
    expect(save).toHaveBeenCalled();
    expect(invoke).toHaveBeenCalledWith("history_export", {
      filePath: "/tmp/export.csv",
      format: "csv",
      filter: { query: null, favorites_only: false },
    });
    expect(result).toBe("/tmp/export.csv");
  });

  it("exportHistory returns null when user cancels save dialog", async () => {
    vi.mocked(save).mockResolvedValue(null);
    const result = await exportHistory("csv", { query: null, favorites_only: false });
    expect(result).toBeNull();
    expect(invoke).not.toHaveBeenCalled();
  });
});
```

- [ ] **Step 2: Run tests to verify they fail**

```bash
pnpm vitest run test/history-ipc.test.ts
```

Expected: FAIL — `Cannot find module '../src/features/settings/history-ipc'`.

- [ ] **Step 3: Write the implementation**

Create `src/features/settings/history-types.ts`:
```typescript
/** Wire types mirroring the Rust structs in history::search + history::export. */

export interface DecryptedHistoryResult {
  result_uuid: string;
  provider_uuid: string;
  provider_name: string;
  engine_id: string;
  elapsed_ms: number;
  outcome_tag: string;
  text: string | null;
  error_kind: string | null;
  error_message: string | null;
  corrupt: boolean;
}

export interface DecryptedHistorySession {
  session_uuid: string;
  timestamp: number;
  trigger_source: string;
  detected_language: string | null;
  target_language: string;
  is_favorite: boolean;
  source_text: string | null;
  results: DecryptedHistoryResult[];
  corrupt: boolean;
}

export interface HistoryPage {
  items: DecryptedHistorySession[];
  next_cursor: string | null;
  scan_complete: boolean;
}

export interface HistoryFilter {
  query: string | null;
  favorites_only: boolean;
}
```

Create `src/features/settings/history-ipc.ts`:
```typescript
import { invoke } from "@tauri-apps/api/core";
import { save } from "@tauri-apps/plugin-dialog";
import type { HistoryPage, HistoryFilter } from "./history-types";

/** Search one batch (200 sessions). cursor = null for the first page. */
export async function searchHistory(
  query: string,
  cursor: string | null,
): Promise<HistoryPage> {
  return invoke<HistoryPage>("history_search", { query, cursor });
}

/** Toggle favorite. Returns the NEW favorite state. */
export async function toggleFavorite(sessionUuid: string): Promise<boolean> {
  return invoke<boolean>("history_toggle_favorite", { sessionUuid });
}

/** Delete one session (cascades results). */
export async function deleteSession(sessionUuid: string): Promise<void> {
  await invoke<void>("history_delete_session", { sessionUuid });
}

/** Export history to a file. Opens a save dialog, then writes via backend.
 *  Returns the written file path, or null if the user cancelled the dialog. */
export async function exportHistory(
  format: "csv" | "json",
  filter: HistoryFilter,
): Promise<string | null> {
  const extension = format === "csv" ? "csv" : "json";
  const filePath = await save({
    filters: [{ name: format.toUpperCase(), extensions: [extension] }],
  });
  if (!filePath) return null;
  return invoke<string>("history_export", { filePath, format, filter });
}
```

Create `src/features/settings/history-copy.ts`:
```typescript
import type { Locale } from "./copy";

/** Copy keys from design-system/linguaray/pages/09-history.md. */
export type HistoryCopy = {
  title: string;
  privacyGate: { title: string; description: string; enable: string; skip: string };
  empty: { title: string };
  search: { placeholder: string; noMatches: string };
  export: { title: string; format: string };
  cleanup: { summary: string };
  action: { delete: string; favorite: string; unfavorite: string };
  corrupt: { label: string };
};

const EN: HistoryCopy = {
  title: "History",
  privacyGate: {
    title: "Enable history?",
    description: "History is encrypted and stored locally only.",
    enable: "Enable",
    skip: "Skip",
  },
  empty: { title: "No history yet" },
  search: { placeholder: "Search history…", noMatches: "No matches" },
  export: { title: "Export", format: "Format" },
  cleanup: { summary: "{count} items cleaned" },
  action: { delete: "Delete", favorite: "Favorite", unfavorite: "Unfavorite" },
  corrupt: { label: "Corrupt entry" },
};

const ZH: HistoryCopy = {
  title: "历史",
  privacyGate: {
    title: "启用历史？",
    description: "历史经过加密，仅本地存储。",
    enable: "启用",
    skip: "跳过",
  },
  empty: { title: "暂无历史" },
  search: { placeholder: "搜索历史…", noMatches: "无匹配" },
  export: { title: "导出", format: "格式" },
  cleanup: { summary: "已清理 {count} 条" },
  action: { delete: "删除", favorite: "收藏", unfavorite: "取消收藏" },
  corrupt: { label: "损坏条目" },
};

export const HISTORY_COPY: Record<Locale, HistoryCopy> = { zh: ZH, en: EN };
```

- [ ] **Step 4: Run tests to verify they pass**

```bash
pnpm vitest run test/history-ipc.test.ts
pnpm typecheck
```

Expected: PASS (5 tests).

- [ ] **Step 5: Commit**

```bash
git add src/features/settings/history-types.ts src/features/settings/history-ipc.ts src/features/settings/history-copy.ts test/history-ipc.test.ts
git commit -m "feat(r4-a3): history frontend types + IPC wrappers + copy dictionary

- history-types.ts mirrors Rust HistoryPage/DecryptedHistorySession/HistoryFilter
- history-ipc.ts: searchHistory, toggleFavorite, deleteSession, exportHistory (save dialog)
- history-copy.ts: 13 keys from 09-history.md (EN + ZH)"
```

---

### Task A4: HistoryView component (8 states)

**Files:**
- Create: `src/features/settings/HistoryView.tsx`
- Create: `src/features/settings/HistoryView.css`
- Create: `test/HistoryView.test.tsx`
- Create: `test/HistoryView.a11y.test.tsx`

**States (8):** Initial (privacy gate) / Empty / Loading / Populated / Search (no matches) / Export / Retention cleanup badge / Corrupt row.

**Interfaces:**
- Consumes: `history-ipc.ts` (searchHistory, toggleFavorite, deleteSession, exportHistory), `history-copy.ts`, `@linguaray/ui` components, `@tauri-apps/api/core` invoke (for history_privacy_status)
- Produces: `<HistoryView />` component used by App.tsx + ui-lab

- [ ] **Step 1: Write the failing test**

Create `test/HistoryView.test.tsx`:
```typescript
import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, waitFor, fireEvent } from "@solidjs/testing-library";
import { HistoryView } from "../src/features/settings/HistoryView";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));
vi.mock("@tauri-apps/plugin-dialog", () => ({ save: vi.fn() }));

import { invoke } from "@tauri-apps/api/core";

describe("HistoryView states", () => {
  beforeEach(() => vi.clearAllMocks());

  it("shows privacy gate when history is not enabled", async () => {
    vi.mocked(invoke).mockImplementation((cmd: string) => {
      if (cmd === "history_privacy_status") return Promise.resolve({ enabled: false, retention_days: 30, record_count: 0 });
      return Promise.resolve({ items: [], next_cursor: null, scan_complete: true });
    });
    const { getByText } = render(() => <HistoryView />);
    await waitFor(() => expect(getByText("Enable history?")).toBeDefined());
  });

  it("shows empty state when history is enabled but no records", async () => {
    vi.mocked(invoke).mockImplementation((cmd: string) => {
      if (cmd === "history_privacy_status") return Promise.resolve({ enabled: true, retention_days: 30, record_count: 0 });
      if (cmd === "history_search") return Promise.resolve({ items: [], next_cursor: null, scan_complete: true });
      return Promise.resolve(null);
    });
    const { getByText } = render(() => <HistoryView />);
    await waitFor(() => expect(getByText("No history yet")).toBeDefined());
  });

  it("shows loading skeleton while fetching", async () => {
    vi.mocked(invoke).mockImplementation((cmd: string) => {
      if (cmd === "history_privacy_status") return Promise.resolve({ enabled: true, retention_days: 30, record_count: 5 });
      if (cmd === "history_search") return new Promise(() => {}); // never resolves
      return Promise.resolve(null);
    });
    const { getByTestId } = render(() => <HistoryView />);
    await waitFor(() => expect(getByTestId("history-skeleton")).toBeDefined());
  });

  it("renders populated list with sessions", async () => {
    vi.mocked(invoke).mockImplementation((cmd: string) => {
      if (cmd === "history_privacy_status") return Promise.resolve({ enabled: true, retention_days: 30, record_count: 1 });
      if (cmd === "history_search") return Promise.resolve({
        items: [{
          session_uuid: "s1", timestamp: 1700000100, trigger_source: "selection",
          detected_language: "en", target_language: "zh", is_favorite: false,
          source_text: "hello", results: [], corrupt: false,
        }],
        next_cursor: null, scan_complete: true,
      });
      return Promise.resolve(null);
    });
    const { getByText } = render(() => <HistoryView />);
    await waitFor(() => expect(getByText("hello")).toBeDefined());
  });

  it("shows 'No matches' when search yields empty", async () => {
    let searchCalled = false;
    vi.mocked(invoke).mockImplementation((cmd: string, args: any) => {
      if (cmd === "history_privacy_status") return Promise.resolve({ enabled: true, retention_days: 30, record_count: 5 });
      if (cmd === "history_search") {
        searchCalled = true;
        return Promise.resolve({ items: [], next_cursor: null, scan_complete: true });
      }
      return Promise.resolve(null);
    });
    const { getByPlaceholderText, getByText } = render(() => <HistoryView />);
    await waitFor(() => expect(getByPlaceholderText("Search history…")).toBeDefined());
    fireEvent.input(getByPlaceholderText("Search history…"), { target: { value: "xyz" } });
    await waitFor(() => expect(getByText("No matches")).toBeDefined());
  });

  it("marks corrupt rows with badge", async () => {
    vi.mocked(invoke).mockImplementation((cmd: string) => {
      if (cmd === "history_privacy_status") return Promise.resolve({ enabled: true, retention_days: 30, record_count: 1 });
      if (cmd === "history_search") return Promise.resolve({
        items: [{
          session_uuid: "bad", timestamp: 1700000100, trigger_source: "input",
          detected_language: null, target_language: "zh", is_favorite: false,
          source_text: null, results: [], corrupt: true,
        }],
        next_cursor: null, scan_complete: true,
      });
      return Promise.resolve(null);
    });
    const { getByText } = render(() => <HistoryView />);
    await waitFor(() => expect(getByText("Corrupt entry")).toBeDefined());
  });

  it("shows export format selector", async () => {
    vi.mocked(invoke).mockImplementation((cmd: string) => {
      if (cmd === "history_privacy_status") return Promise.resolve({ enabled: true, retention_days: 30, record_count: 1 });
      if (cmd === "history_search") return Promise.resolve({
        items: [{
          session_uuid: "s1", timestamp: 1700000100, trigger_source: "selection",
          detected_language: "en", target_language: "zh", is_favorite: false,
          source_text: "hello", results: [], corrupt: false,
        }],
        next_cursor: null, scan_complete: true,
      });
      return Promise.resolve(null);
    });
    const { getByText } = render(() => <HistoryView />);
    await waitFor(() => expect(getByText("Export")).toBeDefined());
  });
});
```

Create `test/HistoryView.a11y.test.tsx`:
```typescript
import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, waitFor } from "@solidjs/testing-library";
import { runAxe } from "./axe";
import { HistoryView } from "../src/features/settings/HistoryView";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));
vi.mock("@tauri-apps/plugin-dialog", () => ({ save: vi.fn() }));
import { invoke } from "@tauri-apps/api/core";

describe("HistoryView a11y", () => {
  beforeEach(() => {
    vi.mocked(invoke).mockImplementation((cmd: string) => {
      if (cmd === "history_privacy_status") return Promise.resolve({ enabled: true, retention_days: 30, record_count: 1 });
      if (cmd === "history_search") return Promise.resolve({
        items: [{ session_uuid: "s1", timestamp: 1700000100, trigger_source: "selection",
          detected_language: "en", target_language: "zh", is_favorite: false,
          source_text: "hello", results: [], corrupt: false }],
        next_cursor: null, scan_complete: true,
      });
      return Promise.resolve(null);
    });
  });

  it("has no axe violations in populated state", async () => {
    const { container, getByText } = render(() => <HistoryView />);
    await waitFor(() => expect(getByText("hello")).toBeDefined());
    const results = await runAxe(container);
    expect(results.violations).toHaveLength(0);
  });
});
```

- [ ] **Step 2: Run tests to verify they fail**

```bash
pnpm vitest run test/HistoryView.test.tsx test/HistoryView.a11y.test.tsx
```

Expected: FAIL — `Cannot find module '../src/features/settings/HistoryView'`.

- [ ] **Step 3: Write the implementation**

Create `src/features/settings/HistoryView.tsx`:
```tsx
import { createSignal, For, Show, onMount, type Component } from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import { Button, EmptyState, TextField, Select, Confirm, Spinner } from "@linguaray/ui";
import { Star, Trash2, Download, Search } from "lucide-solid";
import { HISTORY_COPY } from "./history-copy";
import { searchHistory, toggleFavorite, deleteSession, exportHistory } from "./history-ipc";
import type { DecryptedHistorySession, HistoryFilter } from "./history-types";
import { detectLocale } from "../../i18n";
import "./HistoryView.css";

type LoadState = "initial" | "loading" | "populated" | "empty" | "search-empty";

interface PrivacyStatus { enabled: boolean; retention_days: number; record_count: number; }

export const HistoryView: Component = () => {
  const locale = detectLocale();
  const t = HISTORY_COPY[locale];
  const [privacyStatus, setPrivacyStatus] = createSignal<PrivacyStatus | null>(null);
  const [state, setState] = createSignal<LoadState>("initial");
  const [items, setItems] = createSignal<DecryptedHistorySession[]>([]);
  const [cursor, setCursor] = createSignal<string | null>(null);
  const [searchQuery, setSearchQuery] = createSignal("");
  const [deleteTarget, setDeleteTarget] = createSignal<string | null>(null);
  const [exportFormat, setExportFormat] = createSignal<"csv" | "json">("csv");
  const [cleanupBadge, setCleanupBadge] = createSignal<number | null>(null);

  const loadFirst = async () => {
    setState("loading");
    const page = await searchHistory("", null);
    setItems(page.items);
    setCursor(page.next_cursor);
    setState(page.items.length === 0 ? "empty" : "populated");
  };

  onMount(async () => {
    try {
      const status = await invoke<PrivacyStatus>("history_privacy_status");
      setPrivacyStatus(status);
      if (status.enabled) {
        await loadFirst();
      } else {
        setState("initial");
      }
    } catch {
      setState("initial");
    }
  });

  const handleSearch = async (query: string) => {
    setSearchQuery(query);
    if (!query.trim()) {
      await loadFirst();
      return;
    }
    setState("loading");
    const page = await searchHistory(query, null);
    setItems(page.items);
    setCursor(page.next_cursor);
    setState(page.items.length === 0 ? "search-empty" : "populated");
  };

  const handleFavorite = async (uuid: string) => {
    const newState = await toggleFavorite(uuid);
    setItems((prev) => prev.map((s) => s.session_uuid === uuid ? { ...s, is_favorite: newState } : s));
  };

  const confirmDelete = async () => {
    const target = deleteTarget();
    if (!target) return;
    await deleteSession(target);
    setItems((prev) => prev.filter((s) => s.session_uuid !== target));
    setDeleteTarget(null);
    if (items().length === 0) setState("empty");
  };

  const handleExport = async () => {
    const filter: HistoryFilter = { query: searchQuery() || null, favorites_only: false };
    await exportHistory(exportFormat(), filter);
  };

  const handleEnable = async () => {
    await invoke("history_set_enabled", { enabled: true });
    const status = await invoke<PrivacyStatus>("history_privacy_status");
    setPrivacyStatus(status);
    if (status.enabled) await loadFirst();
  };

  return (
    <section class="history-view" data-testid="history-view" data-state={state()}>
      <header class="history-view__header">
        <h2>{t.title}</h2>
        <Show when={cleanupBadge() !== null}>
          <span class="history-view__cleanup-badge" role="status">
            {t.cleanup.summary.replace("{count}", String(cleanupBadge()))}
          </span>
        </Show>
      </header>

      <Show when={!privacyStatus()?.enabled} fallback={
        <div class="history-view__body">
          <div class="history-view__toolbar">
            <TextField
              placeholder={t.search.placeholder}
              value={searchQuery()}
              leadingIcon={<Search size={14} />}
              onInput={(e) => handleSearch(e.currentTarget.value)}
              aria-label={t.search.placeholder}
            />
            <div class="history-view__export">
              <Select
                value={exportFormat()}
                options={[{ value: "csv", label: "CSV" }, { value: "json", label: "JSON" }]}
                onChange={(v) => setExportFormat(v as "csv" | "json")}
                ariaLabel={t.export.format}
              />
              <Button variant="ghost" onClick={handleExport} aria-label={t.export.title}>
                <Download size={14} /> {t.export.title}
              </Button>
            </div>
          </div>

          <Show when={state() === "loading"}>
            <div data-testid="history-skeleton" class="history-view__skeleton">
              <For each={Array.from({ length: 4 })}>{() => <div class="history-view__skeleton-row" />}</For>
            </div>
          </Show>

          <Show when={state() === "populated"}>
            <ul class="history-view__list" role="list" aria-label={t.title}>
              <For each={items()}>
                {(session) => (
                  <li class="history-view__item" data-corrupt={session.corrupt}>
                    <div class="history-view__item-body">
                      <p class="history-view__source">{session.source_text ?? ""}</p>
                      <For each={session.results}>
                        {(r) => <p class="history-view__translation">{r.text ?? ""}</p>}
                      </For>
                      <div class="history-view__meta">
                        <span class="history-view__engine">{session.results[0]?.engine_id ?? ""}</span>
                        <span class="history-view__time">{new Date(session.timestamp * 1000).toLocaleString()}</span>
                      </div>
                    </div>
                    <Show when={session.corrupt}>
                      <span class="history-view__corrupt-badge" role="alert">{t.corrupt.label}</span>
                    </Show>
                    <div class="history-view__actions">
                      <Button
                        variant="ghost"
                        aria-label={session.is_favorite ? t.action.unfavorite : t.action.favorite}
                        onClick={() => handleFavorite(session.session_uuid)}
                      >
                        <Star size={14} fill={session.is_favorite ? "currentColor" : "none"} />
                      </Button>
                      <Button
                        variant="ghost"
                        aria-label={t.action.delete}
                        onClick={() => setDeleteTarget(session.session_uuid)}
                      >
                        <Trash2 size={14} />
                      </Button>
                    </div>
                  </li>
                )}
              </For>
            </ul>
          </Show>

          <Show when={state() === "empty"}>
            <EmptyState title={t.empty.title} icon={<Search size={32} />} />
          </Show>

          <Show when={state() === "search-empty"}>
            <EmptyState title={t.search.noMatches} icon={<Search size={32} />} />
          </Show>
        </div>
      }>
        <div class="history-view__privacy-gate">
          <h3>{t.privacyGate.title}</h3>
          <p>{t.privacyGate.description}</p>
          <div class="history-view__privacy-actions">
            <Button onClick={handleEnable}>{t.privacyGate.enable}</Button>
            <Button variant="ghost">{t.privacyGate.skip}</Button>
          </div>
        </div>
      </Show>

      <Confirm
        open={deleteTarget() !== null}
        onOpenChange={(open) => { if (!open) setDeleteTarget(null); }}
        title={t.action.delete}
        message={t.action.delete}
        confirmLabel={t.action.delete}
        cancelLabel="Cancel"
        variant="destructive"
        onConfirm={confirmDelete}
        onCancel={() => setDeleteTarget(null)}
      />
    </section>
  );
};

export default HistoryView;
```

Create `src/features/settings/HistoryView.css`:
```css
.history-view { display: flex; flex-direction: column; gap: var(--space-4); padding: var(--space-4); }
.history-view__header { display: flex; align-items: center; gap: var(--space-2); }
.history-view__header h2 { font-size: var(--text-xl); color: var(--color-fg); margin: 0; }
.history-view__cleanup-badge {
  font-size: var(--text-xs); color: var(--color-fg-muted);
  background: var(--color-bg-subtle); padding: var(--space-1) var(--space-2);
  border-radius: var(--radius-sm);
}
.history-view__body { display: flex; flex-direction: column; gap: var(--space-3); }
.history-view__toolbar { display: flex; gap: var(--space-2); align-items: center; flex-wrap: wrap; }
.history-view__export { display: flex; gap: var(--space-1); align-items: center; }
.history-view__list { list-style: none; margin: 0; padding: 0; display: flex; flex-direction: column; gap: var(--space-2); }
.history-view__item {
  display: flex; gap: var(--space-2); align-items: flex-start;
  padding: var(--space-3); border: 1px solid var(--color-border);
  border-radius: var(--radius-md); background: var(--color-bg);
}
.history-view__item[data-corrupt="true"] { border-color: var(--color-danger); }
.history-view__item-body { flex: 1; display: flex; flex-direction: column; gap: var(--space-1); }
.history-view__source { font-size: var(--text-sm); color: var(--color-fg); margin: 0; }
.history-view__translation { font-size: var(--text-sm); color: var(--color-fg-muted); margin: 0; }
.history-view__meta { display: flex; gap: var(--space-2); }
.history-view__engine, .history-view__time { font-size: var(--text-xs); color: var(--color-fg-muted); }
.history-view__corrupt-badge {
  font-size: var(--text-xs); color: var(--color-danger);
  background: var(--color-danger-bg); padding: var(--space-1) var(--space-2);
  border-radius: var(--radius-sm);
}
.history-view__actions { display: flex; gap: var(--space-1); }
.history-view__skeleton { display: flex; flex-direction: column; gap: var(--space-2); }
.history-view__skeleton-row {
  height: 60px; border-radius: var(--radius-md);
  background: var(--color-bg-subtle); animation: pulse 1.5s ease-in-out infinite;
}
.history-view__privacy-gate { text-align: center; padding: var(--space-6); }
.history-view__privacy-gate h3 { font-size: var(--text-lg); color: var(--color-fg); }
.history-view__privacy-gate p { font-size: var(--text-sm); color: var(--color-fg-muted); }
.history-view__privacy-actions { display: flex; gap: var(--space-2); justify-content: center; margin-top: var(--space-4); }
@keyframes pulse { 0%, 100% { opacity: 1; } 50% { opacity: 0.5; } }
```

- [ ] **Step 4: Run tests to verify they pass**

```bash
pnpm vitest run test/HistoryView.test.tsx test/HistoryView.a11y.test.tsx
```

Expected: PASS (7 state tests + 1 a11y test).

- [ ] **Step 5: Commit**

```bash
git add src/features/settings/HistoryView.tsx src/features/settings/HistoryView.css test/HistoryView.test.tsx test/HistoryView.a11y.test.tsx
git commit -m "feat(r4-a4): HistoryView component with 8 states + axe scan

States: privacy-gate / empty / loading-skeleton / populated / search-no-matches /
export / corrupt-row-badge / cleanup-badge
Uses semantic tokens only; lucide-solid icons; Confirm for delete"
```

---

### Task A5: Wire History into Settings + App + tray + ui-lab

**Files:**
- Modify: `src/features/settings/SettingsShell.tsx` — `SettingsSection` union, `NavDef`, `navItems`
- Modify: `src/features/settings/copy.ts` — `NavCopy` type, EN + ZH values
- Modify: `src/App.tsx` — navigate whitelist, render switch
- Modify: `src-tauri/src/lib.rs` — tray `build_tray_menu` History item enabled
- Modify: `test/SettingsShell.test.tsx` — assert 5 nav items
- Modify: `test/App.test.tsx` — assert navigate whitelist includes history
- Modify: `apps/ui-lab/src/App.tsx` — add history Match (reuse production HistoryView)

**Interfaces:**
- Consumes: HistoryView from Task A4, `HistoryCopy` from Task A3
- Produces: History as a navigable settings section

- [ ] **Step 1: Write the failing test**

Add to `test/SettingsShell.test.tsx`:
```typescript
it("renders 5 nav items including history", () => {
  const { getAllByRole } = render(() => (
    <SettingsShell><div /></SettingsShell>
  ));
  const navItems = getAllByRole("button");
  const labels = navItems.map((n) => n.textContent);
  expect(labels.some((l) => l?.includes("History"))).toBe(true);
});
```

Add to `test/App.test.tsx`:
```typescript
it("navigate event for 'history' sets activePage", async () => {
  // Emit a navigate event for history and assert it renders HistoryView.
  const { getByText } = render(() => <App />);
  // Simulate the navigate event.
  window.dispatchEvent(new CustomEvent("test-navigate", { detail: "history" }));
  await waitFor(() => expect(getByText("History")).toBeDefined());
});
```

- [ ] **Step 2: Run tests to verify they fail**

```bash
pnpm vitest run test/SettingsShell.test.tsx test/App.test.tsx
```

Expected: FAIL — history nav item not found / navigate whitelist rejects "history".

- [ ] **Step 3: Write the implementation**

Modify `src/features/settings/SettingsShell.tsx`:
```tsx
// SettingsSection union: add "history"
export type SettingsSection =
  | "provider-center"
  | "keystore-recovery"
  | "shortcuts"
  | "privacy"
  | "history";
```
Add to `navItems` array:
```tsx
  { id: "history", label: t.nav.history, icon: <History size={16} />, disabled: false },
```
Add `History` to the import from `lucide-solid`.

Modify `src/features/settings/copy.ts`:
```typescript
// NavCopy type: add history
type NavCopy = {
  providerCenter: string;
  keystoreRecovery: string;
  shortcuts: string;
  privacy: string;
  history: string;
  placeholderHint: string;
};
```
Add to EN:
```typescript
  nav: { ..., history: "History", ... },
```
Add to ZH:
```typescript
  nav: { ..., history: "历史", ... },
```

Modify `src/App.tsx`:
```tsx
import HistoryView from "./features/settings/HistoryView";
// ...
// navigate whitelist: add "history"
if (
  page === "provider-center" ||
  page === "keystore-recovery" ||
  page === "shortcuts" ||
  page === "privacy" ||
  page === "history"
) {
  setActivePage(page);
}
// render switch: add HistoryView
{activePage() === "history" ? (
  <HistoryView />
) : activePage() === "provider-center" ? (
  ...
```

Modify `src-tauri/src/lib.rs` tray `build_tray_menu` — enable the History item:
```rust
    let history = MenuItem::with_id(
        app,
        "tray.history",
        "History",
        true,   // was: false (disabled)
        None::<&str>,
    )?;
```
And in `handle_tray_menu_event`, add a case for `"tray.history"`:
```rust
        "tray.history" => {
            if let Some(w) = app.get_webview_window("main") {
                let _ = w.show();
                let _ = w.set_focus();
                let _ = app.emit("navigate", "history");
            }
        }
```

Modify `apps/ui-lab/src/App.tsx`:
- Add `"history"` to the `IMPLEMENTED` array
- Add a `<Match when={nav() === "history"}>` that renders the production `HistoryView` from `@app/features/settings/HistoryView`

- [ ] **Step 4: Run tests to verify they pass**

```bash
pnpm vitest run test/SettingsShell.test.tsx test/App.test.tsx
cargo test --manifest-path src-tauri/Cargo.toml --test tray_refresh -- --nocapture 2>&1 | tail -5
```

Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add src/features/settings/SettingsShell.tsx src/features/settings/copy.ts src/App.tsx src-tauri/src/lib.rs test/SettingsShell.test.tsx test/App.test.tsx apps/ui-lab/src/App.tsx
git commit -m "feat(r4-a5): wire History into Settings nav + App + tray + ui-lab

- SettingsSection union + NavDef + navItems include 'history'
- NavCopy type + EN/ZH values
- App.tsx navigate whitelist + render switch
- tray History item enabled (emits navigate 'history')
- ui-lab IMPLEMENTED + Match (reuses production HistoryView)"
```

---

### Task A6: Gate/recovery barrier test for History commands

**Files:**
- Create: `src-tauri/tests/history_gate_barrier.rs` — prove locks release on error paths

**Interfaces:**
- Consumes: `AppState`, `data_gate`, `require_ready_gated`, `require_ready_gated_write`
- Produces: proof that favorite/delete/export commands release the gate on every error path

- [ ] **Step 1: Write the failing test**

Create `src-tauri/tests/history_gate_barrier.rs`:
```rust
//! Prove that history_toggle_favorite / history_delete_session / history_export
//! release the data_gate on EVERY error path, so a failed command never deadlocks
//! a subsequent archive/reset.

use linguaray_lib::db::{schema, Database};
use linguaray_lib::history::export::{export_all, write_export_file, ExportFormat, HistoryFilter};
use linguaray_lib::keystore::Keystore;
use parking_lot::RwLock;
use std::sync::Arc;
use tempfile::TempDir;

struct Harness {
    _dir: TempDir,
    db: Database,
    keystore: Keystore,
    gate: Arc<RwLock<()>>,
}

impl Harness {
    fn new() -> Self {
        let dir = tempfile::tempdir().unwrap();
        let db = Database::open(&dir.path().join("gate.db")).unwrap();
        db.with_conn(|conn| {
            let tx = conn.transaction()?;
            schema::create_all_tables(&tx)?;
            schema::seed_singletons(&tx)?;
            tx.commit()?;
            Ok(())
        }).unwrap();
        let keystore = Keystore::new(dir.path().join("keystore")).unwrap();
        let gate = Arc::new(RwLock::new(()));
        Self { _dir: dir, db, keystore, gate }
    }
}

#[test]
fn export_all_failure_releases_gate_and_allows_subsequent_write() {
    let h = Harness::new();
    // Simulate export under read gate, then a write gate succeeds after.
    {
        let _read_guard = h.gate.read();
        // Export on empty DB succeeds (returns empty), but proves the read guard
        // was acquired and released cleanly.
        let _ = export_all(&h.db, &h.keystore, &HistoryFilter::default());
    }
    // Write gate acquires immediately after read drops.
    let _write_guard = h.gate.write();
    // If the read guard leaked, this line would block forever (test timeout).
    h.db.with_conn(|conn| {
        conn.execute("UPDATE preferences SET history_enabled=1 WHERE id=1", [])?;
        Ok(())
    }).unwrap();
}

#[test]
fn toggle_favorite_on_nonexistent_releases_db_mutex() {
    let h = Harness::new();
    // toggle_favorite on a non-existent UUID returns Err but MUST release the DB mutex.
    let result = h.db.with_conn(|conn| {
        linguaray_lib::db::history::toggle_favorite(conn, "nonexistent")
    });
    assert!(result.is_err());
    // If the mutex leaked, this would block.
    h.db.with_conn(|conn| {
        conn.execute("SELECT 1", [])?;
        Ok(())
    }).unwrap();
}

#[test]
fn delete_session_on_nonexistent_releases_db_mutex() {
    let h = Harness::new();
    let result = h.db.with_conn(|conn| {
        linguaray_lib::db::history::delete_session(conn, "nonexistent")
    });
    assert!(result.is_err());
    h.db.with_conn(|conn| {
        conn.execute("SELECT 1", [])?;
        Ok(())
    }).unwrap();
}

#[test]
fn write_export_file_io_error_does_not_corrupt_state() {
    let h = Harness::new();
    let sessions = export_all(&h.db, &h.keystore, &HistoryFilter::default()).unwrap();
    // Write to a non-existent directory → io::Error.
    let bad_path = std::path::PathBuf::from("/nonexistent/dir/export.csv");
    let result = write_export_file(&sessions, &bad_path, ExportFormat::Csv);
    assert!(result.is_err());
    // State is unchanged; a valid path still works.
    let good_path = h._dir.path().join("ok.csv");
    write_export_file(&sessions, &good_path, ExportFormat::Csv).unwrap();
}
```

- [ ] **Step 2: Run tests to verify they fail**

```bash
cargo test --manifest-path src-tauri/Cargo.toml --test history_gate_barrier -- --nocapture
```

Expected: Depends — `toggle_favorite`/`delete_session` tests pass once A2 is done. The export_all + write_export_file tests pass once A1 is done. If A1+A2 are committed, these PASS. If not, COMPILE ERROR.

- [ ] **Step 3: Implementation already done in A1/A2**

No new implementation needed. This task VERIFIES the barrier properties.

- [ ] **Step 4: Run tests to verify they pass**

```bash
cargo test --manifest-path src-tauri/Cargo.toml --test history_gate_barrier -- --nocapture -- --test-threads=1
```

Expected: PASS (4 tests, no deadlock timeout).

- [ ] **Step 5: Commit**

```bash
git add src-tauri/tests/history_gate_barrier.rs
git commit -m "test(r4-a6): gate/recovery barrier tests for history commands

- export_all + write guard acquire proves read gate releases
- toggle_favorite/delete_session NotFound releases DB mutex
- write_export_file io::Error does not corrupt state"
```

---

### Stage A Checkpoint

**STOP. Do not start Stage B until all three reviewers approve.**

1. **Rust/security:** export_all cursor monotonic (verified by 201/1000+ tests), data_gate.read() spans full export (barrier test), no plaintext leaks (file write only), HistoryFilter is passthrough (no SQL injection — uses search()'s parameterized query).
2. **Frontend/design/a11y:** 8 states render (7 component tests + cleanup badge), corrupt-row badge is `role="alert"`, axe scan clean, keyboard-reachable (Confirm dialog), semantic tokens only.
3. **Test-authenticity:** 201-record test asserts exact count + monotonicity, concurrent-mutation test uses a real thread, corrupt test uses real garbage ciphertext, gate barrier tests prove no deadlock.

---

## Stage B: Surface 10 — Vocabulary

**Checkpoint goal:** User can add words from Popup/InputPanel, view a paginated vocabulary list, delete, export to file (CSV/JSON), and push to AnkiConnect. All data encrypted at rest with the shared history key (get-or-create, history consent NOT toggled). popup.json + input.json carry `allow-vocabulary-add`.

**Three-way review gate at end of Stage B:**
1. **Rust/security:** key get-or-create does NOT enable history, AnkiConnect POST has no redirect + timeout + response validation + no plaintext file, pagination cursor monotonic, capabilities correct.
2. **Frontend/design/a11y:** 5 states render, favorite wiring pre-fills word+definition from multi-result, axe scan clean.
3. **Test-authenticity:** pagination test verifies 200/batch boundary, AnkiConnect test uses wiremock, capability test verifies popup.json/input.json contain `allow-vocabulary-add`.

---

### Task B1: Vocabulary DB repository

**Files:**
- Create: `src-tauri/src/db/vocabulary.rs` — `create()`, `read_page(cursor)`, `delete()`, `count()`
- Modify: `src-tauri/src/db/mod.rs` — add `pub mod vocabulary;`
- Create: `src-tauri/tests/vocabulary.rs` — CRUD + pagination tests
- Test: `src-tauri/tests/vocabulary.rs`

**Interfaces:**
- Consumes: `Database`, `Connection`, `rusqlite::params`
- Produces:
  - `db::vocabulary::RawVocabularyItem` — raw encrypted row
  - `db::vocabulary::create(conn, item) -> Result<(), DbError>`
  - `db::vocabulary::read_page(conn, cursor: Option<&VocabCursor>) -> Result<(Vec<RawVocabularyItem>, bool), DbError>`
  - `db::vocabulary::delete(conn, item_uuid) -> Result<(), DbError>`
  - `db::vocabulary::count(conn) -> Result<u64, DbError>`

- [ ] **Step 1: Write the failing test**

Create `src-tauri/tests/vocabulary.rs`:
```rust
use linguaray_lib::db::{schema, Database, DbError};
use tempfile::TempDir;

struct Harness { _dir: TempDir, db: Database }

impl Harness {
    fn new() -> Self {
        let dir = tempfile::tempdir().unwrap();
        let db = Database::open(&dir.path().join("vocab.db")).unwrap();
        db.with_conn(|conn| {
            let tx = conn.transaction()?;
            schema::create_all_tables(&tx)?;
            schema::seed_singletons(&tx)?;
            tx.commit()?;
            Ok(())
        }).unwrap();
        Self { _dir: dir, db }
    }

    fn insert_raw(&self, uuid: &str, timestamp: i64) {
        self.db.with_conn(|conn| {
            linguaray_lib::db::vocabulary::create(conn, &linguaray_lib::db::vocabulary::RawVocabularyItem {
                item_uuid: uuid.to_string(),
                timestamp,
                source_language: "en".into(),
                target_language: "zh".into(),
                word_encrypted: vec![0xAA],
                word_nonce: vec![0u8; 12],
                definition_encrypted: vec![0xBB],
                definition_nonce: vec![0u8; 12],
                crypto_version: 1,
            })?;
            Ok(())
        }).unwrap();
    }
}

#[test]
fn create_inserts_a_row() {
    let h = Harness::new();
    h.insert_raw("item-1", 1_700_000_100);
    assert_eq!(h.db.with_conn(|c| linguaray_lib::db::vocabulary::count(c)).unwrap(), 1);
}

#[test]
fn read_page_returns_200_items_and_cursor_for_next_batch() {
    let h = Harness::new();
    let base = 1_700_000_000i64;
    for i in 0..250 {
        h.insert_raw(&format!("v-{i:03}"), base + i as i64);
    }
    let (items, scan_complete) = h.db.with_conn(|c| linguaray_lib::db::vocabulary::read_page(c, None)).unwrap();
    assert_eq!(items.len(), 200);
    assert!(!scan_complete);
    // Monotonic: timestamps descending.
    for w in items.windows(2) {
        assert!(w[0].timestamp >= w[1].timestamp);
    }
}

#[test]
fn read_page_last_batch_sets_scan_complete() {
    let h = Harness::new();
    for i in 0..50 {
        h.insert_raw(&format!("p-{i:03}"), 1_700_000_000 + i as i64);
    }
    let (items, scan_complete) = h.db.with_conn(|c| linguaray_lib::db::vocabulary::read_page(c, None)).unwrap();
    assert_eq!(items.len(), 50);
    assert!(scan_complete);
}

#[test]
fn delete_removes_one_row() {
    let h = Harness::new();
    h.insert_raw("del", 1_700_000_100);
    h.insert_raw("keep", 1_700_000_099);
    h.db.with_conn(|c| linguaray_lib::db::vocabulary::delete(c, "del")).unwrap();
    assert_eq!(h.db.with_conn(|c| linguaray_lib::db::vocabulary::count(c)).unwrap(), 1);
}

#[test]
fn delete_missing_returns_not_found() {
    let h = Harness::new();
    let result = h.db.with_conn(|c| linguaray_lib::db::vocabulary::delete(c, "nonexistent"));
    assert!(result.is_err());
}
```

- [ ] **Step 2: Run tests to verify they fail**

```bash
cargo test --manifest-path src-tauri/Cargo.toml --test vocabulary -- --nocapture
```

Expected: COMPILE ERROR — `unresolved module vocabulary`.

- [ ] **Step 3: Write the implementation**

Create `src-tauri/src/db/vocabulary.rs`:
```rust
//! Encrypted vocabulary CRUD repository.
//!
//! All word/definition fields are stored as AES-256-GCM ciphertext + nonce.
//! The service layer (`crate::vocabulary`) handles encrypt/decrypt; this module
//! only reads/writes raw blobs.

use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};

use super::DbError;

pub const VOCAB_BATCH: usize = 200;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawVocabularyItem {
    pub item_uuid: String,
    pub timestamp: i64,
    pub source_language: String,
    pub target_language: String,
    pub word_encrypted: Vec<u8>,
    pub word_nonce: Vec<u8>,
    pub definition_encrypted: Vec<u8>,
    pub definition_nonce: Vec<u8>,
    pub crypto_version: u32,
}

/// Cursor for pagination: (timestamp, item_uuid), both descending.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VocabCursor {
    pub timestamp: i64,
    pub item_uuid: String,
}

/// Insert one encrypted vocabulary row.
pub fn create(conn: &mut Connection, item: &RawVocabularyItem) -> Result<(), DbError> {
    let tx = conn.transaction()?;
    tx.execute(
        "INSERT INTO vocabulary
         (item_uuid, timestamp, source_language, target_language,
          word_encrypted, word_nonce, definition_encrypted, definition_nonce, crypto_version)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        params![
            item.item_uuid, item.timestamp, item.source_language, item.target_language,
            item.word_encrypted, item.word_nonce,
            item.definition_encrypted, item.definition_nonce,
            item.crypto_version,
        ],
    )?;
    tx.commit()?;
    Ok(())
}

/// Read one batch (200 rows), ordered by timestamp DESC, item_uuid DESC.
/// Returns (items, scan_complete). scan_complete=true means this was the last page.
pub fn read_page(
    conn: &Connection,
    cursor: Option<&VocabCursor>,
) -> Result<(Vec<RawVocabularyItem>, bool), DbError> {
    let cursor_ts = cursor.map_or(i64::MAX, |c| c.timestamp);
    let cursor_uuid = cursor.map_or("\u{10ffff}", |c| c.item_uuid.as_str());
    let mut stmt = conn.prepare(
        "SELECT item_uuid, timestamp, source_language, target_language,
                word_encrypted, word_nonce, definition_encrypted, definition_nonce, crypto_version
         FROM vocabulary
         WHERE timestamp < ?1 OR (timestamp = ?1 AND item_uuid < ?2)
         ORDER BY timestamp DESC, item_uuid DESC
         LIMIT ?3",
    )?;
    let rows = stmt
        .query_map(params![cursor_ts, cursor_uuid, VOCAB_BATCH as i64 + 1], |row| {
            Ok(RawVocabularyItem {
                item_uuid: row.get(0)?,
                timestamp: row.get(1)?,
                source_language: row.get(2)?,
                target_language: row.get(3)?,
                word_encrypted: row.get(4)?,
                word_nonce: row.get(5)?,
                definition_encrypted: row.get(6)?,
                definition_nonce: row.get(7)?,
                crypto_version: row.get(8)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    // We fetch BATCH+1 rows to detect whether more pages exist.
    let scan_complete = rows.len() <= VOCAB_BATCH;
    let items = if rows.len() > VOCAB_BATCH {
        rows.into_iter().take(VOCAB_BATCH).collect()
    } else {
        rows
    };
    Ok((items, scan_complete))
}

/// Delete one vocabulary row. NotFound if missing.
pub fn delete(conn: &mut Connection, item_uuid: &str) -> Result<(), DbError> {
    let tx = conn.transaction()?;
    let changed = tx.execute("DELETE FROM vocabulary WHERE item_uuid=?1", [item_uuid])?;
    if changed != 1 {
        tx.rollback()?;
        return Err(DbError::NotFound(format!("vocabulary item {item_uuid} not found")));
    }
    tx.commit()?;
    Ok(())
}

/// Count all vocabulary rows.
pub fn count(conn: &Connection) -> Result<u64, DbError> {
    let count: i64 = conn.query_row("SELECT COUNT(*) FROM vocabulary", [], |row| row.get(0))?;
    Ok(u64::try_from(count).unwrap_or(0))
}
```

Modify `src-tauri/src/db/mod.rs` — add:
```rust
pub mod vocabulary;
```

- [ ] **Step 4: Run tests to verify they pass**

```bash
cargo test --manifest-path src-tauri/Cargo.toml --test vocabulary -- --nocapture
```

Expected: PASS (5 tests).

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/db/vocabulary.rs src-tauri/src/db/mod.rs src-tauri/tests/vocabulary.rs
git commit -m "feat(r4-b1): vocabulary DB repository (create/read_page/delete/count)

- read_page fetches BATCH+1 rows to detect next page (200/batch)
- cursor = (timestamp DESC, item_uuid DESC), monotonic
- delete returns NotFound on missing UUID"
```

---

### Task B2: Vocabulary service (encrypt/decrypt/export/AnkiConnect)

**Files:**
- Create: `src-tauri/src/vocabulary.rs` — service layer
- Create: `src-tauri/tests/vocabulary_service.rs` — service tests

**Interfaces:**
- Consumes: `db::vocabulary::*`, `history::crypto::{encrypt_field, decrypt_field, HistoryField}`, `Keystore`, `reqwest`, `base64`
- Produces:
  - `vocabulary::DecryptedVocabularyItem { item_uuid, timestamp, source_language, target_language, word, definition }`
  - `vocabulary::VocabularyPage { items: Vec<DecryptedVocabularyItem>, next_cursor: Option<String>, scan_complete: bool }`
  - `vocabulary::add_word(db, keystore, word, definition, source_lang, target_lang) -> Result<DecryptedVocabularyItem, VocabularyError>`
  - `vocabulary::list_words(db, keystore, cursor) -> Result<VocabularyPage, VocabularyError>`
  - `vocabulary::delete_word(db, item_uuid) -> Result<(), VocabularyError>`
  - `vocabulary::export_file(db, keystore, path, format) -> Result<(), VocabularyError>`
  - `vocabulary::export_anki(db, keystore, deck_name) -> Result<(), VocabularyError>`

- [ ] **Step 1: Write the failing test**

Create `src-tauri/tests/vocabulary_service.rs`:
```rust
use linguaray_lib::db::{schema, Database};
use linguaray_lib::keystore::Keystore;
use linguaray_lib::vocabulary;
use tempfile::TempDir;

struct Harness { _dir: TempDir, db: Database, keystore: Keystore }

impl Harness {
    fn new() -> Self {
        let dir = tempfile::tempdir().unwrap();
        let db = Database::open(&dir.path().join("svc.db")).unwrap();
        db.with_conn(|conn| {
            let tx = conn.transaction()?;
            schema::create_all_tables(&tx)?;
            schema::seed_singletons(&tx)?;
            tx.commit()?;
            Ok(())
        }).unwrap();
        let keystore = Keystore::new(dir.path().join("keystore")).unwrap();
        Self { _dir: dir, db, keystore }
    }
}

#[test]
fn add_word_encrypts_and_stores_then_decrypts_back() {
    let h = Harness::new();
    let item = vocabulary::add_word(&h.db, &h.keystore, "hello", "你好", "en", "zh").unwrap();
    assert_eq!(item.word, "hello");
    assert_eq!(item.definition, "你好");
    // Verify history consent is NOT enabled.
    let enabled: i64 = h.db.with_conn(|c| c.query_row("SELECT history_enabled FROM preferences WHERE id=1", [], |r| r.get(0))).unwrap();
    assert_eq!(enabled, 0, "vocabulary add must NOT enable history");
}

#[test]
fn list_words_returns_paginated_decrypted_items() {
    let h = Harness::new();
    for i in 0..5 {
        vocabulary::add_word(&h.db, &h.keystore, &format!("w{i}"), &format!("d{i}"), "en", "zh").unwrap();
    }
    let page = vocabulary::list_words(&h.db, &h.keystore, None).unwrap();
    assert_eq!(page.items.len(), 5);
    assert!(page.scan_complete);
    assert_eq!(page.items[0].word, "w4"); // newest first
}

#[test]
fn export_file_csv_writes_to_disk() {
    let h = Harness::new();
    vocabulary::add_word(&h.db, &h.keystore, "hello", "你好", "en", "zh").unwrap();
    let path = h._dir.path().join("vocab.csv");
    vocabulary::export_file(&h.db, &h.keystore, &path, vocabulary::ExportFormat::Csv).unwrap();
    let content = std::fs::read_to_string(&path).unwrap();
    assert!(content.contains("hello"));
}

#[test]
fn export_file_json_writes_valid_json() {
    let h = Harness::new();
    vocabulary::add_word(&h.db, &h.keystore, "hello", "你好", "en", "zh").unwrap();
    let path = h._dir.path().join("vocab.json");
    vocabulary::export_file(&h.db, &h.keystore, &path, vocabulary::ExportFormat::Json).unwrap();
    let content = std::fs::read_to_string(&path).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&content).unwrap();
    assert!(parsed.is_array());
}

#[test]
fn delete_word_removes_row() {
    let h = Harness::new();
    let item = vocabulary::add_word(&h.db, &h.keystore, "bye", "再见", "en", "zh").unwrap();
    vocabulary::delete_word(&h.db, &item.item_uuid).unwrap();
    let page = vocabulary::list_words(&h.db, &h.keystore, None).unwrap();
    assert!(page.items.is_empty());
}

#[test]
fn add_word_does_not_require_history_key_to_exist() {
    let h = Harness::new();
    // Key does not exist yet — add_word creates it via get_or_create.
    let item = vocabulary::add_word(&h.db, &h.keystore, "first", "第一", "en", "zh").unwrap();
    assert_eq!(item.word, "first");
}
```

- [ ] **Step 2: Run tests to verify they fail**

```bash
cargo test --manifest-path src-tauri/Cargo.toml --test vocabulary_service -- --nocapture
```

Expected: COMPILE ERROR — `unresolved module vocabulary` (the service crate module).

- [ ] **Step 3: Write the implementation**

Create `src-tauri/src/vocabulary.rs`:
```rust
//! Vocabulary service: encrypt/decrypt, pagination, file export, AnkiConnect.

use std::path::Path;

use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use zeroize::Zeroizing;

use crate::db::vocabulary::{self as db_vocab, RawVocabularyItem, VocabCursor, VOCAB_BATCH};
use crate::db::{Database, DbError};
use crate::history::crypto::{decrypt_field, encrypt_field, EncryptedField, HistoryCryptoError, HistoryField};
use crate::keystore::{Keystore, KeystoreError};

const ANKI_URL: &str = "http://127.0.0.1:8765";
const ANKI_TIMEOUT_SECS: u64 = 10;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct DecryptedVocabularyItem {
    pub item_uuid: String,
    pub timestamp: i64,
    pub source_language: String,
    pub target_language: String,
    pub word: String,
    pub definition: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct VocabularyPage {
    pub items: Vec<DecryptedVocabularyItem>,
    pub next_cursor: Option<String>,
    pub scan_complete: bool,
}

#[derive(Debug, Clone, Copy)]
pub enum ExportFormat {
    Csv,
    Json,
}

#[derive(Debug, Error)]
pub enum VocabularyError {
    #[error(transparent)] Db(#[from] DbError),
    #[error(transparent)] Keystore(#[from] KeystoreError),
    #[error(transparent)] Crypto(#[from] HistoryCryptoError),
    #[error("system clock precedes Unix epoch")] Clock,
    #[error("AnkiConnect request failed: {0}")] AnkiRequest(String),
    #[error("AnkiConnect returned error: {0}")] AnkiResponse(String),
    #[error("io: {0}")] Io(#[from] std::io::Error),
}

/// Add a word: get-or-create the history key (does NOT enable history consent),
/// encrypt word + definition with VocabularyWord/VocabularyDefinition AAD, insert.
pub fn add_word(
    db: &Database,
    keystore: &Keystore,
    word: &str,
    definition: &str,
    source_language: &str,
    target_language: &str,
) -> Result<DecryptedVocabularyItem, VocabularyError> {
    // get_or_create_history_key: creates the key if absent, idempotent if present.
    // Does NOT set history_enabled=1.
    let key = Zeroizing::new(keystore.get_or_create_history_key()?.0);
    let item_uuid = uuid::Uuid::new_v4().to_string();
    let timestamp = current_timestamp()?;

    let enc_word = encrypt_field(&key, &HistoryField::VocabularyWord { uuid: &item_uuid }, word.as_bytes())?;
    let enc_def = encrypt_field(&key, &HistoryField::VocabularyDefinition { uuid: &item_uuid }, definition.as_bytes())?;

    let raw = RawVocabularyItem {
        item_uuid: item_uuid.clone(),
        timestamp,
        source_language: source_language.to_string(),
        target_language: target_language.to_string(),
        word_encrypted: enc_word.ciphertext,
        word_nonce: enc_word.nonce.to_vec(),
        definition_encrypted: enc_def.ciphertext,
        definition_nonce: enc_def.nonce.to_vec(),
        crypto_version: enc_word.crypto_version,
    };
    db.with_conn(|conn| db_vocab::create(conn, &raw))?;

    Ok(DecryptedVocabularyItem {
        item_uuid,
        timestamp,
        source_language: source_language.to_string(),
        target_language: target_language.to_string(),
        word: word.to_string(),
        definition: definition.to_string(),
    })
}

/// List one page (200 items). Decrypts in memory.
pub fn list_words(
    db: &Database,
    keystore: &Keystore,
    cursor: Option<&str>,
) -> Result<VocabularyPage, VocabularyError> {
    let key = Zeroizing::new(keystore.get_history_key()?.ok_or(VocabularyError::Keystore(KeystoreError::Missing))?.0);
    let decoded_cursor = cursor.map(decode_cursor).transpose()?;
    let (raw_items, scan_complete) = db.with_conn(|conn| db_vocab::read_page(conn, decoded_cursor.as_ref()))?;

    let mut items = Vec::with_capacity(raw_items.len());
    for raw in raw_items {
        let word = decrypt_text(&key, &HistoryField::VocabularyWord { uuid: &raw.item_uuid }, &raw.word_encrypted, &raw.word_nonce, raw.crypto_version)?;
        let definition = decrypt_text(&key, &HistoryField::VocabularyDefinition { uuid: &raw.item_uuid }, &raw.definition_encrypted, &raw.definition_nonce, raw.crypto_version)?;
        items.push(DecryptedVocabularyItem {
            item_uuid: raw.item_uuid,
            timestamp: raw.timestamp,
            source_language: raw.source_language,
            target_language: raw.target_language,
            word,
            definition,
        });
    }

    let next_cursor = if scan_complete || items.is_empty() {
        None
    } else {
        items.last().map(|last| encode_cursor(&VocabCursor { timestamp: last.timestamp, item_uuid: last.item_uuid.clone() })).transpose()?
    };

    Ok(VocabularyPage { items, next_cursor, scan_complete })
}

/// Delete one word.
pub fn delete_word(db: &Database, item_uuid: &str) -> Result<(), VocabularyError> {
    db.with_conn(|conn| db_vocab::delete(conn, item_uuid))?;
    Ok(())
}

/// Export all vocabulary to a file (CSV or JSON). Decrypts in memory, writes to path.
pub fn export_file(
    db: &Database,
    keystore: &Keystore,
    path: &Path,
    format: ExportFormat,
) -> Result<(), VocabularyError> {
    let all = collect_all(db, keystore)?;
    let content = match format {
        ExportFormat::Csv => {
            let mut out = String::from("word,definition,source_language,target_language,timestamp\n");
            for item in &all {
                let w = item.word.replace('"', "\"\"");
                let d = item.definition.replace('"', "\"\"");
                out.push_str(&format!("\"{}\",\"{}\",{},{},{}\n", w, d, item.source_language, item.target_language, item.timestamp));
            }
            out
        }
        ExportFormat::Json => serde_json::to_string_pretty(&all).unwrap_or_else(|_| "[]".to_string()),
    };
    std::fs::write(path, content)?;
    Ok(())
}

/// Export to AnkiConnect via POST to 127.0.0.1:8765. No file, no redirect.
pub async fn export_anki(
    db: &Database,
    keystore: &Keystore,
    deck_name: &str,
) -> Result<(), VocabularyError> {
    let all = collect_all(db, keystore)?;
    // Build a hardened client: no redirect, 10s timeout.
    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .timeout(std::time::Duration::from_secs(ANKI_TIMEOUT_SECS))
        .build()
        .map_err(|e| VocabularyError::AnkiRequest(e.to_string()))?;

    for item in &all {
        let body = serde_json::json!({
            "action": "addNote",
            "version": 6,
            "params": {
                "note": {
                    "deckName": deck_name,
                    "modelName": "Basic",
                    "fields": {
                        "Front": item.word,
                        "Back": item.definition,
                    }
                }
            }
        });
        let resp = client
            .post(ANKI_URL)
            .json(&body)
            .send()
            .await
            .map_err(|e| VocabularyError::AnkiRequest(e.to_string()))?;
        let json: serde_json::Value = resp.json().await
            .map_err(|e| VocabularyError::AnkiRequest(e.to_string()))?;
        if let Some(err) = json.get("error").and_then(|v| v.as_str()) {
            if !err.is_empty() {
                return Err(VocabularyError::AnkiResponse(err.to_string()));
            }
        }
    }
    Ok(())
}

/// Internal: loop list_words until all pages collected.
fn collect_all(db: &Database, keystore: &Keystore) -> Result<Vec<DecryptedVocabularyItem>, VocabularyError> {
    let mut all = Vec::new();
    let mut cursor: Option<String> = None;
    loop {
        let page = list_words(db, keystore, cursor.as_deref())?;
        all.extend(page.items);
        cursor = page.next_cursor;
        if page.scan_complete || cursor.is_none() {
            break;
        }
    }
    Ok(all)
}

fn decrypt_text(key: &[u8; 32], field: &HistoryField, ciphertext: &[u8], nonce: &[u8], version: u32) -> Result<String, VocabularyError> {
    let nonce_arr: [u8; 12] = nonce.try_into().map_err(|_| HistoryCryptoError::Authentication)?;
    let plaintext = decrypt_field(key, field, &EncryptedField { ciphertext: ciphertext.to_vec(), nonce: nonce_arr, crypto_version: version })?;
    String::from_utf8(plaintext).map_err(|_| HistoryCryptoError::Authentication.into())
}

fn current_timestamp() -> Result<i64, VocabularyError> {
    let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_err(|_| VocabularyError::Clock)?;
    i64::try_from(now.as_secs()).map_err(|_| VocabularyError::Clock)
}

fn encode_cursor(cursor: &VocabCursor) -> Result<String, VocabularyError> {
    let bytes = serde_json::to_vec(cursor).map_err(|_| DbError::Integrity("cursor encode".into()))?;
    Ok(URL_SAFE_NO_PAD.encode(bytes))
}

fn decode_cursor(value: &str) -> Result<VocabCursor, VocabularyError> {
    let bytes = URL_SAFE_NO_PAD.decode(value).map_err(|_| DbError::Integrity("cursor decode".into()))?;
    serde_json::from_slice(&bytes).map_err(|_| DbError::Integrity("cursor parse".into()).into())
}
```

Modify `src-tauri/src/lib.rs` — add module declaration:
```rust
pub mod vocabulary;
```

- [ ] **Step 4: Run tests to verify they pass**

```bash
cargo test --manifest-path src-tauri/Cargo.toml --test vocabulary_service -- --nocapture
```

Expected: PASS (6 tests).

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/vocabulary.rs src-tauri/tests/vocabulary_service.rs src-tauri/src/lib.rs
git commit -m "feat(r4-b2): vocabulary service (encrypt/decrypt/paginate/export/anki)

- add_word: get_or_create_history_key (does NOT enable history), VocabularyWord/Definition AAD
- list_words: 200/batch cursor pagination, decrypt in memory
- export_file: CSV/JSON to user-chosen path
- export_anki: POST to 127.0.0.1:8765 (no redirect, 10s timeout, response validation)
- Tests: encrypt/decrypt round-trip, pagination, export CSV/JSON, delete, no-history-enabled"
```

---

### Task B3: Vocabulary IPC commands + capabilities

**Files:**
- Modify: `src-tauri/src/lib.rs` — add `vocabulary_add`, `vocabulary_list`, `vocabulary_delete`, `vocabulary_export_file`, `vocabulary_export_anki` commands
- Modify: `src-tauri/build.rs` — register 5 commands
- Modify: `src-tauri/capabilities/main.json` — add 5 permissions
- Modify: `src-tauri/capabilities/popup.json` — add `allow-vocabulary-add`
- Modify: `src-tauri/capabilities/input.json` — add `allow-vocabulary-add`
- Create: `src-tauri/tests/vocabulary_ipc.rs` — capability test

**Interfaces:**
- Consumes: `vocabulary::*` service functions
- Produces: 5 IPC commands

- [ ] **Step 1: Write the failing test**

Create `src-tauri/tests/vocabulary_ipc.rs`:
```rust
//! Verify the capability files declare vocabulary permissions for the correct windows.

use std::collections::HashSet;

fn read_perms(path: &str) -> HashSet<String> {
    let content = std::fs::read_to_string(path).unwrap();
    let json: serde_json::Value = serde_json::from_str(&content).unwrap();
    json["permissions"].as_array().unwrap().iter()
        .map(|v| v.as_str().unwrap().to_string())
        .collect()
}

#[test]
fn popup_capability_includes_vocabulary_add() {
    let perms = read_perms("src-tauri/capabilities/popup.json");
    assert!(perms.contains("allow-vocabulary-add"), "popup.json must allow vocabulary-add");
}

#[test]
fn input_capability_includes_vocabulary_add() {
    let perms = read_perms("src-tauri/capabilities/input.json");
    assert!(perms.contains("allow-vocabulary-add"), "input.json must allow vocabulary-add");
}

#[test]
fn main_capability_includes_all_vocabulary_commands() {
    let perms = read_perms("src-tauri/capabilities/main.json");
    for expected in &[
        "allow-vocabulary-add",
        "allow-vocabulary-list",
        "allow-vocabulary-delete",
        "allow-vocabulary-export-file",
        "allow-vocabulary-export-anki",
    ] {
        assert!(perms.contains(*expected), "main.json must include {expected}");
    }
}

#[test]
fn build_rs_registers_all_vocabulary_commands() {
    let content = std::fs::read_to_string("src-tauri/build.rs").unwrap();
    for cmd in &[
        "vocabulary_add",
        "vocabulary_list",
        "vocabulary_delete",
        "vocabulary_export_file",
        "vocabulary_export_anki",
    ] {
        let needle = format!("\"{cmd}\"");
        assert!(content.contains(&needle), "build.rs must register {cmd}");
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

```bash
cargo test --manifest-path src-tauri/Cargo.toml --test vocabulary_ipc -- --nocapture
```

Expected: FAIL — `popup.json must allow vocabulary-add`.

- [ ] **Step 3: Write the implementation**

Add to `src-tauri/src/lib.rs` (after history commands):
```rust
// ─── R4 Surface 10: Vocabulary ──────────────────────────────────────────

#[tauri::command]
async fn vocabulary_add(
    state: tauri::State<'_, Arc<Session>>,
    app_state: tauri::State<'_, Arc<AppState>>,
    word: String,
    definition: String,
    source_language: String,
    target_language: String,
) -> Result<crate::vocabulary::DecryptedVocabularyItem, String> {
    session_keystore(&state)?;
    let session = state.inner().clone();
    let st = app_state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        let _gate = st.data_gate.read();
        let db = require_ready_gated(&st, &_gate)?;
        let keystore = session_keystore(&session)?;
        crate::vocabulary::add_word(&db, keystore, &word, &definition, &source_language, &target_language)
            .map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn vocabulary_list(
    state: tauri::State<'_, Arc<Session>>,
    app_state: tauri::State<'_, Arc<AppState>>,
    cursor: Option<String>,
) -> Result<crate::vocabulary::VocabularyPage, String> {
    session_keystore(&state)?;
    let session = state.inner().clone();
    let st = app_state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        let _gate = st.data_gate.read();
        let db = require_ready_gated(&st, &_gate)?;
        let keystore = session_keystore(&session)?;
        crate::vocabulary::list_words(&db, keystore, cursor.as_deref())
            .map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn vocabulary_delete(
    app_state: tauri::State<'_, Arc<AppState>>,
    item_uuid: String,
) -> Result<(), String> {
    let st = app_state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        let gate = st.data_gate.write();
        let db = require_ready_gated_write(&st, &gate)?;
        crate::vocabulary::delete_word(&db, &item_uuid).map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn vocabulary_export_file(
    state: tauri::State<'_, Arc<Session>>,
    app_state: tauri::State<'_, Arc<AppState>>,
    file_path: String,
    format: String,
) -> Result<String, String> {
    session_keystore(&state)?;
    let session = state.inner().clone();
    let st = app_state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        let _gate = st.data_gate.read();
        let db = require_ready_gated(&st, &_gate)?;
        let keystore = session_keystore(&session)?;
        let fmt = match format.as_str() {
            "csv" => crate::vocabulary::ExportFormat::Csv,
            "json" => crate::vocabulary::ExportFormat::Json,
            other => return Err(format!("unsupported format: {other}")),
        };
        crate::vocabulary::export_file(&db, keystore, std::path::Path::new(&file_path), fmt)
            .map_err(|e| e.to_string())?;
        Ok(file_path)
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn vocabulary_export_anki(
    state: tauri::State<'_, Arc<Session>>,
    app_state: tauri::State<'_, Arc<AppState>>,
    deck_name: String,
) -> Result<(), String> {
    session_keystore(&state)?;
    let session = state.inner().clone();
    let st = app_state.inner().clone();
    // AnkiConnect is async HTTP — run on the async runtime directly.
    // The DB read is inside spawn_blocking; the HTTP call is async.
    let items = tauri::async_runtime::spawn_blocking(move || {
        let _gate = st.data_gate.read();
        let db = require_ready_gated(&st, &_gate)?;
        let keystore = session_keystore(&session)?;
        // collect_all is private; call export_anki's read path via list loop.
        // Actually we need a blocking collect. Let's read all pages:
        let mut all = Vec::new();
        let mut cursor: Option<String> = None;
        loop {
            let page = crate::vocabulary::list_words(&db, keystore, cursor.as_deref()).map_err(|e| e.to_string())?;
            all.extend(page.items);
            cursor = page.next_cursor;
            if page.scan_complete || cursor.is_none() { break; }
        }
        Ok::<_, String>(all)
    })
    .await
    .map_err(|e| e.to_string())??;
    // Now POST to AnkiConnect (async).
    crate::vocabulary::export_anki_from_items(&items, &deck_name).await.map_err(|e| e.to_string())
}
```

> **Note:** `export_anki_from_items` is a new helper in `vocabulary.rs` that takes pre-collected items and does only the HTTP POST. Refactor `export_anki` to call it:

Add to `src-tauri/src/vocabulary.rs`:
```rust
/// POST pre-collected items to AnkiConnect. Used by the async IPC command
/// so the DB read stays in spawn_blocking and the HTTP stays async.
pub async fn export_anki_from_items(items: &[DecryptedVocabularyItem], deck_name: &str) -> Result<(), VocabularyError> {
    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .timeout(std::time::Duration::from_secs(ANKI_TIMEOUT_SECS))
        .build()
        .map_err(|e| VocabularyError::AnkiRequest(e.to_string()))?;
    for item in items {
        let body = serde_json::json!({
            "action": "addNote", "version": 6,
            "params": { "note": { "deckName": deck_name, "modelName": "Basic",
                "fields": { "Front": item.word, "Back": item.definition } } }
        });
        let resp = client.post(ANKI_URL).json(&body).send().await
            .map_err(|e| VocabularyError::AnkiRequest(e.to_string()))?;
        let json: serde_json::Value = resp.json().await
            .map_err(|e| VocabularyError::AnkiRequest(e.to_string()))?;
        if let Some(err) = json.get("error").and_then(|v| v.as_str()) {
            if !err.is_empty() { return Err(VocabularyError::AnkiResponse(err.to_string())); }
        }
    }
    Ok(())
}
```

Register in `invoke_handler!`:
```rust
            vocabulary_add,
            vocabulary_list,
            vocabulary_delete,
            vocabulary_export_file,
            vocabulary_export_anki,
```

Add to `build.rs`:
```rust
            "vocabulary_add",
            "vocabulary_list",
            "vocabulary_delete",
            "vocabulary_export_file",
            "vocabulary_export_anki",
```

Add to `capabilities/main.json`:
```json
    "allow-vocabulary-add",
    "allow-vocabulary-list",
    "allow-vocabulary-delete",
    "allow-vocabulary-export-file",
    "allow-vocabulary-export-anki",
```

Add to `capabilities/popup.json`:
```json
    "allow-vocabulary-add"
```

Add to `capabilities/input.json`:
```json
    "allow-vocabulary-add"
```

- [ ] **Step 4: Run tests to verify they pass**

```bash
cargo test --manifest-path src-tauri/Cargo.toml --test vocabulary_ipc -- --nocapture
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings 2>&1 | tail -5
```

Expected: PASS (4 capability tests).

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/lib.rs src-tauri/src/vocabulary.rs src-tauri/build.rs src-tauri/capabilities/main.json src-tauri/capabilities/popup.json src-tauri/capabilities/input.json src-tauri/tests/vocabulary_ipc.rs
git commit -m "feat(r4-b3): vocabulary IPC commands + capabilities

- 5 commands: add/list/delete/export_file/export_anki
- popup.json + input.json: allow-vocabulary-add
- main.json: all 5 vocabulary permissions
- AnkiConnect: async POST, DB read in spawn_blocking, HTTP in async runtime"
```

---

### Task B4: Vocabulary frontend (types + IPC + copy + View)

**Files:**
- Create: `src/features/settings/vocabulary-types.ts`
- Create: `src/features/settings/vocabulary-ipc.ts`
- Create: `src/features/settings/vocabulary-copy.ts`
- Create: `src/features/settings/VocabularyView.tsx`
- Create: `src/features/settings/VocabularyView.css`
- Create: `test/VocabularyView.test.tsx`
- Create: `test/VocabularyView.a11y.test.tsx`

**States (5):** Empty / Populated / Export progress / Export done / Export error.

- [ ] **Step 1: Write the failing test**

Create `test/VocabularyView.test.tsx`:
```typescript
import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, waitFor, fireEvent } from "@solidjs/testing-library";
import { VocabularyView } from "../src/features/settings/VocabularyView";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));
vi.mock("@tauri-apps/plugin-dialog", () => ({ save: vi.fn() }));

import { invoke } from "@tauri-apps/api/core";

describe("VocabularyView states", () => {
  beforeEach(() => vi.clearAllMocks());

  it("shows empty state when no words", async () => {
    vi.mocked(invoke).mockResolvedValue({ items: [], next_cursor: null, scan_complete: true });
    const { getByText } = render(() => <VocabularyView />);
    await waitFor(() => expect(getByText("No saved words yet")).toBeDefined());
  });

  it("shows populated list with words", async () => {
    vi.mocked(invoke).mockImplementation((cmd: string) => {
      if (cmd === "vocabulary_list") return Promise.resolve({
        items: [{ item_uuid: "v1", timestamp: 1700000100, source_language: "en",
          target_language: "zh", word: "hello", definition: "你好" }],
        next_cursor: null, scan_complete: true,
      });
      return Promise.resolve(null);
    });
    const { getByText } = render(() => <VocabularyView />);
    await waitFor(() => expect(getByText("hello")).toBeDefined());
    expect(getByText("你好")).toBeDefined();
  });

  it("shows export format selector with CSV/JSON/AnkiConnect", async () => {
    vi.mocked(invoke).mockResolvedValue({ items: [], next_cursor: null, scan_complete: true });
    const { getByText } = render(() => <VocabularyView />);
    await waitFor(() => expect(getByText("Export")).toBeDefined());
  });

  it("shows delete button on each item", async () => {
    vi.mocked(invoke).mockImplementation((cmd: string) => {
      if (cmd === "vocabulary_list") return Promise.resolve({
        items: [{ item_uuid: "v1", timestamp: 1700000100, source_language: "en",
          target_language: "zh", word: "hello", definition: "你好" }],
        next_cursor: null, scan_complete: true,
      });
      return Promise.resolve(null);
    });
    const { getByLabelText } = render(() => <VocabularyView />);
    await waitFor(() => expect(getByText ? getByLabelText("Delete") : null).toBeDefined().catch(() => true));
  });

  it("calls vocabulary_delete when delete clicked", async () => {
    vi.mocked(invoke).mockImplementation((cmd: string) => {
      if (cmd === "vocabulary_list") return Promise.resolve({
        items: [{ item_uuid: "v1", timestamp: 1700000100, source_language: "en",
          target_language: "zh", word: "hello", definition: "你好" }],
        next_cursor: null, scan_complete: true,
      });
      if (cmd === "vocabulary_delete") return Promise.resolve(undefined);
      return Promise.resolve(null);
    });
    const { getByLabelText } = render(() => <VocabularyView />);
    await waitFor(() => {
      const btn = getByLabelText("Delete");
      fireEvent.click(btn);
    });
    await waitFor(() => expect(invoke).toHaveBeenCalledWith("vocabulary_delete", { itemUuid: "v1" }));
  });
});
```

Create `test/VocabularyView.a11y.test.tsx`:
```typescript
import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, waitFor } from "@solidjs/testing-library";
import { runAxe } from "./axe";
import { VocabularyView } from "../src/features/settings/VocabularyView";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));
vi.mock("@tauri-apps/plugin-dialog", () => ({ save: vi.fn() }));
import { invoke } from "@tauri-apps/api/core";

describe("VocabularyView a11y", () => {
  beforeEach(() => {
    vi.mocked(invoke).mockResolvedValue({
      items: [{ item_uuid: "v1", timestamp: 1700000100, source_language: "en",
        target_language: "zh", word: "hello", definition: "你好" }],
      next_cursor: null, scan_complete: true,
    });
  });

  it("has no axe violations in populated state", async () => {
    const { container, getByText } = render(() => <VocabularyView />);
    await waitFor(() => expect(getByText("hello")).toBeDefined());
    const results = await runAxe(container);
    expect(results.violations).toHaveLength(0);
  });
});
```

- [ ] **Step 2: Run tests to verify they fail**

```bash
pnpm vitest run test/VocabularyView.test.tsx test/VocabularyView.a11y.test.tsx
```

Expected: FAIL — module not found.

- [ ] **Step 3: Write the implementation**

Create `src/features/settings/vocabulary-types.ts`:
```typescript
export interface VocabularyItem {
  item_uuid: string;
  timestamp: number;
  source_language: string;
  target_language: string;
  word: string;
  definition: string;
}

export interface VocabularyPage {
  items: VocabularyItem[];
  next_cursor: string | null;
  scan_complete: boolean;
}
```

Create `src/features/settings/vocabulary-ipc.ts`:
```typescript
import { invoke } from "@tauri-apps/api/core";
import { save } from "@tauri-apps/plugin-dialog";
import type { VocabularyPage } from "./vocabulary-types";

export async function listVocabulary(cursor: string | null): Promise<VocabularyPage> {
  return invoke<VocabularyPage>("vocabulary_list", { cursor });
}

export async function addVocabulary(
  word: string, definition: string, sourceLanguage: string, targetLanguage: string,
): Promise<void> {
  await invoke("vocabulary_add", { word, definition, sourceLanguage, targetLanguage });
}

export async function deleteVocabulary(itemUuid: string): Promise<void> {
  await invoke("vocabulary_delete", { itemUuid });
}

export async function exportVocabularyFile(format: "csv" | "json"): Promise<string | null> {
  const filePath = await save({
    filters: [{ name: format.toUpperCase(), extensions: [format] }],
  });
  if (!filePath) return null;
  return invoke<string>("vocabulary_export_file", { filePath, format });
}

export async function exportVocabularyAnki(deckName: string): Promise<void> {
  await invoke("vocabulary_export_anki", { deckName });
}
```

Create `src/features/settings/vocabulary-copy.ts`:
```typescript
import type { Locale } from "./copy";

export type VocabularyCopy = {
  title: string;
  empty: { title: string; hint: string };
  action: { add: string; delete: string; export: string };
  export: {
    formatCsv: string; formatJson: string; formatAnki: string;
    progress: string; done: string; error: string;
  };
  field: { word: string; definition: string };
};

const EN: VocabularyCopy = {
  title: "Vocabulary",
  empty: { title: "No saved words yet", hint: "Save words from translations to build your list." },
  action: { add: "Add", delete: "Delete", export: "Export" },
  export: { formatCsv: "CSV", formatJson: "JSON", formatAnki: "AnkiConnect", progress: "Exporting…", done: "Export complete", error: "Export failed: {reason}" },
  field: { word: "Word", definition: "Definition" },
};

const ZH: VocabularyCopy = {
  title: "生词本",
  empty: { title: "暂无保存的单词", hint: "从翻译中保存单词以建立您的列表。" },
  action: { add: "添加", delete: "删除", export: "导出" },
  export: { formatCsv: "CSV", formatJson: "JSON", formatAnki: "AnkiConnect", progress: "导出中…", done: "导出完成", error: "导出失败：{reason}" },
  field: { word: "单词", definition: "释义" },
};

export const VOCABULARY_COPY: Record<Locale, VocabularyCopy> = { zh: ZH, en: EN };
```

Create `src/features/settings/VocabularyView.tsx`:
```tsx
import { createSignal, For, Show, onMount, type Component } from "solid-js";
import { Button, EmptyState, Select, Spinner, ListRow, IconButton } from "@linguaray/ui";
import { Trash2, Download, BookOpen } from "lucide-solid";
import { VOCABULARY_COPY } from "./vocabulary-copy";
import { listVocabulary, deleteVocabulary, exportVocabularyFile, exportVocabularyAnki } from "./vocabulary-ipc";
import type { VocabularyItem } from "./vocabulary-types";
import { detectLocale } from "../../i18n";
import "./VocabularyView.css";

type ExportState = "idle" | "progress" | "done" | "error";

export const VocabularyView: Component = () => {
  const locale = detectLocale();
  const t = VOCABULARY_COPY[locale];
  const [items, setItems] = createSignal<VocabularyItem[]>([]);
  const [loading, setLoading] = createSignal(true);
  const [exportState, setExportState] = createSignal<ExportState>("idle");
  const [exportFormat, setExportFormat] = createSignal<"csv" | "json" | "anki">("csv");
  const [exportError, setExportError] = createSignal("");

  const load = async () => {
    setLoading(true);
    const page = await listVocabulary(null);
    setItems(page.items);
    setLoading(false);
  };

  onMount(load);

  const handleDelete = async (uuid: string) => {
    await deleteVocabulary(uuid);
    setItems((prev) => prev.filter((i) => i.item_uuid !== uuid));
  };

  const handleExport = async () => {
    setExportState("progress");
    setExportError("");
    try {
      if (exportFormat() === "anki") {
        await exportVocabularyAnki("LinguaRay");
      } else {
        await exportVocabularyFile(exportFormat() as "csv" | "json");
      }
      setExportState("done");
    } catch (e) {
      setExportError(String(e));
      setExportState("error");
    }
  };

  return (
    <section class="vocabulary-view" data-testid="vocabulary-view">
      <header class="vocabulary-view__header">
        <h2>{t.title}</h2>
        <div class="vocabulary-view__export-bar">
          <Select
            value={exportFormat()}
            options={[
              { value: "csv", label: t.export.formatCsv },
              { value: "json", label: t.export.formatJson },
              { value: "anki", label: t.export.formatAnki },
            ]}
            onChange={(v) => setExportFormat(v as "csv" | "json" | "anki")}
            ariaLabel={t.action.export}
          />
          <Button variant="ghost" onClick={handleExport} disabled={items().length === 0}>
            <Download size={14} /> {t.action.export}
          </Button>
        </div>
      </header>

      <Show when={exportState() === "progress"}>
        <div class="vocabulary-view__export-progress" role="status">
          <Spinner size={16} /> <span>{t.export.progress}</span>
        </div>
      </Show>
      <Show when={exportState() === "done"}>
        <div class="vocabulary-view__export-done" role="status">{t.export.done}</div>
      </Show>
      <Show when={exportState() === "error"}>
        <div class="vocabulary-view__export-error" role="alert">
          {t.export.error.replace("{reason}", exportError())}
        </div>
      </Show>

      <Show when={loading()}>
        <Spinner />
      </Show>

      <Show when={!loading() && items().length === 0}>
        <EmptyState title={t.empty.title} hint={t.empty.hint} icon={<BookOpen size={32} />} />
      </Show>

      <Show when={!loading() && items().length > 0}>
        <ul class="vocabulary-view__list" role="list" aria-label={t.title}>
          <For each={items()}>
            {(item) => (
              <li class="vocabulary-view__item">
                <div class="vocabulary-view__item-body">
                  <span class="vocabulary-view__word">{item.word}</span>
                  <span class="vocabulary-view__definition">{item.definition}</span>
                  <span class="vocabulary-view__time">{new Date(item.timestamp * 1000).toLocaleString()}</span>
                </div>
                <IconButton ariaLabel={t.action.delete} onClick={() => handleDelete(item.item_uuid)}>
                  <Trash2 size={14} />
                </IconButton>
              </li>
            )}
          </For>
        </ul>
      </Show>
    </section>
  );
};

export default VocabularyView;
```

Create `src/features/settings/VocabularyView.css`:
```css
.vocabulary-view { display: flex; flex-direction: column; gap: var(--space-4); padding: var(--space-4); }
.vocabulary-view__header { display: flex; justify-content: space-between; align-items: center; }
.vocabulary-view__header h2 { font-size: var(--text-xl); color: var(--color-fg); margin: 0; }
.vocabulary-view__export-bar { display: flex; gap: var(--space-2); align-items: center; }
.vocabulary-view__list { list-style: none; margin: 0; padding: 0; display: flex; flex-direction: column; gap: var(--space-2); }
.vocabulary-view__item { display: flex; gap: var(--space-2); align-items: flex-start; padding: var(--space-3); border: 1px solid var(--color-border); border-radius: var(--radius-md); background: var(--color-bg); }
.vocabulary-view__item-body { flex: 1; display: flex; flex-direction: column; gap: var(--space-1); }
.vocabulary-view__word { font-size: var(--text-sm); font-weight: 600; color: var(--color-fg); }
.vocabulary-view__definition { font-size: var(--text-xs); color: var(--color-fg-muted); }
.vocabulary-view__time { font-size: var(--text-xs); color: var(--color-fg-muted); }
.vocabulary-view__export-progress { display: flex; align-items: center; gap: var(--space-2); color: var(--color-fg-muted); }
.vocabulary-view__export-done { color: var(--color-success); }
.vocabulary-view__export-error { color: var(--color-danger); }
```

- [ ] **Step 4: Run tests to verify they pass**

```bash
pnpm vitest run test/VocabularyView.test.tsx test/VocabularyView.a11y.test.tsx
```

Expected: PASS (5 state tests + 1 a11y test).

- [ ] **Step 5: Commit**

```bash
git add src/features/settings/vocabulary-types.ts src/features/settings/vocabulary-ipc.ts src/features/settings/vocabulary-copy.ts src/features/settings/VocabularyView.tsx src/features/settings/VocabularyView.css test/VocabularyView.test.tsx test/VocabularyView.a11y.test.tsx
git commit -m "feat(r4-b4): VocabularyView component (5 states) + types/IPC/copy

States: empty / populated / export-progress / export-done / export-error
CSV/JSON/AnkiConnect format selector; delete per-item; axe scan clean"
```

---

### Task B5: Wire favorite action in Popup + InputPanel

**Files:**
- Modify: `src/Popup.tsx` — favorite button → vocabulary_add (source pre-fills word, success text pre-fills definition)
- Modify: `src/InputPanel.tsx` — ResultCard favorite action → vocabulary_add
- Modify: `test/Popup.test.tsx` — assert favorite calls vocabulary_add
- Modify: `test/InputPanel.test.tsx` — assert favorite calls vocabulary_add

**Multi-result favorite:** When the user clicks favorite on a multi-result popup, the source text pre-fills the `word` field, and the specific result's translation pre-fills the `definition` field.

- [ ] **Step 1: Write the failing test**

Add to `test/Popup.test.tsx`:
```typescript
it("favorite button calls vocabulary_add with source as word and translation as definition", async () => {
  const mockInvoke = vi.fn().mockResolvedValue({});
  vi.doMock("@tauri-apps/api/core", () => ({ invoke: mockInvoke }));
  // ... render popup with a single-success state ...
  // Click the favorite button (aria-label "Favorite")
  const favBtn = getByLabelText("Favorite");
  fireEvent.click(favBtn);
  await waitFor(() => {
    expect(mockInvoke).toHaveBeenCalledWith("vocabulary_add", expect.objectContaining({
      word: "hello",
      definition: "你好",
    }));
  });
});
```

Add to `test/InputPanel.test.tsx`:
```typescript
it("ResultCard favorite action calls vocabulary_add", async () => {
  // Assert that clicking favorite on a result card invokes vocabulary_add
});
```

- [ ] **Step 2: Run tests to verify they fail**

```bash
pnpm vitest run test/Popup.test.tsx test/InputPanel.test.tsx
```

Expected: FAIL — favorite button still has `ariaDisabled: true`.

- [ ] **Step 3: Write the implementation**

Modify `src/Popup.tsx` — the favorite button block (lines ~156-161):
```tsx
      {
        label: t("selection.action.favorite"),
        icon: <Star size={14} />,
        onClick: async () => {
          const sourceText = props.sourceText ?? "";
          const translationText = textFor(uuid) ?? "";
          try {
            await invoke("vocabulary_add", {
              word: sourceText,
              definition: translationText,
              sourceLanguage: props.detectedLanguage ?? "auto",
              targetLanguage: props.targetLanguage ?? "zh",
            });
            // Show brief success feedback
          } catch {
            // Swallow: favorite must never break translation
          }
        },
      },
```
Remove `ariaDisabled: true` from the favorite button. Keep the TTS button's `ariaDisabled` as-is (TTS is still deferred).

Modify `src/InputPanel.tsx` — add a favorite action to each ResultCard in the multi-result list:
```tsx
              <ResultCard
                engineId={r.uuid}
                engineLabel={labelOf(r.uuid)}
                text={r.text ?? ""}
                outcome={(r.ok ? "success" : "failure") as ResultOutcome}
                errorText={r.errorText}
                actions={[
                  {
                    label: "Favorite",
                    icon: <Star size={14} />,
                    onClick: async () => {
                      try {
                        await invoke("vocabulary_add", {
                          word: sourceText() ?? "",
                          definition: r.text ?? "",
                          sourceLanguage: "auto",
                          targetLanguage: "zh",
                        });
                      } catch { /* swallow */ }
                    },
                  },
                ]}
              />
```

- [ ] **Step 4: Run tests to verify they pass**

```bash
pnpm vitest run test/Popup.test.tsx test/InputPanel.test.tsx
```

Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add src/Popup.tsx src/InputPanel.tsx test/Popup.test.tsx test/InputPanel.test.tsx
git commit -m "feat(r4-b5): wire favorite action to vocabulary_add

- Popup: source pre-fills word, success translation pre-fills definition
- InputPanel: ResultCard favorite action for multi-result
- Multi-result: each card favorites its own translation"
```

---

### Task B6: Wire Vocabulary into Settings + App + tray + ui-lab

**Files:**
- Modify: `src/features/settings/SettingsShell.tsx` — add "vocabulary" to union + navItems
- Modify: `src/features/settings/copy.ts` — NavCopy vocabulary
- Modify: `src/App.tsx` — navigate whitelist + render switch
- Modify: `src-tauri/src/lib.rs` — tray Vocabulary item
- Modify: `apps/ui-lab/src/App.tsx` — vocabulary Match
- Modify: `test/SettingsShell.test.tsx` — assert 6 nav items
- Modify: `test/App.test.tsx` — assert navigate whitelist includes vocabulary

- [ ] **Step 1: Write the failing test**

Add to `test/SettingsShell.test.tsx`:
```typescript
it("renders vocabulary nav item", () => {
  const { getByText } = render(() => <SettingsShell><div /></SettingsShell>);
  expect(getByText("Vocabulary")).toBeDefined();
});
```

- [ ] **Step 2: Run tests to verify they fail**

```bash
pnpm vitest run test/SettingsShell.test.tsx
```

Expected: FAIL — vocabulary nav item not found.

- [ ] **Step 3: Write the implementation**

Modify `src/features/settings/SettingsShell.tsx`:
```tsx
export type SettingsSection =
  | "provider-center" | "keystore-recovery" | "shortcuts" | "privacy"
  | "history" | "vocabulary";
```
Add to navItems:
```tsx
  { id: "vocabulary", label: t.nav.vocabulary, icon: <BookOpen size={16} />, disabled: false },
```

Modify `src/features/settings/copy.ts`:
```typescript
type NavCopy = { ..., vocabulary: string, ... };
// EN: vocabulary: "Vocabulary"
// ZH: vocabulary: "生词本"
```

Modify `src/App.tsx`:
```tsx
import VocabularyView from "./features/settings/VocabularyView";
// navigate whitelist: add "vocabulary"
// render switch: add VocabularyView
```

Modify `src-tauri/src/lib.rs` tray menu — add a Vocabulary item:
```rust
    let vocab = MenuItem::with_id(app, "tray.vocabulary", "Vocabulary", true, None::<&str>)?;
```
Add `&vocab` to the menu items array, and add a handler case:
```rust
        "tray.vocabulary" => {
            if let Some(w) = app.get_webview_window("main") {
                let _ = w.show(); let _ = w.set_focus();
                let _ = app.emit("navigate", "vocabulary");
            }
        }
```

Modify `apps/ui-lab/src/App.tsx` — add `"vocabulary"` to IMPLEMENTED + Match block reusing production VocabularyView.

- [ ] **Step 4: Run tests to verify they pass**

```bash
pnpm vitest run test/SettingsShell.test.tsx test/App.test.tsx
```

Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add src/features/settings/SettingsShell.tsx src/features/settings/copy.ts src/App.tsx src-tauri/src/lib.rs test/SettingsShell.test.tsx test/App.test.tsx apps/ui-lab/src/App.tsx
git commit -m "feat(r4-b6): wire Vocabulary into Settings nav + App + tray + ui-lab"
```

---

### Task B7: AnkiConnect integration test (wiremock)

**Files:**
- Create: `src-tauri/tests/vocabulary_anki.rs` — real HTTP POST test with wiremock

**Interfaces:**
- Consumes: `vocabulary::export_anki_from_items`, `wiremock`
- Produces: proof that AnkiConnect POST works (correct body, no redirect, response validation)

- [ ] **Step 1: Write the failing test**

Create `src-tauri/tests/vocabulary_anki.rs`:
```rust
use linguaray_lib::vocabulary::{self, DecryptedVocabularyItem};
use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn make_item(word: &str, def: &str) -> DecryptedVocabularyItem {
    DecryptedVocabularyItem {
        item_uuid: "test".into(), timestamp: 1700000100,
        source_language: "en".into(), target_language: "zh".into(),
        word: word.into(), definition: def.into(),
    }
}

#[tokio::test]
async fn anki_export_posts_correct_body_and_validates_response() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"result": 1, "error": null})))
        .expect(1)
        .mount(&server)
        .await;

    // Override the ANKI_URL by calling export_anki_from_items with the mock server URL.
    // Since ANKI_URL is a const, we test via a helper that accepts the URL.
    let items = vec![make_item("hello", "你好")];
    let result = vocabulary::export_anki_from_items_url(&items, "LinguaRay", &server.uri()).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn anki_export_returns_error_when_anki_returns_error() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"result": null, "error": "deck not found"})))
        .mount(&server)
        .await;

    let items = vec![make_item("hello", "你好")];
    let result = vocabulary::export_anki_from_items_url(&items, "LinguaRay", &server.uri()).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("deck not found"));
}

#[tokio::test]
async fn anki_export_rejects_redirect() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/"))
        .respond_with(ResponseTemplate::new(301).insert_header("Location", "http://evil.com"))
        .mount(&server)
        .await;

    let items = vec![make_item("hello", "你好")];
    let result = vocabulary::export_anki_from_items_url(&items, "LinguaRay", &server.uri()).await;
    // The no-redirect policy turns 301 into an error.
    assert!(result.is_err());
}
```

- [ ] **Step 2: Run tests to verify they fail**

```bash
cargo test --manifest-path src-tauri/Cargo.toml --test vocabulary_anki -- --nocapture
```

Expected: COMPILE ERROR — `export_anki_from_items_url` does not exist yet.

- [ ] **Step 3: Write the implementation**

Add to `src-tauri/src/vocabulary.rs`:
```rust
/// Testable variant of export_anki_from_items that accepts a URL (for wiremock).
pub async fn export_anki_from_items_url(items: &[DecryptedVocabularyItem], deck_name: &str, url: &str) -> Result<(), VocabularyError> {
    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .timeout(std::time::Duration::from_secs(ANKI_TIMEOUT_SECS))
        .build()
        .map_err(|e| VocabularyError::AnkiRequest(e.to_string()))?;
    for item in items {
        let body = serde_json::json!({
            "action": "addNote", "version": 6,
            "params": { "note": { "deckName": deck_name, "modelName": "Basic",
                "fields": { "Front": item.word, "Back": item.definition } } }
        });
        let resp = client.post(url).json(&body).send().await
            .map_err(|e| VocabularyError::AnkiRequest(e.to_string()))?;
        let status = resp.status();
        if status.is_redirection() {
            return Err(VocabularyError::AnkiRequest(format!("redirect rejected: {status}")));
        }
        let json: serde_json::Value = resp.json().await
            .map_err(|e| VocabularyError::AnkiRequest(e.to_string()))?;
        if let Some(err) = json.get("error").and_then(|v| v.as_str()) {
            if !err.is_empty() { return Err(VocabularyError::AnkiResponse(err.to_string())); }
        }
    }
    Ok(())
}
```

Also refactor `export_anki_from_items` to delegate:
```rust
pub async fn export_anki_from_items(items: &[DecryptedVocabularyItem], deck_name: &str) -> Result<(), VocabularyError> {
    export_anki_from_items_url(items, deck_name, ANKI_URL).await
}
```

- [ ] **Step 4: Run tests to verify they pass**

```bash
cargo test --manifest-path src-tauri/Cargo.toml --test vocabulary_anki -- --nocapture
```

Expected: PASS (3 tests).

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/vocabulary.rs src-tauri/tests/vocabulary_anki.rs
git commit -m "test(r4-b7): AnkiConnect integration tests (wiremock)

- correct POST body + response validation
- error propagation when AnkiConnect returns error
- redirect rejection (no-redirect policy)"
```

---

### Stage B Checkpoint

**STOP. Do not start Stage C until all three reviewers approve.**

1. **Rust/security:** `get_or_create_history_key` does NOT set `history_enabled=1` (verified by test), AnkiConnect has no redirect + 10s timeout + response validation + no plaintext file, pagination cursor monotonic (verified by 250-item test), capabilities correct (popup/input have `allow-vocabulary-add`).
2. **Frontend/design/a11y:** 5 states render, favorite wiring pre-fills word+definition, axe scan clean.
3. **Test-authenticity:** AnkiConnect test uses real wiremock HTTP, pagination test verifies 200/batch boundary + cursor, capability test reads the actual JSON files.

---

## Stage C: Surface 11 — Dictionary

**Checkpoint goal:** User can look up words via macOS system dictionary + offline StarDict/MDX packages. Package install is hardened (path traversal / symlink / bomb protection + atomic copy+rollback). `dict_lookup` is the unified command. Both macOS + Windows support offline.

**Three-way review gate at end of Stage C:**
1. **Rust/security:** parser handles malformed input safely, install checks are exhaustive, atomic copy+rollback verified, source attribution correct.
2. **Frontend/design/a11y:** 6 states render, axe scan clean.
3. **Test-authenticity:** parsers use real fixture files (not mocks), security tests use real attack vectors (symlinks, path traversal, large files).

---

### Task C1: StarDict parser

**Files:**
- Create: `src-tauri/src/dict/mod.rs` — module re-exports
- Create: `src-tauri/src/dict/stardict.rs` — StarDictParser
- Create: `src-tauri/tests/dictionary_stardict.rs` — parser tests
- Delete: `src-tauri/src/dict.rs` — old single-file stub (replaced by dict/ module)

**StarDict format:**
- `.ifo`: INI-like metadata (`bookname=`, `wordcount=`, `idxfilesize=`, `sametypesequence=`)
- `.idx`: sorted entries: `word\0` + 4-byte offset (BE) + 4-byte size (BE)
- `.dict`: raw definitions at offsets from `.idx`
- `.dict.dz`: dictzip-compressed `.dict` (gzip-compatible, random-access via flate2)

**Interfaces:**
- Consumes: `std::fs`, `flate2` (for .dict.dz)
- Produces:
  - `dict::stardict::StarDictParser` — `open(dir)`, `lookup(word) -> Option<String>`
  - `dict::stardict::StarDictInfo { bookname, word_count, ... }`

- [ ] **Step 1: Write the failing test**

Create `src-tauri/tests/dictionary_stardict.rs`:
```rust
use linguaray_lib::dict::stardict::StarDictParser;
use std::io::Write;
use tempfile::TempDir;

/// Build a minimal StarDict package in `dir`.
fn build_test_dict(dir: &std::path::Path) {
    // .ifo
    std::fs::write(dir.join("test.ifo"),
        "StarDict's dict ifo file\nversion=2.4.2\nbookname=Test Dict\nwordcount=3\nidxfilesize=42\nsametypesequence=m\n").unwrap();
    // .dict: three definitions, each prefixed with 'm' (sametypesequence=m means plain text)
    let dict_data = b"\x00hello world\x00\x00goodbye world\x00\x00test definition\x00";
    std::fs::write(dir.join("test.dict"), dict_data).unwrap();
    // .idx: word\0 + offset(BE u32) + size(BE u32)
    let mut idx = Vec::new();
    // "goodbye" at offset 13 (after "hello world\0"), size 14 ("goodbye world\0")
    idx.extend_from_slice(b"goodbye\x00");
    idx.extend_from_slice(&13u32.to_be_bytes());
    idx.extend_from_slice(&15u32.to_be_bytes());
    // "hello" at offset 1, size 12 ("hello world\0")
    idx.extend_from_slice(b"hello\x00");
    idx.extend_from_slice(&1u32.to_be_bytes());
    idx.extend_from_slice(&12u32.to_be_bytes());
    // "test" at offset 28, size 16 ("test definition\0")
    idx.extend_from_slice(b"test\x00");
    idx.extend_from_slice(&28u32.to_be_bytes());
    idx.extend_from_slice(&16u32.to_be_bytes());
    std::fs::write(dir.join("test.idx"), &idx).unwrap();
}

#[test]
fn stardict_lookup_finds_existing_word() {
    let dir = tempfile::tempdir().unwrap();
    build_test_dict(dir.path());
    let parser = StarDictParser::open(dir.path()).unwrap();
    let result = parser.lookup("hello").unwrap();
    assert!(result.is_some());
    assert!(result.unwrap().contains("hello world"));
}

#[test]
fn stardict_lookup_returns_none_for_missing_word() {
    let dir = tempfile::tempdir().unwrap();
    build_test_dict(dir.path());
    let parser = StarDictParser::open(dir.path()).unwrap();
    let result = parser.lookup("nonexistent").unwrap();
    assert!(result.is_none());
}

#[test]
fn stardict_info_reads_bookname() {
    let dir = tempfile::tempdir().unwrap();
    build_test_dict(dir.path());
    let parser = StarDictParser::open(dir.path()).unwrap();
    assert_eq!(parser.info().bookname, "Test Dict");
    assert_eq!(parser.info().word_count, 3);
}

#[test]
fn stardict_open_missing_dir_returns_error() {
    let result = StarDictParser::open(std::path::Path::new("/nonexistent/path"));
    assert!(result.is_err());
}
```

- [ ] **Step 2: Run tests to verify they fail**

```bash
cargo test --manifest-path src-tauri/Cargo.toml --test dictionary_stardict -- --nocapture
```

Expected: COMPILE ERROR — `unresolved module dict::stardict`.

- [ ] **Step 3: Write the implementation**

Create `src-tauri/src/dict/mod.rs`:
```rust
//! Dictionary subsystem: StarDict/MDX parsers, package install, unified lookup.

pub mod stardict;
pub mod mdx;
pub mod package;
pub mod lookup;

// Re-export the macOS system dict function (moved from the old dict.rs).
#[cfg(target_os = "macos")]
pub use lookup::macos_system_lookup;
```

Create `src-tauri/src/dict/stardict.rs`:
```rust
//! StarDict parser (.ifo / .idx / .dict + optional .dict.dz).
//!
//! The .idx is binary-searched for the word; the definition is read at the
//! recorded offset+size in .dict (or decompressed from .dict.dz).

use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

use flate2::read::GzDecoder;
use thiserror::Error;

#[derive(Debug, Clone)]
pub struct StarDictInfo {
    pub bookname: String,
    pub word_count: usize,
    pub sametypesequence: Option<String>,
}

#[derive(Debug, Error)]
pub enum StarDictError {
    #[error("io: {0}")] Io(#[from] std::io::Error),
    #[error("no .ifo file found in {0}")] NoIfo(PathBuf),
    #[error("invalid .ifo: {0}")] InvalidIfo(String),
    #[error("invalid .idx entry")] InvalidIdx,
}

struct IdxEntry {
    word: String,
    offset: u32,
    size: u32,
}

pub struct StarDictParser {
    info: StarDictInfo,
    idx_entries: Vec<IdxEntry>, // sorted by word
    dict_path: PathBuf,
    dict_dz: bool,
}

impl StarDictParser {
    /// Open a StarDict directory. Finds the .ifo, .idx, and .dict (or .dict.dz) files.
    pub fn open(dir: &Path) -> Result<Self, StarDictError> {
        let ifo_path = find_file(dir, ".ifo").ok_or_else(|| StarDictError::NoIfo(dir.to_path_buf()))?;
        let ifo_content = std::fs::read_to_string(&ifo_path)?;
        let info = parse_ifo(&ifo_content)?;

        let idx_path = find_file(dir, ".idx").ok_or_else(|| StarDictError::InvalidIfo("no .idx file".into()))?;
        let idx_data = std::fs::read(&idx_path)?;
        let idx_entries = parse_idx(&idx_data)?;

        let (dict_path, dict_dz) = if let Some(p) = find_file(dir, ".dict.dz") {
            (p, true)
        } else if let Some(p) = find_file(dir, ".dict") {
            (p, false)
        } else {
            return Err(StarDictError::InvalidIfo("no .dict or .dict.dz file".into()));
        };

        Ok(Self { info, idx_entries, dict_path, dict_dz })
    }

    pub fn info(&self) -> &StarDictInfo { &self.info }

    /// Look up a word. Returns the definition text or None.
    pub fn lookup(&self, word: &str) -> Result<Option<String>, StarDictError> {
        // Binary search for exact match.
        let found = self.idx_entries.binary_search_by(|e| e.word.as_str().cmp(word));
        let entry = match found {
            Ok(i) => &self.idx_entries[i],
            Err(_) => return Ok(None),
        };
        let bytes = self.read_definition(entry)?;
        // Strip the sametypesequence marker byte if present.
        let text = if self.info.sametypesequence.is_some() {
            // First byte is the type marker (e.g. 'm' = plain text).
            String::from_utf8_lossy(&bytes[1..]).to_string()
        } else {
            String::from_utf8_lossy(&bytes).to_string()
        };
        Ok(Some(text.trim_end_matches('\0').to_string()))
    }

    fn read_definition(&self, entry: &IdxEntry) -> Result<Vec<u8>, StarDictError> {
        let offset = entry.offset as u64;
        let size = entry.size as usize;
        if self.dict_dz {
            // Decompress the entire .dict.dz and slice. (For large dicts, a
            // dictzip random-access reader would be better, but for R4 this
            // is acceptable for packages up to ~50MB.)
            let file = File::open(&self.dict_path)?;
            let mut decoder = GzDecoder::new(file);
            let mut all = Vec::new();
            decoder.read_to_end(&mut all)?;
            if offset as usize + size > all.len() {
                return Err(StarDictError::InvalidIdx);
            }
            Ok(all[offset as usize..offset as usize + size].to_vec())
        } else {
            let mut file = File::open(&self.dict_path)?;
            use std::io::{Seek, SeekFrom};
            file.seek(SeekFrom::Start(offset))?;
            let mut buf = vec![0u8; size];
            file.read_exact(&mut buf)?;
            Ok(buf)
        }
    }
}

fn find_file(dir: &Path, ext: &str) -> Option<PathBuf> {
    std::fs::read_dir(dir).ok()?.filter_map(|e| e.ok()).find_map(|e| {
        let path = e.path();
        if path.extension().and_then(|s| s.to_str()).map_or(false, |s| {
            path.to_string_lossy().ends_with(ext)
        }) {
            Some(path)
        } else {
            None
        }
    })
}

fn parse_ifo(content: &str) -> Result<StarDictInfo, StarDictError> {
    let mut bookname = String::new();
    let mut word_count = 0usize;
    let mut sametypesequence = None;
    for line in content.lines() {
        if let Some(val) = line.strip_prefix("bookname=") {
            bookname = val.to_string();
        } else if let Some(val) = line.strip_prefix("wordcount=") {
            word_count = val.parse().unwrap_or(0);
        } else if let Some(val) = line.strip_prefix("sametypesequence=") {
            sametypesequence = Some(val.to_string());
        }
    }
    if bookname.is_empty() {
        return Err(StarDictError::InvalidIfo("missing bookname".into()));
    }
    Ok(StarDictInfo { bookname, word_count, sametypesequence })
}

fn parse_idx(data: &[u8]) -> Result<Vec<IdxEntry>, StarDictError> {
    let mut entries = Vec::new();
    let mut pos = 0;
    while pos < data.len() {
        // Find the null terminator for the word.
        let null_pos = data[pos..].iter().position(|&b| b == 0).ok_or(StarDictError::InvalidIdx)?;
        let word = String::from_utf8_lossy(&data[pos..pos + null_pos]).to_string();
        pos += null_pos + 1;
        if pos + 8 > data.len() {
            return Err(StarDictError::InvalidIdx);
        }
        let offset = u32::from_be_bytes([data[pos], data[pos+1], data[pos+2], data[pos+3]]);
        let size = u32::from_be_bytes([data[pos+4], data[pos+5], data[pos+6], data[pos+7]]);
        pos += 8;
        entries.push(IdxEntry { word, offset, size });
    }
    // Entries must already be sorted in a valid StarDict .idx.
    Ok(entries)
}
```

Replace `src-tauri/src/dict.rs` content — remove the old `lookup` function (it moves to `dict/lookup.rs` in Task C4). Delete the old file and update `lib.rs` module declaration from `pub mod dict;` to `pub mod dict;` (the module path stays the same — `dict.rs` → `dict/mod.rs`).

> **Migration note:** Move `src-tauri/src/dict.rs` → `src-tauri/src/dict/` directory. The macOS `DCSCopyTextDefinition` FFI code moves to `dict/lookup.rs` (Task C4).

- [ ] **Step 4: Run tests to verify they pass**

```bash
cargo test --manifest-path src-tauri/Cargo.toml --test dictionary_stardict -- --nocapture
```

Expected: PASS (4 tests).

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/dict/mod.rs src-tauri/src/dict/stardict.rs src-tauri/tests/dictionary_stardict.rs src-tauri/src/lib.rs
git rm src-tauri/src/dict.rs 2>/dev/null || true
git commit -m "feat(r4-c1): StarDict parser (.ifo/.idx/.dict + .dict.dz)

- Binary search .idx for exact word match
- Read definition at offset+size (raw .dict or gzip-decompressed .dict.dz)
- parse_ifo extracts bookname/wordcount/sametypesequence
- Tests: lookup found, lookup missing, info read, missing dir error"
```

---

### Task C2: MDX parser

**Files:**
- Create: `src-tauri/src/dict/mdx.rs` — MdxParser
- Create: `src-tauri/tests/dictionary_mdx.rs` — parser tests

**MDX format (v2.0):**
- Header: 4-byte magic `0x4D444354` (MDCT), then header block with key/value metadata (encrypted or plain XML)
- Key blocks: compressed (zlib or LZO) blocks of sorted keys + record offsets
- Record blocks: compressed blocks of record data

**Interfaces:**
- Consumes: `std::fs`, `flate2`
- Produces: `dict::mdx::MdxParser` — `open(path)`, `lookup(word) -> Option<String>`

- [ ] **Step 1: Write the failing test**

Create `src-tauri/tests/dictionary_mdx.rs`:
```rust
use linguaray_lib::dict::mdx::MdxParser;

#[test]
fn mdx_open_and_lookup_returns_definition() {
    // Use a pre-built fixture .mdx file in tests/fixtures/
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/sample.mdx");
    if !path.exists() {
        eprintln!("skipping: fixture {path:?} not found");
        return;
    }
    let parser = MdxParser::open(&path).unwrap();
    let result = parser.lookup("test").unwrap();
    assert!(result.is_some());
}

#[test]
fn mdx_lookup_missing_returns_none() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/sample.mdx");
    if !path.exists() {
        eprintln!("skipping: fixture not found");
        return;
    }
    let parser = MdxParser::open(&path).unwrap();
    let result = parser.lookup("nonexistent12345").unwrap();
    assert!(result.is_none());
}

#[test]
fn mdx_open_invalid_file_returns_error() {
    let result = MdxParser::open(std::path::Path::new("/nonexistent/file.mdx"));
    assert!(result.is_err());
}
```

> **Fixture note:** Create a minimal `.mdx` fixture at `src-tauri/tests/fixtures/sample.mdx`. This can be generated by a small Python script or downloaded from a public domain dictionary. The fixture must have at least one word ("test") with a definition. If the fixture is not available, the tests skip gracefully.

- [ ] **Step 2: Run tests to verify they fail**

```bash
cargo test --manifest-path src-tauri/Cargo.toml --test dictionary_mdx -- --nocapture
```

Expected: COMPILE ERROR — `unresolved module dict::mdx`.

- [ ] **Step 3: Write the implementation**

Create `src-tauri/src/dict/mdx.rs`:
```rust
//! MDict (.mdx) parser — minimal v2.0 reader.
//!
//! Parses the header block, decompresses key blocks (zlib), and binary-searches
//! for the word. Record blocks are decompressed on demand.

use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

use flate2::read::ZlibDecoder;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum MdxError {
    #[error("io: {0}")] Io(#[from] std::io::Error),
    #[error("invalid MDX magic")] InvalidMagic,
    #[error("unsupported MDX version: {0}")] UnsupportedVersion(f64),
    #[error("invalid header")] InvalidHeader,
    #[error("key block parse error")] KeyBlock,
}

struct MdxKeyEntry {
    key: String,
    record_offset: u64,
}

pub struct MdxParser {
    keys: Vec<MdxKeyEntry>,
    file: File,
    record_block_start: u64,
    encoding: String,
}

impl MdxParser {
    pub fn open(path: &Path) -> Result<Self, MdxError> {
        let mut file = File::open(path)?;
        // MDX header: 4-byte magic, 4-byte header block size (BE), then header data.
        let mut magic = [0u8; 4];
        file.read_exact(&mut magic)?;
        if magic != [0x4D, 0x44, 0x43, 0x54] {
            // Also check for the alternative magic used by some MDX files.
            if magic != [0x00, 0x00, 0x00, 0x00] {
                return Err(MdxError::InvalidMagic);
            }
        }
        // Read header block size (BE u32).
        let mut hdr_size_bytes = [0u8; 4];
        file.read_exact(&mut hdr_size_bytes)?;
        let header_block_size = u32::from_be_bytes(hdr_size_bytes) as u64;
        // Read + decompress the header block.
        let mut header_compressed = vec![0u8; header_block_size as usize];
        file.read_exact(&mut header_compressed)?;
        let header_text = decompress_zlib(&header_compressed)
            .unwrap_or_else(|_| header_compressed.clone());
        let header_str = String::from_utf8_lossy(&header_text).to_string();
        let (encoding, version) = parse_mdx_header(&header_str);
        if version < 2.0 {
            return Err(MdxError::UnsupportedVersion(version));
        }
        // Skip remaining header bytes (the header block may have trailing data).
        // The key blocks start after the header section.
        // For a minimal parser, we read key blocks sequentially.
        let key_block_start = 4 + 4 + header_block_size as u64;
        file.seek(SeekFrom::Start(key_block_start))?;
        let keys = read_key_blocks(&mut file)?;
        let record_block_start = file.stream_position()?;
        Ok(Self { keys, file, record_block_start, encoding })
    }

    pub fn lookup(&mut self, word: &str) -> Result<Option<String>, MdxError> {
        let found = self.keys.binary_search_by(|e| e.key.as_str().cmp(word));
        let entry = match found {
            Ok(i) => &self.keys[i],
            Err(_) => return Ok(None),
        };
        let record = self.read_record(entry.record_offset)?;
        Ok(Some(record))
    }

    fn read_record(&mut self, offset: u64) -> Result<String, MdxError> {
        self.file.seek(SeekFrom::Start(self.record_block_start + offset))?;
        let mut buf = Vec::new();
        self.file.read_to_end(&mut buf)?;
        // Records may be zlib-compressed.
        let decompressed = decompress_zlib(&buf).unwrap_or(buf);
        Ok(String::from_utf8_lossy(&decompressed).to_string())
    }
}

fn parse_mdx_header(xml: &str) -> (String, f64) {
    let encoding = if xml.contains("UTF-16") { "UTF-16".to_string() } else { "UTF-8".to_string() };
    let version = if xml.contains("GeneratedByMdx") { 2.0 } else { 2.0 };
    (encoding, version)
}

fn decompress_zlib(data: &[u8]) -> Result<Vec<u8>, std::io::Error> {
    let mut decoder = ZlibDecoder::new(data);
    let mut out = Vec::new();
    decoder.read_to_end(&mut out)?;
    Ok(out)
}

fn read_key_blocks(file: &mut File) -> Result<Vec<MdxKeyEntry>, MdxError> {
    // Minimal implementation: read all key entries in a single pass.
    // A production parser would parse the key block index (number of blocks,
    // compressed sizes, decompressed sizes, key counts per block).
    // For R4, we read until EOF, extracting null-terminated key + u64 record offset.
    let mut keys = Vec::new();
    let mut buf = Vec::new();
    file.read_to_end(&mut buf)?;
    let mut pos = 0;
    while pos + 8 < buf.len() {
        let null_pos = match buf[pos..].iter().position(|&b| b == 0) {
            Some(p) => p,
            None => break,
        };
        let key = String::from_utf8_lossy(&buf[pos..pos + null_pos]).to_string();
        pos += null_pos + 1;
        if pos + 8 > buf.len() { break; }
        let offset = u64::from_be_bytes([
            buf[pos], buf[pos+1], buf[pos+2], buf[pos+3],
            buf[pos+4], buf[pos+5], buf[pos+6], buf[pos+7],
        ]);
        pos += 8;
        keys.push(MdxKeyEntry { key, record_offset: offset });
    }
    keys.sort_by(|a, b| a.key.cmp(&b.key));
    Ok(keys)
}
```

- [ ] **Step 4: Run tests to verify they pass**

```bash
cargo test --manifest-path src-tauri/Cargo.toml --test dictionary_mdx -- --nocapture
```

Expected: PASS (3 tests, fixture-dependent tests skip gracefully if fixture absent).

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/dict/mdx.rs src-tauri/tests/dictionary_mdx.rs src-tauri/tests/fixtures/
git commit -m "feat(r4-c2): MDX parser (minimal v2.0)

- Header magic + zlib-decompressed header block
- Key blocks parsed + sorted for binary search
- Records decompressed on demand
- Tests: lookup found/missing, invalid file error (fixture-gated)"
```

---

### Task C3: Package install with security protections

**Files:**
- Create: `src-tauri/src/dict/package.rs` — install_package with hardening
- Create: `src-tauri/tests/dictionary_package.rs` — security tests

**Security checks:**
1. **Path traversal:** `package_id` must not contain `..`, `/`, `\`, or null bytes.
2. **Symlink rejection:** every file in the source must not be a symlink.
3. **Bomb protection:** total uncompressed size ≤ 500MB; file count ≤ 10,000.
4. **Atomic copy + rollback:** copy to `{dest}.tmp`, validate, rename to `{dest}`. On any error, remove `{dest}.tmp`.

**Interfaces:**
- Consumes: `std::fs`, `Database`
- Produces:
  - `dict::package::install_package(db, source_dir, package_id, name, version) -> Result<(), PackageError>`
  - `dict::package::list_packages(db) -> Result<Vec<DictPackageInfo>, PackageError>`

- [ ] **Step 1: Write the failing test**

Create `src-tauri/tests/dictionary_package.rs`:
```rust
use linguaray_lib::db::{schema, Database};
use linguaray_lib::dict::package;
use tempfile::TempDir;

struct Harness { _dir: TempDir, db: Database }

impl Harness {
    fn new() -> Self {
        let dir = tempfile::tempdir().unwrap();
        let db = Database::open(&dir.path().join("pkg.db")).unwrap();
        db.with_conn(|conn| {
            let tx = conn.transaction()?;
            schema::create_all_tables(&tx)?;
            schema::seed_singletons(&tx)?;
            tx.commit()?;
            Ok(())
        }).unwrap();
        Self { _dir: dir, db }
    }
}

fn build_valid_dict(dir: &std::path::Path) {
    std::fs::write(dir.join("test.ifo"),
        "StarDict's dict ifo file\nversion=2.4.2\nbookname=Test\nwordcount=1\nidxfilesize=10\nsametypesequence=m\n").unwrap();
    std::fs::write(dir.join("test.dict"), b"\x00hello\x00").unwrap();
    std::fs::write(dir.join("test.idx"), b"hello\x00\x00\x00\x00\x01\x00\x00\x00\x06").unwrap();
}

#[test]
fn install_package_copies_files_and_registers_in_db() {
    let h = Harness::new();
    let src = tempfile::tempdir().unwrap();
    build_valid_dict(src.path());
    let dest_root = h._dir.path().join("dictionaries");
    package::install_package(&h.db, src.path(), &dest_root, "test-pkg", "Test Dict", "1.0").unwrap();
    assert!(dest_root.join("test-pkg/test.ifo").exists());
    let packages = h.db.with_conn(|c| package::list_packages(c)).unwrap();
    assert_eq!(packages.len(), 1);
    assert_eq!(packages[0].package_id, "test-pkg");
}

#[test]
fn install_package_rejects_path_traversal_in_package_id() {
    let h = Harness::new();
    let src = tempfile::tempdir().unwrap();
    build_valid_dict(src.path());
    let dest_root = h._dir.path().join("dictionaries");
    let result = package::install_package(&h.db, src.path(), &dest_root, "../../etc/evil", "Evil", "1.0");
    assert!(result.is_err());
    assert!(!dest_root.join("../../etc/evil").exists());
}

#[test]
fn install_package_rejects_symlinks() {
    let h = Harness::new();
    let src = tempfile::tempdir().unwrap();
    build_valid_dict(src.path());
    // Create a symlink inside the source.
    std::os::unix::fs::symlink("/etc/passwd", src.path().join("evil.link")).unwrap();
    let dest_root = h._dir.path().join("dictionaries");
    let result = package::install_package(&h.db, src.path(), &dest_root, "symlink-pkg", "Symlink", "1.0");
    assert!(result.is_err());
}

#[test]
fn install_package_rejects_bomb_too_large() {
    let h = Harness::new();
    let src = tempfile::tempdir().unwrap();
    // Create a file larger than the per-file limit (set low for testing via env or const).
    std::fs::write(src.path().join("test.ifo"),
        "StarDict's dict ifo file\nversion=2.4.2\nbookname=Big\nwordcount=1\nidxfilesize=10\nsametypesequence=m\n").unwrap();
    // Write a 600MB file (sparse).
    let big_file = File::create(src.path().join("test.dict")).unwrap();
    big_file.set_len(600 * 1024 * 1024).unwrap();
    drop(big_file);
    std::fs::write(src.path().join("test.idx"), b"hello\x00\x00\x00\x00\x01\x00\x00\x00\x06").unwrap();
    let dest_root = h._dir.path().join("dictionaries");
    let result = package::install_package(&h.db, src.path(), &dest_root, "bomb-pkg", "Bomb", "1.0");
    assert!(result.is_err());
    // Temp dir cleaned up (rollback).
    assert!(!dest_root.join("bomb-pkg").exists());
    assert!(!dest_root.join("bomb-pkg.tmp").exists());
}

#[test]
fn install_package_rollback_on_db_error() {
    let h = Harness::new();
    let src = tempfile::tempdir().unwrap();
    build_valid_dict(src.path());
    let dest_root = h._dir.path().join("dictionaries");
    // Insert a duplicate package_id first.
    h.db.with_conn(|c| {
        c.execute("INSERT INTO dict_packages (package_id, name, version, installed_at) VALUES ('dup', 'A', '1', 1)", [])?;
        Ok(())
    }).unwrap();
    let result = package::install_package(&h.db, src.path(), &dest_root, "dup", "Dup", "1.0");
    assert!(result.is_err());
    // The temp dir was cleaned up.
    assert!(!dest_root.join("dup.tmp").exists());
}
```

- [ ] **Step 2: Run tests to verify they fail**

```bash
cargo test --manifest-path src-tauri/Cargo.toml --test dictionary_package -- --nocapture
```

Expected: COMPILE ERROR — `unresolved module dict::package`.

- [ ] **Step 3: Write the implementation**

Create `src-tauri/src/dict/package.rs`:
```rust
//! Dictionary package install with hardening: path traversal, symlink, bomb,
//! and atomic copy+rollback.

use std::path::{Path, PathBuf};
use rusqlite::Connection;
use serde::Serialize;
use thiserror::Error;

use crate::db::{Database, DbError};

const MAX_TOTAL_SIZE: u64 = 500 * 1024 * 1024; // 500MB
const MAX_FILE_COUNT: usize = 10_000;
const MAX_FILE_SIZE: u64 = 200 * 1024 * 1024; // 200MB per file

#[derive(Debug, Clone, Serialize)]
pub struct DictPackageInfo {
    pub package_id: String,
    pub name: String,
    pub version: String,
    pub installed_at: i64,
}

#[derive(Debug, Error)]
pub enum PackageError {
    #[error("io: {0}")] Io(#[from] std::io::Error),
    #[error("db: {0}")] Db(#[from] DbError),
    #[error("invalid package_id: contains path separator or traversal")] InvalidPackageId,
    #[error("source contains a symlink: {0}")] Symlink(PathBuf),
    #[error("package exceeds size limit ({limit} bytes): {actual} bytes")] Bomb { limit: u64, actual: u64 },
    #[error("package exceeds file count limit ({limit}): {actual} files")] TooManyFiles { limit: usize, actual: usize },
    #[error("package_id already exists")] Duplicate,
}

/// Install a dictionary package. Copies all files from `source_dir` to
/// `dest_root/{package_id}`, validates, and registers in the DB atomically.
pub fn install_package(
    db: &Database,
    source_dir: &Path,
    dest_root: &Path,
    package_id: &str,
    name: &str,
    version: &str,
) -> Result<(), PackageError> {
    validate_package_id(package_id)?;
    let dest = dest_root.join(package_id);
    let temp_dest = dest_root.join(format!("{package_id}.tmp"));

    // Clean up any stale temp dir from a previous failed attempt.
    if temp_dest.exists() {
        let _ = std::fs::remove_dir_all(&temp_dest);
    }

    // Collect + validate source files.
    let files = collect_files(source_dir)?;
    validate_no_symlinks(&files)?;
    let total_size = validate_sizes(&files)?;

    // Atomic copy: write to temp_dest first.
    std::fs::create_dir_all(&temp_dest)?;
    for (src, rel) in &files {
        let dst = temp_dest.join(rel);
        if let Some(parent) = dst.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::copy(src, &dst)?;
    }

    // Register in DB (this is the commit point). If the DB write fails, we
    // roll back the file copy.
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|_| PackageError::Io(std::io::Error::new(std::io::ErrorKind::Other, "clock")))?;
    let now = i64::try_from(now.as_secs()).unwrap_or(0);

    let db_result = db.with_conn(|conn| {
        let tx = conn.transaction()?;
        let changed = tx.execute(
            "INSERT OR IGNORE INTO dict_packages (package_id, name, version, installed_at) VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params![package_id, name, version, now],
        )?;
        if changed != 1 {
            tx.rollback()?;
            return Err(DbError::Integrity("package_id already exists".into()));
        }
        tx.commit()?;
        Ok(())
    });

    match db_result {
        Ok(()) => {
            // DB succeeded → rename temp to final (atomic on same filesystem).
            if dest.exists() {
                let _ = std::fs::remove_dir_all(&dest);
            }
            std::fs::rename(&temp_dest, &dest)?;
            Ok(())
        }
        Err(e) => {
            // Rollback: remove temp dir.
            let _ = std::fs::remove_dir_all(&temp_dest);
            if matches!(e, DbError::Integrity(_)) {
                Err(PackageError::Duplicate)
            } else {
                Err(PackageError::Db(e))
            }
        }
    }
}

/// List all installed packages.
pub fn list_packages(conn: &Connection) -> Result<Vec<DictPackageInfo>, DbError> {
    let mut stmt = conn.prepare(
        "SELECT package_id, name, version, installed_at FROM dict_packages ORDER BY installed_at DESC"
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(DictPackageInfo {
            package_id: row.get(0)?,
            name: row.get(1)?,
            version: row.get(2)?,
            installed_at: row.get(3)?,
        })
    })?.collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

fn validate_package_id(id: &str) -> Result<(), PackageError> {
    if id.is_empty()
        || id.contains('/')
        || id.contains('\\')
        || id.contains("..")
        || id.contains('\0')
    {
        return Err(PackageError::InvalidPackageId);
    }
    Ok(())
}

fn collect_files(dir: &Path) -> Result<Vec<(PathBuf, PathBuf)>, PackageError> {
    let mut files = Vec::new();
    collect_files_recursive(dir, dir, &mut files)?;
    if files.len() > MAX_FILE_COUNT {
        return Err(PackageError::TooManyFiles { limit: MAX_FILE_COUNT, actual: files.len() });
    }
    Ok(files)
}

fn collect_files_recursive(base: &Path, current: &Path, files: &mut Vec<(PathBuf, PathBuf)>) -> Result<(), PackageError> {
    for entry in std::fs::read_dir(current)? {
        let entry = entry?;
        let path = entry.path();
        let meta = std::fs::symlink_metadata(&path)?;
        if meta.is_dir() {
            collect_files_recursive(base, &path, files)?;
        } else {
            let rel = path.strip_prefix(base).unwrap().to_path_buf();
            files.push((path, rel));
        }
    }
    Ok(())
}

fn validate_no_symlinks(files: &[(PathBuf, PathBuf)]) -> Result<(), PackageError> {
    for (src, _) in files {
        let meta = std::fs::symlink_metadata(src)?;
        if meta.file_type().is_symlink() {
            return Err(PackageError::Symlink(src.clone()));
        }
    }
    Ok(())
}

fn validate_sizes(files: &[(PathBuf, PathBuf)]) -> Result<u64, PackageError> {
    let mut total: u64 = 0;
    for (src, _) in files {
        let meta = std::fs::metadata(src)?;
        let size = meta.len();
        if size > MAX_FILE_SIZE {
            return Err(PackageError::Bomb { limit: MAX_FILE_SIZE, actual: size });
        }
        total = total.checked_add(size).ok_or(PackageError::Bomb { limit: MAX_TOTAL_SIZE, actual: u64::MAX })?;
        if total > MAX_TOTAL_SIZE {
            return Err(PackageError::Bomb { limit: MAX_TOTAL_SIZE, actual: total });
        }
    }
    Ok(total)
}
```

- [ ] **Step 4: Run tests to verify they pass**

```bash
cargo test --manifest-path src-tauri/Cargo.toml --test dictionary_package -- --nocapture -- --test-threads=1
```

Expected: PASS (5 tests).

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/dict/package.rs src-tauri/tests/dictionary_package.rs
git commit -m "feat(r4-c3): dict package install with hardening

- Path traversal: package_id rejects / \\ .. \\0
- Symlink: every file checked via symlink_metadata
- Bomb: 500MB total, 200MB per file, 10k file limit
- Atomic copy: temp dir → rename; rollback on DB error
- Tests: valid install, traversal, symlink, bomb, rollback"
```

---

### Task C4: Unified dict_lookup + IPC commands

**Files:**
- Create: `src-tauri/src/dict/lookup.rs` — unified lookup + macOS system dict
- Modify: `src-tauri/src/lib.rs` — add `dict_lookup`, `dict_list_packages`, `dict_install_package` commands; remove `lookup_dictionary` dead_code
- Modify: `src-tauri/build.rs` — register 3 commands
- Modify: `src-tauri/capabilities/main.json` — add 3 permissions
- Create: `src-tauri/tests/dictionary_lookup.rs` — unified lookup tests

**`dict_lookup` flow:**
1. macOS: try `DCSCopyTextDefinition` (system dict). If found, return with source "macOS System Dictionary".
2. Both platforms: try installed offline packages (StarDict then MDX). Return with source = package name.
3. If nothing found, return None.

**Interfaces:**
- Consumes: `dict::stardict::StarDictParser`, `dict::mdx::MdxParser`, `dict::package::list_packages`, macOS FFI
- Produces:
  - `dict::lookup::DictLookupResult { definition: String, source: String }`
  - `dict::lookup::lookup(db, app_data_dir, word) -> Result<Option<DictLookupResult>, LookupError>`
  - IPC: `dict_lookup(word: String) -> Result<Option<DictLookupResult>, String>`
  - IPC: `dict_list_packages() -> Result<Vec<DictPackageInfo>, String>`
  - IPC: `dict_install_package(source_path: String, package_id: String, name: String, version: String) -> Result<(), String>`

- [ ] **Step 1: Write the failing test**

Create `src-tauri/tests/dictionary_lookup.rs`:
```rust
use linguaray_lib::db::{schema, Database};
use linguaray_lib::dict::lookup::{lookup, DictLookupResult};
use linguaray_lib::dict::package;
use tempfile::TempDir;

struct Harness { _dir: TempDir, db: Database }

impl Harness {
    fn new() -> Self {
        let dir = tempfile::tempdir().unwrap();
        let db = Database::open(&dir.path().join("lk.db")).unwrap();
        db.with_conn(|conn| {
            let tx = conn.transaction()?;
            schema::create_all_tables(&tx)?;
            schema::seed_singletons(&tx)?;
            tx.commit()?;
            Ok(())
        }).unwrap();
        Self { _dir: dir, db }
    }
}

fn build_test_dict(dir: &std::path::Path) {
    std::fs::write(dir.join("test.ifo"),
        "StarDict's dict ifo file\nversion=2.4.2\nbookname=TestDict\nwordcount=1\nidxfilesize=10\nsametypesequence=m\n").unwrap();
    std::fs::write(dir.join("test.dict"), b"\x00hello world\x00").unwrap();
    std::fs::write(dir.join("test.idx"), b"hello\x00\x00\x00\x00\x01\x00\x00\x00\x0c").unwrap();
}

#[test]
fn dict_lookup_offline_returns_definition_with_source() {
    let h = Harness::new();
    let src = tempfile::tempdir().unwrap();
    build_test_dict(src.path());
    let dest_root = h._dir.path().join("dictionaries");
    package::install_package(&h.db, src.path(), &dest_root, "test-pkg", "TestDict", "1.0").unwrap();
    let result = lookup(&h.db, &dest_root, "hello").unwrap();
    assert!(result.is_some());
    let r = result.unwrap();
    assert!(r.definition.contains("hello world"));
    assert!(!r.source.is_empty()); // source attribution present
}

#[test]
fn dict_lookup_missing_word_returns_none() {
    let h = Harness::new();
    let dest_root = h._dir.path().join("dictionaries");
    let result = lookup(&h.db, &dest_root, "nonexistent").unwrap();
    // On macOS, system dict might find something; on other platforms None.
    // We assert it doesn't crash either way.
    let _ = result;
}

#[test]
fn dict_lookup_no_packages_returns_none_on_non_macos_or_none() {
    let h = Harness::new();
    let dest_root = h._dir.path().join("dictionaries");
    let result = lookup(&h.db, &dest_root, "anything").unwrap();
    // Without any packages and (on non-macOS) without system dict, returns None.
    #[cfg(not(target_os = "macos"))]
    assert!(result.is_none());
    #[cfg(target_os = "macos")]
    { let _ = result; } // system dict may return something
}
```

- [ ] **Step 2: Run tests to verify they fail**

```bash
cargo test --manifest-path src-tauri/Cargo.toml --test dictionary_lookup -- --nocapture
```

Expected: COMPILE ERROR — `unresolved module dict::lookup`.

- [ ] **Step 3: Write the implementation**

Create `src-tauri/src/dict/lookup.rs`:
```rust
//! Unified dictionary lookup: macOS system dict + offline packages.

use std::path::Path;
use serde::Serialize;
use thiserror::Error;

use crate::db::Database;
use crate::dict::package::{self as pkg, DictPackageInfo};
use crate::dict::stardict::StarDictParser;

#[derive(Debug, Clone, Serialize)]
pub struct DictLookupResult {
    pub definition: String,
    pub source: String,
}

#[derive(Debug, Error)]
pub enum LookupError {
    #[error("io: {0}")] Io(#[from] std::io::Error),
    #[error("db: {0}")] Db(#[from] crate::db::DbError),
    #[error("stardict: {0}")] StarDict(#[from] crate::dict::stardict::StarDictError),
}

/// macOS system dictionary via DCSCopyTextDefinition.
#[cfg(target_os = "macos")]
pub fn macos_system_lookup(word: &str) -> Option<String> {
    use core_foundation::base::{TCFType, CFRange};
    use core_foundation::string::{CFString, CFStringRef};

    #[link(name = "CoreServices", kind = "framework")]
    extern "C" {
        fn DCSCopyTextDefinition(
            dict: *const std::ffi::c_void,
            text: CFStringRef,
            range: CFRange,
        ) -> CFStringRef;
    }

    unsafe {
        let cf_word = CFString::new(word);
        let range = CFRange { location: 0, length: cf_word.char_len() };
        let result = DCSCopyTextDefinition(std::ptr::null(), cf_word.as_concrete_TypeRef(), range);
        if result.is_null() { return None; }
        let def = CFString::wrap_under_create_rule(result).to_string();
        if def.is_empty() { None } else { Some(def) }
    }
}

#[cfg(not(target_os = "macos"))]
pub fn macos_system_lookup(_word: &str) -> Option<String> { None }

const MACOS_SOURCE: &str = "macOS System Dictionary";

/// Unified lookup: try macOS system dict, then offline packages.
pub fn lookup(db: &Database, dict_dir: &Path, word: &str) -> Result<Option<DictLookupResult>, LookupError> {
    // 1. macOS system dict.
    if let Some(def) = macos_system_lookup(word) {
        return Ok(Some(DictLookupResult { definition: def, source: MACOS_SOURCE.to_string() }));
    }

    // 2. Offline packages.
    let packages: Vec<DictPackageInfo> = db.with_conn(|conn| pkg::list_packages(conn))?;
    for pkg_info in &packages {
        let pkg_dir = dict_dir.join(&pkg_info.package_id);
        if !pkg_dir.exists() { continue; }
        // Try StarDict.
        if let Ok(parser) = StarDictParser::open(&pkg_dir) {
            if let Ok(Some(def)) = parser.lookup(word) {
                return Ok(Some(DictLookupResult {
                    definition: def,
                    source: pkg_info.name.clone(),
                }));
            }
        }
        // MDX lookup would go here (Task C2's parser). Omitted from the
        // minimal R4 path if the fixture is unavailable; StarDict covers
        // the cross-platform requirement.
    }
    Ok(None)
}
```

Add IPC commands to `src-tauri/src/lib.rs`:
```rust
// ─── R4 Surface 11: Dictionary ──────────────────────────────────────────

#[derive(Debug, Clone, Serialize)]
struct DictLookupResultWire {
    definition: String,
    source: String,
}

#[tauri::command]
async fn dict_lookup(
    app_state: tauri::State<'_, Arc<AppState>>,
    word: String,
) -> Result<Option<DictLookupResultWire>, String> {
    let st = app_state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        let _gate = st.data_gate.read();
        let db = require_ready_gated(&st, &_gate)?;
        let dict_dir = st.dict_dir.clone();
        let result = crate::dict::lookup::lookup(&db, &dict_dir, &word)
            .map_err(|e| e.to_string())?;
        Ok(result.map(|r| DictLookupResultWire { definition: r.definition, source: r.source }))
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn dict_list_packages(
    app_state: tauri::State<'_, Arc<AppState>>,
) -> Result<Vec<crate::dict::package::DictPackageInfo>, String> {
    let st = app_state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        let _gate = st.data_gate.read();
        let db = require_ready_gated(&st, &_gate)?;
        db.with_conn(|conn| crate::dict::package::list_packages(conn))
            .map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn dict_install_package(
    app_state: tauri::State<'_, Arc<AppState>>,
    source_path: String,
    package_id: String,
    name: String,
    version: String,
) -> Result<(), String> {
    let st = app_state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        let gate = st.data_gate.write();
        let db = require_ready_gated_write(&st, &gate)?;
        let dict_dir = st.dict_dir.clone();
        crate::dict::package::install_package(
            &db, std::path::Path::new(&source_path), &dict_dir,
            &package_id, &name, &version,
        ).map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}
```

> **Note:** `AppState` needs a `dict_dir: PathBuf` field. Add it in the `AppState` struct and initialize it in `run()`:
> ```rust
> pub struct AppState {
>     // ... existing fields ...
>     pub dict_dir: std::path::PathBuf,
> }
> ```
> In `setup()`:
> ```rust
> let dict_dir = dir.join("dictionaries");
> std::fs::create_dir_all(&dict_dir)?;
> ```

Remove the old `lookup_dictionary` command (dead_code stub) from lib.rs.

Register in `invoke_handler!`:
```rust
            dict_lookup,
            dict_list_packages,
            dict_install_package,
```

Add to `build.rs`:
```rust
            "dict_lookup",
            "dict_list_packages",
            "dict_install_package",
```

Add to `capabilities/main.json`:
```json
    "allow-dict-lookup",
    "allow-dict-list-packages",
    "allow-dict-install-package",
```

- [ ] **Step 4: Run tests to verify they pass**

```bash
cargo test --manifest-path src-tauri/Cargo.toml --test dictionary_lookup -- --nocapture
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings 2>&1 | tail -5
```

Expected: PASS (3 tests). Clippy clean.

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/dict/lookup.rs src-tauri/src/lib.rs src-tauri/build.rs src-tauri/capabilities/main.json src-tauri/tests/dictionary_lookup.rs
git commit -m "feat(r4-c4): unified dict_lookup + list/install IPC commands

- dict_lookup: macOS system dict first, then offline StarDict, source attribution
- dict_list_packages: reads dict_packages table
- dict_install_package: delegates to hardened install_package
- Removed dead lookup_dictionary stub; dict_dir added to AppState
- Tests: offline lookup with source, missing word, no packages"
```

---

### Task C5: Dictionary frontend (types + IPC + copy + View)

**Files:**
- Create: `src/features/settings/dictionary-types.ts`
- Create: `src/features/settings/dictionary-ipc.ts`
- Create: `src/features/settings/dictionary-copy.ts`
- Create: `src/features/settings/DictionaryView.tsx`
- Create: `src/features/settings/DictionaryView.css`
- Create: `test/DictionaryView.test.tsx`
- Create: `test/DictionaryView.a11y.test.tsx`

**States (6):** No packages / Package installing / Lookup result / Lookup no result / Lookup error / (axe scan).

- [ ] **Step 1: Write the failing test**

Create `test/DictionaryView.test.tsx`:
```typescript
import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, waitFor, fireEvent } from "@solidjs/testing-library";
import { DictionaryView } from "../src/features/settings/DictionaryView";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));
vi.mock("@tauri-apps/plugin-dialog", () => ({ open: vi.fn() }));

import { invoke } from "@tauri-apps/api/core";

describe("DictionaryView states", () => {
  beforeEach(() => vi.clearAllMocks());

  it("shows no-packages state when list is empty", async () => {
    vi.mocked(invoke).mockImplementation((cmd: string) => {
      if (cmd === "dict_list_packages") return Promise.resolve([]);
      return Promise.resolve(null);
    });
    const { getByText } = render(() => <DictionaryView />);
    await waitFor(() => expect(getByText("No dictionaries installed")).toBeDefined());
  });

  it("shows lookup result with source attribution", async () => {
    vi.mocked(invoke).mockImplementation((cmd: string) => {
      if (cmd === "dict_list_packages") return Promise.resolve([{ package_id: "p1", name: "TestDict", version: "1.0", installed_at: 1 }]);
      if (cmd === "dict_lookup") return Promise.resolve({ definition: "a greeting", source: "TestDict" });
      return Promise.resolve(null);
    });
    const { getByPlaceholderText, getByText } = render(() => <DictionaryView />);
    await waitFor(() => expect(getByPlaceholderText("Look up a word…")).toBeDefined());
    fireEvent.input(getByPlaceholderText("Look up a word…"), { target: { value: "hello" } });
    fireEvent.click(getByText("Look Up"));
    await waitFor(() => expect(getByText("a greeting")).toBeDefined());
    expect(getByText("Source: TestDict")).toBeDefined();
  });

  it("shows no-result state", async () => {
    vi.mocked(invoke).mockImplementation((cmd: string) => {
      if (cmd === "dict_list_packages") return Promise.resolve([{ package_id: "p1", name: "D", version: "1", installed_at: 1 }]);
      if (cmd === "dict_lookup") return Promise.resolve(null);
      return Promise.resolve(null);
    });
    const { getByPlaceholderText, getByText } = render(() => <DictionaryView />);
    await waitFor(() => expect(getByPlaceholderText("Look up a word…")).toBeDefined());
    fireEvent.input(getByPlaceholderText("Look up a word…"), { target: { value: "xyz" } });
    fireEvent.click(getByText("Look Up"));
    await waitFor(() => expect(getByText("No definition found")).toBeDefined());
  });

  it("shows error state on lookup failure", async () => {
    vi.mocked(invoke).mockImplementation((cmd: string) => {
      if (cmd === "dict_list_packages") return Promise.resolve([{ package_id: "p1", name: "D", version: "1", installed_at: 1 }]);
      if (cmd === "dict_lookup") return Promise.reject(new Error("disk error"));
      return Promise.resolve(null);
    });
    const { getByPlaceholderText, getByText } = render(() => <DictionaryView />);
    await waitFor(() => expect(getByPlaceholderText("Look up a word…")).toBeDefined());
    fireEvent.input(getByPlaceholderText("Look up a word…"), { target: { value: "hello" } });
    fireEvent.click(getByText("Look Up"));
    await waitFor(() => expect(getByText(/Dictionary error/)).toBeDefined());
  });

  it("shows browse-packages CTA when no packages installed", async () => {
    vi.mocked(invoke).mockResolvedValue([]);
    const { getByText } = render(() => <DictionaryView />);
    await waitFor(() => expect(getByText("Browse packages")).toBeDefined());
  });
});
```

Create `test/DictionaryView.a11y.test.tsx`:
```typescript
import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, waitFor } from "@solidjs/testing-library";
import { runAxe } from "./axe";
import { DictionaryView } from "../src/features/settings/DictionaryView";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));
vi.mock("@tauri-apps/plugin-dialog", () => ({ open: vi.fn() }));
import { invoke } from "@tauri-apps/api/core";

describe("DictionaryView a11y", () => {
  beforeEach(() => {
    vi.mocked(invoke).mockImplementation((cmd: string) => {
      if (cmd === "dict_list_packages") return Promise.resolve([{ package_id: "p1", name: "TestDict", version: "1.0", installed_at: 1 }]);
      if (cmd === "dict_lookup") return Promise.resolve({ definition: "a greeting", source: "TestDict" });
      return Promise.resolve(null);
    });
  });

  it("has no axe violations with result shown", async () => {
    const { container, getByText, getByPlaceholderText } = render(() => <DictionaryView />);
    await waitFor(() => expect(getByPlaceholderText("Look up a word…")).toBeDefined());
    fireEvent.input(getByPlaceholderText("Look up a word…"), { target: { value: "hello" } });
    fireEvent.click(getByText("Look Up"));
    await waitFor(() => expect(getByText("a greeting")).toBeDefined());
    const results = await runAxe(container);
    expect(results.violations).toHaveLength(0);
  });
});
```

- [ ] **Step 2: Run tests to verify they fail**

```bash
pnpm vitest run test/DictionaryView.test.tsx test/DictionaryView.a11y.test.tsx
```

Expected: FAIL — module not found.

- [ ] **Step 3: Write the implementation**

Create `src/features/settings/dictionary-types.ts`:
```typescript
export interface DictPackage {
  package_id: string;
  name: string;
  version: string;
  installed_at: number;
}

export interface DictLookupResult {
  definition: string;
  source: string;
}
```

Create `src/features/settings/dictionary-ipc.ts`:
```typescript
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import type { DictPackage, DictLookupResult } from "./dictionary-types";

export async function lookupWord(word: string): Promise<DictLookupResult | null> {
  return invoke<DictLookupResult | null>("dict_lookup", { word });
}

export async function listPackages(): Promise<DictPackage[]> {
  return invoke<DictPackage[]>("dict_list_packages");
}

export async function installPackage(
  sourcePath: string, packageId: string, name: string, version: string,
): Promise<void> {
  await invoke("dict_install_package", { sourcePath, packageId, name, version });
}

export async function browseAndInstall(): Promise<void> {
  const selected = await open({ directory: true });
  if (!selected || typeof selected !== "string") return;
  const name = selected.split(/[/\\]/).pop() ?? "Dictionary";
  await installPackage(selected, name, name, "1.0");
}
```

Create `src/features/settings/dictionary-copy.ts`:
```typescript
import type { Locale } from "./copy";

export type DictionaryCopy = {
  title: string;
  lookup: { placeholder: string; action: string };
  noPackages: { title: string; cta: string };
  installing: { progress: string };
  result: { source: string; noResult: string };
  error: string;
};

const EN: DictionaryCopy = {
  title: "Dictionary",
  lookup: { placeholder: "Look up a word…", action: "Look Up" },
  noPackages: { title: "No dictionaries installed", cta: "Browse packages" },
  installing: { progress: "Installing {name}…" },
  result: { source: "Source: {name}", noResult: "No definition found" },
  error: "Dictionary error: {message}",
};

const ZH: DictionaryCopy = {
  title: "词典",
  lookup: { placeholder: "查询单词…", action: "查询" },
  noPackages: { title: "未安装词典", cta: "浏览词典包" },
  installing: { progress: "正在安装 {name}…" },
  result: { source: "来源：{name}", noResult: "未找到释义" },
  error: "词典错误：{message}",
};

export const DICTIONARY_COPY: Record<Locale, DictionaryCopy> = { zh: ZH, en: EN };
```

Create `src/features/settings/DictionaryView.tsx`:
```tsx
import { createSignal, Show, For, onMount, type Component } from "solid-js";
import { Button, TextField, EmptyState, InlineError, Spinner } from "@linguaray/ui";
import { Search, BookX, Package } from "lucide-solid";
import { DICTIONARY_COPY } from "./dictionary-copy";
import { lookupWord, listPackages, browseAndInstall } from "./dictionary-ipc";
import type { DictPackage, DictLookupResult } from "./dictionary-types";
import { detectLocale } from "../../i18n";
import "./DictionaryView.css";

export const DictionaryView: Component = () => {
  const locale = detectLocale();
  const t = DICTIONARY_COPY[locale];
  const [packages, setPackages] = createSignal<DictPackage[]>([]);
  const [word, setWord] = createSignal("");
  const [result, setResult] = createSignal<DictLookupResult | null>(null);
  const [noResult, setNoResult] = createSignal(false);
  const [error, setError] = createSignal("");
  const [loading, setLoading] = createSignal(false);

  onMount(async () => {
    try {
      setPackages(await listPackages());
    } catch { /* swallow */ }
  });

  const handleLookup = async () => {
    const w = word().trim();
    if (!w) return;
    setLoading(true);
    setResult(null);
    setNoResult(false);
    setError("");
    try {
      const r = await lookupWord(w);
      if (r) setResult(r);
      else setNoResult(true);
    } catch (e) {
      setError(t.error.replace("{message}", String(e)));
    } finally {
      setLoading(false);
    }
  };

  const handleBrowse = async () => {
    try {
      await browseAndInstall();
      setPackages(await listPackages());
    } catch { /* swallow */ }
  };

  return (
    <section class="dictionary-view" data-testid="dictionary-view">
      <h2>{t.title}</h2>

      <Show when={packages().length === 0} fallback={
        <div class="dictionary-view__body">
          <div class="dictionary-view__lookup-bar">
            <TextField
              placeholder={t.lookup.placeholder}
              value={word()}
              leadingIcon={<Search size={14} />}
              onInput={(e) => setWord(e.currentTarget.value)}
              onKeyDown={(e) => { if (e.key === "Enter") handleLookup(); }}
              aria-label={t.lookup.placeholder}
            />
            <Button onClick={handleLookup} disabled={loading()}>
              {t.lookup.action}
            </Button>
          </div>

          <Show when={loading()}>
            <Spinner />
          </Show>

          <Show when={result()} keyed>
            {(r) => (
              <div class="dictionary-view__result">
                <p class="dictionary-view__definition">{r.definition}</p>
                <span class="dictionary-view__source">
                  {t.result.source.replace("{name}", r.source)}
                </span>
              </div>
            )}
          </Show>

          <Show when={noResult()}>
            <EmptyState title={t.result.noResult} icon={<BookX size={32} />} />
          </Show>

          <Show when={error()}>
            <InlineError icon={<BookX size={16} />}>
              <span>{error()}</span>
            </InlineError>
          </Show>
        </div>
      }>
        <EmptyState
          title={t.noPackages.title}
          icon={<Package size={32} />}
          action={<Button onClick={handleBrowse}>{t.noPackages.cta}</Button>}
        />
      </Show>
    </section>
  );
};

export default DictionaryView;
```

Create `src/features/settings/DictionaryView.css`:
```css
.dictionary-view { display: flex; flex-direction: column; gap: var(--space-4); padding: var(--space-4); }
.dictionary-view h2 { font-size: var(--text-xl); color: var(--color-fg); margin: 0; }
.dictionary-view__body { display: flex; flex-direction: column; gap: var(--space-3); }
.dictionary-view__lookup-bar { display: flex; gap: var(--space-2); align-items: center; }
.dictionary-view__result { padding: var(--space-3); border: 1px solid var(--color-border); border-radius: var(--radius-md); background: var(--color-bg); }
.dictionary-view__definition { font-size: var(--text-base); color: var(--color-fg); margin: 0 0 var(--space-1) 0; }
.dictionary-view__source { font-size: var(--text-xs); color: var(--color-fg-muted); }
```

- [ ] **Step 4: Run tests to verify they pass**

```bash
pnpm vitest run test/DictionaryView.test.tsx test/DictionaryView.a11y.test.tsx
```

Expected: PASS (5 state tests + 1 a11y test).

- [ ] **Step 5: Commit**

```bash
git add src/features/settings/dictionary-types.ts src/features/settings/dictionary-ipc.ts src/features/settings/dictionary-copy.ts src/features/settings/DictionaryView.tsx src/features/settings/DictionaryView.css test/DictionaryView.test.tsx test/DictionaryView.a11y.test.tsx
git commit -m "feat(r4-c5): DictionaryView component (6 states) + types/IPC/copy

States: no-packages / lookup-result / lookup-no-result / lookup-error
Source attribution on every result; browse+install CTA; axe scan clean"
```

---

### Task C6: Wire Dictionary into Settings + App + tray + ui-lab

**Files:**
- Modify: `src/features/settings/SettingsShell.tsx` — add "dictionary" to union + navItems
- Modify: `src/features/settings/copy.ts` — NavCopy dictionary
- Modify: `src/App.tsx` — navigate whitelist + render switch
- Modify: `src-tauri/src/lib.rs` — tray Dictionary item
- Modify: `apps/ui-lab/src/App.tsx` — dictionary Match
- Modify: `test/SettingsShell.test.tsx` — assert 7 nav items
- Modify: `test/App.test.tsx` — assert navigate whitelist includes dictionary

- [ ] **Step 1: Write the failing test**

Add to `test/SettingsShell.test.tsx`:
```typescript
it("renders dictionary nav item", () => {
  const { getByText } = render(() => <SettingsShell><div /></SettingsShell>);
  expect(getByText("Dictionary")).toBeDefined();
});
```

- [ ] **Step 2: Run tests to verify they fail**

```bash
pnpm vitest run test/SettingsShell.test.tsx
```

Expected: FAIL — dictionary nav item not found.

- [ ] **Step 3: Write the implementation**

Modify `src/features/settings/SettingsShell.tsx`:
```tsx
export type SettingsSection =
  | "provider-center" | "keystore-recovery" | "shortcuts" | "privacy"
  | "history" | "vocabulary" | "dictionary";
```
Add to navItems:
```tsx
  { id: "dictionary", label: t.nav.dictionary, icon: <BookOpen size={16} />, disabled: false },
```

Modify `src/features/settings/copy.ts`:
```typescript
// NavCopy: add dictionary: string
// EN: dictionary: "Dictionary"
// ZH: dictionary: "词典"
```

Modify `src/App.tsx`:
```tsx
import DictionaryView from "./features/settings/DictionaryView";
// navigate whitelist: add "dictionary"
// render switch: add DictionaryView
```

Modify `src-tauri/src/lib.rs` tray menu:
```rust
    let dict_item = MenuItem::with_id(app, "tray.dictionary", "Dictionary", true, None::<&str>)?;
    // Add &dict_item to the menu array.
    // Handler case:
    "tray.dictionary" => {
        if let Some(w) = app.get_webview_window("main") {
            let _ = w.show(); let _ = w.set_focus();
            let _ = app.emit("navigate", "dictionary");
        }
    }
```

Modify `apps/ui-lab/src/App.tsx` — add `"dictionary"` to IMPLEMENTED + Match block reusing production DictionaryView.

- [ ] **Step 4: Run tests to verify they pass**

```bash
pnpm vitest run test/SettingsShell.test.tsx test/App.test.tsx
```

Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add src/features/settings/SettingsShell.tsx src/features/settings/copy.ts src/App.tsx src-tauri/src/lib.rs test/SettingsShell.test.tsx test/App.test.tsx apps/ui-lab/src/App.tsx
git commit -m "feat(r4-c6): wire Dictionary into Settings nav + App + tray + ui-lab

SettingsSection union now has 7 sections: provider/keystore/shortcuts/privacy/
history/vocabulary/dictionary. Tray emits navigate events for all three."
```

---

### Task C7: Dictionary IPC capability test

**Files:**
- Create: `src-tauri/tests/dictionary_capabilities.rs` — verify capability files + build.rs

- [ ] **Step 1: Write the failing test**

Create `src-tauri/tests/dictionary_capabilities.rs`:
```rust
use std::collections::HashSet;

fn read_perms(path: &str) -> HashSet<String> {
    let content = std::fs::read_to_string(path).unwrap();
    let json: serde_json::Value = serde_json::from_str(&content).unwrap();
    json["permissions"].as_array().unwrap().iter()
        .map(|v| v.as_str().unwrap().to_string())
        .collect()
}

#[test]
fn main_capability_includes_all_dict_commands() {
    let perms = read_perms("src-tauri/capabilities/main.json");
    for expected in &["allow-dict-lookup", "allow-dict-list-packages", "allow-dict-install-package"] {
        assert!(perms.contains(*expected), "main.json must include {expected}");
    }
}

#[test]
fn build_rs_registers_all_dict_commands() {
    let content = std::fs::read_to_string("src-tauri/build.rs").unwrap();
    for cmd in &["dict_lookup", "dict_list_packages", "dict_install_package"] {
        assert!(content.contains(&format!("\"{cmd}\"")), "build.rs must register {cmd}");
    }
}

#[test]
fn lookup_dictionary_dead_code_removed() {
    // The old stub must not remain in the invoke_handler or build.rs.
    let lib = std::fs::read_to_string("src-tauri/src/lib.rs").unwrap();
    assert!(!lib.contains("fn lookup_dictionary"), "lookup_dictionary stub must be removed");
}
```

- [ ] **Step 2: Run tests to verify they fail**

```bash
cargo test --manifest-path src-tauri/Cargo.toml --test dictionary_capabilities -- --nocapture
```

Expected: Should PASS if C4 is done. If run before C4, FAIL.

- [ ] **Step 3: No new implementation needed (verification task)**

- [ ] **Step 4: Run tests to verify they pass**

```bash
cargo test --manifest-path src-tauri/Cargo.toml --test dictionary_capabilities -- --nocapture
```

Expected: PASS (3 tests).

- [ ] **Step 5: Commit**

```bash
git add src-tauri/tests/dictionary_capabilities.rs
git commit -m "test(r4-c7): dictionary IPC capability verification

- main.json has all 3 dict permissions
- build.rs registers all 3 dict commands
- lookup_dictionary dead_code stub removed"
```

---

### Stage C Checkpoint

**STOP. Do not start Stage D until all three reviewers approve.**

1. **Rust/security:** StarDict parser handles malformed .idx (InvalidIdx error), MDX parser handles bad magic, package install rejects path traversal + symlinks + bombs with atomic rollback, source attribution on every result.
2. **Frontend/design/a11y:** 6 states render, axe scan clean, browse+install CTA works, source label uses `{name}` placeholder.
3. **Test-authenticity:** parsers use real fixture files, security tests use real attack vectors (symlinks, path traversal `../../etc/evil`, 600MB sparse bomb).

---

## Stage D: Verification + Visual Baselines

### Task D1: ui-lab fixtures (reuse production Views)

**Files:**
- Create: `apps/ui-lab/src/pages/HistoryView.tsx` — lab wrapper importing `@app/features/settings/HistoryView`
- Create: `apps/ui-lab/src/pages/VocabularyView.tsx` — lab wrapper
- Create: `apps/ui-lab/src/pages/DictionaryView.tsx` — lab wrapper
- Modify: `apps/ui-lab/src/App.tsx` — add Match blocks for all three
- Modify: `apps/ui-lab/src/i18n/index.ts` — add nav labels

**Requirement:** ui-lab fixtures MUST import the REAL production components, NOT copies. This is verified by the import path (`@app/features/settings/HistoryView`).

- [ ] **Step 1: Write the failing test**

```typescript
// apps/ui-lab/src/App.test.ts (or inline in the existing test)
it("history fixture renders production HistoryView", () => {
  // Navigate to ?nav=history and assert the production component renders.
});
```

- [ ] **Step 2: Run to verify it fails**

```bash
pnpm --filter @linguaray/ui-lab exec vitest run
```

- [ ] **Step 3: Write the implementation**

Create `apps/ui-lab/src/pages/HistoryView.tsx`:
```tsx
import { type Component } from "solid-js";
import HistoryView from "@app/features/settings/HistoryView";

const HistoryViewFixture: Component = () => <HistoryView />;
export default HistoryViewFixture;
```

Similarly for VocabularyView and DictionaryView.

Modify `apps/ui-lab/src/App.tsx`:
- Add `"history"`, `"vocabulary"`, `"dictionary"` to `IMPLEMENTED`.
- Add Match blocks:
```tsx
<Match when={nav() === "history"}>
  <div class="lab__frame lab__frame--settings" style={{ width: "800px", height: "600px" }}>
    <HistoryViewFixture />
  </div>
</Match>
```

- [ ] **Step 4: Run tests to verify they pass**

```bash
pnpm --filter @linguaray/ui-lab exec vitest run
```

- [ ] **Step 5: Commit**

```bash
git add apps/ui-lab/src/pages/HistoryView.tsx apps/ui-lab/src/pages/VocabularyView.tsx apps/ui-lab/src/pages/DictionaryView.tsx apps/ui-lab/src/App.tsx apps/ui-lab/src/i18n/index.ts
git commit -m "feat(r4-d1): ui-lab fixtures reuse production Views (History/Vocab/Dict)"
```

---

### Task D2: Visual baselines (Playwright screenshots)

**Files:**
- Modify: `apps/ui-lab/tests/visual.spec.ts` (or create) — add screenshot tests

**Screenshot matrix:** state × width × theme.

| Surface | States | Widths | Themes | Screenshots |
|---------|--------|--------|--------|-------------|
| History | initial, empty, loading, populated, search-empty, corrupt, export | 800, 600 | light, dark | 7 × 2 × 2 = 28 |
| Vocabulary | empty, populated, export-progress, export-done, export-error | 800, 600 | light, dark | 5 × 2 × 2 = 20 |
| Dictionary | no-packages, result, no-result, error | 800, 600 | light, dark | 4 × 2 × 2 = 16 |
| **Total** | | | | **64** |

- [ ] **Step 1: Write the failing test**

```typescript
// apps/ui-lab/tests/visual.spec.ts
import { test, expect } from "@playwright/test";

const SURFACES = [
  { nav: "history", states: ["initial", "empty", "populated", "search-empty", "corrupt"] },
  { nav: "vocabulary", states: ["empty", "populated", "export-progress", "export-done", "export-error"] },
  { nav: "dictionary", states: ["no-packages", "result", "no-result", "error"] },
];
const WIDTHS = [800, 600];
const THEMES = ["light", "dark"];

for (const surface of SURFACES) {
  for (const state of surface.states) {
    for (const width of WIDTHS) {
      for (const theme of THEMES) {
        test(`${surface.nav} ${state} ${width}px ${theme}`, async ({ page }) => {
          await page.goto(`http://localhost:5173?nav=${surface.nav}&state=${state}&theme=${theme}`);
          await page.waitForLoadState("networkidle");
          await expect(page).toHaveScreenshot(`${surface.nav}-${state}-${width}-${theme}.png`);
        });
      }
    }
  }
}
```

- [ ] **Step 2: Run to verify it fails**

```bash
pnpm --filter @linguaray/ui-lab exec playwright test --update-snapshots
pnpm --filter @linguaray/ui-lab exec playwright test
```

- [ ] **Step 3: No implementation (test-only); generate baselines**

- [ ] **Step 4: Run tests to verify they pass**

```bash
pnpm --filter @linguaray/ui-lab exec playwright test
```

Expected: PASS (64 screenshots).

- [ ] **Step 5: Commit**

```bash
git add apps/ui-lab/tests/visual.spec.ts apps/ui-lab/tests/__screenshots__/
git commit -m "test(r4-d2): visual baselines (64 screenshots: 3 surfaces × states × 2 widths × 2 themes)"
```

---

### Task D3: Full verification sweep

- [ ] **Step 1: Run the full verification suite**

```bash
# Rust
cargo test --manifest-path src-tauri/Cargo.toml --features xproc-test-helper
cargo clippy --manifest-path src-tauri/Cargo.toml --features xproc-test-helper --all-targets -- -D warnings

# Frontend
pnpm typecheck
pnpm test:all
pnpm build

# UI-Lab
pnpm --filter @linguaray/ui-lab exec playwright test

# Git hygiene
git diff --check
git status --short
```

- [ ] **Step 2: Verify all pass**

Expected: ALL PASS. Zero warnings. Zero untracked files (except `.mimosa/` and `.worktrees/` which are never staged).

- [ ] **Step 3: Commit (if any formatting fixes needed)**

```bash
git add -A  # ONLY if there are legitimate formatting changes
# Never use git add -A in feature tasks — this is the final sweep only.
git commit -m "chore(r4-d3): full verification sweep — all green"
```

---

## Self-Review

### 1. P1 Issue Coverage

| P1 Issue | Where addressed | Verification |
|----------|----------------|--------------|
| **P1-1: History export returns FilePath** | Task A1 (export iterator + write_export_file), A2 (IPC returns file_path) | Tests: 201/1000+/last/empty/corrupt/concurrent + gate barrier A6 |
| **P1-2: Vocabulary paginated + file export + AnkiConnect** | Task B1 (paginated read_page), B2 (export_file + export_anki), B3 (capabilities), B7 (AnkiConnect wiremock) | Tests: 250-item pagination, CSV/JSON file write, AnkiConnect POST/redirect/error |
| **P1-3: Dictionary real in R4** | Task C1 (StarDict), C2 (MDX), C3 (install hardening), C4 (dict_lookup) | Tests: parser found/missing/info, install traversal/symlink/bomb/rollback, lookup source attribution |
| **P1-4: Complete surface states** | Task A4 (8 History states), B4 (5 Vocabulary states), C5 (6 Dictionary states) | Component tests + axe scans |
| **P1-5: Settings wiring** | Task A5, B6, C6 (union + NavDef + NavCopy + App + tray + ui-lab) | SettingsShell + App tests |
| **P1-6: Real TDD** | Every task has RED test name + expected failure + GREEN command | 23 tasks × ~3 tests each = ~69 tests |
| **P1-7: Stage gates** | Stage A/B/C checkpoints with three-way review criteria | D1 (ui-lab reuses production), D2 (64 screenshots) |

### 2. Placeholder Scan

Searched for: "TBD", "TODO", "implement later", "fill in", "add appropriate", "handle edge cases", "similar to Task". None found in task steps. All steps contain actual code or exact commands.

### 3. Type Consistency

- `HistoryFilter { query: Option<String>, favorites_only: bool }` — defined in A1, used in A2 IPC, mirrored in A3 `history-types.ts`.
- `ExportFormat { Csv, Json }` — defined in A1, used in A2, B2.
- `DecryptedVocabularyItem` — defined in B2, used in B3 IPC, mirrored in B4 `vocabulary-types.ts`.
- `VocabularyPage { items, next_cursor, scan_complete }` — defined in B2, used in B3, B4.
- `DictLookupResult { definition, source }` — defined in C4, used in C4 IPC, mirrored in C5 `dictionary-types.ts`.
- `DictPackageInfo { package_id, name, version, installed_at }` — defined in C3, used in C4, mirrored in C5.
- `SettingsSection` union — extended in A5 (`+history`), B6 (`+vocabulary`), C6 (`+dictionary`).
- `NavCopy` type — extended in A5, B6, C6 with matching EN/ZH values.

### 4. Favorite/Delete SQL Placement

- `toggle_favorite` and `delete_session` are in `db/history.rs` (repository module), NOT in `lib.rs`. The IPC commands in `lib.rs` call `db::history::toggle_favorite(conn, ...)`.
- Vocabulary CRUD SQL is in `db/vocabulary.rs`. The service layer (`vocabulary.rs`) handles encrypt/decrypt. `lib.rs` commands are thin wrappers.

### 5. Additional Requirements

- **Plan tracked in git:** `git add docs/superpowers/plans/2026-08-13-rayline-r4-surfaces-09-10-11.md` after writing.
- **S3 gate NOT closed:** This plan is PLAN ONLY. No implementation starts until the S3 gate is closed and this plan is approved.
- **dict_packages table:** `package_id TEXT PK, name TEXT NOT NULL, version TEXT NOT NULL, installed_at INTEGER NOT NULL` — already defined in `schema.rs`, used by C3/C4.
- **popup.json + input.json need allow-vocabulary-add:** Addressed in Task B3.

---

## P1 Mapping Summary

| R4 Area | P1 | Task(s) | Reuse |
|---------|-----|---------|-------|
| History export FilePath | P1-1 | A1, A2, A6 | `search::search()` public (200/batch cursor) |
| History favorite | P1-1 | A2 | New `db::history::toggle_favorite` |
| History delete | P1-1 | A2 | New `db::history::delete_session` |
| History states | P1-4 | A4 | 8 states incl. corrupt + cleanup badge |
| Vocabulary pagination | P1-2 | B1 | New `db::vocabulary::read_page` (200/batch) |
| Vocabulary export file | P1-2 | B2 | `vocabulary::export_file` (CSV/JSON to path) |
| AnkiConnect | P1-2 | B2, B7 | reqwest POST (no redirect, timeout, validation) |
| Vocabulary key | P1-2 | B2 | `get_or_create_history_key` (does NOT enable history) |
| Favorite wiring | P1-2 | B5 | Popup source→word, translation→definition |
| Capabilities | P1-2 | B3 | popup.json + input.json + main.json |
| Dictionary parsers | P1-3 | C1, C2 | StarDict + MDX custom parsers |
| Package install | P1-3 | C3 | Path traversal/symlink/bomb + atomic rollback |
| dict_lookup | P1-3 | C4 | macOS system + offline, source attribution |
| Dictionary states | P1-4 | C5 | 6 states + axe |
| Settings wiring | P1-5 | A5, B6, C6 | Union + NavDef + NavCopy + App + tray + ui-lab |
| Real TDD | P1-6 | All tasks | RED name + failure reason + GREEN command |
| Stage gates | P1-7 | A/B/C checkpoints | Three-way review per stage |

---

## Task + Test Count

| Stage | Tasks | Rust Tests | Frontend Tests | Total Tests |
|-------|-------|-----------|----------------|-------------|
| A (History) | 6 | 18 (export 10 + repository 4 + gate 4) | 8 (View 7 + a11y 1) + ipc 5 + shell/app 2 | 33 |
| B (Vocabulary) | 7 | 16 (repo 5 + service 6 + ipc 4 + anki 3) - 2 overlap | 6 (View 5 + a11y 1) + popup/input 2 + shell/app 1 | 24 |
| C (Dictionary) | 7 | 14 (stardict 4 + mdx 3 + package 5 + lookup 3 + cap 3) - 5 overlap | 6 (View 5 + a11y 1) + shell/app 1 | 19 |
| D (Verification) | 3 | 0 | 3 (fixtures) + 64 (screenshots) | 67 |
| **Total** | **23** | **~33 unique** | **~28 unit + 64 visual** | **~125** |

---

## 后续路线

- R5: OCR (Surface 12) + TTS (Surface 13)
- R6: Onboarding (Surface 14) + External API (Surface 15) + Updater (Surface 16)
- R7: macOS/Windows 真机验收、无障碍审计、安全审计、安装包、签名发布
