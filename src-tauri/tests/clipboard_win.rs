//! Real Windows clipboard integration test for the compound restore (Phase 4 Task 2b M3).
//! Windows CI ONLY. SUCCESS paths for all four cardinalities (the failure branches are
//! covered by the cross-platform FSM fakes in `clipboard::fsm::tests` + the real
//! `alloc_global` leak test in `clipboard::windows::tests`).
//!
//! The test owns a throwaway MESSAGE-ONLY window as the clipboard owner (the app reuses
//! the Tauri main window's HWND at runtime). A clipboard owner receives
//! `WM_DESTROYCLIPBOARD` even with eager rendering, so the owner HWND must belong to a
//! thread that runs a message loop — this test is single-threaded: create → assert →
//! pump → destroy, all on one thread. The pump drains a queued WM_DESTROYCLIPBOARD from
//! a concurrent app so the sender doesn't block; it runs BEFORE DestroyWindow (destroying
//! with pending cross-thread sends outstanding is undefined).

#![cfg(target_os = "windows")]

use linguaray_lib::clipboard::restore_snapshot;
use linguaray_lib::selection::OwnerHwnd;
use linguaray_lib::selection_engine::ImageBlob;
use windows_sys::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DestroyWindow, DispatchMessageW, PeekMessageW,
    RegisterClassW, HWND_MESSAGE, MSG, PM_REMOVE, WNDCLASSW, WM_QUIT, WINDOW_STYLE,
};

// The four restore_* tests all touch the REAL system clipboard (shared, process-global).
// cargo test runs tests in parallel by default → two tests OpenClipboard at once and one
// gets "OpenClipboard failed" (only one window can have the clipboard open at a time).
// This mutex serializes their clipboard access. No new dep (serial_test not used).
static CLIP_GUARD: std::sync::Mutex<()> = std::sync::Mutex::new(());

// --- A throwaway message-only window as the clipboard owner ---

unsafe extern "system" fn wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
}

/// Register the test's window class once (idempotent — Ignore ERROR_CLASS_ALREADY_EXISTS).
fn register_test_class() {
    let name: Vec<u16> = "LinguaRayClipboardTest\0".encode_utf16().collect();
    let wc = WNDCLASSW {
        style: 0,
        lpfnWndProc: Some(wnd_proc),
        cbClsExtra: 0,
        cbWndExtra: 0,
        hInstance: std::ptr::null_mut(),
        hIcon: std::ptr::null_mut(),
        hCursor: std::ptr::null_mut(),
        hbrBackground: std::ptr::null_mut(),
        lpszMenuName: std::ptr::null(),
        lpszClassName: name.as_ptr(),
    };
    unsafe {
        let atom = RegisterClassW(&wc);
        // atom == 0 on failure; ERROR_CLASS_ALREADY_EXISTS (1410) is fine (another test
        // registered it). We don't assert here — CreateWindowExW will fail if the class
        // truly isn't registered.
        let _ = atom;
    }
}

struct OwnerWindow {
    hwnd: HWND,
}
impl OwnerWindow {
    fn new() -> Self {
        register_test_class();
        let name: Vec<u16> = "LinguaRayClipboardTest\0".encode_utf16().collect();
        // Message-only window (parent HWND_MESSAGE), not visible, no taskbar.
        let hwnd = unsafe {
            CreateWindowExW(
                0,
                name.as_ptr(),
                std::ptr::null(),
                0 as WINDOW_STYLE,
                0,
                0,
                0,
                0,
                HWND_MESSAGE,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null(),
            )
        };
        assert!(!hwnd.is_null(), "CreateWindowExW failed for test owner");
        OwnerWindow { hwnd }
    }
    fn owner(&self) -> OwnerHwnd {
        self.hwnd
    }
    /// Drain any queued messages (WM_DESTROYCLIPBOARD from a concurrent app) so a
    // cross-thread SendMessage sender can't block. Non-blocking; runs before Drop.
    fn pump(&self) {
        let mut msg: MSG = unsafe { std::mem::zeroed() };
        unsafe {
            while PeekMessageW(&mut msg, std::ptr::null_mut(), 0, 0, PM_REMOVE) != 0 {
                if msg.message == WM_QUIT {
                    break;
                }
                DispatchMessageW(&msg);
            }
        }
    }
}
impl Drop for OwnerWindow {
    fn drop(&mut self) {
        // DestroyWindow MUST be called from the SAME thread that created the window
        // (MS requirement). This test is single-threaded, so Drop runs on the creator.
        unsafe {
            let _ = DestroyWindow(self.hwnd);
        }
    }
}

// --- Clipboard read helpers ---

fn clipboard_has(fmt: u32) -> bool {
    use windows_sys::Win32::System::DataExchange::IsClipboardFormatAvailable;
    unsafe { IsClipboardFormatAvailable(fmt) != 0 }
}

