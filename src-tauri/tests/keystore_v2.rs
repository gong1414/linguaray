//! S2a tests: versioned inner KeystoreData + load_state + migrate_to_v2.
//!
//! All tests inject identity via the lower-level `encrypt`/`decrypt_with_identity`
//! API (test identity `"test-machine-uuid"`) — they never touch the real machine
//! ID, so they are deterministic across hosts. Isolation via `tempfile::tempdir`.

use linguaray_lib::keystore::{
    backup_keystore_with_identity, backup_path_in, encrypt, store_with_identity, IdentitySource,
    Keystore, KeystoreData, KeystoreLoadState, SerializableKey, KEYSTORE_DATA_VERSION,
    migrate_to_v2_with_identity,
};
use serde_json::json;
use std::collections::HashMap;

/// Identity used by every test — never the real machine identity.
const ID: &str = "test-machine-uuid";

/// Convenience: build a single-entry HashMap.
fn map_of(k: &str, v: &str) -> HashMap<String, String> {
    let mut m = HashMap::new();
    m.insert(k.to_string(), v.to_string());
    m
}

#[test]
fn v2_round_trip_encrypt_decrypt_matches() {
    // Encrypt a KeystoreData payload, decrypt it back, and assert the round trip
    // preserves version + provider_keys. This exercises the to_value/from_value
    // <-> encrypt/decrypt_with_identity path without the filesystem.
    let data = KeystoreData::new_v2(map_of("k1", "v1"));
    let value = data.to_value().unwrap();
    assert_eq!(value["version"], json!(KEYSTORE_DATA_VERSION));

    let env = encrypt(ID, IdentitySource::MacosIoplatformuuid, &value).unwrap();
    let decrypted = linguaray_lib::keystore::decrypt_with_identity(&env, ID).unwrap();
    let back = KeystoreData::from_value(&decrypted).unwrap();

    assert_eq!(back.version, KEYSTORE_DATA_VERSION);
    assert_eq!(back.provider_keys.get("k1").map(String::as_str), Some("v1"));
    assert!(back.history_key.is_none());
    assert!(back.external_api_token.is_none());
}

#[test]
fn v2_optional_fields_round_trip_when_set() {
    // history_key + external_api_token are opt-in: when present they must survive
    // the round trip; when absent they must serialize away (skip_serializing_if).
    let mut data = KeystoreData::new_v2(HashMap::new());
    data.history_key = Some(SerializableKey([1u8; 32]));
    data.external_api_token = Some("tok-xyz".into());

    let value = data.to_value().unwrap();
    // Both fields present in JSON when set.
    assert!(value.get("history_key").is_some());
    assert_eq!(value["external_api_token"], json!("tok-xyz"));

    let env = encrypt(ID, IdentitySource::MacosIoplatformuuid, &value).unwrap();
    let decrypted = linguaray_lib::keystore::decrypt_with_identity(&env, ID).unwrap();
    let back = KeystoreData::from_value(&decrypted).unwrap();

    assert_eq!(back.history_key, Some(SerializableKey([1u8; 32])));
    assert_eq!(back.external_api_token.as_deref(), Some("tok-xyz"));
}

#[test]
fn v2_optional_fields_omitted_from_json_when_none() {
    // skip_serializing_if = Option::is_none must drop the fields entirely so a
    // default S2a payload stays compact and a v1 reader doesn't see noise.
    let data = KeystoreData::new_v2(HashMap::new());
    let value = data.to_value().unwrap();
    assert!(value.get("history_key").is_none());
    assert!(value.get("external_api_token").is_none());
    assert_eq!(value["version"], json!(KEYSTORE_DATA_VERSION));
}

