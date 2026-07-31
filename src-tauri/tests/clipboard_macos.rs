//! Real NSPasteboard integration test for compound text+image restore.
//! Uses pasteboardWithUniqueName for isolation. macOS-only.
#![cfg(target_os = "macos")]

use islandpot_lib::clipboard::restore_compound_to;
use islandpot_lib::selection_engine::ImageBlob;
use objc2_app_kit::{NSPasteboard, NSPasteboardTypeString, NSPasteboardTypeTIFF};

#[test]
fn compound_restore_writes_both_text_and_tiff() {
    unsafe {
        // Create an isolated pasteboard (does NOT touch the system clipboard).
        let pb = NSPasteboard::pasteboardWithUniqueName();

        // 2×2 RGBA test image (all red: R=255,G=0,B=0,A=255 per pixel).
        let img = ImageBlob {
            width: 2,
            height: 2,
            bytes: vec![255, 0, 0, 255, 255, 0, 0, 255, 255, 0, 0, 255, 255, 0, 0, 255],
        };

        restore_compound_to(&pb, "hello world", &img).expect("compound restore should succeed");

        // Assert: exactly one item.
        let items = pb.pasteboardItems().expect("pasteboardItems should return");
        assert_eq!(items.count(), 1, "exactly one pasteboard item");

        // Assert: can read text.
        let text = pb.stringForType(NSPasteboardTypeString)
            .expect("stringForType should return");
        assert_eq!(text.to_string(), "hello world", "text flavor present");

        // Assert: can read TIFF data (non-empty).
        let tiff = pb.dataForType(NSPasteboardTypeTIFF)
            .expect("dataForType(TIFF) should return");
        assert!(!tiff.is_empty(), "TIFF data is non-empty");
    }
}

#[test]
fn compound_restore_invalid_rgba_does_not_modify_pasteboard() {
    unsafe {
        let pb = NSPasteboard::pasteboardWithUniqueName();
        // Write a marker so we can verify it survives a failed restore.
        pb.clearContents();
        let marker = objc2_foundation::NSString::from_str("marker-before");
        let marker_type = objc2_app_kit::NSPasteboardTypeString;
        pb.setString_forType(&marker, marker_type);

        // Invalid image: width*height*4 != bytes.len().
        let bad_img = ImageBlob {
            width: 3,
            height: 3,
            bytes: vec![0; 4], // should be 36, only 4
        };

        let result = restore_compound_to(&pb, "new-text", &bad_img);
        assert!(result.is_err(), "invalid RGBA should return error");

        // The pasteboard should be UNCHANGED — clearContents never ran.
        let text = pb.stringForType(NSPasteboardTypeString)
            .expect("marker should still be readable");
        assert_eq!(
            text.to_string(), "marker-before",
            "pasteboard unchanged after failed conversion (clearContents not called)"
        );
    }
}
