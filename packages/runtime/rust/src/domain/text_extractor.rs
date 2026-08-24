// ── Screen Text Extraction ─────────────────────────────────────────────────
//
// Platform-specific implementations for reading clipboard text and extracting
// text from screen selection (by simulating Copy and watching the clipboard).
//
// macOS:  objc2 (NSPasteboard) + CoreGraphics (CGEvent) + ApplicationServices (AX)
// Windows: extern "system" (user32: SendInput, clipboard APIs)
//
// ── Error ──────────────────────────────────────────────────────────────────

use std::fmt;

use base64::Engine as _;

#[derive(Clone, Debug, uniffi::Record)]
pub struct SelectionExtraction {
    pub text: String,
    pub read_method: String,
    /// A non-fatal warning, most commonly a clipboard snapshot/restore issue.
    pub recoverable_error: Option<String>,
}

#[derive(Debug, Clone)]
pub enum TextExtractorError {
    /// Accessibility permission not granted (macOS only).
    AccessibilityDenied,
    /// No text was selected or clipboard was empty.
    NoTextSelected,
    /// This operation is not supported on the current platform.
    UnsupportedMethod(String),
    /// General operation failure.
    OperationFailed(String),
}

impl fmt::Display for TextExtractorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AccessibilityDenied => write!(f, "accessibility permission denied"),
            Self::NoTextSelected => write!(f, "no text selected"),
            Self::UnsupportedMethod(msg) => write!(f, "unsupported method: {msg}"),
            Self::OperationFailed(msg) => write!(f, "operation failed: {msg}"),
        }
    }
}

impl std::error::Error for TextExtractorError {}

/// Returns the current clipboard image as a base64-encoded PNG. The public
/// OCR boundary deals in ordinary encoded images rather than platform-native
/// BGRA/RGBA buffers.
#[cfg(any(target_os = "macos", target_os = "windows"))]
pub fn extract_image_from_clipboard() -> Result<String, TextExtractorError> {
    let mut clipboard = arboard::Clipboard::new()
        .map_err(|error| TextExtractorError::OperationFailed(error.to_string()))?;
    let image = clipboard
        .get_image()
        .map_err(|error| TextExtractorError::OperationFailed(error.to_string()))?;
    encode_rgba_png(image.width, image.height, image.bytes.as_ref())
}

