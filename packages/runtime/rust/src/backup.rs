use std::collections::HashSet;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipArchive, ZipWriter};

use crate::domain::glossary::GlossaryStore;
use crate::domain::history::HistoryStore;
use crate::domain::settings::Settings;
use crate::domain::vocabulary::VocabularyStore;

const BACKUP_FORMAT: &str = "linguaray-backup";
const BACKUP_VERSION: u32 = 1;
const MAX_ARCHIVE_ENTRIES: usize = 1024;
const MAX_UNCOMPRESSED_BYTES: u64 = 128 * 1024 * 1024;
const DATA_TARGETS: [&str; 4] = [
    "settings.json",
    "history.json",
    "vocabulary.json",
    "glossary",
];

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupManifest {
    pub format: String,
    pub version: u32,
    pub created_at: u64,
    pub includes_secrets: bool,
}

pub struct StagedBackup {
    pub directory: PathBuf,
    pub manifest: BackupManifest,
}

pub struct InstalledBackup {
    data_dir: PathBuf,
    rollback: PathBuf,
    saved_names: Vec<&'static str>,
    installed_names: Vec<&'static str>,
    committed: bool,
}

impl InstalledBackup {
    pub fn commit(mut self) {
        self.committed = true;
        let _ = fs::remove_dir_all(&self.rollback);
    }

    fn restore_originals(&mut self) {
        for name in self.installed_names.iter().copied() {
            let installed = self.data_dir.join(name);
            if installed.is_dir() {
                let _ = fs::remove_dir_all(&installed);
            } else if installed.exists() {
                let _ = fs::remove_file(&installed);
            }
        }
        for name in self.saved_names.iter().copied().rev() {
            let saved = self.rollback.join(name);
            if saved.exists() {
                let _ = fs::rename(saved, self.data_dir.join(name));
            }
        }
        let _ = fs::remove_dir_all(&self.rollback);
    }
}

impl Drop for InstalledBackup {
    fn drop(&mut self) {
        if !self.committed {
            self.restore_originals();
        }
    }
}

pub fn export_backup(data_dir: &Path, destination: &Path) -> Result<(u64, u32), String> {
    let parent = destination
        .parent()
        .ok_or_else(|| "backup destination has no parent".to_owned())?;
    fs::create_dir_all(parent).map_err(|error| {
        format!(
            "failed to create backup directory `{}`: {error}",
            parent.display()
        )
    })?;
    let temporary = destination.with_extension("linguaray-backup.tmp");
    if temporary.exists() {
        fs::remove_file(&temporary).map_err(|error| {
            format!(
                "failed to replace temporary backup `{}`: {error}",
                temporary.display()
            )
        })?;
    }

    let created_at = now_secs()?;
    let manifest = BackupManifest {
        format: BACKUP_FORMAT.to_owned(),
        version: BACKUP_VERSION,
        created_at,
        includes_secrets: false,
    };
    let file = File::create(&temporary)
        .map_err(|error| format!("failed to create backup `{}`: {error}", temporary.display()))?;
    let mut archive = ZipWriter::new(file);
    let options = SimpleFileOptions::default()
        .compression_method(CompressionMethod::Deflated)
        .unix_permissions(0o600);
    let manifest_json = serde_json::to_vec_pretty(&manifest)
        .map_err(|error| format!("failed to encode backup manifest: {error}"))?;
    archive
        .start_file("manifest.json", options)
        .map_err(|error| format!("failed to write backup manifest: {error}"))?;
    archive
        .write_all(&manifest_json)
        .map_err(|error| format!("failed to write backup manifest: {error}"))?;
    let mut file_count = 1_u32;

    for name in ["settings.json", "history.json", "vocabulary.json"] {
        let path = data_dir.join(name);
        if path.is_file() {
            write_file(&mut archive, &path, name, options)?;
            file_count += 1;
        }
    }
    archive
        .add_directory("glossary/", options)
        .map_err(|error| format!("failed to add glossary directory: {error}"))?;
    let glossary = data_dir.join("glossary");
    if glossary.is_dir() {
        let mut entries = fs::read_dir(&glossary)
            .map_err(|error| format!("failed to read glossary: {error}"))?
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .filter(|path| path.extension().and_then(|value| value.to_str()) == Some("json"))
            .collect::<Vec<_>>();
        entries.sort();
        for path in entries {
            let name = path
                .file_name()
                .and_then(|value| value.to_str())
                .ok_or_else(|| "glossary file name is not valid UTF-8".to_owned())?;
            write_file(&mut archive, &path, &format!("glossary/{name}"), options)?;
            file_count += 1;
        }
    }
    archive
        .finish()
        .map_err(|error| format!("failed to finish backup: {error}"))?;

    if destination.exists() {
        fs::remove_file(destination).map_err(|error| {
            format!(
                "failed to replace backup `{}`: {error}",
                destination.display()
            )
        })?;
    }
    fs::rename(&temporary, destination).map_err(|error| {
        format!(
            "failed to publish backup `{}`: {error}",
            destination.display()
        )
    })?;
    Ok((created_at, file_count))
}

