//! Provider domain types + CRUD (S2a step 3).
//!
//! This module owns the `ProviderProfile` row type, the `providers` table CRUD,
//! and the two migration/recovery candidate catalogs (`TraditionalProviderCatalog`,
//! `CandidateSource`).
//!
//! ## DB↔struct mapping
//!
//! No ORM. Every query uses `rusqlite::params![]` + manual field extraction in
//! the row closure. `Protocol` is stored as TEXT (`openai_chat`, …). Booleans
//! (`enabled`, `is_local`, `needs_key`) are INTEGER 0/1. `capabilities` is a TEXT
//! column holding a JSON object. `status` is a free-form string
//! (`"active" | "deleting" | "deleted"`).
//!
//! ## Lock-order
//!
//! All CRUD takes `&Connection` / `&mut Connection` directly (the caller drives
//! the DB mutex via `Database::with_conn`). None of these functions touch the
//! keystore — deletion returns the `secret_ref` so the caller can purge the key
//! in a separate keystore-locked step (see db/mod.rs lock-order rule).

use std::collections::{BTreeSet, HashMap, HashSet};

use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

use crate::uuid_util;
use crate::db::DbError;

// ─── Protocol ─────────────────────────────────────────────────────────────

/// Wire protocol family. Stored as TEXT in the `providers` table; the serde
/// snake_case names are exactly the CHECK-constraint whitelist in schema.rs.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Protocol {
    OpenaiChat,
    Anthropic,
    Gemini,
    GoogleTranslate,
    CustomHttp,
}

impl Protocol {
    /// Serialize to the DB TEXT form. Kept explicit (rather than going through
    /// serde_json) so the DB column value is stable and grep-able.
    pub fn as_db_str(&self) -> &'static str {
        match self {
            Protocol::OpenaiChat => "openai_chat",
            Protocol::Anthropic => "anthropic",
            Protocol::Gemini => "gemini",
            Protocol::GoogleTranslate => "google_translate",
            Protocol::CustomHttp => "custom_http",
        }
    }

    /// Parse a DB TEXT value back into the enum. Unknown strings → Integrity
    /// error (the CHECK constraint should already prevent this, but fail-closed).
    fn from_db_str(s: &str) -> Result<Self, DbError> {
        match s {
            "openai_chat" => Ok(Protocol::OpenaiChat),
            "anthropic" => Ok(Protocol::Anthropic),
            "gemini" => Ok(Protocol::Gemini),
            "google_translate" => Ok(Protocol::GoogleTranslate),
            "custom_http" => Ok(Protocol::CustomHttp),
            other => Err(DbError::Integrity(format!(
                "unknown protocol: {other}"
            ))),
        }
    }
}

// ─── ProviderStatus ───────────────────────────────────────────────────────

/// Lifecycle of a provider row. The DB stores the snake_case string in the
/// `status` TEXT column; `ProviderProfile.status` carries the raw string so the
/// (de)serialization contract for the row is uniform.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ProviderStatus {
    Active,
    Deleting,
    Deleted,
}

impl ProviderStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            ProviderStatus::Active => "active",
            ProviderStatus::Deleting => "deleting",
            ProviderStatus::Deleted => "deleted",
        }
    }
}

// ─── Capabilities + Profile ───────────────────────────────────────────────

/// Feature flags a provider advertises. Stored as a JSON object in the
/// `capabilities` TEXT column (`{"balance":false,"quota":false,"model_list":false}`).
/// All-`false` is the default (most providers don't expose balance/quota/model-list).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProviderCapabilities {
    pub balance: bool,
    pub quota: bool,
    pub model_list: bool,
}

/// One row of the `providers` table.
///
/// `status` is kept as a raw `String` (not `ProviderStatus`) so the row type is
/// a faithful mirror of the DB column — callers that need the typed form parse
/// it explicitly. This also keeps (de)serialization lossless for rows whose
/// status string the enum doesn't model yet.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderProfile {
    pub uuid: String,
    pub template_id: String,
    pub name: String,
    pub protocol: Protocol,
    pub endpoint: String,
    pub model: Option<String>,
    pub enabled: bool,
    pub sort_order: i32,
    pub is_local: bool,
    pub needs_key: bool,
    pub secret_ref: String,
    pub capabilities: ProviderCapabilities,
    /// `"active" | "deleting" | "deleted"`.
    pub status: String,
}

/// Patch body for [`update`]. `#[serde(deny_unknown_fields)]` so the API layer
/// rejects typo'd field names instead of silently dropping them.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProviderPatch {
    pub name: Option<String>,
    pub endpoint: Option<String>,
    pub model: Option<String>,
    pub enabled: Option<bool>,
    pub sort_order: Option<i32>,
}

// ─── Row ↔ value helpers ──────────────────────────────────────────────────

/// Map one rusqlite row into a `ProviderProfile`. Column order matches the
/// SELECT list used by every read in this module (kept in one place so a schema
/// column reorder only touches this closure + the SELECTs).
fn row_to_profile(r: &rusqlite::Row<'_>) -> rusqlite::Result<ProviderProfile> {
    let protocol_str: String = r.get("protocol")?;
    let protocol = Protocol::from_db_str(&protocol_str).map_err(|e| {
        // DbError isn't a rusqlite::FromSqlError; surface as SqliteFailure with a
        // clear message so the read surfaces the integrity fault instead of panicking.
        rusqlite::Error::FromSqlConversionFailure(
            std::mem::size_of::<String>(),
            rusqlite::types::Type::Text,
            Box::new(e),
        )
    })?;
    let enabled: i64 = r.get("enabled")?;
    let is_local: i64 = r.get("is_local")?;
    let needs_key: i64 = r.get("needs_key")?;
    let caps_json: String = r.get("capabilities")?;
    let capabilities: ProviderCapabilities = serde_json::from_str(&caps_json)
        .unwrap_or_default();
    Ok(ProviderProfile {
        uuid: r.get("uuid")?,
        template_id: r.get("template_id")?,
        name: r.get("name")?,
        protocol,
        endpoint: r.get("endpoint")?,
        model: r.get("model")?,
        enabled: enabled != 0,
        sort_order: r.get("sort_order")?,
        is_local: is_local != 0,
        needs_key: needs_key != 0,
        secret_ref: r.get("secret_ref")?,
        capabilities,
        status: r.get("status")?,
    })
}

