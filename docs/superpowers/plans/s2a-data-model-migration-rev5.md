# S2a — Data Model + Migration: Implementation Plan (rev-5)

**Branch:** `codex/s2a-data-model-migration`. **Backend-only.** **PLAN ONLY.**
Depends on revised `s0-erratum-phase5-verification.md` (committed alongside, dated 2026-08-02).

Resolves all 7 P1s from rev-4 review:

| P1 | § |
|----|---|
| #1 Phase order violates S0 + AfterBackup only backs up settings | §1.4 |
| #2 Failpoint discriminant match is wrong + missing checkpoints | §10 |
| #3 Windows file handle not guaranteed closed | §7 |
| #4 ProviderCoordinator undecided + putex typo + mark_has_key undefined | §6 |
| #5 needs_key not uniquely computable | §2 (DB column) + erratum |
| #6 Consent duplicate-recipient check never triggers | §5 |
| #7 DB-loss repair not in profile builder | §3 |

---

## 1. Migration — correct phase order, no lock nesting

### 1.1 Database API (unchanged)
```rust
pub struct Database { conn: parking_lot::Mutex<rusqlite::Connection> }
impl Database {
    pub fn with_conn<T>(&self, f: impl FnOnce(&mut Connection) -> Result<T, DbError>) -> Result<T, DbError> {
        let mut conn = self.conn.lock(); f(&mut *conn)
    }
}
```

### 1.2 Open — dir-first, synchronous=FULL, rusqlite 0.40.1
```rust
pub fn open(path: &Path) -> Result<Self, DbError> {
    let dir = path.parent().unwrap_or(Path::new("."));
    fs::create_dir_all(dir)?;
    fs_acl::secure_dir(dir)?;               // 0700 / DACL on dir FIRST
    let conn = rusqlite::Connection::open(path)?;
    fs_acl::secure_file(path)?;             // 0600 / DACL on file
    conn.pragma_update(None, "foreign_keys", "ON")?;
    conn.pragma_update(None, "busy_timeout", 5000)?;
    conn.pragma_update(None, "journal_mode", "DELETE")?;
    conn.pragma_update(None, "synchronous", "FULL")?;
    Ok(Self { conn: parking_lot::Mutex::new(conn) })
}
```
`tauri_plugin_store::resolve_store_path` (crate-root re-export). `rusqlite = "0.40.1"` + `bundled`.

### 1.3 Lock-order rule
DB Mutex + keystore flock **never nested**. Cross-store ops follow: data-gate read → per-provider mutex → DB snapshot (short tx) → unlock DB → keystore op → DB finalize (short tx). See §6.

### 1.4 run_migration — Phase 1 (backup) BEFORE Phase 2 (schema), per frozen S0 §8.5

```rust
pub fn run_migration(db: &Database, keystore_dir: &Path, settings_path: &Path, fp: &FailpointCell) -> Result<(), MigrationError> {
    // ── PHASE 1: Backup (FIRST, per frozen S0 §8.5) ──
    // Both backups created here, BEFORE any schema. Create-new only.
    let settings = parse_settings_raw(settings_path)?;
    backup_settings(settings_path)?;                        // settings.json → .bak-pre-migration
    backup_keystore(keystore_dir)?;                         // keystore.json → .bak-pre-migration (under keystore lock)
    fp.maybe_fail(Failpoint::AfterBackup)?;

    // ── PHASE 2: DB schema (idempotent, short tx) ──
    db.with_conn(|conn| {
        let tx = conn.transaction()?;
        schema::create_all_tables(&tx)?;
        schema::seed_singletons(&tx)?;
        tx.commit()
    })?;
    fp.maybe_fail(Failpoint::AfterSchema)?;

    // Idempotent early return.
    if db.with_conn(|c| schema::migration_complete(c))? { return Ok(()); }

    // ── PHASE 2b: Seed preferences from settings (short tx) ──
    db.with_conn(|conn| seed_preferences(conn, settings.as_ref()))?;
    fp.maybe_fail(Failpoint::AfterPreferences)?;

    // ── PHASE 3: Enumerate candidates + insert profiles ──
    let ks_state = keystore::load_state(keystore_dir);      // DB NOT locked
    match &ks_state {
        KeystoreLoadState::Corrupt(e) => return Err(MigrationError::NeedsKeystoreRecovery(e.to_string())),
        _ => {}
    }
    let candidates = enumerate_candidates(&ks_state, settings.as_ref());
    // Deterministic order: BTreeSet → sorted. Tests know the exact order.
    for legacy_id in &candidates {
        let profile = build_profile(legacy_id, &ks_state, settings.as_ref())?;
        db.with_conn(|conn| {
            let tx = conn.transaction()?;
            providers::insert_or_ignore(&tx, &profile)?;
            tx.commit()
        })?;
        fp.maybe_fail(Failpoint::AfterProfileInsert(legacy_id.clone()))?;
    }
    db.with_conn(|conn| seed_selection(conn, settings.as_ref(), &candidates))?;
    fp.maybe_fail(Failpoint::AfterProfiles)?;

    // ── PHASE 4: Keystore rewrite (DB NOT locked) ──
    if let KeystoreLoadState::LegacyV1(map) = &ks_state {
        keystore::migrate_to_v2(keystore_dir, map.clone())?;  // under keystore lock only
    }
    fp.maybe_fail(Failpoint::AfterKeystoreRewrite)?;

    // ── PHASE 5: Verify + complete ──
    let ks_after = keystore::load_state(keystore_dir);
    db.with_conn(|conn| {
        verify_key_bearing_profiles(conn, &ks_after)?;
        let tx = conn.transaction()?;
        tx.execute("UPDATE _schema_migrations SET migration_complete=1, migrated_at=? WHERE id=1", params![now()])?;
        tx.commit()
    })?;
    fp.maybe_fail(Failpoint::BeforeComplete)?;  // AFTER the commit — tests "complete not yet inspectable before crash"
    Ok(())
}
```