fn encode_rgba_png(
    width: usize,
    height: usize,
    bytes: &[u8],
) -> Result<String, TextExtractorError> {
    let expected = width.saturating_mul(height).saturating_mul(4);
    if width == 0 || height == 0 || bytes.len() != expected {
        return Err(TextExtractorError::OperationFailed(
            "clipboard image has invalid RGBA dimensions".to_owned(),
        ));
    }
    let mut encoded = Vec::new();
    {
        let mut encoder = png::Encoder::new(&mut encoded, width as u32, height as u32);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        let mut writer = encoder
            .write_header()
            .map_err(|error| TextExtractorError::OperationFailed(error.to_string()))?;
        writer
            .write_image_data(bytes)
            .map_err(|error| TextExtractorError::OperationFailed(error.to_string()))?;
    }
    Ok(base64::engine::general_purpose::STANDARD.encode(encoded))
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
pub fn extract_image_from_clipboard() -> Result<String, TextExtractorError> {
    Err(TextExtractorError::UnsupportedMethod(
        "clipboard image OCR is supported on macOS and Windows".to_owned(),
    ))
}

// ═══════════════════════════════════════════════════════════════════════════
//  macOS
// ═══════════════════════════════════════════════════════════════════════════

#[cfg(target_os = "macos")]
mod platform {
    use std::borrow::Cow;
    use std::ffi::{c_char, c_void, CStr, CString};
    use std::ptr::null_mut;
    use std::thread;
    use std::time::{Duration, Instant};

    use objc2::{msg_send, runtime::AnyObject};

    use super::{SelectionExtraction, TextExtractorError};

    struct ClipboardContents {
        text: Option<String>,
        image: Option<(usize, usize, Vec<u8>)>,
    }

    fn snapshot_clipboard_contents() -> Result<ClipboardContents, String> {
        let mut clipboard = arboard::Clipboard::new().map_err(|error| error.to_string())?;
        let text = clipboard.get_text().ok();
        let image = if text.is_none() {
            clipboard
                .get_image()
                .ok()
                .map(|image| (image.width, image.height, image.bytes.into_owned()))
        } else {
            None
        };
        Ok(ClipboardContents { text, image })
    }

    fn restore_clipboard_contents(contents: ClipboardContents) -> Result<(), String> {
        let mut clipboard = arboard::Clipboard::new().map_err(|error| error.to_string())?;
        if let Some(text) = contents.text {
            return clipboard.set_text(text).map_err(|error| error.to_string());
        }
        if let Some((width, height, bytes)) = contents.image {
            return clipboard
                .set_image(arboard::ImageData {
                    width,
                    height,
                    bytes: Cow::Owned(bytes),
                })
                .map_err(|error| error.to_string());
        }
        clipboard.clear().map_err(|error| error.to_string())
    }

    // ── Helper: &str → NSString ────────────────────────────────────────

    unsafe fn ns_string(s: &str) -> *mut AnyObject {
        let c = CString::new(s).unwrap();
        msg_send![objc2::class!(NSString), stringWithUTF8String: c.as_ptr()]
    }

    unsafe fn rust_string(ns: *mut AnyObject) -> String {
        if ns.is_null() {
            return String::new();
        }
        let c: *const c_char = msg_send![ns, UTF8String];
        if c.is_null() {
            String::new()
        } else {
            CStr::from_ptr(c).to_string_lossy().into_owned()
        }
    }

    // ── CoreGraphics / ApplicationServices C FFI ────────────────────────

    #[link(name = "CoreGraphics", kind = "framework")]
    extern "C" {
        fn CGEventCreateKeyboardEvent(
            source: *const c_void,
            keycode: u16,
            keydown: bool,
        ) -> *mut c_void;
        fn CGEventSetFlags(event: *mut c_void, flags: u64);
        fn CGEventPost(tap: u32, event: *mut c_void);
        fn CFRelease(event: *mut c_void);
    }

    // ── Constants ───────────────────────────────────────────────────────

    /// kCGHIDEventTap
    const CG_HID_EVENT_TAP: u32 = 0;
    /// kVK_ANSI_C
    const KVK_ANSI_C: u16 = 0x08;
    /// kCGEventFlagMaskCommand
    const CG_EVENT_FLAG_COMMAND: u64 = 1 << 20;

    // ── Public API ──────────────────────────────────────────────────────

    /// Read current clipboard text from NSPasteboard.
    pub fn read_clipboard_text() -> Result<String, TextExtractorError> {
        unsafe {
            let pasteboard: *mut AnyObject =
                msg_send![objc2::class!(NSPasteboard), generalPasteboard];
            let text: *mut AnyObject = msg_send![
                pasteboard,
                stringForType: ns_string("public.utf8-plain-text")
            ];
            if text.is_null() {
                return Err(TextExtractorError::NoTextSelected);
            }
            let s = rust_string(text);
            if s.is_empty() {
                Err(TextExtractorError::NoTextSelected)
            } else {
                Ok(s)
            }
        }
    }

    /// Simulate Cmd+C key press using CGEvent.
    pub fn simulate_copy_key_press() -> Result<(), TextExtractorError> {
        unsafe {
            // Key down
            let down = CGEventCreateKeyboardEvent(null_mut(), KVK_ANSI_C, true);
            if down.is_null() {
                return Err(TextExtractorError::OperationFailed(
                    "failed to create CGEvent (key down)".into(),
                ));
            }
            CGEventSetFlags(down, CG_EVENT_FLAG_COMMAND);
            CGEventPost(CG_HID_EVENT_TAP, down);
            CFRelease(down);

            // Key up
            let up = CGEventCreateKeyboardEvent(null_mut(), KVK_ANSI_C, false);
            if up.is_null() {
                return Err(TextExtractorError::OperationFailed(
                    "failed to create CGEvent (key up)".into(),
                ));
            }
            CGEventSetFlags(up, CG_EVENT_FLAG_COMMAND);
            CGEventPost(CG_HID_EVENT_TAP, up);
            CFRelease(up);
        }
        Ok(())
    }

    /// Snapshot of the current pasteboard change count for change detection.
    pub struct ClipboardSnapshot {
        change_count: isize,
    }

    /// Take a snapshot of the current clipboard state.
    pub fn clipboard_snapshot() -> ClipboardSnapshot {
        unsafe {
            let pasteboard: *mut AnyObject =
                msg_send![objc2::class!(NSPasteboard), generalPasteboard];
            let change_count: isize = msg_send![pasteboard, changeCount];
            ClipboardSnapshot { change_count }
        }
    }

    /// Check if clipboard has changed since the snapshot.
    pub fn clipboard_changed(snapshot: &ClipboardSnapshot) -> bool {
        unsafe {
            let pasteboard: *mut AnyObject =
                msg_send![objc2::class!(NSPasteboard), generalPasteboard];
            let current: isize = msg_send![pasteboard, changeCount];
            current != snapshot.change_count
        }
    }

    /// Extract text from screen selection: simulate Cmd+C, poll for
    /// clipboard change, then read clipboard.
    pub fn extract_from_screen_selection() -> Result<SelectionExtraction, TextExtractorError> {
        if !crate::domain::permissions::is_accessibility_permission_granted() {
            return Err(TextExtractorError::AccessibilityDenied);
        }
        let (contents, snapshot_warning) = match snapshot_clipboard_contents() {
            Ok(contents) => (Some(contents), None),
            Err(error) => (None, Some(format!("failed to snapshot clipboard: {error}"))),
        };
        let snapshot = clipboard_snapshot();
        simulate_copy_key_press()?;

        let poll_interval = Duration::from_millis(50);
        let timeout = Duration::from_secs(3);
        let start = Instant::now();

        let result = loop {
            if clipboard_changed(&snapshot) {
                break read_clipboard_text();
            }
            if start.elapsed() >= timeout {
                break Err(TextExtractorError::NoTextSelected);
            }
            thread::sleep(poll_interval);
        };

        let restore_warning = contents.and_then(|contents| {
            restore_clipboard_contents(contents)
                .err()
                .map(|error| format!("failed to restore clipboard: {error}"))
        });
        result.map(|text| SelectionExtraction {
            text,
            read_method: "simulatedCopy".to_owned(),
            recoverable_error: snapshot_warning.or(restore_warning),
        })
    }

}

// ═══════════════════════════════════════════════════════════════════════════
//  Windows
// ═══════════════════════════════════════════════════════════════════════════

#[cfg(target_os = "windows")]
mod platform {
    use std::borrow::Cow;
    use std::ffi::c_void;
    use std::ptr::null_mut;
    use std::thread;
    use std::time::{Duration, Instant};

    use super::{SelectionExtraction, TextExtractorError};

    struct ClipboardContents {
        text: Option<String>,
        image: Option<(usize, usize, Vec<u8>)>,
    }

    fn snapshot_clipboard_contents() -> Result<ClipboardContents, String> {
        let mut clipboard = arboard::Clipboard::new().map_err(|error| error.to_string())?;
        let text = clipboard.get_text().ok();
        let image = if text.is_none() {
            clipboard
                .get_image()
                .ok()
                .map(|image| (image.width, image.height, image.bytes.into_owned()))
        } else {
            None
        };
        Ok(ClipboardContents { text, image })
    }

    fn restore_clipboard_contents(contents: ClipboardContents) -> Result<(), String> {
        let mut clipboard = arboard::Clipboard::new().map_err(|error| error.to_string())?;
        if let Some(text) = contents.text {
            return clipboard.set_text(text).map_err(|error| error.to_string());
        }
        if let Some((width, height, bytes)) = contents.image {
            return clipboard
                .set_image(arboard::ImageData {
                    width,
                    height,
                    bytes: Cow::Owned(bytes),
                })
                .map_err(|error| error.to_string());
        }
        clipboard.clear().map_err(|error| error.to_string())
    }

    // ── Win32 types ────────────────────────────────────────────────────

    #[repr(C)]
    struct INPUT {
        type_: u32,
        padding: [u8; 4],
        ki: KEYBDINPUT,
    }

    #[repr(C)]
    struct KEYBDINPUT {
        w_vk: u16,
        w_scan: u16,
        dw_flags: u32,
        time: u32,
        dw_extra_info: usize,
    }

    const INPUT_KEYBOARD: u32 = 1;
    const KEYEVENTF_KEYUP: u32 = 0x0002;
    const VK_CONTROL: u16 = 0x11;

    const CF_UNICODETEXT: u32 = 13;

    // ── user32 FFI ──────────────────────────────────────────────────────

    #[link(name = "user32")]
    extern "system" {
        fn SendInput(cInputs: u32, pInputs: *mut INPUT, cbSize: i32) -> u32;
        fn OpenClipboard(hWnd: *mut c_void) -> i32;
        fn CloseClipboard() -> i32;
        fn GetClipboardData(uFormat: u32) -> *mut c_void;
        fn GlobalLock(hMem: *mut c_void) -> *mut c_void;
        fn GlobalUnlock(hMem: *mut c_void) -> i32;
        fn GlobalSize(hMem: *mut c_void) -> usize;
        fn GetClipboardSequenceNumber() -> u32;
    }

    // ── Helpers ─────────────────────────────────────────────────────────

    fn send_input(inputs: &mut [INPUT]) {
        unsafe {
            SendInput(
                inputs.len() as u32,
                inputs.as_mut_ptr(),
                std::mem::size_of::<INPUT>() as i32,
            );
        }
    }

    /// Wait until all modifier keys are unpressed.
    unsafe fn wait_modifiers_released() {
        // Use __readfsdword or similar on MSVC; for MinGW/GNU we use GetAsyncKeyState
        // through extern "system".
        extern "system" {
            fn GetAsyncKeyState(vKey: i32) -> i16;
        }

        loop {
            let lwin = unsafe { GetAsyncKeyState(0x5B) };
            let rwin = unsafe { GetAsyncKeyState(0x5C) };
            let shift = unsafe { GetAsyncKeyState(0x10) };
            let alt = unsafe { GetAsyncKeyState(0x12) };
            let ctrl = unsafe { GetAsyncKeyState(0x11) };
            if lwin >= 0 && rwin >= 0 && shift >= 0 && alt >= 0 && ctrl >= 0 {
                break;
            }
            thread::sleep(Duration::from_millis(10));
        }
    }

    // ── Public API ──────────────────────────────────────────────────────

    /// Read current clipboard text.
    pub fn read_clipboard_text() -> Result<String, TextExtractorError> {
        unsafe {
            if OpenClipboard(null_mut()) == 0 {
                return Err(TextExtractorError::OperationFailed(
                    "failed to open clipboard".into(),
                ));
            }

            let h_data = GetClipboardData(CF_UNICODETEXT);
            if h_data.is_null() {
                CloseClipboard();
                return Err(TextExtractorError::NoTextSelected);
            }

            let locked = GlobalLock(h_data);
            if locked.is_null() {
                CloseClipboard();
                return Err(TextExtractorError::OperationFailed(
                    "failed to lock clipboard data".into(),
                ));
            }

            // Convert wide string to Rust String
            let len = GlobalSize(h_data) / 2; // number of UTF-16 code units
            let wide_slice = std::slice::from_raw_parts(locked as *const u16, len);
            // Trim trailing null
            let trimmed = match wide_slice.iter().position(|&c| c == 0) {
                Some(pos) => &wide_slice[..pos],
                None => wide_slice,
            };

            let text = String::from_utf16_lossy(trimmed);

            GlobalUnlock(h_data);
            CloseClipboard();

            if text.is_empty() {
                Err(TextExtractorError::NoTextSelected)
            } else {
                Ok(text)
            }
        }
    }

    /// Simulate Ctrl+C key press using SendInput.
    pub fn simulate_copy_key_press() -> Result<(), TextExtractorError> {
        unsafe {
            wait_modifiers_released();

            let mut inputs = [
                INPUT {
                    type_: INPUT_KEYBOARD,
                    padding: [0; 4],
                    ki: KEYBDINPUT {
                        w_vk: VK_CONTROL,
                        w_scan: 0,
                        dw_flags: 0,
                        time: 0,
                        dw_extra_info: 0,
                    },
                },
                INPUT {
                    type_: INPUT_KEYBOARD,
                    padding: [0; 4],
                    ki: KEYBDINPUT {
                        w_vk: 'C' as u16,
                        w_scan: 0,
                        dw_flags: 0,
                        time: 0,
                        dw_extra_info: 0,
                    },
                },
                INPUT {
                    type_: INPUT_KEYBOARD,
                    padding: [0; 4],
                    ki: KEYBDINPUT {
                        w_vk: 'C' as u16,
                        w_scan: 0,
                        dw_flags: KEYEVENTF_KEYUP,
                        time: 0,
                        dw_extra_info: 0,
                    },
                },
                INPUT {
                    type_: INPUT_KEYBOARD,
                    padding: [0; 4],
                    ki: KEYBDINPUT {
                        w_vk: VK_CONTROL,
                        w_scan: 0,
                        dw_flags: KEYEVENTF_KEYUP,
                        time: 0,
                        dw_extra_info: 0,
                    },
                },
            ];

            send_input(&mut inputs);
        }
        Ok(())
    }

    /// Snapshot of the current clipboard sequence number for change detection.
    pub struct ClipboardSnapshot {
        seq: u32,
    }

    /// Take a snapshot of the current clipboard state.
    pub fn clipboard_snapshot() -> ClipboardSnapshot {
        let seq = unsafe { GetClipboardSequenceNumber() };
        ClipboardSnapshot { seq }
    }

    /// Check if clipboard has changed since the snapshot.
    pub fn clipboard_changed(snapshot: &ClipboardSnapshot) -> bool {
        let current = unsafe { GetClipboardSequenceNumber() };
        current != snapshot.seq
    }

    /// Extract text from screen selection: simulate Ctrl+C, poll for
    /// clipboard change, then read clipboard.
    pub fn extract_from_screen_selection() -> Result<SelectionExtraction, TextExtractorError> {
        let (contents, snapshot_warning) = match snapshot_clipboard_contents() {
            Ok(contents) => (Some(contents), None),
            Err(error) => (None, Some(format!("failed to snapshot clipboard: {error}"))),
        };
        let snapshot = clipboard_snapshot();
        simulate_copy_key_press()?;

        let poll_interval = Duration::from_millis(50);
        let timeout = Duration::from_secs(3);
        let start = Instant::now();

        let result = loop {
            if clipboard_changed(&snapshot) {
                break read_clipboard_text();
            }
            if start.elapsed() >= timeout {
                break Err(TextExtractorError::NoTextSelected);
            }
            thread::sleep(poll_interval);
        };

        let restore_warning = contents.and_then(|contents| {
            restore_clipboard_contents(contents)
                .err()
                .map(|error| format!("failed to restore clipboard: {error}"))
        });
        result.map(|text| SelectionExtraction {
            text,
            read_method: "simulatedCopy".to_owned(),
            recoverable_error: snapshot_warning.or(restore_warning),
        })
    }
}

// ═══════════════════════════════════════════════════════════════════════════
//  Public API (platform-agnostic)
// ═══════════════════════════════════════════════════════════════════════════

/// Simulate Ctrl+C / Cmd+C key press.
///
/// On macOS this posts a CGEvent with Cmd modifier.
/// On Windows this uses SendInput with Ctrl modifier.
pub fn simulate_copy_key_press() -> Result<(), TextExtractorError> {
    #[cfg(any(target_os = "macos", target_os = "windows"))]
    {
        platform::simulate_copy_key_press()
    }
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        Err(TextExtractorError::UnsupportedMethod(
            "key press simulation is not supported on this platform".into(),
        ))
    }
}

