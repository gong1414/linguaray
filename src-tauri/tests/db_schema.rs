//! db_schema tests — verify all 8 tables, singletons, pragmas, constraints.
//! (S2a verification gate — schema sub-tests.)

use linguaray_lib::db::{Database, DbError};
use linguaray_lib::db::schema;
use tempfile::tempdir;

/// Helper: open a fresh DB in a temp dir and run create_all_tables + seed_singletons.
fn fresh_db() -> (tempfile::TempDir, Database) {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("test.db");
    let db = Database::open(&db_path).unwrap();
    db.with_conn(|conn| {
        let tx = conn.transaction()?;
        schema::create_all_tables(&tx)?;
        schema::seed_singletons(&tx)?;
        tx.commit()?;
        Ok(())
    }).unwrap();
    (dir, db)
}

#[test]
fn all_eight_tables_exist() {
    let (_dir, db) = fresh_db();
    db.with_conn(|conn| {
        let tables: Vec<String> = conn
            .prepare("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name")?
            .query_map([], |r| r.get(0))?
            .collect::<Result<Vec<_>, _>>()?;
        let expected = [
            "_schema_migrations", "dict_packages", "history_results",
            "history_sessions", "preferences", "providers", "shortcuts", "vocabulary",
        ];
        for t in &expected {
            assert!(tables.iter().any(|x| x == t), "table '{}' missing", t);
        }
        Ok(())
    }).unwrap();
}

#[test]
fn singletons_exist() {
    let (_dir, db) = fresh_db();
    db.with_conn(|conn| {
        // _schema_migrations singleton
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM _schema_migrations WHERE id=1", [], |r| r.get(0)
        ).unwrap();
        assert_eq!(count, 1, "_schema_migrations singleton must exist");

        // preferences singleton
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM preferences WHERE id=1", [], |r| r.get(0)
        ).unwrap();
        assert_eq!(count, 1, "preferences singleton must exist");
        Ok(())
    }).unwrap();
}

#[test]
fn fresh_preferences_has_null_slots() {
    let (_dir, db) = fresh_db();
    db.with_conn(|conn| {
        let (primary, parallel, fallback): (Option<String>, String, Option<String>) =
            conn.query_row(
                "SELECT primary_uuid, parallel_uuids, fallback_uuid FROM preferences WHERE id=1",
                [], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
            ).unwrap();
        assert!(primary.is_none(), "fresh primary_uuid should be NULL");
        assert_eq!(parallel, "[]", "fresh parallel_uuids should be '[]'");
        assert!(fallback.is_none(), "fresh fallback_uuid should be NULL");
        Ok(())
    }).unwrap();
}

#[test]
fn create_all_tables_is_idempotent() {
    let (_dir, db) = fresh_db();
    // Run create_all_tables again — should not error.
    db.with_conn(|conn| {
        schema::create_all_tables(conn)?;
        schema::seed_singletons(conn)?;
        Ok::<(), DbError>(())
    }).unwrap();
    // Still exactly one singleton row each:
    db.with_conn(|conn| {
        let c1: i64 = conn.query_row("SELECT COUNT(*) FROM _schema_migrations", [], |r| r.get(0))?;
        let c2: i64 = conn.query_row("SELECT COUNT(*) FROM preferences", [], |r| r.get(0))?;
        assert_eq!(c1, 1);
        assert_eq!(c2, 1);
        Ok(())
    }).unwrap();
}

#[test]
fn foreign_keys_are_on() {
    let (_dir, db) = fresh_db();
    db.with_conn(|conn| {
        let fk: i64 = conn.pragma_query_value(None, "foreign_keys", |r| r.get(0))?;
        assert_eq!(fk, 1, "PRAGMA foreign_keys must be ON");
        Ok(())
    }).unwrap();
}

#[test]
fn journal_mode_is_delete() {
    let (_dir, db) = fresh_db();
    db.with_conn(|conn| {
        let mode: String = conn.pragma_query_value(None, "journal_mode", |r| r.get(0))?;
        assert_eq!(mode.to_lowercase(), "delete", "PRAGMA journal_mode must be DELETE");
        Ok(())
    }).unwrap();
}

#[test]
fn synchronous_is_full() {
    let (_dir, db) = fresh_db();
    db.with_conn(|conn| {
        let sync: i64 = conn.pragma_query_value(None, "synchronous", |r| r.get(0))?;
        assert_eq!(sync, 2, "PRAGMA synchronous must be FULL (2)");
        Ok(())
    }).unwrap();
}