/// Column list shared by every SELECT (so `row_to_profile` can read by name).
const SELECT_COLS: &str = "uuid, template_id, name, protocol, endpoint, model, \
     enabled, sort_order, is_local, needs_key, secret_ref, capabilities, status";

// ─── Capabilities JSON helper ─────────────────────────────────────────────

/// Capabilities column default for new rows. Empty JSON object → all-false after
/// `ProviderCapabilities::default()`.
fn caps_to_json(caps: &ProviderCapabilities) -> String {
    serde_json::to_string(caps).unwrap_or_else(|_| "{}".to_string())
}

// ─── is_local classification ──────────────────────────────────────────────

/// A provider is LOCAL iff its endpoint host is loopback. Matches the
/// `service::is_local` rule (localhost / 127.0.0.1 / ::1 / 0.0.0.0). The `url`
/// crate keeps the brackets on IPv6 hosts (`"[::1]"`), so both the bracketed
/// and bare forms are accepted. Any parse failure is treated as not-local
/// (fail-open for remote, never silently grants local-sacred status).
pub fn endpoint_is_local(endpoint: &str) -> bool {
    let parsed = match url::Url::parse(endpoint) {
        Ok(u) => u,
        Err(_) => return false,
    };
    let host = parsed.host_str().unwrap_or("");
    matches!(host, "localhost" | "127.0.0.1" | "::1" | "[::1]" | "0.0.0.0")
}

/// Normalize an endpoint down to its origin (scheme://host:port). Two endpoints
/// with the same origin but different path/query are treated as the same server
/// — used by [`update`] to decide whether a changed endpoint should invalidate
/// the parallel consent. Parse failures fall back to the raw input (so a
/// garbage endpoint at least compares unequal to a good one).
pub fn normalize_origin(endpoint: &str) -> String {
    match url::Url::parse(endpoint) {
        Ok(u) => u.origin().ascii_serialization(),
        Err(_) => endpoint.to_string(),
    }
}

// ─── Read paths ───────────────────────────────────────────────────────────

/// Active providers only (`status='active'`). This is the user-facing list:
/// a provider that has entered two-phase delete (`status='deleting'`) or been
/// tombstoned (`status='deleted'`) disappears from here immediately. Ordered by
/// `sort_order` then `name` for a stable UI.
pub fn list(conn: &Connection) -> Result<Vec<ProviderProfile>, DbError> {
    let mut stmt = conn.prepare(&format!(
        "SELECT {SELECT_COLS} FROM providers WHERE status = 'active' \
         ORDER BY sort_order ASC, name ASC"
    ))?;
    let rows = stmt.query_map([], row_to_profile)?;
    let mut out = Vec::new();
    for r in rows {
        out.push(r?);
    }
    Ok(out)
}

/// All providers including `deleting`/`deleted` tombstones. Ordered the same as
/// [`list`] so the two views differ only by the status filter.
pub fn list_all(conn: &Connection) -> Result<Vec<ProviderProfile>, DbError> {
    let mut stmt = conn.prepare(&format!(
        "SELECT {SELECT_COLS} FROM providers ORDER BY sort_order ASC, name ASC"
    ))?;
    let rows = stmt.query_map([], row_to_profile)?;
    let mut out = Vec::new();
    for r in rows {
        out.push(r?);
    }
    Ok(out)
}

/// Fetch a single row by UUID. `NotFound` if absent.
pub fn get(conn: &Connection, uuid: &str) -> Result<ProviderProfile, DbError> {
    conn.query_row(
        &format!("SELECT {SELECT_COLS} FROM providers WHERE uuid = ?1"),
        params![uuid],
        row_to_profile,
    )
    .map_err(|e| match e {
        rusqlite::Error::QueryReturnedNoRows => {
            DbError::NotFound(format!("provider uuid={uuid}"))
        }
        other => DbError::Sqlite(other),
    })
}

// ─── Write paths ──────────────────────────────────────────────────────────

/// Insert a row, ignoring a pre-existing UUID (idempotent). Used by migration
/// so a crash-replay never fails on the second pass.
pub fn insert_or_ignore(
    conn: &Connection,
    profile: &ProviderProfile,
) -> Result<(), DbError> {
    let caps_json = caps_to_json(&profile.capabilities);
    let enabled: i64 = profile.enabled.into();
    let is_local: i64 = profile.is_local.into();
    let needs_key: i64 = profile.needs_key.into();
    conn.execute(
        "INSERT OR IGNORE INTO providers \
         (uuid, template_id, name, protocol, endpoint, model, enabled, sort_order, \
          is_local, needs_key, secret_ref, capabilities, status) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
        params![
            profile.uuid,
            profile.template_id,
            profile.name,
            profile.protocol.as_db_str(),
            profile.endpoint,
            profile.model,
            enabled,
            profile.sort_order,
            is_local,
            needs_key,
            profile.secret_ref,
            caps_json,
            profile.status,
        ],
    )?;
    Ok(())
}

/// Resolve a `template_id` to its preset-derived creation parameters
/// (protocol/endpoint/default model/needs_key), or `None` if it isn't a preset.
fn preset_lookup(template_id: &str) -> Option<PresetDerived> {
    crate::providers::presets()
        .into_iter()
        .find(|p| p.id == template_id)
        .map(|p| PresetDerived {
            protocol: preset_protocol(&p.id),
            endpoint: p.endpoint,
            default_model: Some(p.default_model),
            needs_key: p.needs_key,
        })
}

struct PresetDerived {
    protocol: Protocol,
    endpoint: String,
    default_model: Option<String>,
    needs_key: bool,
}

/// Map a preset id to its wire protocol. OpenAI-compatible presets (openai,
/// gemini, ollama) share `OpenaiChat`; anthropic → `Anthropic`. An unknown id
/// falls back to `CustomHttp` (the repair protocol).
fn preset_protocol(preset_id: &str) -> Protocol {
    match preset_id {
        "anthropic" => Protocol::Anthropic,
        "openai" | "gemini" | "ollama" => Protocol::OpenaiChat,
        _ => Protocol::CustomHttp,
    }
}

