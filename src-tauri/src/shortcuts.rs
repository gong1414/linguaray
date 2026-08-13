//! Shortcut domain model and transactional controller for Surface 07.

use std::collections::HashSet;
use std::fmt;
use std::sync::Arc;

use parking_lot::Mutex;
use serde::{Deserialize, Serialize};

use crate::db::shortcuts as shortcut_db;
use crate::db::{Database, DbError};

pub use shortcut_db::{ShortcutAction, ShortcutBinding};

/// OS adapter. Implementations must replace the complete registration set
/// atomically or leave the previously registered set untouched on failure.
pub trait Registrar: Send + Sync {
    fn replace_all(&self, shortcuts: &[(ShortcutAction, String)]) -> Result<(), String>;
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShortcutEntry {
    pub action: ShortcutAction,
    pub combo: String,
    pub available: bool,
    pub registration_state: ShortcutRegistrationState,
    pub registration_error: Option<String>,
    pub recording: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ShortcutRegistrationState {
    Registered,
    RegistrationFailed,
    Unavailable,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShortcutSnapshot {
    pub revision: u64,
    pub entries: Vec<ShortcutEntry>,
}

impl ShortcutSnapshot {
    pub fn entry(&self, action: ShortcutAction) -> Option<&ShortcutEntry> {
        self.entries.iter().find(|entry| entry.action == action)
    }

    pub fn combo(&self, action: ShortcutAction) -> Option<&str> {
        self.entry(action).map(|entry| entry.combo.as_str())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "error", rename_all = "snake_case")]
pub enum ShortcutError {
    InvalidCombo { message: String },
    Unavailable { action: ShortcutAction },
    Conflict { action: ShortcutAction },
    StaleRevision { expected: u64, actual: u64 },
    InvalidOverride { action: ShortcutAction },
    RegistrationFailed { message: String },
    DatabaseFailed { message: String },
    RollbackFailed { operation: String, rollback: String },
}

impl fmt::Display for ShortcutError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidCombo { message } => write!(formatter, "invalid shortcut: {message}"),
            Self::Unavailable { action } => {
                write!(
                    formatter,
                    "shortcut action `{}` is unavailable",
                    action.as_str()
                )
            }
            Self::Conflict { action } => {
                write!(formatter, "shortcut conflicts with `{}`", action.as_str())
            }
            Self::StaleRevision { expected, actual } => {
                write!(
                    formatter,
                    "stale shortcut revision {expected}; current is {actual}"
                )
            }
            Self::InvalidOverride { action } => {
                write!(
                    formatter,
                    "override action `{}` is not the conflict",
                    action.as_str()
                )
            }
            Self::RegistrationFailed { message } => {
                write!(formatter, "registration failed: {message}")
            }
            Self::DatabaseFailed { message } => write!(formatter, "database failed: {message}"),
            Self::RollbackFailed {
                operation,
                rollback,
            } => write!(
                formatter,
                "database failed: {operation}; registration rollback failed: {rollback}"
            ),
        }
    }
}

impl std::error::Error for ShortcutError {}

impl From<DbError> for ShortcutError {
    fn from(error: DbError) -> Self {
        Self::DatabaseFailed {
            message: error.to_string(),
        }
    }
}

struct ControllerState {
    revision: u64,
    recording: Option<ShortcutAction>,
    registration_error: Option<String>,
}

pub struct ShortcutController {
    db: Arc<Database>,
    registrar: Arc<dyn Registrar>,
    state: Mutex<ControllerState>,
}

impl ShortcutController {
    pub fn new(db: Arc<Database>, registrar: Arc<dyn Registrar>) -> Result<Self, ShortcutError> {
        shortcut_db::ensure_defaults(&db)?;
        let bindings = load_canonical(&db)?;
        // Startup is fail-soft: an OS-reserved conflict must not crash the
        // application or hide the rest of Settings. Keep the persisted map and
        // surface the authoritative registration error in the snapshot.
        let registration_error = registrar.replace_all(&registrable(&bindings)).err();
        Ok(Self {
            db,
            registrar,
            state: Mutex::new(ControllerState {
                revision: 0,
                recording: None,
                registration_error,
            }),
        })
    }

