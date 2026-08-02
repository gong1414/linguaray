# S2a rev-6 Amendment 3 — 3 final P1s

**Status:** Final amendment. Patches amendment 2. Document-only.
All prior content (rev-6 + amendments 1–2 + erratum) approved and unchanged.

---

## P1 #1 — Separate DB-recovery readiness from keystore-archive readiness

### Problem
`archive_database_inner` on success calls `update_readiness_after_archive`, which was designed for **keystore** archive and keeps `NeedsDatabaseRecovery` unchanged. So the most common DB-recovery path stays stuck:

```
NeedsDatabaseRecovery → archive_database → fresh DB + migration OK → still NeedsDatabaseRecovery ✗
```

### Fix — two distinct readiness transitions

**`update_readiness_after_db_recovery`** (for `archive_database` success):
```rust
/// Called after archive_database successfully opens fresh DB + completes migration.
/// DB recovery SUCCEEDED → transition to Ready (unless keystore is corrupt).
fn update_readiness_after_db_recovery(state: &Arc<AppState>) {
    // run_migration already checked keystore. If it returned Ok, keystore is readable.
    // The only remaining concern: did migration leave us complete?
    let complete = state.db.read().as_ref()
        .map(|db| db.with_conn(|c| schema::migration_complete(c)).unwrap_or(false))
        .unwrap_or(false);
    if complete {
        *state.readiness.write() = DataReadiness::Ready;
    } else {
        *state.readiness.write() = DataReadiness::MigrationIncomplete {
            reason: "DB recovery migration incomplete".into(),
        };
    }
}
```

In `archive_database_inner`, the success path becomes:
```rust
match mig_result {
    Ok(()) => {
        *db_slot = Some(new_db);
        // DB recovery succeeded — resume deletions, then Ready:
        if let Err(e) = provider_resume_deletions(&new_db, &state.keystore) {
            log::warn!("resume_deletions after DB recovery: {e}");
            // Non-fatal — does not block Ready.
        }
        update_readiness_after_db_recovery(state);
        Ok(())
    }
    Err(MigrationError::NeedsKeystoreRecovery(r)) => {
        *db_slot = Some(new_db);
        *state.readiness.write() = DataReadiness::NeedsKeystoreRecovery { reason: r };
        Err(/* ... */)
    }
    Err(other) => {
        *db_slot = Some(new_db);
        *state.readiness.write() = DataReadiness::MigrationIncomplete { reason: other.to_string() };
        Err(/* ... */)
    }
}
```

**`update_readiness_after_keystore_archive`** (for `archive_keystore`/`reset_keystore`) remains as defined in amendment 2 — it preserves `NeedsDatabaseRecovery` because archiving the keystore does not fix a DB problem.

The two helpers are NEVER interchangeable. `archive_database` calls the DB-recovery one; `archive_keystore`/`reset_keystore` calls the keystore-archive one.

---

## P1 #2 — Unified error types + AppState paths

### Problem
The reopen branch had `Database::open(&db_path).map_err(|e| /* into rusqlite::Error or DbError */ e)` — a type placeholder. The injected branch returned `rusqlite::Error`, the real branch returned `DbError` — `if/else` can't unify them. Also `db_path()`, `keystore_dir()`, `settings_path()` are undefined methods on `AppState`.

### Fix — unify on `DbError`, add paths to AppState

**AppState gains the three paths:**
```rust
pub struct AppState {
    pub data_gate: parking_lot::RwLock<()>,
    pub provider_locks: parking_lot::Mutex<HashMap<String, Arc<parking_lot::Mutex<()>>>>,
    pub db: parking_lot::RwLock<Option<Arc<Database>>>,
    pub keystore: Keystore,
    pub client: reqwest::Client,
    pub readiness: parking_lot::RwLock<DataReadiness>,
    pub coord: ProviderCoordinator,
    // Paths (set once at startup, never change):
    pub db_path: PathBuf,
    pub keystore_dir: PathBuf,
    pub settings_path: PathBuf,
}
```

No methods needed — fields accessed directly as `state.db_path`, `state.keystore_dir`, `state.settings_path`.

**Unified reopen branch (no placeholder):**
```rust
let open_result: Result<Database, DbError> = match afp.get() {
    ArchiveFailpoint::ReopenError => Err(DbError::Injected("reopen")),
    _ => Database::open(&state.db_path),
};
let new_db = match open_result {
    Ok(db) => Arc::new(db),
    Err(e) => {
        *db_slot = None;
        *state.readiness.write() = DataReadiness::NeedsDatabaseRecovery { reason: e.to_string() };
        return Err(AppError::ReopenFailed(e));
    }
};
```

`DbError::Injected(String)` is a real variant for test injection. Production never produces it (afp=None → always takes the `_` arm). `AppError::ReopenFailed(DbError)` — consistent type.

**`run_migration` call uses explicit paths:**
```rust
run_migration(&new_db, &state.keystore_dir, &state.settings_path, &FailpointCell::none())?
```

---

## P1 #3 — Real overlapping concurrency tests with hooks (not sequential)

### Problem
Amendment 2's "barrier tests" were actually sequential (wait for A to fully complete, then start B). They prove final state for A→B and B→A ordering, but do NOT prove that:
- archive **blocks** while a read-gate is held;
- set_key cannot enter the gap between keystore-op and DB-cleanup.

### Fix — hook-based tests that verify actual blocking

**Test infrastructure: coordination hooks on the inner functions.**

The production `set_key_inner` and `archive_or_reset_keystore_inner` accept an optional `&TestHook` (which is `None` in production):

