use linguaray_lib::db::{schema, Database};
use linguaray_lib::history::export::{write_export_file, ExportFormat};
use linguaray_lib::history::search::DecryptedHistorySession;
use tempfile::TempDir;

#[test]
fn history_export_writes_caller_supplied_path() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("chosen-history.json");
    let sessions = vec![DecryptedHistorySession {
        session_uuid: "s1".into(),
        timestamp: 1,
        trigger_source: "input".into(),
        detected_language: None,
        target_language: "zh".into(),
        is_favorite: false,
        source_text: Some("hello".into()),
        results: vec![],
        corrupt: false,
    }];
    write_export_file(&sessions, &path, ExportFormat::Json).expect("write");
    let body = std::fs::read_to_string(&path).unwrap();
    assert!(body.contains("hello"));
}

#[test]
fn vocabulary_export_file_uses_supplied_path() {
    let dir = TempDir::new().unwrap();
    let db = Database::open(&dir.path().join("v.db")).unwrap();
    db.with_conn(|conn| {
        let tx = conn.transaction()?;
        schema::create_all_tables(&tx)?;
        schema::seed_singletons(&tx)?;
        tx.commit()?;
        Ok(())
    })
    .unwrap();
    let ks = linguaray_lib::keystore::Keystore::new(dir.path().join("ks")).unwrap();
    let path = dir.path().join("chosen-vocab.csv");
    let written =
        linguaray_lib::vocabulary::export_file(&db, &ks, path.to_str().unwrap(), "csv").unwrap();
    assert_eq!(std::path::Path::new(&written), path.as_path());
    assert!(std::fs::read_to_string(&path).unwrap().contains("word"));
}