**`backup_keystore`:** performed under the keystore fs2 lock (same lock as any keystore RMW). Create-new only: if `.bak-pre-migration` exists, skip. This ensures BOTH backups exist at `AfterBackup`, before schema.

---

## 2. DDL — needs_key column + domain CHECKs (P1 #5, P2)

```sql
CREATE TABLE IF NOT EXISTS _schema_migrations (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    schema_version INTEGER NOT NULL,
    migration_complete INTEGER NOT NULL DEFAULT 0,
    migration_checkpoint TEXT,
    migrated_at INTEGER
);

CREATE TABLE IF NOT EXISTS preferences (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    target_language TEXT NOT NULL DEFAULT 'zh',
    primary_uuid TEXT,
    parallel_uuids TEXT NOT NULL DEFAULT '[]',
    fallback_uuid TEXT,
    parallel_consent_version INTEGER,
    parallel_consent_scope TEXT,
    history_enabled INTEGER NOT NULL DEFAULT 0,
    history_retention_days INTEGER NOT NULL DEFAULT 30
);

CREATE TABLE IF NOT EXISTS providers (
    uuid TEXT PRIMARY KEY,
    template_id TEXT NOT NULL,
    name TEXT NOT NULL,
    protocol TEXT NOT NULL CHECK (protocol IN ('openai_chat','anthropic','gemini','google_translate','custom_http')),
    endpoint TEXT NOT NULL,
    model TEXT,
    enabled INTEGER NOT NULL DEFAULT 1 CHECK (enabled IN (0,1)),
    sort_order INTEGER NOT NULL DEFAULT 0,
    is_local INTEGER NOT NULL DEFAULT 0 CHECK (is_local IN (0,1)),
    needs_key INTEGER NOT NULL CHECK (needs_key IN (0,1)),
    secret_ref TEXT NOT NULL UNIQUE,
    capabilities TEXT NOT NULL DEFAULT '{}',
    status TEXT NOT NULL DEFAULT 'active' CHECK (status IN ('active','deleting','deleted'))
);
CREATE INDEX IF NOT EXISTS idx_providers_status ON providers(status);

CREATE TABLE IF NOT EXISTS shortcuts ( action TEXT PRIMARY KEY, keys TEXT NOT NULL );

CREATE TABLE IF NOT EXISTS history_sessions (
    session_uuid TEXT PRIMARY KEY, timestamp INTEGER NOT NULL,
    trigger_source TEXT NOT NULL, detected_language TEXT,
    target_language TEXT NOT NULL, is_favorite INTEGER NOT NULL DEFAULT 0,
    source_text_encrypted BLOB NOT NULL, source_text_nonce BLOB NOT NULL,
    crypto_version INTEGER NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_history_sessions_ts ON history_sessions(timestamp DESC);

CREATE TABLE IF NOT EXISTS history_results (
    result_uuid TEXT PRIMARY KEY,
    session_uuid TEXT NOT NULL REFERENCES history_sessions(session_uuid) ON DELETE CASCADE,
    provider_uuid TEXT NOT NULL,
    provider_name_snapshot TEXT NOT NULL, engine_id TEXT NOT NULL, elapsed_ms INTEGER NOT NULL,
    outcome_tag TEXT NOT NULL CHECK (outcome_tag IN ('success','failure')),
    result_text_encrypted BLOB, result_text_nonce BLOB,
    error_kind TEXT, error_message_encrypted BLOB, error_message_nonce BLOB,
    crypto_version INTEGER NOT NULL,
    CHECK (
        (outcome_tag='success' AND result_text_encrypted IS NOT NULL AND result_text_nonce IS NOT NULL
         AND error_kind IS NULL AND error_message_encrypted IS NULL AND error_message_nonce IS NULL)
        OR (outcome_tag='failure' AND error_kind IS NOT NULL AND result_text_encrypted IS NULL AND result_text_nonce IS NULL
         AND (error_message_encrypted IS NULL AND error_message_nonce IS NULL
              OR error_message_encrypted IS NOT NULL AND error_message_nonce IS NOT NULL))
    )
);

CREATE TABLE IF NOT EXISTS vocabulary (
    item_uuid TEXT PRIMARY KEY, timestamp INTEGER NOT NULL,
    source_language TEXT NOT NULL, target_language TEXT NOT NULL,
    word_encrypted BLOB NOT NULL, word_nonce BLOB NOT NULL,
    definition_encrypted BLOB NOT NULL, definition_nonce BLOB NOT NULL,
    crypto_version INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS dict_packages (
    package_id TEXT PRIMARY KEY, name TEXT NOT NULL, version TEXT NOT NULL, installed_at INTEGER NOT NULL
);
```

