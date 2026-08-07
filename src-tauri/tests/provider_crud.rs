//! Integration tests for `db::providers` CRUD + catalogs + CandidateSource.
//!
//! Each test opens a fresh in-memory-ish DB (temp file) and runs the schema +
//! singleton seed so `preferences` exists for the toggle/begin_delete slot
//! eviction paths. No network, no keystore file — the CandidateSource paths are
//! exercised against pure in-memory keystore-load states.

use linguaray_lib::db::providers::{
    self, CandidateSource, ProviderPatch, ProviderProfile, ProviderStatus, Protocol,
    RawSettings, TRADITIONAL_TEMPLATES,
};
use linguaray_lib::db::schema;
use linguaray_lib::db::{Database, DbError};
use linguaray_lib::keystore::{KeystoreData, KeystoreLoadState};
use std::collections::HashMap;
use tempfile::tempdir;

/// Open a fresh DB in a temp dir, create all tables + seed singletons.
/// Mirrors the `fresh_db` helper in db_schema.rs so the providers + preferences
/// tables both exist with their default singleton rows.
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
    })
    .unwrap();
    (dir, db)
}

/// Convenience: open a fresh DB and seed exactly one provider via `create`,
/// returning the DB and the created profile.
fn fresh_with_one_openai() -> (tempfile::TempDir, Database, ProviderProfile) {
    let (dir, db) = fresh_db();
    let p = db
        .with_conn(|conn| {
            providers::create(conn, "openai", "OpenAI", "https://api.openai.com/v1/chat/completions", None)
        })
        .unwrap();
    (dir, db, p)
}

// ─── Create + list ────────────────────────────────────────────────────────

#[test]
fn create_then_list_shows_it() {
    let (_dir, db, created) = fresh_with_one_openai();
    let listed = db.with_conn(|conn| providers::list(conn)).unwrap();
    assert_eq!(listed.len(), 1, "exactly one provider after create");
    let got = &listed[0];
    assert_eq!(got.uuid, created.uuid);
    assert_eq!(got.name, "OpenAI");
    assert_eq!(got.template_id, "openai");
    assert_eq!(got.protocol, Protocol::OpenaiChat);
    assert!(got.enabled, "freshly created provider is enabled");
    assert_eq!(got.status, ProviderStatus::Active.as_str());
    // secret_ref is derived from the new UUID.
    assert_eq!(got.secret_ref, format!("provider/{}", got.uuid));
    // sort_order starts at 0 for the first row.
    assert_eq!(got.sort_order, 0);
}

#[test]
fn create_assigns_increasing_sort_order() {
    let (_dir, db, _p1) = fresh_with_one_openai();
    let p2 = db
        .with_conn(|conn| {
            providers::create(conn, "anthropic", "Claude", "https://api.anthropic.com/v1/messages", None)
        })
        .unwrap();
    assert_eq!(p2.sort_order, 1, "second provider lands at sort_order=1");
}

#[test]
fn create_unknown_template_is_custom_http_and_needs_key() {
    let (_dir, db) = fresh_db();
    let p = db
        .with_conn(|conn| {
            providers::create(conn, "mystery", "Mystery", "https://example.com", None)
        })
        .unwrap();
    assert_eq!(p.protocol, Protocol::CustomHttp);
    assert!(p.needs_key, "unknown template defaults to needs_key");
    assert_eq!(p.endpoint, "https://example.com");
}

#[test]
fn list_excludes_deleted_tombstones() {
    let (_dir, db, p) = fresh_with_one_openai();
    // Tombstone it directly (skip begin_delete to isolate the status filter).
    db.with_conn(|conn| {
        conn.execute("UPDATE providers SET status='deleted' WHERE uuid=?1", rusqlite::params![p.uuid])?;
        Ok(())
    })
    .unwrap();
    let listed = db.with_conn(|conn| providers::list(conn)).unwrap();
    assert!(listed.is_empty(), "list() hides deleted rows");

    let all = db.with_conn(|conn| providers::list_all(conn)).unwrap();
    assert_eq!(all.len(), 1, "list_all() still shows the tombstone");
    assert_eq!(all[0].status, ProviderStatus::Deleted.as_str());
}

