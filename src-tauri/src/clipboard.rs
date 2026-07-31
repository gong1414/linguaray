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
/// after clearContents the pasteboard is left empty. (No guarantee is made that
/// writeObjects cannot fail; callers must treat a writeObjects failure as
/// "clipboard now empty", same as any other app clearing it.)
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
        // (None, None): the original clipboard was EMPTY (or held only unsupported
        // formats). The §B restore must return to that empty state — the sentinel we
        // wrote during capture is still on the pasteboard, so clearContents is required.
        // (round-11 review P1 #1: the prev `Ok(())` left the sentinel behind, violating §B.)
        // round-12 review P1 #1: delegate to a testable inner fn so the production path
        // and a real-pasteboard test share ONE implementation (a Fake-only test was a
        // false green — the Fake cleared (None,None) even before the prod fix).
        (None, None) => {
            let pb = NSPasteboard::generalPasteboard();
            restore_empty_to(&pb)
        }
    }
}

/// Inner: clear a GIVEN NSPasteboard (testable with `pasteboardWithUniqueName`).
/// Used by the (None, None) restore branch — an empty original snapshot must still
/// remove the §B sentinel. Returns Ok after clearContents (no payload to write).
#[cfg(target_os = "macos")]
fn restore_empty_to(pb: &objc2_app_kit::NSPasteboard) -> std::result::Result<(), String> {
    pb.clearContents();
    Ok(())
}