**P2 CHECKs added:** `protocol`, `status`, `enabled`, `is_local`, `needs_key` all domain-constrained. `needs_key` is a real stored column (not derived) — unambiguous, executable in SQL.

---

## 3. Profile builder — includes DB-loss repair (P1 #7)

```rust
fn build_profile(source: &CandidateSource, ks: &KeystoreLoadState) -> Result<ProviderProfile, MigrationError> {
    match source {
        // ── Normal migration: legacy_id from flat map or settings ──
        CandidateSource::LegacyId(id) => {
            if let Some(preset) = presets().find(id) {
                Ok(profile_from_preset(preset))           // AI/local preset
            } else if let Some(tc) = traditional_catalog().iter().find(|t| t.template_id == *id) {
                Ok(profile_from_traditional(tc))          // Google etc.
            } else {
                Ok(repair_profile(id, id))                // unknown: custom_http, endpoint="", needs_key=1, enabled=false
            }
        }
        // ── DB-loss recovery: "provider/<uuid>" key from v2 keystore ──
        CandidateSource::ProviderKey(secret_ref) => {
            // Try to parse the UUID from "provider/<uuid>":
            if let Some(uuid) = secret_ref.strip_prefix("provider/") {
                if let Ok(parsed) = Uuid::parse_str(uuid) {
                    return Ok(ProviderProfile {
                        uuid: parsed.to_string(),
                        secret_ref: secret_ref.clone(),
                        template_id: "unknown".into(), protocol: Protocol::CustomHttp,
                        endpoint: "".into(), model: None, enabled: false,
                        is_local: false, needs_key: 1, status: "active",
                        ..repair_defaults()
                    });
                }
            }
            // Unparseable: deterministic UUIDv5 from full secret_ref (idempotent replay).
            let uuid = uuid_v5("linguaray:recovered-key:", secret_ref);
            Ok(repair_profile(&uuid.to_string(), secret_ref))
        }
    }
}

fn repair_profile(uuid: &str, secret_ref: &str) -> ProviderProfile {
    ProviderProfile {
        uuid: uuid.into(), secret_ref: secret_ref.into(),
        template_id: "unknown".into(), protocol: Protocol::CustomHttp,
        endpoint: "".into(), model: None, enabled: false,
        is_local: false, needs_key: 1, status: "active", sort_order: 999,
        capabilities: default_capabilities(),
    }
}
```

`CandidateSource` enum distinguishes legacy IDs from v2 `provider/<uuid>` keys:

