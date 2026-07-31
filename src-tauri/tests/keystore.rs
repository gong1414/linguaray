use linguaray_lib::keystore::{encrypt, decrypt_with_identity, IdentitySource};
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
    assert!(matches!(err, linguaray_lib::keystore::KeystoreError::AuthFailed));
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
    assert!(matches!(err, linguaray_lib::keystore::KeystoreError::AuthFailed));
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
    assert!(matches!(err, linguaray_lib::keystore::KeystoreError::Envelope(_)));
}

#[test]
fn bad_length_fails_before_decrypt() {
    let mut env = encrypt("m", IdentitySource::MacosIoplatformuuid, &json!({})).unwrap();
    env.salt = base64::engine::general_purpose::STANDARD.encode([0u8; 3]);
    let err = decrypt_with_identity(&env, "m").unwrap_err();
    assert!(matches!(err, linguaray_lib::keystore::KeystoreError::Envelope(_)));
}

#[test]
fn update_keys_concurrent_no_clobber() {
    // The real Keystore uses IdentitySource::CURRENT (this machine) — deterministic
    // on a given host, so this integration test is stable. Two concurrent
    // update_keys calls (one adds "a", one adds "b") must both land — the old
    // load()+store() path would interleave and clobber one.
    use linguaray_lib::keystore::Keystore;
    use std::sync::Arc;
    use std::thread;
    let dir = std::env::temp_dir().join(format!("linguaray-keystore-conc-{}", std::process::id()));
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
    use linguaray_lib::keystore::Keystore;
    use std::thread;
    let dir = std::env::temp_dir().join(format!("linguaray-ks-samedir-{}", std::process::id()));
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
fn different_dirs_do_not_block_each_other() {
    // Round-4 review: SAME-PROCESS two threads + two Keystores, channel sync.
    // A process-global lock (the old OnceLock regression) WOULD serialize these
    // within one process — this test catches that (the old child-process approach
    // couldn't, since each process has its own statics).
    use linguaray_lib::keystore::Keystore;
    use std::sync::mpsc;
    use std::thread;
    use std::time::{Duration, Instant};

    let pid = std::process::id();
    let dir1 = std::env::temp_dir().join(format!("linguaray-ks-dd1-{}", pid));
    let dir2 = std::env::temp_dir().join(format!("linguaray-ks-dd2-{}", pid));
    let _ = std::fs::remove_dir_all(&dir1);
    let _ = std::fs::remove_dir_all(&dir2);

    let ks1 = Keystore::new(dir1.clone()).unwrap();
    let ks2 = Keystore::new(dir2.clone()).unwrap();

    // Thread A holds dir1's lock inside update_keys (sleeps), signals via channel.
    // dir2 is empty (no keystore.json → load() returns {} without KDF → deterministic).
    let (tx, rx) = mpsc::channel();
    let h1 = thread::spawn(move || {
        ks1.update_keys(|_k| {
            tx.send(()).expect("signal held");
            thread::sleep(Duration::from_secs(2));
        }).unwrap();
    });
    rx.recv_timeout(Duration::from_secs(3)).expect("dir1 did not acquire lock");

    // dir2's load() on an EMPTY dir — no KDF, no keystore.json. Must complete
    // instantly if per-dir locks; blocks ~2s if a global lock shared them.
    let start = Instant::now();
    let result = ks2.load().expect("dir2 load should succeed (empty)");
    let elapsed = start.elapsed();

    assert!(result.as_object().unwrap().is_empty(), "dir2 should be empty");
    assert!(
        elapsed < Duration::from_millis(500),
        "dir2 load took {elapsed:?} — blocked by dir1's lock (process-global lock bug)"
    );

    // Round-6 review P1 #3: timing alone can't distinguish "per-dir flock" from
    // "static OnceLock<File> where the second open reuses one fd that flock sees as
    // already-held". The OnceLock regression would NOT create dir2's sidecar (it
    // reuses the first one). Asserting BOTH sidecars exist on disk proves each
    // Keystore opens its OWN keystore.lock per call.
    assert!(
        dir1.join("keystore.lock").exists(),
        "dir1/keystore.lock sidecar must exist (per-dir lock file)"
    );
    assert!(
        dir2.join("keystore.lock").exists(),
        "dir2/keystore.lock sidecar must exist — a static OnceLock<File> would NOT create this (it'd reuse dir1's), so this catches that regression"
    );

    h1.join().unwrap();
    let _ = std::fs::remove_dir_all(&dir1);
    let _ = std::fs::remove_dir_all(&dir2);
}

#[test]
fn cross_process_lock_mutual_exclusion() {
    // Round-2 review P1 #6: PROVE the fs2 flock provides cross-process mutual
    // exclusion, not just same-process. Spawn the xproc-lock-holder binary (holds
    // the lock for HOLD_SECS), wait for it to confirm it's holding, then time our
    // own update_keys — it must block until the holder releases (≈ HOLD_SECS).
    use linguaray_lib::keystore::Keystore;
    use std::process::Command;
    use std::time::Instant;

    let dir = std::env::temp_dir().join(format!("linguaray-ks-xproc-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    const HOLD_SECS: u64 = 2;

    // Compile-time-resolved path to the helper bin (round-2 review P1 #6: use
    // CARGO_BIN_EXE, hard-fail if missing — not a silent skip). `cargo test` does
    // not build bin targets by default, so build it on demand if absent.
    let bin = env!("CARGO_BIN_EXE_xproc-lock-holder");
    if !std::path::Path::new(bin).exists() {
        // Build the helper bin so the test is self-contained (no special CI flag).
        let status = std::process::Command::new(
            std::env::var("CARGO").unwrap_or_else(|_| "cargo".into()),
        )
        .args(["build", "--bin", "xproc-lock-holder", "--tests"])
        .status()
        .expect("failed to spawn cargo to build xproc-lock-holder");
        assert!(status.success(), "building xproc-lock-holder failed");
    }
    assert!(std::path::Path::new(bin).exists(), "xproc-lock-holder still not built at {bin}");

    // Spawn the holder; it acquires the lock and sleeps HOLD_SECS.
    let mut child = Command::new(bin)
        .arg(&dir)
        .arg(HOLD_SECS.to_string())
        .spawn()
        .expect("spawn holder");

    // Wait for the holder to confirm it's holding the lock (flag file appears).
    let flag = dir.join("holding");
    let mut waited = 0;
    while !flag.exists() && waited < 200 {
        std::thread::sleep(std::time::Duration::from_millis(50));
        waited += 1;
    }
    assert!(flag.exists(), "holder did not signal it's holding the lock");

    // Now our load() on the SAME dir must BLOCK until the holder releases (≈ HOLD_SECS).
    // The dir is empty (no keystore.json) so load() does NO KDF — the ONLY thing
    // that can make it block is the fs2 flock. If the lock weren't cross-process,
    // elapsed would be ~0 (instant load of empty {}).
    let ks = Keystore::new(dir.clone()).unwrap();
    let start = Instant::now();
    let result = ks.load().expect("load should succeed");
    let elapsed = start.elapsed();

    assert!(result.as_object().unwrap().is_empty(), "dir should be empty");
    // It should have waited roughly HOLD_SECS (allow slack: the child acquires +
    // signals + we start timing with some delay). If the lock weren't cross-process,
    // elapsed would be ~0 (instant load of empty {}).
    assert!(
        elapsed >= std::time::Duration::from_millis((HOLD_SECS * 1000).saturating_sub(500)),
        "load returned in {elapsed:?} — cross-process lock did NOT block (expected ~{HOLD_SECS}s)"
    );

    let _ = child.wait();
    let _ = std::fs::remove_dir_all(&dir);
}
