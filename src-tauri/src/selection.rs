//! Wires the §B engine to the real OS clipboard + enigo keystroke simulation.
use crate::clipboard;
use crate::selection_engine::{self, Capture, ClipboardLike};

/// The clipboard-owner handle threaded into `restore_snapshot` on Windows (the Tauri
/// main window's HWND, which runs the message loop that receives `WM_DESTROYCLIPBOARD`).
/// On non-Windows this is a unit placeholder so the cross-platform `capture_selection`
/// signature is uniform; the value is ignored there. (Phase 4 Task 2b M3.)
#[cfg(target_os = "windows")]
pub type OwnerHwnd = windows_sys::Win32::Foundation::HWND; // *mut c_void
#[cfg(not(target_os = "windows"))]
pub type OwnerHwnd = ();

/// The real OS clipboard adapter. On Windows it carries the owner HWND needed by the
/// compound restore (`clipboard::restore_snapshot(owner, …)`); elsewhere it's a unit
/// struct (no owner). The `ClipboardLike` trait signature is uniform across targets —
/// only the `restore_snapshot` body differs by cfg.
struct OsClipboard {
    #[cfg(target_os = "windows")]
    owner: OwnerHwnd,
}
impl ClipboardLike for OsClipboard {
    fn get_text(&self) -> Result<String, String> { clipboard::get_text() }
    fn set_text(&self, s: &str) -> Result<(), String> { clipboard::set_text(s) }
    fn get_image(&self) -> Result<Option<selection_engine::ImageBlob>, String> { clipboard::get_image() }
    fn set_image(&self, img: &selection_engine::ImageBlob) -> Result<(), String> { clipboard::set_image(img) }
    fn restore_snapshot(
        &self,
        text: Option<&str>,
        image: Option<&selection_engine::ImageBlob>,
    ) -> Result<(), String> {
        // Windows: compound write needs the owner HWND. macOS/other: 2-arg restore.
        #[cfg(target_os = "windows")]
        { clipboard::restore_snapshot(self.owner, text, image) }
        #[cfg(not(target_os = "windows"))]
        { let _ = self; clipboard::restore_snapshot(text, image) }
    }
    fn sequence(&self) -> u64 { clipboard::sequence() }
}

/// Simulate the platform copy keystroke: Cmd+C on macOS, Ctrl+C elsewhere.
fn simulate_copy() -> Result<(), String> {
    use enigo::{Direction, Enigo, Key, Keyboard, Settings};
    let mut enigo = Enigo::new(&Settings::default()).map_err(|e| e.to_string())?;
    #[cfg(target_os = "macos")]
    let modifier = Key::Meta;
    #[cfg(not(target_os = "macos"))]
    let modifier = Key::Control;
    // The draft used Layout('c'); enigo 0.2.1 has no Layout variant — Unicode('c')
    // routes through the layout/keymap (keysym), which is what we want.
    enigo
        .key(modifier, Direction::Press)
        .map_err(|e| e.to_string())?;
    enigo
        .key(Key::Unicode('c'), Direction::Click)
        .map_err(|e| e.to_string())?;
    enigo
        .key(modifier, Direction::Release)
        .map_err(|e| e.to_string())?;
    Ok(())
}

/// Capture the current selection via the §B hybrid algorithm (spec §B):
/// 1. Try the macOS AX direct-read FIRST (no clipboard touched — cleanest). On
///    success, return immediately.
/// 2. On AX failure/empty/untrusted, fall back to the sentinel simulate-copy
///    path (selection_engine). ~timeout_ms total for the fallback.
///
/// `owner` is the clipboard-owner HWND on Windows (threaded into the compound restore);
/// `()` on other targets (uniform signature, value ignored there). (Phase 4 Task 2b M3.)
pub fn capture_selection(timeout_ms: u64, owner: OwnerHwnd) -> Result<Capture, String> {
    capture_selection_with_ax(crate::a11y::read_selection, timeout_ms, owner)
}

/// Same as capture_selection but with an injectable AX reader (for testing the
/// AX-first → copy-fallback routing without the real AX FFI).
pub fn capture_selection_with_ax<A: FnOnce() -> Option<String>>(
    ax_reader: A,
    timeout_ms: u64,
    #[cfg_attr(not(target_os = "windows"), allow(unused_variables))] owner: OwnerHwnd,
) -> Result<Capture, String> {
    // §B AX-first.
    if let Some(text) = ax_reader() {
        if !text.trim().is_empty() {
            return Ok(Capture::Selected(text));
        }
    }
    // Copy-fallback.
    let iters = (timeout_ms / 20) as usize;
    #[cfg(target_os = "windows")]
    let clip = OsClipboard { owner };
    #[cfg(not(target_os = "windows"))]
    let clip = OsClipboard {};
    selection_engine::capture(&clip, simulate_copy, iters)
}