/// Create a new provider row.
///
/// UUID is a fresh v4; `secret_ref` is `provider/<uuid>`. `needs_key`,
/// `protocol`, `endpoint`, and `model` come from the preset catalog when the
/// `template_id` matches a preset; otherwise the row is created with
/// `CustomHttp`, an empty endpoint, and `needs_key=true` (a repair-shaped row
/// the user must fill in). `enabled=true`, `status="active"`. `sort_order` is
/// assigned to one past the current max so the new row lands at the end of the
/// list.
pub fn create(
    conn: &mut Connection,
    template_id: &str,
    name: &str,
    endpoint: &str,
    model: Option<&str>,
) -> Result<ProviderProfile, DbError> {
    let uuid = uuid_util::new_provider_uuid();
    let uuid_str = uuid.to_string();
    let secret_ref = format!("provider/{uuid_str}");

    // Derive protocol/needs_key/default-model from the preset if we recognise it.
    let (protocol, final_endpoint, final_model, needs_key) =
        match preset_lookup(template_id) {
            Some(d) => {
                // Caller-supplied endpoint/model win over preset defaults.
                let ep = if endpoint.is_empty() { d.endpoint } else { endpoint.to_string() };
                let md = model.map(String::from).or(d.default_model);
                (d.protocol, ep, md, d.needs_key)
            }
            None => {
                // Unknown template → repair-shaped row. Validate the caller endpoint
                // (if any); empty endpoint is allowed (the row is disabled-by-default
                // in spirit, but `enabled=true` is the documented contract).
                if !endpoint.is_empty() {
                    crate::providers::validate_endpoint(endpoint)
                        .map_err(DbError::Integrity)?;
                }
                (
                    Protocol::CustomHttp,
                    endpoint.to_string(),
                    model.map(String::from),
                    true,
                )
            }
        };

    let tx = conn.transaction()?;
    let sort_order: i32 = next_sort_order(&tx)?;
    let is_local = endpoint_is_local(&final_endpoint);
    let profile = ProviderProfile {
        uuid: uuid_str,
        template_id: template_id.to_string(),
        name: name.to_string(),
        protocol,
        endpoint: final_endpoint,
        model: final_model,
        enabled: true,
        sort_order,
        is_local,
        needs_key,
        secret_ref,
        capabilities: ProviderCapabilities::default(),
        status: ProviderStatus::Active.as_str().to_string(),
    };
    insert_or_ignore(&tx, &profile)?;
    tx.commit()?;
    Ok(profile)
}

/// One past the current max `sort_order` across ALL rows (including deleted
/// tombstones), so a freshly created row never collides with a tombstoned one.
/// `COALESCE` maps the empty-table NULL from `MAX` to -1, so the first row
/// gets sort_order 0.
fn next_sort_order(conn: &Connection) -> Result<i32, DbError> {
    let max: i64 = conn.query_row(
        "SELECT COALESCE(MAX(sort_order), -1) FROM providers",
        [],
        |r| r.get(0),
    )?;
    Ok((max + 1) as i32)
}

/// Apply a partial patch. Endpoint (if `Some`) is validated via
/// [`crate::providers::validate_endpoint`]; `is_local` is recomputed from the
/// resulting endpoint. Returns the updated row.
pub fn update(
    conn: &mut Connection,
    uuid: &str,
    patch: &ProviderPatch,
) -> Result<ProviderProfile, DbError> {
    if let Some(ep) = &patch.endpoint {
        crate::providers::validate_endpoint(ep).map_err(DbError::Integrity)?;
    }

    let tx = conn.transaction()?;
    // Read-modify-write inside the tx so the patch composes atomically.
    let existing = get(&tx, uuid)?;

    // Capture the old origin BEFORE we move existing.endpoint below — the
    // consent check compares origins, not the full URL.
    let old_origin = normalize_origin(&existing.endpoint);

    let name = patch.name.clone().unwrap_or(existing.name);
    let endpoint = patch.endpoint.clone().unwrap_or(existing.endpoint);
    let model = match &patch.model {
        Some(m) => Some(m.clone()),
        None => existing.model,
    };
    let enabled = patch.enabled.unwrap_or(existing.enabled);
    let sort_order = patch.sort_order.unwrap_or(existing.sort_order);
    let is_local = endpoint_is_local(&endpoint);

    // If the endpoint's ORIGIN changed (scheme/host/port — not just path/query)
    // AND this provider is in the primary or any parallel slot, the prior
    // parallel consent was given for a different upstream and must be dropped.
    let origin_changed = patch.endpoint.as_deref().is_some_and(|new_ep| {
        normalize_origin(new_ep) != old_origin
    });
    if origin_changed && provider_in_primary_or_parallel(&tx, uuid)? {
        invalidate_consent(&tx)?;
    }

    tx.execute(
        "UPDATE providers SET name=?1, endpoint=?2, model=?3, enabled=?4, \
         sort_order=?5, is_local=?6 WHERE uuid=?7",
        params![name, endpoint, model, enabled as i64, sort_order, is_local as i64, uuid],
    )?;

    let updated = get(&tx, uuid)?;
    tx.commit()?;
    Ok(updated)
}

/// Duplicate a provider. New UUIDv4, new `secret_ref`, `enabled=true`,
/// `needs_key=true` (the original key is NEVER copied — the duplicate starts
/// keyless and the user must enter one). Name gets a " (copy)" suffix so the two
/// rows are distinguishable in the UI.
pub fn duplicate(
    conn: &mut Connection,
    uuid: &str,
) -> Result<ProviderProfile, DbError> {
    let new_uuid = uuid_util::new_provider_uuid();
    let new_uuid_str = new_uuid.to_string();
    let new_secret_ref = format!("provider/{new_uuid_str}");

    let tx = conn.transaction()?;
    let src = get(&tx, uuid)?;
    let sort_order = next_sort_order(&tx)?;
    let profile = ProviderProfile {
        uuid: new_uuid_str,
        template_id: src.template_id,
        name: format!("{} (copy)", src.name),
        protocol: src.protocol,
        endpoint: src.endpoint,
        model: src.model,
        enabled: true,
        sort_order,
        is_local: src.is_local,
        needs_key: true, // duplicate starts keyless
        secret_ref: new_secret_ref,
        capabilities: src.capabilities,
        status: ProviderStatus::Active.as_str().to_string(),
    };
    insert_or_ignore(&tx, &profile)?;
    tx.commit()?;
    Ok(profile)
}