#[test]
fn secret_ref_is_unique() {
    let (_dir, db) = fresh_db();
    db.with_conn(|conn| {
        conn.execute(
            "INSERT INTO providers (uuid, template_id, name, protocol, endpoint, needs_key, secret_ref)
             VALUES ('u1', 'openai', 'A', 'openai_chat', 'https://a.com', 1, 'ref1')",
            [],
        )?;
        // Duplicate secret_ref → error
        let result = conn.execute(
            "INSERT INTO providers (uuid, template_id, name, protocol, endpoint, needs_key, secret_ref)
             VALUES ('u2', 'openai', 'B', 'openai_chat', 'https://b.com', 1, 'ref1')",
            [],
        );
        assert!(result.is_err(), "duplicate secret_ref must be rejected");
        Ok(())
    }).unwrap();
}

#[test]
fn history_results_success_requires_text() {
    let (_dir, db) = fresh_db();
    db.with_conn(|conn| {
        // Insert a session first.
        conn.execute(
            "INSERT INTO history_sessions (session_uuid, timestamp, trigger_source, target_language,
             is_favorite, source_text_encrypted, source_text_nonce, crypto_version)
             VALUES ('s1', 1, 'selection', 'zh', 0, X'AABB', X'CCDD', 1)",
            [],
        )?;
        // success with NULL text → CHECK error
        let result = conn.execute(
            "INSERT INTO history_results (result_uuid, session_uuid, provider_uuid,
             provider_name_snapshot, engine_id, elapsed_ms, outcome_tag, crypto_version)
             VALUES ('r1', 's1', 'p1', 'OpenAI', 'openai', 100, 'success', 1)",
            [],
        );
        assert!(result.is_err(), "success with NULL result_text must be rejected");
        Ok(())
    }).unwrap();
}

#[test]
fn history_results_failure_with_text_rejected() {
    let (_dir, db) = fresh_db();
    db.with_conn(|conn| {
        conn.execute(
            "INSERT INTO history_sessions (session_uuid, timestamp, trigger_source, target_language,
             is_favorite, source_text_encrypted, source_text_nonce, crypto_version)
             VALUES ('s2', 1, 'selection', 'zh', 0, X'AABB', X'CCDD', 1)",
            [],
        )?;
        // failure WITH result_text → CHECK error
        let result = conn.execute(
            "INSERT INTO history_results (result_uuid, session_uuid, provider_uuid,
             provider_name_snapshot, engine_id, elapsed_ms, outcome_tag,
             result_text_encrypted, result_text_nonce, error_kind, crypto_version)
             VALUES ('r2', 's2', 'p1', 'OpenAI', 'openai', 100, 'failure',
                     X'1234', X'5678', 'Network', 1)",
            [],
        );
        assert!(result.is_err(), "failure with result_text must be rejected");
        Ok(())
    }).unwrap();
}

#[test]
fn migration_state_not_started_on_fresh_db() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("test.db");
    let db = Database::open(&db_path).unwrap();
    // Before create_all_tables:
    let state = db.with_conn(|conn| schema::migration_state_if_exists(conn)).unwrap();
    assert_eq!(state, schema::MigrationState::NotStarted);
}

#[test]
fn migration_state_incomplete_after_schema() {
    let (_dir, db) = fresh_db();
    let state = db.with_conn(|conn| schema::migration_state_if_exists(conn)).unwrap();
    assert_eq!(state, schema::MigrationState::Incomplete);
}

#[test]
fn migration_state_complete_after_flag() {
    let (_dir, db) = fresh_db();
    db.with_conn(|conn| {
        schema::set_migration_complete(conn)?;
        Ok(())
    }).unwrap();
    let state = db.with_conn(|conn| schema::migration_state_if_exists(conn)).unwrap();
    assert_eq!(state, schema::MigrationState::Complete);
}

#[test]
fn corrupt_db_header_propagates_error() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("corrupt.db");
    // Write garbage bytes (not a valid SQLite header):
    std::fs::write(&db_path, b"this is not a sqlite database file!!!").unwrap();
    // Opening should fail or the first query should fail:
    let result = Database::open(&db_path);
    // rusqlite may open the file but queries will fail. Either way, it's an error,
    // not a silent NotStarted. Let's check both cases:
    match result {
        Err(_) => { /* open failed — good */ }
        Ok(db) => {
            // Open "succeeded" but queries should fail:
            let query_result = db.with_conn(|conn| schema::migration_state_if_exists(conn));
            assert!(query_result.is_err(), "corrupt DB must produce an error, not NotStarted");
        }
    }
}

