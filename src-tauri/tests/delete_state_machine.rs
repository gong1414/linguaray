//! Delete state machine fault-injection tests (S2a step 5) — D1–D5.
//!
//! These tests exercise [`provider_resume_deletions`] against a REAL temp DB +
//! REAL temp keystore. They simulate a crash at each checkpoint of the 3-step
//! delete flow by running only some steps, inspecting disk, then invoking resume
//! and asserting the resulting state is fully consistent.
//!
//! ## The 3-step delete model
//! 1. `begin_delete`   — DB tx commits: `status='deleting'`, `enabled=0`,
//!    evicted from slots, consent invalidated. (key STILL in keystore)
//! 2. keystore remove  — key purged from keystore (committed). (status still
//!    `deleting`)
//! 3. `finalize_delete`— DB tx commits: `status='deleted'`, name → `deleted: …`.
//!
//! ## Test matrix
//! - **D1**: full delete (no crash) → status=deleted, key absent, not in list.
//! - **D2**: crash after step 1 (DB committed, key STILL EXISTS) → resume removes
//!   key + tombstones.
//! - **D3**: crash after step 2 (key already removed, status still `deleting`) →
//!   resume tombstones (no error on the already-absent key).
//! - **D4**: crash after step 3 (tombstone) → resume is a no-op.
//! - **D5**: a `deleting` provider is excluded from `list()` and from active
//!   selection validation.
//!
//! All keystore operations use the injected identity `"test-machine-uuid"` via
//! the `*_with_identity` seams — they NEVER touch the real OS identity, so they
//! are deterministic across hosts. Isolation via `tempfile::tempdir`.

use linguaray_lib::db::delete::provider_resume_deletions_with_identity;
use linguaray_lib::db::providers::{self, ProviderProfile, ProviderStatus};
use linguaray_lib::db::schema;
use linguaray_lib::db::{Database, DbError};
use linguaray_lib::keystore::{
    self, store_with_identity, IdentitySource, KeystoreData, KeystoreLoadState,
};
use std::collections::HashMap;
use tempfile::tempdir;

/// Injected identity used by every test — never the real machine identity.
const ID: &str = "test-machine-uuid";

/// A fresh fixture: a temp DB dir + keystore dir, a seeded DB (schema +
/// singletons + one OpenAI provider), and a keystore holding that provider's
/// key under `provider/<uuid>`.
struct Fixture {
    /// Holds both the DB temp dir and the keystore temp dir alive.
    _db_dir: tempfile::TempDir,
    _ks_dir: tempfile::TempDir,
    db: Database,
    ks_dir_path: std::path::PathBuf,
    profile: ProviderProfile,
}

impl Fixture {
    fn keystore_dir(&self) -> &std::path::Path {
        &self.ks_dir_path
    }
}

/// Build a fresh fixture. The provider's `secret_ref` is `provider/<uuid>` and
/// the keystore's `provider_keys` map is seeded with exactly that key → value.
fn fixture() -> Fixture {
    // DB in its own temp dir.
    let db_dir = tempdir().unwrap();
    let db_path = db_dir.path().join("test.db");
    let db = Database::open(&db_path).unwrap();
    db.with_conn(|conn| {
        let tx = conn.transaction()?;
        schema::create_all_tables(&tx)?;
        schema::seed_singletons(&tx)?;
        tx.commit()?;
        Ok(())
    })
    .unwrap();

    // Create the OpenAI provider (secret_ref = provider/<uuid>).
    let profile = db
        .with_conn(|conn| {
            providers::create(
                conn,
                "openai",
                "OpenAI",
                "https://api.openai.com/v1/chat/completions",
                None,
            )
        })
        .unwrap();

    // Keystore in its own temp dir, seeded with the provider key.
    let ks_dir = tempdir().unwrap();
    let ks_dir_path = ks_dir.path().to_path_buf();
    let mut keys = HashMap::new();
    keys.insert(profile.secret_ref.clone(), "sk-test-secret".to_string());
    let data = KeystoreData::new_v2(keys);
    store_with_identity(
        &ks_dir_path,
        ID,
        IdentitySource::MacosIoplatformuuid,
        &data.to_value().unwrap(),
    )
    .unwrap();

    Fixture {
        _db_dir: db_dir,
        _ks_dir: ks_dir,
        db,
        ks_dir_path,
        profile,
    }
}