    /// IPC `list` domain operation.
    pub fn snapshot(&self) -> Result<ShortcutSnapshot, ShortcutError> {
        let state = self.state.lock();
        let bindings = load_canonical(&self.db)?;
        Ok(snapshot_from(
            &bindings,
            state.revision,
            state.recording,
            state.registration_error.as_deref(),
        ))
    }

    /// IPC `check_conflict(action, combo, revision)` domain operation.
    pub fn check_conflict(
        &self,
        action: ShortcutAction,
        combo: &str,
        revision: u64,
    ) -> Result<Option<ShortcutAction>, ShortcutError> {
        if !action.available() {
            return Err(ShortcutError::Unavailable { action });
        }
        let state = self.state.lock();
        require_revision(&state, revision)?;
        let combo = canonicalize(combo)?;
        let bindings = load_canonical(&self.db)?;
        Ok(find_conflict(&bindings, action, &combo))
    }

    /// IPC `save(action, combo, expectedRevision, overrideAction?)` operation.
    pub fn save(
        &self,
        action: ShortcutAction,
        combo: &str,
        expected_revision: u64,
        override_action: Option<ShortcutAction>,
    ) -> Result<ShortcutSnapshot, ShortcutError> {
        if !action.available() {
            return Err(ShortcutError::Unavailable { action });
        }
        let combo = canonicalize(combo)?;
        let mut state = self.state.lock();
        require_revision(&state, expected_revision)?;
        let old = load_canonical(&self.db)?;
        let mut next = old.clone();
        let conflict = find_conflict(&next, action, &combo);

        match (conflict, override_action) {
            (Some(conflict), Some(override_action)) if conflict == override_action => {
                if !conflict.available() {
                    return Err(ShortcutError::Unavailable { action: conflict });
                }
                let previous = binding(&next, action)?.combo.clone();
                binding_mut(&mut next, action)?.combo = combo;
                binding_mut(&mut next, conflict)?.combo = previous;
            }
            (Some(_conflict), Some(override_action)) => {
                return Err(ShortcutError::InvalidOverride {
                    action: override_action,
                });
            }
            (Some(conflict), None) => return Err(ShortcutError::Conflict { action: conflict }),
            (None, Some(override_action)) => {
                return Err(ShortcutError::InvalidOverride {
                    action: override_action,
                });
            }
            (None, None) => binding_mut(&mut next, action)?.combo = combo,
        }

        persist_with_registration_rollback(&self.db, self.registrar.as_ref(), &old, &next)?;
        state.revision = state.revision.saturating_add(1);
        state.recording = None;
        state.registration_error = None;
        Ok(snapshot_from(&next, state.revision, state.recording, None))
    }

    /// IPC `reset_defaults(expectedRevision)` domain operation.
    pub fn reset_defaults(
        &self,
        expected_revision: u64,
    ) -> Result<ShortcutSnapshot, ShortcutError> {
        let mut state = self.state.lock();
        require_revision(&state, expected_revision)?;
        let old = load_canonical(&self.db)?;
        let next = shortcut_db::defaults();
        persist_with_registration_rollback(&self.db, self.registrar.as_ref(), &old, &next)?;
        state.revision = state.revision.saturating_add(1);
        state.recording = None;
        state.registration_error = None;
        Ok(snapshot_from(&next, state.revision, state.recording, None))
    }

    /// IPC `recording_begin(action)` domain operation. Recording is ephemeral.
    pub fn recording_begin(&self, action: ShortcutAction) -> Result<(), ShortcutError> {
        if !action.available() {
            return Err(ShortcutError::Unavailable { action });
        }
        self.state.lock().recording = Some(action);
        Ok(())
    }