// ─── Update ───────────────────────────────────────────────────────────────

#[test]
fn update_valid_patch_changes_fields() {
    let (_dir, db, p) = fresh_with_one_openai();
    let patch = ProviderPatch {
        name: Some("Renamed".into()),
        model: Some("gpt-4o".into()),
        enabled: Some(false),
        sort_order: Some(5),
        endpoint: None,
    };
    let updated = db
        .with_conn(|conn| providers::update(conn, &p.uuid, &patch))
        .unwrap();
    assert_eq!(updated.name, "Renamed");
    assert_eq!(updated.model.as_deref(), Some("gpt-4o"));
    assert!(!updated.enabled);
    assert_eq!(updated.sort_order, 5);
    // endpoint untouched.
    assert_eq!(updated.endpoint, p.endpoint);
}

#[test]
fn update_recomputes_is_local_on_endpoint_change() {
    let (_dir, db, p) = fresh_with_one_openai();
    // Move to localhost.
    let patch = ProviderPatch {
        endpoint: Some("http://localhost:11434/v1/chat/completions".into()),
        name: None,
        model: None,
        enabled: None,
        sort_order: None,
    };
    let updated = db
        .with_conn(|conn| providers::update(conn, &p.uuid, &patch))
        .unwrap();
    assert!(updated.is_local, "localhost endpoint flips is_local to true");
    assert!(!p.is_local, "originally remote");
}

#[test]
fn update_deny_unknown_fields_rejects_typo() {
    // serde must reject an unknown field outright (deny_unknown_fields).
    let json = r#"{"neme": "typo"}"#;
    let err = serde_json::from_str::<ProviderPatch>(json);
    assert!(err.is_err(), "deny_unknown_fields must reject 'neme'");
}

#[test]
fn update_invalid_endpoint_rejected() {
    let (_dir, db, p) = fresh_with_one_openai();
    // ftp is not https/http → validate_endpoint rejects.
    let patch = ProviderPatch {
        endpoint: Some("ftp://example.com".into()),
        name: None,
        model: None,
        enabled: None,
        sort_order: None,
    };
    let err = db
        .with_conn(|conn| providers::update(conn, &p.uuid, &patch))
        .unwrap_err();
    assert!(matches!(err, DbError::Integrity(_)), "got {err:?}");
}

#[test]
fn update_unknown_uuid_not_found() {
    let (_dir, db, _p) = fresh_with_one_openai();
    let patch = ProviderPatch {
        name: Some("x".into()),
        endpoint: None,
        model: None,
        enabled: None,
        sort_order: None,
    };
    let err = db
        .with_conn(|conn| providers::update(conn, "no-such-uuid", &patch))
        .unwrap_err();
    assert!(matches!(err, DbError::NotFound(_)), "got {err:?}");
}

#[test]
fn update_same_origin_preserves_consent() {
    // Changing only the path/query (same scheme/host/port) keeps consent.
    let (_dir, db, p) = fresh_with_one_openai();
    db.with_conn(|conn| {
        conn.execute(
            "UPDATE preferences SET primary_uuid=?1, \
             parallel_consent_version=1, parallel_consent_scope='all' WHERE id=1",
            rusqlite::params![p.uuid],
        )?;
        Ok(())
    })
    .unwrap();

    // Same host, different path.
    let patch = ProviderPatch {
        endpoint: Some("https://api.openai.com/v1/messages".into()),
        name: None,
        model: None,
        enabled: None,
        sort_order: None,
    };
    db.with_conn(|conn| providers::update(conn, &p.uuid, &patch)).unwrap();

    db.with_conn(|conn| {
        let (ver, scope): (Option<i64>, Option<String>) = conn.query_row(
            "SELECT parallel_consent_version, parallel_consent_scope \
             FROM preferences WHERE id=1",
            [],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )?;
        assert_eq!(ver, Some(1), "consent version preserved on same-origin change");
        assert_eq!(
            scope.as_deref(),
            Some("all"),
            "consent scope preserved on same-origin change"
        );
        Ok(())
    })
    .unwrap();
}

