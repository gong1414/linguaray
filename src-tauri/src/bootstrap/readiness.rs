//! Startup readiness + hardened builders + recovery cleanup (moved verbatim
//! from `lib.rs` in refactor P3.1).

use std::path::Path;
use std::sync::Arc;

use crate::bootstrap::state::AppState;
use crate::db::readiness::DataReadiness;
use crate::{keystore, providers};

/// Startup readiness reducer (S2a P1.4).
///
/// Computes the pre-migration [`DataReadiness`] from the three independent
/// startup outcomes — DB open, settings-path resolution, keystore init — with a
/// load-bearing **priority** rule: a failed DB open (`NeedsDatabaseRecovery`)
/// is locked in and MUST NOT be masked by a later settings or keystore error.
/// The DB is the foundation; settings/keystore failures only become the visible
/// banner when the DB itself opened successfully.
///
/// Priority order (highest first):
/// 1. **DB failure** → `NeedsDatabaseRecovery`. Nothing else can override this:
///    there is no DB to migrate or to gate provider writes on.
/// 2. **Keystore failure** → `NeedsKeystoreRecovery`. A healthy DB + healthy
///    migration are useless without a usable keystore (provider writes need
///    it), so this beats a settings failure.
/// 3. **Settings failure** → `MigrationIncomplete` ("settings_path"). Migration
///    reads + backs up the legacy settings, so an unresolvable path must skip
///    migration entirely.
/// 4. **All healthy** → the [`DataReadiness::default`] pre-migration state
///    (`MigrationIncomplete "startup not complete"`). The reducer intentionally
///    does NOT return `Ready`: running the migration is what promotes to Ready,
///    and the `setup()` migration block unconditionally assigns this result.
///
/// This is a pure function so the priority matrix is unit-testable without
/// spinning up Tauri or a real DB. The `setup()` closure in [`crate::run`] calls
/// this reducer then, when healthy, overlays the migration outcome on top.
pub fn compute_startup_readiness(
    db_open: Result<(), String>,
    settings_error: Option<String>,
    keystore_error: Option<String>,
) -> DataReadiness {
    // 1. DB failure locks the readiness. NEVER override.
    if let Err(reason) = db_open {
        return DataReadiness::NeedsDatabaseRecovery { reason };
    }
    // DB opened. keystore failure beats settings failure (writes need a
    // usable keystore, so a healthy DB+migration is useless without one).
    if let Some(reason) = keystore_error {
        return DataReadiness::NeedsKeystoreRecovery { reason };
    }
    // Settings path didn't resolve: migration can't run safely against a
    // guessed path, so degrade to MigrationIncomplete and skip migration.
    if let Some(reason) = settings_error {
        return DataReadiness::migration_incomplete("settings_path", reason);
    }
    // All healthy: pre-migration default. The migration step in setup() is
    // what flips this to Ready (or a more specific failure state).
    DataReadiness::default()
}

/// Startup migration gate (round-3 P1.3): decide whether migration may run,
/// and return the REAL settings path it must run against.
///
/// Migration's Phase 1 parses + backs up the legacy settings file, so it needs
/// the actual resolved path. A `None` settings path (resolution failed) must
/// REFUSE migration — running it against a guessed path would read/write the
/// WRONG settings file (on Windows the store plugin targets AppData Roaming
/// while `dir` here is AppLocalData Local). A failed keystore init is also a
/// hard stop (migration's Phase 1 keystore backup needs a usable keystore).
///
/// Returns `Ok(path)` ONLY when the caller may run `run_migration` against
/// `path`; `Err(reason)` otherwise. The refusal decision is what the tests pin
/// down: when this returns `Err`, `run_migration` is never reached, so NO
/// backup is produced and NO DB write occurs (the readiness reducer above
/// already reflects the refusal in `DataReadiness`).
pub fn startup_migration_guard<'a>(
    settings_path: Option<&'a Path>,
    keystore_init_error: Option<&str>,
) -> Result<&'a Path, String> {
    if let Some(e) = keystore_init_error {
        return Err(format!("keystore init failed: {e}"));
    }
    settings_path.ok_or_else(|| {
        "settings path could not be resolved; migration refused (no backup, no DB write)"
            .to_string()
    })
}