// ─── keystore inspection helpers ──────────────────────────────────────────

/// Does the keystore still carry the key named `secret_ref`? Drives the
/// sanctioned `load_state_with_identity` classification path.
fn keystore_has_key(ks_dir: &std::path::Path, secret_ref: &str) -> bool {
    match keystore::load_state_with_identity(ks_dir, ID) {
        KeystoreLoadState::CurrentV2(data) => data.get_provider_key(secret_ref).is_some(),
        KeystoreLoadState::LegacyV1(map) => map.contains_key(secret_ref),
        // Missing / Corrupt → no key.
        _ => false,
    }
}

/// Remove the key named `secret_ref` from the keystore, simulating the
/// already-completed step-2 (keystore remove) of the delete flow. Uses the
/// sanctioned `update_keys_with_identity` RMW so the on-disk shape is exactly
/// what production leaves behind. Handles both v2 (`provider_keys` map) and
/// legacy v1 (flat map) shapes, mirroring `remove_provider_key_mut` in the lib.
fn keystore_remove_key(ks_dir: &std::path::Path, secret_ref: &str) {
    let ks = keystore::Keystore::new(ks_dir.to_path_buf()).unwrap();
    ks.update_keys_with_identity(
        |keys| {
            let Some(obj) = keys.as_object_mut() else {
                return;
            };
            if obj.contains_key("provider_keys") {
                if let Some(inner) = obj.get_mut("provider_keys").and_then(|v| v.as_object_mut()) {
                    inner.remove(secret_ref);
                }
            } else {
                obj.remove(secret_ref);
            }
        },
        ID,
    )
    .unwrap();
}

/// Run the resume sweep with the test identity and assert it finalized exactly
/// `expected` rows.
fn resume(f: &Fixture, expected: usize) {
    let n = provider_resume_deletions_with_identity(&f.db, f.keystore_dir(), ID).unwrap();
    assert_eq!(
        n, expected,
        "resume finalized {n} rows, expected {expected}"
    );
}

// ─── D1: full delete (no crash) ───────────────────────────────────────────

/// D1 — happy path: begin → keystore remove → finalize runs to completion, then
/// a redundant resume is a no-op. Asserts the terminal state: status=deleted,
/// key absent, name tombstoned, hidden from `list()`.
#[test]
fn d1_full_delete_no_crash() {
    let f = fixture();
    let orig_name = f.profile.name.clone();

    // Full delete flow, driven manually to exercise every step.
    let secret_ref = f
        .db
        .with_conn(|conn| providers::begin_delete(conn, &f.profile.uuid))
        .unwrap();
    assert_eq!(secret_ref, f.profile.secret_ref);

    // Step 1 committed: deleting + key STILL present.
    assert!(keystore_has_key(f.keystore_dir(), &f.profile.secret_ref));
    let mid = f
        .db
        .with_conn(|conn| providers::get(conn, &f.profile.uuid))
        .unwrap();
    assert_eq!(mid.status, ProviderStatus::Deleting.as_str());

    keystore_remove_key(f.keystore_dir(), &f.profile.secret_ref);
    f.db
        .with_conn(|conn| providers::finalize_delete(conn, &f.profile.uuid))
        .unwrap();

    // Terminal state.
    let done = f
        .db
        .with_conn(|conn| providers::get(conn, &f.profile.uuid))
        .unwrap();
    assert_eq!(done.status, ProviderStatus::Deleted.as_str());
    assert_eq!(done.name, format!("deleted: {orig_name}"));
    assert!(!keystore_has_key(f.keystore_dir(), &f.profile.secret_ref));
    assert!(
        f.db
            .with_conn(|conn| providers::list(conn))
            .unwrap()
            .is_empty(),
        "tombstoned provider is hidden from list()"
    );

    // Redundant resume is a no-op (no `deleting` rows left).
    resume(&f, 0);
}

// ─── D2: crash after step 1 (DB committed, key STILL EXISTS) ──────────────