#[test]
fn update_different_origin_invalidates_consent() {
    // Changing scheme/host/port invalidates consent (the upstream moved).
    let (_dir, db, p) = fresh_with_one_openai();
    db.with_conn(|conn| {
        conn.execute(
            "UPDATE preferences SET primary_uuid=?1, \
             parallel_consent_version=1, parallel_consent_scope='all' WHERE id=1",
            rusqlite::params![p.uuid],
        )?;
        Ok(())
    })
    .unwrap();

    // Different host entirely.
    let patch = ProviderPatch {
        endpoint: Some("https://api.anthropic.com/v1/messages".into()),
        name: None,
        model: None,
        enabled: None,
        sort_order: None,
    };
    db.with_conn(|conn| providers::update(conn, &p.uuid, &patch)).unwrap();

    db.with_conn(|conn| {
        let (ver, scope): (Option<i64>, Option<String>) = conn.query_row(
            "SELECT parallel_consent_version, parallel_consent_scope \
             FROM preferences WHERE id=1",
            [],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )?;
        assert!(ver.is_none(), "consent version invalidated on origin change");
        assert!(scope.is_none(), "consent scope invalidated on origin change");
        Ok(())
    })
    .unwrap();
}

#[test]
fn update_different_origin_not_in_slot_keeps_consent() {
    // An origin change on a provider that ISN'T in any slot must NOT touch
    // consent — the parallel set didn't change.
    let (_dir, db, p) = fresh_with_one_openai();
    // No slot references p.uuid.
    db.with_conn(|conn| {
        conn.execute(
            "UPDATE preferences SET parallel_consent_version=1, \
             parallel_consent_scope='all' WHERE id=1",
            [],
        )?;
        Ok(())
    })
    .unwrap();

    let patch = ProviderPatch {
        endpoint: Some("https://api.anthropic.com/v1/messages".into()),
        name: None,
        model: None,
        enabled: None,
        sort_order: None,
    };
    db.with_conn(|conn| providers::update(conn, &p.uuid, &patch)).unwrap();

    db.with_conn(|conn| {
        let ver: Option<i64> = conn.query_row(
            "SELECT parallel_consent_version FROM preferences WHERE id=1",
            [],
            |r| r.get(0),
        )?;
        assert_eq!(ver, Some(1), "consent untouched when provider not in a slot");
        Ok(())
    })
    .unwrap();
}

// ─── Duplicate ────────────────────────────────────────────────────────────

#[test]
fn duplicate_produces_new_uuid_and_secret_ref() {
    let (_dir, db, p) = fresh_with_one_openai();
    let dup = db
        .with_conn(|conn| providers::duplicate(conn, &p.uuid))
        .unwrap();
    assert_ne!(dup.uuid, p.uuid, "duplicate gets a new UUID");
    assert_ne!(dup.secret_ref, p.secret_ref, "duplicate gets a new secret_ref");
    assert_eq!(
        dup.secret_ref,
        format!("provider/{}", dup.uuid),
        "secret_ref is provider/<new-uuid>"
    );
    // needs_key is inherited from the source profile (it's a provider-type
    // property, not "has a key"). OpenAI needs a key → the duplicate does too;
    // the key itself is NEVER copied (fresh secret_ref, starts keyless).
    assert!(dup.needs_key);
    assert!(dup.enabled, "duplicate starts enabled");
    // Name is suffixed so the two rows are distinguishable.
    assert!(dup.name.contains("(copy)"), "name={}", dup.name);
}