    /// IPC `recording_end()` domain operation.
    pub fn recording_end(&self) {
        self.state.lock().recording = None;
    }

    /// Global shortcut callbacks use this to no-op while the settings page is
    /// capturing a replacement key combination.
    pub fn is_recording(&self) -> bool {
        self.state.lock().recording.is_some()
    }
}

/// Parse and canonicalize a platform-neutral shortcut string.
///
/// Modifiers are emitted in `Ctrl+Alt+Shift+Super+Key` order. At least one
/// modifier and exactly one primary key are required.
pub fn canonicalize(value: &str) -> Result<String, ShortcutError> {
    let parts = value.split('+').map(str::trim).collect::<Vec<_>>();
    if parts.iter().any(|part| part.is_empty()) {
        return invalid("empty shortcut segment");
    }

    let mut modifiers = [false; 4];
    let mut primary = None;
    for part in parts {
        if let Some(index) = modifier_index(part) {
            if modifiers[index] {
                return invalid(format!("duplicate modifier `{part}`"));
            }
            modifiers[index] = true;
        } else {
            if primary.is_some() {
                return invalid("exactly one primary key is required");
            }
            primary = Some(canonical_key(part)?);
        }
    }

    if !modifiers.into_iter().any(|present| present) {
        return invalid("at least one modifier is required");
    }
    let primary = primary.ok_or_else(|| ShortcutError::InvalidCombo {
        message: "a primary key is required".into(),
    })?;

    let mut output = Vec::with_capacity(5);
    for (present, name) in modifiers.into_iter().zip(["Ctrl", "Alt", "Shift", "Super"]) {
        if present {
            output.push(name.to_owned());
        }
    }
    output.push(primary);
    Ok(output.join("+"))
}

fn modifier_index(value: &str) -> Option<usize> {
    if value.eq_ignore_ascii_case("ctrl") || value.eq_ignore_ascii_case("control") {
        Some(0)
    } else if value.eq_ignore_ascii_case("alt") || value.eq_ignore_ascii_case("option") {
        Some(1)
    } else if value.eq_ignore_ascii_case("shift") {
        Some(2)
    } else if ["super", "cmd", "command", "meta", "win", "windows"]
        .iter()
        .any(|alias| value.eq_ignore_ascii_case(alias))
    {
        Some(3)
    } else {
        None
    }
}

fn canonical_key(value: &str) -> Result<String, ShortcutError> {
    if value.chars().count() == 1 {
        let character = value.chars().next().expect("one character checked");
        if character.is_ascii_alphanumeric() {
            return Ok(character.to_ascii_uppercase().to_string());
        }
    }

    let lower = value.to_ascii_lowercase();
    let named = match lower.as_str() {
        "space" => Some("Space"),
        "enter" | "return" => Some("Enter"),
        "tab" => Some("Tab"),
        "escape" | "esc" => Some("Escape"),
        "backspace" => Some("Backspace"),
        "delete" | "del" => Some("Delete"),
        "insert" | "ins" => Some("Insert"),
        "home" => Some("Home"),
        "end" => Some("End"),
        "pageup" => Some("PageUp"),
        "pagedown" => Some("PageDown"),
        "up" | "arrowup" => Some("ArrowUp"),
        "down" | "arrowdown" => Some("ArrowDown"),
        "left" | "arrowleft" => Some("ArrowLeft"),
        "right" | "arrowright" => Some("ArrowRight"),
        _ => None,
    };
    if let Some(named) = named {
        return Ok(named.to_owned());
    }
    if let Some(number) = lower
        .strip_prefix('f')
        .and_then(|number| number.parse::<u8>().ok())
    {
        if (1..=24).contains(&number) {
            return Ok(format!("F{number}"));
        }
    }
    invalid(format!("unsupported primary key `{value}`"))
}