/// D2 — crash after step 1: `begin_delete` committed (status=deleting, key still
/// live). Resume must remove the key and finalize the tombstone.
#[test]
fn d2_crash_after_step1_resume_removes_key_and_tombstones() {
    let f = fixture();

    // Crash point: ONLY step 1 ran. Key is still present on disk.
    f.db
        .with_conn(|conn| providers::begin_delete(conn, &f.profile.uuid))
        .unwrap();
    assert!(
        keystore_has_key(f.keystore_dir(), &f.profile.secret_ref),
        "precondition: key still present after step-1-only crash"
    );
    let mid = f
        .db
        .with_conn(|conn| providers::get(conn, &f.profile.uuid))
        .unwrap();
    assert_eq!(mid.status, ProviderStatus::Deleting.as_str());

    // Resume finishes the delete.
    resume(&f, 1);

    assert!(
        !keystore_has_key(f.keystore_dir(), &f.profile.secret_ref),
        "resume removed the key"
    );
    let done = f
        .db
        .with_conn(|conn| providers::get(conn, &f.profile.uuid))
        .unwrap();
    assert_eq!(done.status, ProviderStatus::Deleted.as_str());
    assert!(done.name.starts_with("deleted: "));
    assert!(
        f.db
            .with_conn(|conn| providers::list(conn))
            .unwrap()
            .is_empty()
    );
}

// ─── D3: crash after step 2 (key removed, status still deleting) ──────────

/// D3 — crash after step 2: the key was already purged from the keystore, but
/// `finalize_delete` never ran (status still `deleting`). Resume must tombstone
/// the row WITHOUT erroring on the already-absent key (idempotent removal).
#[test]
fn d3_crash_after_step2_resume_tombstones_no_error() {
    let f = fixture();

    // Steps 1 + 2 ran; step 3 (finalize) did NOT.
    f.db
        .with_conn(|conn| providers::begin_delete(conn, &f.profile.uuid))
        .unwrap();
    keystore_remove_key(f.keystore_dir(), &f.profile.secret_ref);

    // Precondition: key is gone but status is still `deleting`.
    assert!(!keystore_has_key(f.keystore_dir(), &f.profile.secret_ref));
    let mid = f
        .db
        .with_conn(|conn| providers::get(conn, &f.profile.uuid))
        .unwrap();
    assert_eq!(mid.status, ProviderStatus::Deleting.as_str());

    // Resume tombstones without erroring on the already-absent key.
    resume(&f, 1);

    assert!(!keystore_has_key(f.keystore_dir(), &f.profile.secret_ref));
    let done = f
        .db
        .with_conn(|conn| providers::get(conn, &f.profile.uuid))
        .unwrap();
    assert_eq!(done.status, ProviderStatus::Deleted.as_str());
    assert!(done.name.starts_with("deleted: "));
}

// ─── D4: crash after step 3 (tombstone) — resume is a no-op ───────────────

/// D4 — crash after step 3: the full delete ran (status=deleted, tombstone).
/// Resume must be a no-op: 0 rows finalized, state unchanged, no error.
#[test]
fn d4_crash_after_step3_resume_is_noop() {
    let f = fixture();

    // Full delete already completed.
    f.db
        .with_conn(|conn| providers::begin_delete(conn, &f.profile.uuid))
        .unwrap();
    keystore_remove_key(f.keystore_dir(), &f.profile.secret_ref);
    f.db
        .with_conn(|conn| providers::finalize_delete(conn, &f.profile.uuid))
        .unwrap();

    let before = f
        .db
        .with_conn(|conn| providers::get(conn, &f.profile.uuid))
        .unwrap();
    assert_eq!(before.status, ProviderStatus::Deleted.as_str());

    // Resume is a no-op.
    resume(&f, 0);

    let after = f
        .db
        .with_conn(|conn| providers::get(conn, &f.profile.uuid))
        .unwrap();
    assert_eq!(after.status, before.status);
    assert_eq!(after.name, before.name);
    assert!(!keystore_has_key(f.keystore_dir(), &f.profile.secret_ref));
}

// ─── D5: deleting provider excluded from list + active selection ──────────