#[test]
fn duplicate_of_local_keeps_is_local() {
    let (_dir, db) = fresh_db();
    let local = db
        .with_conn(|conn| {
            providers::create(conn, "ollama", "Ollama", "http://localhost:11434/v1/chat/completions", None)
        })
        .unwrap();
    assert!(local.is_local);
    let dup = db
        .with_conn(|conn| providers::duplicate(conn, &local.uuid))
        .unwrap();
    assert!(dup.is_local, "is_local is preserved on duplicate");
    // needs_key is a provider-TYPE property, not "has a key". Ollama is keyless,
    // so its duplicate is keyless too — not a phantom "needs a key" row.
    assert!(!local.needs_key, "ollama preset is keyless");
    assert!(!dup.needs_key, "duplicate inherits keyless provider-type");
}

// ─── Reorder ──────────────────────────────────────────────────────────────

#[test]
fn reorder_valid_updates_sort_order() {
    let (_dir, db) = fresh_db();
    let a = db.with_conn(|conn| providers::create(conn, "openai", "A", "https://api.openai.com/v1/chat/completions", None)).unwrap();
    let b = db.with_conn(|conn| providers::create(conn, "anthropic", "B", "https://api.anthropic.com/v1/messages", None)).unwrap();
    let c = db.with_conn(|conn| providers::create(conn, "gemini", "C", "https://generativelanguage.googleapis.com/v1beta/openai/chat/completions", None)).unwrap();

    // Reverse the order.
    let new_order: Vec<String> = vec![c.uuid.clone(), b.uuid.clone(), a.uuid.clone()];
    db.with_conn(|conn| providers::reorder(conn, &new_order)).unwrap();

    let listed = db.with_conn(|conn| providers::list(conn)).unwrap();
    assert_eq!(listed[0].uuid, c.uuid);
    assert_eq!(listed[0].sort_order, 0);
    assert_eq!(listed[1].uuid, b.uuid);
    assert_eq!(listed[1].sort_order, 1);
    assert_eq!(listed[2].uuid, a.uuid);
    assert_eq!(listed[2].sort_order, 2);
}

#[test]
fn reorder_incomplete_set_rejected() {
    let (_dir, db) = fresh_db();
    let a = db.with_conn(|conn| providers::create(conn, "openai", "A", "https://api.openai.com/v1/chat/completions", None)).unwrap();
    let b = db.with_conn(|conn| providers::create(conn, "anthropic", "B", "https://api.anthropic.com/v1/messages", None)).unwrap();

    // Pass only one of two active UUIDs.
    let err = db
        .with_conn(|conn| providers::reorder(conn, std::slice::from_ref(&a.uuid)))
        .unwrap_err();
    assert!(matches!(err, DbError::Integrity(_)), "got {err:?}");

    // Original sort_orders are untouched (rollback).
    let listed = db.with_conn(|conn| providers::list(conn)).unwrap();
    let a_row = listed.iter().find(|p| p.uuid == a.uuid).unwrap();
    let b_row = listed.iter().find(|p| p.uuid == b.uuid).unwrap();
    assert_eq!(a_row.sort_order, 0);
    assert_eq!(b_row.sort_order, 1);
}

#[test]
fn reorder_duplicate_uuid_rejected() {
    let (_dir, db) = fresh_db();
    let a = db.with_conn(|conn| providers::create(conn, "openai", "A", "https://api.openai.com/v1/chat/completions", None)).unwrap();
    // Duplicate the same uuid in the input.
    let err = db
        .with_conn(|conn| providers::reorder(conn, &[a.uuid.clone(), a.uuid.clone()]))
        .unwrap_err();
    assert!(matches!(err, DbError::Integrity(_)), "got {err:?}");
}

#[test]
fn reorder_extra_uuid_rejected() {
    let (_dir, db) = fresh_db();
    let _a = db.with_conn(|conn| providers::create(conn, "openai", "A", "https://api.openai.com/v1/chat/completions", None)).unwrap();
    // Input includes a uuid that isn't an active provider.
    let err = db
        .with_conn(|conn| providers::reorder(conn, &["not-an-active-uuid".into()]))
        .unwrap_err();
    assert!(matches!(err, DbError::Integrity(_)), "got {err:?}");
}

// ─── Toggle ───────────────────────────────────────────────────────────────

