# S2a rev-6 Amendment — 3 final P1s

**Status:** Amendment to `s2a-data-model-migration-rev6.md`. Document-only.
**Scope:** Patches 3 specific issues. All other rev-6 content is approved and unchanged.

---

## P1 #1 — preflight must not swallow DB corruption (OptionalExtension)

### Problem
`migration_state_if_exists` uses `.unwrap_or(false)` which converts ALL errors (corrupt header, NotADatabase, invalid column) into "table doesn't exist" or "incomplete", then proceeds with migration instead of entering `NeedsDatabaseRecovery`.

### Fix — use `OptionalExtension::optional()`
```rust
use rusqlite::OptionalExtension;

fn migration_state_if_exists(db: &Database) -> Result<MigrationState, DbError> {
    db.with_conn(|conn| {
        // Check if _schema_migrations table exists. Only QueryReturnedNoRows → None.
        // All other errors (corrupt DB, NotADatabase, etc.) propagate as Err.
        let exists: Option<i64> = conn.query_row(
            "SELECT 1 FROM sqlite_master WHERE type='table' AND name='_schema_migrations'",
            [], |r| r.get(0),
        ).optional()?;
        if exists.is_none() { return Ok(MigrationState::NotStarted); }

        let complete: Option<i64> = conn.query_row(
            "SELECT migration_complete FROM _schema_migrations WHERE id=1",
            [], |r| r.get(0),
        ).optional()?;  // row missing → None (incomplete); other errors → Err
        Ok(match complete {
            Some(1) => MigrationState::Complete,
            _ => MigrationState::Incomplete,
        })
    })
}
```

`optional()` converts ONLY `QueryReturnedNoRows` → `Ok(None)`. Corrupt-header / NotADatabase / IO errors propagate as `Err(DbError)` → caller maps to `NeedsDatabaseRecovery`.

### Tests
- **Corrupt DB header** (write garbage bytes to the .db file): `migration_state_if_exists` returns `Err`, NOT `Ok(NotStarted)`. Startup maps to `NeedsDatabaseRecovery`.
- **Table exists but query fails** (simulated column type mismatch): returns `Err`, not `Ok(Incomplete)`.

---

## P1 #2 — archive/reset keystore: atomic single write-gate, no self-deadlock

### Problem
`archive_keystore`/`reset_keystore` hold `data_gate.write()`, then call `post_keystore_archive` which ALSO acquires `data_gate.write()` → self-deadlock (parking_lot RwLock is non-reentrant). Or if the gate is released between keystore-op and DB-cleanup, `set_key` can write a new key in the gap.

### Fix — single write-gate covers keystore-op + DB-cleanup + readiness

The archive/reset command is ONE atomic operation under a single `data_gate.write()`:

```rust
fn archive_or_reset_keystore(state: &Arc<AppState>, mode: ArchiveMode) -> Result<(), AppError> {
    let _gate = state.data_gate.write();  // ── single write gate for entire op ──

    // 1. Keystore archive/reset (under keystore's own lock, data_gate held):
    state.keystore.archive_or_reset(mode)?;

    // 2. DB cleanup — helper does NOT acquire data_gate (caller already holds it):
    post_archive_db_cleanup_locked(state)?;

    // 3. Readiness update (still under gate):
    update_readiness_after_archive(state);
    Ok(())
}

/// Helper: caller MUST already hold data_gate.write().
fn post_archive_db_cleanup_locked(state: &Arc<AppState>) -> Result<(), AppError> {
    let db_guard = state.db.read();
    let db = match db_guard.as_ref() {
        Some(db) => db.clone(),
        None => return Ok(()),  // DB unavailable — leave readiness as-is
    };
    drop(db_guard);
    db.with_conn(|conn| {
        let tx = conn.transaction()?;
        tx.execute("UPDATE providers SET enabled=0 WHERE needs_key=1", [])?;
        tx.execute("UPDATE preferences SET primary_uuid=NULL, parallel_uuids='[]', fallback_uuid=NULL, parallel_consent_scope=NULL, parallel_consent_version=NULL WHERE id=1", [])?;
        tx.execute("UPDATE _schema_migrations SET migration_complete=1 WHERE id=1", [])?;
        tx.commit()
    })?;
    Ok(())
}

/// Readiness transition based on old state + cleanup result.
fn update_readiness_after_archive(state: &Arc<AppState>) {
    let mut readiness = state.readiness.write();
    let db_exists = state.db.read().is_some();
    match (&*readiness, db_exists) {
        // DB unavailable → stay in DB recovery regardless of keystore fix:
        (NeedsDatabaseRecovery { .. }, _) => { /* keep */ }
        // DB exists + was Ready or NeedsKeystoreRecovery → keystore fixed → Ready:
        (Ready, true) | (NeedsKeystoreRecovery { .. }, true) => *readiness = Ready,
        // DB exists but was MigrationIncomplete for other reasons → keep
        // (unless we can prove keystore was the sole cause — see below):
        (MigrationIncomplete { .. }, true) => { /* keep — re-run migration to resolve */ }
        // No DB → don't claim Ready:
        (_, false) => { /* keep NeedsDatabaseRecovery or whatever it was */ }
    }
}
```

