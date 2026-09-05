//! File commits shared by settings, personal libraries and backup export.
//!
//! Stage next to the destination, flush its contents, then atomically replace
//! the destination. A failed write never deletes the previous committed file.
//! Uncommitted staging files have no `.json` extension and are never loaded or
//! exported as user data. Process termination may leave one behind; subsequent
//! writes use unique names so they do not trust or collide with that file.

use std::fs::{self, File};
use std::io::Write;
use std::path::Path;

pub(crate) fn write_bytes(path: &Path, bytes: &[u8]) -> Result<(), String> {
    write_atomic(path, |file| {
        file.write_all(bytes)
            .map_err(|error| format!("failed to write `{}`: {error}", path.display()))
    })
}

pub(crate) fn write_atomic<T>(
    path: &Path,
    write: impl FnOnce(&mut File) -> Result<T, String>,
) -> Result<T, String> {
    let parent = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    fs::create_dir_all(parent)
        .map_err(|error| format!("failed to create `{}`: {error}", parent.display()))?;
    let mut staged = tempfile::Builder::new()
        .prefix(".linguaray-")
        .suffix(".tmp")
        .tempfile_in(parent)
        .map_err(|error| format!("failed to stage `{}`: {error}", path.display()))?;
    let value = write(staged.as_file_mut())?;
    staged
        .as_file()
        .sync_all()
        .map_err(|error| format!("failed to flush `{}`: {error}", path.display()))?;
    staged
        .persist(path)
        .map_err(|error| format!("failed to commit `{}`: {}", path.display(), error.error))?;

    // The commit has already happened. Report a directory-sync failure without
    // returning a false rollback result to the in-memory transaction owner.
    // Windows does not expose directory fsync through std::fs. Power-loss
    // durability ultimately depends on the filesystem and storage hardware.
    #[cfg(unix)]
    if let Err(error) = File::open(parent).and_then(|directory| directory.sync_all()) {
        eprintln!(
            "[storage] committed `{}` but directory sync failed: {error}",
            path.display()
        );
    }
    Ok(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn replaces_only_after_the_complete_write() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("settings.json");
        fs::write(&path, b"old").unwrap();
        write_atomic(&path, |file| {
            file.write_all(b"new").unwrap();
            assert_eq!(fs::read(&path).unwrap(), b"old");
            Ok(())
        })
        .unwrap();
        assert_eq!(fs::read(&path).unwrap(), b"new");
        assert_eq!(fs::read_dir(dir.path()).unwrap().count(), 1);
    }

    #[test]
    fn failed_write_preserves_previous_file_and_removes_staging() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("history.json");
        fs::write(&path, b"committed").unwrap();
        let result: Result<(), String> = write_atomic(&path, |file| {
            file.write_all(b"partial").unwrap();
            Err("injected write failure".to_owned())
        });
        assert!(result.is_err());
        assert_eq!(fs::read(&path).unwrap(), b"committed");
        assert_eq!(fs::read_dir(dir.path()).unwrap().count(), 1);
    }

    #[test]
    fn failed_replacement_preserves_destination_and_removes_staging() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("glossary.json");
        fs::create_dir(&path).unwrap();
        fs::write(path.join("sentinel"), b"keep").unwrap();
        assert!(write_bytes(&path, b"new").is_err());
        assert_eq!(fs::read(path.join("sentinel")).unwrap(), b"keep");
        assert_eq!(fs::read_dir(dir.path()).unwrap().count(), 1);
    }

    #[test]
    fn writes_new_nested_file_without_consuming_abandoned_staging() {
        let dir = tempfile::tempdir().unwrap();
        let parent = dir.path().join("library");
        let path = parent.join("vocabulary.json");
        write_bytes(&path, b"first").unwrap();
        let abandoned = parent.join(".linguaray-abandoned.tmp");
        fs::write(&abandoned, b"incomplete").unwrap();
        write_bytes(&path, b"second").unwrap();
        assert_eq!(fs::read(&path).unwrap(), b"second");
        assert_eq!(fs::read(&abandoned).unwrap(), b"incomplete");
    }

    #[test]
    fn terminated_writer_leaves_the_committed_file_readable() {
        const CHILD_PATH: &str = "LINGUARAY_STORAGE_CRASH_FIXTURE";
        if let Some(path) = std::env::var_os(CHILD_PATH) {
            let _: Result<(), String> = write_atomic(Path::new(&path), |file| {
                file.write_all(b"partial").unwrap();
                file.sync_all().unwrap();
                std::process::exit(17);
            });
            unreachable!();
        }
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("settings.json");
        fs::write(&path, b"committed").unwrap();
        let status = std::process::Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "storage::tests::terminated_writer_leaves_the_committed_file_readable",
            ])
            .env(CHILD_PATH, &path)
            .status()
            .unwrap();
        assert_eq!(status.code(), Some(17));
        assert_eq!(fs::read(&path).unwrap(), b"committed");
        write_bytes(&path, b"recovered").unwrap();
        assert_eq!(fs::read(&path).unwrap(), b"recovered");
    }
}