#[test]
fn load_state_detects_legacy_v1_flat_map() {
    // A v1 keystore is a flat map with NO "version" field. load_state must
    // classify it as LegacyV1 (the map), not CurrentV2 or Missing.
    let dir = tempfile::tempdir().unwrap();
    let legacy = json!({"openai": "sk-x", "anthropic": "sk-y"});
    store_with_identity(dir.path(), ID, IdentitySource::MacosIoplatformuuid, &legacy).unwrap();

    let ks = Keystore::new(dir.path().to_path_buf()).unwrap();
    let state = ks.load_state_with_identity(ID);

    match state {
        KeystoreLoadState::LegacyV1(map) => {
            assert_eq!(map.len(), 2);
            assert_eq!(map.get("openai").map(String::as_str), Some("sk-x"));
            assert_eq!(map.get("anthropic").map(String::as_str), Some("sk-y"));
        }
        other => panic!("expected LegacyV1, got {other:?}"),
    }
}

#[test]
fn load_state_detects_missing() {
    // No keystore file → Missing (NOT LegacyV1({})). This is the key behavioral
    // difference from the old load() which returned {} for both.
    let dir = tempfile::tempdir().unwrap();
    let ks = Keystore::new(dir.path().to_path_buf()).unwrap();
    let state = ks.load_state_with_identity(ID);
    assert!(matches!(state, KeystoreLoadState::Missing));
}

#[test]
fn load_state_free_function_detects_missing() {
    // The free function load_state(dir) also returns Missing without creating the dir.
    let dir = tempfile::tempdir().unwrap();
    let state = linguaray_lib::keystore::load_state(dir.path());
    assert!(matches!(state, KeystoreLoadState::Missing));
}

#[test]
fn load_state_detects_current_v2() {
    // A v2 keystore (payload with version:2) classifies as CurrentV2(KeystoreData).
    let dir = tempfile::tempdir().unwrap();
    let data = KeystoreData::new_v2(map_of("openai", "sk-v2"));
    store_with_identity(
        dir.path(),
        ID,
        IdentitySource::MacosIoplatformuuid,
        &data.to_value().unwrap(),
    )
    .unwrap();

    let ks = Keystore::new(dir.path().to_path_buf()).unwrap();
    match ks.load_state_with_identity(ID) {
        KeystoreLoadState::CurrentV2(d) => {
            assert_eq!(d.version, KEYSTORE_DATA_VERSION);
            assert_eq!(d.get_provider_key("openai"), Some("sk-v2"));
        }
        other => panic!("expected CurrentV2, got {other:?}"),
    }
}

#[test]
fn load_state_detects_corrupt_on_wrong_identity() {
    // Encrypted with one identity, probed with another → decrypt fails → Corrupt.
    let dir = tempfile::tempdir().unwrap();
    store_with_identity(
        dir.path(),
        "other-machine",
        IdentitySource::MacosIoplatformuuid,
        &json!({"a": "b"}),
    )
    .unwrap();

    let ks = Keystore::new(dir.path().to_path_buf()).unwrap();
    match ks.load_state_with_identity(ID) {
        KeystoreLoadState::Corrupt(_) => {}
        other => panic!("expected Corrupt, got {other:?}"),
    }
}

#[test]
fn load_state_detects_corrupt_on_tampered_ciphertext() {
    // Tamper with the on-disk ciphertext bytes → GCM auth fails → Corrupt.
    use base64::Engine;
    let dir = tempfile::tempdir().unwrap();
    let env = encrypt(ID, IdentitySource::MacosIoplatformuuid, &json!({"a": "b"})).unwrap();
    // Read back, flip a ciphertext byte, re-serialize to disk.
    let path = dir.path().join("keystore.json");
    let mut env2 = env.clone();
    let mut bytes = base64::engine::general_purpose::STANDARD.decode(&env2.ciphertext).unwrap();
    bytes[0] ^= 0xFF;
    env2.ciphertext = base64::engine::general_purpose::STANDARD.encode(&bytes);
    let json_bytes = serde_json::to_vec(&env2).unwrap();
    std::fs::write(&path, &json_bytes).unwrap();

    let ks = Keystore::new(dir.path().to_path_buf()).unwrap();
    match ks.load_state_with_identity(ID) {
        KeystoreLoadState::Corrupt(_) => {}
        other => panic!("expected Corrupt, got {other:?}"),
    }
}