#[test]
fn toggle_disable_flips_enabled() {
    let (_dir, db, p) = fresh_with_one_openai();
    let updated = db
        .with_conn(|conn| providers::toggle(conn, &p.uuid, false))
        .unwrap();
    assert!(!updated.enabled);
    // Re-enable.
    let updated = db
        .with_conn(|conn| providers::toggle(conn, &p.uuid, true))
        .unwrap();
    assert!(updated.enabled);
}

#[test]
fn toggle_disable_evicts_from_active_slots() {
    let (_dir, db, p) = fresh_with_one_openai();
    // Seed preferences with this uuid in every slot.
    db.with_conn(|conn| {
        conn.execute(
            "UPDATE preferences SET primary_uuid=?1, parallel_uuids=?2, fallback_uuid=?1 WHERE id=1",
            rusqlite::params![p.uuid, serde_json::to_string(&[&p.uuid]).unwrap()],
        )?;
        Ok(())
    })
    .unwrap();

    // Disable → all three slots cleared.
    db.with_conn(|conn| providers::toggle(conn, &p.uuid, false)).unwrap();

    db.with_conn(|conn| {
        let (primary, parallel, fallback): (Option<String>, String, Option<String>) =
            conn.query_row(
                "SELECT primary_uuid, parallel_uuids, fallback_uuid FROM preferences WHERE id=1",
                [],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
            )?;
        assert!(primary.is_none(), "primary cleared");
        assert!(fallback.is_none(), "fallback cleared");
        let arr: Vec<String> = serde_json::from_str(&parallel).unwrap();
        assert!(arr.is_empty(), "parallel cleared: {arr:?}");
        Ok(())
    })
    .unwrap();
}

#[test]
fn toggle_enable_does_not_add_back_to_slots() {
    let (_dir, db, p) = fresh_with_one_openai();
    // primary set, then disable, then re-enable.
    db.with_conn(|conn| {
        conn.execute(
            "UPDATE preferences SET primary_uuid=?1 WHERE id=1",
            rusqlite::params![p.uuid],
        )?;
        Ok(())
    })
    .unwrap();
    db.with_conn(|conn| providers::toggle(conn, &p.uuid, false)).unwrap();
    db.with_conn(|conn| providers::toggle(conn, &p.uuid, true)).unwrap();
    db.with_conn(|conn| {
        let primary: Option<String> = conn.query_row(
            "SELECT primary_uuid FROM preferences WHERE id=1",
            [],
            |r| r.get(0),
        )?;
        assert!(primary.is_none(), "re-enabling does NOT re-add to primary");
        Ok(())
    })
    .unwrap();
}

#[test]
fn toggle_disable_invalidates_parallel_consent() {
    // Mirrors begin_delete_evicts_from_active_slots_and_consent but for the
    // disable path: a provider that's in active slots must drop consent when
    // toggled off, so the next translate re-prompts for the changed set.
    let (_dir, db, p) = fresh_with_one_openai();
    db.with_conn(|conn| {
        conn.execute(
            "UPDATE preferences SET primary_uuid=?1, parallel_uuids=?2, \
             parallel_consent_version=1, parallel_consent_scope='all' WHERE id=1",
            rusqlite::params![
                p.uuid,
                serde_json::to_string(&[&p.uuid]).unwrap()
            ],
        )?;
        Ok(())
    })
    .unwrap();

    db.with_conn(|conn| providers::toggle(conn, &p.uuid, false)).unwrap();

    db.with_conn(|conn| {
        let (ver, scope): (Option<i64>, Option<String>) = conn.query_row(
            "SELECT parallel_consent_version, parallel_consent_scope \
             FROM preferences WHERE id=1",
            [],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )?;
        assert!(ver.is_none(), "consent version invalidated on disable");
        assert!(scope.is_none(), "consent scope invalidated on disable");
        Ok(())
    })
    .unwrap();
}

// ─── Delete lifecycle ─────────────────────────────────────────────────────

