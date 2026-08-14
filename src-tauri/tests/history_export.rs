use linguaray_lib::db::{schema, Database};
use linguaray_lib::history::crypto::{encrypt_field, HistoryField};
use linguaray_lib::history::export::{export_all, write_export_file, ExportFormat, HistoryFilter};
use linguaray_lib::keystore::Keystore;
use std::sync::Arc;
use tempfile::TempDir;
use zeroize::Zeroizing;

struct Harness {
    _dir: TempDir,
    db: Arc<Database>,
    keystore: Keystore,
}

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
        })
        .unwrap();
        let keystore = Keystore::new(dir.path().join("keystore")).unwrap();
        let _ = keystore.get_or_create_history_key().unwrap();
        Self {
            _dir: dir,
            db: Arc::new(db),
            keystore,
        }
    }

    fn now_base() -> i64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64
    }

    fn history_key(&self) -> Zeroizing<[u8; 32]> {
        Zeroizing::new(self.keystore.get_or_create_history_key().unwrap().0)
    }

    fn insert_encrypted_session(&self, uuid: &str, timestamp: i64, source: &str) {
        let key = self.history_key();
        self.insert_many(&key, &[(uuid, timestamp, source)]);
    }

    fn insert_many(&self, key: &[u8; 32], rows: &[(&str, i64, &str)]) {
        self.db
            .with_conn(|conn| {
                let tx = conn.transaction()?;
                for (uuid, timestamp, source) in rows {
                    let enc = encrypt_field(key, &HistoryField::SessionSource { uuid }, source.as_bytes())
                        .unwrap();
                    tx.execute(
                        "INSERT INTO history_sessions
                         (session_uuid, timestamp, trigger_source, target_language, is_favorite,
                          source_text_encrypted, source_text_nonce, crypto_version)
                         VALUES (?1, ?2, 'input', 'zh', 0, ?3, ?4, ?5)",
                        rusqlite::params![
                            uuid,
                            timestamp,
                            enc.ciphertext,
                            enc.nonce.as_slice(),
                            enc.crypto_version
                        ],
                    )?;
                }
                tx.commit()?;
                Ok(())
            })
            .unwrap();
    }
}

#[test]
fn export_all_collects_exactly_201_records_across_two_batches() {
    let h = Harness::new();
    let key = h.history_key();
    let base = Harness::now_base();
    let owned: Vec<(String, i64, String)> = (0..201)
        .map(|i| (format!("sess-{i:03}"), base + i as i64, format!("text-{i}")))
        .collect();
    let refs: Vec<(&str, i64, &str)> = owned
        .iter()
        .map(|(u, t, s)| (u.as_str(), *t, s.as_str()))
        .collect();
    h.insert_many(&key, &refs);
    let sessions = export_all(&h.db, &h.keystore, &HistoryFilter::default()).unwrap();
    assert_eq!(sessions.len(), 201);
    for w in sessions.windows(2) {
        assert!(
            w[0].timestamp >= w[1].timestamp,
            "non-monotonic: {} then {}",
            w[0].timestamp,
            w[1].timestamp
        );
    }
}

#[test]
fn export_all_handles_1000_plus_records() {
    let h = Harness::new();
    let key = h.history_key();
    let base = Harness::now_base();
    let owned: Vec<(String, i64, String)> = (0..1050)
        .map(|i| (format!("big-{i:04}"), base + i as i64, format!("word-{i}")))
        .collect();
    let refs: Vec<(&str, i64, &str)> = owned
        .iter()
        .map(|(u, t, s)| (u.as_str(), *t, s.as_str()))
        .collect();
    h.insert_many(&key, &refs);
    let sessions = export_all(&h.db, &h.keystore, &HistoryFilter::default()).unwrap();
    assert_eq!(sessions.len(), 1050);
}

#[test]
fn export_all_last_page_with_partial_batch() {
    let h = Harness::new();
    let key = h.history_key();
    let base = Harness::now_base();
    let owned: Vec<(String, i64, String)> = (0..200)
        .map(|i| (format!("p-{i:03}"), base + i as i64, format!("t-{i}")))
        .collect();
    let refs: Vec<(&str, i64, &str)> = owned
        .iter()
        .map(|(u, t, s)| (u.as_str(), *t, s.as_str()))
        .collect();
    h.insert_many(&key, &refs);
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
    let now = Harness::now_base();
    h.insert_encrypted_session("good", now, "hello");
    h.db
        .with_conn(|conn| {
            conn.execute(
                "INSERT INTO history_sessions
                 (session_uuid, timestamp, trigger_source, target_language, is_favorite,
                  source_text_encrypted, source_text_nonce, crypto_version)
                 VALUES ('bad', ?1, 'input', 'zh', 0, X'DEADBEEF', X'000102030405060708090A0B', 1)",
                rusqlite::params![now - 1],
            )?;
            Ok(())
        })
        .unwrap();
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
    let key = h.history_key();
    let base = Harness::now_base();
    let owned: Vec<(String, i64, String)> = (0..50)
        .map(|i| (format!("c-{i:03}"), base + i as i64, format!("s-{i}")))
        .collect();
    let refs: Vec<(&str, i64, &str)> = owned
        .iter()
        .map(|(u, t, s)| (u.as_str(), *t, s.as_str()))
        .collect();
    h.insert_many(&key, &refs);
    let db2 = h.db.clone();
    let deleter = std::thread::spawn(move || {
        db2.with_conn(|conn| {
            conn.execute("DELETE FROM history_sessions WHERE session_uuid='c-000'", [])?;
            Ok(())
        })
        .unwrap();
    });
    let sessions = export_all(&h.db, &h.keystore, &HistoryFilter::default()).unwrap();
    deleter.join().unwrap();
    assert!(sessions.len() == 49 || sessions.len() == 50);
}

#[test]
fn write_export_file_csv_writes_valid_csv() {
    let h = Harness::new();
    h.insert_encrypted_session("s1", Harness::now_base(), "hello");
    let sessions = export_all(&h.db, &h.keystore, &HistoryFilter::default()).unwrap();
    let out = h._dir.path().join("export.csv");
    write_export_file(&sessions, &out, ExportFormat::Csv).unwrap();
    let content = std::fs::read_to_string(&out).unwrap();
    assert!(content.contains("hello"));
    assert!(content.lines().count() >= 2);
}

#[test]
fn write_export_file_json_writes_valid_json() {
    let h = Harness::new();
    h.insert_encrypted_session("s1", Harness::now_base(), "hello");
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
    let now = Harness::now_base();
    h.insert_encrypted_session("fav", now, "favorite text");
    h.insert_encrypted_session("norm", now - 1, "normal text");
    h.db
        .with_conn(|conn| {
            conn.execute(
                "UPDATE history_sessions SET is_favorite=1 WHERE session_uuid='fav'",
                [],
            )?;
            Ok(())
        })
        .unwrap();
    let sessions = export_all(
        &h.db,
        &h.keystore,
        &HistoryFilter {
            query: None,
            favorites_only: true,
        },
    )
    .unwrap();
    assert_eq!(sessions.len(), 1);
    assert_eq!(sessions[0].session_uuid, "fav");
}