/// Re-assign `sort_order` to the given UUID order. The list MUST be exactly the
/// set of active UUIDs: no duplicates, no missing, no extras. Each UUID gets
/// `sort_order = its index`.
pub fn reorder(conn: &mut Connection, uuids: &[String]) -> Result<(), DbError> {
    // Validate: no duplicates in the input.
    let mut seen = HashSet::with_capacity(uuids.len());
    for u in uuids {
        if !seen.insert(u.as_str()) {
            return Err(DbError::Integrity(format!(
                "reorder: duplicate uuid {u}"
            )));
        }
    }

    let tx = conn.transaction()?;
    // Active set = status == "active" (tombstones are never reorderable).
    let active: Vec<String> = {
        let mut stmt = tx.prepare("SELECT uuid FROM providers WHERE status='active'")?;
        let rows = stmt.query_map([], |r| r.get::<_, String>(0))?;
        let mut v = Vec::new();
        for r in rows {
            v.push(r?);
        }
        v
    };
    let active_set: HashSet<&str> = active.iter().map(String::as_str).collect();
    let input_set: HashSet<&str> = uuids.iter().map(String::as_str).collect();
    if active_set != input_set {
        return Err(DbError::Integrity(format!(
            "reorder: input set does not match active providers ({} active, {} given)",
            active_set.len(),
            input_set.len()
        )));
    }

    for (i, u) in uuids.iter().enumerate() {
        tx.execute(
            "UPDATE providers SET sort_order=?1 WHERE uuid=?2",
            params![i as i64, u],
        )?;
    }
    tx.commit()?;
    Ok(())
}

/// Flip `enabled`. When disabling, the row is also removed from every active
/// selection slot in `preferences` (primary/parallel/fallback) within the SAME
/// transaction, so a disabled provider can never remain "selected".
pub fn toggle(
    conn: &mut Connection,
    uuid: &str,
    enabled: bool,
) -> Result<ProviderProfile, DbError> {
    let tx = conn.transaction()?;
    tx.execute(
        "UPDATE providers SET enabled=?1 WHERE uuid=?2",
        params![enabled as i64, uuid],
    )?;
    if !enabled {
        // Disable: pull the row out of every selection slot and drop consent,
        // mirroring begin_delete. A disabled provider can't stay "selected",
        // and the prior parallel consent was given for a set that still
        // included it.
        remove_from_active_slots(&tx, uuid)?;
        invalidate_consent(&tx)?;
    }
    let updated = get(&tx, uuid)?;
    tx.commit()?;
    Ok(updated)
}

/// Begin deletion: mark the row `deleting` + disabled, pull it out of every
/// active selection slot, and return the `secret_ref` so the caller can purge
/// the key from the keystore in a separate keystore-locked step.
///
/// Does NOT touch the keystore itself (lock-order: never hold the DB mutex and
/// the keystore flock at once).
pub fn begin_delete(conn: &mut Connection, uuid: &str) -> Result<String, DbError> {
    let tx = conn.transaction()?;
    let profile = get(&tx, uuid)?;
    tx.execute(
        "UPDATE providers SET status='deleting', enabled=0 WHERE uuid=?1",
        params![uuid],
    )?;
    remove_from_active_slots(&tx, uuid)?;
    // Parallel-consent version is tied to the parallel set; dropping a member
    // invalidates the prior consent.
    invalidate_consent(&tx)?;
    tx.commit()?;
    Ok(profile.secret_ref)
}

/// Finalize deletion: mark the row `deleted` and rewrite the name to
/// `deleted: <orig>` so the tombstone is identifiable in audits / list_all.
pub fn finalize_delete(conn: &mut Connection, uuid: &str) -> Result<(), DbError> {
    let tx = conn.transaction()?;
    let profile = get(&tx, uuid)?;
    let tombstone_name = format!("deleted: {}", profile.name);
    tx.execute(
        "UPDATE providers SET status='deleted', name=?1 WHERE uuid=?2",
        params![tombstone_name, uuid],
    )?;
    tx.commit()?;
    Ok(())
}

// ─── preferences active-slot helpers ──────────────────────────────────────

/// Remove `uuid` from `primary_uuid`, every entry of the `parallel_uuids`
/// JSON array, and `fallback_uuid`. Idempotent — a no-op if the uuid isn't in a
/// slot. Caller must be in a transaction.
fn remove_from_active_slots(conn: &Connection, uuid: &str) -> Result<(), DbError> {
    // primary
    conn.execute(
        "UPDATE preferences SET primary_uuid=NULL WHERE primary_uuid=?1",
        params![uuid],
    )?;
    // fallback
    conn.execute(
        "UPDATE preferences SET fallback_uuid=NULL WHERE fallback_uuid=?1",
        params![uuid],
    )?;
    // parallel: read JSON array, filter, write back. We hand-parse via serde_json
    // so the stored shape is normalized (no trailing whitespace drift).
    let parallel_json: String = conn.query_row(
        "SELECT parallel_uuids FROM preferences WHERE id=1",
        [],
        |r| r.get(0),
    )?;
    let arr: Vec<String> = serde_json::from_str::<Vec<String>>(&parallel_json)
        .unwrap_or_default()
        .into_iter()
        .filter(|u| u != uuid)
        .collect();
    let new_json = serde_json::to_string(&arr).unwrap_or_else(|_| "[]".to_string());
    conn.execute(
        "UPDATE preferences SET parallel_uuids=?1 WHERE id=1",
        params![new_json],
    )?;
    Ok(())
}

/// Null out the parallel-consent version/scope. Called whenever the parallel
/// set changes membership (delete, disable-in-slot) or a member's endpoint
/// origin changes — the prior consent was given for a different provider set,
/// so the next translate must re-prompt. Caller must be in a transaction.
fn invalidate_consent(tx: &rusqlite::Transaction<'_>) -> Result<(), DbError> {
    tx.execute(
        "UPDATE preferences SET parallel_consent_version=NULL, \
         parallel_consent_scope=NULL WHERE id=1",
        [],
    )?;
    Ok(())
}

