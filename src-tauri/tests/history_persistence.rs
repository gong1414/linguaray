use std::sync::Arc;

use linguaray_lib::db::{schema, Database};
use linguaray_lib::error::Error;
use linguaray_lib::history::crypto::{
    decrypt_field, EncryptedField, HistoryField, HISTORY_CRYPTO_VERSION,
};
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
    (dir, db, keystore)
}

#[test]
fn disabled_history_writes_nothing_and_does_not_create_a_key() {
    let (_dir, db, keystore) = harness();
    let outcomes = vec![TranslationOutcome {
        uuid: "provider-1".into(),
        result: Ok(Translation {
            text: "private result".into(),
            engine: "provider/ref".into(),
        }),
    }];

    assert!(!persist_translation_session(
        &db,
        &keystore,
        "input",
        "private source",
        None,
        "zh",
        &outcomes,
        12,
    )
    .unwrap());
    assert!(keystore.get_history_key().unwrap().is_none());
    assert_eq!(
        db.with_conn(|conn| Ok(conn.query_row(
            "SELECT COUNT(*) FROM history_sessions",
            [],
            |row| row.get::<_, i64>(0),
        )?))
        .unwrap(),
        0
    );
}

#[test]
fn enabled_session_encrypts_source_success_and_failure_in_one_transaction() {
    let (dir, db, keystore) = harness();
    set_enabled(&db, &keystore, true).unwrap();
    let outcomes = vec![
        TranslationOutcome {
            uuid: "provider-success".into(),
            result: Ok(Translation {
                text: "private translated result".into(),
                engine: "provider/success".into(),
            }),
        },
        TranslationOutcome {
            uuid: "provider-failure".into(),
            result: Err(Error::LocalNoFallback),
        },
    ];

    assert!(persist_translation_session(
        &db,
        &keystore,
        "selection",
        "private selected source",
        Some("en"),
        "zh",
        &outcomes,
        27,
    )
    .unwrap());

    let (session_uuid, source_cipher, source_nonce): (String, Vec<u8>, Vec<u8>) = db
        .with_conn(|conn| {
            Ok(conn.query_row(
                "SELECT session_uuid, source_text_encrypted, source_text_nonce FROM history_sessions",
                [],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )?)
        })
        .unwrap();
    let nonce: [u8; 12] = source_nonce.try_into().unwrap();
    let encrypted = EncryptedField {
        ciphertext: source_cipher,
        nonce,
        crypto_version: HISTORY_CRYPTO_VERSION,
    };
    let key = keystore.get_history_key().unwrap().unwrap();
    assert_eq!(
        decrypt_field(
            &key.0,
            &HistoryField::SessionSource {
                uuid: &session_uuid
            },
            &encrypted,
        )
        .unwrap(),
        b"private selected source"
    );

    let tags = db
        .with_conn(|conn| {
            let mut statement = conn.prepare(
                "SELECT outcome_tag, elapsed_ms FROM history_results ORDER BY outcome_tag",
            )?;
            let rows = statement
                .query_map([], |row| {
                    Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
                })?
                .collect::<Result<Vec<_>, _>>()?;
            Ok(rows)
        })
        .unwrap();
    assert_eq!(tags, vec![("failure".into(), 27), ("success".into(), 27)]);

    drop(db);
    let bytes = std::fs::read(dir.path().join("history.sqlite3")).unwrap();
    for plaintext in [
        b"private selected source".as_slice(),
        b"private translated result".as_slice(),
        b"no fallback available".as_slice(),
    ] {
        assert!(!bytes
            .windows(plaintext.len())
            .any(|window| window == plaintext));
    }
}