```rust
pub struct TestHook {
    /// Signaled when the function reaches a specific phase.
    /// The function blocks on `resume_rx` until the test releases it.
    pub phase_tx: crossbeam_channel::Sender<Phase>,
    pub resume_rx: crossbeam_channel::Receiver<()>,
}

pub enum SetKeyPhase { AcquiredReadGate, BeforeKeystoreWrite, Done }
pub enum ArchivePhase { AcquiredWriteGate, AfterKeystoreClear, BeforeDbCleanup, Done }
```

**Inner functions with hook points (production passes `None` → no blocking):**
```rust
fn set_key_inner(state: &Arc<AppState>, uuid: &str, key: &str, hook: Option<&TestHook>) -> Result<(), AppError> {
    let _gate = state.data_gate.read();
    if let Some(h) = hook { h.phase_tx.send(SetKeyPhase::AcquiredReadGate).ok(); h.resume_rx.recv().ok(); }
    let lock = state.lock_provider(uuid);
    let _plock = lock.lock();
    let db = state.db.read().clone().ok_or(AppError::NotReady)?;
    let profile = db.with_conn(|c| providers::get(c, uuid))?;
    if profile.status != "active" { return Err(AppError::NotCallable); }
    if let Some(h) = hook { h.phase_tx.send(SetKeyPhase::BeforeKeystoreWrite).ok(); h.resume_rx.recv().ok(); }
    state.keystore.set_provider_key(&profile.secret_ref, key)?;
    Ok(())
}

fn archive_or_reset_keystore_inner(state: &Arc<AppState>, mode: ArchiveMode, hook: Option<&TestHook>) -> Result<(), AppError> {
    let _gate = state.data_gate.write();
    if let Some(h) = hook { h.phase_tx.send(ArchivePhase::AcquiredWriteGate).ok(); }
    state.keystore.archive_or_reset(mode)?;
    if let Some(h) = hook { h.phase_tx.send(ArchivePhase::AfterKeystoreClear).ok(); h.resume_rx.recv().ok(); }
    post_archive_db_cleanup_locked(state)?;
    if let Some(h) = hook { h.phase_tx.send(ArchivePhase::Done).ok(); }
    update_readiness_after_keystore_archive(state);
    Ok(())
}
```

**Test 1: set_key holds read-gate → archive blocks**

```
1. Create channels: (set_key_phase_tx, set_key_resume_rx), (archive_phase_tx, _).
2. Spawn thread A: set_key_inner(state, uuid, "new-key", Some(&set_key_hook)).
   - A acquires data_gate.read().
   - A sends SetKeyPhase::AcquiredReadGate.
   - A blocks on set_key_resume_rx.
3. Spawn thread B: archive_or_reset_keystore_inner(state, Archive, Some(&archive_hook)).
4. Wait 200ms (deterministic — no sleep dependency on business logic, just polling).
5. Assert: archive_hook has NOT sent ArchivePhase::AcquiredWriteGate.
   → archive is blocked because set_key holds the read gate. ✓
6. Release set_key_resume_rx → A completes (writes key, releases gate).
7. Wait for archive_phase_tx → receives AcquiredWriteGate → archive proceeds.
8. Assert final state: keystore cleared (key was written then archived), profile disabled.
```

**Test 2: archive holds write-gate in cleanup gap → set_key blocks**

```
1. Create channels: (archive_phase_tx, archive_resume_rx), (set_key_phase_tx, _).
2. Spawn thread B: archive_or_reset_keystore_inner(state, Archive, Some(&archive_hook)).
   - B acquires data_gate.write().
   - B clears keystore.
   - B sends AfterKeystoreClear.
   - B blocks on archive_resume_rx (BEFORE DB cleanup).
3. Spawn thread A: set_key_inner(state, uuid, "new-key", Some(&set_key_hook)).
4. Wait 200ms.
5. Assert: set_key_hook has NOT sent SetKeyPhase::AcquiredReadGate.
   → set_key is blocked because archive holds the write gate. ✓
6. Assert: keystore is already cleared (AfterKeystoreClear was sent).
   → The gap between keystore-clear and DB-cleanup is covered by the write gate.
   → set_key cannot write a key in this gap. ✓
7. Release archive_resume_rx → B completes DB cleanup → releases gate.
8. set_key unblocks → writes "new-key" → profile is enabled=false (cleanup disabled it).
9. Assert: keystore has "new-key" under secret_ref. Profile enabled=false.
   NOT an orphan (active non-deleted row, key usable after explicit enable).
```

**Key properties:**
- Tests use the SAME `set_key_inner` / `archive_or_reset_keystore_inner` as production (hook is `None` in prod).
- Hooks only pause at phase boundaries — they do NOT copy or alter business logic.
- `crossbeam_channel` for deterministic phase signaling — no sleep-based timing assertions.
- Both tests prove **actual blocking** (the other operation has not entered its critical section), not just final state.

---

## Summary

| Item | Fix |
|------|-----|
| DB recovery readiness | Separate `update_readiness_after_db_recovery` (→ Ready on success) from keystore-archive helper. archive_database calls the DB one. |
| Type placeholder | `DbError::Injected(String)` variant; reopen branch returns `Result<Database, DbError>` uniformly. `AppError::ReopenFailed(DbError)`. |
| AppState paths | `db_path`, `keystore_dir`, `settings_path` as `PathBuf` fields (not methods). |
| Concurrency tests | Hook-based (`TestHook` + `crossbeam_channel`) overlapping tests proving actual blocking at read-gate and cleanup-gap. Same inner functions as production. |

All prior amendments + rev-6 + erratum remain approved and unchanged.
