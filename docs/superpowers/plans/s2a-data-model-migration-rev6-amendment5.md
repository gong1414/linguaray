# S2a rev-6 Amendment 5 — hook recv order + cfg(test) gating

**Status:** Final amendment. Patches amendment 4. Document-only.
All prior content (rev-6 + amendments 1–4 + erratum) approved and unchanged.

---

## P1 — Hook event recv order + cfg(test) dependency scoping

### Problem 1: Test assertions skip the first emitted event

`set_key_inner` emits events in this order:
```
SetKeyAttemptingRead  →  SetKeyAcquiredRead  →  SetKeyBeforeWrite
```
But Test 1's first `recv()` asserted `SetKeyAcquiredRead`, which would actually receive `SetKeyAttemptingRead` first → test fails.

Similarly, `archive_or_reset_keystore_inner` emits:
```
ArchiveAttemptingWrite  →  ArchiveAcquiredWrite  →  ArchiveAfterClear  →  ArchiveDone
```
Test 2 skipped `ArchiveAttemptingWrite`.

### Fix — recv every event in emitted order

**Test 1 corrected:**
```rust
// Thread A: set_key — pauses at SetKeyAcquiredRead (holds read gate).
s.spawn(move || { let _ = set_key_inner(&state_a, uuid, "new-key", Some(&h)); });

// set_key emits AttemptingRead, then AcquiredRead (where it pauses):
assert_eq!(sk_rx.recv(), Ok(HookPhase::SetKeyAttemptingRead));
assert_eq!(sk_rx.recv(), Ok(HookPhase::SetKeyAcquiredRead));  // paused here, holding read gate

// Thread B: archive — should block on data_gate.write().
s.spawn(move || { let _ = archive_or_reset_keystore_inner(&state_b, ArchiveMode::Archive, Some(&h2)); });

// archive emits AttemptingWrite (trying), then blocks:
assert_eq!(ar_rx.recv(), Ok(HookPhase::ArchiveAttemptingWrite));
assert!(ar_rx.is_empty());  // no AcquiredWrite → blocked. ✓

// Release set_key → completes → archive proceeds:
let _ = sk_resume_tx.send(());
assert_eq!(ar_rx.recv(), Ok(HookPhase::ArchiveAcquiredWrite));
```

**Test 2 corrected:**
```rust
// Thread B: archive — pauses at ArchiveAfterClear (holds write gate).
s.spawn(move || { let _ = archive_or_reset_keystore_inner(&state_b, ArchiveMode::Archive, Some(&h)); });

// archive emits AttemptingWrite → AcquiredWrite → AfterClear (where it pauses):
assert_eq!(ar_rx2.recv(), Ok(HookPhase::ArchiveAttemptingWrite));
assert_eq!(ar_rx2.recv(), Ok(HookPhase::ArchiveAcquiredWrite));
assert_eq!(ar_rx2.recv(), Ok(HookPhase::ArchiveAfterClear));  // paused here, gate held

// Thread A: set_key — should block on data_gate.read().
s.spawn(move || { let _ = set_key_inner(&state_a, uuid, "new-key", Some(&h2)); });

// set_key emits AttemptingRead, then blocks:
assert_eq!(sk_rx2.recv(), Ok(HookPhase::SetKeyAttemptingRead));
assert!(sk_rx2.is_empty());  // no AcquiredRead → blocked. ✓

// Release archive → cleanup → gate released → set_key proceeds:
let _ = ar_resume_tx2.send(());
assert_eq!(sk_rx2.recv(), Ok(HookPhase::SetKeyAcquiredRead));
```

Every emitted event is now received in order. No skips.

---

### Problem 2: TestHook in production signatures → crossbeam-channel must be a real dependency

`TestHook` appears in `set_key_inner` / `archive_or_reset_keystore_inner` signatures. If compiled into the release binary, `crossbeam-channel` must be a full dependency — undesirable for a test-only coordination facility.

### Fix — gate TestHook behind `#[cfg(test)]`

**`crossbeam-channel` goes in `[dev-dependencies]` only:**
```toml
[dev-dependencies]
crossbeam-channel = "0.5"
```

**`TestHook`, `HookPhase`, and the `hook` parameter are `#[cfg(test)]`-only:**
```rust
#[cfg(test)]
#[derive(Clone)]
pub struct TestHook {
    pub pause_at: Option<HookPhase>,
    pub phase_tx: crossbeam_channel::Sender<HookPhase>,
    pub resume_rx: crossbeam_channel::Receiver<()>,
}

#[cfg(test)]
impl TestHook {
    pub fn hit(&self, phase: HookPhase) {
        let _ = self.phase_tx.send(phase);
        if self.pause_at == Some(phase) { let _ = self.resume_rx.recv(); }
    }
}

#[cfg(test)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum HookPhase { /* ... */ }
```

**Production inner functions take no hook parameter:**
```rust
// Production (no cfg(test) hook):
fn set_key_inner(state: &Arc<AppState>, uuid: &str, key: &str) -> Result<(), AppError> {
    let _gate = state.data_gate.read();
    // ... no hook.hit() calls ...
}

// Test-only variant with hook:
#[cfg(test)]
fn set_key_inner_with_hook(state: &Arc<AppState>, uuid: &str, key: &str, hook: &TestHook) -> Result<(), AppError> {
    hook.hit(HookPhase::SetKeyAttemptingRead);
    let _gate = state.data_gate.read();
    hook.hit(HookPhase::SetKeyAcquiredRead);
    // ... rest identical to production, with hook.hit() at boundaries ...
}
```

The test variant delegates the actual business logic (gate acquisition, keystore call, DB snapshot) to the same code paths — the only difference is the `hook.hit()` calls at phase boundaries. The hook never alters business logic.

**Alternative (cleaner, avoids code duplication):** use a trait-based hook that is a no-op in production:
```rust
pub trait MigrationHook {
    fn hit(&self, _phase: HookPhase) {}
}
pub struct NoopHook;
impl MigrationHook for NoopHook {}
// production passes &NoopHook; tests pass &TestHook
```
But this still puts `HookPhase` in the type system. The `#[cfg(test)]` dual-function approach is simpler and keeps the release binary completely free of test infrastructure. Either is acceptable; the constraint is: **`crossbeam-channel` must NOT be a release dependency.**

---

## Summary

| Item | Fix |
|------|-----|
| Test 1 recv order | `AttemptingRead` then `AcquiredRead` — both received |
| Test 2 recv order | `AttemptingWrite` → `AcquiredWrite` → `AfterClear` — all three received |
| `#[derive(Clone)] TestHook` | Explicitly declared |
| `crossbeam-channel` scope | `[dev-dependencies]` only; TestHook + HookPhase + hook params behind `#[cfg(test)]`; release binary has zero test infrastructure |

All prior amendments + rev-6 + erratum remain approved and unchanged.
