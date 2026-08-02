# S2a rev-6 Amendment 4 — 2 P1s + 1 compile fix

**Status:** Final amendment. Patches amendments 2 and 3. Document-only.
All prior content (rev-6 + amendments 1–3 + erratum) approved and unchanged.

---

## P1 #1 — DB recovery success path self-deadlock + move-after-use + contract violation

### Problems in amendment 3's `update_readiness_after_db_recovery`

1. **Self-deadlock:** `archive_database_inner` holds `state.db.write()` for the entire function. The helper calls `state.db.read()` on the same thread → parking_lot RwLock is non-reentrant → deadlock.
2. **Move-after-use:** `*db_slot = Some(new_db)` moves `new_db` into the slot, then the helper borrows `&new_db` again → compile error.
3. **Contract violation:** `provider_resume_deletions` failure is logged and ignored, proceeding to `Ready`. The frozen contract says any failure → `MigrationIncomplete`.
4. **Error swallowing:** `.unwrap_or(false)` in the helper's re-query swallows DB errors.

### Fix — delete the helper entirely; inline the success path with no re-query

`run_migration == Ok(())` already proves migration is complete and keystore is readable. No second query needed. The success branch is written inline in `archive_database_inner`:

```rust
match mig_result {
    Ok(()) => {
        // run_migration Ok ⇒ migration_complete=1, keystore readable. No re-query needed.
        match provider_resume_deletions(&new_db, &state.keystore) {
            Ok(()) => {
                *db_slot = Some(new_db);   // move happens here, after &new_db is no longer borrowed
                *state.readiness.write() = DataReadiness::Ready;
                Ok(())
            }
            Err(e) => {
                *db_slot = Some(new_db);
                // Frozen contract: any failure → MigrationIncomplete (NOT Ready).
                *state.readiness.write() = DataReadiness::MigrationIncomplete {
                    reason: format!("resume_deletions failed: {e}"),
                };
                Err(AppError::ResumeDeletionsFailed(e))
            }
        }
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

**No `update_readiness_after_db_recovery` function exists.** The logic is inline. No `state.db.read()` while holding `state.db.write()`. No move-after-use (`&new_db` used in `resume_deletions` before the move into `db_slot`). `resume_deletions` failure → `MigrationIncomplete` (contract honored).

**Note on `db_slot` borrow:** The `db_slot` write-guard (`state.db.write()`) is held for the entire `archive_database_inner`. `provider_resume_deletions` takes `&new_db` (a local `Arc<Database>`), not `&*db_slot`, so there is no borrow conflict. The `*db_slot = Some(new_db)` assignment happens after `resume_deletions` returns, when `&new_db` is no longer used.

---

## P1 #2 — TestHook unified enum, single pause point, Attempting phases

### Problems in amendment 3's hook design

1. **Undefined `Phase` type:** `Sender<Phase>` references an undefined enum. `SetKeyPhase` and `ArchivePhase` are separate types that can't share one channel.
2. **Multiple pause points:** `set_key_inner` blocks at both `AcquiredReadGate` and `BeforeKeystoreWrite`, but tests only resume once → stuck at the second.
3. **200ms doesn't prove blocking:** "spawned 200ms ago, no signal" doesn't prove the thread attempted to acquire the lock — it may just not be scheduled.
4. **Cross-thread references:** hooks need `Arc<TestHook>` or `thread::scope`.

### Fix — unified `HookPhase` enum, `pause_at` single pause, Attempting phases

```rust
/// Unified phase enum for both set_key and archive hooks.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum HookPhase {
    // set_key phases:
    SetKeyAttemptingRead,    // about to call data_gate.read()
    SetKeyAcquiredRead,      // read() returned
    SetKeyBeforeWrite,       // about to call keystore.set_provider_key
    // archive phases:
    ArchiveAttemptingWrite,  // about to call data_gate.write()
    ArchiveAcquiredWrite,    // write() returned
    ArchiveAfterClear,       // keystore cleared, about to do DB cleanup
    ArchiveDone,
}

pub struct TestHook {
    /// The single phase at which to pause. None = never pause.
    pub pause_at: Option<HookPhase>,
    pub phase_tx: crossbeam_channel::Sender<HookPhase>,
    pub resume_rx: crossbeam_channel::Receiver<()>,
}

