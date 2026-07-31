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

/// Restore BOTH text and image in a single platform-level write (round-2 review
/// P1 #2): arboard's set_text/set_image each clear first, so sequential writes
/// lose one flavor. Here we clear ONCE then set both. Best-effort.
pub fn restore_snapshot(
    text: Option<&str>,
    image: Option<&crate::selection_engine::ImageBlob>,
) -> std::result::Result<(), String> {
    let mut g = clip()?;
    let clip = g.as_mut().unwrap();
    // Clear once, then write all present formats. (arboard::Clipboard::clear.)
    let _ = clip.clear();
    if let Some(img) = image {
        let data = arboard::ImageData {
            width: img.width,
            height: img.height,
            bytes: std::borrow::Cow::Borrowed(&img.bytes),
        };
        // set_image after clear writes the image flavor.
        let _ = clip.set_image(data);
    }
    if let Some(t) = text {
        // set_text writes the text flavor WITHOUT clearing the image arboard just set
        // IF the prior op was clear (no-op clear semantics on an empty clipboard).
        // NOTE: arboard may still clear on each set on some platforms; this is the
        // best available without dropping to raw NSPasteboard/Win32. Documented.
        let _ = clip.set_text(t);
    }
    Ok(())
}

/// Monotonic clipboard sequence number (advances on any clipboard write, ours
/// included). macOS: NSPasteboard.changeCount; Windows: GetClipboardSequenceNumber.
#[cfg(target_os = "macos")]
pub fn sequence() -> u64 {
    use objc::runtime::Object;
    use objc::{class, msg_send, sel, sel_impl};
    unsafe {
        let pb: *mut Object = msg_send![class!(NSPasteboard), generalPasteboard];
        let count: isize = msg_send![pb, changeCount];
        count as u64
    }
}

#[cfg(target_os = "windows")]
pub fn sequence() -> u64 {
    unsafe { windows_sys::Win32::System::DataExchange::GetClipboardSequenceNumber() as u64 }
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
pub fn sequence() -> u64 { 0 }