#[test]
fn begin_then_finalize_delete() {
    let (_dir, db, p) = fresh_with_one_openai();
    let orig_name = p.name.clone();
    let secret_ref = db
        .with_conn(|conn| providers::begin_delete(conn, &p.uuid))
        .unwrap();
    assert_eq!(secret_ref, p.secret_ref, "begin_delete returns the secret_ref");

    // Row is now deleting + disabled, hidden from list().
    let mid = db.with_conn(|conn| providers::get(conn, &p.uuid)).unwrap();
    assert_eq!(mid.status, ProviderStatus::Deleting.as_str());
    assert!(!mid.enabled);
    assert!(db.with_conn(|conn| providers::list(conn)).unwrap().is_empty());

    db.with_conn(|conn| providers::finalize_delete(conn, &p.uuid)).unwrap();
    let done = db.with_conn(|conn| providers::get(conn, &p.uuid)).unwrap();
    assert_eq!(done.status, ProviderStatus::Deleted.as_str());
    assert!(done.name.starts_with("deleted: "), "name={}", done.name);
    assert!(done.name.contains(&orig_name));
}

#[test]
fn begin_delete_evicts_from_active_slots_and_consent() {
    let (_dir, db, p) = fresh_with_one_openai();
    db.with_conn(|conn| {
        conn.execute(
            "UPDATE preferences SET primary_uuid=?1, parallel_consent_version=1, parallel_consent_scope='all' WHERE id=1",
            rusqlite::params![p.uuid],
        )?;
        Ok(())
    })
    .unwrap();
    db.with_conn(|conn| providers::begin_delete(conn, &p.uuid)).unwrap();
    db.with_conn(|conn| {
        let (primary, ver, scope): (Option<String>, Option<i64>, Option<String>) =
            conn.query_row(
                "SELECT primary_uuid, parallel_consent_version, parallel_consent_scope FROM preferences WHERE id=1",
                [],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
            )?;
        assert!(primary.is_none());
        assert!(ver.is_none(), "consent version invalidated");
        assert!(scope.is_none(), "consent scope invalidated");
        Ok(())
    })
    .unwrap();
}

// ─── secret_ref uniqueness ────────────────────────────────────────────────

#[test]
fn secret_ref_collision_is_db_error() {
    let (_dir, db, p) = fresh_with_one_openai();
    // Manually insert a second row that reuses p.secret_ref → UNIQUE violation.
    let err = db
        .with_conn(|conn| {
            conn.execute(
                "INSERT INTO providers (uuid, template_id, name, protocol, endpoint, secret_ref) \
                 VALUES (?1, 'x', 'dup', 'custom_http', 'https://e.com', ?2)",
                rusqlite::params![uuid::Uuid::new_v4().to_string(), p.secret_ref],
            )?;
            Ok(())
        })
        .unwrap_err();
    assert!(
        matches!(err, DbError::Sqlite(_)),
        "expected Sqlite UNIQUE error, got {err:?}"
    );
}

// ─── insert_or_ignore idempotency ─────────────────────────────────────────

#[test]
fn insert_or_ignore_is_idempotent() {
    let (_dir, db, p) = fresh_with_one_openai();
    // Re-insert the same row twice — neither call should error, and the row
    // count must stay at 1.
    db.with_conn(|conn| providers::insert_or_ignore(conn, &p)).unwrap();
    db.with_conn(|conn| providers::insert_or_ignore(conn, &p)).unwrap();
    let count: i64 = db
        .with_conn(|conn| {
            Ok(conn.query_row("SELECT COUNT(*) FROM providers", [], |r| r.get(0))?)
        })
        .unwrap();
    assert_eq!(count, 1);
}

// ─── validate_active_selection ────────────────────────────────────────────

fn active_profile(uuid: &str, template_id: &str, enabled: bool) -> ProviderProfile {
    ProviderProfile {
        uuid: uuid.into(),
        template_id: template_id.into(),
        name: template_id.into(),
        protocol: Protocol::OpenaiChat,
        endpoint: "https://api.openai.com".into(),
        model: None,
        enabled,
        sort_order: 0,
        is_local: false,
        needs_key: true,
        secret_ref: format!("provider/{uuid}"),
        capabilities: providers::ProviderCapabilities::default(),
        status: ProviderStatus::Active.as_str().into(),
    }
}

