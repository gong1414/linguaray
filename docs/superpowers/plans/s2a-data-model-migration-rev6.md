# S2a — Data Model + Migration: Implementation Plan (rev-6)

**Branch:** `codex/s2a-data-model-migration`. **Backend-only.** **PLAN ONLY.**
Depends on revised `s0-erratum-phase5-verification.md` (committed alongside).

Resolves all 4 P1s from rev-5 review. All previously-approved items (lock hierarchy, HashSet consent dedup, `needs_key` DB column, Google catalog, history CHECK, origin-only consent, M4–M10 real disk inspection, failpoint PartialEq) are carried forward unchanged.

| P1 | § |
|----|---|
| #1 Completed migration still reads settings + backs up | §1.4 (preflight) |
| #2 Profile migration pseudocode has 3 compile errors | §3 (CandidateSource types) |
| #3 archive_database can't compile + incomplete failure recovery | §7 (Arc + close + failure table) |
| #4 set_key/enabled semantics contradict + touch_updated_at undefined | §6 (semantics fixed) |

---

## 1. Migration — preflight before any backup/mutation (P1 #1)

### 1.1–1.3 (unchanged from rev-5)
Database API (`with_conn` passes `&mut Connection`), open (dir-first, synchronous=FULL, rusqlite 0.40.1), lock-order rule (data_gate → per-provider mutex → DB/keystore, never nested).

### 1.4 run_migration — read-only preflight BEFORE Phase 1

```rust
pub fn run_migration(db: &Database, keystore_dir: &Path, settings_path: &Path, fp: &FailpointCell) -> Result<(), MigrationError> {
    // ── PREFLIGHT: read-only check if migration already complete ──
    // This is BEFORE any backup or mutation. It does NOT parse settings,
    // does NOT create backups. If complete → preflight keystore + return.
    match schema::migration_state_if_exists(db)? {
        MigrationState::Complete => {
            // Migration done on a prior run. Only verify keystore is still readable.
            preflight_keystore(keystore_dir)?;
            return Ok(());
        }
        MigrationState::Incomplete | MigrationState::NotStarted => { /* proceed */ }
    }

    // ── PHASE 1: Backup (FIRST persistent action) ──
    let settings = parse_settings_raw(settings_path)?;
    backup_settings(settings_path)?;
    backup_keystore(keystore_dir)?;    // Missing/CurrentV2 → no-op; only backs up LegacyV1
    fp.maybe_fail(Failpoint::AfterBackup)?;

    // ── PHASE 2: DB schema ──
    db.with_conn(|conn| { let tx = conn.transaction()?; schema::create_all_tables(&tx)?; schema::seed_singletons(&tx)?; tx.commit() })?;
    fp.maybe_fail(Failpoint::AfterSchema)?;

    // (rest identical to rev-5: seed preferences, enumerate candidates,
    //  insert profiles, keystore rewrite, verify + complete)
    // ...

    fp.maybe_fail(Failpoint::AfterCompleteCommit)?;  // renamed from BeforeComplete
    Ok(())
}
```

**`migration_state_if_exists`:**
```rust
enum MigrationState { NotStarted, Incomplete, Complete }
fn migration_state_if_exists(db: &Database) -> Result<MigrationState, DbError> {
    db.with_conn(|conn| {
        // Check if _schema_migrations table exists (read-only):
        let exists: bool = conn.query_row(
            "SELECT 1 FROM sqlite_master WHERE type='table' AND name='_schema_migrations'", [], |_| Ok(true)
        ).unwrap_or(false);
        if !exists { return Ok(MigrationState::NotStarted); }
        let complete: bool = conn.query_row(
            "SELECT migration_complete FROM _schema_migrations WHERE id=1", [], |r| r.get(0)
        ).unwrap_or(false);
        Ok(if complete { MigrationState::Complete } else { MigrationState::Incomplete })
    })
}
```

**`backup_keystore` semantics:** only backs up `LegacyV1` (the only state that needs migration). `Missing` → no-op (nothing to back up). `CurrentV2` → no-op (already migrated, no v1 to preserve). `Corrupt` → returns `Err` (cannot back up unreadable file; migration stops → `NeedsKeystoreRecovery`).

**`preflight_keystore`:** loads keystore state; if `Corrupt` → returns `Err(NeedsKeystoreRecovery)`. Otherwise Ok. This catches a keystore that went corrupt after a previously-complete migration.

---

## 2. DDL — boolean CHECKs added (P2)

Same as rev-5 §2, with these additions:
```sql
migration_complete INTEGER NOT NULL DEFAULT 0 CHECK (migration_complete IN (0,1)),
-- in history_sessions:
is_favorite INTEGER NOT NULL DEFAULT 0 CHECK (is_favorite IN (0,1)),
-- in preferences:
history_enabled INTEGER NOT NULL DEFAULT 0 CHECK (history_enabled IN (0,1)),
```
All boolean columns now have `(0,1)` CHECK constraints.

