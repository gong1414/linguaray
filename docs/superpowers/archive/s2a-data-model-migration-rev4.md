Archived-on: 2026-08-14 · reason: superseded by linguaray-plugin-core-design / completed, see git history

# S2a — Data Model + Migration: Implementation Plan (rev-4)

**Branch:** `codex/s2a-data-model-migration`
**Scope:** Backend-only — root `src/` untouched.
**Status:** PLAN ONLY. Depends on `s0-erratum-phase5-verification.md` (committed alongside).

Resolves all 8 P1s from rev-3 review. P1 → section index:

| P1 | § |
|----|---|
| #1 Migration nests DB + keystore locks | §1.3 (phased, no outer with_conn) |
| #2 M1–M14 not real fault injection | §10 (MigrationFailpoint table) |
| #3 Candidate/Google/M2/M14 contradictions | §3 (unified contract) |
| #4 DataReadiness recovery unrealizable | §7 (replaceable DB slot) |
| #5 set-key/delete race (orphan keys) | §6 (provider mutation coordinator) |
| #6 Consent algorithm imprecise | §5 (strict scope, origin-only invalidation) |
| #7 history_results CHECK wrong | §2 (fixed DDL) |
| #8 DB open compile/safety gaps | §1.2 (dir-first, synchronous=FULL, resolve_store_path root, rusqlite 0.40.1) |

---

## 1. Database API + open safety

### 1.1 Signature
```rust
pub struct Database { conn: parking_lot::Mutex<rusqlite::Connection> }

impl Database {
    pub fn with_conn<T>(&self, f: impl FnOnce(&mut Connection) -> Result<T, DbError>) -> Result<T, DbError> {
        let mut conn = self.conn.lock();
        f(&mut *conn)
    }
}
```

### 1.2 Open — corrected order + pragmas
```rust
pub fn open(path: &Path) -> Result<Self, DbError> {
    let dir = path.parent().unwrap_or(Path::new("."));
    fs::create_dir_all(dir)?;               // 1. Create + protect PARENT DIR first
    fs_acl::secure_dir(dir)?;               // 2. 0700 / DACL on dir
    let conn = rusqlite::Connection::open(path)?;  // 3. THEN open (file created in protected dir)
    fs_acl::secure_file(path)?;             // 4. 0600 / DACL on file
    conn.pragma_update(None, "foreign_keys", "ON")?;
    conn.pragma_update(None, "busy_timeout", 5000)?;
    conn.pragma_update(None, "journal_mode", "DELETE")?;
    conn.pragma_update(None, "synchronous", "FULL")?;  // crash-safe commits
    Ok(Self { conn: parking_lot::Mutex::new(conn) })
}
```

- `synchronous=FULL` added (consistent with crash-safe claims).
- `journal_mode=DELETE` explicit (no WAL, no -wal/-shm sidecar files to miss in backup).

**Store path:** `tauri_plugin_store::resolve_store_path(&app_handle, "settings.json")?` — the crate-root re-export (confirmed: `lib.rs:21` re-exports `resolve_store_path`; the `store` module itself is not separately public).

**rusqlite version:** `0.40.1` + `bundled`. (Confirmed current stable via docs.rs. Rev-3 incorrectly stated 0.37.)

### 1.3 Migration — NO outer DB lock (P1 #1)

The entire `run_migration` is **NOT** wrapped in `db.with_conn`. Each phase acquires its own short DB transaction, commits, and releases the DB lock before any keystore operation:

```rust
pub fn run_migration(db: &Database, keystore_dir: &Path, settings_path: &Path, failpoint: &FailpointCell) -> Result<(), MigrationError> {
    // ── Phase 2: DB schema (short tx) ──
    db.with_conn(|conn| {
        let tx = conn.transaction()?;
        schema::create_all_tables(&tx)?;
        schema::seed_singletons(&tx)?;
        tx.commit()
    })?;
    failpoint.maybe_fail(Failpoint::AfterSchema)?;

    // Idempotent: if complete, return early.
    if db.with_conn(|c| schema::migration_complete(c))? { return Ok(()); }

    // ── Phase 1: Backup (DB NOT locked) ──
    let settings = parse_settings_raw(settings_path)?;  // read settings (no DB lock)
    backup_settings(settings_path)?;                     // copy settings.json → .bak (create-new)
    // Keystore backup happens inside migrate_to_v2 (under keystore lock, not DB lock).
    failpoint.maybe_fail(Failpoint::AfterBackup)?;

    // ── Phase 2b: Seed preferences (short tx) ──
    db.with_conn(|conn| seed_preferences(conn, settings.as_ref()))?;
    failpoint.maybe_fail(Failpoint::AfterPreferences)?;

    // ── Phase 3: Enumerate candidates + insert profiles ──
    // Read keystore state (DB NOT locked):
    let ks_state = keystore::load_state(keystore_dir);
    match &ks_state {
        KeystoreLoadState::Corrupt(e) => return Err(MigrationError::NeedsKeystoreRecovery(e.to_string())),
        KeystoreLoadState::Missing => { /* still enumerate settings-only candidates */ }
        _ => {}
    }
    let candidates = enumerate_candidates(&ks_state, settings.as_ref());
    for legacy_id in &candidates {
        let profile = build_profile_from_legacy(legacy_id, &ks_state, &presets, &traditional_catalog)?;
        db.with_conn(|conn| {
            let tx = conn.transaction()?;
            providers::insert_or_ignore(&tx, &profile)?;
            tx.commit()
        })?;
        failpoint.maybe_fail(Failpoint::AfterProfileInsert(legacy_id.clone()))?;
    }
    // Seed selection with write-guards (short tx):
    db.with_conn(|conn| seed_selection(conn, settings.as_ref(), &candidates))?;
    failpoint.maybe_fail(Failpoint::AfterProfiles)?;

    // ── Phase 4: Keystore rewrite (DB NOT locked) ──
    if let KeystoreLoadState::LegacyV1(map) = &ks_state {
        keystore::migrate_to_v2(keystore_dir, map.clone())?;  // under keystore lock only
    }
    failpoint.maybe_fail(Failpoint::AfterKeystoreRewrite)?;

    // ── Phase 5: Verify + complete (short tx) ──
    let ks_after = keystore::load_state(keystore_dir);  // read (DB NOT locked)
    db.with_conn(|conn| {
        verify_key_bearing_profiles(conn, &ks_after)?;  // erratum-scoped
        let tx = conn.transaction()?;
        tx.execute("UPDATE _schema_migrations SET migration_complete=1, migrated_at=? WHERE id=1",
            params![now()])?;
        tx.commit()
    })?;
    Ok(())
}
```

**No lock nesting:** Every `db.with_conn` block contains only DB work and commits before returning. Keystore operations (`load_state`, `migrate_to_v2`) happen between DB blocks, with no DB lock held.

---

## 2. Full DDL — corrected CHECK + NOT NULL (P1 #7)

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
    protocol TEXT NOT NULL,
    endpoint TEXT NOT NULL,
    model TEXT,
    enabled INTEGER NOT NULL DEFAULT 1,
    sort_order INTEGER NOT NULL DEFAULT 0,
    is_local INTEGER NOT NULL DEFAULT 0,
    secret_ref TEXT NOT NULL UNIQUE,
    capabilities TEXT NOT NULL DEFAULT '{}',
    status TEXT NOT NULL DEFAULT 'active'
);
CREATE INDEX IF NOT EXISTS idx_providers_status ON providers(status);