```rust
fn enumerate_candidates(ks: &KeystoreLoadState, settings: Option<&RawSettings>) -> Vec<CandidateSource> {
    let mut out: BTreeSet<String> = BTreeSet::new();  // deterministic sorted order
    match ks {
        LegacyV1(map) | CurrentV2(KeystoreData { provider_keys: map, .. }) => {
            for key in map.keys() { out.insert(key.clone()); }
        }
        _ => {}
    }
    if let Some(s) = settings {
        if let Some(dp) = &s.default_provider { out.insert(dp.clone()); }
        if let Some(fb) = &s.fallback_engine { out.insert(fb.clone()); }
    }
    out.into_iter().map(|key| {
        if key.starts_with("provider/") { CandidateSource::ProviderKey(key) }
        else { CandidateSource::LegacyId(key) }
    }).collect()
}
```

---

## 4. Keystore typed state + backup (unchanged from rev-4)

`KeystoreLoadState { Missing, LegacyV1(map), CurrentV2(data), Corrupt(err) }`. Backup: create-new, under keystore lock, 0600/DACL. Corrupt → no auto-archive.

---

## 5. Consent — strict scope with HashSet dedup (P1 #6)

### 5.1 compute_scope — no silent drops, HashSet duplicate detection
```rust
fn compute_scope(primary: &str, parallel: &[String], providers: &[ProviderProfile]) -> Result<String, ConsentError> {
    let by_uuid: HashMap<&str, &ProviderProfile> = providers.iter().map(|p| (p.uuid.as_str(), p)).collect();
    let mut seen: HashSet<String> = HashSet::new();
    let mut recipients: Vec<(String, String)> = Vec::with_capacity(1 + parallel.len());

    for uuid in once(primary.to_owned()).chain(parallel.iter().cloned()) {
        // Duplicate detection FIRST (before any lookup):
        if !seen.insert(uuid.clone()) {
            return Err(ConsentError::DuplicateRecipient(uuid));
        }
        let p = by_uuid.get(uuid.as_str()).ok_or(ConsentError::UnknownRecipient(uuid))?;
        if !p.enabled || p.status != "active" { return Err(ConsentError::NotCallable(uuid)); }
        recipients.push((uuid, normalize_origin(&p.endpoint)?));
    }
    recipients.sort_by(|a, b| a.0.cmp(&b.0));
    let parts: Vec<String> = recipients.iter().map(|(u, o)| format!("{}|{}", u, o)).collect();
    Ok(format!("v1:{{{}}}", parts.join(",")))
}
```

`HashSet::insert` returns false on duplicate → `DuplicateRecipient` error. This catches primary-in-parallel and parallel-internal duplicates.

### 5.2 Origin-only invalidation
`normalize_origin` = scheme + host + port. `provider_update` invalidates consent only if `normalize_origin(old) != normalize_origin(new)`. Path/query changes preserve consent.

### 5.3 Single-transaction confirm (unchanged from rev-4)
`provider_confirm_and_set_active`: re-read providers, recompute scope server-side, check `expected_scope`, write selection + consent in one tx.

### 5.4 Consent tests
- primary duplicated in parallel → `DuplicateRecipient`
- parallel internal duplicate → `DuplicateRecipient`
- missing UUID → `UnknownRecipient`
- disabled UUID → `NotCallable`
- same origin, different path → scope unchanged (consent preserved)

---

## 6. Provider mutation coordinator — concrete protocol (P1 #4)

### 6.1 Lock hierarchy (decided — no more "or")
```
data_gate (RwLock) → per-provider mutex → DB / keystore (never nested)
```

```rust
pub struct AppState {
    pub data_gate: parking_lot::RwLock<()>,           // global: read for provider cmds, write for archive/reset/recovery
    pub provider_locks: parking_lot::Mutex<HashMap<String, Arc<parking_lot::Mutex<()>>>>,
    pub db: parking_lot::RwLock<Option<Arc<Database>>>,
    pub keystore: Keystore,
    pub client: reqwest::Client,
    pub readiness: parking_lot::RwLock<DataReadiness>,
}

impl AppState {
    fn lock_provider(&self, uuid: &str) -> Arc<parking_lot::Mutex<()>> {
        self.provider_locks.lock().entry(uuid.into())
            .or_insert_with(|| Arc::new(parking_lot::Mutex::new(()))).clone()
    }
}
```

