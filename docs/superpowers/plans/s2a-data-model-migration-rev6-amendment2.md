# S2a rev-6 Amendment 2 — 2 final P1s + 2 defensive fixes

**Status:** Final amendment to `s2a-data-model-migration-rev6.md` + amendment 1.
**Document-only.** All prior content approved. This patches the last 2 P1s.

---

## P1 #1 — ArchiveFailpoint wired into the production code path

### Problem
Amendment 1 defined `ArchiveFailpointCell` but the example used undefined `afp`, compared without `PartialEq`, and the `RenameError` recovery (reopen canonical) didn't share the production `fs::rename` failure branch. Tests would validate a bypass, not real recovery code.

### Fix — `archive_database_inner` takes failpoint; production + test share branches

```rust
#[derive(Clone, PartialEq, Eq)]
pub enum ArchiveFailpoint {
    None,
    CloseError,
    RenameError,
    ReopenError,
    MigrationKeystoreCorrupt,
    MigrationOther,
}

pub struct ArchiveFailpointCell(pub parking_lot::Mutex<ArchiveFailpoint>);
impl ArchiveFailpointCell {
    pub fn none() -> Self { Self(parking_lot::Mutex::new(ArchiveFailpoint::None)) }
    pub fn get(&self) -> ArchiveFailpoint { self.0.lock().clone() }
}

/// Production path: Tauri command calls this with ArchiveFailpointCell::none().
/// Tests call this with a specific failpoint.
fn archive_database_inner(state: &Arc<AppState>, afp: &ArchiveFailpointCell) -> Result<(), AppError> {
    let _gate = state.data_gate.write();
    let mut db_slot = state.db.write();
    let old_db = db_slot.take();

    // ── CLOSE (unified branch: injected + real share same recovery) ──
    if let Some(arc) = old_db {
        let owned = match Arc::try_unwrap(arc) {
            Ok(db) => db,
            Err(back) => { *db_slot = Some(back); return Err(AppError::DbInUse); }
        };
        // Injected OR real close — both go through the same match:
        let close_result = if afp.get() == ArchiveFailpoint::CloseError {
            Err((owned, rusqlite::Error::InvalidParameterCount(0, 1)))  // deterministic fake
        } else {
            owned.close()  // Result<(), (Database, rusqlite::Error)>
        };
        if let Err((db_back, e)) = close_result {
            // Restore — DB is still usable (close failed but handle may persist):
            *db_slot = Some(Arc::new(db_back));
            return Err(AppError::CloseFailed(e));
        }
    }

    // ── RENAME (unified branch) ──
    let db_path = state.db_path();
    let broken = db_path.with_extension(format!("db.broken-{}", now()));
    let rename_result = if afp.get() == ArchiveFailpoint::RenameError {
        Err(std::io::Error::new(std::io::ErrorKind::Other, "injected rename error"))
    } else {
        fs::rename(&db_path, &broken)
    };
    if let Err(e) = rename_result {
        // Canonical file untouched at db_path. Reopen to restore slot:
        match Database::open(&db_path) {
            Ok(db) => { *db_slot = Some(Arc::new(db)); }  // restored
            Err(reopen_err) => {
                *db_slot = None;
                *state.readiness.write() = DataReadiness::NeedsDatabaseRecovery {
                    reason: format!("reopen after rename fail: {reopen_err}"),
                };
            }
        }
        return Err(AppError::RenameFailed(e));
    }

    // ── REOPEN (unified branch) ──
    let open_result = if afp.get() == ArchiveFailpoint::ReopenError {
        Err(rusqlite::Error::InvalidColumnName("injected".into()))
    } else {
        // Database::open returns its own error type; map here:
        Database::open(&db_path).map_err(|e| /* into rusqlite::Error or DbError */ e)
    };
    let new_db = match open_result {
        Ok(db) => Arc::new(db),
        Err(e) => {
            *db_slot = None;
            *state.readiness.write() = DataReadiness::NeedsDatabaseRecovery { reason: e.to_string() };
            return Err(AppError::ReopenFailed(e));
        }
    };

    // ── MIGRATION (unified branch) ──
    let mig_result = if afp.get() == ArchiveFailpoint::MigrationKeystoreCorrupt {
        Err(MigrationError::NeedsKeystoreRecovery("injected".into()))
    } else if afp.get() == ArchiveFailpoint::MigrationOther {
        Err(MigrationError::Other("injected".into()))
    } else {
        run_migration(&new_db, &state.keystore_dir(), &state.settings_path(), &FailpointCell::none())
    };
    match mig_result {
        Ok(()) => {
            *db_slot = Some(new_db);
            update_readiness_after_archive(state);
            Ok(())
        }
        Err(e) => {
            *db_slot = Some(new_db);
            let readiness = match e {
                MigrationError::NeedsKeystoreRecovery(r) => DataReadiness::NeedsKeystoreRecovery { reason: r },
                other => DataReadiness::MigrationIncomplete { reason: other.to_string() },
            };
            *state.readiness.write() = readiness;
            Err(e.into())
        }
    }
}
```

**Key property:** every failpoint (`CloseError`, `RenameError`, `ReopenError`, `Migration*`) enters the SAME match arm as a real failure. The test exercises the production recovery code (restore slot, reopen canonical, set readiness), not a bypass.

