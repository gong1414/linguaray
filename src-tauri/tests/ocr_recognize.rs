use linguaray_lib::ocr::{self, fixture};

fn assert_hello_or_engine_unavailable(result: Result<ocr::OcrResult, ocr::OcrError>) {
    match result {
        Ok(recognized) => {
            let normalized = recognized
                .text
                .chars()
                .filter(|c| c.is_ascii_alphabetic())
                .collect::<String>()
                .to_ascii_uppercase();
            assert!(
                normalized.contains(fixture::FIXTURE_SUBSTRING),
                "ocr text {:?} (normalized {normalized}) must contain {}",
                recognized.text,
                fixture::FIXTURE_SUBSTRING
            );
        }
        Err(err) => {
            // macOS Vision is always present. Windows.Media.Ocr needs an OCR
            // language pack; GitHub windows-latest may not have one.
            #[cfg(target_os = "macos")]
            panic!("vision ocr: {err}");
            #[cfg(not(target_os = "macos"))]
            {
                let msg = err.to_string().to_ascii_lowercase();
                assert!(
                    msg.contains("unavailable")
                        || msg.contains("language")
                        || msg.contains("ocr"),
                    "unexpected ocr error: {err}"
                );
            }
        }
    }
}

#[test]
fn recognize_image_bytes_reads_hello_fixture() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/hello-ocr.png");
    fixture::write_hello_png(&path).unwrap();
    let bytes = std::fs::read(&path).unwrap();
    assert_hello_or_engine_unavailable(ocr::recognize_image_bytes(&bytes));
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
    assert_hello_or_engine_unavailable(ocr::recognize_rgba(w, h, decoded.as_raw()));
}
