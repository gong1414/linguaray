//! Dictionary package install with hardening: path traversal, symlink, bomb,
//! and atomic copy+rollback.

use std::path::{Path, PathBuf};

use rusqlite::Connection;
use serde::Serialize;
use thiserror::Error;

use crate::db::{Database, DbError};

const MAX_TOTAL_SIZE: u64 = 500 * 1024 * 1024;
const MAX_FILE_COUNT: usize = 10_000;
const MAX_FILE_SIZE: u64 = 200 * 1024 * 1024;

#[derive(Debug, Clone, Serialize)]
pub struct DictPackageInfo {
    pub package_id: String,
    pub name: String,
    pub version: String,
    pub installed_at: i64,
}

#[derive(Debug, Error)]
pub enum PackageError {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("db: {0}")]
    Db(#[from] DbError),
    #[error("invalid package_id: contains path separator or traversal")]
    InvalidPackageId,
    #[error("source contains a symlink: {0}")]
    Symlink(PathBuf),
    #[error("package exceeds size limit ({limit} bytes): {actual} bytes")]
    Bomb { limit: u64, actual: u64 },
    #[error("package exceeds file count limit ({limit}): {actual} files")]
    TooManyFiles { limit: usize, actual: usize },
    #[error("package_id already exists")]
    Duplicate,
}

/// Install a dictionary package. Copies files from `source_dir` to
/// `dest_root/{package_id}`, validates, and registers in the DB atomically.
pub fn install_package(
    db: &Database,
    source_dir: &Path,
    dest_root: &Path,
    package_id: &str,
    name: &str,
    version: &str,
) -> Result<(), PackageError> {
    validate_package_id(package_id)?;
    let dest = dest_root.join(package_id);
    let temp_dest = dest_root.join(format!("{package_id}.tmp"));

    if temp_dest.exists() {
        let _ = std::fs::remove_dir_all(&temp_dest);
    }

    let files = collect_files(source_dir)?;
    validate_no_symlinks(&files)?;
    let _total_size = validate_sizes(&files)?;

    std::fs::create_dir_all(&temp_dest)?;
    for (src, rel) in &files {
        let dst = temp_dest.join(rel);
        if let Some(parent) = dst.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::copy(src, &dst)?;
    }

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|_| {
            PackageError::Io(std::io::Error::other(
                "system clock precedes Unix epoch",
            ))
        })?;
    let now = i64::try_from(now.as_secs()).unwrap_or(0);

    let db_result = db.with_conn(|conn| {
        let tx = conn.transaction()?;
        let changed = tx.execute(
            "INSERT OR IGNORE INTO dict_packages (package_id, name, version, installed_at) VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params![package_id, name, version, now],
        )?;
        if changed != 1 {
            tx.rollback()?;
            return Err(DbError::Integrity("package_id already exists".into()));
        }
        tx.commit()?;
        Ok(())
    });

    match db_result {
        Ok(()) => {
            if dest.exists() {
                let _ = std::fs::remove_dir_all(&dest);
            }
            if let Err(e) = std::fs::rename(&temp_dest, &dest) {
                // The row is already visible. Roll it back so a failed
                // publish cannot leave "installed in DB, files missing".
                let _ = db.with_conn(|conn| {
                    conn.execute(
                        "DELETE FROM dict_packages WHERE package_id = ?1",
                        [package_id],
                    )?;
                    Ok(())
                });
                let _ = std::fs::remove_dir_all(&temp_dest);
                return Err(PackageError::Io(e));
            }
            Ok(())
        }
        Err(e) => {
            let _ = std::fs::remove_dir_all(&temp_dest);
            if matches!(e, DbError::Integrity(_)) {
                Err(PackageError::Duplicate)
            } else {
                Err(PackageError::Db(e))
            }
        }
    }
}

/// List all installed packages.
pub fn list_packages(conn: &mut Connection) -> Result<Vec<DictPackageInfo>, DbError> {
    let mut stmt = conn.prepare(
        "SELECT package_id, name, version, installed_at FROM dict_packages ORDER BY name",
    )?;
    let rows = stmt
        .query_map([], |row| {
            Ok(DictPackageInfo {
                package_id: row.get(0)?,
                name: row.get(1)?,
                version: row.get(2)?,
                installed_at: row.get(3)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

fn validate_package_id(id: &str) -> Result<(), PackageError> {
    if id.is_empty()
        || id.contains('/')
        || id.contains('\\')
        || id.contains("..")
        || id.contains('\0')
    {
        return Err(PackageError::InvalidPackageId);
    }
    Ok(())
}

fn collect_files(dir: &Path) -> Result<Vec<(PathBuf, PathBuf)>, PackageError> {
    let mut files = Vec::new();
    collect_files_recursive(dir, dir, &mut files)?;
    if files.len() > MAX_FILE_COUNT {
        return Err(PackageError::TooManyFiles {
            limit: MAX_FILE_COUNT,
            actual: files.len(),
        });
    }
    Ok(files)
}

fn collect_files_recursive(
    base: &Path,
    current: &Path,
    files: &mut Vec<(PathBuf, PathBuf)>,
) -> Result<(), PackageError> {
    for entry in std::fs::read_dir(current)? {
        let entry = entry?;
        let path = entry.path();
        let meta = std::fs::symlink_metadata(&path)?;
        if is_forbidden_link(&meta) {
            return Err(PackageError::Symlink(path));
        }
        if meta.is_dir() {
            collect_files_recursive(base, &path, files)?;
        } else {
            let rel = path.strip_prefix(base).unwrap().to_path_buf();
            if rel
                .components()
                .any(|c| matches!(c, std::path::Component::ParentDir))
            {
                return Err(PackageError::InvalidPackageId);
            }
            files.push((path, rel));
        }
    }
    Ok(())
}

fn validate_no_symlinks(files: &[(PathBuf, PathBuf)]) -> Result<(), PackageError> {
    for (src, _) in files {
        let meta = std::fs::symlink_metadata(src)?;
        if is_forbidden_link(&meta) {
            return Err(PackageError::Symlink(src.clone()));
        }
    }
    Ok(())
}

fn is_forbidden_link(meta: &std::fs::Metadata) -> bool {
    if meta.file_type().is_symlink() {
        return true;
    }
    // Windows junctions / mount points are reparse points, not POSIX
    // symlinks. Rust's `is_symlink()` is false for them, but they are
    // the same class of install-time escape and must be rejected.
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        const FILE_ATTRIBUTE_REPARSE_POINT: u32 = 0x400;
        if meta.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0 {
            return true;
        }
    }
    false
}

fn validate_sizes(files: &[(PathBuf, PathBuf)]) -> Result<u64, PackageError> {
    let mut total: u64 = 0;
    for (src, _) in files {
        let meta = std::fs::metadata(src)?;
        let size = meta.len();
        if size > MAX_FILE_SIZE {
            return Err(PackageError::Bomb {
                limit: MAX_FILE_SIZE,
                actual: size,
            });
        }
        total = total.checked_add(size).ok_or(PackageError::Bomb {
            limit: MAX_TOTAL_SIZE,
            actual: u64::MAX,
        })?;
        if total > MAX_TOTAL_SIZE {
            return Err(PackageError::Bomb {
                limit: MAX_TOTAL_SIZE,
                actual: total,
            });
        }
    }
    Ok(total)
}
