//! Thin OS clipboard abstraction. `arboard` for get/set text; a per-platform
//! sequence number (Win: GetClipboardSequenceNumber; macOS: NSPasteboard.changeCount).
//! The sequence number is load-bearing for the §B restore guard.
use std::sync::Mutex;

// arboard::Clipboard is not safe to share raw across threads; guard it.
static CLIP: Mutex<Option<arboard::Clipboard>> = Mutex::new(None);

fn clip() -> std::result::Result<std::sync::MutexGuard<'static, Option<arboard::Clipboard>>, String> {
    let mut g = CLIP.lock().map_err(|e| e.to_string())?;
    if g.is_none() {
        *g = Some(arboard::Clipboard::new().map_err(|e| e.to_string())?);
    }
    Ok(g)
}

pub fn get_text() -> std::result::Result<String, String> {
    let mut g = clip()?;
    g.as_mut().unwrap().get_text().map_err(|e| e.to_string())
}

pub fn set_text(s: &str) -> std::result::Result<(), String> {
    let mut g = clip()?;
    g.as_mut().unwrap().set_text(s).map_err(|e| e.to_string())
}

/// Get the clipboard image (RGBA), if any. None if no image / unsupported.
pub fn get_image() -> std::result::Result<Option<crate::selection_engine::ImageBlob>, String> {
    let mut g = clip()?;
    match g.as_mut().unwrap().get_image() {
        Ok(img) => Ok(Some(crate::selection_engine::ImageBlob {
            width: img.width,
            height: img.height,
            bytes: img.bytes.into_owned(),
        })),
        Err(_) => Ok(None), // best-effort: no image / read error → None
    }
}

/// Set the clipboard image (RGBA). Best-effort.
pub fn set_image(img: &crate::selection_engine::ImageBlob) -> std::result::Result<(), String> {
    let mut g = clip()?;
    let data = arboard::ImageData {
        width: img.width,
        height: img.height,
        bytes: std::borrow::Cow::Borrowed(&img.bytes),
    };
    g.as_mut().unwrap().set_image(data).map_err(|e| e.to_string())
}

/// Restore BOTH text and image in a SINGLE platform-level write (round-3 review
/// P1). macOS: build one NSPasteboardItem carrying BOTH NSString + TIFF types,
/// then clearContents + writeObjects once. "All-or-nothing" here means a
/// CONVERSION-PHASE failure (RGBA→TIFF, setString/setData) returns Err BEFORE
/// clearContents is ever called — the pasteboard is untouched. clearContents and
/// writeObjects are NOT themselves transactional: if writeObjects returns false
/// after clearContents the clipboard is left empty. In practice writeObjects only
/// fails on invalid item shape, which the pre-checks above rule out.
#[cfg(target_os = "macos")]
pub fn restore_snapshot(
    text: Option<&str>,
    image: Option<&crate::selection_engine::ImageBlob>,
) -> std::result::Result<(), String> {
    use objc2_app_kit::NSPasteboard;

    match (text, image) {
        // Single-flavor: arboard is fine (only one write, no clear conflict).
        (Some(t), None) => {
            let mut g = clip()?;
            g.as_mut().unwrap().set_text(t).map_err(|e| e.to_string())
        }
        (None, Some(img)) => {
            let mut g = clip()?;
            let data = arboard::ImageData {
                width: img.width, height: img.height,
                bytes: std::borrow::Cow::Borrowed(&img.bytes),
            };
            g.as_mut().unwrap().set_image(data).map_err(|e| e.to_string())
        }
        (Some(t), Some(img)) => {
            // Delegate to the testable inner fn with the system pasteboard.
            let pb = NSPasteboard::generalPasteboard();
            restore_compound_to(&pb, t, img)
        }
        (None, None) => Ok(()),
    }
}

