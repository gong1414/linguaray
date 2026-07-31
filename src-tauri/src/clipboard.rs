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
/// then clearContents + writeObjects once — all-or-nothing (any conversion failure
/// returns an error BEFORE touching the system pasteboard).
#[cfg(target_os = "macos")]
pub fn restore_snapshot(
    text: Option<&str>,
    image: Option<&crate::selection_engine::ImageBlob>,
) -> std::result::Result<(), String> {
    use objc2_app_kit::{
        NSBitmapImageRep, NSPasteboard, NSPasteboardItem,
        NSPasteboardTypeString, NSPasteboardTypeTIFF,
    };
    use objc2_foundation::{NSArray, NSString};
    use objc2::rc::Retained;
    use objc2::AnyThread;

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
        (Some(t), Some(img)) => unsafe {
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
            let color_space = objc2_foundation::NSString::from_str("NSCalibratedRGBColorSpace");
            let rep = NSBitmapImageRep::initWithBitmapDataPlanes_pixelsWide_pixelsHigh_bitsPerSample_samplesPerPixel_hasAlpha_isPlanar_colorSpaceName_bytesPerRow_bitsPerPixel(
                NSBitmapImageRep::alloc(),  // from Allocated trait
                std::ptr::null_mut(),
                w, h, 8, 4, true, false,
                &color_space,
                w * 4, 32,
            ).ok_or_else(|| "NSBitmapImageRep init failed".to_string())?;
            // Copy RGBA into the rep's bitmap data.
            let data_ptr = NSBitmapImageRep::bitmapData(&rep);
            if data_ptr.is_null() {
                return Err("NSBitmapImageRep bitmapData null".into());
            }
            std::ptr::copy_nonoverlapping(img.bytes.as_ptr(), data_ptr, img.bytes.len());
            // Get TIFF representation.
            let tiff_data = NSBitmapImageRep::TIFFRepresentation(&rep)
                .ok_or_else(|| "TIFF conversion failed".to_string())?;

            // Step 3: build the NSPasteboardItem with BOTH types (BEFORE touching
            // the pasteboard — all-or-nothing; any failure returns an error).
            let item = NSPasteboardItem::new();
            let text_ns = NSString::from_str(t);
            if !item.setString_forType(&text_ns, NSPasteboardTypeString) {
                return Err("setString_forType failed".into());
            }
            if !item.setData_forType(&tiff_data, NSPasteboardTypeTIFF) {
                return Err("setData_forType(TIFF) failed".into());
            }

            // Step 4-6: all conversions succeeded — now clearContents + writeObjects.
            let pb = NSPasteboard::generalPasteboard();
            pb.clearContents();
            // writeObjects takes NSArray<ProtocolObject<dyn NSPasteboardWriting>>.
            let writing_item: Retained<objc2::runtime::ProtocolObject<dyn objc2_app_kit::NSPasteboardWriting>> =
                objc2::runtime::ProtocolObject::from_retained(item);
            let items = NSArray::arrayWithObject(&*writing_item);
            if !pb.writeObjects(&items) {
                return Err("writeObjects failed".into());
            }
            Ok(())
        },
        (None, None) => Ok(()),
    }
}

#[cfg(not(target_os = "macos"))]
pub fn restore_snapshot(
    text: Option<&str>,
    image: Option<&crate::selection_engine::ImageBlob>,
) -> std::result::Result<(), String> {
    // Non-macOS: single-flavor via arboard (sequential; both-present loses one —
    // documented P2). Windows multi-format write is a P2 follow-up (Win32
    // OpenClipboard/EmptyClipboard + SetClipboardData for CF_UNICODETEXT + CF_DIB).
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