CREATE TABLE IF NOT EXISTS shortcuts (
    action TEXT PRIMARY KEY,
    keys TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS history_sessions (
    session_uuid TEXT PRIMARY KEY,
    timestamp INTEGER NOT NULL,
    trigger_source TEXT NOT NULL,
    detected_language TEXT,
    target_language TEXT NOT NULL,
    is_favorite INTEGER NOT NULL DEFAULT 0,
    source_text_encrypted BLOB NOT NULL,
    source_text_nonce BLOB NOT NULL,
    crypto_version INTEGER NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_history_sessions_ts ON history_sessions(timestamp DESC);

CREATE TABLE IF NOT EXISTS history_results (
    result_uuid TEXT PRIMARY KEY,
    session_uuid TEXT NOT NULL REFERENCES history_sessions(session_uuid) ON DELETE CASCADE,
    provider_uuid TEXT NOT NULL,               -- NOT NULL (S0 §5.3: String; tombstone = soft delete)
    provider_name_snapshot TEXT NOT NULL,
    engine_id TEXT NOT NULL,
    elapsed_ms INTEGER NOT NULL,
    outcome_tag TEXT NOT NULL CHECK (outcome_tag IN ('success', 'failure')),
    result_text_encrypted BLOB,
    result_text_nonce BLOB,
    error_kind TEXT,
    error_message_encrypted BLOB,
    error_message_nonce BLOB,
    crypto_version INTEGER NOT NULL,
    CHECK (
        (outcome_tag = 'success'
         AND result_text_encrypted IS NOT NULL
         AND result_text_nonce IS NOT NULL
         AND error_kind IS NULL
         AND error_message_encrypted IS NULL
         AND error_message_nonce IS NULL)
        OR
        (outcome_tag = 'failure'
         AND error_kind IS NOT NULL
         AND result_text_encrypted IS NULL
         AND result_text_nonce IS NULL
         AND (error_message_encrypted IS NULL AND error_message_nonce IS NULL    -- plaintext error only
              OR error_message_encrypted IS NOT NULL AND error_message_nonce IS NOT NULL))  -- encrypted detail
    )
);

CREATE TABLE IF NOT EXISTS vocabulary (
    item_uuid TEXT PRIMARY KEY,
    timestamp INTEGER NOT NULL,
    source_language TEXT NOT NULL,
    target_language TEXT NOT NULL,
    word_encrypted BLOB NOT NULL,
    word_nonce BLOB NOT NULL,
    definition_encrypted BLOB NOT NULL,
    definition_nonce BLOB NOT NULL,
    crypto_version INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS dict_packages (
    package_id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    version TEXT NOT NULL,
    installed_at INTEGER NOT NULL
);
```

**Corrections from rev-3:**
- `provider_uuid TEXT NOT NULL` (was nullable — S0 §5.3 defines it as `String`; tombstone is soft-delete, not NULL).
- Success branch: ALL error fields must be NULL (`error_message_nonce` included).
- Failure branch: result fields both NULL; error message ciphertext+nonce must be **both NULL or both NOT NULL** (not independently nullable).

---

## 3. Candidate enumeration + Google mapping — unified contract (P1 #3)

### 3.1 Decision: traditional engines ARE providers

Traditional engines (Google, DeepL, etc.) **do** enter the `providers` table as profiles. This is the only way `fallback_uuid` can reference a UUID. The `TraditionalProviderCatalog` (see erratum) provides the endpoint + template_id.

### 3.2 Candidate enumeration (settings always considered, even if keystore Missing)

```rust
fn enumerate_candidates(ks: &KeystoreLoadState, settings: Option<&RawSettings>) -> Vec<String> {
    let mut ids: BTreeSet<String> = BTreeSet::new();
    // 1. Keys from keystore (works for both LegacyV1 and CurrentV2):
    match ks {
        KeystoreLoadState::LegacyV1(map) | KeystoreLoadState::CurrentV2(KeystoreData { provider_keys: map, .. }) => {
            ids.extend(map.keys().cloned());
        }
        KeystoreLoadState::Missing | KeystoreLoadState::Corrupt(_) => {}
    }
    // 2. Settings defaults (ALWAYS, even if keystore Missing):
    if let Some(s) = settings {
        if let Some(dp) = &s.default_provider { ids.insert(dp.clone()); }
        if let Some(fb) = &s.fallback_engine { ids.insert(fb.clone()); }
    }
    ids.into_iter().collect()
}
```

### 3.3 Profile building per candidate

| Candidate matches | template_id | protocol | endpoint | model | is_local | needs_key | enabled |
|---|---|---|---|---|---|---|---|
| AI preset (openai/anthropic/gemini) | preset.id | preset → Protocol | preset.endpoint | preset.default_model | false | true | true |
| Local preset (ollama) | "ollama" | OpenAIChat | "http://localhost:11434/..." | "qwen2.5:7b" | true | false | true |
| Traditional catalog (google) | "google" | google_translate | catalog.endpoint | NULL | false | false | true |
| Unknown legacy_id | legacy_id | custom_http | "" | NULL | false | true | **false** (repair) |

### 3.4 M2 + M14 corrected contracts

**M2 (upgrade happy path):** settings has `fallback_engine="google"`. Google IS a candidate (from settings). A Google profile IS created (from traditional catalog). `fallback_uuid = legacy_provider_uuid("google")`. Keystore has keys for openai + anthropic; Google is keyless (valid per erratum).

**M14 (Ollama, no key):** settings has `default_provider="ollama"`. Keystore Missing or has no "ollama" key. Ollama IS a candidate (from settings, even with Missing keystore). Profile created from preset (is_local=true, needs_key=false). `primary_uuid = legacy_provider_uuid("ollama")`. Phase 5 passes (Ollama is keyless, valid per erratum).

---

## 4. Keystore typed state + backup

(Unchanged from rev-3 §4, restated for completeness.)

```rust
pub enum KeystoreLoadState {
    Missing,
    LegacyV1(HashMap<String, String>),
    CurrentV2(KeystoreData),
    Corrupt(KeystoreError),
}
```

Backup: create-new only (never overwrite), under keystore lock, 0600/DACL. Corrupt → no auto-archive.

---

## 5. Consent — strict scope, origin-only invalidation (P1 #6)

### 5.1 Strict scope computation (no silent drops)
```rust
fn compute_scope(primary: &str, parallel: &[String], providers: &[ProviderProfile]) -> Result<String, ConsentError> {
    let by_uuid: HashMap<&str, &ProviderProfile> = providers.iter().map(|p| (p.uuid.as_str(), p)).collect();
    let mut recipients: Vec<(String, String)> = once(primary.to_owned())
        .chain(parallel.iter().cloned())
        .map(|uuid| {
            let p = by_uuid.get(uuid.as_str())
                .ok_or(ConsentError::UnknownRecipient(uuid.clone()))?;
            if !p.enabled || p.status != "active" {
                return Err(ConsentError::NotCallable(uuid));
            }
            Ok((uuid, normalize_origin(&p.endpoint)?))
        })
        .collect::<Result<Vec<_>, _>>()?;
    if recipients.len() != 1 + parallel.len() {
        return Err(ConsentError::DuplicateRecipient);
    }
    recipients.sort_by(|a, b| a.0.cmp(&b.0));
    let parts: Vec<String> = recipients.iter().map(|(u, o)| format!("{}|{}", u, o)).collect();
    Ok(format!("v1:{{{}}}", parts.join(",")))
}
```

- `once(primary.to_owned()).chain(parallel.iter().cloned())` — no type ambiguity.
- Every UUID is resolved; missing/disabled → error, not silently dropped.
- Duplicate check: if `recipients.len() != 1 + parallel.len()`, a UUID appeared twice → error.

### 5.2 Origin-only consent invalidation

`normalize_origin` extracts scheme + host + port (drops path/query). Consent is invalidated **only when the origin changes** — a path/query update on the endpoint does not invalidate a previously-approved scope. This is implemented by comparing `normalize_origin(old_endpoint)` vs `normalize_origin(new_endpoint)` in `provider_update`:

```rust
// In provider_update, if endpoint changed:
let old_origin = normalize_origin(&old_endpoint)?;
let new_origin = normalize_origin(&new_endpoint)?;
if old_origin != new_origin && uuid_in_active_slots {
    invalidate_consent(tx);  // scope + version cleared
}
// If only path/query changed (same origin) → consent preserved.
```

### 5.3 Single-transaction confirm (unchanged from rev-3 §5.2)

`provider_confirm_and_set_active` re-reads providers, recomputes scope server-side, checks `expected_scope`, writes selection + consent in one tx.

---

## 6. Provider mutation coordinator — no orphan keys (P1 #5)

### 6.1 The race

Without a higher-level coordinator:
- `set_key` reads provider → active. Concurrently, `delete` marks deleting + removes key. `set_key` writes key back → orphan key.

### 6.2 Solution: per-provider mutation mutex

```rust
pub struct ProviderCoordinator {
    /// One mutex per provider UUID. Held for the ENTIRE DB-snapshot → keystore-op → DB-finalize flow.
    locks: parking_lot::Mutex<HashMap<String, Arc<parking_lot::Mutex<()>>>>,
}

impl ProviderCoordinator {
    fn lock_for(&self, uuid: &str) -> Arc<parking_lot::Mutex<()>> {
        let mut map = self.locks.lock();
        map.entry(uuid.to_owned()).or_insert_with(|| Arc::new(putex::mutex())).clone()
    }
}
```

**Every cross-store operation acquires the per-provider mutex FIRST, then does DB snapshot → keystore op → DB finalize:**

```rust
// set_key:
fn set_key(db, keystore, coord, uuid, key) -> Result<()> {
    let lock = coord.lock_for(&uuid);
    let _guard = lock.lock();  // ── per-provider mutation lock held ──
    // 1. DB snapshot (short lock):
    let profile = db.with_conn(|c| providers::get(c, &uuid))?;
    if profile.status != "active" { return Err(NotCallable); }
    // 2. Keystore write (keystore lock only, DB NOT locked):
    keystore.set_provider_key(&profile.secret_ref, &key)?;
    // 3. DB finalize (short lock):
    db.with_conn(|c| providers::mark_has_key(c, &uuid))?;
    Ok(())
}

// delete:
fn delete(db, keystore, coord, uuid) -> Result<()> {
    let lock = coord.lock_for(&uuid);
    let _guard = lock.lock();  // ── per-provider mutation lock held ──
    // Step 1: DB tx — status=deleting, remove slots, invalidate consent.
    db.with_conn(|c| providers::begin_delete(c, &uuid))?;
    // Step 2: keystore — remove key.
    keystore.remove_provider_key(&secret_ref)?;
    // Step 3: DB tx — tombstone.
    db.with_conn(|c| providers::finalize_delete(c, &uuid))?;
    Ok(())
}
```

Because `set_key` and `delete` for the same UUID acquire the same per-provider mutex, they are fully serialized. No orphan keys. The DB and keystore locks are still never nested (the per-provider mutex is a third, higher-level lock that coordinates the sequence).

**Participants:** `set_key`, `delete`, `resume_deletions` (per deleting-UUID), `archive_keystore`/`reset_keystore` (acquires all — or drains via a global read-write lock). Stored in `AppState`.

---

## 7. DataReadiness — replaceable DB slot (P1 #4)

### 7.1 Mutable DB slot
```rust
pub struct AppState {
    pub db: parking_lot::RwLock<Option<Arc<Database>>>,
    pub keystore: Keystore,
    pub client: reqwest::Client,
    pub readiness: parking_lot::RwLock<DataReadiness>,
    pub coord: ProviderCoordinator,
}
```

### 7.2 Readiness → error mapping (corrected)
| Condition | Readiness |
|-----------|-----------|
| DB open fails (corrupt/NotADatabase) | `NeedsDatabaseRecovery` |
| Keystore `Corrupt` | `NeedsKeystoreRecovery` |
| Migration returns `SettingsCorrupt` or other non-fatal | `MigrationIncomplete` |
| Migration returns `KeystoreCorrupt` | `NeedsKeystoreRecovery` |
| All succeed | `Ready` |

`provider_resume_deletions` failure → logged + `MigrationIncomplete` (does NOT exit setup).

### 7.3 archive_database protocol (serial close/rename/reopen)
```rust
async fn archive_database(state: State<'_, AppState>) -> Result<(), String> {
    // 1. Take the DB write lock — blocks all provider commands (they hold read lock).
    let mut db_slot = state.db.write();
    // 2. Drop the old Arc<Database> (closes the rusqlite Connection — releases file handle).
    *db_slot = None;
    // 3. Rename linguaray.db → linguaray.db.broken-<ts>.
    fs::rename(&db_path, &broken_path)?;
    // 4. Open fresh DB + create_all_tables.
    let new_db = Arc::new(Database::open(&db_path)?);
    new_db.with_conn(|c| schema::create_all_tables(c))?;
    // 5. Re-run migration (from v2 keystore if present → repair profiles).
    run_migration(&new_db, &keystore_dir, &settings_path, &FailpointCell::none())?;
    // 6. Replace slot.
    *db_slot = Some(new_db);
    drop(db_slot);
    // 7. Update readiness.
    *state.readiness.write() = DataReadiness::Ready;
    Ok(())
}
```

On Windows, the Connection must be fully dropped (step 2) before rename — the RwLock write ensures no reader holds an Arc. `drop` on the last `Arc<Database>` closes the `rusqlite::Connection` which releases the file handle.

### 7.4 Post keystore archive/reset — DB cleanup transaction
```rust
// After archive_keystore or reset_keystore completes (keystore now empty/missing):
db.with_conn(|conn| {
    let tx = conn.transaction()?;
    // Disable all providers that needs_key=true (they lost their keys).
    tx.execute("UPDATE providers SET enabled=0 WHERE ... needs_key derivation ...", [])?;
    // Clear active selection (all slots now potentially invalid).
    tx.execute("UPDATE preferences SET primary_uuid=NULL, parallel_uuids='[]', fallback_uuid=NULL, parallel_consent_scope=NULL, parallel_consent_version=NULL WHERE id=1", [])?;
    // Re-run migration Phase 5 (keystore is empty → verification passes trivially).
    tx.execute("UPDATE _schema_migrations SET migration_complete=1 WHERE id=1", [])?;
    tx.commit()
})?;
*state.readiness.write() = DataReadiness::Ready;
```

### 7.5 Command gating matrix (unchanged from rev-3)

provider_* commands require `Ready`. health/archive/reset/archive_database/get_data_readiness always available.

---

## 8. Delete state machine

(Unchanged from rev-3 §8. Steps 1→2→3, crash recovery via `resume_deletions`, forward-only. Per-provider mutex coordinates steps.)

---

## 9. Startup + command execution

(Unchanged from rev-3 §9. Static `invoke_handler!`, runtime gating via `DataReadiness`. Hybrid commands read snapshot in spawn_blocking, HTTP in async. `provider_resume_deletions` failure does not exit setup.)

---

## 10. MigrationFailpoint — real fault injection (P1 #2)

### 10.1 Failpoint mechanism
```rust
#[derive(Clone)]
pub enum MigrationFailpoint {
    None,
    AfterSchema,
    AfterBackup,
    AfterPreferences,
    AfterProfileInsert(String),  // after inserting this legacy_id
    AfterProfiles,
    AfterKeystoreRewrite,
    BeforeComplete,
}

pub struct FailpointCell(parking_lot::Mutex<MigrationFailpoint>);

impl FailpointCell {
    pub fn maybe_fail(&self, point: MigrationFailpoint) -> Result<(), MigrationError> {
        let fp = self.0.lock().clone();
        if std::mem::discriminant(&fp) == std::mem::discriminant(&point) {
            return Err(MigrationError::InjectedFail(point));
        }
        Ok(())
    }
}
```

In production, `FailpointCell::none()` is passed — no failures injected. In tests, each M-test sets a specific failpoint.

### 10.2 Real fault-injection test methodology

Each test:
1. Sets `FailpointCell` to a specific checkpoint.
2. Runs the REAL `run_migration` → it executes all phases up to the checkpoint, persists state, then returns `Err(InjectedFail)`.
3. **Inspects the on-disk DB + keystore** to verify the coordinator persisted the correct intermediate state (not pre-seeded by the test).
4. Resets failpoint to `None`, re-runs `run_migration` → should complete successfully.
5. Asserts final state is correct + idempotent.

### 10.3 Corrected test matrix

| # | Failpoint | What coordinator persists before failing | After re-run (failpoint=None) |
|---|-----------|------------------------------------------|-------------------------------|
| M1 | None (fresh) | — | complete=1, preferences(id=1,NULL slots), 0 profiles |
| M2 | None (upgrade) | — | 2 AI profiles + 1 Google profile, keystore v2, selection seeded, complete=1 |
| M3 | None (re-run M2) | — | No dupes, no-op |
| M4 | `AfterBackup` | Tables created, singletons seeded, backups created, NO profiles | Re-run inserts profiles → completes |
| M5 | `AfterPreferences` | Tables + preferences seeded (target_language from settings), NO profiles | Re-run inserts profiles → completes |
| M6 | `AfterProfileInsert("openai")` | Only openai profile row; anthropic NOT inserted; keystore v1 | Re-run: INSERT OR IGNORE openai, inserts anthropic → Phase 4 → completes |
| M7 | `AfterProfiles` | Both profiles in DB; keystore still v1 | Re-run: Phase 4 converts keystore to v2 → Phase 5 → complete |
| M8 | `BeforeComplete` | Profiles in DB; keystore v2; complete=0 | Re-run: Phase 5 verifies → complete=1 |
| M9 | None (complete=1) | — | Early return, no writes |
| M10 | None (.bak exists) | — | Backup no-op (original preserved) |
| M11 | None (settings corrupt) | — | SettingsCorrupt error, complete=0, canonical untouched |
| M12 | None (keystore corrupt) | — | NeedsKeystoreRecovery, complete=0, no auto-archive |
| M13 | None (unknown legacy_id) | — | custom_http profile, endpoint="", enabled=false, key preserved |
| M14 | None (Ollama no key) | — | Profile from preset, is_local, primary_uuid set, Phase 5 passes (keyless valid) |

**Each M4–M8 test inspects real persisted state (DB rows + keystore file) at the failpoint**, proving the coordinator wrote the correct intermediate state — not a test-pre-seeded state.

### 10.4 Delete fault-injection (D1–D5, corrected)

| # | Method | Assert |
|---|--------|--------|
| D1 | Full `provider_delete` (no failpoint) | status=deleted, key absent, not in list |
| D2 | Failpoint after step 1 (DB commit, before keystore remove): inspect disk → status=deleting, key EXISTS. Then resume. | resume: key removed, tombstone |
| D3 | Failpoint after step 2 (keystore remove, before tombstone): inspect → status=deleting, key ABSENT. Then resume. | resume: tombstone (no error on already-absent key) |
| D4 | Complete delete, then resume again | no-op |
| D5 | status=deleting provider | not in list, not in selection |

---

## 11. ProviderPatch, invariants, CRUD tests

(Unchanged from rev-3 §6, §13. `deny_unknown_fields`, `validate_active_selection`, secret_ref UNIQUE, etc.)

---

## 12. Keystore v2 tests

(Unchanged from rev-3 §14.)

---

## 13. db_schema tests

Added: `synchronous=FULL` assertion. (Rest unchanged from rev-3 §15.)

---

## 14. Build sequence

1. Cargo deps (`rusqlite = "0.40.1"` bundled, `uuid`) + `uuid_util.rs` + `fs_acl.rs` → check.
2. `db/mod.rs` + `db/schema.rs` + `tests/db_schema.rs`.
3. `keystore.rs` v2 + `load_state` + `migrate_to_v2` + `tests/keystore_v2.rs`.
4. `TraditionalProviderCatalog` + `db/providers.rs` + `tests/provider_crud.rs`.
5. `MigrationFailpoint` + `db/migration.rs` + `tests/migration.rs` (M1–M14 with real failpoints).
6. `ProviderCoordinator` + delete state machine + `tests/delete_state_machine.rs` (D1–D5).
7. `db/readiness.rs` + consent + `tests/consent.rs`.
8. `lib.rs` wiring + `archive_database` + permissions.
9. S0 erratum already committed.
10. `cargo test` green.

---

## 15. Verification gate

S0 §11 sub-gates covered by: M1–M14 (migration with real failpoints), §11 (CRUD), §12 (keystore v2).
