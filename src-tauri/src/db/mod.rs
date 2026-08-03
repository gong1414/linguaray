//! SQLite database wrapper (S2a — S0 §8.1).
//!
//! `Database` wraps a `rusqlite::Connection` in a `parking_lot::Mutex`.
//! The SOLE access pattern is `with_conn`, which passes `&mut Connection`
//! (enabling `conn.transaction()` for atomic writes).
//!
//! ## Lock-order rule (load-bearing)
//!
//! The DB Mutex and the keystore's fs2 flock are **never held simultaneously**.
//! Any operation touching both (delete, migration, set-key) follows:
//! 1. Lock DB → read snapshot → **unlock DB**.
//! 2. Perform keystore operation (under keystore's own lock).
//! 3. Lock DB → write result → unlock DB.
//!
//! There is no deadlock window because the two locks are never nested.

use std::path::Path;
use parking_lot::Mutex;
use rusqlite::Connection;

pub mod schema;
pub mod providers;
pub mod migration;
pub mod delete;
pub mod readiness;

// Re-export the readiness type at the crate root (lib.rs uses `readiness::`).
pub use readiness::DataReadiness;

/// Convenience re-export: derive the pre-migration settings backup path. Tests
/// assert on it without hardcoding the `.bak-pre-migration` suffix.
pub fn migration_settings_bak_path(settings_path: &Path) -> std::path::PathBuf {
    migration::settings_bak_path(settings_path)
}

/// Database error type. Wraps rusqlite errors + domain-specific errors.
#[derive(Debug)]
pub enum DbError {
    Sqlite(rusqlite::Error),
    Io(std::io::Error),
    Integrity(String),
    NotFound(String),
    /// Test-injected failure. Production code passes afp=None so this is never
    /// produced at runtime, but the variant must exist in release builds because
    /// the archive_database failpoint path references it in match arms.
    Injected(String),
}

impl std::fmt::Display for DbError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DbError::Sqlite(e) => write!(f, "sqlite: {e}"),
            DbError::Io(e) => write!(f, "io: {e}"),
            DbError::Integrity(s) => write!(f, "integrity: {s}"),
            DbError::NotFound(s) => write!(f, "not found: {s}"),
            DbError::Injected(s) => write!(f, "injected: {s}"),
        }
    }
}

impl std::error::Error for DbError {}

impl From<rusqlite::Error> for DbError {
    fn from(e: rusqlite::Error) -> Self { DbError::Sqlite(e) }
}

impl From<std::io::Error> for DbError {
    fn from(e: std::io::Error) -> Self { DbError::Io(e) }
}

impl From<crate::fs_acl::AclError> for DbError {
    fn from(e: crate::fs_acl::AclError) -> Self {
        match e {
            crate::fs_acl::AclError::Io(io) => DbError::Io(io),
            crate::fs_acl::AclError::Win32(s) => DbError::Integrity(s),
        }
    }
}

/// The SQLite database wrapper. Connection is behind a Mutex; all access
/// goes through `with_conn`.
pub struct Database {
    conn: Mutex<Connection>,
}

impl Database {
    /// Open (or create) the database at `path`.
    ///
    /// Order: create + secure parent dir → open connection → secure file
    /// → set pragmas (foreign_keys, busy_timeout, journal_mode=DELETE,
    /// synchronous=FULL).
    pub fn open(path: &Path) -> Result<Self, DbError> {
        let dir = path.parent().unwrap_or(Path::new("."));
        std::fs::create_dir_all(dir)?;
        crate::fs_acl::secure_dir(dir)?;
        let conn = Connection::open(path)?;
        crate::fs_acl::secure_file(path)?;
        conn.pragma_update(None, "foreign_keys", "ON")?;
        conn.pragma_update(None, "busy_timeout", 5000)?;
        conn.pragma_update(None, "journal_mode", "DELETE")?;
        conn.pragma_update(None, "synchronous", "FULL")?;
        Ok(Self { conn: Mutex::new(conn) })
    }

    /// The SOLE access pattern. `f` receives `&mut Connection` so it can
    /// call `conn.transaction()` for atomic multi-statement writes.
    /// The Mutex is held only for the duration of `f`.
    pub fn with_conn<T>(
        &self,
        f: impl FnOnce(&mut Connection) -> Result<T, DbError>,
    ) -> Result<T, DbError> {
        let mut conn = self.conn.lock();
        f(&mut conn)
    }

    /// Explicitly close the connection. Consumes self.
    ///
    /// Matches rusqlite 0.40.1's `Connection::close(self) -> Result<(), (Self, Error)>`:
    /// on failure, returns the Connection back so the caller can recover it.
    /// The large Err variant is inherent to the rusqlite contract (must return
    /// the Connection for recovery) — boxing would complicate the caller.
    #[allow(clippy::result_large_err)]
    pub fn close(self) -> Result<(), (Database, rusqlite::Error)> {
        let conn = self.conn.into_inner();
        conn.close().map_err(|(conn, e)| (Database { conn: Mutex::new(conn) }, e))
    }
}
