# S2a — Data Model + Migration: Implementation Plan (rev-3)

**Branch:** `codex/s2a-data-model-migration` (from main `ef7b378`)
**Scope:** Backend-only — root `src/` (frontend) untouched.
**Status:** PLAN ONLY — not yet implemented. Awaiting freeze review.

This rev resolves all 4 P1s from the rev-2 review. Section index:

| Review P1 | Resolved in § |
|-----------|-------------|
| #1 Consent TOCTOU | §5 (single-transaction confirm) |
| #2 `db:None` incomplete recovery model | §7 (DataReadiness state model) |
| #3 Legacy candidate / preferences not closed | §3 + §4 (candidate enumeration, repair semantics, erratum) |
| #4 No auditable deliverable | This document (committed, full DDL + M1–M14/D1–D5) |

---

## 0. Dependencies + file layout

### Cargo.toml additions
```toml
rusqlite = { version = "0.37", features = ["bundled"] }
uuid = { version = "1", features = ["v4", "v5"] }
```

**rusqlite version rationale:** No MSRV pin exists in the project (no `rust-toolchain.toml`, no `rust-version` field; host is `rustc 1.95.0`). rusqlite `0.37` is the current stable release line as of 2026; its MSRV (~1.80) is well below the host compiler. `bundled` compiles SQLite into the binary so there is no Windows system-SQLite dependency. `Cargo.lock` pins the exact patch. If a newer `0.3x` is current at implementation time and builds clean, it may be used — the constraint is "current stable + `bundled`", not a specific number.

### New files
| File | Purpose |
|------|---------|
| `src-tauri/src/db/mod.rs` | `Database` wrapper (`&mut Connection` access) |
| `src-tauri/src/db/schema.rs` | DDL, `create_all_tables`, singleton seeding |
| `src-tauri/src/db/providers.rs` | `ProviderProfile` domain types, CRUD, invariants |
| `src-tauri/src/db/migration.rs` | §8.5 5-phase coordinator |
| `src-tauri/src/db/keystore_state.rs` | `KeystoreLoadState` typed enum |
| `src-tauri/src/db/readiness.rs` | `DataReadiness` state model + command gating |
| `src-tauri/src/uuid_util.rs` | UUIDv5 deterministic + UUIDv4 |
| `src-tauri/src/fs_acl.rs` | Shared file/dir permission helpers (extracted from keystore Win32 ACL) |
| `src-tauri/tests/db_schema.rs` | Schema tests |
| `src-tauri/tests/provider_crud.rs` | CRUD + invariant tests |
| `src-tauri/tests/migration.rs` | Migration fault-injection M1–M14 |
| `src-tauri/tests/keystore_v2.rs` | Keystore v2 + load-state tests |
| `src-tauri/tests/delete_state_machine.rs` | Delete checkpoint tests D1–D5 |

### Modified files
`keystore.rs` (KeystoreData v2 + load_state + migrate_to_v2), `lib.rs` (AppState + setup + command registration), `providers.rs` (preset lookup for migration), `build.rs` + `permissions/` (new command permissions).

---

## 1. Database API — `&mut Connection`, lock-order discipline

```rust
pub struct Database {
    conn: parking_lot::Mutex<rusqlite::Connection>,
}

impl Database {
    pub fn open(path: &Path) -> Result<Self, DbError> {
        let conn = rusqlite::Connection::open(path)?;
        conn.pragma_update(None, "foreign_keys", "ON")?;
        conn.pragma_update(None, "busy_timeout", 5000)?;
        conn.pragma_update(None, "journal_mode", "DELETE")?; // explicit, not "not WAL"
        fs_acl::secure_file(&path)?; // 0600 / Windows DACL
        Ok(Self { conn: parking_lot::Mutex::new(conn) })
    }

    /// SOLE access pattern. f receives &mut Connection so it can call
    /// conn.transaction() for atomic multi-statement writes.
    /// The Mutex is held ONLY for the duration of f.
    pub fn with_conn<T>(
        &self,
        f: impl FnOnce(&mut rusqlite::Connection) -> Result<T, DbError>,
    ) -> Result<T, DbError> {
        let mut conn = self.conn.lock();
        f(&mut *conn)
    }
}
```

**Lock-order rule (load-bearing, documented in a module-level comment):**