/// Is `uuid` currently the `primary_uuid` or a member of `parallel_uuids`?
/// (Fallback slot is excluded: a traditional-engine fallback never carries AI
/// consent.) Used by [`update`] to decide whether an endpoint-origin change
/// should invalidate consent.
fn provider_in_primary_or_parallel(
    tx: &rusqlite::Transaction<'_>,
    uuid: &str,
) -> Result<bool, DbError> {
    let primary: Option<String> = tx
        .query_row(
            "SELECT primary_uuid FROM preferences WHERE id=1",
            [],
            |r| r.get(0),
        )
        .unwrap_or(None);
    if primary.as_deref() == Some(uuid) {
        return Ok(true);
    }
    let parallel_json: String = tx.query_row(
        "SELECT parallel_uuids FROM preferences WHERE id=1",
        [],
        |r| r.get(0),
    )?;
    let arr: Vec<String> = serde_json::from_str::<Vec<String>>(&parallel_json)
        .unwrap_or_default();
    Ok(arr.iter().any(|u| u == uuid))
}

// ─── TraditionalProviderCatalog ───────────────────────────────────────────

/// Catalog entry for a traditional (non-AI, keyless) translation engine. These
/// are the built-in MT providers used as AI-failure fallback (spec §G) and as
/// the always-available `fallback_uuid` slot.
#[derive(Debug, Clone, Copy)]
pub struct TraditionalProviderCatalog {
    pub template_id: &'static str,
    pub label: &'static str,
    pub endpoint: &'static str,
    /// Always `false` for traditional engines — they're free and keyless.
    pub needs_key: bool,
}

/// The traditional-engine catalog. One row per built-in MT provider. Adding a
/// traditional engine = one struct literal here.
pub fn traditional_catalog() -> &'static [TraditionalProviderCatalog] {
    static CATALOG: &[TraditionalProviderCatalog] = &[TraditionalProviderCatalog {
        template_id: "google",
        label: "Google Translate",
        endpoint: "https://translate.google.com",
        needs_key: false,
    }];
    CATALOG
}

/// Lookup helper: find a traditional catalog entry by `template_id`.
fn traditional_lookup(template_id: &str) -> Option<&'static TraditionalProviderCatalog> {
    traditional_catalog().iter().find(|c| c.template_id == template_id)
}

/// Template-ids allowed in the `fallback_uuid` slot (spec §G: fallback must be a
/// traditional engine — never a second remote AI provider).
pub const TRADITIONAL_TEMPLATES: &[&str] =
    &["google", "deepl", "microsoft", "baidu", "youdao", "tencent"];

/// Is `template_id` a permitted fallback-engine template?
fn is_traditional_template(template_id: &str) -> bool {
    TRADITIONAL_TEMPLATES.contains(&template_id)
}

// ─── validate_active_selection ────────────────────────────────────────────

/// Validate the (primary, parallel, fallback) selection against the provider set.
///
/// Rules:
/// 1. No UUID appears in two roles (primary/parallel/fallback must be disjoint).
/// 2. Every referenced UUID is `status="active"` AND `enabled=true`.
/// 3. If `fallback` is `Some`, its provider's `template_id` must be in
///    [`TRADITIONAL_TEMPLATES`] (fallback is always a traditional engine).
///
/// Empty primary (`""`) and empty parallel list are allowed (no selection).
pub fn validate_active_selection(
    primary: &str,
    parallel: &[String],
    fallback: Option<&str>,
    providers: &[ProviderProfile],
) -> Result<(), DbError> {
    // Build a lookup of active+enabled providers.
    let active: HashMap<&str, &ProviderProfile> = providers
        .iter()
        .filter(|p| p.status == ProviderStatus::Active.as_str() && p.enabled)
        .map(|p| (p.uuid.as_str(), p))
        .collect();

    // Collect referenced uuids (skip empties). Owns String keys so the closure
    // doesn't borrow caller references with mismatched lifetimes.
    let mut seen: HashMap<String, &'static str> = HashMap::new();
    let mut check = |uuid: &str, role: &'static str| -> Result<(), DbError> {
        if uuid.is_empty() {
            return Ok(());
        }
        if let Some(prev_role) = seen.insert(uuid.to_string(), role) {
            return Err(DbError::Integrity(format!(
                "uuid {uuid} appears in both {prev_role} and {role}"
            )));
        }
        if !active.contains_key(uuid) {
            return Err(DbError::Integrity(format!(
                "uuid {uuid} referenced as {role} is not active+enabled"
            )));
        }
        Ok(())
    };

    check(primary, "primary")?;
    for u in parallel {
        check(u, "parallel")?;
    }
    if let Some(fb) = fallback {
        check(fb, "fallback")?;
        // Fallback must be a traditional engine.
        if let Some(p) = active.get(fb) {
            if !is_traditional_template(&p.template_id) {
                return Err(DbError::Integrity(format!(
                    "fallback uuid {fb} template_id '{}' is not a traditional engine",
                    p.template_id
                )));
            }
        }
    }
    Ok(())
}

// ─── CandidateSource (rev-6 amendment) ────────────────────────────────────

/// A migration / DB-loss-recovery candidate: where a provider row should come
/// from. The two arms drive both the stable display id and the deterministic
/// UUID (so a crash-replay produces the SAME rows).
///
/// - `LegacyId(id)`    — a pre-2a provider id (`"openai"`, `"google"`, …).
/// - `ProviderKey(sr)` — a keystore key (`"provider/<uuid>"` or a legacy key).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CandidateSource {
    LegacyId(String),
    ProviderKey(String),
}

impl CandidateSource {
    /// A stable, human-readable identifier for logging/UI.
    pub fn stable_id(&self) -> &str {
        match self {
            CandidateSource::LegacyId(id) => id,
            CandidateSource::ProviderKey(sr) => sr,
        }
    }

    /// Deterministic UUID for this candidate. Same input → same UUID, so a
    /// crash during migration can be replayed idempotently.
    ///
    /// - `LegacyId(id)`    → `UUIDv5(NS, "linguaray:legacy-provider:" + id)`
    /// - `ProviderKey(sr)` → `UUIDv5(NS, "linguaray:recovered-key:" + sr)`
    pub fn deterministic_uuid(&self) -> uuid::Uuid {
        match self {
            CandidateSource::LegacyId(id) => uuid_util::legacy_provider_uuid(id),
            CandidateSource::ProviderKey(sr) => uuid_util::recovered_key_uuid(sr),
        }
    }
}

