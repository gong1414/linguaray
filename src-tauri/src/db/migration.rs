//! Migration coordinator (S2a step 4) — the 5-phase crash-safe idempotent
//! migration from pre-2a state (legacy keystore + legacy settings.json) to the
//! 2a SQLite schema + v2 keystore.
//!
//! ## Design (frozen S0 §8.5 / plan rev-6 §1.4)
//!
//! The coordinator is split into 5 independent phases. Crucially there is NO
//! outer `db.with_conn` wrapping the whole migration — each phase that touches
//! the DB is a short transaction, so a crash can never leave a long-lived
//! DB-mutex / keystore-flock deadlock window (see `db/mod.rs` lock-order rule).
//!
//! ```text
//! PREFLIGHT  migration_state(db)?
//!            Complete      → preflight_keystore(ks_dir)? → Ok
//!            Incomplete    → proceed (replay from where it left off)
//!            NotStarted    → proceed
//!
//! PHASE 1  Backup (FIRST persistent action, per frozen S0 — backup BEFORE schema)
//!            parse settings.json (raw)
//!            backup settings.json → settings.json.bak-pre-migration (create-new)
//!            backup keystore (Missing/CurrentV2 → no-op; LegacyV1 → backup; Corrupt → Err)
//!            fp.maybe_fail(AfterBackup)?
//!
//! PHASE 2  DB schema (idempotent, short tx)
//!            create_all_tables + seed_singletons
//!            fp.maybe_fail(AfterSchema)?
//!
//! PHASE 2b Seed preferences from settings (short tx)
//!            fp.maybe_fail(AfterPreferences)?
//!
//! PHASE 3  Enumerate candidates + insert profiles (DB NOT locked during keystore read)
//!            load keystore state
//!            enumerate candidates (BTreeSet, deterministic)
//!            for each candidate: insert_or_ignore + commit
//!              fp.maybe_fail(AfterProfileInsert(source))?
//!            seed selection (write-guarded)
//!            fp.maybe_fail(AfterProfiles)?
//!
//! PHASE 4  Keystore rewrite (keystore flock only, DB NOT locked)
//!            if LegacyV1: migrate_to_v2
//!            fp.maybe_fail(AfterKeystoreRewrite)?
//!
//! PHASE 5  Verify + complete (short tx)
//!            load keystore state (DB NOT locked)
//!            verify key-bearing profiles
//!            set_migration_complete
//!            fp.maybe_fail(AfterCompleteCommit)?
//! ```
//!
//! Idempotency: every DB write is `INSERT OR IGNORE` / guarded UPDATE; the
//! keystore rewrite is atomic-replace; backups are create-new. Re-running the
//! coordinator after a crash picks up where it left off and converges to the
//! complete state.
//!
//! ## Testing seam
//!
//! Production calls [`run_migration`] (reads the real machine identity for the
//! keystore). Tests call [`run_migration_with_identity`], which drives the SAME
//! coordinator core through the `*_with_identity` keystore seams (an injected
//! identity string instead of the OS identity). Both paths share one
//! [`run_migration_core`] so the logic cannot drift.

use std::path::Path;

use parking_lot::Mutex;
use rusqlite::Connection;

use crate::db::providers::{
    build_profile, enumerate_candidates, insert_or_ignore, CandidateSource, RawSettings,
};
use crate::db::schema::{
    self, migration_state_if_exists, set_migration_complete, MigrationState,
};
use crate::db::{Database, DbError};
use crate::keystore::{
    self, backup_keystore, migrate_to_v2, KeystoreError, KeystoreLoadState,
};

// ─── MigrationError ───────────────────────────────────────────────────────