- **All provider commands:** acquire `data_gate.read()` for the entire command, then `lock_provider(uuid)` for cross-store ops.
- **archive/reset/recovery:** acquire `data_gate.write()` — blocks ALL provider commands until done.

### 6.2 set_key (concrete — no undefined symbols)
```rust
fn set_key(state: &AppState, uuid: &str, key: &str) -> Result<(), AppError> {
    let _gate = state.data_gate.read();            // ── global read gate ──
    let lock = state.lock_provider(uuid);
    let _plock = lock.lock();                        // ── per-provider mutex ──
    let db = state.db.read().clone().ok_or(NotReady)?;
    // 1. DB snapshot:
    let (secret_ref, status) = db.with_conn(|c| {
        providers::get(c, uuid).map(|p| (p.secret_ref, p.status))
    })?;
    if status != "active" { return Err(NotCallable); }
    // 2. Keystore write (DB NOT locked):
    state.keystore.set_provider_key(&secret_ref, key)?;
    // 3. DB finalize — 'has_key' is NOT a DB column; we simply ensure enabled stays true.
    //    (Key existence is checked at runtime via keystore lookup, not stored in DB.)
    db.with_conn(|c| providers::touch_updated_at(c, uuid))?;
    Ok(())
}
```

**No `mark_has_key` column** — key existence is a runtime keystore lookup, not a DB field. `touch_updated_at` is optional bookkeeping.

### 6.3 delete (concrete)
```rust
fn delete(state: &AppState, uuid: &str) -> Result<(), AppError> {
    let _gate = state.data_gate.read();
    let lock = state.lock_provider(uuid);
    let _plock = lock.lock();
    let db = state.db.read().clone().ok_or(NotReady)?;
    // Step 1: DB tx — status=deleting, enabled=0, remove slots, invalidate consent.
    let secret_ref = db.with_conn(|c| providers::begin_delete(c, uuid))??;  // returns secret_ref
    // Step 2: keystore — remove key (idempotent).
    state.keystore.remove_provider_key(&secret_ref)?;
    // Step 3: DB tx — tombstone.
    db.with_conn(|c| providers::finalize_delete(c, uuid))?;
    Ok(())
}
```

`begin_delete` returns the `secret_ref` so step 2 has it without a second DB read.

### 6.4 Concurrency tests
- `set_key × delete` (same UUID): serialized by per-provider mutex → no orphan key.
- `set_key × archive_keystore` (any UUID): archive holds data_gate write → set_key blocked until archive done.
- DB command running + `archive_database`: archive waits for data_gate write → all read guards released first.

---

## 7. DataReadiness — guaranteed Windows handle close (P1 #3)

### 7.1 archive_database — Arc::try_unwrap + explicit close + spawn_blocking
```rust
#[tauri::command]
async fn archive_database(state: State<'_, AppState>) -> Result<(), String> {
    let state = state.inner().clone();
    spawn_blocking(move || {
        let _write_gate = state.data_gate.write();       // block all provider commands
        let mut db_slot = state.db.write();
        // Take the old Arc out of the slot:
        let old_db = db_slot.take();                       // slot is now None
        if let Some(arc) = old_db {
            // try_unwrap succeeds only if no other Arc exists (all commands released read guard).
            match Arc::try_unwrap(arc) {
                Ok(db) => { drop(db); }                    // Connection dropped → file handle released
                Err(arc_back) => {
                    // A command cloned the Arc and is still running — should not happen
                    // because data_gate.write() blocks all command entry. Restore + fail.
                    *db_slot = Some(arc_back);
                    return Err("DB still in use".into());
                }
            }
        }
        // Rename (file handle released on both macOS and Windows):
        let broken = db_path.with_extension(format!("db.broken-{}", now()));
        fs::rename(&db_path, &broken)?;
        // Open fresh + migrate:
        match Database::open(&db_path) {
            Ok(new_db) => {
                let new_db = Arc::new(new_db);
                // Re-run migration (failpoint=None for recovery):
                if let Err(e) = run_migration(&new_db, &keystore_dir, &settings_path, &FailpointCell::none()) {
                    *db_slot = Some(new_db);
                    *state.readiness.write() = DataReadiness::MigrationIncomplete { reason: e.to_string() };
                    return Err(format!("re-migration failed: {e}"));
                }
                *db_slot = Some(new_db);
                *state.readiness.write() = DataReadiness::Ready;
                Ok(())
            }
            Err(e) => {
                *state.readiness.write() = DataReadiness::NeedsDatabaseRecovery { reason: e.to_string() };
                Err(format!("DB reopen failed: {e}"))
            }
        }
    }).await.map_err(flatten)?
}
```