/// Inner: compound text+image write to a GIVEN NSPasteboard (testable with
/// `pasteboardWithUniqueName`). All conversions happen BEFORE clearContents; on
/// any error the pasteboard is untouched (returns Err at the failing step).
///
/// This is a SAFE function: the only FFI invariants are "the pasteboard pointer is
/// a valid NSPasteboard" (guaranteed by the `&NSPasteboard` borrow) and the
/// `NSBitmapImageRep::alloc()` + init calls, which are wrapped in their own
/// `unsafe` blocks below. NSPasteboard / NSBitmapImageRep are NOT UI objects and
/// are not main-thread-only, so this is safe to call from the async-runtime worker
/// thread that runs the hotkey capture path.
#[cfg(target_os = "macos")]
fn restore_compound_to(
    pb: &objc2_app_kit::NSPasteboard,
    text: &str,
    img: &crate::selection_engine::ImageBlob,
) -> std::result::Result<(), String> {
    use objc2_app_kit::{
        NSBitmapImageRep, NSPasteboardItem,
        NSPasteboardTypeString, NSPasteboardTypeTIFF,
    };
    use objc2_foundation::{NSArray, NSString};
    use objc2::rc::Retained;
    use objc2::AnyThread;

    // Step 1: validate dimensions + overflow.
    let expected = img.width.checked_mul(img.height)
        .and_then(|n| n.checked_mul(4))
        .ok_or_else(|| "image dimensions overflow".to_string())?;
    if img.bytes.len() != expected {
        return Err(format!(
            "image bytes {} != width*height*4 ({})", img.bytes.len(), expected
        ));
    }

    // Step 2: convert RGBA → TIFF NSData via NSBitmapImageRep.
    let w = img.width as isize;
    let h = img.height as isize;
    let color_space = NSString::from_str("NSCalibratedRGBColorSpace");
    // Safety: this is the designated initializer for NSBitmapImageRep; `planes`
    // is null (the rep allocates its own buffer), dims/strides are validated
    // above, and the color-space name is a statically-known constant. objc2
    // marks the init method `pub unsafe fn` because a wrong pointer/plane layout
    // would be UB; the contract here is honored.
    let rep = unsafe {
        NSBitmapImageRep::initWithBitmapDataPlanes_pixelsWide_pixelsHigh_bitsPerSample_samplesPerPixel_hasAlpha_isPlanar_colorSpaceName_bytesPerRow_bitsPerPixel(
            NSBitmapImageRep::alloc(),
            std::ptr::null_mut(),
            w, h, 8, 4, true, false,
            &color_space,
            w * 4, 32,
        )
    }.ok_or_else(|| "NSBitmapImageRep init failed".to_string())?;
    let data_ptr = NSBitmapImageRep::bitmapData(&rep);
    if data_ptr.is_null() {
        return Err("NSBitmapImageRep bitmapData null".into());
    }
    // Safety: data_ptr points to a w*h*4-byte buffer owned by `rep` (verified
    // non-null above; len matches the validated dimensions); `rep` outlives the
    // copy. The source slice is a valid &[u8] of the same length.
    unsafe {
        std::ptr::copy_nonoverlapping(img.bytes.as_ptr(), data_ptr, img.bytes.len());
    }
    let tiff_data = NSBitmapImageRep::TIFFRepresentation(&rep)
        .ok_or_else(|| "TIFF conversion failed".to_string())?;

    // Step 3: build the NSPasteboardItem with BOTH types (BEFORE touching pb).
    // Safety: NSPasteboardTypeString / NSPasteboardTypeTIFF are immutable
    // framework-provided NSString constants (extern statics); reading them is the
    // documented way to obtain these UTIs and cannot alias or race.
    let item = NSPasteboardItem::new();
    let text_ns = NSString::from_str(text);
    let str_type = unsafe { NSPasteboardTypeString };
    let tiff_type = unsafe { NSPasteboardTypeTIFF };
    if !item.setString_forType(&text_ns, str_type) {
        return Err("setString_forType failed".into());
    }
    if !item.setData_forType(&tiff_data, tiff_type) {
        return Err("setData_forType(TIFF) failed".into());
    }

    // Step 4-5: all conversions succeeded — now clearContents + writeObjects.
    pb.clearContents();
    let writing_item: Retained<objc2::runtime::ProtocolObject<dyn objc2_app_kit::NSPasteboardWriting>> =
        objc2::runtime::ProtocolObject::from_retained(item);
    let items = NSArray::arrayWithObject(&*writing_item);
    if !pb.writeObjects(&items) {
        return Err("writeObjects failed".into());
    }
    Ok(())
}