**No self-deadlock:** `post_archive_db_cleanup_locked` does NOT acquire `data_gate` — the caller holds it. **No race:** the single write-gate covers keystore-op → DB-cleanup → readiness, so no `set_key` can slip in between.

If cleanup fails (DB transaction error), readiness becomes `MigrationIncomplete` (not stale `NeedsKeystoreRecovery`):
```rust
// In archive_or_reset_keystore, if post_archive_db_cleanup_locked returns Err:
if let Err(e) = post_archive_db_cleanup_locked(state) {
    *state.readiness.write() = DataReadiness::MigrationIncomplete {
        reason: format!("post-archive cleanup failed: {e}"),
    };
    return Err(e.into());
}
```

### Test — set_key × archive/reset race
```
Thread A: set_key(uuid, "new-key")   — blocks on data_gate.read()
Thread B: archive_keystore()          — holds data_gate.write(), clears keystore, disables profiles
Thread A: (unblocks after B releases) — set_key proceeds, but profile is now enabled=false
Assert: after both complete, keystore has NO orphan key for a disabled/deleted profile.
        If set_key runs after archive, it writes to an enabled=false profile — the key
        exists but the profile is not callable. This is acceptable (user re-enabled manually).
        The test asserts no key exists for profiles that were cleaned up BEFORE set_key ran.
```

---

## P1 #3 — Database::close correct signature + ArchiveFailpoint injection

### Problem
rusqlite 0.40.1 `Connection::close(self) -> Result<(), (Self, Error)>` — on failure, returns the Connection back. Rev-6 declared `Result<(), rusqlite::Error>` which is wrong. Also no testable injection mechanism for close/rename/open failures.

### Fix — close returns recoverable Database
```rust
impl Database {
    /// Close the connection. On failure, returns the Database back so the
    /// caller can restore it (matching rusqlite's (Self, Error) pattern).
    pub fn close(self) -> Result<(), (Database, rusqlite::Error)> {
        let conn = self.conn.into_inner();  // parking_lot::Mutex::into_inner — panics if locked
        conn.close().map_err(|(conn, e)| (Database { conn: parking_lot::Mutex::new(conn) }, e))
    }
}
```

### archive_database — uses correct close, restores on failure
```rust
// Inside archive_database spawn_blocking, after taking old_db:
let old_db = db_slot.take();  // Option<Arc<Database>>
let Some(arc) = old_db else {
    // No DB — nothing to close, proceed to open fresh.
    /* ... open + migrate ... */
};

// try_unwrap to get owned Database:
let owned_db = match Arc::try_unwrap(arc) {
    Ok(db) => db,
    Err(arc_back) => {
        *db_slot = Some(arc_back);  // restore — someone still holds a clone
        return Err("DB still in use".into());
    }
};

// close — on failure, restore the Database:
match owned_db.close() {
    Ok(()) => { /* handle released, proceed to rename */ }
    Err((db_back, e)) => {
        // close failed — restore the Database in the slot, don't rename.
        *db_slot = Some(Arc::new(db_back));
        log::error!("DB close failed: {e}");
        // readiness stays as-is (was Ready or whatever). DB is still usable.
        return Err(format!("DB close failed: {e}"));
    }
}
// Proceed to rename → open → migrate (with ArchiveFailpoint, below).
```

