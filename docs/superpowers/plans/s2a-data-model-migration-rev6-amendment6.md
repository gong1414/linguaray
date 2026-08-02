# S2a rev-6 Amendment 6 — unified core, call-site consistency

**Status:** Final amendment. Patches amendment 5. Document-only.
All prior content (rev-6 + amendments 1–5 + erratum) approved and unchanged.

---

## P1 — test call-sites vs production signatures: single core, no duplication

### Problem
Amendment 5's test code calls `set_key_inner(..., Some(&h))` but the production `set_key_inner` takes no hook parameter (it was split into `set_key_inner_with_hook`). Also `_with_hook`'s "rest identical" duplicates business logic — tests don't cover the production code path. Archive has no `_with_hook` variant defined at all.

### Fix — one `*_core` function holds ALL business logic; thin wrappers dispatch

```rust
// ═══════════════════════════════════════════════════════
// set_key: single core, cfg(test) hook parameter
// ═══════════════════════════════════════════════════════

/// The ONE implementation of set_key. All business logic lives here.
/// The hook parameter exists only under cfg(test); production compiles
/// without it entirely.
fn set_key_core(
    state: &Arc<AppState>,
    uuid: &str,
    key: &str,
    #[cfg(test)] hook: Option<&TestHook>,
) -> Result<(), AppError> {
    #[cfg(test)]
    { if let Some(h) = hook { h.hit(HookPhase::SetKeyAttemptingRead); } }

    let _gate = state.data_gate.read();

    #[cfg(test)]
    { if let Some(h) = hook { h.hit(HookPhase::SetKeyAcquiredRead); } }

    let lock = state.lock_provider(uuid);
    let _plock = lock.lock();
    let db = state.db.read().clone().ok_or(AppError::NotReady)?;
    let profile = db.with_conn(|c| providers::get(c, uuid))?;
    if profile.status != "active" { return Err(AppError::NotCallable); }

    #[cfg(test)]
    { if let Some(h) = hook { h.hit(HookPhase::SetKeyBeforeWrite); } }

    state.keystore.set_provider_key(&profile.secret_ref, key)?;
    Ok(())
}

/// Production entry point — no hook.
fn set_key_inner(state: &Arc<AppState>, uuid: &str, key: &str) -> Result<(), AppError> {
    #[cfg(test)]
    { return set_key_core(state, uuid, key, None); }
    #[cfg(not(test))]
    { return set_key_core(state, uuid, key); }
}

/// Test-only entry point — with hook.
#[cfg(test)]
fn set_key_inner_with_hook(
    state: &Arc<AppState>, uuid: &str, key: &str, hook: &TestHook,
) -> Result<(), AppError> {
    set_key_core(state, uuid, key, Some(hook))
}
```

```rust
// ═══════════════════════════════════════════════════════
// archive_or_reset: same pattern
// ═══════════════════════════════════════════════════════

fn archive_or_reset_core(
    state: &Arc<AppState>,
    mode: ArchiveMode,
    #[cfg(test)] hook: Option<&TestHook>,
) -> Result<(), AppError> {
    #[cfg(test)]
    { if let Some(h) = hook { h.hit(HookPhase::ArchiveAttemptingWrite); } }

    let _gate = state.data_gate.write();

    #[cfg(test)]
    { if let Some(h) = hook { h.hit(HookPhase::ArchiveAcquiredWrite); } }

    state.keystore.archive_or_reset(mode)?;

    #[cfg(test)]
    { if let Some(h) = hook { h.hit(HookPhase::ArchiveAfterClear); } }

    post_archive_db_cleanup_locked(state)?;

    #[cfg(test)]
    { if let Some(h) = hook { h.hit(HookPhase::ArchiveDone); } }

    update_readiness_after_keystore_archive(state);
    Ok(())
}

fn archive_or_reset_keystore_inner(state: &Arc<AppState>, mode: ArchiveMode) -> Result<(), AppError> {
    #[cfg(test)]
    { return archive_or_reset_core(state, mode, None); }
    #[cfg(not(test))]
    { return archive_or_reset_core(state, mode); }
}

#[cfg(test)]
fn archive_or_reset_keystore_inner_with_hook(
    state: &Arc<AppState>, mode: ArchiveMode, hook: &TestHook,
) -> Result<(), AppError> {
    archive_or_reset_core(state, mode, Some(hook))
}
```

### Test call-sites — use `_with_hook` variants

**Test 1:**
```rust
s.spawn(move || {
    let _ = set_key_inner_with_hook(&state_a, uuid, "new-key", &h);
});
// ...
s.spawn(move || {
    let _ = archive_or_reset_keystore_inner_with_hook(&state_b, ArchiveMode::Archive, &h2);
});
```

**Test 2:**
```rust
s.spawn(move || {
    let _ = archive_or_reset_keystore_inner_with_hook(&state_b, ArchiveMode::Archive, &h);
});
// ...
s.spawn(move || {
    let _ = set_key_inner_with_hook(&state_a, uuid, "new-key", &h2);
});
```

### Properties
- **Single source of truth:** `set_key_core` / `archive_or_reset_core` contain the only copy of business logic. `hook.hit()` calls are inline but gated by `#[cfg(test)]` blocks that compile to nothing in release.
- **Production has zero test infrastructure:** `#[cfg(not(test))]` compiles `set_key_inner` → `set_key_core(state, uuid, key)` with no hook parameter, no `TestHook` type, no `HookPhase` enum, no `crossbeam-channel`.
- **Tests cover production code:** `_with_hook` calls the same `*_core` — the hook only pauses at boundaries, never alters logic.
- **No signature mismatch:** tests call `_with_hook` (takes hook); Tauri commands call `_inner` (no hook).

---

## Summary

| Item | Fix |
|------|-----|
| Call-site vs signature conflict | Unified `*_core` with `#[cfg(test)] hook` param; `_inner` (prod, no hook) and `_with_hook` (test) are thin wrappers |
| Business logic duplication | Eliminated — single `*_core` holds all logic; hook calls are `#[cfg(test)]` no-ops in release |
| Archive `_with_hook` | Defined: `archive_or_reset_keystore_inner_with_hook` |
| crossbeam-channel scope | Still `[dev-dependencies]` only; `#[cfg(test)]` gates everything |

All prior amendments + rev-6 + erratum remain approved and unchanged.