#[test]
fn load_state_detects_corrupt_on_garbage_file() {
    // A file that isn't even a valid Envelope JSON → Corrupt (malformed).
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("keystore.json");
    std::fs::write(&path, b"not json at all").unwrap();

    let ks = Keystore::new(dir.path().to_path_buf()).unwrap();
    match ks.load_state_with_identity(ID) {
        KeystoreLoadState::Corrupt(_) => {}
        other => panic!("expected Corrupt, got {other:?}"),
    }
}

#[test]
fn migrate_to_v2_converts_legacy_map() {
    // LegacyV1({"openai":"sk-x"}) → migrate → load_state returns CurrentV2 with
    // version=2, provider_keys={"openai":"sk-x"}, history_key=None. Original keys
    // preserved (the migration copies the map, it doesn't lose entries).
    let dir = tempfile::tempdir().unwrap();
    // Seed a v1 keystore.
    store_with_identity(
        dir.path(),
        ID,
        IdentitySource::MacosIoplatformuuid,
        &json!({"openai": "sk-x", "google": "gk-y"}),
    )
    .unwrap();

    // Migrate using the injected-identity helper.
    let legacy = {
        let mut m = HashMap::new();
        m.insert("openai".to_string(), "sk-x".to_string());
        m.insert("google".to_string(), "gk-y".to_string());
        m
    };
    migrate_to_v2_with_identity(dir.path(), legacy, ID).unwrap();

    // Now load_state must see CurrentV2.
    let ks = Keystore::new(dir.path().to_path_buf()).unwrap();
    match ks.load_state_with_identity(ID) {
        KeystoreLoadState::CurrentV2(d) => {
            assert_eq!(d.version, KEYSTORE_DATA_VERSION);
            assert_eq!(d.provider_keys.len(), 2);
            assert_eq!(d.get_provider_key("openai"), Some("sk-x"));
            assert_eq!(d.get_provider_key("google"), Some("gk-y"));
            assert!(d.history_key.is_none());
            assert!(d.external_api_token.is_none());
        }
        other => panic!("expected CurrentV2 after migrate, got {other:?}"),
    }
}

#[test]
fn migrate_to_v2_creates_backup() {
    // Migration must create keystore.json.bak-pre-migration holding the original
    // (v1) bytes, secured with fs_acl::secure_file.
    let dir = tempfile::tempdir().unwrap();
    store_with_identity(
        dir.path(),
        ID,
        IdentitySource::MacosIoplatformuuid,
        &json!({"openai": "sk-x"}),
    )
    .unwrap();
    let original_bytes = std::fs::read(dir.path().join("keystore.json")).unwrap();

    migrate_to_v2_with_identity(dir.path(), map_of("openai", "sk-x"), ID).unwrap();

    let bak = backup_path_in(dir.path());
    assert!(bak.exists(), "backup must be created");
    assert_eq!(
        std::fs::read(&bak).unwrap(),
        original_bytes,
        "backup must hold the pre-migration v1 bytes"
    );
}

#[test]
fn migrate_to_v2_backup_is_create_new_not_overwrite() {
    // If a backup already exists from a prior run, migrate_to_v2 must NOT
    // overwrite it (create-new semantics). We pre-create the backup, capture its
    // bytes, run a migration that would otherwise overwrite, and assert the bytes
    // are unchanged.
    let dir = tempfile::tempdir().unwrap();
    // Seed a v1 keystore.
    store_with_identity(
        dir.path(),
        ID,
        IdentitySource::MacosIoplatformuuid,
        &json!({"openai": "first"}),
    )
    .unwrap();

    // Pre-create the backup with sentinel content (simulating a prior migration's
    // backup). secure_file it so perms match the prod path.
    let bak = backup_path_in(dir.path());
    std::fs::write(&bak, b"PRE-EXISTING-BACKUP-SENTINEL").unwrap();
    let before = std::fs::read(&bak).unwrap();

    // Now migrate — this would overwrite the backup if the logic were wrong.
    migrate_to_v2_with_identity(dir.path(), map_of("openai", "first"), ID).unwrap();

    let after = std::fs::read(&bak).unwrap();
    assert_eq!(
        before, after,
        "prior backup must NOT be overwritten (create-new semantics)"
    );
}