/// Every failure mode the migration coordinator can surface. A test-injected
/// failpoint returns [`MigrationError::InjectedFail`] carrying the matched
/// [`Failpoint`]; everything else wraps a domain error with context.
#[derive(Debug)]
pub enum MigrationError {
    Db(DbError),
    Keystore(KeystoreError),
    /// `settings.json` exists but couldn't be parsed as JSON / expected shape.
    SettingsCorrupt(String),
    /// The keystore is unreadable/corrupt and needs the user's explicit recovery
    /// (archive + re-enter) — migration cannot proceed automatically.
    NeedsKeystoreRecovery(String),
    /// Test-injected failpoint fired at `point`.
    InjectedFail(Failpoint),
    /// A backup step (settings or keystore) failed for an IO reason.
    BackupFailed(String),
    /// Catch-all for unexpected failures with a human-readable context string.
    Other(String),
}

impl std::fmt::Display for MigrationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MigrationError::Db(e) => write!(f, "db: {e}"),
            MigrationError::Keystore(e) => write!(f, "keystore: {e}"),
            MigrationError::SettingsCorrupt(s) => write!(f, "settings corrupt: {s}"),
            MigrationError::NeedsKeystoreRecovery(s) => {
                write!(f, "keystore needs recovery: {s}")
            }
            MigrationError::InjectedFail(fp) => write!(f, "injected failpoint: {fp:?}"),
            MigrationError::BackupFailed(s) => write!(f, "backup failed: {s}"),
            MigrationError::Other(s) => write!(f, "{s}"),
        }
    }
}

impl std::error::Error for MigrationError {}

impl From<DbError> for MigrationError {
    fn from(e: DbError) -> Self {
        MigrationError::Db(e)
    }
}

impl From<KeystoreError> for MigrationError {
    fn from(e: KeystoreError) -> Self {
        MigrationError::Keystore(e)
    }
}

// ─── Failpoint + FailpointCell ───────────────────────────────────────────

/// A checkpoint where a test can ask the coordinator to crash (return
/// `Err(InjectedFail(point))`) AFTER persisting the phase's state.
///
/// `AfterProfileInsert(String)` carries the `stable_id` of the candidate whose
/// insert the failpoint follows; equality is FULL (value, not discriminant), so
/// `AfterProfileInsert("openai")` only fires after the openai candidate.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Failpoint {
    None,
    AfterBackup,
    AfterSchema,
    AfterPreferences,
    /// Fires after the candidate with this `stable_id` is inserted.
    AfterProfileInsert(String),
    AfterProfiles,
    AfterKeystoreRewrite,
    AfterCompleteCommit,
}

/// Shared mutable failpoint. `FailpointCell::none()` is the production default
/// (no injected failure). A test sets the desired checkpoint, runs the
/// migration, and inspects the on-disk intermediate state.
pub struct FailpointCell(Mutex<Failpoint>);

impl FailpointCell {
    /// No failpoint — production default.
    pub fn none() -> Self {
        Self(Mutex::new(Failpoint::None))
    }

    /// Set the failpoint to a new checkpoint (test-only in practice).
    pub fn set(&self, fp: Failpoint) {
        *self.0.lock() = fp;
    }

    /// If the cell's failpoint equals `point`, return `Err(InjectedFail(point))`;
    /// otherwise `Ok(())`. Full `PartialEq` comparison — `AfterProfileInsert`
    /// matches only when the carried id is equal.
    pub fn maybe_fail(&self, point: Failpoint) -> Result<(), MigrationError> {
        let guard = self.0.lock();
        if *guard == point {
            return Err(MigrationError::InjectedFail(point));
        }
        drop(guard);
        Ok(())
    }
}

// ─── KeystoreIo seam (production vs test identity) ───────────────────────

/// The keystore operations the coordinator uses. Production implements this with
/// the machine-identity free functions; the test seam implements it with the
/// `*_with_identity` helpers. Both go through the same core so the phase logic
/// never drifts.
trait KeystoreIo {
    fn load_state(&self, dir: &Path) -> KeystoreLoadState;
    fn backup(&self, dir: &Path) -> Result<(), KeystoreError>;
    fn migrate_to_v2(&self, dir: &Path, map: std::collections::HashMap<String, String>)
        -> Result<(), KeystoreError>;
}