pub fn stage_backup(source: &Path) -> Result<StagedBackup, String> {
    let file = File::open(source)
        .map_err(|error| format!("failed to open backup `{}`: {error}", source.display()))?;
    let mut archive =
        ZipArchive::new(file).map_err(|error| format!("failed to read backup archive: {error}"))?;
    if archive.len() > MAX_ARCHIVE_ENTRIES {
        return Err("backup contains too many files".to_owned());
    }
    let total_size = (0..archive.len()).try_fold(0_u64, |total, index| {
        let entry = archive
            .by_index(index)
            .map_err(|error| format!("failed to inspect backup entry: {error}"))?;
        total
            .checked_add(entry.size())
            .ok_or_else(|| "backup size overflow".to_owned())
    })?;
    if total_size > MAX_UNCOMPRESSED_BYTES {
        return Err("backup is larger than 128 MiB after extraction".to_owned());
    }

    let staging = std::env::temp_dir().join(format!(
        "linguaray-restore-{}-{}",
        std::process::id(),
        now_nanos()?
    ));
    fs::create_dir(&staging)
        .map_err(|error| format!("failed to create restore staging directory: {error}"))?;
    let extract_result = (|| {
        let mut paths = HashSet::new();
        for index in 0..archive.len() {
            let mut entry = archive
                .by_index(index)
                .map_err(|error| format!("failed to read backup entry: {error}"))?;
            let relative = entry
                .enclosed_name()
                .ok_or_else(|| "backup contains an unsafe path".to_owned())?;
            if entry.is_symlink() || !allowed_entry(&relative, entry.is_dir()) {
                return Err(format!(
                    "backup contains unsupported entry `{}`",
                    relative.display()
                ));
            }
            if !paths.insert(relative.clone()) {
                return Err(format!(
                    "backup contains duplicate entry `{}`",
                    relative.display()
                ));
            }
            let destination = staging.join(&relative);
            if entry.is_dir() {
                fs::create_dir_all(&destination)
                    .map_err(|error| format!("failed to create restore directory: {error}"))?;
                continue;
            }
            if let Some(parent) = destination.parent() {
                fs::create_dir_all(parent)
                    .map_err(|error| format!("failed to create restore directory: {error}"))?;
            }
            let mut output = File::create(&destination)
                .map_err(|error| format!("failed to create restore file: {error}"))?;
            std::io::copy(&mut entry, &mut output)
                .map_err(|error| format!("failed to extract restore file: {error}"))?;
        }
        Ok::<(), String>(())
    })();
    if let Err(error) = extract_result {
        let _ = fs::remove_dir_all(&staging);
        return Err(error);
    }

    let manifest_path = staging.join("manifest.json");
    let manifest = match fs::read_to_string(&manifest_path)
        .map_err(|error| format!("backup manifest is missing: {error}"))
        .and_then(|value| {
            serde_json::from_str::<BackupManifest>(&value)
                .map_err(|error| format!("backup manifest is invalid: {error}"))
        }) {
        Ok(manifest) => manifest,
        Err(error) => {
            let _ = fs::remove_dir_all(&staging);
            return Err(error);
        }
    };
    if manifest.format != BACKUP_FORMAT || manifest.version != BACKUP_VERSION {
        let _ = fs::remove_dir_all(&staging);
        return Err("backup format or version is not supported".to_owned());
    }
    if manifest.includes_secrets {
        let _ = fs::remove_dir_all(&staging);
        return Err("backups containing embedded secrets are not supported".to_owned());
    }
    if !staging.join("settings.json").is_file() {
        let _ = fs::remove_dir_all(&staging);
        return Err("backup does not contain settings.json".to_owned());
    }
    if let Err(error) = Settings::load(staging.join("settings.json"))
        .map(|_| ())
        .and_then(|()| HistoryStore::validate_backup(&staging))
        .and_then(|()| VocabularyStore::validate_backup(&staging))
        .and_then(|()| GlossaryStore::validate_backup(&staging))
    {
        let _ = fs::remove_dir_all(&staging);
        return Err(format!("backup data is invalid: {error}"));
    }
    Ok(StagedBackup {
        directory: staging,
        manifest,
    })
}

