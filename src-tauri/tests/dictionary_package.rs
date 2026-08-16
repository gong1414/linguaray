use linguaray_lib::db::{schema, Database};
use linguaray_lib::dict::package;
use std::fs::File;
use tempfile::TempDir;

struct Harness {
    _dir: TempDir,
    db: Database,
}

impl Harness {
    fn new() -> Self {
        let dir = tempfile::tempdir().unwrap();
        let db = Database::open(&dir.path().join("pkg.db")).unwrap();
        db.with_conn(|conn| {
            let tx = conn.transaction()?;
            schema::create_all_tables(&tx)?;
            schema::seed_singletons(&tx)?;
            tx.commit()?;
            Ok(())
        })
        .unwrap();
        Self { _dir: dir, db }
    }
}

fn build_valid_dict(dir: &std::path::Path) {
    std::fs::write(
        dir.join("test.ifo"),
        "StarDict's dict ifo file\nversion=2.4.2\nbookname=Test\nwordcount=1\nidxfilesize=10\nsametypesequence=m\n",
    )
    .unwrap();
    std::fs::write(dir.join("test.dict"), b"mhello\0").unwrap();
    std::fs::write(
        dir.join("test.idx"),
        b"hello\x00\x00\x00\x00\x00\x00\x00\x00\x07",
    )
    .unwrap();
}

#[test]
fn install_package_copies_files_and_registers_in_db() {
    let h = Harness::new();
    let src = tempfile::tempdir().unwrap();
    build_valid_dict(src.path());
    let dest_root = h._dir.path().join("dictionaries");
    package::install_package(
        &h.db,
        src.path(),
        &dest_root,
        "test-pkg",
        "Test Dict",
        "1.0",
    )
    .unwrap();
    assert!(dest_root.join("test-pkg/test.ifo").exists());
    let packages = h.db.with_conn(package::list_packages).unwrap();
    assert_eq!(packages.len(), 1);
    assert_eq!(packages[0].package_id, "test-pkg");
}

#[test]
fn install_package_rejects_path_traversal_in_package_id() {
    let h = Harness::new();
    let src = tempfile::tempdir().unwrap();
    build_valid_dict(src.path());
    let dest_root = h._dir.path().join("dictionaries");
    let result = package::install_package(
        &h.db,
        src.path(),
        &dest_root,
        "../../etc/evil",
        "Evil",
        "1.0",
    );
    assert!(result.is_err());
    assert!(!dest_root.join("../../etc/evil").exists());
}

#[test]
fn install_package_rejects_symlinks() {
    let h = Harness::new();
    let src = tempfile::tempdir().unwrap();
    build_valid_dict(src.path());
    create_forbidden_link(&src.path().join("evil.link")).expect("create link for installer to reject");
    let dest_root = h._dir.path().join("dictionaries");
    let result = package::install_package(
        &h.db,
        src.path(),
        &dest_root,
        "symlink-pkg",
        "Symlink",
        "1.0",
    );
    assert!(result.is_err());
}

fn create_forbidden_link(link: &std::path::Path) -> std::io::Result<()> {
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink("/etc/passwd", link)
    }
    #[cfg(windows)]
    {
        // Junctions do not need SeCreateSymbolicLinkPrivilege. The installer
        // must reject any reparse point, not only POSIX-style symlinks.
        let target = link.with_extension("junction-src");
        std::fs::create_dir_all(&target)?;
        let status = std::process::Command::new("cmd")
            .args([
                "/C",
                "mklink",
                "/J",
                &link.to_string_lossy(),
                &target.to_string_lossy(),
            ])
            .status()?;
        if status.success() {
            Ok(())
        } else {
            Err(std::io::Error::other("mklink /J failed"))
        }
    }
    #[cfg(not(any(unix, windows)))]
    {
        let _ = link;
        Err(std::io::Error::other("no symlink API on this host"))
    }
}

#[test]
fn install_package_rejects_bomb_too_large() {
    let h = Harness::new();
    let src = tempfile::tempdir().unwrap();
    std::fs::write(
        src.path().join("test.ifo"),
        "StarDict's dict ifo file\nversion=2.4.2\nbookname=Big\nwordcount=1\nidxfilesize=10\nsametypesequence=m\n",
    )
    .unwrap();
    let big_file = File::create(src.path().join("test.dict")).unwrap();
    big_file.set_len(600 * 1024 * 1024).unwrap();
    drop(big_file);
    std::fs::write(
        src.path().join("test.idx"),
        b"hello\x00\x00\x00\x00\x01\x00\x00\x00\x06",
    )
    .unwrap();
    let dest_root = h._dir.path().join("dictionaries");
    let result = package::install_package(&h.db, src.path(), &dest_root, "bomb-pkg", "Bomb", "1.0");
    assert!(result.is_err());
    assert!(!dest_root.join("bomb-pkg").exists());
    assert!(!dest_root.join("bomb-pkg.tmp").exists());
}

#[test]
fn install_package_rollback_on_db_error() {
    let h = Harness::new();
    let src = tempfile::tempdir().unwrap();
    build_valid_dict(src.path());
    let dest_root = h._dir.path().join("dictionaries");
    h.db.with_conn(|c| {
        c.execute(
            "INSERT INTO dict_packages (package_id, name, version, installed_at) VALUES ('dup', 'A', '1', 1)",
            [],
        )?;
        Ok(())
    })
    .unwrap();
    let result = package::install_package(&h.db, src.path(), &dest_root, "dup", "Dup", "1.0");
    assert!(result.is_err());
    assert!(!dest_root.join("dup.tmp").exists());
}