/// Production: reads the real machine identity (via `IdentitySource::CURRENT`).
struct MachineKeystore;

impl KeystoreIo for MachineKeystore {
    fn load_state(&self, dir: &Path) -> KeystoreLoadState {
        keystore::load_state(dir)
    }
    fn backup(&self, dir: &Path) -> Result<(), KeystoreError> {
        backup_keystore(dir)
    }
    fn migrate_to_v2(
        &self,
        dir: &Path,
        map: std::collections::HashMap<String, String>,
    ) -> Result<(), KeystoreError> {
        migrate_to_v2(dir, map)
    }
}

/// Test seam: decrypts with an injected identity string instead of the OS
/// identity. Exercises the SAME `*_with_identity` cores as production.
pub(crate) struct IdentityKeystore<'a> {
    identity: &'a str,
}

impl<'a> IdentityKeystore<'a> {
    pub(crate) fn new(identity: &'a str) -> Self {
        Self { identity }
    }
}

impl<'a> KeystoreIo for IdentityKeystore<'a> {
    fn load_state(&self, dir: &Path) -> KeystoreLoadState {
        keystore::load_state_with_identity(dir, self.identity)
    }
    fn backup(&self, dir: &Path) -> Result<(), KeystoreError> {
        keystore::backup_keystore_with_identity(dir, self.identity)
    }
    fn migrate_to_v2(
        &self,
        dir: &Path,
        map: std::collections::HashMap<String, String>,
    ) -> Result<(), KeystoreError> {
        keystore::migrate_to_v2_with_identity(dir, map, self.identity)
    }
}

// ─── Settings parsing + backup ───────────────────────────────────────────

/// Read `settings.json` MANUALLY (not via tauri-plugin-store, which swallows
/// parse errors and returns defaults — migration must fail-closed on a corrupt
/// settings file).
///
/// - File missing  → `Ok(None)` (fresh install; migration proceeds without defaults).
/// - Invalid JSON  → `Err(SettingsCorrupt)`.
/// - Valid object  → `Ok(Some(RawSettings))`. Unknown keys are ignored; missing
///   keys stay `None`.
pub fn parse_settings_raw(path: &Path) -> Result<Option<RawSettings>, MigrationError> {
    let bytes = match std::fs::read(path) {
        Ok(b) => b,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(MigrationError::BackupFailed(format!("read settings: {e}"))),
    };
    let v: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| {
        MigrationError::SettingsCorrupt(format!("{}: {e}", path.display()))
    })?;
    // A non-object (array / number / null) is a corrupt settings file.
    let obj = v.as_object().ok_or_else(|| {
        MigrationError::SettingsCorrupt(format!(
            "{}: settings root is not a JSON object",
            path.display()
        ))
    })?;
    let get_str = |k: &str| obj.get(k).and_then(|x| x.as_str()).map(String::from);
    Ok(Some(RawSettings {
        default_provider: get_str("default_provider"),
        target_language: get_str("target_language"),
        fallback_engine: get_str("fallback_engine"),
    }))
}