fn invalid<T>(message: impl Into<String>) -> Result<T, ShortcutError> {
    Err(ShortcutError::InvalidCombo {
        message: message.into(),
    })
}

fn require_revision(state: &ControllerState, expected: u64) -> Result<(), ShortcutError> {
    if state.revision == expected {
        Ok(())
    } else {
        Err(ShortcutError::StaleRevision {
            expected,
            actual: state.revision,
        })
    }
}

fn load_canonical(db: &Database) -> Result<Vec<ShortcutBinding>, ShortcutError> {
    let mut bindings = shortcut_db::load(db)?;
    let mut combos = HashSet::with_capacity(bindings.len());
    for binding in &mut bindings {
        binding.combo = canonicalize(&binding.combo)?;
        if !combos.insert(binding.combo.clone()) {
            return Err(ShortcutError::DatabaseFailed {
                message: format!("duplicate shortcut combo `{}`", binding.combo),
            });
        }
    }
    Ok(bindings)
}

fn find_conflict(
    bindings: &[ShortcutBinding],
    action: ShortcutAction,
    combo: &str,
) -> Option<ShortcutAction> {
    bindings
        .iter()
        .find(|binding| binding.action != action && binding.combo == combo)
        .map(|binding| binding.action)
}

fn binding(
    bindings: &[ShortcutBinding],
    action: ShortcutAction,
) -> Result<&ShortcutBinding, ShortcutError> {
    bindings
        .iter()
        .find(|binding| binding.action == action)
        .ok_or_else(|| ShortcutError::DatabaseFailed {
            message: format!("missing `{}` binding", action.as_str()),
        })
}

fn binding_mut(
    bindings: &mut [ShortcutBinding],
    action: ShortcutAction,
) -> Result<&mut ShortcutBinding, ShortcutError> {
    bindings
        .iter_mut()
        .find(|binding| binding.action == action)
        .ok_or_else(|| ShortcutError::DatabaseFailed {
            message: format!("missing `{}` binding", action.as_str()),
        })
}

fn registrable(bindings: &[ShortcutBinding]) -> Vec<(ShortcutAction, String)> {
    bindings
        .iter()
        .filter(|binding| binding.action.available())
        .map(|binding| (binding.action, binding.combo.clone()))
        .collect()
}

fn persist_with_registration_rollback(
    db: &Database,
    registrar: &dyn Registrar,
    old: &[ShortcutBinding],
    next: &[ShortcutBinding],
) -> Result<(), ShortcutError> {
    registrar
        .replace_all(&registrable(next))
        .map_err(|message| ShortcutError::RegistrationFailed { message })?;
    if let Err(operation) = shortcut_db::replace_all(db, next) {
        return match registrar.replace_all(&registrable(old)) {
            Ok(()) => Err(ShortcutError::DatabaseFailed {
                message: operation.to_string(),
            }),
            Err(rollback) => Err(ShortcutError::RollbackFailed {
                operation: operation.to_string(),
                rollback,
            }),
        };
    }
    Ok(())
}

fn snapshot_from(
    bindings: &[ShortcutBinding],
    revision: u64,
    recording: Option<ShortcutAction>,
    registration_error: Option<&str>,
) -> ShortcutSnapshot {
    ShortcutSnapshot {
        revision,
        entries: bindings
            .iter()
            .map(|binding| ShortcutEntry {
                action: binding.action,
                combo: binding.combo.clone(),
                available: binding.action.available(),
                registration_state: if !binding.action.available() {
                    ShortcutRegistrationState::Unavailable
                } else if registration_error.is_some() {
                    ShortcutRegistrationState::RegistrationFailed
                } else {
                    ShortcutRegistrationState::Registered
                },
                registration_error: binding
                    .action
                    .available()
                    .then(|| registration_error.map(str::to_owned))
                    .flatten(),
                recording: recording == Some(binding.action),
            })
            .collect(),
    }
}
