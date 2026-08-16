//! Persistence for the four global-shortcut actions.
//!
//! This module deliberately stores the complete shortcut map in one
//! transaction. Callers never observe a partially updated set of bindings.

use std::collections::HashSet;

use rusqlite::params;
use serde::{Deserialize, Serialize};

use super::{Database, DbError};

/// Stable identifiers used by the frontend and persisted in SQLite.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, specta::Type)]
pub enum ShortcutAction {
    #[serde(rename = "translate_selection")]
    Selection,
    #[serde(rename = "translate_input")]
    Input,
    #[serde(rename = "ocr_translate")]
    Ocr,
    #[serde(rename = "translate_clipboard")]
    Clipboard,
}

impl ShortcutAction {
    pub const ALL: [Self; 4] = [Self::Selection, Self::Input, Self::Clipboard, Self::Ocr];

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Selection => "translate_selection",
            Self::Input => "translate_input",
            Self::Ocr => "ocr_translate",
            Self::Clipboard => "translate_clipboard",
        }
    }

    pub const fn default_combo(self) -> &'static str {
        match self {
            Self::Selection => "Alt+Space",
            Self::Input => "Ctrl+Space",
            Self::Ocr => "Alt+Shift+Space",
            Self::Clipboard => "Ctrl+Alt+Space",
        }
    }

    pub const fn available(self) -> bool {
        true
    }

    fn parse(value: &str) -> Result<Self, DbError> {
        match value {
            "translate_selection" => Ok(Self::Selection),
            "translate_input" => Ok(Self::Input),
            "ocr_translate" => Ok(Self::Ocr),
            "translate_clipboard" => Ok(Self::Clipboard),
            other => Err(DbError::Integrity(format!(
                "unknown shortcut action `{other}`"
            ))),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, specta::Type)]
pub struct ShortcutBinding {
    pub action: ShortcutAction,
    pub combo: String,
}

pub fn defaults() -> Vec<ShortcutBinding> {
    ShortcutAction::ALL
        .into_iter()
        .map(|action| ShortcutBinding {
            action,
            combo: action.default_combo().to_owned(),
        })
        .collect()
}

/// Add any defaults missing from an existing database without overwriting
/// user choices.
pub fn ensure_defaults(db: &Database) -> Result<(), DbError> {
    db.with_conn(|conn| {
        let tx = conn.transaction()?;
        for binding in defaults() {
            tx.execute(
                "INSERT OR IGNORE INTO shortcuts(action, keys) VALUES (?1, ?2)",
                params![binding.action.as_str(), binding.combo],
            )?;
        }
        tx.commit()?;
        Ok(())
    })
}

/// Load an exact, deterministically ordered four-action snapshot.
pub fn load(db: &Database) -> Result<Vec<ShortcutBinding>, DbError> {
    let rows = db.with_conn(|conn| {
        let mut statement = conn.prepare("SELECT action, keys FROM shortcuts")?;
        let rows = statement
            .query_map([], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(rows)
    })?;

    let mut bindings = Vec::with_capacity(rows.len());
    for (action, combo) in rows {
        bindings.push(ShortcutBinding {
            action: ShortcutAction::parse(&action)?,
            combo,
        });
    }
    validate_complete(&bindings)?;
    bindings.sort_by_key(|binding| action_index(binding.action));
    Ok(bindings)
}

/// Replace all bindings in a single SQLite transaction.
pub fn replace_all(db: &Database, bindings: &[ShortcutBinding]) -> Result<(), DbError> {
    validate_complete(bindings)?;
    db.with_conn(|conn| {
        let tx = conn.transaction()?;
        tx.execute("DELETE FROM shortcuts", [])?;
        {
            let mut statement =
                tx.prepare("INSERT INTO shortcuts(action, keys) VALUES (?1, ?2)")?;
            for binding in bindings {
                statement.execute(params![binding.action.as_str(), binding.combo])?;
            }
        }
        tx.commit()?;
        Ok(())
    })
}

fn validate_complete(bindings: &[ShortcutBinding]) -> Result<(), DbError> {
    if bindings.len() != ShortcutAction::ALL.len() {
        return Err(DbError::Integrity(format!(
            "shortcut map must contain exactly four actions, got {}",
            bindings.len()
        )));
    }

    let mut actions = HashSet::with_capacity(bindings.len());
    let mut combos = HashSet::with_capacity(bindings.len());
    for binding in bindings {
        if !actions.insert(binding.action) {
            return Err(DbError::Integrity(format!(
                "duplicate shortcut action `{}`",
                binding.action.as_str()
            )));
        }
        if binding.combo.trim().is_empty() {
            return Err(DbError::Integrity(format!(
                "empty shortcut for `{}`",
                binding.action.as_str()
            )));
        }
        if !combos.insert(binding.combo.as_str()) {
            return Err(DbError::Integrity(format!(
                "duplicate shortcut combo `{}`",
                binding.combo
            )));
        }
    }

    if actions.len() != ShortcutAction::ALL.len() {
        return Err(DbError::Integrity(
            "shortcut action set is incomplete".into(),
        ));
    }
    Ok(())
}

const fn action_index(action: ShortcutAction) -> usize {
    match action {
        ShortcutAction::Selection => 0,
        ShortcutAction::Input => 1,
        ShortcutAction::Clipboard => 2,
        ShortcutAction::Ocr => 3,
    }
}