/// Copy `settings.json` → `settings.json.bak-pre-migration` with TRUE
/// create-new (no-clobber) semantics: skip if the backup already exists (never
/// overwrite a prior backup). Idempotent across migration replays.
///
/// The backup is written DIRECTLY to the final path via `create_new` (O_CREAT |
/// O_EXCL), so the existence check + creation are one atomic step. A concurrent
/// migration that already created the backup loses the `O_EXCL` race and we
/// treat `AlreadyExists` as success (no-clobber). This avoids the
/// `exists()` + `rename()` TOCTOU of the old shape, where `rename` would
/// silently clobber a prior backup on Unix.
///
/// The bytes are fsynced + the file is secured (`fs_acl::secure_file`) before
/// the handle is dropped, so the backup is durable and never observable on disk
/// in an unprotected state.
pub(crate) fn backup_settings(settings_path: &Path) -> Result<(), MigrationError> {
    let bak = settings_bak_path(settings_path);
    // Missing source (fresh install) is a no-op.
    if !settings_path.exists() {
        return Ok(());
    }
    let bytes = std::fs::read(settings_path).map_err(|e| {
        MigrationError::BackupFailed(format!("read settings for backup: {e}"))
    })?;
    // Atomically create the FINAL backup path with O_EXCL. If it already exists
    // (a prior backup from this run or a crashed-but-completed prior attempt),
    // this returns `AlreadyExists` and we skip — TRUE no-clobber, no overwrite.
    use std::io::Write;
    let mut dst = match std::fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(&bak)
    {
        Ok(f) => f,
        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
            // A backup already exists — leave it untouched.
            return Ok(());
        }
        Err(e) => {
            return Err(MigrationError::BackupFailed(format!(
                "create settings backup {}: {e}",
                bak.display()
            )));
        }
    };
    dst.write_all(&bytes).map_err(|e| {
        MigrationError::BackupFailed(format!("write settings backup: {e}"))
    })?;
    // Flush userspace buffers + ask the OS to push the bytes to disk so the
    // backup is durable before we secure + close the handle.
    dst.flush().map_err(|e| {
        MigrationError::BackupFailed(format!("flush settings backup: {e}"))
    })?;
    dst.sync_all().map_err(|e| {
        MigrationError::BackupFailed(format!("fsync settings backup: {e}"))
    })?;
    drop(dst);
    // Secure the final backup file (0600 / restricted ACL). Done AFTER fsync so
    // the bytes are durable; the file is private from the moment of creation
    // because the parent dir is already secured and the create was atomic.
    crate::fs_acl::secure_file(&bak).map_err(|e| {
        MigrationError::BackupFailed(format!("secure settings backup: {e}"))
    })?;
    Ok(())
}

/// Derive the pre-migration backup path for a settings file. Appends
/// `.bak-pre-migration` to the source path (so `settings.json` →
/// `settings.json.bak-pre-migration`).
pub fn settings_bak_path(settings_path: &Path) -> std::path::PathBuf {
    let mut s = settings_path.as_os_str().to_os_string();
    s.push(".bak-pre-migration");
    std::path::PathBuf::from(s)
}

// ─── Seed helpers ─────────────────────────────────────────────────────────

/// Update `preferences.target_language` from settings if present. No-op when
/// settings is `None` or `target_language` is absent (the schema default `zh`
/// stays). Called inside a short transaction by the coordinator.
pub(crate) fn seed_preferences(
    conn: &Connection,
    settings: Option<&RawSettings>,
) -> Result<(), DbError> {
    if let Some(s) = settings {
        if let Some(lang) = &s.target_language {
            conn.execute(
                "UPDATE preferences SET target_language=?1 WHERE id=1",
                rusqlite::params![lang],
            )?;
        }
    }
    Ok(())
}

