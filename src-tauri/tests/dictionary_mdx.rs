use linguaray_lib::dict::mdx::{write_minimal_fixture, MdxParser};

fn sample_mdx_path() -> std::path::PathBuf {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/sample.mdx");
    if !path.exists() {
        write_minimal_fixture(&path, &[("test", "a definition")]).unwrap();
    }
    path
}

#[test]
fn mdx_open_and_lookup_returns_definition() {
    let path = sample_mdx_path();
    let mut parser = MdxParser::open(&path).unwrap();
    let result = parser.lookup("test").unwrap();
    assert!(result.is_some());
    assert!(result.unwrap().contains("definition"));
}

#[test]
fn mdx_lookup_missing_returns_none() {
    let path = sample_mdx_path();
    let mut parser = MdxParser::open(&path).unwrap();
    let result = parser.lookup("nonexistent12345").unwrap();
    assert!(result.is_none());
}

#[test]
fn mdx_open_invalid_file_returns_error() {
    let result = MdxParser::open(std::path::Path::new("/nonexistent/file.mdx"));
    assert!(result.is_err());
}

#[test]
fn mdx_open_rejects_bad_magic() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("bad.mdx");
    std::fs::write(&path, b"NOTM").unwrap();
    assert!(MdxParser::open(&path).is_err());
}
