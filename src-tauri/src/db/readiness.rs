//! Data readiness state (S2a step 6).
//!
//! [`DataReadiness`] is the single source of truth for "can the app serve
//! provider/data commands right now?" It's computed once at startup from the
//! DB-open + migration outcomes and held in [`crate::AppState`]. Provider
//! commands gate on it via [`crate::require_ready`]; a handful of commands
//! (`keystore_health`, `archive_keystore`, `reset_keystore`,
//! `get_data_readiness`) are always available so the UI can surface the
//! recovery banner and the user can act on it.
//!
//! The variants are ordered by "how broken":
//! - [`DataReadiness::Ready`] — DB open, migration complete, keystore healthy.
//! - [`DataReadiness::NeedsKeystoreRecovery`] — keystore unreadable; the user
//!   must archive + re-enter keys. No DB write that touches the keystore can
//!   proceed.
//! - [`DataReadiness::NeedsDatabaseRecovery`] — the SQLite file couldn't be
//!   opened at all (corrupt header / IO error). There is no DB to gate on.
//! - [`DataReadiness::MigrationIncomplete`] — the DB opened but migration did
//!   not reach `Complete` (a crash mid-migration, a corrupt settings file, or
//!   a resume-deletions failure). The optional checkpoint carries the
//!   failpoint / phase string for diagnostics.

use serde::{Deserialize, Serialize};

/// The app's data-readiness state. Held in `AppState.readiness` behind a
/// `parking_lot::RwLock` and read by every gated command.
///
/// `Clone + PartialEq + Eq` so `require_ready` can compare against `Ready`
/// cheaply, and so the frontend can diff the serialized form to drive the
/// recovery banner.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum DataReadiness {
    /// DB open, migration complete, keystore healthy — provider commands may run.
    Ready,
    /// Keystore is unreadable/corrupt; user must archive + re-enter keys.
    NeedsKeystoreRecovery {
        /// Human-readable reason (surfaced in the recovery banner).
        reason: String,
    },
    /// The SQLite database file could not be opened. No DB handle exists.
    NeedsDatabaseRecovery {
        /// Human-readable reason (surfaced in the recovery banner).
        reason: String,
    },
    /// DB opened but migration didn't reach `Complete` (crash replay, corrupt
    /// settings, or a resume-deletions failure). The DB may be partially set up.
    MigrationIncomplete {
        /// Failpoint / phase identifier when known (diagnostics). `None` when the
        /// failure wasn't at a named checkpoint.
        checkpoint: Option<String>,
        /// Human-readable reason.
        reason: String,
    },
}

impl Default for DataReadiness {
    /// The safe pre-setup default: assume not ready until startup says otherwise.
    /// A freshly constructed `AppState` (before `setup` runs the migration) reads
    /// as `MigrationIncomplete` so every gated command fails closed.
    fn default() -> Self {
        DataReadiness::MigrationIncomplete {
            checkpoint: None,
            reason: "startup not complete".to_string(),
        }
    }
}

impl DataReadiness {
    /// Convenience: is this `Ready`? Equivalent to `*self == DataReadiness::Ready`
    /// but reads clearer at call sites and avoids constructing a `Ready` for the
    /// comparison.
    pub fn is_ready(&self) -> bool {
        matches!(self, DataReadiness::Ready)
    }

    /// Convenience constructor for the common "migration failed at a named phase"
    /// case. Keeps the call sites in `setup` terse.
    pub fn migration_incomplete(checkpoint: impl Into<String>, reason: impl Into<String>) -> Self {
        DataReadiness::MigrationIncomplete {
            checkpoint: Some(checkpoint.into()),
            reason: reason.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_migration_incomplete() {
        // The pre-setup default must fail closed — gated commands must not run
        // before the migration has decided the readiness.
        let d = DataReadiness::default();
        assert!(!d.is_ready());
        assert!(matches!(
            d,
            DataReadiness::MigrationIncomplete {
                checkpoint: None,
                ..
            }
        ));
    }

    #[test]
    fn is_ready_only_for_ready() {
        assert!(DataReadiness::Ready.is_ready());
        assert!(!DataReadiness::NeedsKeystoreRecovery {
            reason: "x".into()
        }
        .is_ready());
        assert!(!DataReadiness::NeedsDatabaseRecovery {
            reason: "x".into()
        }
        .is_ready());
        assert!(
            !DataReadiness::MigrationIncomplete {
                checkpoint: None,
                reason: "x".into()
            }
            .is_ready()
        );
    }

    #[test]
    fn equality_is_value_based() {
        // require_ready compares `*readiness != DataReadiness::Ready`, so the
        // variants must be `Eq` on the full payload.
        let a = DataReadiness::NeedsKeystoreRecovery {
            reason: "corrupt".into(),
        };
        let b = DataReadiness::NeedsKeystoreRecovery {
            reason: "corrupt".into(),
        };
        assert_eq!(a, b);
        let c = DataReadiness::NeedsKeystoreRecovery {
            reason: "different".into(),
        };
        assert_ne!(a, c);
    }

    #[test]
    fn serde_round_trip_ready() {
        let s = serde_json::to_string(&DataReadiness::Ready).unwrap();
        let back: DataReadiness = serde_json::from_str(&s).unwrap();
        assert_eq!(back, DataReadiness::Ready);
        // Tagged form: { "state": "ready" }
        assert!(s.contains("\"state\":\"ready\""));
    }

    #[test]
    fn serde_round_trip_needs_keystore() {
        let v = DataReadiness::NeedsKeystoreRecovery {
            reason: "envelope malformed".into(),
        };
        let s = serde_json::to_string(&v).unwrap();
        let back: DataReadiness = serde_json::from_str(&s).unwrap();
        assert_eq!(back, v);
        assert!(s.contains("\"state\":\"needs_keystore_recovery\""));
        assert!(s.contains("\"reason\":\"envelope malformed\""));
    }

    #[test]
    fn serde_round_trip_migration_incomplete_with_checkpoint() {
        let v = DataReadiness::migration_incomplete("AfterSchema", "io error");
        let s = serde_json::to_string(&v).unwrap();
        let back: DataReadiness = serde_json::from_str(&s).unwrap();
        assert_eq!(back, v);
        assert!(s.contains("\"checkpoint\":\"AfterSchema\""));
    }

    #[test]
    fn migration_incomplete_helper_sets_checkpoint_some() {
        let v = DataReadiness::migration_incomplete("phase", "r");
        assert!(matches!(
            v,
            DataReadiness::MigrationIncomplete {
                checkpoint: Some(_),
                ..
            }
        ));
    }
}