/// Build the translate HTTP client (S2a Task 5b).
///
/// Spec §Privacy: no cross-origin redirects, a 30s total request timeout so a
/// hung connection can't freeze translate + the popup, and a 10s connect
/// timeout so `wire::call` can classify a real `Timeout`. This MUST NOT panic
/// and MUST NOT silently downgrade the privacy policy: the hardened builder
/// (with `redirect(Policy::none())`) is the ONLY client we ever return. On a
/// builder failure (pathological TLS-init environment) we return `Err` so the
/// caller can log it and enter a degraded startup state — a default client
/// would drop `redirect(Policy::none())`, re-opening the cross-origin-redirect
/// leak the policy exists to close.
pub(crate) fn build_http_client() -> Result<reqwest::Client, String> {
    reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .timeout(std::time::Duration::from_secs(30))
        .connect_timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| format!("hardened HTTP client build failed: {e}"))
}

/// Init a last-resort keystore in a PID-uniquified temp subdir (S2a Task 5b).
///
/// Called only when both the canonical-dir keystore AND the shared temp-dir
/// fallback failed to init. A PID-suffixed subdir sidesteps any flock/perm
/// state the earlier attempts left behind, and a second concurrent instance
/// gets its own dir. This MUST NOT panic: it keeps trying uniquified dirs and,
/// if all of them fail (genuinely unreachable — `Keystore::new` only does
/// `create_dir_all` + `set_dir_perms`), returns `Err` so the caller can record
/// the failure and degrade to `NeedsKeystoreRecovery`. The OS temp allocator is
/// effectively always writable, so the `Err` path is defence-in-depth.
pub(crate) fn init_last_resort_keystore() -> Result<keystore::Keystore, String> {
    let pid = std::process::id();
    // Try up to 17 PID-suffixed temp subdirs. Each attempt logs on failure; the
    // loop returns Ok on the first success. If all 17 fail (genuinely
    // unreachable — the OS temp allocator is effectively always writable), make
    // one final attempt with a random name and surface the error.
    let mut last_err = String::new();
    for suffix in 0..=16 {
        let candidate =
            std::env::temp_dir().join(format!("linguaray-keystore-lastresort-{pid}-{suffix}"));
        match keystore::Keystore::new(candidate) {
            Ok(ks) => return Ok(ks),
            Err(e) => {
                log::warn!("last-resort keystore dir {suffix} failed: {e}");
                last_err = e.to_string();
            }
        }
    }
    // Defence-in-depth: all PID-suffixed dirs failed. Construct one final
    // attempt with a random-ish name, then surface the error (no panic).
    let nano = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let final_dir = std::env::temp_dir().join(format!("linguaray-keystore-{pid}-{nano}"));
    log::error!(
        "all last-resort keystore dirs failed; final attempt: {}",
        final_dir.display()
    );
    match keystore::Keystore::new(final_dir) {
        Ok(ks) => Ok(ks),
        Err(e) => Err(format!(
            "cannot init any keystore (last error: {e}; prior loop: {last_err})"
        )),
    }
}

/// Validate every shipped preset endpoint (S2a Task 5b, spec §Privacy).
///
/// Thin wrapper over [`validate_preset_endpoints`] reading the hardcoded
/// catalog. Kept separate so the core validation is injectable in tests (a test
/// catalog containing a deliberately-invalid endpoint can prove the fail-closed
/// chain end-to-end instead of only exercising `validate_endpoint` in
/// isolation).
pub(crate) fn validate_all_preset_endpoints() -> Vec<String> {
    validate_preset_endpoints(&providers::presets())
}

/// Validate a PRESET LIST's endpoints (spec §Privacy): every endpoint must be
/// HTTPS (loopback HTTP allowed for local engines like Ollama). Returns the ids
/// whose endpoints FAILED scheme validation (empty = all valid).
///
/// Each failure is logged + the offending preset is effectively disabled (its
/// engine won't be usable until the catalog is fixed), but a single bad preset
/// does not crash startup — this is a far better failure mode than refusing to
/// launch. The caller decides the fail-closed response via
/// [`preset_gate_allows_client`].
pub(crate) fn validate_preset_endpoints(list: &[providers::ProviderPreset]) -> Vec<String> {
    let mut invalid = Vec::new();
    for p in list {
        // setup_required rows (Azure / Custom) ship with an empty endpoint.
        // They are uncallable until the user pastes a URL; they must not fail
        // the whole client gate.
        if p.endpoint.is_empty() {
            continue;
        }
        if let Err(e) = providers::validate_endpoint(&p.endpoint) {
            log::error!(
                "preset '{}' endpoint '{}' failed scheme validation ({e}); engine disabled",
                p.id,
                p.endpoint
            );
            invalid.push(p.id.clone());
        }
    }
    invalid
}