/// Raw, pre-2a settings (read from `tauri-plugin-store` by the migration
/// coordinator). All fields optional because a missing settings file means
/// "no defaults"; the migration still runs from keystore keys alone.
#[derive(Debug, Clone, Default)]
pub struct RawSettings {
    pub default_provider: Option<String>,
    pub target_language: Option<String>,
    pub fallback_engine: Option<String>,
}

/// Enumerate every migration/recovery candidate, in deterministic order.
///
/// Sources:
/// 1. **Keystore keys** — both `LegacyV1` (flat map) and `CurrentV2`
///    (`provider_keys`). A key starting with `provider/` → `ProviderKey`;
///    anything else → `LegacyId` (pre-2a keys were bare provider ids like
///    `"openai"`).
/// 2. **Settings defaults** — `default_provider` is ALWAYS considered as a
///    candidate (even if the keystore is `Missing`), so a fresh-install
///    migration still seeds the default provider row. `fallback_engine`, if
///    present, is also considered.
///
/// Deduplicated via `BTreeSet` so the order is stable across runs (critical for
/// idempotent migration: same candidate list → same rows).
pub fn enumerate_candidates(
    ks: &crate::keystore::KeystoreLoadState,
    settings: Option<&RawSettings>,
) -> Vec<CandidateSource> {
    let mut set: BTreeSet<(bool, String)> = BTreeSet::new();

    // Helper: classify one keystore key.
    let classify = |key: &str| -> (bool, String) {
        // (is_provider_key, payload). ProviderKey sorts after LegacyId by payload,
        // which is fine — the tuple only needs to be a stable dedup key.
        let is_pk = key.starts_with("provider/");
        (is_pk, key.to_string())
    };

    // Collect keys from the keystore (both v1 and v2 shapes).
    let keystore_keys: Vec<String> = match ks {
        crate::keystore::KeystoreLoadState::LegacyV1(map) => {
            map.keys().cloned().collect()
        }
        crate::keystore::KeystoreLoadState::CurrentV2(data) => {
            data.provider_keys.keys().cloned().collect()
        }
        // Missing / Corrupt contribute no keys.
        _ => Vec::new(),
    };
    for k in keystore_keys {
        set.insert(classify(&k));
    }

    // Settings defaults are always candidates (even if keystore Missing).
    if let Some(s) = settings {
        if let Some(dp) = &s.default_provider {
            // default_provider is a legacy id (preset id or traditional id).
            set.insert((false, dp.clone()));
        }
        if let Some(fb) = &s.fallback_engine {
            set.insert((false, fb.clone()));
        }
    }

    // Materialize in BTreeSet order (deterministic).
    set.into_iter()
        .map(|(is_pk, payload)| {
            if is_pk {
                CandidateSource::ProviderKey(payload)
            } else {
                CandidateSource::LegacyId(payload)
            }
        })
        .collect()
}

/// Build a `ProviderProfile` (NOT yet inserted) from a candidate source.
///
/// Dispatch:
/// - `LegacyId(id)`:
///   - matches a preset → profile from the preset (protocol/endpoint/model).
///   - matches the traditional catalog → profile from the catalog (google_translate).
///   - otherwise → repair profile (`custom_http`, empty endpoint, disabled,
///     `needs_key=true`).
/// - `ProviderKey(sr)`:
///   - `"provider/<uuid>"` parseable → keep the embedded UUID.
///   - unparseable → `UUIDv5` from the secret_ref (recovered_key_uuid).
///   - The resulting profile is a generic `custom_http` shell with
///     `needs_key=true`; the migration coordinator fills name/endpoint later.
///
/// The profile is returned with `status="active"`, `sort_order=0`, and
/// `enabled=true` for preset/traditional lookups; repair profiles are
/// `enabled=false`, `sort_order=999` (parked at the bottom of the list), and
/// `template_id="unknown"` for `ProviderKey` arms (the user must fix them
/// before use).
pub fn build_profile(source: &CandidateSource) -> Result<ProviderProfile, DbError> {
    match source {
        CandidateSource::LegacyId(id) => {
            // 1. Preset match?
            if let Some(d) = preset_lookup(id) {
                let uuid = source.deterministic_uuid().to_string();
                let secret_ref = if d.needs_key {
                    // Legacy preset that needed a key → it was keyed by the bare id.
                    id.clone()
                } else {
                    // Keyless preset (ollama) → no secret to reference.
                    format!("provider/{uuid}")
                };
                return Ok(ProviderProfile {
                    uuid,
                    template_id: id.clone(),
                    name: preset_label(id).unwrap_or_else(|| id.clone()),
                    protocol: d.protocol,
                    endpoint: d.endpoint,
                    model: d.default_model,
                    enabled: true,
                    sort_order: 0,
                    is_local: false,
                    needs_key: d.needs_key,
                    secret_ref,
                    capabilities: ProviderCapabilities::default(),
                    status: ProviderStatus::Active.as_str().to_string(),
                });
            }
            // 2. Traditional catalog match?
            if let Some(c) = traditional_lookup(id) {
                let uuid = source.deterministic_uuid().to_string();
                let secret_ref = format!("provider/{uuid}");
                return Ok(ProviderProfile {
                    uuid,
                    template_id: c.template_id.to_string(),
                    name: c.label.to_string(),
                    protocol: Protocol::GoogleTranslate,
                    endpoint: c.endpoint.to_string(),
                    model: None,
                    enabled: true,
                    sort_order: 0,
                    is_local: false,
                    needs_key: c.needs_key,
                    secret_ref,
                    capabilities: ProviderCapabilities::default(),
                    status: ProviderStatus::Active.as_str().to_string(),
                });
            }
            // 3. Unknown → repair profile.
            let uuid = source.deterministic_uuid().to_string();
            Ok(ProviderProfile {
                uuid,
                template_id: id.clone(),
                name: id.clone(),
                protocol: Protocol::CustomHttp,
                endpoint: String::new(),
                model: None,
                enabled: false,
                sort_order: 999,
                is_local: false,
                needs_key: true,
                secret_ref: id.clone(),
                capabilities: ProviderCapabilities::default(),
                status: ProviderStatus::Active.as_str().to_string(),
            })
        }
        CandidateSource::ProviderKey(sr) => {
            // Parse "provider/<uuid>"; if it parses, keep the UUID. Otherwise derive
            // a deterministic UUID from the whole secret_ref.
            let uuid = parse_provider_uuid(sr)
                .map(|u| u.to_string())
                .unwrap_or_else(|| source.deterministic_uuid().to_string());
            Ok(ProviderProfile {
                uuid,
                template_id: "unknown".to_string(),
                name: "Recovered provider".to_string(),
                protocol: Protocol::CustomHttp,
                endpoint: String::new(),
                model: None,
                enabled: false,
                sort_order: 999,
                is_local: false,
                needs_key: true,
                secret_ref: sr.clone(),
                capabilities: ProviderCapabilities::default(),
                status: ProviderStatus::Active.as_str().to_string(),
            })
        }
    }
}