---

## 3. CandidateSource + build_profile — types fixed, signatures match (P1 #2)

### 3.1 CandidateSource with stable_id
```rust
pub enum CandidateSource {
    LegacyId(String),
    ProviderKey(String),   // a "provider/<uuid>" key from v2 keystore (DB-loss recovery)
}

impl CandidateSource {
    /// Stable identifier for failpoint matching and deterministic UUID generation.
    /// For LegacyId: the legacy_id itself.
    /// For ProviderKey: the full secret_ref string.
    pub fn stable_id(&self) -> &str {
        match self { CandidateSource::LegacyId(s) | CandidateSource::ProviderKey(s) => s }
    }

    /// Deterministic UUID for this candidate.
    /// LegacyId → UUIDv5("linguaray:legacy-provider:" + id)
    /// ProviderKey → UUIDv5("linguaray:recovered-key:" + secret_ref)
    pub fn deterministic_uuid(&self) -> Uuid {
        match self {
            CandidateSource::LegacyId(id) => uuid_v5("linguaray:legacy-provider:", id),
            CandidateSource::ProviderKey(ref_str) => uuid_v5("linguaray:recovered-key:", ref_str),
        }
    }
}
```

### 3.2 build_profile — 2 args, types consistent
```rust
/// Build a ProviderProfile from a candidate source + the current keystore state.
/// ks is used to determine if the candidate has a key (affects nothing in S2a —
/// key existence is checked at runtime, not stored in the profile row).
fn build_profile(source: &CandidateSource, ks: &KeystoreLoadState) -> Result<ProviderProfile, MigrationError> {
    let uuid = source.deterministic_uuid().to_string();
    match source {
        CandidateSource::LegacyId(id) => {
            if let Some(preset) = find_preset(id) {
                Ok(profile_from_preset(&uuid, preset))
            } else if let Some(tc) = traditional_catalog().iter().find(|t| t.template_id == *id) {
                Ok(profile_from_traditional(&uuid, tc))
            } else {
                // Unknown legacy_id: repair profile. UUID is deterministic UUIDv5.
                Ok(repair_profile(&uuid, id))  // secret_ref = legacy_id (same as normal migration)
            }
        }
        CandidateSource::ProviderKey(secret_ref) => {
            // DB-loss recovery. Try to parse UUID from "provider/<uuid>":
            let final_uuid = if let Some(rest) = secret_ref.strip_prefix("provider/") {
                Uuid::parse_str(rest).map(|u| u.to_string()).unwrap_or_else(|_| uuid.clone())
            } else { uuid.clone() };
            Ok(repair_profile(&final_uuid, secret_ref))
        }
    }
}

fn repair_profile(uuid: &str, secret_ref: &str) -> ProviderProfile {
    ProviderProfile {
        uuid: uuid.into(), secret_ref: secret_ref.into(),
        template_id: "unknown".into(), protocol: Protocol::CustomHttp,
        endpoint: "".into(), model: None, enabled: false,
        is_local: false, needs_key: 1, status: "active".into(), sort_order: 999,
        capabilities: "{}".into(),
    }
}
```

**No `repair_profile(id, id)`** — the UUID is always `source.deterministic_uuid()`, never the raw legacy_id. `"custom-engine"` → `UUIDv5("linguaray:legacy-provider:custom-engine")`.

### 3.3 Coordinator call site — consistent types
```rust
let candidates = enumerate_candidates(&ks_state, settings.as_ref());  // Vec<CandidateSource>
for source in &candidates {
    let profile = build_profile(source, &ks_state)?;
    db.with_conn(|conn| { let tx = conn.transaction()?; providers::insert_or_ignore(&tx, &profile)?; tx.commit() })?;
    fp.maybe_fail(Failpoint::AfterProfileInsert(source.stable_id().to_owned()))?;  // String, matches PartialEq
}
```

`source.stable_id().to_owned()` → `String`, matches `Failpoint::AfterProfileInsert(String)` in the `PartialEq` comparison.

### 3.4 M15 assertion
M15 (unknown legacy_id) asserts: the profile's UUID is a valid UUID string AND is identical across re-runs (deterministic UUIDv5 from the same legacy_id).

---

## 4. Keystore typed state + backup (unchanged from rev-5)

`KeystoreLoadState { Missing, LegacyV1, CurrentV2, Corrupt }`. `backup_keystore`: LegacyV1 only, create-new, under keystore lock, 0600/DACL. Missing/CurrentV2 → no-op. Corrupt → Err.

---

## 5. Consent — strict scope, HashSet dedup (unchanged from rev-5)

