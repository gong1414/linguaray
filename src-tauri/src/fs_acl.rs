//! Shared file/directory permission helpers.
//!
//! On Unix: sets 0o700 for directories and 0o600 for files (owner-only).
//! On Windows: the DB file does not hold secrets (keys live in the keystore,
//! which has its own Win32 DACL via `keystore.rs`). Windows is a no-op here;
//! the keystore module handles its own ACLs.

use std::path::Path;

#[cfg(unix)]
fn set_mode(path: &Path, mode: u32) -> std::io::Result<()> {
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(mode))
}

/// Secure a directory: 0o700 on Unix (owner-only read/write/execute).
pub fn secure_dir(dir: &Path) -> std::io::Result<()> {
    #[cfg(unix)]
    { set_mode(dir, 0o700) }
    #[cfg(not(unix))]
    { let _ = dir; Ok(()) }
}

/// Secure a file: 0o600 on Unix (owner-only read/write).
pub fn secure_file(path: &Path) -> std::io::Result<()> {
    #[cfg(unix)]
    { set_mode(path, 0o600) }
    #[cfg(not(unix))]
    { let _ = path; Ok(()) }
}