impl TestHook {
    /// Always sends the phase signal. Only blocks if phase == pause_at.
    pub fn hit(&self, phase: HookPhase) {
        let _ = self.phase_tx.send(phase);
        if self.pause_at == Some(phase) {
            let _ = self.resume_rx.recv();
        }
    }
}
```

**Inner functions call `hook.hit(phase)` at each boundary (hook is `Option<&TestHook>`, None in production):**

```rust
fn set_key_inner(state: &Arc<AppState>, uuid: &str, key: &str, hook: Option<&TestHook>) -> Result<(), AppError> {
    if let Some(h) = hook { h.hit(HookPhase::SetKeyAttemptingRead); }
    let _gate = state.data_gate.read();
    if let Some(h) = hook { h.hit(HookPhase::SetKeyAcquiredRead); }
    let lock = state.lock_provider(uuid);
    let _plock = lock.lock();
    let db = state.db.read().clone().ok_or(AppError::NotReady)?;
    let profile = db.with_conn(|c| providers::get(c, uuid))?;
    if profile.status != "active" { return Err(AppError::NotCallable); }
    if let Some(h) = hook { h.hit(HookPhase::SetKeyBeforeWrite); }
    state.keystore.set_provider_key(&profile.secret_ref, key)?;
    Ok(())
}

fn archive_or_reset_keystore_inner(state: &Arc<AppState>, mode: ArchiveMode, hook: Option<&TestHook>) -> Result<(), AppError> {
    if let Some(h) = hook { h.hit(HookPhase::ArchiveAttemptingWrite); }
    let _gate = state.data_gate.write();
    if let Some(h) = hook { h.hit(HookPhase::ArchiveAcquiredWrite); }
    state.keystore.archive_or_reset(mode)?;
    if let Some(h) = hook { h.hit(HookPhase::ArchiveAfterClear); }
    post_archive_db_cleanup_locked(state)?;
    if let Some(h) = hook { h.hit(HookPhase::ArchiveDone); }
    update_readiness_after_keystore_archive(state);
    Ok(())
}
```

`hit()` always sends the phase, but only blocks at `pause_at`. A thread with `pause_at = None` (or at a different phase) signals every phase without blocking. This means tests can observe the exact phase a thread reached without relying on timing.

### Test 1: set_key holds read-gate → archive blocks

```rust
// set_key pauses AFTER acquiring read gate:
let (sk_tx, sk_rx) = crossbeam_channel::bounded(16);
let (sk_resume_tx, sk_resume_rx) = crossbeam_channel::bounded(1);
let sk_hook = TestHook { pause_at: Some(HookPhase::SetKeyAcquiredRead), phase_tx: sk_tx, resume_rx: sk_resume_rx };

// archive does NOT pause (pause_at = None), but signals every phase:
let (ar_tx, ar_rx) = crossbeam_channel::bounded(16);
let (_, ar_resume_rx) = crossbeam_channel::bounded(1);
let ar_hook = TestHook { pause_at: None, phase_tx: ar_tx, resume_rx: ar_resume_rx };

std::thread::scope(|s| {
    // Thread A: set_key — will pause at SetKeyAcquiredRead (holds read gate).
    let state_a = state.clone();
    let h = sk_hook.clone();  // TestHook needs Clone (Arc or derived)
    s.spawn(move || { let _ = set_key_inner(&state_a, uuid, "new-key", Some(&h)); });

    // Wait for set_key to signal it acquired the read gate:
    assert_eq!(sk_rx.recv(), Ok(HookPhase::SetKeyAcquiredRead));

    // Thread B: archive — should block on data_gate.write().
    let state_b = state.clone();
    let h2 = ar_hook.clone();
    s.spawn(move || { let _ = archive_or_reset_keystore_inner(&state_b, ArchiveMode::Archive, Some(&h2)); });

    // archive sent AttemptingWrite (it's trying), but has NOT sent AcquiredWrite:
    assert_eq!(ar_rx.recv(), Ok(HookPhase::ArchiveAttemptingWrite));
    assert!(ar_rx.is_empty());  // archive is blocked — no AcquiredWrite yet. ✓

    // Release set_key → it completes, releases read gate → archive proceeds.
    let _ = sk_resume_tx.send(());
    // archive continues through all phases:
    assert_eq!(ar_rx.recv(), Ok(HookPhase::ArchiveAcquiredWrite));
});
```

**Why this proves blocking:** `ArchiveAttemptingWrite` is sent BEFORE `data_gate.write()`. `ArchiveAcquiredWrite` is sent AFTER it returns. The test receives `AttemptingWrite` but `is_empty()` confirms `AcquiredWrite` has NOT arrived → the thread is blocked inside `write()`. No timing assumption.

### Test 2: archive holds write-gate in cleanup gap → set_key blocks

```rust
// archive pauses AFTER keystore clear, BEFORE DB cleanup:
let (ar_tx2, ar_rx2) = crossbeam_channel::bounded(16);
let (ar_resume_tx2, ar_resume_rx2) = crossbeam_channel::bounded(1);
let ar_hook2 = TestHook { pause_at: Some(HookPhase::ArchiveAfterClear), phase_tx: ar_tx2, resume_rx: ar_resume_rx2 };

