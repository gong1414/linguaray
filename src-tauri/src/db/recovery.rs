//! Archive/recovery core (S2a P1) — the shared close → rename → reopen → migrate
//! pipeline used by the [`archive_database`](crate::archive_database) Tauri command
//! (production) AND by the recovery failpoint tests.
//!
//! ## Why a separate core
//!
//! The production `archive_database` is a `#[tauri::command]` that takes a
//! `tauri::State<'_, Arc<AppState>>`. Tests can't construct that without standing
//! up a full Tauri app. To make the close/rename/reopen/migrate pipeline testable
//! in isolation, the logic lives here in [`archive_database_core`], which takes a
//! bare `&Arc<AppState>` and an [`ArchiveFailpoint`]. The Tauri command is a thin
//! wrapper that calls the core with [`ArchiveFailpoint::None`].
//!
//! ## Failpoints
//!
//! [`ArchiveFailpoint`] injects a deterministic error at one of the destructive
//! steps so tests can assert the slot + readiness + on-disk state after each
//! failure mode. Each failpoint documents the recovery contract it must uphold:
//! a failure BEFORE the rename leaves the original DB usable; a failure AFTER the
//! rename leaves the slot `None` and readiness `NeedsDatabaseRecovery`.

use std::path::PathBuf;
use std::sync::Arc;

use crate::db::migration::{run_migration, FailpointCell, MigrationError};
use crate::db::readiness::DataReadiness;
use crate::db::Database;
use crate::AppState;

/// A deterministic injection point for the archive/recovery pipeline. Production
/// passes [`ArchiveFailpoint::None`]; tests pass a specific failpoint to exercise
/// a failure mode and assert the resulting slot/readiness/file state.
///
/// `Clone + PartialEq + Eq` so tests can parameterize over the full set and
/// compare outcomes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ArchiveFailpoint {
    /// No injected failure — the production path.
    None,
    /// Simulate `Database::close` failure. The DB object is recovered and
    /// re-installed in the slot; the original file is untouched; readiness stays
    /// what the caller set before invoking the core. Contract: failure BEFORE
    /// the rename, original DB still usable.
    CloseError,
    /// Simulate `fs::rename` failure. The original DB file is still at
    /// `db_path`; the slot is restored by reopening it (the original file is
    /// usable). Contract: failure BEFORE the rename is committed, original DB
    /// still usable.
    RenameError,
    /// Simulate `Database::open` failure on the fresh DB (after the rename
    /// succeeded). The file was renamed away and the reopen failed too, so
    /// there's nothing to serve — readiness becomes `NeedsDatabaseRecovery` and
    /// the slot is `None`. Contract: failure AFTER the rename, slot `None`.
    ReopenError,
    /// The migration returns `NeedsKeystoreRecovery` (a corrupt keystore at
    /// migration time). The fresh DB is installed in the slot so a later retry
    /// can proceed; readiness becomes `NeedsKeystoreRecovery`. Contract: failure
    /// AFTER the rename, slot `Some`, fresh (possibly partially-migrated) DB.
    MigrationKeystoreCorrupt,
    /// The migration returns some other error. The fresh DB is installed in the
    /// slot so a later recovery/migration retry can proceed; readiness becomes
    /// `MigrationIncomplete`. Contract: failure AFTER the rename, slot `Some`.
    MigrationOther,
}

/// A `Cell`-shaped wrapper retained for symmetry with [`FailpointCell`] (the
/// migration failpoint). `archive_database_core` takes an [`ArchiveFailpoint`]
/// by value, so this is currently unused at runtime — but it documents the
/// intended future shape (an injectable, resettable failpoint for an in-process
/// Tauri command) and keeps the public surface stable if/when the core grows a
/// settable variant.
#[doc(hidden)]
pub struct ArchiveFailpointCell(std::sync::Mutex<ArchiveFailpoint>);

impl ArchiveFailpointCell {
    /// No failpoint — production default.
    pub fn none() -> Self {
        Self(std::sync::Mutex::new(ArchiveFailpoint::None))
    }
    #[doc(hidden)]
    pub fn set(&self, fp: ArchiveFailpoint) {
        *self.0.lock().unwrap() = fp;
    }
}

