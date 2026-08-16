//! Delete state machine — crash-safe resume sweep (S2a step 5).
//!
//! The 3-step delete flow lives in [`super::providers`]:
//! 1. `begin_delete`  — DB tx: `status='deleting'`, `enabled=0`, evict from slots,
//!    invalidate consent. Returns `secret_ref`. (committed to disk)
//! 2. keystore remove — purge the key named by `secret_ref` from the keystore
//!    (idempotent — a no-op if the key is already gone). (committed to disk)
//! 3. `finalize_delete`— DB tx: `status='deleted'`, name rewritten to
//!    `deleted: <orig>`. (committed to disk — the tombstone)
//!
//! A crash after step 1 (DB tx committed, key still present) or step 2 (key
//! removed, status still `deleting`) leaves the row mid-flight. This module's
//! [`provider_resume_deletions`] is the startup sweep that finishes every
//! in-flight delete: for each `status='deleting'` row it removes the key
//! (idempotent) then finalizes the tombstone. Because each step is committed to
//! disk before the next, replay is always safe and forward-only.
//!
//! ## Lock-order (load-bearing — see [`super`] module docs)
//!
//! Resume is a cross-store operation but the two locks (DB Mutex, keystore flock)
//! are NEVER held simultaneously:
//! 1. Lock DB → read the list of `status='deleting'` rows (uuid, secret_ref) →
//!    **unlock DB**.
//! 2. For each row: `Keystore::update_keys` (keystore flock only, DB NOT locked).
//! 3. For each row: `finalize_delete` (DB Mutex only, keystore NOT locked).
//!
//! There is no deadlock window because the two locks are never nested. A crash
//! at any point leaves the system in a state the next resume can finish.

use std::path::Path;

use crate::db::providers::finalize_delete;
use crate::db::{Database, DbError};
use crate::keystore::{Keystore, KeystoreError};

/// One row of the deleting-provider snapshot read out of the DB under the DB
/// Mutex, then released. The keystore + finalize steps run with the DB unlocked.
struct DeletingRow {
    uuid: String,
    secret_ref: String,
}

/// Snapshot every `status='deleting'` provider (uuid + secret_ref). Run under
/// the DB Mutex; the Vec is owned so the Mutex is released before the keystore
/// step. Returns rows in deterministic `uuid` order so a replay is stable.
fn snapshot_deleting(db: &Database) -> Result<Vec<DeletingRow>, DbError> {
    db.with_conn(|conn| {
        let mut stmt = conn.prepare(
            "SELECT uuid, secret_ref FROM providers WHERE status='deleting' ORDER BY uuid ASC",
        )?;
        let rows = stmt.query_map([], |r| {
            Ok(DeletingRow {
                uuid: r.get::<_, String>(0)?,
                secret_ref: r.get::<_, String>(1)?,
            })
        })?;
        let mut out = Vec::new();
        for r in rows {
            out.push(r?);
        }
        Ok(out)
    })
}

/// Remove `secret_ref` from a decrypted keystore payload, mutating it in place.
/// Idempotent: a no-op if the key is already absent (the post-step-2 crash
/// state). Handles both on-disk shapes:
/// - **v2** (`KeystoreData`): the keys live in the nested `provider_keys` map.
/// - **v1** (legacy flat map): the keys ARE the top-level object.
///
/// A non-object payload (empty/missing keystore → `{}`) has nothing to remove.
/// Shared between the production `update_keys` path and the `update_keys_with_identity`
/// test seam so the shape logic cannot drift.
fn remove_provider_key_mut(keys: &mut serde_json::Value, secret_ref: &str) {
    let Some(obj) = keys.as_object_mut() else {
        return; // non-object ({}/missing) → nothing to remove
    };
    // v2: { "version": 2, "provider_keys": { "<secret_ref>": "..." } }
    if obj.contains_key("provider_keys") {
        if let Some(inner) = obj
            .get_mut("provider_keys")
            .and_then(|v| v.as_object_mut())
        {
            inner.remove(secret_ref);
        }
        return;
    }
    // v1 legacy flat map: { "<secret_ref>": "..." }
    obj.remove(secret_ref);
}

/// Production key removal via the sanctioned [`Keystore::update_keys`] RMW
/// (reads the real OS identity via `IdentitySource::CURRENT`).
fn remove_key_from_payload(ks: &Keystore, secret_ref: &str) -> Result<(), KeystoreError> {
    ks.update_keys(|keys| remove_provider_key_mut(keys, secret_ref))
}