pub fn install_staged_backup(data_dir: &Path, staging: &Path) -> Result<InstalledBackup, String> {
    fs::create_dir_all(data_dir)
        .map_err(|error| format!("failed to create runtime data directory: {error}"))?;
    let rollback = std::env::temp_dir().join(format!(
        "linguaray-rollback-{}-{}",
        std::process::id(),
        now_nanos()?
    ));
    fs::create_dir(&rollback)
        .map_err(|error| format!("failed to create restore rollback directory: {error}"))?;

    let mut saved_names = Vec::new();
    let mut installed_names = Vec::new();
    let install = (|| {
        for name in DATA_TARGETS {
            let current = data_dir.join(name);
            if current.exists() {
                fs::rename(&current, rollback.join(name)).map_err(|error| {
                    format!("failed to stage existing `{}`: {error}", current.display())
                })?;
                saved_names.push(name);
            }
        }
        for name in DATA_TARGETS {
            let source = staging.join(name);
            if source.exists() {
                fs::rename(&source, data_dir.join(name)).map_err(|error| {
                    format!("failed to restore `{}`: {error}", source.display())
                })?;
                installed_names.push(name);
            }
        }
        Ok::<(), String>(())
    })();

    if let Err(error) = install {
        for name in installed_names {
            let installed = data_dir.join(name);
            if installed.is_dir() {
                let _ = fs::remove_dir_all(&installed);
            } else if installed.exists() {
                let _ = fs::remove_file(&installed);
            }
        }
        for name in saved_names.into_iter().rev() {
            let saved = rollback.join(name);
            if saved.exists() {
                let _ = fs::rename(saved, data_dir.join(name));
            }
        }
        let _ = fs::remove_dir_all(&rollback);
        return Err(error);
    }
    Ok(InstalledBackup {
        data_dir: data_dir.to_path_buf(),
        rollback,
        saved_names,
        installed_names,
        committed: false,
    })
}

pub fn discard_staging(staging: &Path) {
    let _ = fs::remove_dir_all(staging);
}

fn write_file(
    archive: &mut ZipWriter<File>,
    source: &Path,
    archive_name: &str,
    options: SimpleFileOptions,
) -> Result<(), String> {
    let metadata = fs::symlink_metadata(source)
        .map_err(|error| format!("failed to inspect `{}`: {error}", source.display()))?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(format!(
            "refusing to archive non-regular file `{}`",
            source.display()
        ));
    }
    archive
        .start_file(archive_name, options)
        .map_err(|error| format!("failed to add `{archive_name}` to backup: {error}"))?;
    let mut source_file = File::open(source)
        .map_err(|error| format!("failed to read `{}`: {error}", source.display()))?;
    std::io::copy(&mut source_file, archive)
        .map_err(|error| format!("failed to copy `{archive_name}` to backup: {error}"))?;
    Ok(())
}

