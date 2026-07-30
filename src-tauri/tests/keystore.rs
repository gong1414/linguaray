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