#[test]
fn migrate_to_v2_backup_secured() {
    // The backup must be secured via fs_acl::secure_file (owner-only on Unix,
    // protected DACL on Windows). On Unix we can assert the mode directly.
    let dir = tempfile::tempdir().unwrap();
    store_with_identity(
        dir.path(),
        ID,
        IdentitySource::MacosIoplatformuuid,
        &json!({"openai": "sk-x"}),
    )
    .unwrap();
    migrate_to_v2_with_identity(dir.path(), map_of("openai", "sk-x"), ID).unwrap();

    let bak = backup_path_in(dir.path());
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = std::fs::metadata(&bak).unwrap().permissions().mode();
        assert_eq!(
            mode & 0o777,
            0o600,
            "backup must be owner-only (0o600) on Unix; got {mode:o}"
        );
    }
    #[cfg(not(unix))]
    {
        // On Windows the secure_file ACL is verified by the dedicated win32_dacl
        // test in keystore.rs; here we just assert the file exists.
        assert!(bak.exists());
    }
}

#[test]
fn backup_keystore_creates_backup_for_legacy_v1() {
    // Standalone backup (Phase 1 of the migration coordinator): a LegacyV1
    // keystore must get a keystore.json.bak-pre-migration holding the original
    // v1 bytes, without performing the v2 rewrite. Exercises the same
    // backup_locked core the full migration uses.
    let dir = tempfile::tempdir().unwrap();
    store_with_identity(
        dir.path(),
        ID,
        IdentitySource::MacosIoplatformuuid,
        &json!({"openai": "sk-x"}),
    )
    .unwrap();
    let original_bytes = std::fs::read(dir.path().join("keystore.json")).unwrap();

    backup_keystore_with_identity(dir.path(), ID).unwrap();

    let bak = backup_path_in(dir.path());
    assert!(bak.exists(), "standalone backup must create the .bak file");
    assert_eq!(
        std::fs::read(&bak).unwrap(),
        original_bytes,
        "backup must hold the original v1 bytes"
    );
    // The canonical keystore is NOT rewritten by a standalone backup.
    assert_eq!(
        std::fs::read(dir.path().join("keystore.json")).unwrap(),
        original_bytes,
        "standalone backup must not touch the canonical keystore"
    );
}

#[test]
fn backup_keystore_missing_is_noop() {
    // Missing → Ok(()) with no backup file created.
    let dir = tempfile::tempdir().unwrap();
    backup_keystore_with_identity(dir.path(), ID).unwrap();
    assert!(!backup_path_in(dir.path()).exists(), "no backup for a missing keystore");
}

#[test]
fn backup_keystore_current_v2_is_noop() {
    // CurrentV2 → Ok(()) (already migrated; no v1 to preserve). No backup created.
    let dir = tempfile::tempdir().unwrap();
    let data = KeystoreData::new_v2(map_of("openai", "sk-v2"));
    store_with_identity(
        dir.path(),
        ID,
        IdentitySource::MacosIoplatformuuid,
        &data.to_value().unwrap(),
    )
    .unwrap();

    backup_keystore_with_identity(dir.path(), ID).unwrap();
    assert!(
        !backup_path_in(dir.path()).exists(),
        "no backup when keystore is already v2"
    );
}

#[test]
fn backup_keystore_corrupt_errors() {
    // Corrupt → Err (cannot back up an unreadable file). No backup created.
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("keystore.json");
    std::fs::write(&path, b"not json at all").unwrap();

    let err = backup_keystore_with_identity(dir.path(), ID).unwrap_err();
    assert!(
        !backup_path_in(dir.path()).exists(),
        "no backup when the keystore is corrupt"
    );
    // Surface the underlying failure kind so a silent Ok(()) regression trips this.
    let _ = err.to_string();
}