fn allowed_entry(path: &Path, is_dir: bool) -> bool {
    let text = path.to_string_lossy().replace('\\', "/");
    if is_dir {
        return text == "glossary/" || text == "glossary";
    }
    if matches!(
        text.as_str(),
        "manifest.json" | "settings.json" | "history.json" | "vocabulary.json"
    ) {
        return true;
    }
    let Some(name) = text.strip_prefix("glossary/") else {
        return false;
    };
    !name.is_empty() && !name.contains('/') && name.ends_with(".json")
}

fn now_secs() -> Result<u64, String> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .map_err(|error| format!("system clock is before unix epoch: {error}"))
}

fn now_nanos() -> Result<u128, String> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .map_err(|error| format!("system clock is before unix epoch: {error}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temporary(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "linguaray-backup-test-{name}-{}-{}",
            std::process::id(),
            now_nanos().expect("clock")
        ))
    }

    #[test]
    fn round_trip_keeps_supported_runtime_files_and_excludes_secrets() {
        let root = temporary("round-trip");
        let data = root.join("data");
        fs::create_dir_all(data.join("glossary")).expect("create data");
        fs::write(data.join("settings.json"), r#"{"advanced":{}}"#).expect("write settings");
        fs::write(data.join("history.json"), r#"{"version":1,"entries":[]}"#)
            .expect("write history");
        fs::write(
            data.join("glossary").join("terms.json"),
            r#"{"id":"terms","name":"Terms","entries":[]}"#,
        )
        .expect("write glossary");
        let archive = root.join("backup.zip");

        let (_, count) = export_backup(&data, &archive).expect("export backup");
        assert_eq!(count, 4);
        let staged = stage_backup(&archive).expect("stage backup");
        assert!(!staged.manifest.includes_secrets);
        assert!(staged.directory.join("settings.json").is_file());
        assert!(staged.directory.join("history.json").is_file());
        assert!(staged.directory.join("glossary/terms.json").is_file());

        discard_staging(&staged.directory);
        fs::remove_dir_all(root).expect("remove test data");
    }

    #[test]
    fn rejects_paths_outside_the_backup_root() {
        let root = temporary("unsafe");
        fs::create_dir_all(&root).expect("create root");
        let archive_path = root.join("unsafe.zip");
        let file = File::create(&archive_path).expect("create archive");
        let mut archive = ZipWriter::new(file);
        archive
            .start_file("../settings.json", SimpleFileOptions::default())
            .expect("start file");
        archive.write_all(b"{}").expect("write file");
        archive.finish().expect("finish archive");

        assert!(stage_backup(&archive_path).is_err());
        fs::remove_dir_all(root).expect("remove test data");
    }

    #[test]
    fn rejects_semantically_invalid_persisted_data() {
        let root = temporary("invalid-data");
        let data = root.join("data");
        fs::create_dir_all(&data).expect("create data");
        fs::write(data.join("settings.json"), "{}").expect("write settings");
        fs::write(data.join("history.json"), r#"{"version":99,"entries":[]}"#)
            .expect("write history");
        let archive = root.join("backup.zip");
        export_backup(&data, &archive).expect("export backup");

        assert!(stage_backup(&archive).is_err());
        fs::remove_dir_all(root).expect("remove test data");
    }

    #[test]
    fn dropping_an_uncommitted_install_restores_the_previous_data() {
        let root = temporary("rollback");
        let data = root.join("data");
        let staging = root.join("staging");
        fs::create_dir_all(&data).expect("create data");
        fs::create_dir_all(&staging).expect("create staging");
        fs::write(data.join("settings.json"), "old").expect("write old data");
        fs::write(staging.join("settings.json"), "new").expect("write new data");

        let transaction = install_staged_backup(&data, &staging).expect("install staged backup");
        assert_eq!(
            fs::read_to_string(data.join("settings.json")).unwrap(),
            "new"
        );
        drop(transaction);
        assert_eq!(
            fs::read_to_string(data.join("settings.json")).unwrap(),
            "old"
        );

        fs::remove_dir_all(root).expect("remove test data");
    }
}