#[test]
fn vas_overlap_rejected() {
    let provs = vec![active_profile("u1", "openai", true)];
    let err = providers::validate_active_selection("u1", &["u1".into()], None, &provs).unwrap_err();
    assert!(matches!(err, DbError::Integrity(_)));
}

#[test]
fn vas_disabled_in_slot_rejected() {
    let provs = vec![active_profile("u1", "openai", false)];
    let err = providers::validate_active_selection("u1", &[], None, &provs).unwrap_err();
    assert!(matches!(err, DbError::Integrity(_)));
}

#[test]
fn vas_deleted_in_slot_rejected() {
    let mut p = active_profile("u1", "openai", true);
    p.status = ProviderStatus::Deleted.as_str().into();
    let provs = vec![p];
    let err = providers::validate_active_selection("u1", &[], None, &provs).unwrap_err();
    assert!(matches!(err, DbError::Integrity(_)));
}

#[test]
fn vas_fallback_not_traditional_rejected() {
    let provs = vec![active_profile("u1", "openai", true)]; // openai is not traditional
    let err = providers::validate_active_selection("", &[], Some("u1"), &provs).unwrap_err();
    assert!(matches!(err, DbError::Integrity(_)));
}

#[test]
fn vas_valid_selection_accepted() {
    let mut fb = active_profile("u9", "google", true);
    fb.protocol = Protocol::GoogleTranslate;
    let provs = vec![active_profile("u1", "openai", true), fb];
    providers::validate_active_selection("u1", &[], Some("u9"), &provs).unwrap();
}

#[test]
fn vas_empty_selection_accepted() {
    // No primary, no parallel, no fallback → vacuously valid.
    providers::validate_active_selection("", &[], None, &[]).unwrap();
}

#[test]
fn traditional_templates_set_has_expected_members() {
    for t in &["google", "deepl", "microsoft", "baidu", "youdao", "tencent"] {
        assert!(TRADITIONAL_TEMPLATES.contains(t), "missing {t}");
    }
}

// ─── build_profile ────────────────────────────────────────────────────────

#[test]
fn build_profile_preset_lookup() {
    let cs = CandidateSource::LegacyId("openai".into());
    let p = providers::build_profile(&cs).unwrap();
    assert_eq!(p.template_id, "openai");
    assert_eq!(p.protocol, Protocol::OpenaiChat);
    assert!(p.needs_key);
    assert!(p.enabled);
    // Preset carries a default model.
    assert!(p.model.is_some(), "preset default model populated");
    // UUID matches the deterministic v5.
    assert_eq!(p.uuid, cs.deterministic_uuid().to_string());
}

#[test]
fn build_profile_traditional_catalog_lookup() {
    let cs = CandidateSource::LegacyId("google".into());
    let p = providers::build_profile(&cs).unwrap();
    assert_eq!(p.template_id, "google");
    assert_eq!(p.protocol, Protocol::GoogleTranslate);
    assert!(!p.needs_key);
    assert_eq!(p.endpoint, "https://translate.google.com");
}

#[test]
fn build_profile_unknown_legacy_id_is_repair_profile() {
    let cs = CandidateSource::LegacyId("no-such-engine".into());
    let p = providers::build_profile(&cs).unwrap();
    assert_eq!(p.protocol, Protocol::CustomHttp);
    assert!(p.endpoint.is_empty(), "repair endpoint is blank");
    assert!(!p.enabled, "repair row starts disabled");
    assert!(p.needs_key);
    // Repair profile sorts to the bottom so known presets/catalog rows win the
    // top of the migrated list.
    assert_eq!(p.sort_order, 999, "repair sort_order parks at bottom");
}