/// Fail-closed gate for the HTTP client (round-3 P1.1).
///
/// Given the preset ids whose endpoints failed validation, decide whether ANY
/// outbound request may be shipped. A single invalid preset disables the client
/// ENTIRELY (the setup path builds `None` and every translate entry-point then
/// refuses via `session_client`) — a leaked/broken catalog endpoint must never
/// fire a request just because the user happens to select a different engine.
///
/// This is the deterministic, testable decision behind the setup inline branch:
/// `Ok(c) if preset_gate_allows_client(&invalid) => Some(c)`, else `None`.
pub(crate) fn preset_gate_allows_client(invalid_presets: &[String]) -> bool {
    invalid_presets.is_empty()
}

/// Shared DB cleanup + readiness update for keystore recovery
/// (`archive_keystore` / `reset_keystore`). Review P1 #2.
///
/// Must be called while the caller holds the `data_gate` write guard so no
/// provider command races the cleanup. Performs (inside one DB transaction):
/// 1. `UPDATE providers SET enabled=0 WHERE needs_key=1` — a provider that needs
///    a key can't be used until the user re-enters one (the archived keystore
///    just lost them all). Keyless providers (Ollama, traditional engines) keep
///    their enabled state.
/// 2. Disable encrypted history and delete history/vocabulary ciphertext that
///    can no longer be decrypted after the key archive/reset.
/// 3. Clear `primary_uuid`, `parallel_uuids`, `fallback_uuid` — the prior
///    selection referenced providers whose keys may be gone, so a stale
///    selection can't drive a translate.
/// 4. Clear `parallel_consent_version` / `parallel_consent_scope` — consent was
///    given for the now-archived key set.
/// 5. `UPDATE _schema_migrations SET migration_complete=1` — a recovery completes
///    migration (the DB is now in a known-good state, just without keys).
///    **Guaranteed only when the OLD readiness was `Ready` or
///    `NeedsKeystoreRecovery`.** When the OLD readiness was
///    `MigrationIncomplete`, this UPDATE is SKIPPED: the prior migration did
///    not reach `Complete` for a reason this archive does not address (a
///    half-applied schema, a corrupt settings file, a resume-deletions fault),
///    so writing `migration_complete=1` would persist a half-applied DB as
///    complete and permanently lock it (the next startup sees `Complete` and
///    skips migration).
///
/// Returns `Err` if the cleanup transaction fails (so the caller can surface
/// the failure — previously it was only logged and readiness was bumped to
/// Ready, hiding a real consistency break).
///
/// Readiness transition rules:
/// - DB cleanup tx FAILED → `MigrationIncomplete` (the DB is in an unknown
///   state; the user must see the recovery banner). The OLD readiness is NOT
///   promoted.
/// - Old `Ready` or `NeedsKeystoreRecovery` + cleanup OK + DB exists → `Ready`
///   (the keystore problem is fixed and the DB is consistent).
/// - Old `NeedsDatabaseRecovery` → kept as-is (a keystore archive does NOT fix
///   a corrupt/unopenable DB). The cleanup tx isn't attempted (no DB handle).
/// - Old `MigrationIncomplete` → kept as-is even when the cleanup succeeds. The
///   prior migration was incomplete for a reason this archive doesn't address
///   (a half-applied schema, a corrupt settings file, a resume-deletions
///   fault); promoting to Ready would hide that. The user must retry the
///   recovery that targets the migration itself.
pub(crate) fn apply_keystore_recovery_db_cleanup(app: &Arc<AppState>) -> Result<(), String> {
    // Capture the OLD readiness BEFORE any mutation so the post-state logic can
    // branch on it, and so the cleanup tx knows whether it may mark migration
    // complete (only Ready / NeedsKeystoreRecovery were fully migrated before
    // the archive; MigrationIncomplete must NOT be promoted on disk).
    let old_readiness = app.readiness.read().clone();
    let may_mark_complete = matches!(
        &old_readiness,
        DataReadiness::Ready | DataReadiness::NeedsKeystoreRecovery { .. }
    );

    // Try the DB cleanup only when a DB handle exists. A NeedsDatabaseRecovery
    // state means there's no handle; the cleanup is a no-op and we keep that
    // state (a keystore archive doesn't fix a corrupt DB).
    //
    // `cleanup_result` is Ok(true) when a handle existed and the tx succeeded;
    // Ok(false) when there was no handle to clean up (NeedsDatabaseRecovery);
    // Err(_) when a handle existed but the tx failed (the DB is now in an
    // unknown state).
    let cleanup_result: Result<bool, String> = match app.db.read().clone() {
        Some(db) => db
            .with_conn(|conn| {
                let tx = conn.transaction()?;
                // 1. Disable needs-key providers (their keys are gone after archive).
                tx.execute("UPDATE providers SET enabled=0 WHERE needs_key=1", [])?;
                // 2. The history/vocabulary key was just archived. Keeping
                // undecryptable ciphertext would strand private data while the
                // UI claims recovery succeeded, so clear it atomically and
                // revoke future history consent.
                tx.execute("DELETE FROM history_sessions", [])?;
                tx.execute("DELETE FROM vocabulary", [])?;
                tx.execute("UPDATE preferences SET history_enabled=0 WHERE id=1", [])?;
                // 3-4. Clear active selection + consent.
                tx.execute(
                    "UPDATE preferences SET primary_uuid=NULL, parallel_uuids='[]', \
                     fallback_uuid=NULL, parallel_consent_version=NULL, \
                     parallel_consent_scope=NULL WHERE id=1",
                    [],
                )?;
                // 4. Mark migration complete — ONLY when the OLD readiness was
                //    Ready / NeedsKeystoreRecovery. When it was
                //    MigrationIncomplete, the prior migration did not reach
                //    Complete and this archive doesn't fix that; writing
                //    complete=1 would persist the half-migrated DB as final and
                //    the next startup would skip migration entirely.
                if may_mark_complete {
                    tx.execute(
                        "UPDATE _schema_migrations SET migration_complete=1 WHERE id=1",
                        [],
                    )?;
                }
                tx.commit()?;
                Ok(())
            })
            .map(|_| true)
            .map_err(|e| {
                // Surface the failure: the cleanup tx is part of the recovery's
                // atomic contract; logging + bumping to Ready would hide a real
                // consistency break.
                log::error!("keystore recovery DB cleanup failed: {e}");
                e.to_string()
            }),
        None => Ok(false), // No DB handle (NeedsDatabaseRecovery): nothing to clean up.
    };

    // Compute the new readiness. The cleanup tx FAILED → MigrationIncomplete
    // (the DB is in an unknown state). On success, the transition depends on
    // the OLD readiness (see the doc comment above).
    let new_readiness = match cleanup_result {
        Err(_) => DataReadiness::migration_incomplete(
            "keystore_recovery_cleanup",
            "DB cleanup transaction failed after keystore archive",
        ),
        // No DB handle (NeedsDatabaseRecovery): keep that state regardless of
        // whether the archive succeeded.
        Ok(false) => match &old_readiness {
            DataReadiness::NeedsDatabaseRecovery { reason } => {
                DataReadiness::NeedsDatabaseRecovery {
                    reason: reason.clone(),
                }
            }
            // Defensive: cleanup is a no-op but there's no DB handle, so do NOT
            // claim Ready (Ready implies an open DB). Keep the old state.
            other => other.clone(),
        },
        // Cleanup succeeded against a real DB. Only Ready / NeedsKeystoreRecovery
        // promote to Ready; MigrationIncomplete is kept (the prior migration was
        // incomplete for a reason this archive doesn't fix); NeedsDatabaseRecovery
        // can't reach this arm (no handle → Ok(false)).
        Ok(true) => match &old_readiness {
            DataReadiness::Ready | DataReadiness::NeedsKeystoreRecovery { .. } => {
                DataReadiness::Ready
            }
            DataReadiness::MigrationIncomplete { checkpoint, reason } => {
                DataReadiness::MigrationIncomplete {
                    checkpoint: checkpoint.clone(),
                    reason: reason.clone(),
                }
            }
            DataReadiness::NeedsDatabaseRecovery { reason } => {
                DataReadiness::NeedsDatabaseRecovery {
                    reason: reason.clone(),
                }
            }
        },
    };
    *app.readiness.write() = new_readiness;
    // Propagate a cleanup-tx failure so the caller (archive_keystore /
    // reset_keystore) surfaces it to the frontend; the readiness has already
    // been set to MigrationIncomplete above.
    cleanup_result.map(|_| ())
}