/// Inner: compound text+image write to a GIVEN NSPasteboard (testable with
/// `pasteboardWithUniqueName`). Preflight (dimension/range checks) and all TIFF
/// conversion happen BEFORE clearContents, so a preflight or conversion error
/// leaves the pasteboard untouched. This does NOT cover a `writeObjects` failure:
/// that runs AFTER clearContents, so the pasteboard would already be empty if it
/// returned false (no guarantee that writeObjects cannot fail — treat such a
/// failure as "pasteboard now empty").
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

    // Step 1: full FFI preflight. These values feed a designated initializer that
    // takes NSInteger (isize); a negative/overflowed dimension or stride would be
    // UB at the FFI boundary. The old check (usize checked_mul of width*height*4)
    // was insufficient: e.g. (width=usize::MAX, height=0, bytes=[]) passed it, then
    // `width as isize` wrapped negative and `w*4` overflowed. Reject zero dims and
    // use checked conversions for EVERY value that crosses into Cocoa.
    if img.width == 0 || img.height == 0 {
        return Err(format!("image has zero dimension ({}x{})", img.width, img.height));
    }
    // isize/NSInteger range — also bounds width so bytes_per_row can't overflow.
    let w = isize::try_from(img.width)
        .map_err(|_| format!("image width {} exceeds isize range", img.width))?;
    let h = isize::try_from(img.height)
        .map_err(|_| format!("image height {} exceeds isize range", img.height))?;
    let bytes_per_row = w.checked_mul(4)
        .ok_or_else(|| "row stride (width*4) overflows isize".to_string())?;
    // Total bytes, validated against the actual slice length. Use u64 to avoid any
    // platform-dependent usize overflow on the intermediate product.
    let total = (img.width as u64)
        .checked_mul(img.height as u64)
        .and_then(|n| n.checked_mul(4))
        .ok_or_else(|| "image byte count overflows u64".to_string())?;
    let expected = usize::try_from(total)
        .map_err(|_| "image byte count exceeds usize".to_string())?;
    if img.bytes.len() != expected {
        return Err(format!(
            "image bytes {} != width*height*4 ({})", img.bytes.len(), expected
        ));
    }

    // Step 2: convert RGBA → TIFF NSData via NSBitmapImageRep.
    let color_space = NSString::from_str("NSCalibratedRGBColorSpace");
    // Safety: this is the designated initializer for NSBitmapImageRep; `planes`
    // is null (the rep allocates its own buffer), dims/strides are validated and
    // in-range above, and the color-space name is a statically-known constant.
    // objc2 marks the init method `pub unsafe fn` because a wrong pointer/plane
    // layout would be UB; the contract here is honored.
    let rep = unsafe {
        NSBitmapImageRep::initWithBitmapDataPlanes_pixelsWide_pixelsHigh_bitsPerSample_samplesPerPixel_hasAlpha_isPlanar_colorSpaceName_bytesPerRow_bitsPerPixel(
            NSBitmapImageRep::alloc(),
            std::ptr::null_mut(),
            w, h, 8, 4, true, false,
            &color_space,
            bytes_per_row, 32,
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
    // Non-macOS (this cfg-arm covers Windows AND any unsupported target). Windows
    // gets its own compound impl in Phase 4 Task 2b (Win32 OpenClipboard/
    // EmptyClipboard + SetClipboardData for CF_UNICODETEXT + CF_DIBV5). Until that
    // task lands, Windows falls through to THIS sequential-arboard path, which
    // loses one flavor when both text+image are present (documented limitation).
    let mut g = clip()?;
    let c = g.as_mut().unwrap();
    // (None, None): original clipboard was empty/unsupported — clear to remove the
    // §B sentinel (round-11 review P1 #1). Task 2b's Windows FSM models this as the
    // zero-format case (OpenClipboard → EmptyClipboard → CloseClipboard).
    if text.is_none() && image.is_none() {
        return c.clear().map_err(|e| e.to_string());
    }
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
        // back via NSBitmapImageRep catches channel-order and alpha bugs that a
        // bare "non-empty" check would miss (round-6 review P2). It does NOT by
        // itself prove row stride / direction — the source is uniformly red, so a
        // wrong stride would still read red. All 4 pixels are checked (so a stride
        // bug that read garbage from padding would be caught), but a definitive
        // stride test needs a non-uniform image (deferred — not a round-7 blocker).
        let tiff = pb.dataForType(unsafe { NSPasteboardTypeTIFF })
            .expect("dataForType(TIFF) should return");
        assert!(!tiff.is_empty(), "TIFF data is non-empty");

        let rep = NSBitmapImageRep::imageRepWithData(&tiff)
            .expect("TIFF should decode back to a bitmap rep");
        assert_eq!(rep.pixelsWide(), 2, "decoded width");
        assert_eq!(rep.pixelsHigh(), 2, "decoded height");
        assert_eq!(rep.samplesPerPixel(), 4, "decoded samples per pixel (RGBA)");

        // Decode ALL 4 pixels. NSBitmapImageRep owns its buffer; bitmapData is a
        // raw pointer we must read in an unsafe block (verified non-null first).
        let px = NSBitmapImageRep::bitmapData(&rep);
        assert!(!px.is_null(), "decoded bitmapData non-null");
        let rgba = unsafe { std::slice::from_raw_parts(px, 16) };
        assert_eq!(
            rgba,
            &[255, 0, 0, 255, 255, 0, 0, 255, 255, 0, 0, 255, 255, 0, 0, 255],
            "all 4 pixels are opaque red (R,G,B,A) — channel order + reads across rows"
        );
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

    #[test]
    fn compound_restore_pathological_dimensions_do_not_touch_pasteboard() {
        // Round-7 review P1: the preflight must reject dimensions that would be UB
        // at the Cocoa FFI boundary (NSInteger/isize). The OLD check
        // (width.checked_mul(height).checked_mul(4) == bytes.len()) let
        // (usize::MAX, 0, []) through, then `width as isize` wrapped negative and
        // `w*4` overflowed — feeding garbage to the designated initializer.
        let pb = NSPasteboard::pasteboardWithUniqueName();
        pb.clearContents();
        let marker = NSString::from_str("marker-before");
        pb.setString_forType(&marker, unsafe { NSPasteboardTypeString });

        let check = |w: usize, h: usize, bytes: Vec<u8>, label: &str| {
            let img = ImageBlob { width: w, height: h, bytes };
            let res = restore_compound_to(&pb, "x", &img);
            assert!(res.is_err(), "{label}: expected preflight rejection");
            // Pasteboard untouched — the unsafe initializer never ran.
            let t = pb.stringForType(unsafe { NSPasteboardTypeString })
                .expect("marker readable");
            assert_eq!(t.to_string(), "marker-before", "{label}: pasteboard changed");
        };

        // The case that defeated the old check: huge width, zero height, empty bytes
        // (width*height*4 == 0 == bytes.len()). Must be rejected for zero dimension.
        check(usize::MAX, 0, vec![], "max-width × zero-height × empty");

        // Zero width, non-zero height.
        check(0, 4, vec![], "zero-width");

        // Width that fits in usize but overflows isize (NSInteger) on this platform.
        // On 64-bit, isize::MAX+1 as usize. (Skipped where usize==isize bits can't
        // represent it, but on 64-bit it always can.)
        let over_isize = (isize::MAX as usize).wrapping_add(1);
        if over_isize > isize::MAX as usize {
            check(over_isize, 1, vec![], "width overflows isize");
        }

        // Width whose row-stride (w*4) overflows isize, even though w itself fits.
        // w = isize::MAX/4 + 1 → w*4 > isize::MAX. Height 1, but bytes need not be
        // huge for the stride check to trip (it runs before the length check).
        let stride_overflow_w = (isize::MAX as usize / 4) + 1;
        check(stride_overflow_w, 1, vec![], "row stride overflows isize");
    }

    #[test]
    fn restore_empty_removes_sentinel_on_real_pasteboard() {
        // Round-12 review P1 #1: the empty-original regression test must exercise the
        // PRODUCTION clear path, not just the Fake. A Fake-only test was a false green
        // (the Fake cleared (None,None) even before the clipboard.rs fix). This test
        // drives the same `restore_empty_to` the production (None,None) branch calls,
        // against an isolated real NSPasteboard. Write a sentinel, clear via the helper,
        // assert the sentinel is gone — if the helper regressed to a no-op, this fails.
        let pb = NSPasteboard::pasteboardWithUniqueName();
        // Seed the pasteboard with a §B sentinel (what capture writes).
        pb.clearContents();
        let sentinel = NSString::from_str("__islandpot_sel_test__");
        pb.setString_forType(&sentinel, unsafe { NSPasteboardTypeString });
        // Confirm it's there before the restore.
        let before = pb.stringForType(unsafe { NSPasteboardTypeString })
            .expect("sentinel readable before restore");
        assert_eq!(before.to_string(), "__islandpot_sel_test__");

        // The production (None,None) path:
        super::restore_empty_to(&pb).expect("restore_empty_to should succeed");

        // After restore, the sentinel must be gone. clearContents removes the items, so
        // the String-type lookup returns None (no item carries that type anymore). If the
        // helper regressed to a no-op, stringForType would still return the sentinel.
        let after = pb.stringForType(unsafe { NSPasteboardTypeString });
        match after {
            Some(s) => assert!(
                s.to_string() != "__islandpot_sel_test__",
                "sentinel must be removed by restore_empty_to (still got {:?})",
                s.to_string()
            ),
            None => { /* expected: cleared pasteboard has no String-typed item */ }
        }
        assert_eq!(pb.pasteboardItems().map(|i| i.count()).unwrap_or(0), 0,
            "no items remain after clearing the empty snapshot");
    }
}