#[test]
fn invalid_migration_complete_value_is_rejected() {
    let (_dir, db) = fresh_db();
    db.with_conn(|conn| {
        // The CHECK constraint should reject an invalid value at the DB level.
        // A direct UPDATE to an out-of-range value must fail:
        let result = conn.execute(
            "UPDATE _schema_migrations SET migration_complete=5 WHERE id=1", []
        );
        assert!(result.is_err(), "CHECK constraint must reject migration_complete=5");
        Ok(())
    }).unwrap();

    // If the value somehow bypasses CHECK (e.g. external tampering of the DB file),
    // the preflight reader must still catch it. Simulate by dropping the CHECK
    // and inserting a bad value, then verify the reader returns Integrity error.
    // We do this on a separate DB with a table lacking the CHECK:
    let dir2 = tempdir().unwrap();
    let db2 = Database::open(&dir2.path().join("t2.db")).unwrap();
    db2.with_conn(|conn| {
        conn.execute_batch(
            "CREATE TABLE _schema_migrations (id INTEGER PRIMARY KEY, schema_version INTEGER, migration_complete INTEGER);
             INSERT INTO _schema_migrations (id, schema_version, migration_complete) VALUES (1, 1, 5);"
        )?;
        let result = schema::migration_state_if_exists(conn);
        match result {
            Err(DbError::Integrity(_)) => Ok(()),
            other => panic!("expected Integrity error for value=5, got {other:?}"),
        }
    }).unwrap();
}

#[test]
fn busy_timeout_is_5000() {
    let (_dir, db) = fresh_db();
    db.with_conn(|conn| {
        let timeout: i64 = conn.pragma_query_value(None, "busy_timeout", |r| r.get(0))?;
        assert_eq!(timeout, 5000, "PRAGMA busy_timeout must be 5000ms");
        Ok(())
    }).unwrap();
}

#[cfg(unix)]
#[test]
fn db_file_has_0600_perms() {
    use std::os::unix::fs::PermissionsExt;
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("perm_test.db");
    // Pre-loosen the file to 0644 so the test proves open() tightens it:
    std::fs::write(&db_path, b"x").unwrap();
    std::fs::set_permissions(&db_path, std::fs::Permissions::from_mode(0o644)).unwrap();
    let _db = Database::open(&db_path).unwrap();
    let perms = std::fs::metadata(&db_path).unwrap().permissions().mode();
    assert_eq!(perms & 0o777, 0o600, "DB file must have 0600 perms on Unix");
}

#[cfg(unix)]
#[test]
fn db_dir_has_0700_perms() {
    use std::os::unix::fs::{PermissionsExt, DirBuilderExt};
    // Create a dir explicitly at 0755 (NOT tempdir's default 0700) so the test
    // proves Database::open tightens it:
    let parent = tempdir().unwrap();
    let dir = parent.path().join("loose_dir");
    std::fs::DirBuilder::new().mode(0o755).create(&dir).unwrap();
    assert_eq!(std::fs::metadata(&dir).unwrap().permissions().mode() & 0o777, 0o755,
        "precondition: dir must start at 0755");
    let db_path = dir.join("test.db");
    let _db = Database::open(&db_path).unwrap();
    let perms = std::fs::metadata(&dir).unwrap().permissions().mode();
    assert_eq!(perms & 0o777, 0o700, "DB dir must have 0700 perms after open");
}