`compute_scope` with `HashSet::insert` duplicate detection, `once().chain()`, origin-only invalidation. Single-transaction confirm.

---

## 6. Provider mutation coordinator — set_key/enabled semantics fixed (P1 #4)

### 6.1 Lock hierarchy (unchanged)
`data_gate (RwLock) → per-provider mutex (parking_lot::Mutex) → DB/keystore (never nested)`

### 6.2 set_key — writes key ONLY, does NOT modify enabled
```rust
fn set_key(state: &Arc<AppState>, uuid: &str, key: &str) -> Result<(), AppError> {
    let _gate = state.data_gate.read();
    let lock = state.lock_provider(uuid);
    let _plock = lock.lock();
    let db = state.db.read().clone().ok_or(AppError::NotReady)?;
    // 1. DB snapshot:
    let profile = db.with_conn(|c| providers::get(c, uuid))?;
    if profile.status != "active" { return Err(AppError::NotCallable); }
    // 2. Keystore write (DB NOT locked):
    state.keystore.set_provider_key(&profile.secret_ref, key)?;
    // 3. No DB finalize needed — key existence is a runtime keystore lookup.
    //    enabled is NOT modified by set_key.
    Ok(())
}
```

**No `touch_updated_at`** — removed entirely. `set_key` writes only to the keystore. Enabling a provider is a separate `provider_toggle(uuid, true)` call. This avoids silently re-enabling profiles the user had disabled.

### 6.3 delete — single `?` on begin_delete
```rust
fn delete(state: &Arc<AppState>, uuid: &str) -> Result<(), AppError> {
    let _gate = state.data_gate.read();
    let lock = state.lock_provider(uuid);
    let _plock = lock.lock();
    let db = state.db.read().clone().ok_or(AppError::NotReady)?;
    // Step 1: DB tx — returns secret_ref.
    let secret_ref = db.with_conn(|c| providers::begin_delete(c, uuid))?;
    // Step 2: keystore remove (idempotent).
    state.keystore.remove_provider_key(&secret_ref)?;
    // Step 3: DB tx — tombstone.
    db.with_conn(|c| providers::finalize_delete(c, uuid))?;
    Ok(())
}
```

`with_conn(...)?` — single `?`, not `??`. `begin_delete` returns `Result<String, DbError>` (the secret_ref), and `with_conn` returns `Result<T, DbError>`, so one `?` unwraps both.

### 6.4 Post keystore archive/reset — conditional Ready
```rust
fn post_keystore_archive(state: &Arc<AppState>) -> Result<(), AppError> {
    let _write_gate = state.data_gate.write();
    // Only proceed if DB exists:
    let db_guard = state.db.read();
    let db = match db_guard.as_ref() {
        Some(db) => db.clone(),
        None => {
            // DB not available — leave readiness as-is (NeedsDatabaseRecovery).
            // Keystore archive doesn't fix a DB problem.
            return Ok(());
        }
    };
    drop(db_guard);
    // Cleanup transaction:
    db.with_conn(|conn| {
        let tx = conn.transaction()?;
        tx.execute("UPDATE providers SET enabled=0 WHERE needs_key=1", [])?;
        tx.execute("UPDATE preferences SET primary_uuid=NULL, parallel_uuids='[]', fallback_uuid=NULL, parallel_consent_scope=NULL, parallel_consent_version=NULL WHERE id=1", [])?;
        tx.execute("UPDATE _schema_migrations SET migration_complete=1 WHERE id=1", [])?;
        tx.commit()
    })?;
    // Only set Ready if no other blocker:
    *state.readiness.write() = DataReadiness::Ready;
    Ok(())
}
```

---

## 7. archive_database — Arc<AppState>, Database::close, failure recovery table (P1 #3)

### 7.1 AppState managed as Arc (matches existing Session pattern)
```rust
// setup:
app.manage(Arc::new(AppState { data_gate, provider_locks, db, keystore, client, readiness }));

// commands (matches existing codebase pattern State<'_, Arc<Session>>):
#[tauri::command]
async fn archive_database(state: State<'_, Arc<AppState>>) -> Result<(), String> {
    let state = state.inner().clone();  // Arc<AppState>, satisfies 'static
    spawn_blocking(move || { /* ... */ }).await.map_err(flatten)?
}
```

This matches the existing `State<'_, Arc<Session>>` + `state.inner().clone()` pattern in `lib.rs:64,358`.

### 7.2 Database::close — explicit Connection::close
```rust
impl Database {
    /// Explicitly close the connection. Consumes self.
    /// Returns the rusqlite result so close errors propagate.
    pub fn close(self) -> rusqlite::Result<()> {
        let conn = self.conn.into_inner();  // parking_lot::Mutex::into_inner
        conn.close()
    }
}
```