/// Seed `primary_uuid` / `fallback_uuid` from settings, with write-guards:
/// - `primary_uuid` is only written if the referenced profile is `status='active'`,
///   `enabled=1`, AND its protocol is a real AI protocol (not `custom_http` —
///   repair profiles are `enabled=false` and never selected).
/// - `fallback_uuid` is only written if the referenced profile is a traditional
///   engine (`google_translate` protocol / `google` template_id) that is
///   active+enabled.
///
/// Unknown / repair profiles are NEVER written into primary/fallback — they sit
/// at the bottom of the list disabled, for the user to fix.
pub(crate) fn seed_selection(
    conn: &Connection,
    settings: Option<&RawSettings>,
    candidates: &[CandidateSource],
) -> Result<(), DbError> {
    // Build template_id → uuid map for the just-inserted profiles. Owns the
    // template_id key so the map outlives the transient ProviderProfile.
    let mut by_template: std::collections::HashMap<String, String> =
        std::collections::HashMap::new();
    for c in candidates {
        // Build the profile (pure; no DB write) to read template_id + protocol.
        // A candidate's UUID is deterministic, so this matches the inserted row.
        if let Ok(p) = build_profile(c) {
            by_template.insert(p.template_id.clone(), p.uuid.clone());
        }
    }

    if let Some(s) = settings {
        // Primary: default_provider, only if it built into a selectable (active+
        // enabled, real AI protocol) profile.
        if let Some(dp) = &s.default_provider {
            if let Some(uuid) = by_template.get(dp.as_str()) {
                if profile_is_selectable(conn, uuid)? {
                    conn.execute(
                        "UPDATE preferences SET primary_uuid=?1 WHERE id=1",
                        rusqlite::params![uuid],
                    )?;
                }
            }
        }
        // Fallback: fallback_engine, only if it built into a traditional engine
        // profile that is active+enabled.
        if let Some(fb) = &s.fallback_engine {
            if let Some(uuid) = by_template.get(fb.as_str()) {
                if profile_is_traditional_selectable(conn, uuid)? {
                    conn.execute(
                        "UPDATE preferences SET fallback_uuid=?1 WHERE id=1",
                        rusqlite::params![uuid],
                    )?;
                }
            }
        }
    }
    Ok(())
}

/// Is the provider row `uuid` active+enabled AND not a `custom_http` repair
/// shell? (Repair profiles are `enabled=false` and parked at sort_order 999.)
fn profile_is_selectable(conn: &Connection, uuid: &str) -> Result<bool, DbError> {
    let n: i64 = conn.query_row(
        "SELECT COUNT(*) FROM providers \
         WHERE uuid=?1 AND status='active' AND enabled=1 \
         AND protocol != 'custom_http'",
        rusqlite::params![uuid],
        |r| r.get(0),
    )?;
    Ok(n == 1)
}

/// Is the provider row `uuid` an active+enabled traditional engine?
fn profile_is_traditional_selectable(conn: &Connection, uuid: &str) -> Result<bool, DbError> {
    let n: i64 = conn.query_row(
        "SELECT COUNT(*) FROM providers \
         WHERE uuid=?1 AND status='active' AND enabled=1 \
         AND protocol='google_translate'",
        rusqlite::params![uuid],
        |r| r.get(0),
    )?;
    Ok(n == 1)
}

// ─── verify_key_bearing_profiles ──────────────────────────────────────────

/// Per S0 erratum: enumerate the keys carried by the keystore and assert every
/// key-bearing key (`provider/<uuid>`, bare legacy preset id, or any recovered
/// key) has a matching non-deleted DB profile. A key with no DB row means the
/// migration lost a secret's owner — fail-closed so the user can repair.
///
/// Returns `DbError::Integrity` for a key with no matching row; the coordinator
/// maps that to [`MigrationError::NeedsKeystoreRecovery`] (in Phase 5's
/// transaction this is the only source of an Integrity error, so the mapping is
/// unambiguous). Kept returning `DbError` so it composes with `Database::with_conn`.
pub(crate) fn verify_key_bearing_profiles(
    conn: &Connection,
    ks: &KeystoreLoadState,
) -> Result<(), DbError> {
    let key_set: Vec<String> = match ks {
        KeystoreLoadState::LegacyV1(map) => map.keys().cloned().collect(),
        KeystoreLoadState::CurrentV2(data) => data.provider_keys.keys().cloned().collect(),
        // Missing or Corrupt carry no keys to verify (Corrupt would already have
        // failed in preflight_keystore / Phase 4).
        _ => Vec::new(),
    };

    for secret_ref in &key_set {
        // Every keystore key is stored verbatim as a provider's `secret_ref`
        // (legacy preset rows use the bare id like "openai"; v2 / recovered rows
        // use the full "provider/<uuid>" string). Query by the FULL secret_ref
        // so a key always lands on the profile that actually owns it — never on
        // an unrelated row that happens to embed the same uuid (e.g. a recovered
        // key whose uuid coincides with a preset row's deterministic uuid).
        // status != 'deleted' excludes tombstones (a 'deleting' row still owns
        // its secret until finalize).
        let owned: i64 = conn.query_row(
            "SELECT COUNT(*) FROM providers WHERE secret_ref=?1 AND status != 'deleted'",
            rusqlite::params![secret_ref],
            |r| r.get(0),
        )?;
        if owned == 0 {
            return Err(DbError::Integrity(format!(
                "keystore key '{secret_ref}' has no matching active provider row"
            )));
        }
    }
    Ok(())
}