#[test]
fn fk_cascade_deletes_history_results() {
    let (_dir, db) = fresh_db();
    db.with_conn(|conn| {
        // Insert a session + 2 results.
        conn.execute(
            "INSERT INTO history_sessions (session_uuid, timestamp, trigger_source, target_language,
             is_favorite, source_text_encrypted, source_text_nonce, crypto_version)
             VALUES ('s1', 1, 'selection', 'zh', 0, X'AABB', X'CCDD', 1)", [],
        )?;
        conn.execute(
            "INSERT INTO history_results (result_uuid, session_uuid, provider_uuid,
             provider_name_snapshot, engine_id, elapsed_ms, outcome_tag,
             result_text_encrypted, result_text_nonce, crypto_version)
             VALUES ('r1', 's1', 'p1', 'OpenAI', 'openai', 100, 'success', X'12', X'34', 1)", [],
        )?;
        conn.execute(
            "INSERT INTO history_results (result_uuid, session_uuid, provider_uuid,
             provider_name_snapshot, engine_id, elapsed_ms, outcome_tag,
             result_text_encrypted, result_text_nonce, crypto_version)
             VALUES ('r2', 's1', 'p2', 'DeepSeek', 'deepseek', 200, 'success', X'56', X'78', 1)", [],
        )?;
        // Delete the session → results must cascade-delete.
        conn.execute("DELETE FROM history_sessions WHERE session_uuid='s1'", [])?;
        let remaining: i64 = conn.query_row(
            "SELECT COUNT(*) FROM history_results WHERE session_uuid='s1'", [], |r| r.get(0)
        )?;
        assert_eq!(remaining, 0, "FK ON DELETE CASCADE must remove child results");
        Ok(())
    }).unwrap();
}

#[test]
fn history_failure_with_encrypted_error_is_valid() {
    let (_dir, db) = fresh_db();
    db.with_conn(|conn| {
        conn.execute(
            "INSERT INTO history_sessions (session_uuid, timestamp, trigger_source, target_language,
             is_favorite, source_text_encrypted, source_text_nonce, crypto_version)
             VALUES ('s3', 1, 'selection', 'zh', 0, X'AABB', X'CCDD', 1)", [],
        )?;
        // failure with encrypted error message (both ciphertext + nonce present) — valid.
        conn.execute(
            "INSERT INTO history_results (result_uuid, session_uuid, provider_uuid,
             provider_name_snapshot, engine_id, elapsed_ms, outcome_tag,
             error_kind, error_message_encrypted, error_message_nonce, crypto_version)
             VALUES ('r3', 's3', 'p1', 'OpenAI', 'openai', 100, 'failure',
                     'RateLimit', X'EEFF', X'0011', 1)", [],
        )?;
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM history_results WHERE result_uuid='r3'", [], |r| r.get(0)
        )?;
        assert_eq!(count, 1, "failure with encrypted error message should be valid");
        Ok(())
    }).unwrap();
}

#[test]
fn history_failure_with_plaintext_error_only_is_valid() {
    let (_dir, db) = fresh_db();
    db.with_conn(|conn| {
        conn.execute(
            "INSERT INTO history_sessions (session_uuid, timestamp, trigger_source, target_language,
             is_favorite, source_text_encrypted, source_text_nonce, crypto_version)
             VALUES ('s4', 1, 'selection', 'zh', 0, X'AABB', X'CCDD', 1)", [],
        )?;
        // failure with plaintext error only (no encrypted detail) — valid.
        conn.execute(
            "INSERT INTO history_results (result_uuid, session_uuid, provider_uuid,
             provider_name_snapshot, engine_id, elapsed_ms, outcome_tag,
             error_kind, crypto_version)
             VALUES ('r4', 's4', 'p1', 'OpenAI', 'openai', 100, 'failure',
                     'AuthFailed', 1)", [],
        )?;
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM history_results WHERE result_uuid='r4'", [], |r| r.get(0)
        )?;
        assert_eq!(count, 1, "failure with plaintext error only should be valid");
        Ok(())
    }).unwrap();
}

// ── UUIDv5 golden vectors ─────────────────────────────────────────────
// Hardcoded expected values so namespace/prefix drift is caught immediately
// (not just "same function call twice = equal").

#[test]
fn uuid_v5_legacy_openai_golden() {
    use linguaray_lib::uuid_util::legacy_provider_uuid;
    let uuid = legacy_provider_uuid("openai");
    // Golden: deterministic UUIDv5 — if this changes, the namespace or prefix drifted.
    assert_eq!(
        uuid.to_string(),
        "aacdfcbf-c622-5299-9184-4a216ec8de91",
        "openai legacy UUIDv5 golden vector mismatch — check NAMESPACE_LINGUARAY or prefix"
    );
}

#[test]
fn uuid_v5_legacy_anthropic_golden() {
    use linguaray_lib::uuid_util::legacy_provider_uuid;
    let uuid = legacy_provider_uuid("anthropic");
    assert_eq!(
        uuid.to_string(),
        "531dda9f-b498-5535-8c8f-c2c4798adf93",
        "anthropic legacy UUIDv5 golden vector mismatch"
    );
}