**All DB commands hold `data_gate.read()` + `db.read()`** for their entire duration. `archive_database` holds `data_gate.write()` → guarantees no command holds an Arc when `try_unwrap` runs. `spawn_blocking` ensures no async runtime blocking.

### 7.2 Startup keystore pre-check before Ready
After `migration_complete=1` early return, before setting `Ready`:
```rust
if db.with_conn(|c| schema::migration_complete(c))? {
    // Migration was previously complete — but verify keystore is still readable:
    match keystore::load_state(&keystore_dir) {
        KeystoreLoadState::Corrupt(e) => {
            *readiness = NeedsKeystoreRecovery { reason: e.to_string() };
        }
        _ => { *readiness = Ready; }
    }
    return Ok(());
}
```

### 7.3 Readiness → error mapping
| Condition | Readiness |
|-----------|-----------|
| `Database::open` fails (corrupt/NotADatabase/IO) | `NeedsDatabaseRecovery` |
| Keystore `Corrupt` (at migration or startup pre-check) | `NeedsKeystoreRecovery` |
| Migration `SettingsCorrupt` / non-keystore error | `MigrationIncomplete` |
| `provider_resume_deletions` fails | logged + `MigrationIncomplete` (does NOT exit setup) |
| All succeed | `Ready` |

### 7.4 Post keystore archive/reset — DB cleanup
```rust
// Executed under data_gate.write():
db.with_conn(|conn| {
    let tx = conn.transaction()?;
    tx.execute("UPDATE providers SET enabled=0 WHERE needs_key=1", [])?;  // executable SQL
    tx.execute("UPDATE preferences SET primary_uuid=NULL, parallel_uuids='[]', fallback_uuid=NULL, parallel_consent_scope=NULL, parallel_consent_version=NULL WHERE id=1", [])?;
    tx.execute("UPDATE _schema_migrations SET migration_complete=1 WHERE id=1", [])?;
    tx.commit()
})?;
*state.readiness.write() = DataReadiness::Ready;
```

---

## 8. Delete state machine (unchanged from rev-4, per-provider mutex coordinates steps)

---

## 9. Startup + command gating (unchanged from rev-4, data_gate added to all provider commands)

---

## 10. MigrationFailpoint — full value match, every checkpoint (P1 #2)

### 10.1 Failpoint — derive PartialEq/Eq, compare full values
```rust
#[derive(Clone, PartialEq, Eq)]
pub enum Failpoint {
    None,
    AfterBackup,
    AfterSchema,
    AfterPreferences,
    AfterProfileInsert(String),   // compares the FULL (String) value, not just discriminant
    AfterProfiles,
    AfterKeystoreRewrite,
    BeforeComplete,               // called AFTER the complete-commit (tests "committed but not yet returned")
}

pub struct FailpointCell(parking_lot::Mutex<Failpoint>);
impl FailpointCell {
    pub fn maybe_fail(&self, point: Failpoint) -> Result<(), MigrationError> {
        let fp = self.0.lock();
        if *fp == point { return Err(MigrationError::InjectedFail(point)); }
        Ok(())
    }
}
```

`#[derive(PartialEq, Eq)]` + `*fp == point` compares the full value including the `String` in `AfterProfileInsert`. `AfterProfileInsert("openai")` only matches when the coordinator has just inserted `"openai"`.

### 10.2 Test matrix — every checkpoint has injection + disk inspect + replay