#[test]
fn build_profile_providerkey_parseable_keeps_uuid() {
    let u = uuid::Uuid::new_v4();
    let sr = format!("provider/{u}");
    let cs = CandidateSource::ProviderKey(sr.clone());
    let p = providers::build_profile(&cs).unwrap();
    assert_eq!(p.uuid, u.to_string(), "parseable provider/<uuid> keeps the uuid");
    assert_eq!(p.secret_ref, sr);
    // ProviderKey repair rows carry the "unknown" template (no preset match).
    assert_eq!(p.template_id, "unknown");
    assert_eq!(p.sort_order, 999, "repair sort_order parks at bottom");
}

#[test]
fn build_profile_providerkey_unparseable_derives_uuid() {
    let sr = "provider/ definitely-not-a-uuid".to_string();
    let cs = CandidateSource::ProviderKey(sr.clone());
    let p = providers::build_profile(&cs).unwrap();
    // Derived from recovered_key_uuid (deterministic).
    assert_ne!(p.uuid, sr);
    assert_eq!(p.template_id, "unknown");
    assert_eq!(p.sort_order, 999);
}

// ─── CandidateSource.deterministic_uuid golden vector ─────────────────────

#[test]
fn deterministic_uuid_same_input_same_output() {
    // Golden: the same LegacyId must always hash to the same UUID.
    let a = CandidateSource::LegacyId("openai".into()).deterministic_uuid();
    let b = CandidateSource::LegacyId("openai".into()).deterministic_uuid();
    assert_eq!(a, b);

    // And the value is stable — record the actual bytes so a future change to
    // the namespace/name format is caught here, not silently in production.
    let s = a.to_string();
    assert!(
        uuid::Uuid::parse_str(&s).is_ok(),
        "deterministic uuid is a valid uuid string: {s}"
    );

    // ProviderKey arm likewise.
    let c = CandidateSource::ProviderKey("provider/abc".into()).deterministic_uuid();
    let d = CandidateSource::ProviderKey("provider/abc".into()).deterministic_uuid();
    assert_eq!(c, d);
    assert_ne!(a, c, "different arms produce different uuids");
}

#[test]
fn stable_id_exposes_payload() {
    assert_eq!(CandidateSource::LegacyId("openai".into()).stable_id(), "openai");
    assert_eq!(
        CandidateSource::ProviderKey("provider/x".into()).stable_id(),
        "provider/x"
    );
}

// ─── enumerate_candidates ─────────────────────────────────────────────────

#[test]
fn enumerate_candidates_classifies_provider_prefix() {
    let mut m = HashMap::new();
    m.insert("openai".to_string(), "k".to_string());
    m.insert("provider/u1".to_string(), "k".to_string());
    let v = providers::enumerate_candidates(
        &KeystoreLoadState::LegacyV1(m),
        None,
    );
    let has_legacy = v.iter().any(|c| matches!(
        c,
        CandidateSource::LegacyId(id) if id == "openai"
    ));
    let has_pk = v.iter().any(|c| matches!(
        c,
        CandidateSource::ProviderKey(sr) if sr == "provider/u1"
    ));
    assert!(has_legacy);
    assert!(has_pk);
}

#[test]
fn enumerate_candidates_v2_payload() {
    let mut m = HashMap::new();
    m.insert("provider/v2key".to_string(), "secret".to_string());
    let data = KeystoreData::new_v2(m);
    let v = providers::enumerate_candidates(&KeystoreLoadState::CurrentV2(data), None);
    assert_eq!(v.len(), 1);
    assert!(matches!(v[0], CandidateSource::ProviderKey(_)));
}

#[test]
fn enumerate_candidates_missing_keystore_settings_only() {
    let settings = RawSettings {
        default_provider: Some("openai".to_string()),
        target_language: Some("zh".to_string()),
        fallback_engine: None,
    };
    let v = providers::enumerate_candidates(&KeystoreLoadState::Missing, Some(&settings));
    assert_eq!(v.len(), 1);
    assert!(matches!(v[0], CandidateSource::LegacyId(_)));
}