#[test]
fn backup_keystore_is_idempotent() {
    // A second backup_keystore call must NOT overwrite a prior backup
    // (create-new semantics) and must NOT error.
    let dir = tempfile::tempdir().unwrap();
    store_with_identity(
        dir.path(),
        ID,
        IdentitySource::MacosIoplatformuuid,
        &json!({"openai": "sk-x"}),
    )
    .unwrap();
    backup_keystore_with_identity(dir.path(), ID).unwrap();
    let first = std::fs::read(backup_path_in(dir.path())).unwrap();

    // Second call: backup already exists → no-op, prior bytes untouched.
    backup_keystore_with_identity(dir.path(), ID).unwrap();
    let second = std::fs::read(backup_path_in(dir.path())).unwrap();
    assert_eq!(first, second, "idempotent backup must not overwrite");
}

#[test]
fn migrate_to_v2_skips_backup_if_already_backed_up() {
    // If a standalone backup was made first (Phase 1), the subsequent
    // migrate_to_v2 (Phase 4) must reuse it rather than overwrite — the core
    // treats backup_locked as idempotent.
    let dir = tempfile::tempdir().unwrap();
    store_with_identity(
        dir.path(),
        ID,
        IdentitySource::MacosIoplatformuuid,
        &json!({"openai": "sk-x"}),
    )
    .unwrap();
    // Phase 1: standalone backup.
    backup_keystore_with_identity(dir.path(), ID).unwrap();
    let phase1_backup = std::fs::read(backup_path_in(dir.path())).unwrap();

    // Phase 4: full migration (backup + rewrite). Backup must be unchanged.
    migrate_to_v2_with_identity(dir.path(), map_of("openai", "sk-x"), ID).unwrap();
    let phase4_backup = std::fs::read(backup_path_in(dir.path())).unwrap();
    assert_eq!(phase1_backup, phase4_backup, "migration must not overwrite an existing backup");

    // And the canonical keystore is now v2.
    let ks = Keystore::new(dir.path().to_path_buf()).unwrap();
    match ks.load_state_with_identity(ID) {
        KeystoreLoadState::CurrentV2(d) => {
            assert_eq!(d.version, KEYSTORE_DATA_VERSION);
            assert_eq!(d.get_provider_key("openai"), Some("sk-x"));
        }
        other => panic!("expected CurrentV2 after migrate, got {other:?}"),
    }
}

#[test]
fn keystore_data_get_set_remove_provider_key() {
    // KeystoreData methods on provider_keys (simple HashMap operations). Verifies
    // the typed accessors work and return the right Option<&str>/Option<String>.
    let mut data = KeystoreData::new_v2(HashMap::new());

    // get on empty → None.
    assert!(data.get_provider_key("openai").is_none());

    // set + get.
    data.set_provider_key("openai", "sk-abc");
    assert_eq!(data.get_provider_key("openai"), Some("sk-abc"));

    // set again overwrites.
    data.set_provider_key("openai", "sk-def");
    assert_eq!(data.get_provider_key("openai"), Some("sk-def"));

    // remove returns the old value.
    let removed = data.remove_provider_key("openai");
    assert_eq!(removed.as_deref(), Some("sk-def"));
    assert!(data.get_provider_key("openai").is_none());
    assert!(data.provider_keys.is_empty());

    // remove on absent key → None (no panic).
    assert!(data.remove_provider_key("nope").is_none());
}

#[test]
fn load_state_missing_is_fast() {
    // A Missing probe returns BEFORE taking the flock (no file → no KDF → no
    // lock), so it is effectively instantaneous. This is a latency guard for the
    // fast path, NOT a concurrency test (we can't reliably hold a cross-process
    // lock here); the lock-acquisition ordering itself is unit-tested via the
    // fast-path branch in load_state_with.
    use std::time::Instant;
    let dir = tempfile::tempdir().unwrap();
    let ks = Keystore::new(dir.path().to_path_buf()).unwrap();
    let start = Instant::now();
    let state = ks.load_state_with_identity(ID);
    let elapsed = start.elapsed();
    assert!(matches!(state, KeystoreLoadState::Missing));
    assert!(
        elapsed < std::time::Duration::from_millis(500),
        "Missing probe took {elapsed:?} — should skip the lock entirely"
    );
}
