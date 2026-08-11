//! Database schema — all 8 tables (S0 §8.1) + singleton seeding + migration state.
//!
//! All DDL uses `CREATE TABLE IF NOT EXISTS` for idempotency (§8.5 Phase 2).

use rusqlite::{Connection, OptionalExtension};
use crate::db::DbError;

/// Current schema version. Bumped only on breaking schema changes.
///
/// v2 (R2-E): added `providers.version INTEGER NOT NULL DEFAULT 1` for
/// optimistic-lock (CAS) updates. See [`migrate_v1_to_v2`].
pub const SCHEMA_VERSION: u32 = 2;

/// Migration state for the preflight read-only check.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MigrationState {
    /// `_schema_migrations` table doesn't exist yet.
    NotStarted,
    /// Table exists but `migration_complete = 0`.
    Incomplete,
    /// `migration_complete = 1`.
    Complete,
    /// `schema_version` is HIGHER than the app's `SCHEMA_VERSION`: the DB was
    /// written by a NEWER app build. A downgrade must NOT run the migration
    /// (it doesn't understand the schema) NOR trust `migration_complete=1`. The
    /// migration core treats this as a hard stop: it returns `Err` so no backup
    /// is produced and no writes occur. The user must upgrade or archive.
    Incompatible,
}

/// Create all 8 tables + seed singletons. Must be called inside a transaction.
/// Idempotent: safe to call multiple times (CREATE TABLE IF NOT EXISTS).
pub fn create_all_tables(conn: &Connection) -> Result<(), DbError> {
    // ── _schema_migrations: singleton migration state ──
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS _schema_migrations (
            id INTEGER PRIMARY KEY CHECK (id = 1),
            schema_version INTEGER NOT NULL,
            migration_complete INTEGER NOT NULL DEFAULT 0 CHECK (migration_complete IN (0,1)),
            migration_checkpoint TEXT,
            migrated_at INTEGER
        );"
    )?;

    // ── preferences: singleton active selection + settings ──
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS preferences (
            id INTEGER PRIMARY KEY CHECK (id = 1),
            target_language TEXT NOT NULL DEFAULT 'zh',
            primary_uuid TEXT,
            parallel_uuids TEXT NOT NULL DEFAULT '[]',
            fallback_uuid TEXT,
            parallel_consent_version INTEGER,
            parallel_consent_scope TEXT,
            history_enabled INTEGER NOT NULL DEFAULT 0 CHECK (history_enabled IN (0,1)),
            history_retention_days INTEGER NOT NULL DEFAULT 30
        );"
    )?;

    // ── providers: ProviderProfile rows ──
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS providers (
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
            status TEXT NOT NULL DEFAULT 'active' CHECK (status IN ('active','deleting','deleted')),
            version INTEGER NOT NULL DEFAULT 1
        );
        CREATE INDEX IF NOT EXISTS idx_providers_status ON providers(status);"
    )?;

    // ── shortcuts ──
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS shortcuts (
            action TEXT PRIMARY KEY,
            keys TEXT NOT NULL
        );"
    )?;

    // ── history_sessions ──
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS history_sessions (
            session_uuid TEXT PRIMARY KEY,
            timestamp INTEGER NOT NULL,
            trigger_source TEXT NOT NULL,
            detected_language TEXT,
            target_language TEXT NOT NULL,
            is_favorite INTEGER NOT NULL DEFAULT 0 CHECK (is_favorite IN (0,1)),
            source_text_encrypted BLOB NOT NULL,
            source_text_nonce BLOB NOT NULL,
            crypto_version INTEGER NOT NULL
        );
        CREATE INDEX IF NOT EXISTS idx_history_sessions_ts ON history_sessions(timestamp DESC);"
    )?;

    // ── history_results ──
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS history_results (
            result_uuid TEXT PRIMARY KEY,
            session_uuid TEXT NOT NULL REFERENCES history_sessions(session_uuid) ON DELETE CASCADE,
            provider_uuid TEXT NOT NULL,
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
                 AND (error_message_encrypted IS NULL AND error_message_nonce IS NULL
                      OR error_message_encrypted IS NOT NULL AND error_message_nonce IS NOT NULL))
            )
        );"
    )?;

    // ── vocabulary ──
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS vocabulary (
            item_uuid TEXT PRIMARY KEY,
            timestamp INTEGER NOT NULL,
            source_language TEXT NOT NULL,
            target_language TEXT NOT NULL,
            word_encrypted BLOB NOT NULL,
            word_nonce BLOB NOT NULL,
            definition_encrypted BLOB NOT NULL,
            definition_nonce BLOB NOT NULL,
            crypto_version INTEGER NOT NULL
        );"
    )?;

    // ── dict_packages ──
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS dict_packages (
            package_id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            version TEXT NOT NULL,
            installed_at INTEGER NOT NULL
        );"
    )?;

    Ok(())
}

