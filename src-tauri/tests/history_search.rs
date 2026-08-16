use std::sync::Arc;

use linguaray_lib::db::{schema, Database};
use linguaray_lib::history::crypto::{encrypt_field, HistoryField};
use linguaray_lib::history::search::{search, HISTORY_SEARCH_BATCH};
use linguaray_lib::history::{persist_translation_session, set_enabled};
use linguaray_lib::keystore::Keystore;
use linguaray_lib::service::{Translation, TranslationOutcome};

fn harness() -> (tempfile::TempDir, Arc<Database>, Keystore) {
    let dir = tempfile::tempdir().unwrap();
    let db = Arc::new(Database::open(&dir.path().join("history.sqlite3")).unwrap());
    db.with_conn(|conn| {
        let tx = conn.transaction()?;
        schema::create_all_tables(&tx)?;
        schema::seed_singletons(&tx)?;
        tx.commit()?;
        Ok(())
    })
    .unwrap();
    let keystore = Keystore::new(dir.path().join("keystore")).unwrap();
    set_enabled(&db, &keystore, true).unwrap();
    (dir, db, keystore)
}

fn persist(db: &Database, keystore: &Keystore, source: &str, result: &str) {
    persist_translation_session(
        db,
        keystore,
        "input",
        source,
        None,
        "zh",
        &[TranslationOutcome {
            uuid: "provider-1".into(),
            result: Ok(Translation {
                text: result.into(),
                engine: "engine-1".into(),
            }),
        }],
        11,
    )
    .unwrap();
}

#[test]
fn search_decrypts_source_and_results_with_nfkc_casefold_matching() {
    let (_dir, db, keystore) = harness();
    persist(&db, &keystore, "Ｆｏｏ Straße source", "Translated BAR");

    let by_source = search(&db, &keystore, "foo", None).unwrap();
    assert_eq!(by_source.items.len(), 1);
    assert_eq!(
        by_source.items[0].source_text.as_deref(),
        Some("Ｆｏｏ Straße source")
    );
    assert_eq!(
        by_source.items[0].results[0].text.as_deref(),
        Some("Translated BAR")
    );
    assert!(!by_source.items[0].corrupt);

    let by_result = search(&db, &keystore, "bar", None).unwrap();
    assert_eq!(by_result.items.len(), 1);
    let by_full_casefold = search(&db, &keystore, "STRASSE", None).unwrap();
    assert_eq!(by_full_casefold.items.len(), 1);
}

#[test]
fn fixed_200_batch_cursor_scans_without_duplicate_sessions() {
    let (_dir, db, keystore) = harness();
    // Build the fixture with one key read instead of 201 full keystore RMW
    // cycles; the production persistence path has independent round-trip tests.
    let key = keystore.get_history_key().unwrap().unwrap().0;
    db.with_conn(|conn| {
        let tx = conn.transaction()?;
        for index in 0..=HISTORY_SEARCH_BATCH {
            let uuid = format!("session-{index:03}");
            let encrypted = encrypt_field(
                &key,
                &HistoryField::SessionSource { uuid: &uuid },
                format!("match source {index}").as_bytes(),
            )
            .unwrap();
            tx.execute(
                "INSERT INTO history_sessions
                 (session_uuid, timestamp, trigger_source, target_language,
                  is_favorite, source_text_encrypted, source_text_nonce, crypto_version)
                 VALUES (?1, ?2, 'input', 'zh', 0, ?3, ?4, ?5)",
                rusqlite::params![
                    uuid,
                    1_800_000_000_i64 - i64::try_from(index).unwrap(),
                    encrypted.ciphertext,
                    encrypted.nonce.as_slice(),
                    encrypted.crypto_version,
                ],
            )?;
        }
        tx.commit()?;
        Ok(())
    })
    .unwrap();

    let first = search(&db, &keystore, "match", None).unwrap();
    assert_eq!(first.items.len(), HISTORY_SEARCH_BATCH);
    assert!(!first.scan_complete);
    let cursor = first.next_cursor.as_deref().expect("opaque cursor");

    let second = search(&db, &keystore, "match", Some(cursor)).unwrap();
    assert_eq!(second.items.len(), 1);
    assert!(second.scan_complete);
    assert!(second.next_cursor.is_none());
    assert!(!first
        .items
        .iter()
        .any(|item| item.session_uuid == second.items[0].session_uuid));
}

#[test]
fn corrupt_record_is_returned_as_corrupt_and_does_not_abort_the_batch() {
    let (_dir, db, keystore) = harness();
    persist(&db, &keystore, "healthy source", "healthy result");
    persist(&db, &keystore, "tampered source", "tampered result");
    db.with_conn(|conn| {
        let uuid: String = conn.query_row(
            "SELECT session_uuid FROM history_sessions ORDER BY rowid DESC LIMIT 1",
            [],
            |row| row.get(0),
        )?;
        conn.execute(
            "UPDATE history_sessions SET source_text_encrypted=X'00' WHERE session_uuid=?1",
            [uuid],
        )?;
        Ok(())
    })
    .unwrap();

    let page = search(&db, &keystore, "healthy", None).unwrap();
    assert_eq!(
        page.items.len(),
        2,
        "matching healthy + visible corrupt row"
    );
    assert_eq!(page.items.iter().filter(|item| item.corrupt).count(), 1);
    assert!(page.items.iter().any(|item| !item.corrupt));
}

#[test]
fn invalid_cursor_and_missing_key_fail_closed() {
    let (_dir, db, keystore) = harness();
    assert!(search(&db, &keystore, "", Some("not-a-cursor")).is_err());
    keystore.clear_history_key().unwrap();
    assert!(search(&db, &keystore, "", None).is_err());
}