// set_key does NOT pause:
let (sk_tx2, sk_rx2) = crossbeam_channel::bounded(16);
let (_, sk_resume_rx2) = crossbeam_channel::bounded(1);
let sk_hook2 = TestHook { pause_at: None, phase_tx: sk_tx2, resume_rx: sk_resume_rx2 };

std::thread::scope(|s| {
    // Thread B: archive — pauses at ArchiveAfterClear (holds write gate).
    let state_b = state.clone();
    let h = ar_hook2.clone();
    s.spawn(move || { let _ = archive_or_reset_keystore_inner(&state_b, ArchiveMode::Archive, Some(&h)); });

    // Wait for archive to reach AfterKeystoreClear (keystore cleared, gate held):
    assert_eq!(ar_rx2.recv(), Ok(HookPhase::ArchiveAcquiredWrite));
    assert_eq!(ar_rx2.recv(), Ok(HookPhase::ArchiveAfterClear));

    // Thread A: set_key — should block on data_gate.read().
    let state_a = state.clone();
    let h2 = sk_hook2.clone();
    s.spawn(move || { let _ = set_key_inner(&state_a, uuid, "new-key", Some(&h2)); });

    // set_key sent AttemptingRead, but NOT AcquiredRead:
    assert_eq!(sk_rx2.recv(), Ok(HookPhase::SetKeyAttemptingRead));
    assert!(sk_rx2.is_empty());  // set_key blocked — gate held by archive. ✓

    // Release archive → cleanup completes → gate released → set_key proceeds.
    let _ = ar_resume_tx2.send(());
    // set_key acquires read, writes key:
    assert_eq!(sk_rx2.recv(), Ok(HookPhase::SetKeyAcquiredRead));
});
// Final assert: keystore has "new-key". Profile enabled=false (cleanup disabled it).
// Not an orphan: active non-deleted row, key usable after explicit enable.
```

**TestHook needs `Clone`** — either derive it (channels are `Send + Clone`) or wrap in `Arc<TestHook>`. Since `crossbeam_channel::Sender/Receiver` are `Clone`, `#[derive(Clone)]` on `TestHook` works, and each scoped thread gets a clone.

---

## Compile fix — `DbError::Injected(String)` construction

In amendment 2's reopen branch:
```rust
ArchiveFailpoint::ReopenError => Err(DbError::Injected("reopen".into())),
```
Not `DbError::Injected("reopen")` — the variant takes `String`, so `.into()` or `.to_owned()` is required.

---

## Summary

| Item | Fix |
|------|-----|
| DB recovery self-deadlock | Deleted `update_readiness_after_db_recovery`. Success path inline: `resume_deletions` → `*db_slot = Some(new_db)` → `Ready`. No `db.read()` while holding `db.write()`. No move-after-use. |
| resume_deletions failure | → `MigrationIncomplete` (contract honored, not ignored) |
| TestHook undefined Phase | Unified `HookPhase` enum; `TestHook.hit(phase)` always sends, blocks only at `pause_at` |
| Attempting phases | `SetKeyAttemptingRead` / `ArchiveAttemptingWrite` sent BEFORE lock acquisition → test proves blocking by receiving Attempting but not Acquired |
| Multiple pause deadlock | Single `pause_at` — `hit()` blocks only once |
| Cross-thread | `thread::scope` + `#[derive(Clone)] TestHook` |
| `DbError::Injected` | `.into()` on string literal |

All prior amendments + rev-6 + erratum remain approved and unchanged.