/// D5 — a `deleting` provider disappears from the user-facing `list()` and is
/// rejected by `validate_active_selection` (it isn't active+enabled). This is
/// the guarantee `begin_delete` makes the instant its tx commits: the row is
/// gone from the UI and from any selection slot before the key is even purged.
#[test]
fn d5_deleting_provider_excluded_from_list_and_selection() {
    let f = fixture();
    let uuid = f.profile.uuid.clone();

    // begin_delete evicts the row from every selection slot and flips status.
    f.db
        .with_conn(|conn| providers::begin_delete(conn, &uuid))
        .unwrap();

    // list() hides it immediately.
    let listed = f.db.with_conn(|conn| providers::list(conn)).unwrap();
    assert!(
        listed.iter().all(|p| p.uuid != uuid),
        "deleting provider is hidden from list()"
    );

    // It's still visible in list_all() (audit view).
    let all = f.db.with_conn(|conn| providers::list_all(conn)).unwrap();
    assert!(all.iter().any(|p| p.uuid == uuid));

    // validate_active_selection rejects a deleting row in the primary slot.
    let deleting_row = all.iter().find(|p| p.uuid == uuid).unwrap();
    let err = providers::validate_active_selection(
        &uuid,
        &[],
        None,
        std::slice::from_ref(deleting_row),
    )
    .unwrap_err();
    assert!(
        matches!(err, DbError::Integrity(_)),
        "deleting provider must be rejected as a selection, got {err:?}"
    );

    // And a second resume on this fixture still finishes cleanly (defensive:
    // the selection-exclusion guarantee holds regardless of resume state).
    resume(&f, 1);
    assert!(
        f.db
            .with_conn(|conn| providers::list(conn))
            .unwrap()
            .is_empty()
    );
}

// ─── bonus: multiple deleting rows all resume in one sweep ────────────────

/// Bonus guard: when several providers are mid-delete at once, a single resume
/// sweep finalizes ALL of them (in deterministic uuid order) and leaves no
/// `deleting` row behind. This is the realistic multi-provider startup case.
#[test]
fn resume_handles_multiple_deleting_rows() {
    let f = fixture();

    // Create a second provider and begin_delete on both.
    let p2 = f
        .db
        .with_conn(|conn| {
            providers::create(
                conn,
                "anthropic",
                "Claude",
                "https://api.anthropic.com/v1/messages",
                None,
            )
        })
        .unwrap();
    // Seed the keystore with both keys.
    let mut keys = HashMap::new();
    keys.insert(f.profile.secret_ref.clone(), "sk-one".to_string());
    keys.insert(p2.secret_ref.clone(), "sk-two".to_string());
    let data = KeystoreData::new_v2(keys);
    store_with_identity(
        f.keystore_dir(),
        ID,
        IdentitySource::MacosIoplatformuuid,
        &data.to_value().unwrap(),
    )
    .unwrap();

    // begin_delete both — crash before keystore removal (D2-style for two rows).
    f.db
        .with_conn(|conn| providers::begin_delete(conn, &f.profile.uuid))
        .unwrap();
    f.db
        .with_conn(|conn| providers::begin_delete(conn, &p2.uuid))
        .unwrap();

    // Both keys still present, both rows `deleting`.
    assert!(keystore_has_key(f.keystore_dir(), &f.profile.secret_ref));
    assert!(keystore_has_key(f.keystore_dir(), &p2.secret_ref));

    // One sweep finalizes both.
    resume(&f, 2);

    assert!(!keystore_has_key(f.keystore_dir(), &f.profile.secret_ref));
    assert!(!keystore_has_key(f.keystore_dir(), &p2.secret_ref));
    let statuses = f.db.with_conn(|conn| {
        let all = providers::list_all(conn)?;
        Ok(all
            .iter()
            .map(|p| (p.uuid.clone(), p.status.clone()))
            .collect::<Vec<_>>())
    }).unwrap();
    for (_u, s) in &statuses {
        assert_eq!(s, ProviderStatus::Deleted.as_str(), "every row tombstoned");
    }
    // A redundant sweep finds nothing.
    resume(&f, 0);
}