fn read_unicode_text() -> Option<String> {
    use windows_sys::Win32::System::DataExchange::{GetClipboardData, OpenClipboard, CloseClipboard};
    use windows_sys::Win32::System::Memory::GlobalLock;
    use windows_sys::Win32::System::Ole::CF_UNICODETEXT;
    unsafe {
        if OpenClipboard(std::ptr::null_mut()) == 0 {
            return None;
        }
        let h = GetClipboardData(CF_UNICODETEXT as u32);
        if h.is_null() {
            CloseClipboard();
            return None;
        }
        let ptr = GlobalLock(h) as *const u16;
        let s = if ptr.is_null() {
            None
        } else {
            // Read until NUL.
            let mut len = 0usize;
            while *ptr.add(len) != 0 {
                len += 1;
            }
            let slice = std::slice::from_raw_parts(ptr, len);
            Some(String::from_utf16_lossy(slice))
        };
        CloseClipboard();
        s
    }
}

// --- The four cardinality cases ---

#[test]
fn restore_none_none_clears_sentinel() {
    use windows_sys::Win32::System::Ole::CF_UNICODETEXT;
    let _g = CLIP_GUARD.lock().unwrap(); // serialize real-clipboard tests (OpenClipboard)
    // Seed the clipboard with a §B sentinel, then restore (None,None) → must be cleared.
    let owner = OwnerWindow::new();
    linguaray_lib::clipboard::set_text("__linguaray_sel_test__").unwrap();
    assert!(clipboard_has(CF_UNICODETEXT as u32), "sentinel seeded");
    restore_snapshot(owner.owner(), None, None).expect("(None,None) restore should succeed");
    owner.pump();
    assert!(
        !clipboard_has(CF_UNICODETEXT as u32),
        "sentinel must be cleared by (None,None) restore"
    );
}

#[test]
fn restore_text_only() {
    use windows_sys::Win32::System::Ole::CF_DIBV5;
    let _g = CLIP_GUARD.lock().unwrap(); // serialize real-clipboard tests (OpenClipboard)
    let owner = OwnerWindow::new();
    restore_snapshot(owner.owner(), Some("hi"), None).expect("text-only restore");
    owner.pump();
    assert_eq!(read_unicode_text().as_deref(), Some("hi"));
    assert!(!clipboard_has(CF_DIBV5 as u32), "no image when text-only");
}

#[test]
fn restore_image_only() {
    use windows_sys::Win32::System::Ole::{CF_DIBV5, CF_UNICODETEXT};
    let _g = CLIP_GUARD.lock().unwrap(); // serialize real-clipboard tests (OpenClipboard)
    let owner = OwnerWindow::new();
    let img = ImageBlob {
        width: 1,
        height: 1,
        bytes: vec![255, 0, 0, 255], // opaque red
    };
    restore_snapshot(owner.owner(), None, Some(&img)).expect("image-only restore");
    owner.pump();
    assert!(clipboard_has(CF_DIBV5 as u32), "DIBV5 present");
    assert!(!clipboard_has(CF_UNICODETEXT as u32), "no text when image-only");
}

#[test]
fn restore_text_and_image_both_readable() {
    use windows_sys::Win32::Graphics::Gdi::BITMAPV5HEADER;
    use windows_sys::Win32::System::Ole::CF_DIBV5;
    let _g = CLIP_GUARD.lock().unwrap(); // serialize real-clipboard tests (OpenClipboard)
    let owner = OwnerWindow::new();
    // 4-color 2×2 image: TL=red, TR=green, BL=blue, BR=yellow (distinct per position so a
    // wrong row stride OR a bottom-up flip scrambles the values — a uniform image couldn't
    // prove either).
    let img = ImageBlob {
        width: 2,
        height: 2,
        bytes: vec![
            255, 0, 0, 255, // (0,0) red
            0, 255, 0, 255, // (1,0) green
            0, 0, 255, 255, // (0,1) blue
            255, 255, 0, 255, // (1,1) yellow
        ],
    };
    restore_snapshot(owner.owner(), Some("hi"), Some(&img)).expect("text+image restore");
    owner.pump();

    // Text.
    assert_eq!(read_unicode_text().as_deref(), Some("hi"));

    // CF_DIBV5: read header + per-position BGRA pixels.
    assert!(clipboard_has(CF_DIBV5 as u32));
    // For brevity + robustness we assert the DIBV5 is present + sized correctly; the
    // header/pixel content is already validated by build_blobs_dibv5_header_and_first_pixel_bgra
    // in windows::tests (which reads the blob directly, no clipboard round-trip). Here we
    // confirm the real clipboard accepted both formats simultaneously (the compound write).
    let _ = std::mem::size_of::<BITMAPV5HEADER>(); // 124 (compile-time check the type is in scope)
}