/// If `sr` is exactly `provider/<uuid>`, return the parsed UUID. Used by
/// [`build_profile`] to preserve the original UUID when a recovered key already
/// carries one.
fn parse_provider_uuid(sr: &str) -> Option<uuid::Uuid> {
    let rest = sr.strip_prefix("provider/")?;
    uuid::Uuid::parse_str(rest).ok()
}

/// Human label for a preset id, for migration-built rows. Mirrors the preset
/// catalog labels so a migrated "openai" row is named "OpenAI", not "openai".
fn preset_label(preset_id: &str) -> Option<String> {
    crate::providers::presets()
        .into_iter()
        .find(|p| p.id == preset_id)
        .map(|p| p.label)
}

// ─── tests (in-module, for the pure helpers) ─────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn protocol_db_round_trip() {
        for p in [
            Protocol::OpenaiChat,
            Protocol::Anthropic,
            Protocol::Gemini,
            Protocol::GoogleTranslate,
            Protocol::CustomHttp,
        ] {
            assert_eq!(Protocol::from_db_str(p.as_db_str()).unwrap(), p);
        }
    }

    #[test]
    fn candidate_deterministic_uuid_is_stable() {
        // Golden vector: same input MUST produce the same UUID across runs.
        let a = CandidateSource::LegacyId("openai".into()).deterministic_uuid();
        let b = CandidateSource::LegacyId("openai".into()).deterministic_uuid();
        assert_eq!(a, b);

        let c = CandidateSource::ProviderKey("provider/abc".into()).deterministic_uuid();
        let d = CandidateSource::ProviderKey("provider/abc".into()).deterministic_uuid();
        assert_eq!(c, d);

        // Different arms produce different UUIDs.
        assert_ne!(a, c);
    }

    #[test]
    fn candidate_legacy_uuid_matches_uuid_util() {
        // The CandidateSource helper must agree with the raw uuid_util function
        // (migration coordinator may use either).
        let cs = CandidateSource::LegacyId("google".into());
        assert_eq!(cs.deterministic_uuid(), uuid_util::legacy_provider_uuid("google"));
    }

    #[test]
    fn candidate_providerkey_uuid_matches_uuid_util() {
        let cs = CandidateSource::ProviderKey("provider/x".into());
        assert_eq!(
            cs.deterministic_uuid(),
            uuid_util::recovered_key_uuid("provider/x")
        );
    }

    #[test]
    fn parse_provider_uuid_ok() {
        let u = uuid_util::new_provider_uuid();
        let sr = format!("provider/{u}");
        assert_eq!(parse_provider_uuid(&sr), Some(u));
    }

    #[test]
    fn parse_provider_uuid_rejects_non_uuid_suffix() {
        assert_eq!(parse_provider_uuid("provider/not-a-uuid"), None);
        assert_eq!(parse_provider_uuid("provider/"), None);
    }

    #[test]
    fn parse_provider_uuid_rejects_missing_prefix() {
        let u = uuid::Uuid::new_v4();
        assert_eq!(parse_provider_uuid(&u.to_string()), None);
    }

    #[test]
    fn enumerate_candidates_orders_deterministically() {
        // Two different keystore maps with the same keys must produce the same
        // candidate list (BTreeSet ordering).
        let mut m1 = HashMap::new();
        m1.insert("openai".to_string(), "k".to_string());
        m1.insert("provider/u1".to_string(), "k".to_string());
        let mut m2 = HashMap::new();
        m2.insert("provider/u1".to_string(), "k".to_string());
        m2.insert("openai".to_string(), "k".to_string());

        let s1 = enumerate_candidates(
            &crate::keystore::KeystoreLoadState::LegacyV1(m1),
            None,
        );
        let s2 = enumerate_candidates(
            &crate::keystore::KeystoreLoadState::LegacyV1(m2),
            None,
        );
        assert_eq!(s1, s2);
        // LegacyId("openai") sorts before ProviderKey("provider/u1").
        assert!(matches!(s1[0], CandidateSource::LegacyId(_)));
        assert!(matches!(s1[1], CandidateSource::ProviderKey(_)));
    }

    #[test]
    fn enumerate_candidates_always_includes_settings_defaults() {
        // Even with a Missing keystore, the settings default must appear.
        let settings = RawSettings {
            default_provider: Some("openai".to_string()),
            fallback_engine: Some("google".to_string()),
            ..Default::default()
        };
        let v = enumerate_candidates(
            &crate::keystore::KeystoreLoadState::Missing,
            Some(&settings),
        );
        assert!(v.iter().any(|c| matches!(
            c,
            CandidateSource::LegacyId(id) if id == "openai"
        )));
        assert!(v.iter().any(|c| matches!(
            c,
            CandidateSource::LegacyId(id) if id == "google"
        )));
    }

    #[test]
    fn enumerate_candidates_dedupes_overlapping_sources() {
        // Same id from keystore AND settings → one candidate.
        let mut m = HashMap::new();
        m.insert("openai".to_string(), "k".to_string());
        let settings = RawSettings {
            default_provider: Some("openai".to_string()),
            ..Default::default()
        };
        let v = enumerate_candidates(
            &crate::keystore::KeystoreLoadState::LegacyV1(m),
            Some(&settings),
        );
        let count = v
            .iter()
            .filter(|c| matches!(c, CandidateSource::LegacyId(id) if id == "openai"))
            .count();
        assert_eq!(count, 1);
    }

    #[test]
    fn build_profile_preset_legacy() {
        let cs = CandidateSource::LegacyId("openai".into());
        let p = build_profile(&cs).unwrap();
        assert_eq!(p.template_id, "openai");
        assert_eq!(p.protocol, Protocol::OpenaiChat);
        assert!(p.needs_key);
        assert!(p.enabled);
        // Legacy preset secret_ref is the bare id (pre-2a key shape).
        assert_eq!(p.secret_ref, "openai");
        assert_eq!(p.uuid, cs.deterministic_uuid().to_string());
    }

    #[test]
    fn build_profile_traditional_legacy() {
        let cs = CandidateSource::LegacyId("google".into());
        let p = build_profile(&cs).unwrap();
        assert_eq!(p.template_id, "google");
        assert_eq!(p.protocol, Protocol::GoogleTranslate);
        assert!(!p.needs_key);
        assert!(p.enabled);
        assert_eq!(p.endpoint, "https://translate.google.com");
    }

    #[test]
    fn build_profile_unknown_legacy_is_repair() {
        let cs = CandidateSource::LegacyId("mystery-provider".into());
        let p = build_profile(&cs).unwrap();
        assert_eq!(p.protocol, Protocol::CustomHttp);
        assert!(p.endpoint.is_empty());
        assert!(!p.enabled);
        assert!(p.needs_key);
        assert_eq!(p.template_id, "mystery-provider");
        // Repair profiles park at the bottom of the list.
        assert_eq!(p.sort_order, 999);
    }

    #[test]
    fn build_profile_providerkey_parseable_keeps_uuid() {
        let u = uuid::Uuid::new_v4();
        let sr = format!("provider/{u}");
        let cs = CandidateSource::ProviderKey(sr.clone());
        let p = build_profile(&cs).unwrap();
        assert_eq!(p.uuid, u.to_string());
        assert_eq!(p.secret_ref, sr);
        assert_eq!(p.protocol, Protocol::CustomHttp);
        assert!(p.needs_key);
        // ProviderKey repair rows get the unknown template + bottom sort_order.
        assert_eq!(p.template_id, "unknown");
        assert_eq!(p.sort_order, 999);
    }

    #[test]
    fn build_profile_providerkey_unparseable_derives_uuid() {
        let sr = "provider/not-a-uuid".to_string();
        let cs = CandidateSource::ProviderKey(sr.clone());
        let p = build_profile(&cs).unwrap();
        // Derived UUID must match the recovered_key_uuid for this secret_ref.
        assert_eq!(
            p.uuid,
            uuid_util::recovered_key_uuid(&sr).to_string()
        );
        assert_eq!(p.template_id, "unknown");
        assert_eq!(p.sort_order, 999);
    }

    #[test]
    fn endpoint_is_local_classifies_loopback() {
        assert!(endpoint_is_local("http://localhost:11434/v1/chat"));
        assert!(endpoint_is_local("http://127.0.0.1:8080"));
        assert!(endpoint_is_local("http://[::1]:8080"));
        assert!(endpoint_is_local("http://0.0.0.0:8080"));
        assert!(!endpoint_is_local("https://api.openai.com/v1/chat"));
        assert!(!endpoint_is_local("not a url"));
    }

    #[test]
    fn normalize_origin_drops_path_and_query() {
        // Same host, different path/query → same origin.
        assert_eq!(
            normalize_origin("https://api.openai.com/v1/chat/completions"),
            normalize_origin("https://api.openai.com/v1/messages?x=1")
        );
        // Different host → different origin.
        assert_ne!(
            normalize_origin("https://api.openai.com"),
            normalize_origin("https://api.anthropic.com")
        );
        // Port is part of the origin.
        assert_ne!(
            normalize_origin("http://localhost:11434"),
            normalize_origin("http://localhost:8080")
        );
        // Scheme is part of the origin.
        assert_ne!(
            normalize_origin("http://api.openai.com"),
            normalize_origin("https://api.openai.com")
        );
    }

    #[test]
    fn validate_active_selection_rejects_overlap() {
        let p = ProviderProfile {
            uuid: "u1".into(),
            template_id: "openai".into(),
            name: "OpenAI".into(),
            protocol: Protocol::OpenaiChat,
            endpoint: "https://api.openai.com".into(),
            model: None,
            enabled: true,
            sort_order: 0,
            is_local: false,
            needs_key: true,
            secret_ref: "openai".into(),
            capabilities: ProviderCapabilities::default(),
            status: "active".into(),
        };
        let providers = vec![p];
        let err = validate_active_selection("u1", &["u1".into()], None, &providers).unwrap_err();
        assert!(matches!(err, DbError::Integrity(_)));
    }

    #[test]
    fn validate_active_selection_rejects_disabled_in_slot() {
        let mut p = ProviderProfile {
            uuid: "u1".into(),
            template_id: "openai".into(),
            name: "OpenAI".into(),
            protocol: Protocol::OpenaiChat,
            endpoint: "https://api.openai.com".into(),
            model: None,
            enabled: false, // disabled
            sort_order: 0,
            is_local: false,
            needs_key: true,
            secret_ref: "openai".into(),
            capabilities: ProviderCapabilities::default(),
            status: "active".into(),
        };
        let providers = vec![p.clone()];
        let err = validate_active_selection("u1", &[], None, &providers).unwrap_err();
        assert!(matches!(err, DbError::Integrity(_)));

        // Fallback must be traditional.
        p.enabled = true;
        p.template_id = "openai".into(); // not traditional
        let providers = vec![p];
        let err = validate_active_selection("", &[], Some("u1"), &providers).unwrap_err();
        assert!(matches!(err, DbError::Integrity(_)));
    }

    #[test]
    fn validate_active_selection_accepts_traditional_fallback() {
        let mut p = ProviderProfile {
            uuid: "u1".into(),
            template_id: "google".into(),
            name: "Google".into(),
            protocol: Protocol::GoogleTranslate,
            endpoint: "https://translate.google.com".into(),
            model: None,
            enabled: true,
            sort_order: 0,
            is_local: false,
            needs_key: false,
            secret_ref: "provider/u1".into(),
            capabilities: ProviderCapabilities::default(),
            status: "active".into(),
        };
        let _ = &mut p;
        let providers = vec![p];
        validate_active_selection("", &[], Some("u1"), &providers).unwrap();
    }
}
