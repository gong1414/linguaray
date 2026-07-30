use islandpot_lib::keystore::{encrypt, decrypt_with_identity, IdentitySource};
use serde_json::json;
use base64::Engine;

#[test]
fn encrypt_decrypt_roundtrip_with_injected_identity() {
    let id = "machine-x";
    let keys = json!({"openai": "sk-abc"});
    let env = encrypt(id, IdentitySource::MacosIoplatformuuid, &keys).unwrap();
    let out = decrypt_with_identity(&env, id).unwrap();
    assert_eq!(out, keys);
}

#[test]
fn wrong_identity_fails_closed() {
    let env = encrypt("machine-x", IdentitySource::MacosIoplatformuuid, &json!({"a":"b"})).unwrap();
    let err = decrypt_with_identity(&env, "machine-y").unwrap_err();
    assert!(matches!(err, islandpot_lib::keystore::KeystoreError::AuthFailed));
}

#[test]
fn tamper_fails_closed() {
    let env = encrypt("m", IdentitySource::MacosIoplatformuuid, &json!({"a":"b"})).unwrap();
    let mut env2 = env.clone();
    // Flip a decoded ciphertext byte, then re-encode. Tampering the *last* base64
    // char is unreliable: small payloads end in `==` padding, so the last char is
    // `=` and swapping it corrupts the base64 framing (=> Envelope error) rather
    // than the ciphertext bits. Byte-level tamper guarantees GCM detects it.
    let mut bytes = base64::engine::general_purpose::STANDARD.decode(&env2.ciphertext).unwrap();
    bytes[0] ^= 0xFF;
    env2.ciphertext = base64::engine::general_purpose::STANDARD.encode(&bytes);
    let err = decrypt_with_identity(&env2, "m").unwrap_err();
    assert!(matches!(err, islandpot_lib::keystore::KeystoreError::AuthFailed));
}

#[test]
fn nonce_is_fresh_each_write() {
    let a = encrypt("m", IdentitySource::MacosIoplatformuuid, &json!({})).unwrap();
    let b = encrypt("m", IdentitySource::MacosIoplatformuuid, &json!({})).unwrap();
    assert_ne!(a.nonce, b.nonce);
}

#[test]
fn tampered_kdf_params_rejected_not_honored() {
    let mut env = encrypt("m", IdentitySource::MacosIoplatformuuid, &json!({})).unwrap();
    env.kdf_params.m_kib = 999999;
    let err = decrypt_with_identity(&env, "m").unwrap_err();
    assert!(matches!(err, islandpot_lib::keystore::KeystoreError::Envelope(_)));
}

#[test]
fn bad_length_fails_before_decrypt() {
    let mut env = encrypt("m", IdentitySource::MacosIoplatformuuid, &json!({})).unwrap();
    env.salt = base64::engine::general_purpose::STANDARD.encode([0u8; 3]);
    let err = decrypt_with_identity(&env, "m").unwrap_err();
    assert!(matches!(err, islandpot_lib::keystore::KeystoreError::Envelope(_)));
}

#[test]
fn update_keys_concurrent_no_clobber() {
    // The real Keystore uses IdentitySource::CURRENT (this machine) — deterministic
    // on a given host, so this integration test is stable. Two concurrent
    // update_keys calls (one adds "a", one adds "b") must both land — the old
    // load()+store() path would interleave and clobber one.
    use islandpot_lib::keystore::Keystore;
    use std::sync::Arc;
    use std::thread;
    let dir = std::env::temp_dir().join(format!("islandpot-keystore-conc-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let ks = Arc::new(Keystore::new(dir.clone()).unwrap());

    let ks1 = Arc::clone(&ks);
    let ks2 = Arc::clone(&ks);
    let h1 = thread::spawn(move || ks1.update_keys(|k| { k["a"] = json!("1"); }).unwrap());
    let h2 = thread::spawn(move || ks2.update_keys(|k| { k["b"] = json!("2"); }).unwrap());
    h1.join().unwrap();
    h2.join().unwrap();

    let loaded = ks.load().unwrap();
    assert_eq!(loaded["a"], json!("1"), "key a survived concurrent write");
    assert_eq!(loaded["b"], json!("2"), "key b survived concurrent write");

    let _ = std::fs::remove_dir_all(&dir);
}
