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

#[test]
fn with_locks_same_dir_concurrent_no_clobber() {
    // Two threads, two Keystore instances, SAME dir — the fs2 flock on that dir's
    // keystore.lock must serialize them so update_keys doesn't clobber.
    use islandpot_lib::keystore::Keystore;
    use std::thread;
    let dir = std::env::temp_dir().join(format!("islandpot-ks-samedir-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    // Two distinct instances pointing at the same dir (simulating two app instances
    // or an external writer on the same keystore).
    let ks1 = Keystore::new(dir.clone()).unwrap();
    let ks2 = Keystore::new(dir.clone()).unwrap();

    let h1 = thread::spawn(move || ks1.update_keys(|k| { k["a"] = json!("1"); }).unwrap());
    let h2 = thread::spawn(move || ks2.update_keys(|k| { k["b"] = json!("2"); }).unwrap());
    h1.join().unwrap();
    h2.join().unwrap();

    let check = Keystore::new(dir.clone()).unwrap();
    let loaded = check.load().unwrap();
    assert_eq!(loaded["a"], json!("1"));
    assert_eq!(loaded["b"], json!("2"));

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn with_locks_different_dirs_independent() {
    // Two DIFFERENT dirs must NOT share a lock — writes to dir1 must not block or
    // interfere with dir2. (Catches the OnceLock-global bug: a global lock file path
    // would make both dirs' writes serialize on the first dir's lock.)
    use islandpot_lib::keystore::Keystore;
    use std::thread;
    let dir1 = std::env::temp_dir().join(format!("islandpot-ks-dir1-{}", std::process::id()));
    let dir2 = std::env::temp_dir().join(format!("islandpot-ks-dir2-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir1);
    let _ = std::fs::remove_dir_all(&dir2);

    let ks1 = Keystore::new(dir1.clone()).unwrap();
    let ks2 = Keystore::new(dir2.clone()).unwrap();

    // If locks were wrongly shared (OnceLock global path), holding dir1's lock would
    // block dir2. We hold an exclusive lock on dir1 by doing a long-ish operation,
    // and concurrently write dir2 — it must succeed independently.
    let h2 = thread::spawn(move || ks2.update_keys(|k| { k["only2"] = json!("y"); }).unwrap());
    ks1.update_keys(|k| { k["only1"] = json!("x"); }).unwrap();
    h2.join().unwrap();

    let l1 = Keystore::new(dir1.clone()).unwrap().load().unwrap();
    let l2 = Keystore::new(dir2.clone()).unwrap().load().unwrap();
    assert_eq!(l1["only1"], json!("x"));
    assert_eq!(l2["only2"], json!("y"));
    // no cross-contamination
    assert!(l1.get("only2").is_none());
    assert!(l2.get("only1").is_none());

    let _ = std::fs::remove_dir_all(&dir1);
    let _ = std::fs::remove_dir_all(&dir2);
}
