use linguaray_lib::ocr::{self, fixture};

#[test]
fn recognize_image_bytes_reads_hello_fixture() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/hello-ocr.png");
    fixture::write_hello_png(&path).unwrap();
    let bytes = std::fs::read(&path).unwrap();
    let result = ocr::recognize_image_bytes(&bytes).expect("vision ocr");
    let normalized = result
        .text
        .chars()
        .filter(|c| c.is_ascii_alphabetic())
        .collect::<String>()
        .to_ascii_uppercase();
    assert!(
        normalized.contains(fixture::FIXTURE_SUBSTRING),
        "ocr text {:?} (normalized {normalized}) must contain {}",
        result.text,
        fixture::FIXTURE_SUBSTRING
    );
}

#[test]
fn recognize_empty_bytes_errors() {
    assert!(ocr::recognize_image_bytes(&[]).is_err());
}

#[test]
fn recognize_rgba_roundtrip_from_fixture_png() {
    let png = fixture::hello_png_bytes();
    let decoded = image::load_from_memory(&png).unwrap().to_rgba8();
    let (w, h) = decoded.dimensions();
    let result = ocr::recognize_rgba(w, h, decoded.as_raw()).expect("rgba ocr");
    let normalized = result
        .text
        .chars()
        .filter(|c| c.is_ascii_alphabetic())
        .collect::<String>()
        .to_ascii_uppercase();
    assert!(
        normalized.contains(fixture::FIXTURE_SUBSTRING),
        "{:?}",
        result.text
    );
}