| # | Failpoint | Coordinator persists before failing (inspected on disk) | After replay (fp=None) |
|---|-----------|----------------------------------------------------------|------------------------|
| M1 | None | — | complete=1, prefs(id=1,NULL), 0 profiles |
| M2 | None | — | profiles for openai+anthropic (from keys) + google (from settings fallback), keystore v2, selection seeded |
| M3 | None | — | no dupes, no-op |
| **M4** | `AfterBackup` | settings.bak + keystore.bak exist; NO tables in DB | re-run: schema → prefs → profiles → keystore v2 → complete |
| **M5** | `AfterSchema` | tables exist, singletons seeded; NO prefs from settings; NO profiles | re-run: prefs seeded → profiles → keystore v2 → complete |
| **M6** | `AfterPreferences` | tables + prefs (target_language from settings); NO profiles | re-run: profiles inserted → keystore v2 → complete |
| **M7** | `AfterProfileInsert("anthropic")` | anthropic profile row exists; google + openai NOT yet; keystore v1. (BTreeSet order: anthropic < google < openai) | re-run: INSERT OR IGNORE anthropic, inserts google + openai → Phase 4 → complete |
| **M8** | `AfterProfiles` | all profiles in DB; keystore still v1 | re-run: Phase 4 → keystore v2 → Phase 5 → complete |
| **M9** | `AfterKeystoreRewrite` | profiles in DB; keystore v2; complete=0 | re-run: Phase 5 verify → complete=1 |
| **M10** | `BeforeComplete` | complete=1 committed (but coordinator returns Err before Ok) | re-run: early return (complete=1), no-op |
| M11 | None (complete=1 rerun) | — | early return |
| M12 | None (.bak exists) | — | backup no-op, original preserved |
| M13 | None (settings corrupt) | — | SettingsCorrupt, complete=0 |
| M14 | None (keystore corrupt) | — | NeedsKeystoreRecovery, complete=0 |
| M15 | None (unknown legacy_id) | — | custom_http repair profile, needs_key=1, enabled=false |
| M16 | None (Ollama no key) | — | preset profile, needs_key=0, enabled=true, primary set |
| **M17** | None (DB-loss: v2 keystore, empty DB) | — | repair profiles for provider/<uuid> keys; deterministic UUIDv5 for unparseable; Phase 5 passes |

**BTreeSet candidate order (known to tests):** for M2 settings {default_provider="openai", fallback_engine="google"} + keystore {openai, anthropic}: candidates = {anthropic, google, openai} (sorted). M7 fails after "anthropic" (first in order).

### 10.3 Methodology (every M4–M10)
1. Set `FailpointCell` to the checkpoint.
2. Run real `run_migration` → executes + persists to checkpoint → returns `InjectedFail`.
3. **Open the DB file directly (separate Connection)** and inspect rows/tables — assert coordinator wrote correct intermediate state.
4. **Read the keystore file directly** — assert v1/v2 state.
5. Set fp=None, re-run `run_migration` → completes.
6. Assert final state correct + idempotent.

---

## 11. Delete fault injection (D1–D5, corrected)
| # | Failpoint / method | Disk inspect | After resume |
|---|---|---|---|
| D1 | Full delete | status=deleted, key absent | — |
| D2 | Fail after step 1 | status=deleting, key EXISTS | resume: key removed, tombstone |
| D3 | Fail after step 2 | status=deleting, key ABSENT | resume: tombstone (idempotent) |
| D4 | Fail after step 3 | status=deleted | resume: no-op |
| D5 | — | status=deleting | not in list, not in selection |

---

## 12. ProviderPatch + CRUD tests (unchanged + consent tests from §5.4)

---

## 13. Keystore v2 + db_schema tests (unchanged + synchronous=FULL + needs_key CHECK assertions)

---

## 14. Build sequence
1. Cargo deps (`rusqlite 0.40.1` bundled, `uuid`) + `uuid_util.rs` + `fs_acl.rs` → check.
2. `db/mod.rs` + `db/schema.rs` + `tests/db_schema.rs`.
3. `keystore.rs` v2 + `load_state` + `migrate_to_v2` + `backup_keystore` + `tests/keystore_v2.rs`.
4. `TraditionalProviderCatalog` + `CandidateSource` + `build_profile` (incl DB-loss repair) + `db/providers.rs` + `tests/provider_crud.rs`.
5. `Failpoint` + `FailpointCell` + `db/migration.rs` + `tests/migration.rs` (M1–M17 with real failpoints + disk inspection).
6. `AppState` (data_gate + provider_locks) + `ProviderCoordinator` + delete + `tests/delete_state_machine.rs` (D1–D5) + `tests/concurrency.rs` (set_key×delete, set_key×archive).
7. `consent.rs` (compute_scope HashSet + origin-only) + `tests/consent.rs`.
8. `db/readiness.rs` + `lib.rs` (setup, archive_database spawn_blocking, post-archive cleanup) + permissions.
9. Erratum already committed.
10. `cargo test` green.

---

## 15. Verification gate
S0 §11: migration (M1–M17 real failpoints), CRUD (§12), keystore v2 (§13). Root `src/` untouched.