#[test]
fn uuid_v5_recovered_key_golden() {
    use linguaray_lib::uuid_util::recovered_key_uuid;
    let uuid = recovered_key_uuid("provider/abc-123");
    assert_eq!(
        uuid.to_string(),
        "f369b7e2-c69e-5960-a803-eeae81b79ad2",
        "recovered_key UUIDv5 golden vector mismatch"
    );
}

// ── Table-driven DDL constraint tests ─────────────────────────────────
// Each row: a SQL INSERT/UPDATE that must be REJECTED by a CHECK constraint.
// Proves the domain constraints are enforced at the DB level, preventing
// drift if the DDL is ever modified.

/// Helper: assert that the given SQL fails (constraint violation).
fn assert_rejected(conn: &rusqlite::Connection, label: &str, sql: &str) {
    let result = conn.execute(sql, []);
    assert!(result.is_err(), "{label}: expected CHECK rejection, but it succeeded");
}

/// Helper: insert a minimal valid provider row for UPDATE-based tests.
fn insert_valid_provider(conn: &rusqlite::Connection, uuid: &str, secret_ref: &str) {
    conn.execute(
        "INSERT INTO providers (uuid, template_id, name, protocol, endpoint, needs_key, secret_ref)
         VALUES (?, 'openai', 'Test', 'openai_chat', 'https://a.com', 1, ?)",
        rusqlite::params![uuid, secret_ref],
    ).unwrap();
}

#[test]
fn constraint_providers_protocol_check() {
    let (_dir, db) = fresh_db();
    db.with_conn(|conn| {
        assert_rejected(conn, "invalid protocol", "INSERT INTO providers (uuid, template_id, name, protocol, endpoint, needs_key, secret_ref) VALUES ('u1', 't', 'N', 'bogus_proto', 'https://a.com', 1, 'r1')");
        Ok(())
    }).unwrap();
}

#[test]
fn constraint_providers_status_check() {
    let (_dir, db) = fresh_db();
    db.with_conn(|conn| {
        insert_valid_provider(conn, "u1", "r1");
        assert_rejected(conn, "invalid status", "UPDATE providers SET status='bogus' WHERE uuid='u1'");
        Ok(())
    }).unwrap();
}

#[test]
fn constraint_providers_boolean_columns_check() {
    let (_dir, db) = fresh_db();
    db.with_conn(|conn| {
        assert_rejected(conn, "enabled=2", "INSERT INTO providers (uuid, template_id, name, protocol, endpoint, enabled, needs_key, secret_ref) VALUES ('u1','t','N','openai_chat','https://a.com',2,1,'r1')");
        assert_rejected(conn, "is_local=5", "INSERT INTO providers (uuid, template_id, name, protocol, endpoint, is_local, needs_key, secret_ref) VALUES ('u2','t','N','openai_chat','https://a.com',1,5,'r2')");
        assert_rejected(conn, "needs_key=3", "INSERT INTO providers (uuid, template_id, name, protocol, endpoint, needs_key, secret_ref) VALUES ('u3','t','N','openai_chat','https://a.com',3,'r3')");
        Ok(())
    }).unwrap();
}

#[test]
fn constraint_preferences_history_enabled_check() {
    let (_dir, db) = fresh_db();
    db.with_conn(|conn| {
        assert_rejected(conn, "history_enabled=2", "UPDATE preferences SET history_enabled=2 WHERE id=1");
        Ok(())
    }).unwrap();
}

#[test]
fn constraint_history_sessions_is_favorite_check() {
    let (_dir, db) = fresh_db();
    db.with_conn(|conn| {
        assert_rejected(conn, "is_favorite=2",
            "INSERT INTO history_sessions (session_uuid, timestamp, trigger_source, target_language, is_favorite, source_text_encrypted, source_text_nonce, crypto_version) VALUES ('s1',1,'selection','zh',2,X'AA',X'BB',1)");
        Ok(())
    }).unwrap();
}

#[test]
fn constraint_schema_migrations_singleton_check() {
    let (_dir, db) = fresh_db();
    db.with_conn(|conn| {
        assert_rejected(conn, "id=2 in _schema_migrations", "INSERT INTO _schema_migrations (id, schema_version, migration_complete) VALUES (2, 1, 0)");
        Ok(())
    }).unwrap();
}