#[cfg(not(target_os = "macos"))]
pub fn restore_snapshot(
    text: Option<&str>,
    image: Option<&crate::selection_engine::ImageBlob>,
) -> std::result::Result<(), String> {
    // Non-macOS non-Windows: single-flavor via arboard (sequential; both-present
    // loses one — documented P2, no supported target hits this). Windows gets its
    // own compound impl in Phase 4 Task 2b (Win32 OpenClipboard/EmptyClipboard +
    // SetClipboardData for CF_UNICODETEXT + CF_DIBV5); until then this cfg-arm is
    // unreachable on supported targets (Win+macOS only).
    let mut g = clip()?;
    let c = g.as_mut().unwrap();
    if let Some(img) = image {
        let data = arboard::ImageData {
            width: img.width, height: img.height,
            bytes: std::borrow::Cow::Borrowed(&img.bytes),
        };
        let _ = c.set_image(data);
    }
    if let Some(t) = text {
        let _ = c.set_text(t);
    }
    Ok(())
}

/// Monotonic clipboard sequence number (advances on any clipboard write, ours
/// included). macOS: NSPasteboard.changeCount; Windows: GetClipboardSequenceNumber.
#[cfg(target_os = "macos")]
pub fn sequence() -> u64 {
    use objc2_app_kit::NSPasteboard;
    let pb = NSPasteboard::generalPasteboard();
    pb.changeCount() as u64
}

#[cfg(target_os = "windows")]
pub fn sequence() -> u64 {
    unsafe { windows_sys::Win32::System::DataExchange::GetClipboardSequenceNumber() as u64 }
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
pub fn sequence() -> u64 { 0 }

#[cfg(all(test, target_os = "macos"))]
mod tests {
    //! Real NSPasteboard integration tests for the compound text+image write.
    //! Use `pasteboardWithUniqueName` for isolation (does NOT touch the system
    //! clipboard). These live in-module so `restore_compound_to` can stay a safe
    //! private helper (round-6 review: don't widen the API surface just to test).
    use super::restore_compound_to;
    use crate::selection_engine::ImageBlob;
    use objc2_app_kit::{
        NSBitmapImageRep, NSPasteboard, NSPasteboardTypeString, NSPasteboardTypeTIFF,
    };
    use objc2_foundation::NSString;

    #[test]
    fn compound_restore_writes_both_text_and_tiff() {
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
        let text = pb.stringForType(unsafe { NSPasteboardTypeString })
            .expect("stringForType should return");
        assert_eq!(text.to_string(), "hello world", "text flavor present");

        // Assert: TIFF data round-trips to the expected 2×2 red image. Reading it
        // back via NSBitmapImageRep catches channel-order, stride, and alpha bugs
        // that a bare "non-empty" check would miss (round-6 review P2).
        let tiff = pb.dataForType(unsafe { NSPasteboardTypeTIFF })
            .expect("dataForType(TIFF) should return");
        assert!(!tiff.is_empty(), "TIFF data is non-empty");

        let rep = NSBitmapImageRep::imageRepWithData(&tiff)
            .expect("TIFF should decode back to a bitmap rep");
        assert_eq!(rep.pixelsWide(), 2, "decoded width");
        assert_eq!(rep.pixelsHigh(), 2, "decoded height");
        assert_eq!(rep.samplesPerPixel(), 4, "decoded samples per pixel (RGBA)");

        // Decode the first pixel. NSBitmapImageRep owns its buffer; bitmapData is a
        // raw pointer we must read in an unsafe block (verified non-null first).
        let px = NSBitmapImageRep::bitmapData(&rep);
        assert!(!px.is_null(), "decoded bitmapData non-null");
        let rgba = unsafe { std::slice::from_raw_parts(px, 4) };
        assert_eq!(rgba, &[255, 0, 0, 255], "first pixel is opaque red (R,G,B,A)");
    }

    #[test]
    fn compound_restore_invalid_rgba_does_not_modify_pasteboard() {
        let pb = NSPasteboard::pasteboardWithUniqueName();
        // Write a marker so we can verify it survives a failed restore.
        pb.clearContents();
        let marker = NSString::from_str("marker-before");
        let marker_type = unsafe { NSPasteboardTypeString };
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
        let text = pb.stringForType(unsafe { NSPasteboardTypeString })
            .expect("marker should still be readable");
        assert_eq!(
            text.to_string(), "marker-before",
            "pasteboard unchanged after failed conversion (clearContents not called)"
        );
    }
}