**Tauri command:**
```rust
#[tauri::command]
async fn archive_database(state: State<'_, Arc<AppState>>) -> Result<(), String> {
    let state = state.inner().clone();
    let afp = ArchiveFailpointCell::none();  // production: no injection
    spawn_blocking(move || archive_database_inner(&state, &afp))
        .await.map_err(flatten)?.map_err(flatten)
}
```

---

## P1 #2 — set_key × archive deterministic barrier tests

### Problem
Amendment 1's test description was self-contradictory: required "no orphan key" but then said a key written to a disabled profile after archive is acceptable. The ordering was ambiguous.

### Fix — two deterministic tests with explicit barriers

**Test A: set_key acquires read-gate FIRST**
```
Setup: provider P with needs_key=1, enabled=true, key="old-key" in keystore.
1. Thread A: calls set_key(P, "new-key")
   — acquires data_gate.read() (succeeds immediately)
   — acquires per-provider mutex
   — writes "new-key" to keystore
   — releases everything
2. Barrier: wait until Thread A completes (channel/drop signal).
3. Thread B: calls archive_keystore()
   — acquires data_gate.write() (A already released)
   — archives keystore (all keys gone)
   — DB cleanup: UPDATE providers SET enabled=0 WHERE needs_key=1
   — releases gate
Assert: keystore is empty (no "new-key" — it was archived). P.enabled = false.
        No orphan: keystore has no keys at all.
```

**Test B: archive acquires write-gate FIRST**
```
Setup: provider P with needs_key=1, enabled=true, key="old-key" in keystore.
1. Thread B: calls archive_keystore()
   — acquires data_gate.write()
   — archives keystore (keys gone)
   — DB cleanup: P.enabled = 0
   — releases gate
2. Barrier: wait until Thread B completes.
3. Thread A: calls set_key(P, "new-key")
   — acquires data_gate.read() (B released)
   — acquires per-provider mutex
   — reads P: status=active, enabled=false (set_key does NOT check enabled)
   — writes "new-key" to keystore.provider_keys[P.secret_ref]
   — releases
Assert: keystore has "new-key" under P.secret_ref. P.enabled = false (set_key
        did NOT re-enable). This is NOT an orphan: P is an active (non-deleted)
        row, the key is valid but P is not callable until user toggles enabled.
        The key is usable once the user explicitly enables P.
```

**Removed:** the contradictory "cleanup happens first → key doesn't exist" assertion. In Test B, cleanup does happen first, but set_key writes afterward — the key exists, and that's correct behavior (not an orphan).

---

## Defensive fix 1 — `update_readiness_after_archive` guards `Ready + db=None`

```rust
fn update_readiness_after_archive(state: &Arc<AppState>) {
    let mut readiness = state.readiness.write();
    let db_exists = state.db.read().is_some();
    match (&*readiness, db_exists) {
        // Defensive: Ready but no DB is an impossible/inconsistent state → force recovery:
        (Ready, false) => *readiness = DataReadiness::NeedsDatabaseRecovery {
            reason: "DB missing after archive".into(),
        },
        (NeedsDatabaseRecovery { .. }, _) => { /* keep */ }
        (Ready, true) | (NeedsKeystoreRecovery { .. }, true) => *readiness = Ready,
        (MigrationIncomplete { .. }, true) => { /* keep */ }
        (_, false) => { /* keep NeedsDatabaseRecovery or whatever */ }
    }
}
```

The `(Ready, false)` arm is a safety net — it should never occur, but if it does, it forces `NeedsDatabaseRecovery` rather than allowing an inconsistent `Ready + no DB`.

---

## Defensive fix 2 — preflight rejects invalid `migration_complete` values

```rust
// In migration_state_if_exists, after finding the table:
let complete: Option<i64> = conn.query_row(
    "SELECT migration_complete FROM _schema_migrations WHERE id=1",
    [], |r| r.get(0),
).optional()?;

match complete {
    None => Ok(MigrationState::Incomplete),      // row missing
    Some(0) => Ok(MigrationState::Incomplete),
    Some(1) => Ok(MigrationState::Complete),
    Some(other) => Err(DbError::IntegrityError(  // value outside {0,1} = schema corruption
        format!("invalid migration_complete value: {other}")
    )),
}
```

The DDL CHECK constraint (`CHECK (migration_complete IN (0,1))`) prevents this at insert time, but a manually-tampered DB or a schema bug could bypass it. The preflight treats any other value as an integrity error → `NeedsDatabaseRecovery`, not `Incomplete`.

---

## Summary

| Item | Fix |
|------|-----|
| ArchiveFailpoint production path | `archive_database_inner(state, afp)`; production passes `None`; failpoint + real error share same match branch |
| ArchiveFailpoint derive | `#[derive(Clone, PartialEq, Eq)]` |
| set_key × archive test | Two deterministic barrier tests (A: set_key first, B: archive first); removed contradictory assertion |
| Ready + db=None | Defensive arm forces `NeedsDatabaseRecovery` |
| migration_complete invalid value | Treated as integrity error, not Incomplete |

All prior amendments + rev-6 + erratum content remains approved and unchanged.
