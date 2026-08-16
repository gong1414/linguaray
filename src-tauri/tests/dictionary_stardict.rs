use flate2::write::GzEncoder;
use flate2::Compression;
use linguaray_lib::dict::stardict::StarDictParser;
use std::io::Write;

fn write_ifo(dir: &std::path::Path, bookname: &str, word_count: usize) {
    std::fs::write(
        dir.join("test.ifo"),
        format!(
            "StarDict's dict ifo file\nversion=2.4.2\nbookname={bookname}\nwordcount={word_count}\nidxfilesize=42\nsametypesequence=m\n"
        ),
    )
    .unwrap();
}

/// Build a minimal StarDict package. Definitions use the `m` type marker.
fn build_test_dict(dir: &std::path::Path) {
    write_ifo(dir, "Test Dict", 3);
    let hello = b"mhello world\0";
    let goodbye = b"mgoodbye world\0";
    let test = b"mtest definition\0";
    let mut dict_data = Vec::new();
    dict_data.extend_from_slice(hello);
    dict_data.extend_from_slice(goodbye);
    dict_data.extend_from_slice(test);
    std::fs::write(dir.join("test.dict"), &dict_data).unwrap();

    let mut idx = Vec::new();
    idx.extend_from_slice(b"goodbye\x00");
    idx.extend_from_slice(&13u32.to_be_bytes());
    idx.extend_from_slice(&15u32.to_be_bytes());
    idx.extend_from_slice(b"hello\x00");
    idx.extend_from_slice(&0u32.to_be_bytes());
    idx.extend_from_slice(&13u32.to_be_bytes());
    idx.extend_from_slice(b"test\x00");
    idx.extend_from_slice(&28u32.to_be_bytes());
    idx.extend_from_slice(&17u32.to_be_bytes());
    std::fs::write(dir.join("test.idx"), &idx).unwrap();
}

#[test]
fn stardict_lookup_finds_existing_word() {
    let dir = tempfile::tempdir().unwrap();
    build_test_dict(dir.path());
    let parser = StarDictParser::open(dir.path()).unwrap();
    let result = parser.lookup("hello").unwrap();
    assert!(result.is_some());
    assert!(result.unwrap().contains("hello world"));
}

#[test]
fn stardict_lookup_returns_none_for_missing_word() {
    let dir = tempfile::tempdir().unwrap();
    build_test_dict(dir.path());
    let parser = StarDictParser::open(dir.path()).unwrap();
    let result = parser.lookup("nonexistent").unwrap();
    assert!(result.is_none());
}

#[test]
fn stardict_info_reads_bookname() {
    let dir = tempfile::tempdir().unwrap();
    build_test_dict(dir.path());
    let parser = StarDictParser::open(dir.path()).unwrap();
    assert_eq!(parser.info().bookname, "Test Dict");
    assert_eq!(parser.info().word_count, 3);
}

#[test]
fn stardict_open_missing_dir_returns_error() {
    let result = StarDictParser::open(std::path::Path::new("/nonexistent/path"));
    assert!(result.is_err());
}

#[test]
fn stardict_lookup_reads_gzip_dict_dz() {
    let dir = tempfile::tempdir().unwrap();
    write_ifo(dir.path(), "Gzip Dict", 1);
    let dict_data = b"mhello world\0";
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(dict_data).unwrap();
    std::fs::write(dir.path().join("test.dict.dz"), encoder.finish().unwrap()).unwrap();
    let mut idx = Vec::new();
    idx.extend_from_slice(b"hello\x00");
    idx.extend_from_slice(&0u32.to_be_bytes());
    idx.extend_from_slice(&(dict_data.len() as u32).to_be_bytes());
    std::fs::write(dir.path().join("test.idx"), &idx).unwrap();

    let parser = StarDictParser::open(dir.path()).unwrap();
    let result = parser.lookup("hello").unwrap().unwrap();
    assert!(result.contains("hello world"));
}