### 7.3 archive_database — full failure recovery
```rust
spawn_blocking(move || {
    let _write_gate = state.data_gate.write();       // block all provider commands
    let mut db_slot = state.db.write();
    let old_db = db_slot.take();                       // slot = None

    let close_result = match old_db {
        Some(arc) => match Arc::try_unwrap(arc) {
            Ok(db) => db.close(),                      // explicit close, propagates error
            Err(arc_back) => { *db_slot = Some(arc_back); return Err("DB still in use".into()); }
        },
        None => Ok(()),                                // no DB to close
    };

    // ── Failure recovery table ──
    // close failed: restore? No — the Connection is consumed by close() even on error.
    //   rusqlite::Connection::close() takes self; on error the connection is still dropped.
    //   The file handle IS released (Drop runs). We can proceed to rename.
    if let Err(e) = close_result {
        log::warn!("DB close error (handle still released via Drop): {e}");
    }

    // rename:
    let db_path = /* ... */;
    let broken_path = db_path.with_extension(format!("db.broken-{}", now()));
    if let Err(e) = fs::rename(&db_path, &broken_path) {
        // rename failed: slot is None, canonical file untouched at original path.
        // readiness: DB is effectively gone (slot None). Mark NeedsDatabaseRecovery.
        *state.readiness.write() = DataReadiness::NeedsDatabaseRecovery {
            reason: format!("rename failed: {e}"),
        };
        return Err(format!("rename failed: {e}"));
    }

    // open fresh + migrate:
    match Database::open(&db_path) {
        Ok(new_db) => {
            let new_db = Arc::new(new_db);
            match run_migration(&new_db, &keystore_dir, &settings_path, &FailpointCell::none()) {
                Ok(()) => {
                    *db_slot = Some(new_db);
                    // Verify keystore before Ready:
                    match keystore::load_state(&keystore_dir) {
                        Corrupt(e) => *state.readiness.write() = NeedsKeystoreRecovery { reason: e.to_string() },
                        _ => *state.readiness.write() = Ready,
                    }
                    Ok(())
                }
                Err(e) => {
                    *db_slot = Some(new_db);  // DB exists but migration incomplete
                    let readiness = match &e {
                        MigrationError::NeedsKeystoreRecovery(r) => NeedsKeystoreRecovery { reason: r },
                        _ => MigrationIncomplete { reason: e.to_string() },
                    };
                    *state.readiness.write() = readiness;
                    Err(format!("re-migration failed: {e}"))
                }
            }
        }
        Err(e) => {
            // reopen failed: slot stays None, broken file exists at broken_path.
            *state.readiness.write() = NeedsDatabaseRecovery { reason: e.to_string() };
            Err(format!("DB reopen failed: {e}"))
        }
    }
})
```

### 7.4 archive_database failure injection tests
| Scenario | Failpoint | Slot after | Readiness after | Canonical file |
|----------|-----------|------------|-----------------|----------------|
| Close error | Inject rusqlite close error | None → reopened | Ready (or recovery per migration) | renamed to .broken, fresh created |
| Rename fails | `fs::rename` returns error | None | NeedsDatabaseRecovery | original untouched at db_path |
| Reopen fails | `Database::open` returns error | None | NeedsDatabaseRecovery | .broken exists, no new db |
| Re-migration KeystoreCorrupt | run_migration returns KeystoreCorrupt | Some(new_db) | NeedsKeystoreRecovery | fresh db, keystore corrupt |
| Re-migration other error | run_migration returns other | Some(new_db) | MigrationIncomplete | fresh db, migration partial |

---

## 8. Failpoint — AfterCompleteCommit rename (P2)

`BeforeComplete` renamed to `AfterCompleteCommit`. It fires AFTER the `UPDATE _schema_migrations SET migration_complete=1` commit succeeds but before the function returns Ok. M10 asserts: `complete=1` is committed (visible via separate read connection), re-run is a no-op early return.

---

## 9. M4 fixture clarification (P2)

M4 (`AfterBackup`) fixture **must** include both legacy `settings.json` AND a `LegacyV1` keystore, so both `.bak-pre-migration` files are created. Without a LegacyV1 keystore, `backup_keystore` is a no-op and only the settings backup exists. The test setup explicitly seeds both.

---

## 10. Test matrix (unchanged M1–M17 from rev-5, with M4 fixture clarified + M10 renamed + M15 UUID assertion + archive_database failure tests from §7.4)

---

## 11. Build sequence (unchanged from rev-5)

---

## 12. Verification gate
S0 §11: migration (M1–M17 real failpoints + archive_database failure tests), CRUD (§12 of rev-5), keystore v2 (§13 of rev-5). Root `src/` untouched.