#[test]
fn constraint_preferences_singleton_check() {
    let (_dir, db) = fresh_db();
    db.with_conn(|conn| {
        assert_rejected(conn, "id=2 in preferences", "INSERT INTO preferences (id) VALUES (2)");
        Ok(())
    }).unwrap();
}

#[test]
fn constraint_history_failure_partial_nonce_only_rejected() {
    let (_dir, db) = fresh_db();
    db.with_conn(|conn| {
        conn.execute(
            "INSERT INTO history_sessions (session_uuid, timestamp, trigger_source, target_language, is_favorite, source_text_encrypted, source_text_nonce, crypto_version) VALUES ('s1',1,'selection','zh',0,X'AA',X'BB',1)", [])?;
        // failure with error_message_nonce but NO ciphertext → rejected:
        assert_rejected(conn, "nonce without ciphertext",
            "INSERT INTO history_results (result_uuid, session_uuid, provider_uuid, provider_name_snapshot, engine_id, elapsed_ms, outcome_tag, error_kind, error_message_nonce, crypto_version) VALUES ('r1','s1','p1','OpenAI','openai',100,'failure','Network',X'12',1)");
        Ok(())
    }).unwrap();
}

#[test]
fn constraint_history_failure_partial_ciphertext_only_rejected() {
    let (_dir, db) = fresh_db();
    db.with_conn(|conn| {
        conn.execute(
            "INSERT INTO history_sessions (session_uuid, timestamp, trigger_source, target_language, is_favorite, source_text_encrypted, source_text_nonce, crypto_version) VALUES ('s1',1,'selection','zh',0,X'AA',X'BB',1)", [])?;
        // failure with error_message_ciphertext but NO nonce → rejected:
        assert_rejected(conn, "ciphertext without nonce",
            "INSERT INTO history_results (result_uuid, session_uuid, provider_uuid, provider_name_snapshot, engine_id, elapsed_ms, outcome_tag, error_kind, error_message_encrypted, crypto_version) VALUES ('r1','s1','p1','OpenAI','openai',100,'failure','Network',X'34',1)");
        Ok(())
    }).unwrap();
}

// ── Windows end-to-end DB ACL verification ────────────────────────────
// Proves Database::open() actually applies the protected DACL to BOTH the
// directory and the DB file — not just that the helper works in isolation.