/// Preflight keystore health: load_state; Corrupt → Err so the user is sent to
/// recovery. Missing/CurrentV2/LegacyV1 → Ok (proceed).
fn preflight_keystore<K: KeystoreIo>(
    ks: &K,
    keystore_dir: &Path,
) -> Result<(), MigrationError> {
    match ks.load_state(keystore_dir) {
        KeystoreLoadState::Corrupt(e) => Err(MigrationError::NeedsKeystoreRecovery(format!(
            "keystore corrupt: {e}"
        ))),
        _ => Ok(()),
    }
}

// ─── THE migration core (shared by production + test seams) ───────────────

/// Run the 5-phase migration. Shared by [`run_migration`] (machine identity)
/// and [`run_migration_with_identity`] (test seam) so the phase logic cannot
/// drift between production and tests.
///
/// See the module docs for the phase order. `ks` abstracts the keystore IO
/// (identity source); `fp` is the failpoint cell.
fn run_migration_core<K: KeystoreIo>(
    db: &Database,
    keystore_dir: &Path,
    settings_path: &Path,
    fp: &FailpointCell,
    ks: &K,
) -> Result<(), MigrationError> {
    // ── PREFLIGHT ────────────────────────────────────────────────────────
    let state = db.with_conn(|conn| migration_state_if_exists(conn))?;
    if state == MigrationState::Complete {
        // Already complete: just confirm the keystore is healthy and bail.
        preflight_keystore(ks, keystore_dir)?;
        return Ok(());
    }
    // Incomplete or NotStarted → proceed (replay-safe).

    // ── PHASE 1: Backup (FIRST persistent action) ────────────────────────
    let settings = parse_settings_raw(settings_path)?;
    backup_settings(settings_path)?;
    // Keystore backup: Missing/CurrentV2 → no-op; LegacyV1 → backup; Corrupt → Err.
    // A backup failure (notably a Corrupt keystore surfacing as Err) means the
    // keystore needs user recovery — we can't safely proceed past Phase 1.
    ks.backup(keystore_dir)
        .map_err(|e| MigrationError::NeedsKeystoreRecovery(format!("keystore backup failed: {e}")))?;
    fp.maybe_fail(Failpoint::AfterBackup)?;

    // ── PHASE 2: DB schema (idempotent, short tx) ────────────────────────
    db.with_conn(|conn| {
        let tx = conn.transaction()?;
        schema::create_all_tables(&tx)?;
        schema::seed_singletons(&tx)?;
        tx.commit()?;
        Ok(())
    })?;
    fp.maybe_fail(Failpoint::AfterSchema)?;

    // ── PHASE 2b: Seed preferences from settings (short tx) ──────────────
    db.with_conn(|conn| {
        let tx = conn.transaction()?;
        seed_preferences(&tx, settings.as_ref())?;
        tx.commit()?;
        Ok(())
    })?;
    fp.maybe_fail(Failpoint::AfterPreferences)?;

    // ── PHASE 3: Enumerate candidates + insert profiles ──────────────────
    // DB is NOT locked during the keystore read (load_state takes only the
    // keystore's own flock).
    let ks_state = ks.load_state(keystore_dir);
    // A Corrupt keystore here is a hard stop (we can't enumerate keys safely).
    if matches!(ks_state, KeystoreLoadState::Corrupt(_)) {
        return Err(MigrationError::NeedsKeystoreRecovery(format!(
            "keystore corrupt at Phase 3: {:?}",
            ks_state
        )));
    }
    let candidates = enumerate_candidates(&ks_state, settings.as_ref());
    for source in &candidates {
        let profile = build_profile(source)?;
        // Each insert is its own short tx so a crash mid-loop leaves a consistent
        // subset (and the loop is idempotent on replay via INSERT OR IGNORE).
        db.with_conn(|conn| {
            let tx = conn.transaction()?;
            insert_or_ignore(&tx, &profile)?;
            tx.commit()?;
            Ok(())
        })?;
        fp.maybe_fail(Failpoint::AfterProfileInsert(
            source.stable_id().to_owned(),
        ))?;
    }
    // Seed primary/fallback selection from settings (write-guarded).
    db.with_conn(|conn| {
        let tx = conn.transaction()?;
        seed_selection(&tx, settings.as_ref(), &candidates)?;
        tx.commit()?;
        Ok(())
    })?;
    fp.maybe_fail(Failpoint::AfterProfiles)?;

    // ── PHASE 4: Keystore rewrite (keystore flock only, DB NOT locked) ───
    if let KeystoreLoadState::LegacyV1(map) = &ks_state {
        ks.migrate_to_v2(keystore_dir, map.clone())?;
    }
    fp.maybe_fail(Failpoint::AfterKeystoreRewrite)?;

    // ── PHASE 5: Verify + complete (short tx) ────────────────────────────
    // Re-load the keystore (DB NOT locked) so verification sees the post-rewrite
    // (v2) key set.
    let ks_after = ks.load_state(keystore_dir);
    if matches!(ks_after, KeystoreLoadState::Corrupt(_)) {
        return Err(MigrationError::NeedsKeystoreRecovery(format!(
            "keystore corrupt at Phase 5: {:?}",
            ks_after
        )));
    }
    db.with_conn(|conn| {
        let tx = conn.transaction()?;
        verify_key_bearing_profiles(&tx, &ks_after)?;
        set_migration_complete(&tx)?;
        tx.commit()?;
        Ok(())
    })
    .map_err(|e| match e {
        // A verification Integrity failure means a keystore key lost its owner —
        // surface as NeedsKeystoreRecovery so the user is sent to repair.
        DbError::Integrity(msg) if msg.contains("keystore key '") => {
            MigrationError::NeedsKeystoreRecovery(msg)
        }
        other => MigrationError::Db(other),
    })?;
    fp.maybe_fail(Failpoint::AfterCompleteCommit)?;

    Ok(())
}

// ─── Public entry points ──────────────────────────────────────────────────

/// Run the migration using the real machine identity for keystore operations.
/// This is the production entry point (called from app setup).
pub fn run_migration(
    db: &Database,
    keystore_dir: &Path,
    settings_path: &Path,
    fp: &FailpointCell,
) -> Result<(), MigrationError> {
    run_migration_core(db, keystore_dir, settings_path, fp, &MachineKeystore)
}

/// Test-only: run the migration with an injected keystore identity string
/// instead of reading the OS identity. Drives the SAME phase core as production
/// (no duplicated logic) via the `*_with_identity` keystore seams.
#[doc(hidden)]
pub fn run_migration_with_identity(
    db: &Database,
    keystore_dir: &Path,
    settings_path: &Path,
    fp: &FailpointCell,
    identity: &str,
) -> Result<(), MigrationError> {
    let ks = IdentityKeystore::new(identity);
    run_migration_core(db, keystore_dir, settings_path, fp, &ks)
}
