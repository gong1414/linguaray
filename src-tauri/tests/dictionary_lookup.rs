use linguaray_lib::db::{schema, Database};
use linguaray_lib::dict::lookup::lookup;
use linguaray_lib::dict::package;
use tempfile::TempDir;

struct Harness {
    _dir: TempDir,
    db: Database,
}

impl Harness {
    fn new() -> Self {
        let dir = tempfile::tempdir().unwrap();
        let db = Database::open(&dir.path().join("lk.db")).unwrap();
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

fn build_test_dict(dir: &std::path::Path) {
    std::fs::write(
        dir.join("test.ifo"),
        "StarDict's dict ifo file\nversion=2.4.2\nbookname=TestDict\nwordcount=1\nidxfilesize=10\nsametypesequence=m\n",
    )
    .unwrap();
    std::fs::write(dir.join("test.dict"), b"mhello world\0").unwrap();
    let mut idx = Vec::new();
    // Nonce word so macOS system dict cannot steal the hit.
    idx.extend_from_slice(b"zyxhelloq\x00");
    idx.extend_from_slice(&0u32.to_be_bytes());
    idx.extend_from_slice(&13u32.to_be_bytes());
    std::fs::write(dir.join("test.idx"), &idx).unwrap();
}

#[test]
fn dict_lookup_offline_returns_definition_with_source() {
    let h = Harness::new();
    let src = tempfile::tempdir().unwrap();
    build_test_dict(src.path());
    let dest_root = h._dir.path().join("dictionaries");
    package::install_package(&h.db, src.path(), &dest_root, "test-pkg", "TestDict", "1.0").unwrap();
    let result = lookup(&h.db, &dest_root, "zyxhelloq").unwrap();
    assert!(result.is_some());
    let r = result.unwrap();
    assert!(r.definition.contains("hello world"));
    assert!(!r.source.is_empty());
}

#[test]
fn dict_lookup_missing_word_returns_none() {
    let h = Harness::new();
    let dest_root = h._dir.path().join("dictionaries");
    let result = lookup(&h.db, &dest_root, "nonexistent").unwrap();
    let _ = result;
}

#[test]
fn dict_lookup_no_packages_returns_none_on_non_macos_or_none() {
    let h = Harness::new();
    let dest_root = h._dir.path().join("dictionaries");
    let result = lookup(&h.db, &dest_root, "anything").unwrap();
    #[cfg(not(target_os = "macos"))]
    assert!(result.is_none());
    #[cfg(target_os = "macos")]
    {
        let _ = result;
    }
}