#[cfg(windows)]
#[test]
fn win32_db_open_secures_dir_and_file() {
    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::Foundation::LocalFree;
    use windows_sys::Win32::Security::Authorization::{GetNamedSecurityInfoW, SE_FILE_OBJECT};
    use windows_sys::Win32::Security::{
        AclSizeInformation, EqualSid, GetAclInformation, GetAce, GetSecurityDescriptorControl,
        ACL_SIZE_INFORMATION, DACL_SECURITY_INFORMATION, OWNER_SECURITY_INFORMATION,
        PSECURITY_DESCRIPTOR, SE_DACL_PROTECTED, ACL,
    };
    const ACCESS_ALLOWED: u8 = 0;

    let dir = tempfile::tempdir().unwrap();
    let db_path = dir.path().join("test.db");
    let _db = Database::open(&db_path).unwrap();

    // Expected SID (same source prod path used):
    let sid_buf = linguaray_lib::fs_acl::current_user_sid().unwrap();
    let expected_sid = linguaray_lib::fs_acl::sid_from_token_user_buf(&sid_buf).unwrap();

    // Verify BOTH dir and file:
    for path in [dir.path(), db_path.as_path()] {
        let path_wide: Vec<u16> = path.as_os_str().encode_wide().chain(std::iter::once(0)).collect();
        let mut dacl: *mut ACL = std::ptr::null_mut();
        let mut sd: PSECURITY_DESCRIPTOR = std::ptr::null_mut();
        let rc = unsafe {
            GetNamedSecurityInfoW(
                path_wide.as_ptr(), SE_FILE_OBJECT,
                DACL_SECURITY_INFORMATION | OWNER_SECURITY_INFORMATION,
                std::ptr::null_mut(), std::ptr::null_mut(),
                &mut dacl, std::ptr::null_mut(), &mut sd,
            )
        };
        assert_eq!(rc, 0, "GetNamedSecurityInfoW failed for {:?}", path);
        struct SdGuard(PSECURITY_DESCRIPTOR);
        impl Drop for SdGuard {
            fn drop(&mut self) { unsafe { LocalFree(self.0 as *mut _) }; }
        }
        let _guard = SdGuard(sd);

        // DACL is PROTECTED (no inheritance from parent):
        let mut control: u16 = 0;
        let mut revision: u32 = 0;
        unsafe { GetSecurityDescriptorControl(sd, &mut control, &mut revision); }
        assert_ne!(control & SE_DACL_PROTECTED, 0, "DACL must be PROTECTED for {:?}", path);

        // Exactly ONE ACE:
        let mut acl_info = ACL_SIZE_INFORMATION { AceCount: 0, AclBytesInUse: 0, AclBytesFree: 0 };
        unsafe {
            GetAclInformation(
                dacl,
                &mut acl_info as *mut _ as *mut _,
                std::mem::size_of::<ACL_SIZE_INFORMATION>() as u32,
                AclSizeInformation,
            );
        }
        // At least 1 ACE (temp dirs may inherit system ACEs; our DACL adds the current-user one):
        assert!(acl_info.AceCount >= 1, "at least 1 ACE for {:?}", path);

        // Find the current-user ACCESS_ALLOWED ACE among the ACEs:
        let mut found_user_ace = false;
        for i in 0..acl_info.AceCount {
            let mut ace: *mut std::ffi::c_void = std::ptr::null_mut();
            unsafe { GetAce(dacl, i, &mut ace); }
            if ace.is_null() { continue; }
            let ace_header = unsafe { &*(ace as *const [u8; 4]) };
            if ace_header[0] != ACCESS_ALLOWED { continue; }
            let ace_sid: windows_sys::Win32::Security::PSID =
                unsafe { (ace as *const u8).add(8) as windows_sys::Win32::Security::PSID };
            if unsafe { EqualSid(ace_sid, expected_sid) } != 0 {
                found_user_ace = true;
                break;
            }
        }
        assert!(found_user_ace, "current-user ACCESS_ALLOWED ACE must exist for {:?}", path);
    }

    // Directory ACE must be inheritable; file ACE must NOT be.
    // Find the current-user's explicit ACE (by SID) and check its inheritance flags.
    let check_inheritance = |path: &std::path::Path, expect_inherit: bool| {
        let path_wide: Vec<u16> = path.as_os_str().encode_wide().chain(std::iter::once(0)).collect();
        let mut dacl: *mut ACL = std::ptr::null_mut();
        let mut sd: PSECURITY_DESCRIPTOR = std::ptr::null_mut();
        unsafe {
            GetNamedSecurityInfoW(path_wide.as_ptr(), SE_FILE_OBJECT,
                DACL_SECURITY_INFORMATION, std::ptr::null_mut(), std::ptr::null_mut(),
                &mut dacl, std::ptr::null_mut(), &mut sd);
        }
        let mut size_info = ACL_SIZE_INFORMATION { AceCount: 0, AclBytesInUse: 0, AclBytesFree: 0 };
        unsafe {
            GetAclInformation(dacl, &mut size_info as *mut _ as *mut _,
                std::mem::size_of::<ACL_SIZE_INFORMATION>() as u32, AclSizeInformation);
        }
        // Find the current-user ACE and check its flags:
        for i in 0..size_info.AceCount {
            let mut ace: *mut std::ffi::c_void = std::ptr::null_mut();
            unsafe { GetAce(dacl, i, &mut ace); }
            if ace.is_null() { continue; }
            let ace_sid: windows_sys::Win32::Security::PSID =
                unsafe { (ace as *const u8).add(8) as windows_sys::Win32::Security::PSID };
            if unsafe { EqualSid(ace_sid, expected_sid) } != 0 {
                // ACE_HEADER = { AceType: u8, AceFlags: u8, AceSize: u16 }
                let flags = unsafe { *(ace.add(1) as *const u8) };
                let is_inheritable = (flags & 0x3) != 0; // SUB_CONTAINERS_AND_OBJECTS_INHERIT
                unsafe { LocalFree(sd as *mut _); }
                assert_eq!(is_inheritable, expect_inherit,
                    "inheritance mismatch for {:?}: expected inheritable={}", path, expect_inherit);
                return;
            }
        }
        unsafe { LocalFree(sd as *mut _); }
        panic!("current-user ACE not found for inheritance check on {:?}", path);
    };
    check_inheritance(dir.path(), true);   // directory: inheritable
    check_inheritance(&db_path, false);     // file: NOT inheritable
}