### ArchiveFailpoint — testable injection for each step
```rust
#[derive(Clone)]
pub enum ArchiveFailpoint {
    None,
    CloseError,      // simulate close failure
    RenameError,     // simulate rename failure
    ReopenError,     // simulate open failure
    MigrationKeystoreCorrupt,  // re-migration returns NeedsKeystoreRecovery
    MigrationOther,            // re-migration returns other error
}

pub struct ArchiveFailpointCell(pub parking_lot::Mutex<ArchiveFailpoint>);
```

In tests, each failpoint is injected by setting the cell. The archive_database implementation checks the cell at each step:

```rust
// After close succeeds, before rename:
match afp.0.lock().clone() {
    ArchiveFailpoint::RenameError => {
        // restore slot (re-open the closed DB? No — close consumed it).
        // The file handle is released (Drop on close). Reopen to restore:
        match Database::open(&db_path) {
            Ok(db) => *db_slot = Some(Arc::new(db)),
            Err(_) => *state.readiness.write() = NeedsDatabaseRecovery { reason: "reopen after rename-fail inject".into() },
        }
        return Err("injected rename error".into());
    }
    _ => {}
}
fs::rename(&db_path, &broken_path)?;
// After rename, before reopen:
if afp == ReopenError { /* don't open, set NeedsDatabaseRecovery */ }
// etc.
```

### Corrected failure recovery table (with failpoint mechanism)
| Failpoint | How injected | Slot after | Readiness after | Canonical file |
|-----------|-------------|------------|-----------------|----------------|
| CloseError | `ArchiveFailpoint::CloseError` | Restored (re-wrapped) | Unchanged (was Ready/whatever) | Original untouched |
| RenameError | `ArchiveFailpoint::RenameError` | Reopened original | Unchanged or NeedsDatabaseRecovery | Original untouched |
| ReopenError | `ArchiveFailpoint::ReopenError` | None | NeedsDatabaseRecovery | .broken exists, no new db |
| MigrationKeystoreCorrupt | `ArchiveFailpoint::MigrationKeystoreCorrupt` | Some(new_db) | NeedsKeystoreRecovery | Fresh db, keystore corrupt |
| MigrationOther | `ArchiveFailpoint::MigrationOther` | Some(new_db) | MigrationIncomplete | Fresh db, partial migration |

Each is a real, deterministic injection — no reliance on permission tricks or unreleased statements.

---

## Compile fix — build_profile unused `ks`

`build_profile(source, _ks)` — the `ks` parameter is unused in S2a (key existence is a runtime lookup, not needed at profile-build time). Rename to `_ks` to satisfy `-D warnings`, or remove the parameter entirely. Remove is cleaner:

```rust
fn build_profile(source: &CandidateSource) -> Result<ProviderProfile, MigrationError> { ... }
```

Call site: `let profile = build_profile(source)?;`

---

## Summary of what changed vs rev-6

| Item | rev-6 | Amendment |
|------|-------|-----------|
| migration_state_if_exists | `.unwrap_or(false)` swallows errors | `.optional()?` propagates corruption |
| archive/reset + post_archive | self-deadlock or race | single `data_gate.write()` covers keystore-op + cleanup + readiness |
| readiness after archive | unconditional `Ready` | conditional based on old readiness + DB existence + cleanup result |
| Database::close | `Result<(), rusqlite::Error>` (wrong) | `Result<(), (Database, rusqlite::Error)>` + restore on failure |
| archive failure tests | "inject" with no mechanism | `ArchiveFailpoint` enum + cell, 5 deterministic injections |
| build_profile | unused `ks` param | removed |

All other rev-6 content is approved and unchanged.