> The DB Mutex and the keystore's fs2 flock are **never held simultaneously**. Any operation touching both (delete, migration, set-key) follows:
> 1. Lock DB → read snapshot → **unlock DB**.
> 2. Perform keystore operation (under keystore's own fs2 lock + in-proc Mutex).
> 3. Lock DB → write result → unlock DB.
>
> There is no deadlock window because the two locks are never nested. Step 1's
> snapshot is a value copy (e.g. `ProviderProfile`), not a reference into the DB.

**DB file permissions:** `0600` file / `0700` dir on macOS; Windows user-owner DACL. Extracted from `keystore.rs` Win32 ACL helpers into shared `fs_acl` module so both DB and keystore use the same code.

---

## 2. Full DDL (all 8 tables — auditable)

Applied by `create_all_tables(conn)` in a single transaction. All `CREATE TABLE IF NOT EXISTS` (idempotent per §8.5 Phase 2).

```sql
-- ── _schema_migrations: singleton migration state ──────────────────────
CREATE TABLE IF NOT EXISTS _schema_migrations (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    schema_version INTEGER NOT NULL,
    migration_complete INTEGER NOT NULL DEFAULT 0,  -- 0 = incomplete, 1 = complete
    migration_checkpoint TEXT,                       -- human-readable last checkpoint
    migrated_at INTEGER
);
-- Seeded: INSERT OR IGNORE INTO _schema_migrations (id, schema_version, migration_complete)
--         VALUES (1, 1, 0);

-- ── preferences: singleton active selection + settings ─────────────────
CREATE TABLE IF NOT EXISTS preferences (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    target_language TEXT NOT NULL DEFAULT 'zh',
    primary_uuid TEXT,                               -- NULL = no primary selected
    parallel_uuids TEXT NOT NULL DEFAULT '[]',       -- JSON array of UUIDs
    fallback_uuid TEXT,                              -- NULL = no fallback
    parallel_consent_version INTEGER,                -- NULL = no consent
    parallel_consent_scope TEXT,                     -- canonical scope key (§5)
    history_enabled INTEGER NOT NULL DEFAULT 0,
    history_retention_days INTEGER NOT NULL DEFAULT 30
);
-- Seeded: INSERT OR IGNORE INTO preferences (id) VALUES (1);

-- ── providers: ProviderProfile rows ────────────────────────────────────
CREATE TABLE IF NOT EXISTS providers (
    uuid TEXT PRIMARY KEY,
    template_id TEXT NOT NULL,
    name TEXT NOT NULL,
    protocol TEXT NOT NULL,                          -- "openai_chat"|"anthropic"|"gemini"|"google_translate"|"custom_http"
    endpoint TEXT NOT NULL,
    model TEXT,
    enabled INTEGER NOT NULL DEFAULT 1,
    sort_order INTEGER NOT NULL DEFAULT 0,
    is_local INTEGER NOT NULL DEFAULT 0,
    secret_ref TEXT NOT NULL UNIQUE,                 -- keystore lookup key; UNIQUE enforced at DB level
    capabilities TEXT NOT NULL DEFAULT '{}',         -- JSON ProviderCapabilities
    status TEXT NOT NULL DEFAULT 'active'            -- "active"|"deleting"|"deleted"
);
CREATE INDEX IF NOT EXISTS idx_providers_status ON providers(status);

-- ── shortcuts: keyboard shortcuts (unused in S2a, created per §8.1) ────
CREATE TABLE IF NOT EXISTS shortcuts (
    action TEXT PRIMARY KEY,
    keys TEXT NOT NULL
);

-- ── history_sessions: encrypted sessions (S2b writes; S2a creates table)
CREATE TABLE IF NOT EXISTS history_sessions (
    session_uuid TEXT PRIMARY KEY,
    timestamp INTEGER NOT NULL,
    trigger_source TEXT NOT NULL,
    detected_language TEXT,
    target_language TEXT NOT NULL,
    is_favorite INTEGER NOT NULL DEFAULT 0,
    source_text_encrypted BLOB NOT NULL,             -- NOT NULL per §5.3
    source_text_nonce BLOB NOT NULL,                 -- 12 bytes
    crypto_version INTEGER NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_history_sessions_ts ON history_sessions(timestamp DESC);

-- ── history_results: per-provider results (S2b writes) ─────────────────
CREATE TABLE IF NOT EXISTS history_results (
    result_uuid TEXT PRIMARY KEY,
    session_uuid TEXT NOT NULL REFERENCES history_sessions(session_uuid) ON DELETE CASCADE,
    provider_uuid TEXT,                              -- nullable: may dangle if provider deleted
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
    -- Mutual exclusion: success → text present + no error; failure → error present + no text
    CHECK (
        (outcome_tag = 'success'
         AND result_text_encrypted IS NOT NULL
         AND result_text_nonce IS NOT NULL
         AND error_kind IS NULL
         AND error_message_encrypted IS NULL)
        OR
        (outcome_tag = 'failure'
         AND error_kind IS NOT NULL
         AND result_text_encrypted IS NULL
         AND error_message_nonce IS NULL)
    )
);

-- ── vocabulary: encrypted word/definition pairs (S4 scope) ─────────────
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

-- ── dict_packages: installed dictionary packages (S4 scope) ────────────
CREATE TABLE IF NOT EXISTS dict_packages (
    package_id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    version TEXT NOT NULL,
    installed_at INTEGER NOT NULL
);
```

**Schema assertions verified by `tests/db_schema.rs`:**
- All 8 tables exist after `create_all_tables`.
- `_schema_migrations(id=1)` singleton row exists (migration_complete=0).
- `preferences(id=1)` singleton row exists with NULL primary_uuid/fallback_uuid.
- `create_all_tables` called twice → no error, no duplicate tables.
- `PRAGMA foreign_keys` = ON, `PRAGMA journal_mode` = DELETE.
- `secret_ref UNIQUE`: inserting two providers with same secret_ref → constraint error.
- `history_results` CHECK: inserting `outcome_tag='success'` with NULL `result_text_encrypted` → error.
- `history_results` CHECK: inserting `outcome_tag='failure'` with non-NULL `result_text_encrypted` → error.

---

## 3. Legacy data mapping — complete resolution (P1 #3)

### 3.1 settings.json path resolution

`tauri-plugin-store` resolves via `BaseDirectory::AppData` (`app_data_dir()`), confirmed in the plugin source (`resolve_store_path` → `app.path().resolve(path, BaseDirectory::AppData)`). The keystore uses `app_local_data_dir()`. **On Windows these differ** (`%APPDATA%` vs `%LOCALAPPDATA%`).

Migration resolves the exact path via the plugin's own resolver:
```rust
let store_path = tauri_plugin_store::store::resolve_store_path(&app_handle, "settings.json")?;
```

### 3.2 settings.json parsing — no error swallowing

New `parse_settings_raw(path: &Path) -> Result<Option<RawSettings>, MigrationError>`:
```rust
struct RawSettings {
    default_provider: Option<String>,
    target_language: Option<String>,
    fallback_engine: Option<String>,
}
```
- File missing → `Ok(None)` (legitimate fresh install).
- File exists, unreadable (IO error) → `Err(MigrationError::SettingsCorrupt(io_err))`.
- File exists, invalid JSON → `Err(MigrationError::SettingsCorrupt(json_err))`.
- File exists, valid → `Ok(Some(RawSettings { ... }))`. Each field is `Option` — absent keys are `None`, not default-swallowed.

**Never calls `settings::load()`** (which returns defaults for missing keys and silently returns `Settings::default()` on store errors).

### 3.3 Migration candidate enumeration (complete)

```
candidate_legacy_ids = Set::new()

1. Enumerate keystore keys:
   match keystore_load_state(dir):
     Missing          → no candidates from keystore
     LegacyV1(map)    → candidate_legacy_ids.extend(map.keys())
     CurrentV2(data)  → candidate_legacy_ids.extend(data.provider_keys.keys())
                        (DB-loss recovery: v2 keystore may have keys whose DB rows vanished)
     Corrupt(_)       → abort migration (NeedsKeystoreRecovery)

2. Merge settings defaults:
   if let Some(settings) = parse_settings_raw(settings_path)?:
     if let Some(dp) = settings.default_provider:
       candidate_legacy_ids.insert(dp)
     if let Some(fb) = settings.fallback_engine:
       candidate_legacy_ids.insert(fb)
```

**For each `legacy_id`:**

| Condition | template_id | protocol | endpoint | model | is_local | enabled | Notes |
|-----------|-------------|----------|----------|-------|----------|---------|-------|
| Matches a preset (openai/anthropic/gemini/ollama) | preset.id | preset.api_kind → Protocol | preset.endpoint | preset.default_model | preset.is_local | true | Full preset data |
| Matches Google (traditional engine) | `"google"` | `"google_translate"` | Google engine endpoint (from `engines::registry`) | NULL | false | true | Maps to `google_translate` protocol; is a traditional engine eligible for fallback |
| Unknown legacy_id (not in presets/engines) | legacy_id itself | `"custom_http"` | `""` (empty) | NULL | false | **false** | "Needs repair" profile — key preserved, user must fill endpoint |

**Unknown-profile repair semantics:**
- `enabled = false` so it doesn't appear in the active selection or default list.
- `endpoint = ""` — an empty string, not a fabricated invalid URL. `provider_test_connection` / `provider_get_models` reject empty endpoints before any HTTP.
- The profile IS `status = 'active'` (not deleted) — it appears in `provider_list` so the user can see and repair it. A UI indicator (separate slice) shows "needs endpoint".
- The key in the keystore under `secret_ref = legacy_id` is preserved — not lost.

### 3.4 preferences seeding (write-guards)

After Phase 3 inserts all profiles, Phase 2 seeds preferences **only with validated UUIDs**:

```
primary_uuid:
  let dp = settings.default_provider
  if dp matches a migrated profile AND that profile is active AND enabled AND has valid protocol:
    primary_uuid = legacy_provider_uuid(dp)
  else:
    primary_uuid = NULL    // do not write an unvalidatable primary

fallback_uuid:
  let fb = settings.fallback_engine
  if fb matches a migrated profile AND it is a traditional engine (google/deepl/...):
    fallback_uuid = legacy_provider_uuid(fb)
  else:
    fallback_uuid = NULL
```

A profile that is `enabled=false` (unknown/repair) or missing a valid endpoint is **never** written into primary/fallback. This prevents an invalid selection.

### 3.5 DB-loss recovery (keystore is v2, DB is fresh/empty)

If `keystore_load_state` returns `CurrentV2(data)` but the DB has no providers (migration_complete=0, DB was lost/rebuilt):
- Enumerate `data.provider_keys` as candidate sources (step 1 above).
- For each key: if the key starts with `"provider/"` (new-style UUID key), the corresponding profile's `template_id`/`endpoint`/`name` are **not recoverable** from the keystore alone.
  - These are created as repair profiles: `template_id = "unknown"`, `protocol = "custom_http"`, `endpoint = ""`, `name = "Recovered (<secret_ref>)"`, `enabled = false`.
  - The `uuid` is extracted from `"provider/<uuid>"` if parseable; otherwise a new UUIDv4 is generated and the key is re-linked.
  - User must manually re-associate or re-create the profile.
- For legacy keys (no `"provider/"` prefix): same logic as §3.3 (preset lookup or unknown-repair).

### 3.6 S0 Erratum (to be appended to the spec file during implementation)

> **§8.5 Phase 5 Erratum:** The verification step ("assert every DB profile's `secret_ref` exists in `provider_keys`") applies **only to key-bearing profiles** — i.e., profiles whose `secret_ref` was derived from an actual key enumerated from the keystore. Profiles that legitimately have no key (Ollama `is_local`, Google fallback configured without a key, key-missing providers) are **not** required to have a key in `provider_keys`. The verification enumerates keys from the keystore and checks those keys have a matching DB `secret_ref`; it does not check the reverse for keyless profiles.

This erratum will be written into `docs/superpowers/specs/2026-08-01-linguaray-product-baseline.md` as an annotated note on §8.5 (not altering the frozen text, appended as an erratum block).

---

## 4. Keystore typed state + backup protocol (P1 #3 continuation)

### 4.1 Typed load state

```rust
pub enum KeystoreLoadState {
    /// File does not exist → fresh install.
    Missing,
    /// Legacy v1 flat map (no "version" field in the decrypted payload).
    LegacyV1(HashMap<String, String>),
    /// Current v2 structure (version == 2).
    CurrentV2(KeystoreData),
    /// File exists but is unreadable / fails authentication. Do NOT overwrite.
    Corrupt(KeystoreError),
}
```

`load_state(dir) -> KeystoreLoadState` replaces the current `load()` which returns `{}` for both missing and empty. This distinguishes fresh install from empty-but-present.

### 4.2 KeystoreData v2

```rust
#[derive(Serialize, Deserialize)]
pub struct KeystoreData {
    pub version: u32,                                  // 2
    pub provider_keys: HashMap<String, String>,        // keyed by secret_ref
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub history_key: Option<SerializableKey>,           // opt-in; not populated in S2a
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub external_api_token: Option<String>,              // opt-in; not populated in S2a
}
```

### 4.3 Backup protocol

- **Under lock:** the keystore backup is created under the keystore's fs2 flock + in-proc Mutex (same lock as any keystore RMW). The settings backup is created after confirming the resolved store path exists.
- **Create-new only:** `if !backup_path.exists() { copy canonical → backup }`. A replay after crash must NOT overwrite the original v1 backup with a now-v2 file. If `.bak-pre-migration` already exists, the backup step is a no-op.
- **Permissions:** backup files get `0600` / Windows DACL (same as canonical, via `fs_acl`).
- **Failure:** if either backup fails → `Err(MigrationError::BackupFailed { which })` → migration stops immediately, stays `migration_complete = 0`, canonical files untouched.
- **Corrupt keystore:** migration sees `KeystoreLoadState::Corrupt`. It does NOT auto-archive. Migration transitions `DataReadiness` to `NeedsKeystoreRecovery`. Only the user-initiated `archive_keystore` / `reset_keystore` commands move `.broken-*` files. After the user archives, the recovery path is defined in §7.

---

## 5. Consent — single-transaction confirm (P1 #1)

### 5.1 The TOCTOU problem (eliminated)

The previous design had `confirm_consent` write the scope, then `provider_set_active` write the selection in a separate transaction. Between the two, an endpoint could change, a provider could be disabled, or a recipient could be removed — making the confirmed scope stale.

### 5.2 Single-transaction confirm-and-commit

`provider_confirm_and_set_active` is a **single Tauri command** that performs the entire consent + selection commit in one DB transaction:

```rust
/// Called by the frontend AFTER the user confirms the consent dialog.
/// The frontend passes the candidate selection + the scope_key it showed
/// to the user (for audit), but the backend does NOT trust the frontend's
/// scope — it recomputes from the live DB state.
#[tauri::command]
async fn provider_confirm_and_set_active(
    primary: String,
    parallel: Vec<String>,
    fallback: Option<String>,
    expected_scope: String,            // what the user saw and approved
    state: State<'_, AppState>,
) -> Result<(), String> {
    let db = require_ready(&state)?;
    let db = db.clone();
    spawn_blocking(move || db.with_conn(|tx| {
        let tx = tx.transaction()?;
        // 1. Re-read ALL providers from DB (inside the tx — no TOCTOU window).
        let providers = providers::list_all(&tx)?;
        // 2. Validate the candidate selection against live providers.
        providers::validate_active_selection(&primary, &parallel, &fallback, &providers)?;
        // 3. Backend recomputes the canonical scope (authoritative).
        let actual_scope = consent::compute_scope(&primary, &parallel, &providers)?;
        // 4. Assert the frontend's expected_scope matches the backend's actual_scope.
        //    If they differ, the user approved a stale scope — reject.
        if expected_scope != actual_scope {
            return Err(DbError::ConsentScopeMismatch {
                expected: expected_scope, actual: actual_scope,
            });
        }
        // 5. Write selection + consent in the SAME transaction.
        tx.execute("UPDATE preferences SET primary_uuid=?, parallel_uuids=?, fallback_uuid=? WHERE id=1",
            rusqlite::params![primary, serde_json::to_string(&parallel)?, fallback])?;
        let next_version = current_consent_version(&tx)? + 1;
        tx.execute("UPDATE preferences SET parallel_consent_scope=?, parallel_consent_version=? WHERE id=1",
            rusqlite::params![actual_scope, next_version])?;
        tx.commit()?;
        Ok(())
    })).await.map_err(flatten)?
}
```

### 5.3 Consent invalidation (synchronized in each respective transaction)

Any operation that changes the recipient set **invalidates consent in the same transaction**:

| Operation | Consent effect (same tx) |
|-----------|--------------------------|
| `provider_update` (endpoint changed) | If the provider is in primary/parallel: set `parallel_consent_scope = NULL`, `parallel_consent_version += 1`. Frontend must re-confirm. |
| `provider_toggle` (disabled) | If disabled provider is in active slots: remove from slots + invalidate consent. |
| `provider_delete` (step 1) | Remove from active slots + invalidate consent. |
| `provider_confirm_and_set_active` | Sets new scope + version (§5.2). |

`provider_set_active` (without consent, for single-engine or non-parallel changes):
- If `parallel` is empty → no consent needed, write selection directly (consent scope = NULL is valid for single-engine).
- If `parallel` is non-empty AND the stored scope doesn't match the new recipient set → return `Err(ConsentRequired { actual_scope })`. Frontend shows consent dialog, then calls `provider_confirm_and_set_active`.

### 5.4 Canonical scope computation (backend-authoritative)

```rust
/// Computes the canonical consent scope key from the LIVE provider list.
/// Same algorithm as the UI Lab: primary + parallel recipients,
/// endpoint-origin-normalized, UUID-sorted. Fallback is EXCLUDED.
fn compute_scope(primary: &str, parallel: &[String], providers: &[ProviderProfile]) -> Result<String> {
    let by_uuid: HashMap<&str, &ProviderProfile> = providers.iter().map(|p| (p.uuid.as_str(), p)).collect();
    let mut recipients: Vec<(String, String)> = vec![primary]
        .into_iter().chain(parallel.iter().cloned())
        .filter_map(|uuid| {
            let p = by_uuid.get(uuid.as_str())?;
            Some((uuid, normalize_origin(&p.endpoint)))
        })
        .collect();
    recipients.sort_by(|a, b| a.0.cmp(&b.0));
    let parts: Vec<String> = recipients.iter().map(|(uuid, origin)| format!("{}|{}", uuid, origin)).collect();
    Ok(format!("v1:{{{}}}", parts.join(",")))
}
```

---

## 6. ProviderPatch + invariants

### 6.1 ProviderPatch — deny_unknown_fields

```rust
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]   // unknown JSON fields → deserialization error
pub struct ProviderPatch {
    pub name: Option<String>,
    pub endpoint: Option<String>,    // validated via providers::validate_endpoint if Some
    pub model: Option<String>,
    pub enabled: Option<bool>,
    pub sort_order: Option<i32>,
}
```

### 6.2 Backend invariants (all in-transaction)

```rust
fn validate_active_selection(
    primary: &str, parallel: &[str], fallback: Option<&str>,
    providers: &[ProviderProfile],
) -> Result<(), DbError>
```
- **Disjoint:** no UUID appears in more than one of {primary, parallel, fallback}.
- **Callable:** all referenced UUIDs have `status = 'active'` AND `enabled = true`.
- **Fallback type:** if `fallback` is Some, its `template_id` must be in `TRADITIONAL_TEMPLATES`.
- **Reorder:** `provider_reorder(uuids)` — the UUID list must be exactly the set of active provider UUIDs (no duplicates, no missing, no extra).
- **secret_ref UNIQUE:** enforced by DDL constraint (§2). Attempting to create two profiles with the same `secret_ref` → DB error.
- **Slot cleanup:** disable/update(delete-step-1)/delete all clean active slots (remove the UUID from primary/parallel/fallback) in the same transaction.

---

## 7. DataReadiness state model + command gating (P1 #2)

### 7.1 Typed readiness state

```rust
pub enum DataReadiness {
    /// DB open, migration complete, keystore readable. All commands available.
    Ready,

    /// DB is fine, but keystore is corrupt/unreadable. User must archive/reset.
    NeedsKeystoreRecovery { reason: String },

    /// Keystore is fine, but DB failed to open or is corrupt. User must archive/rebuild DB.
    NeedsDatabaseRecovery { reason: String },

    /// DB opened but migration did not reach migration_complete=1.
    /// May be a crash-replay pending, or a verification failure.
    MigrationIncomplete { checkpoint: Option<String>, reason: String },
}
```

`AppState` holds:
```rust
pub struct AppState {
    pub db: Option<Arc<Database>>,
    pub keystore: Keystore,
    pub client: reqwest::Client,
    pub readiness: parking_lot::RwLock<DataReadiness>,
}
```

### 7.2 Startup readiness determination

```
setup(app):
  1. Determine keystore state: load_state(dir)
     Corrupt → readiness = NeedsKeystoreRecovery; db may still open for health/archive.
  2. Open DB:
     Err(e) → readiness = NeedsDatabaseRecovery { reason: e }; db = None.
     Ok(db) →
       run_migration(&db, ...):
         Ok(()) → provider_resume_deletions(&db, &keystore)?; readiness = Ready.
         Err(MigrationError::SettingsCorrupt | KeystoreCorrupt) →
           readiness = MigrationIncomplete { checkpoint, reason }.
  3. app.manage(AppState { db, keystore, client, readiness });
```

**No `.expect()` crash.** App always launches. The frontend reads readiness via a `get_data_readiness` command and shows the appropriate recovery UI.

### 7.3 Command gating matrix

| Command class | Ready | NeedsKeystoreRecovery | NeedsDatabaseRecovery | MigrationIncomplete |
|---------------|-------|-----------------------|-----------------------|---------------------|
| provider_* (list/create/update/delete/reorder/toggle/set_key/set_active/confirm) | ✅ | ❌ `NotReady` | ❌ `NotReady` | ❌ `NotReady` |
| provider_test_connection / provider_get_models | ✅ | ❌ | ❌ | ❌ |
| keystore_health | ✅ | ✅ | ✅ | ✅ |
| archive_keystore / reset_keystore | ✅ | ✅ | ✅ | ✅ |
| **archive_database** (NEW) | ✅ | ✅ | ✅ | ✅ |
| get_data_readiness | ✅ | ✅ | ✅ | ✅ |

### 7.4 `archive_database` — DB recovery path

New command:
```rust
#[tauri::command]
async fn archive_database(state: State<'_, AppState>) -> Result<(), String> {
    // Move linguaray.db → linguaray.db.broken-<ts> (recoverable, not deleted).
    // Re-open fresh DB → create_all_tables → re-run migration.
    // If keystore is v2: enumerate provider_keys → create repair profiles (§3.5).
    // Set readiness = Ready (or MigrationIncomplete if re-migration also fails).
}
```

### 7.5 Post-archive keystore recovery path

After user calls `archive_keystore` (corrupt keystore → `.broken-<ts>`):
1. Keystore file is gone → `load_state` returns `Missing`.
2. Migration re-runs: Phase 5 verification now sees no keys → it checks only that key-bearing profiles (none, since keystore is empty) are verified → passes trivially.
3. Existing DB profiles lose their keys (provider_keys is empty). They remain in the DB with `enabled = false` (key-missing). User re-enters keys via `provider_set_key`.
4. `migration_complete = 1` → `readiness = Ready`.
5. **Phase 5 does NOT permanently fail** because the erratum (§3.6) scopes verification to keys actually enumerated from the keystore — an empty keystore has no keys to verify.

---

## 8. Provider delete state machine — exact checkpoint semantics

```
Step 1 (DB tx):
  UPDATE providers SET status='deleting', enabled=0 WHERE uuid=?;
  remove uuid from primary/parallel/fallback in preferences (same tx);
  invalidate consent if uuid was in primary/parallel (same tx);
  COMMIT.
  → Persistent state: status='deleting', key STILL EXISTS in keystore.

Step 2 (keystore, under keystore lock — DB NOT locked):
  remove_provider_key(secret_ref);
  atomic write.
  → Persistent state: status='deleting', key DELETED from keystore.

Step 3 (DB tx):
  UPDATE providers SET status='deleted', name='deleted: '||name WHERE uuid=?;
  COMMIT.
  → Persistent state: tombstone (status='deleted').
```

**Startup sweep (`provider_resume_deletions`):** for every `status = 'deleting'` row:
- Attempt step 2 (remove key — idempotent: if already absent, `remove_provider_key` returns Ok without error).
- Then step 3 (convert to tombstone).
- Forward-only: `deleting` never reverts to `active`.

---

## 9. Tauri startup + command execution model

### 9.1 Startup (correct Builder model)

`invoke_handler!` is a static Builder macro — all commands are always registered. The gating is runtime via `DataReadiness` (§7).

```rust
.setup(|app| {
    let dir = app.path().app_local_data_dir()?;
    let keystore = keystore::Keystore::new(&dir)?;  // does not crash on corrupt — returns Ok, load_state handles it
    let store_path = resolve_store_path(&app.handle(), "settings.json")?;
    let db_path = dir.join("linguaray.db");

    let (db, readiness) = match Database::open(&db_path) {
        Err(e) => (None, DataReadiness::NeedsDatabaseRecovery { reason: e.to_string() }),
        Ok(db) => {
            let db = Arc::new(db);
            match run_migration(&db, &dir, &store_path) {
                Ok(()) => {
                    provider_resume_deletions(&db, &keystore)?;
                    (Some(db), DataReadiness::Ready)
                }
                Err(e) => (Some(db), DataReadiness::MigrationIncomplete {
                    checkpoint: e.checkpoint(), reason: e.to_string(),
                }),
            }
        }
    };
    app.manage(AppState { db, keystore, client, readiness: RwLock::new(readiness) });
    Ok(())
})
```

### 9.2 Command execution patterns

**Pure DB command (list/create/update/reorder/toggle/set_active/confirm):**
```rust
async fn provider_list(state: State<'_, AppState>) -> Result<Vec<ProviderProfile>, String> {
    let db = require_ready(&state)?;     // checks readiness == Ready
    let db = db.clone();
    spawn_blocking(move || db.with_conn(|conn| providers::list(conn)))
        .await.map_err(flatten)?.map_err(flatten)
}
```

**Hybrid command (test_connection / get_models — DB snapshot + async HTTP):**
```rust
async fn provider_test_connection(uuid: String, state: State<'_, AppState>) -> Result<ConnectionResult, String> {
    let db = require_ready(&state)?.clone();
    let client = state.client.clone();
    // 1. Read profile snapshot in spawn_blocking (DB lock released after).
    let profile = spawn_blocking(move || db.with_conn(|c| providers::get(c, &uuid)))
        .await??;
    // 2. HTTP test via async reqwest (NOT under DB lock).
    wire::test_connection(&client, &profile).await
}
```

**Always-available command (keystore_health / archive / reset / archive_database / get_data_readiness):**
No `require_ready` check. These work in any readiness state for recovery.

---

## 10. Migration coordinator — 5 phases with checkpoints

```rust
pub fn run_migration(db: &Database, keystore_dir: &Path, settings_path: &Path) -> Result<(), MigrationError> {
    db.with_conn(|conn| {
        // ── Phase 2: DB schema (idempotent) ──
        let tx = conn.transaction()?;
        schema::create_all_tables(&tx)?;
        schema::seed_singletons(&tx)?;
        tx.commit()?;
        // checkpoint: "schema-applied"

        // If migration already complete → early return (idempotent no-op).
        if schema::migration_complete(conn)? { return Ok(()); }

        // ── Phase 1: Backup (after schema, before data) ──
        // Backups created under respective locks. Create-new only.
        backup_settings(settings_path)?;                   // settings.json.bak-pre-migration
        // Keystore backup under keystore lock:
        //   (handled inside keystore module — see Phase 1 detail below)
        // checkpoint: "backups-created"

        // ── Phase 2b: Seed preferences from settings (if preferences is default) ──
        let settings = parse_settings_raw(settings_path)?;
        // (preferences row already exists from seed_singletons; update target_language if settings present)
        // checkpoint: "preferences-seeded"

        // ── Phase 3: Profile migration (crash-safe, idempotent) ──
        let ks_state = keystore::load_state(keystore_dir);
        match &ks_state {
            KeystoreLoadState::Corrupt(e) => return Err(MigrationError::KeystoreCorrupt(e.clone())),
            KeystoreLoadState::Missing => { /* no keys to migrate */ }
            KeystoreLoadState::LegacyV1(map) | KeystoreLoadState::CurrentV2(KeystoreData { provider_keys: map, .. }) => {
                let candidates = enumerate_candidates(map, settings.as_ref());
                for legacy_id in &candidates {
                    let profile = build_profile_from_legacy(legacy_id, map, &presets)?;
                    let tx = conn.transaction()?;
                    // INSERT OR IGNORE — idempotent on re-run after crash.
                    providers::insert_or_ignore(&tx, &profile)?;
                    tx.commit()?;
                    // checkpoint: "profile-<legacy_id>-inserted"
                }
                // Seed primary/fallback UUIDs with write-guards (§3.4).
                seed_preferences_selection(conn, settings.as_ref(), &candidates)?;
            }
        }
        // checkpoint: "profiles-committed"

        // ── Phase 4: Keystore atomic rewrite (if LegacyV1) ──
        if let KeystoreLoadState::LegacyV1(map) = &ks_state {
            // Under keystore lock (DB NOT locked):
            keystore::migrate_to_v2(keystore_dir, map.clone())?;
            // checkpoint: "keystore-v2"
        }

        // ── Phase 5: Verify + complete ──
        let ks_state_after = keystore::load_state(keystore_dir);
        verify_key_bearing_profiles(conn, &ks_state_after)?;  // erratum-scoped (§3.6)
        let tx = conn.transaction()?;
        tx.execute("UPDATE _schema_migrations SET migration_complete=1, migrated_at=? WHERE id=1",
            params![now()])?;
        tx.commit()?;
        // checkpoint: "complete"
        Ok(())
    })
}
```

**Phase 1 backup detail:** The settings backup (`settings.json` → `settings.json.bak-pre-migration`) is a simple file copy with create-new semantics. The keystore backup (`keystore.json` → `keystore.json.bak-pre-migration`) is performed **inside `keystore::migrate_to_v2`** which holds the fs2 flock — so the backup + rewrite happen under one keystore lock acquisition. If the backup file already exists (re-run after crash), it is NOT overwritten.

---

## 11. Fault-injection test matrix — M1–M14

**Methodology:** Each test runs the REAL `run_migration` coordinator against a real temp SQLite file + real temp keystore. Intermediate crash states are simulated by **pre-seeding the DB/keystore to the exact state that would exist at a checkpoint**, then calling `run_migration` and asserting the resulting persistent state. No mocking of the coordinator itself.

| # | Scenario | Failpoint simulation (how the intermediate state is created) | Expected persistent state after re-run |
|---|----------|--------------------------------------------------------------|----------------------------------------|
| **M1** | Fresh install | Empty temp dir — no settings.json, no keystore, no DB | `_schema_migrations(id=1, complete=1)`; `preferences(id=1, primary_uuid=NULL, fallback_uuid=NULL)`; 0 providers; no keystore file created |
| **M2** | Upgrade (happy path) | Pre-create: `settings.json` {default_provider:"openai", target_language:"zh", fallback_engine:"google"} + legacy keystore flat map {"openai":"sk-a","anthropic":"sk-b"} | 2 providers: openai (UUIDv5 deterministic, secret_ref="openai", enabled), anthropic (UUIDv5, secret_ref="anthropic"). Google NOT a separate profile unless it has a key. preferences: primary_uuid=UUIDv5("openai"), target_language="zh". Keystore is v2 (version=2, provider_keys has both keys). `complete=1`. |
| **M3** | Idempotent re-run | Run M2, then call `run_migration` again | No duplicate providers (INSERT OR IGNORE). Keystore stays v2 (detects version=2, skips Phase 4). `complete=1` unchanged. |
| **M4** | Crash: backups created, schema not applied | Manually create `.bak-pre-migration` files + empty DB file (no tables) | Re-run: `create_all_tables` creates tables (IF NOT EXISTS), proceeds through all phases. |
| **M5** | Crash: schema committed, preferences not seeded from settings | Create tables + singletons (complete=0), place `settings.json` with values, preferences row has default target_language | Re-run: Phase 2b updates target_language from settings, seeds primary/fallback if valid. |
| **M6** | Crash: 1 of 2 profiles inserted (mid-Phase-3) | Insert only the openai profile row (INSERT), leave anthropic un-inserted. Keystore still v1. | Re-run: INSERT OR IGNORE skips openai (already exists), inserts anthropic. Both present. Proceeds to Phase 4. |
| **M7** | Crash: all profiles in DB, keystore still v1 | Insert both profile rows (complete=0). Keystore is v1 flat map (no version field). | Re-run: Phase 4 detects LegacyV1 → migrate_to_v2. Keystore becomes v2, provider_keys has both keys under same names. Phase 5 verifies. `complete=1`. |
| **M8** | Crash: keystore v2, complete flag not written | Both profiles in DB. Keystore is v2. `_schema_migrations.complete=0`. | Re-run: Phase 4 skips (already v2). Phase 5 verifies → sets `complete=1`. |
| **M9** | complete=1, run again | `complete=1` | Early return after Phase 2 (no-op). No writes. |
| **M10** | Backup idempotency | `.bak-pre-migration` files already exist (from a prior run) | Backup step is no-op (create-new only). Original v1 backup content preserved (assert bytes unchanged). |
| **M11** | settings.json corrupt JSON | Write malformed JSON to settings.json | `run_migration` returns `Err(SettingsCorrupt)`. `complete` stays 0. Canonical settings.json untouched. DB tables exist (Phase 2 ran) but no data migrated. |
| **M12** | Keystore corrupt | Write an unreadable keystore file (wrong identity / tampered ciphertext) | `load_state` returns `Corrupt`. `run_migration` returns `Err(KeystoreCorrupt)`. `complete=0`. No `.broken-*` created. DB tables exist. |
| **M13** | Unknown legacy_id in flat map | Legacy keystore flat map has {"custom-engine":"sk-x"} (not in presets) | Profile created: template_id="custom-engine", protocol="custom_http", endpoint="", enabled=false. Key preserved in keystore under secret_ref="custom-engine". Phase 5 verifies this key exists. |
| **M14** | Ollama in settings, no key | settings default_provider="ollama". Keystore flat map empty or lacks "ollama". | Profile created from preset (is_local=true, protocol="openai_chat", endpoint="http://localhost:11434/...", enabled=true, needs_key=false). Phase 5 does NOT require a key (erratum: ollama is keyless). primary_uuid = UUIDv5("ollama"). |

### Additional migration assertions (all tests):
- `.bak-pre-migration` files exist after any migration from legacy data (M2, M4–M8).
- `.bak-pre-migration` files have `0600` / DACL permissions.
- UUIDv5 determinism: `legacy_provider_uuid("openai")` is the same across M2, M3, M6, M7 (same input → same UUID).
- No mixed keystore state: after Phase 4, the keystore payload has `version: 2` (not a partial mix).

---

## 12. Delete state machine tests — D1–D5

**Methodology:** Real DB + real keystore on temp paths. Crash states are created by running step 1 only (or step 1+2) and committing, then calling `provider_resume_deletions` and asserting the final state. No mocking.

| # | Scenario | Failpoint simulation | Expected persistent state |
|---|----------|---------------------|---------------------------|
| **D1** | Full delete (no crash) | Create provider with key. Call full `provider_delete` (steps 1→2→3). | `status='deleted'`, `name='deleted: <orig>'`. Keystore: `provider_keys[secret_ref]` absent. Provider not in `provider_list`. |
| **D2** | Crash after step 1 | Run step 1 only (status='deleting', enabled=false, removed from slots). Key still in keystore. Do NOT run step 2 or 3. | Call `resume_deletions`: step 2 removes key (now absent), step 3 tombstones. Final: `status='deleted'`, key absent. |
| **D3** | Crash after step 2 | Run steps 1+2 (status='deleting', key already removed). Do NOT run step 3. | Call `resume_deletions`: step 2 idempotent (key already absent → no error), step 3 tombstones. Final: `status='deleted'`, key absent. |
| **D4** | Crash after step 3 (tombstone) | Run all 3 steps. Then call `resume_deletions` again. | No-op: `status='deleted'` unchanged. No error. Key still absent (idempotent). |
| **D5** | Deleting provider is excluded | Provider with `status='deleting'`. | Not returned by `provider_list`. Not in `primary_uuid`/`parallel_uuids`/`fallback_uuid`. |

### Delete checkpoint key-state summary (corrected):
| Checkpoint | status | key in keystore |
|------------|--------|-----------------|
| After step 1 | `deleting` | **EXISTS** (not yet removed) |
| After step 2 | `deleting` | **ABSENT** (removed) |
| After step 3 | `deleted` | **ABSENT** |

---

## 13. Provider CRUD tests

| Test | Actions | Assertions |
|------|---------|------------|
| Create | `provider_create("openai", "My OpenAI", endpoint, "gpt-4o")` | Returns profile with UUIDv4, secret_ref="provider/<uuid>", status='active'. Appears in `provider_list`. |
| Update (valid) | `provider_update(uuid, {name: "Renamed"})` | name changes; other fields unchanged. |
| Update (deny_unknown_fields) | `provider_update(uuid, json!({"evil_field": "x"}))` | Serde error — command returns error, no write. |
| Update (invalid endpoint) | `provider_update(uuid, {endpoint: "http://evil.com"})` | Rejected (HTTP non-loopback). No write. |
| Duplicate | `provider_duplicate(uuid)` | New UUIDv4, different secret_ref, enabled=true, original key NOT copied (new profile has no key until set). |
| Reorder (valid) | `provider_reorder([uuid1, uuid2, uuid3])` | sort_order reflects new order. |
| Reorder (incomplete) | `provider_reorder([uuid1, uuid2])` (missing uuid3) | Rejected — not a complete permutation. |
| Reorder (duplicate) | `provider_reorder([uuid1, uuid1, uuid3])` | Rejected. |
| Toggle (disable) | `provider_toggle(uuid, false)` on a primary provider | enabled=false; removed from primary_uuid (cleanup in same tx); consent invalidated. |
| Set active (single) | `provider_set_active(primary, [], None)` | Written directly (no parallel → no consent needed). |
| Set active (parallel, no consent) | `provider_set_active(primary, [uuid2], None)` when scope mismatches | Returns `Err(ConsentRequired { actual_scope })`. |
| Confirm + set active | `provider_confirm_and_set_active(primary, [uuid2], None, expected_scope)` | Selection + consent scope + version written in one tx. |
| Confirm (stale scope) | Same but `expected_scope` differs from backend recomputation | `Err(ConsentScopeMismatch)`. No write. |
| Secret_ref collision | Create two profiles, force same secret_ref | DB UNIQUE constraint error. |

---

## 14. Keystore v2 tests

| Test | Actions | Assertions |
|------|---------|------------|
| v2 round-trip | Encrypt KeystoreData{version:2, provider_keys:{"k1":"v1"}}, decrypt | Payload matches. version=2. |
| LegacyV1 detection | Decrypt a payload that is a flat map (no "version" field) | `load_state` returns `LegacyV1(map)`. |
| Missing detection | No keystore file | `load_state` returns `Missing` (not `LegacyV1({})`). |
| Corrupt detection | Tampered ciphertext / wrong identity | `load_state` returns `Corrupt(error)`. |
| migrate_to_v2 | `LegacyV1({"openai":"sk-x"})` → `migrate_to_v2` | version=2, provider_keys={"openai":"sk-x"}, history_key=None. Original keys preserved. |
| get/set/remove key | On a v2 keystore: set, get, remove | Operations work on provider_keys by secret_ref. |
| Backup create-new | `.bak-pre-migration` exists → migrate_to_v2 | Original backup NOT overwritten (assert bytes unchanged). |

---

## 15. db_schema tests

| Test | Assertions |
|------|------------|
| All tables created | 8 tables exist after `create_all_tables`. |
| Singletons exist | `_schema_migrations(id=1)` and `preferences(id=1)` rows exist. |
| Fresh preferences | `preferences(id=1)` has `primary_uuid=NULL`, `fallback_uuid=NULL`, `parallel_uuids='[]'`. |
| Idempotent create | `create_all_tables` twice → no error. |
| foreign_keys ON | `PRAGMA foreign_keys` = 1. |
| journal_mode DELETE | `PRAGMA journal_mode` = "delete". |
| secret_ref UNIQUE | Insert two providers with same secret_ref → error. |
| outcome CHECK | success with NULL text → error; failure with non-NULL text → error. |

---

## 16. Build sequence

1. `Cargo.toml` deps (rusqlite + uuid) + `uuid_util.rs` + `fs_acl.rs` → `cargo check`.
2. `db/mod.rs` + `db/schema.rs` (DDL + create_all_tables + seed_singletons) + `tests/db_schema.rs`.
3. `keystore.rs` KeystoreData v2 + `KeystoreLoadState` + `load_state` + `migrate_to_v2` + `tests/keystore_v2.rs`.
4. `db/providers.rs` (domain types, CRUD, `validate_active_selection`, `ProviderPatch deny_unknown_fields`) + `tests/provider_crud.rs`.
5. `db/migration.rs` (5-phase coordinator, `parse_settings_raw`, backup, `enumerate_candidates`, `build_profile_from_legacy`) + `tests/migration.rs` (M1–M14).
6. Delete state machine (`provider_delete` + `provider_resume_deletions`) + `tests/delete_state_machine.rs` (D1–D5).
7. `db/readiness.rs` (`DataReadiness` + `require_ready`) + consent (`compute_scope` + `provider_confirm_and_set_active`).
8. `lib.rs` wiring (AppState, setup graceful-fail, command gating, `archive_database`) + `build.rs` + `permissions/`.
9. Append S0 erratum to spec file (§3.6).
10. `cargo test` green on macOS + Windows CI.

---

## 17. Verification gate (S0 §11)

> **S2a:** Migration tested (fresh + upgrade + crash-replay); ProviderProfile CRUD unit-tested; keystore versioned structure tested.

| Sub-gate | Covered by |
|----------|------------|
| Migration (fresh + upgrade + crash-replay) | M1 (fresh), M2 (upgrade), M4–M9 (crash-replay at every checkpoint) |
| ProviderProfile CRUD | §13 (create/update/duplicate/reorder/toggle/delete + invariants) |
| Keystore versioned structure | §14 (v2 round-trip, load states, migrate_to_v2) |

Root `src/` (frontend) untouched. S2a is backend-only.
