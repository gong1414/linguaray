//! Provider IPC split by domain (refactor P3.2; signatures unchanged):
//! - [`catalog`] — preset catalog rows.
//! - [`crud`] — list/create/update/duplicate/delete/reorder/toggle.
//! - [`keys`] — keystore key set/clear (cross-store, lock-ordered).
//! - [`roles`] — active-selection + consent + tray switch core.
//! - [`connection`] — diagnostics: models / probe / balance.
//!
//! The shared structured error/result enums live here; every historical
//! `crate::commands::providers::X` path keeps resolving via the re-exports.

mod catalog;
mod connection;
mod crud;
mod keys;
mod roles;

pub use catalog::{provider_list_presets, CatalogPresetDto};
pub use connection::{
    measure_latency_ms, provider_get_balance, provider_get_models, provider_test_connection,
    ConnectionResult, ModelInfo,
};
// parse_model_ids is exercised by the lib.rs unit tests via this path.
#[cfg(test)]
pub(crate) use connection::parse_model_ids;
pub use crud::{
    provider_create, provider_delete, provider_duplicate, provider_list, provider_reorder,
    provider_toggle, provider_update,
};
pub use keys::{provider_set_key, set_key_blocking};
pub use roles::{
    db_set_active_primary, handle_switch_provider, handle_switch_provider_core,
    provider_confirm_and_set_active, provider_get_active_selection, provider_set_active,
    SetActiveOutcome,
};

/// Result of [`provider_set_active`] (P1.1). A serializable tagged union so the
/// frontend distinguishes "written" from "needs consent" via a structured
/// payload instead of parsing an error string.
///
/// Wire shapes:
/// - `Written` → `{"outcome":"written"}`
/// - `NeedsConsent { actual_scope }` → `{"outcome":"needs_consent","actual_scope":"..."}`
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq, specta::Type)]
#[serde(tag = "outcome", rename_all = "snake_case")]
pub enum SetActiveResult {
    /// The selection was written (no consent needed, or scope already matched).
    Written,
    /// A non-empty parallel selection needs explicit consent; carries the
    /// canonical scope the frontend must echo back via
    /// `provider_confirm_and_set_active`.
    NeedsConsent { actual_scope: String },
}

/// Structured error type for provider IPC commands (P1.1 fix).
/// Replaces free-form `String` errors so the frontend can pattern-match
/// instead of parsing string prefixes.
/// Wire shape for StaleScope: `{"error":"stale_scope","actual_scope":"..."}`
/// Wire shape for StaleVersion (R2-E): `{"error":"stale_version","actual_version":N}`
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq, specta::Type)]
#[serde(tag = "error", rename_all = "snake_case")]
pub enum ProviderCommandError {
    StaleScope {
        actual_scope: String,
    },
    /// Optimistic-lock mismatch (R2-E): the provider row was modified elsewhere
    /// since the frontend last read it. Carries the row's actual version so the
    /// UI can prompt a reload.
    StaleVersion {
        actual_version: i64,
    },
    /// Generic database or validation error.
    Db {
        message: String,
    },
    /// Provider not found, invalid selection, etc.
    Validation {
        message: String,
    },
}

impl std::fmt::Display for ProviderCommandError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::StaleScope { actual_scope } => {
                write!(f, "stale scope: {actual_scope}")
            }
            Self::StaleVersion { actual_version } => {
                write!(f, "stale version: row is at version {actual_version}")
            }
            Self::Db { message } => write!(f, "{message}"),
            Self::Validation { message } => write!(f, "{message}"),
        }
    }
}

impl From<crate::db::DbError> for ProviderCommandError {
    fn from(e: crate::db::DbError) -> Self {
        ProviderCommandError::Db {
            message: e.to_string(),
        }
    }
}

impl From<String> for ProviderCommandError {
    fn from(message: String) -> Self {
        ProviderCommandError::Validation { message }
    }
}