/// Seed singleton rows in `_schema_migrations` and `preferences`.
/// Uses INSERT OR IGNORE so it's idempotent.
pub fn seed_singletons(conn: &Connection) -> Result<(), DbError> {
    conn.execute(
        "INSERT OR IGNORE INTO _schema_migrations (id, schema_version, migration_complete)
         VALUES (1, ?, 0)",
        rusqlite::params![SCHEMA_VERSION as i64],
    )?;
    conn.execute("INSERT OR IGNORE INTO preferences (id) VALUES (1)", [])?;
    Ok(())
}

/// Read-only preflight: check migration state WITHOUT parsing settings or
/// creating backups. Uses `OptionalExtension` so only `QueryReturnedNoRows`
/// → None; corrupt-header / NotADatabase / IO errors propagate as Err.
pub fn migration_state_if_exists(conn: &Connection) -> Result<MigrationState, DbError> {
    let exists: Option<i64> = conn
        .query_row(
            "SELECT 1 FROM sqlite_master WHERE type='table' AND name='_schema_migrations'",
            [],
            |r| r.get(0),
        )
        .optional()?;
    if exists.is_none() {
        return Ok(MigrationState::NotStarted);
    }
    // Read schema_version + migration_complete in one row. Use `optional()` so
    // ONLY a missing row (QueryReturnedNoRows) maps to None; a corrupt-header /
    // NotADatabase / IO error propagates as Err (fail-closed) instead of being
    // swallowed into a silent Incomplete by a blanket `.unwrap_or`.
    //
    // schema_version is validated (S2a Task 5d): a version higher than
    // SCHEMA_VERSION means the DB was written by a NEWER app build. A downgrade
    // must NOT silently treat that DB as Complete, AND must NOT run the
    // migration against an unknown schema. We return `Incompatible` so the
    // migration core treats it as a hard stop (no backup, no writes). (A
    // lower/equal version is fine.)
    let row: Option<(i64, i64)> = conn
        .query_row(
            "SELECT schema_version, migration_complete FROM _schema_migrations WHERE id=1",
            [],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .optional()?;
    let (schema_version, complete) = match row {
        Some((sv, mc)) => (Some(sv), Some(mc)),
        // Row missing entirely → treat as an incomplete migration (the singleton
        // seed didn't run, or the row was deleted).
        None => (None, None),
    };
    if let Some(v) = schema_version {
        if v > SCHEMA_VERSION as i64 {
            // Future schema → hard stop. The migration core must NOT proceed
            // (it would back up + write against an unknown schema), so return
            // Incompatible rather than Incomplete.
            return Ok(MigrationState::Incompatible);
        }
        if v < SCHEMA_VERSION as i64 {
            // Older schema with migration_complete=1 → must NOT trust Complete.
            // A future SCHEMA_VERSION bump (e.g. 1→2) would cause v1 DBs to
            // silently skip the v2 migration. Force Incomplete so the migration
            // runs and upgrades the schema.
            return Ok(MigrationState::Incomplete);
        }
        // v == SCHEMA_VERSION: trust migration_complete below.
    }
    match complete {
        None => Ok(MigrationState::Incomplete),
        Some(0) => Ok(MigrationState::Incomplete),
        Some(1) => Ok(MigrationState::Complete),
        Some(other) => Err(DbError::Integrity(format!(
            "invalid migration_complete value: {other}"
        ))),
    }
}

/// Convenience: is migration complete? (wraps `migration_state_if_exists`)
pub fn migration_complete(conn: &Connection) -> Result<bool, DbError> {
    Ok(migration_state_if_exists(conn)? == MigrationState::Complete)
}

/// Mark migration as complete. Caller should be inside a transaction.
pub fn set_migration_complete(conn: &Connection) -> Result<(), DbError> {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;
    conn.execute(
        "UPDATE _schema_migrations SET migration_complete=1, migrated_at=? WHERE id=1",
        rusqlite::params![now],
    )?;
    Ok(())
}

/// Per-column metadata read from `PRAGMA table_info`. Used by
/// [`migrate_v1_to_v2`] to validate a pre-existing `version` column's shape.
#[derive(Debug)]
struct ColumnInfo {
    name: String,
    /// Declared type, upper-cased for case-insensitive compare (e.g. "INTEGER").
    col_type: String,
    notnull: bool,
    /// The declared DEFAULT, as SQLite reports it verbatim (e.g. Some("1")).
    dflt: Option<String>,
}

/// Read every column descriptor for a table via `PRAGMA table_info`.
///
/// `PRAGMA table_info` columns: (cid, name, type, notnull, dflt_value, pk).
/// `type` is the empty string for a column declared without a type; `dflt_value`
/// is NULL when no DEFAULT was declared.
fn table_columns(conn: &Connection, table: &str) -> Result<Vec<ColumnInfo>, DbError> {
    // `table` is an internal literal ("providers"), not user input, so building
    // the PRAGMA statement by formatting is safe (PRAGMA table_info does not
    // accept a bound parameter in the supported SQLite versions).
    let sql = format!("PRAGMA table_info({table})");
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map([], |r| {
        Ok(ColumnInfo {
            name: r.get::<_, String>(1)?,
            col_type: r.get::<_, String>(2).unwrap_or_default().to_ascii_uppercase(),
            notnull: r.get::<_, i64>(3)? != 0,
            dflt: r.get::<_, Option<String>>(4)?,
        })
    })?;
    let mut out = Vec::new();
    for row in rows {
        out.push(row?);
    }
    Ok(out)
}

/// Does a `version` column on `providers` have the exact shape the optimistic
/// lock requires: `INTEGER NOT NULL DEFAULT 1`?
fn version_column_is_correct(col: &ColumnInfo) -> bool {
    col.col_type == "INTEGER" && col.notnull && col.dflt.as_deref() == Some("1")
}

/// v1 → v2 migration (R2-E): add the `providers.version` optimistic-lock column.
///
/// Idempotent: probes `PRAGMA table_info(providers)` for a `version` column.
/// - **No `version` column** → `ALTER TABLE ADD COLUMN version INTEGER NOT NULL
///   DEFAULT 1`, then re-reads PRAGMA to verify the column landed correctly.
/// - **A `version` column with the correct shape** (INTEGER NOT NULL DEFAULT 1)
///   → no-op (crash-replay / already-v2 DB / mid-migration all converge here).
/// - **A `version` column with the WRONG shape** (TEXT, nullable, wrong default)
///   → **fail-closed**: return `Err(DbError::Integrity(...))`. A pre-existing
///   incompatible column must NOT be silently adopted — the optimistic lock
///   assumes INTEGER NOT NULL DEFAULT 1, and a TEXT/nullable column would either
///   mis-serialize the version or let a NULL slip in and break every CAS update.
///
/// After ensuring the column exists, bumps `_schema_migrations.schema_version`
/// to 2.
///
/// Caller MUST be inside a transaction (the ALTER + the version bump commit
/// atomically — a crash between them would leave schema_version stale and force
/// a harmless re-run on the next startup, but co-locating them in one tx is
/// cleaner and matches the create_all_tables + seed_singletons pattern).
pub fn migrate_v1_to_v2(conn: &Connection) -> Result<(), DbError> {
    let columns = table_columns(conn, "providers")?;
    let existing = columns.iter().find(|c| c.name == "version");
    match existing {
        Some(col) => {
            // A version column already exists. Fail-closed unless its shape
            // matches exactly — never silently adopt an incompatible column.
            if !version_column_is_correct(col) {
                return Err(DbError::Integrity(format!(
                    "providers.version has an incompatible shape: \
                     type={}, notnull={}, default={:?}; \
                     expected INTEGER NOT NULL DEFAULT 1",
                    col.col_type, col.notnull, col.dflt
                )));
            }
            // Correct shape — nothing to ALTER (idempotent re-run / already-v2).
        }
        None => {
            conn.execute(
                "ALTER TABLE providers ADD COLUMN version INTEGER NOT NULL DEFAULT 1",
                [],
            )?;
            // Re-read PRAGMA and verify the ALTER landed the expected shape.
            // SQLite honours the declared type/notnull/default for ADD COLUMN,
            // but we re-check fail-closed so a future schema change or a SQLite
            // quirk can't leave a subtly-wrong column undetected.
            let after = table_columns(conn, "providers")?;
            let landed = after.iter().find(|c| c.name == "version");
            match landed {
                Some(col) if version_column_is_correct(col) => { /* ok */ }
                other => return Err(DbError::Integrity(format!(
                    "ALTER TABLE providers ADD COLUMN version did not produce the \
                     expected INTEGER NOT NULL DEFAULT 1 column; got: {other:?}"
                ))),
            }
        }
    }
    // Bump the recorded schema version (idempotent: re-writing 2→2 is a no-op).
    conn.execute(
        "UPDATE _schema_migrations SET schema_version=?1 WHERE id=1",
        rusqlite::params![SCHEMA_VERSION as i64],
    )?;
    Ok(())
}