/// THE resume core, shared by the production path (machine identity) and the
/// test seam (injected identity). See the module docs for the lock-order and
/// crash-safety argument.
///
/// `ks_ops` abstracts the keystore key-removal so the test seam can drive it
/// with an injected identity (the production path reads the real OS identity
/// inside `Keystore::update_keys`).
fn resume_core(
    db: &Database,
    ks: &Keystore,
    ks_ops: &dyn ResumeKeystoreOps,
) -> Result<usize, DbError> {
    // 1. Snapshot deleting rows (DB locked → released).
    let rows = snapshot_deleting(db)?;

    // 2. + 3. For each row: keystore remove (DB unlocked) → finalize (keystore
    //    unlocked). Each iteration is independent and idempotent, so a crash
    //    mid-loop leaves a consistent subset that the next resume finishes.
    let mut finalized = 0usize;
    for row in &rows {
        // 2. Keystore key removal. Idempotent — if the key is already gone
        //    (post-step-2 crash) this is a successful no-op. A keystore error
        //    here is fatal to THIS row: we skip the finalize so the row stays
        //    `deleting` and the next startup retry can attempt the removal again.
        ks_ops.remove_key(ks, &row.secret_ref).map_err(keystore_err_to_db)?;

        // 3. Finalize the tombstone. Only run if the keystore step succeeded so
        //    we never tombstone a row whose key is still live.
        db.with_conn(|conn| finalize_delete(conn, &row.uuid))?;
        finalized += 1;
    }
    Ok(finalized)
}

/// Trait that isolates the one keystore operation resume performs, so the
/// production path (machine identity) and the test seam (injected identity)
/// share the SAME core loop (no duplicated snapshot/remove/finalize logic to
/// drift). Mirrors the `KeystoreIo` seam in `migration.rs`.
trait ResumeKeystoreOps {
    fn remove_key(&self, ks: &Keystore, secret_ref: &str) -> Result<(), KeystoreError>;
}

/// Production keystore ops: uses `Keystore::update_keys` (reads the real OS
/// identity via `IdentitySource::CURRENT`).
struct MachineResumeOps;

impl ResumeKeystoreOps for MachineResumeOps {
    fn remove_key(&self, ks: &Keystore, secret_ref: &str) -> Result<(), KeystoreError> {
        remove_key_from_payload(ks, secret_ref)
    }
}

/// Test seam: uses `Keystore::update_keys_with_identity` so tests drive the
/// sanctioned RMW with an injected identity instead of touching real OS
/// identity. Delegates to the SAME `update_keys_core` as the production path.
struct IdentityResumeOps {
    identity: String,
}

impl IdentityResumeOps {
    fn new(identity: &str) -> Self {
        Self { identity: identity.to_string() }
    }
}

impl ResumeKeystoreOps for IdentityResumeOps {
    fn remove_key(&self, ks: &Keystore, secret_ref: &str) -> Result<(), KeystoreError> {
        ks.update_keys_with_identity(
            |keys| remove_provider_key_mut(keys, secret_ref),
            &self.identity,
        )
    }
}

/// Map a [`KeystoreError`] into a [`DbError`]. Keystore failures are surfaced as
/// `Integrity` (the DB row is fine, but the cross-store invariant is broken) so
/// callers see a domain error rather than a raw IO/crypto string.
fn keystore_err_to_db(e: KeystoreError) -> DbError {
    DbError::Integrity(format!("keystore error during resume: {e}"))
}

// ─── Public entry points ──────────────────────────────────────────────────

/// Startup sweep: finish every in-flight (`status='deleting'`) provider delete.
///
/// Production entry point. Internally creates a [`Keystore`] for `keystore_dir`
/// and uses the machine identity for keystore operations. Returns the number of
/// deletes finalized (0 if there was nothing to resume).
///
/// See the module docs for the 3-step model, the lock-order rule, and the
/// crash-safety argument. Safe to call on every startup; a no-op when there are
/// no `deleting` rows.
pub fn provider_resume_deletions(
    db: &Database,
    keystore_dir: &Path,
) -> Result<usize, DbError> {
    let ks = Keystore::new(keystore_dir.to_path_buf()).map_err(keystore_err_to_db)?;
    resume_core(db, &ks, &MachineResumeOps)
}

/// Test-only: same as [`provider_resume_deletions`] but the keystore RMW uses
/// an injected identity string instead of reading the OS identity. Lets the
/// fault-injection tests drive the sanctioned `update_keys` path without
/// touching real machine identity. Delegates to the SAME `resume_core` as
/// production (no duplicated snapshot/remove/finalize logic to drift).
#[doc(hidden)]
pub fn provider_resume_deletions_with_identity(
    db: &Database,
    keystore_dir: &Path,
    identity: &str,
) -> Result<usize, DbError> {
    let ks = Keystore::new(keystore_dir.to_path_buf()).map_err(keystore_err_to_db)?;
    resume_core(db, &ks, &IdentityResumeOps::new(identity))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `remove_key_from_payload` is idempotent: removing an absent key from an
    /// empty object is a no-op (the post-step-2 crash state where the key was
    /// already purged). This guards the resume core's "no error on already-absent
    /// key" contract at the unit level.
    #[test]
    fn remove_key_from_empty_keystore_is_noop() {
        let dir = tempfile::tempdir().unwrap();
        let ks = Keystore::new(dir.path().to_path_buf()).unwrap();
        // No keystore file → load returns {} → remove is a no-op → Ok.
        remove_key_from_payload(&ks, "provider/missing").unwrap();
    }
}