/// The shared archive/recovery core (S2a P1).
///
/// Pipeline (mirrors the production contract documented on the `archive_database`
/// Tauri command):
/// 1. PREFLIGHT: resolve `settings_path` BEFORE any destructive op. If it's
///    `None`, refuse immediately — the DB is untouched and usable.
/// 2. Acquire `data_gate.write()` (blocks every provider command).
/// 3. Take the `Arc<Database>` out of the slot; `Arc::try_unwrap` + `close`.
/// 4. `fs::rename(db_path, broken_path)`.
/// 5. Open a fresh DB at `db_path` + run migration + resume deletions.
/// 6. Install the new handle + `Ready`.
///
/// `afp` injects a deterministic failure at the matching step (see
/// [`ArchiveFailpoint`]). Production callers pass [`ArchiveFailpoint::None`].
///
/// Any failure AFTER the rename leaves the slot `None` (or, for migration
/// failures, `Some` fresh DB) and a non-`Ready` readiness — the caller surfaces
/// the recovery banner. Any failure BEFORE the rename leaves the original DB
/// untouched and usable.
pub fn archive_database_core(
    app: &Arc<AppState>,
    afp: ArchiveFailpoint,
) -> Result<String, String> {
    // ── 1. PREFLIGHT: settings_path must be resolved before any destructive op.
    //    `settings_path = None` means the canonical settings path couldn't be
    //    resolved at startup. Migration reads + backs up the legacy settings
    //    file, so running it against a guessed path would touch the wrong file
    //    (on Windows the store plugin targets AppData (Roaming) while `dir` is
    //    AppLocalData (Local)). Refuse UP FRONT so we never close/rename a
    //    working DB only to discover we can't migrate it.
    let settings_path: PathBuf = match app.settings_path.as_ref() {
        Some(p) => p.clone(),
        None => return Err("cannot archive: settings path unresolved".into()),
    };

    // ── 2. data_gate write guard for the whole operation. Acquired AFTER the
    //    preflight (so a preflight refusal doesn't block provider commands) but
    //    BEFORE any destructive op. Once we hold the write guard no provider
    //    command can start a new with_conn.
    let _gate = app.data_gate.write();

    let db_path = app.db_path.clone();

    // ── 3. Close the existing connection (if any) so the file handle is
    //      released before the rename. The slot is left None across the rename
    //      so a concurrent reader observes "no DB" rather than a handle pointing
    //      at a renamed file.
    let closed = if afp == ArchiveFailpoint::CloseError {
        // Simulate close failure. If a live handle exists, recover it and
        // re-install it so the original DB keeps serving; otherwise there's
        // nothing to close. Either way we bail before the rename — the original
        // DB is untouched and usable.
        //
        // The handle is taken out in its OWN statement so the `db` write guard
        // is dropped before any re-acquire. (The naive
        // `if let Some(arc) = app.db.write().take()` shape keeps the scrutinee's
        // write guard alive for the whole body, and re-acquiring `app.db.write()`
        // inside would self-deadlock — parking_lot write guards are not
        // reentrant. The real close/rename arms below observe the same rule.)
        let taken = app.db.write().take();
        if let Some(arc) = taken {
            match Arc::try_unwrap(arc) {
                Ok(owned) => {
                    *app.db.write() = Some(Arc::new(owned));
                }
                Err(arc) => {
                    *app.db.write() = Some(arc);
                }
            }
        }
        return Err("close linguaray.db before archive: simulated close failure".into());
    } else {
        // Take the handle out in its own statement (guard released at the `;`),
        // THEN close/recover. The close-failure + handle-still-shared recovery
        // arms re-acquire `app.db.write()` to re-install, which would self-
        // deadlock if the take-guard were still held.
        let taken = app.db.write().take();
        match taken {
            Some(arc) => match Arc::try_unwrap(arc) {
                Ok(owned) => match owned.close() {
                    Ok(()) => true,
                    Err((recovered_db, e)) => {
                        // Close failed — the file handle may still be open but
                        // the DB object is still usable. Restore the slot so the
                        // app keeps running against the original file and bail.
                        let reason = e.to_string();
                        *app.db.write() = Some(Arc::new(recovered_db));
                        return Err(format!("close linguaray.db before archive: {reason}"));
                    }
                },
                Err(arc) => {
                    // Someone still holds a clone. The write gate should make
                    // this impossible; restore + bail rather than risk a
                    // split-brain handle racing the rename.
                    *app.db.write() = Some(arc);
                    return Err(
                        "archive_database: DB handle still in use (data_gate violated)".into(),
                    );
                }
            },
            None => false, // No handle to close (e.g. NeedsDatabaseRecovery).
        }
    };

    // ── 4. Archive the DB file aside (recoverable). If the file doesn't exist,
    //      there's nothing to archive — proceed straight to opening a fresh
    //      one. The archive is published with atomic no-clobber semantics
    //      (create_new + retry on collision) so a prior recoverable archive is
    //      NEVER overwritten by a same-nanosecond collision — the old `rename`
    //      silently clobbered on Unix. On archive failure the original file is
    //      still at db_path, so we can restore the slot by reopening it.
    let archived_path = if afp == ArchiveFailpoint::RenameError {
        // Simulate rename failure WITHOUT moving the file. Restore the slot by
        // reopening the original (only if we previously held a handle) so the
        // app keeps running against the untouched original.
        if closed {
            if let Ok(db) = Database::open(&db_path) {
                *app.db.write() = Some(Arc::new(db));
            } else {
                *app.readiness.write() = DataReadiness::NeedsDatabaseRecovery {
                    reason: format!(
                        "simulated rename failure and reopen also failed for {}",
                        db_path.display()
                    ),
                };
            }
        }
        return Err("rename linguaray.db aside: simulated rename failure".into());
    } else if db_path.exists() {
        // Nanosecond-precision suffix so two archives taken within the same
        // second are unlikely to collide; on a rare same-nanosecond collision
        // the atomic publish retries with a counter suffix so a prior archive
        // is preserved (the old second-only suffix + rename would silently
        // overwrite the first).
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default();
        let dst = db_path.with_extension(format!(
            "db.broken-{}-{}",
            now.as_secs(),
            now.subsec_nanos()
        ));
        // Atomic no-clobber copy: archive is created + secured + fsynced via
        // create_new; the canonical file is removed only AFTER the archive is
        // durable. On failure the original file is still at db_path.
        match crate::fs_acl::atomic_archive_no_clobber(&db_path, &dst) {
            Ok(written) => {
                // Archive durable — remove the canonical file so a fresh open
                // at db_path starts clean. A remove failure here leaves both
                // the archive AND the original; reopen the original so the app
                // keeps serving, and surface the error.
                if let Err(e) = std::fs::remove_file(&db_path) {
                    if closed {
                        if let Ok(db) = Database::open(&db_path) {
                            *app.db.write() = Some(Arc::new(db));
                        } else {
                            *app.readiness.write() = DataReadiness::NeedsDatabaseRecovery {
                                reason: format!(
                                    "rename {} failed ({e}) and reopen also failed",
                                    db_path.display()
                                ),
                            };
                        }
                    }
                    return Err(format!("rename linguaray.db aside: {e}"));
                }
                written.to_string_lossy().into_owned()
            }
            Err(e) => {
                // Archive failed: the original file is still at the original
                // path. Restore the slot by reopening it (only if we previously
                // held a handle, i.e. the app was using this DB) so the user
                // isn't left with a None slot over a usable file.
                if closed {
                    if let Ok(db) = Database::open(&db_path) {
                        *app.db.write() = Some(Arc::new(db));
                    } else {
                        *app.readiness.write() = DataReadiness::NeedsDatabaseRecovery {
                            reason: format!(
                                "rename {} failed ({e}) and reopen also failed",
                                db_path.display()
                            ),
                        };
                    }
                }
                return Err(format!("rename linguaray.db aside: {e}"));
            }
        }
    } else {
        String::new()
    };

    // ── 5. Open a fresh DB at the original path + migrate + resume.
    if afp == ArchiveFailpoint::ReopenError {
        // Simulate reopen failure. The file was renamed away; the reopen fails
        // too. There's nothing to serve — NeedsDatabaseRecovery, slot None.
        *app.readiness.write() = DataReadiness::NeedsDatabaseRecovery {
            reason: "reopen linguaray.db: simulated reopen failure".into(),
        };
        return Err("reopen linguaray.db: simulated reopen failure".into());
    }
    match Database::open(&db_path) {
        Ok(db) => {
            let db = Arc::new(db);
            // The migration failpoints (MigrationKeystoreCorrupt / MigrationOther)
            // inject a DETERMINISTIC synthetic error at the migration step — they
            // do NOT run the real migration. This keeps the failpoint tests
            // independent of on-disk keystore/settings state (the migration
            // outcome is the thing under test, not the migration itself, which is
            // covered by tests/migration.rs). Production (afp=None) and the
            // close/rename/reopen failpoints run the REAL migration.
            let migration_result: Result<(), MigrationError> = match afp {
                ArchiveFailpoint::MigrationKeystoreCorrupt => Err(
                    MigrationError::NeedsKeystoreRecovery(
                        "simulated keystore corruption at migration".into(),
                    ),
                ),
                ArchiveFailpoint::MigrationOther => Err(MigrationError::Other(
                    "simulated migration failure".into(),
                )),
                _ => {
                    let fp = FailpointCell::none();
                    run_migration(&db, &app.keystore_dir, &settings_path, &fp)
                }
            };
            match migration_result {
                Ok(()) => {}
                Err(MigrationError::NeedsKeystoreRecovery(reason)) => {
                    // A keystore-needs-recovery outcome must surface as
                    // NeedsKeystoreRecovery (NOT MigrationIncomplete) so the
                    // recovery banner routes the user to the keystore flow
                    // rather than the migration flow.
                    let msg = reason.clone();
                    *app.readiness.write() = DataReadiness::NeedsKeystoreRecovery {
                        reason: msg.clone(),
                    };
                    // Still install the handle so a later retry can proceed.
                    *app.db.write() = Some(db);
                    return Err(msg);
                }
                Err(e) => {
                    let reason = format!("migration after archive: {e}");
                    *app.readiness.write() = DataReadiness::migration_incomplete(
                        "archive_database",
                        reason.clone(),
                    );
                    // Still install the handle so a later recovery/migration
                    // retry can proceed (NeedsDatabaseRecovery would hide the
                    // partially-migrated DB).
                    *app.db.write() = Some(db);
                    return Err(reason);
                }
            }
            // Resume in-flight deletes against the fresh DB. A failure here is
            // a real consistency problem — surface MigrationIncomplete (NOT
            // Ready) so the user sees the recovery banner.
            if let Err(e) = crate::db::delete::provider_resume_deletions(&db, &app.keystore_dir) {
                let reason = format!("resume_deletions after archive: {e}");
                *app.readiness.write() = DataReadiness::migration_incomplete(
                    "archive_database",
                    reason.clone(),
                );
                *app.db.write() = Some(db);
                return Err(reason);
            }
            *app.db.write() = Some(db);
            *app.readiness.write() = DataReadiness::Ready;
            Ok(archived_path)
        }
        Err(e) => {
            // The file was renamed away (or never existed); the reopen failed
            // too. There's nothing to serve — NeedsDatabaseRecovery.
            let reason = format!("reopen linguaray.db: {e}");
            *app.readiness.write() = DataReadiness::NeedsDatabaseRecovery {
                reason: reason.clone(),
            };
            Err(reason)
        }
    }
}

#[cfg(test)]
mod tests {
    // The integration coverage for archive_database_core lives in
    // tests/recovery.rs (it needs the `linguaray_lib` crate's public re-exports
    // + tempfile). This module is kept as an anchor for future unit tests that
    // don't need a temp DB.
}
