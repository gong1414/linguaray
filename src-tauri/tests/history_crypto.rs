//! R3b A1: history field encryption and typed history-key storage.

use linguaray_lib::db::history::crypto::{
    decrypt_field, encrypt_field, HistoryField, HISTORY_CRYPTO_VERSION,
};
use linguaray_lib::keystore::{store_with_identity, IdentitySource, Keystore, KeystoreData};

const ID: &str = "history-crypto-test-machine";

fn key() -> [u8; 32] {
    [0x5a; 32]
}

#[test]
fn exact_domain_separated_aad_and_roundtrip_for_every_field() {
    let cases = [
        (
            HistoryField::SessionSource { uuid: "s-1" },
            "linguaray-history-v1|session|s-1|source",
        ),
        (
            HistoryField::ResultText { uuid: "r-1" },
            "linguaray-history-v1|result|r-1|text",
        ),
        (
            HistoryField::ResultError { uuid: "r-1" },
            "linguaray-history-v1|result|r-1|error",
        ),
        (
            HistoryField::VocabularyWord { uuid: "v-1" },
            "linguaray-vocab-v1|item|v-1|word",
        ),
        (
            HistoryField::VocabularyDefinition { uuid: "v-1" },
            "linguaray-vocab-v1|item|v-1|definition",
        ),
    ];

    for (field, expected_aad) in cases {
        assert_eq!(field.aad(), expected_aad.as_bytes());
        let encrypted = encrypt_field(&key(), &field, b"private text").unwrap();
        assert_eq!(encrypted.crypto_version, HISTORY_CRYPTO_VERSION);
        assert_eq!(
            decrypt_field(&key(), &field, &encrypted).unwrap(),
            b"private text"
        );
    }
}

#[test]
fn every_encryption_uses_a_fresh_nonce_and_persisted_bytes_hide_plaintext() {
    let field = HistoryField::SessionSource { uuid: "s-1" };
    let first = encrypt_field(&key(), &field, b"needle-private-content").unwrap();
    let second = encrypt_field(&key(), &field, b"needle-private-content").unwrap();

    assert_ne!(first.nonce, second.nonce);
    assert_ne!(first.ciphertext, second.ciphertext);
    assert!(!first
        .ciphertext
        .windows(b"needle-private-content".len())
        .any(|window| window == b"needle-private-content"));
}

#[test]
fn uuid_or_domain_swap_and_tamper_fail_closed() {
    let original = HistoryField::ResultText { uuid: "r-1" };
    let encrypted = encrypt_field(&key(), &original, b"translated text").unwrap();

    assert!(decrypt_field(
        &key(),
        &HistoryField::ResultText { uuid: "r-2" },
        &encrypted
    )
    .is_err());
    assert!(decrypt_field(
        &key(),
        &HistoryField::ResultError { uuid: "r-1" },
        &encrypted
    )
    .is_err());

    let mut tampered = encrypted;
    tampered.ciphertext[0] ^= 0x80;
    assert!(decrypt_field(&key(), &original, &tampered).is_err());
}

#[test]
fn typed_history_key_is_created_once_read_and_cleared_atomically() {
    let dir = tempfile::tempdir().unwrap();
    let ks = Keystore::new(dir.path().to_path_buf()).unwrap();

    assert_eq!(ks.get_history_key_with_identity(ID).unwrap(), None);
    let first = ks.get_or_create_history_key_with_identity(ID).unwrap();
    let second = ks.get_or_create_history_key_with_identity(ID).unwrap();
    assert_eq!(
        first, second,
        "enabling twice must not rotate the history key"
    );
    assert_eq!(ks.get_history_key_with_identity(ID).unwrap(), Some(first));

    ks.clear_history_key_with_identity(ID).unwrap();
    assert_eq!(ks.get_history_key_with_identity(ID).unwrap(), None);
}

#[test]
fn malformed_history_key_is_rejected_not_treated_as_missing() {
    let dir = tempfile::tempdir().unwrap();
    let malformed = serde_json::json!({
        "version": 2,
        "provider_keys": {},
        "history_key": [1, 2, 3]
    });
    store_with_identity(
        dir.path(),
        ID,
        IdentitySource::MacosIoplatformuuid,
        &malformed,
    )
    .unwrap();
    let ks = Keystore::new(dir.path().to_path_buf()).unwrap();

    assert!(ks.get_history_key_with_identity(ID).is_err());
    assert!(ks.get_or_create_history_key_with_identity(ID).is_err());
}

#[test]
fn creating_history_key_preserves_provider_keys() {
    let dir = tempfile::tempdir().unwrap();
    let mut data = KeystoreData::new_v2(std::collections::HashMap::from([(
        "provider/ref".to_string(),
        "sk-secret".to_string(),
    )]));
    data.history_key = None;
    store_with_identity(
        dir.path(),
        ID,
        IdentitySource::MacosIoplatformuuid,
        &data.to_value().unwrap(),
    )
    .unwrap();
    let ks = Keystore::new(dir.path().to_path_buf()).unwrap();

    ks.get_or_create_history_key_with_identity(ID).unwrap();
    assert_eq!(
        ks.get_key_with_identity("provider/ref", ID)
            .unwrap()
            .as_deref(),
        Some("sk-secret")
    );
}