/// Read the current clipboard text.
pub fn extract_from_clipboard() -> Result<String, TextExtractorError> {
    #[cfg(any(target_os = "macos", target_os = "windows"))]
    {
        platform::read_clipboard_text()
    }
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        Err(TextExtractorError::UnsupportedMethod(
            "clipboard is not supported on this platform".into(),
        ))
    }
}


/// Extract text from the current screen selection.
///
/// **macOS / Windows:** Simulates Cmd+C / Ctrl+C, polls the clipboard until
/// the content changes (or timeout), then returns the clipboard text.
///
pub fn extract_from_screen_selection() -> Result<String, TextExtractorError> {
    extract_from_screen_selection_detailed().map(|result| result.text)
}

pub fn extract_from_screen_selection_detailed() -> Result<SelectionExtraction, TextExtractorError> {
    #[cfg(any(target_os = "macos", target_os = "windows"))]
    {
        platform::extract_from_screen_selection()
    }
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        Err(TextExtractorError::UnsupportedMethod(
            "screen text extraction is not supported on this platform".into(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use base64::Engine as _;

    use super::encode_rgba_png;

    #[test]
    fn clipboard_rgba_is_encoded_as_png() {
        let encoded = encode_rgba_png(1, 1, &[255, 0, 0, 255]).unwrap();
        let bytes = base64::engine::general_purpose::STANDARD
            .decode(encoded)
            .unwrap();
        assert_eq!(&bytes[..8], b"\x89PNG\r\n\x1a\n");
    }

    #[test]
    fn clipboard_rgba_rejects_invalid_dimensions() {
        assert!(encode_rgba_png(2, 1, &[0; 4]).is_err());
    }
}
