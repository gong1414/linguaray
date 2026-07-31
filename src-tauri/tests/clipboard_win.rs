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
// All read paths use RAII so GlobalUnlock runs BEFORE CloseClipboard (Win32 contract:
// a clipboard memory object must be unlocked before the clipboard is closed). Round-14 P1.

fn clipboard_has(fmt: u32) -> bool {
    use windows_sys::Win32::System::DataExchange::IsClipboardFormatAvailable;
    unsafe { IsClipboardFormatAvailable(fmt) != 0 }
}

/// RAII: owns a locked clipboard HGLOBAL; Drop GlobalUnlocks it. Must be dropped BEFORE
/// the owning clipboard session (CloseClipboard) closes.
struct LockedClipData {
    handle: windows_sys::Win32::Foundation::HANDLE,
}
impl LockedClipData {
    /// Returns the locked pointer + the total GlobalSize (bytes). Null ptr = failure.
    unsafe fn lock(handle: windows_sys::Win32::Foundation::HANDLE) -> Option<(*const u8, usize)> {
        use windows_sys::Win32::System::Memory::{GlobalLock, GlobalSize};
        unsafe {
            let ptr = GlobalLock(handle);
            if ptr.is_null() {
                return None;
            }
            let size = GlobalSize(handle); // 0 on failure; treat as "no data"
            Some((ptr as *const u8, size))
        }
    }
}
impl Drop for LockedClipData {
    fn drop(&mut self) {
        // SAFETY: handle was GlobalLock'd in lock(); GlobalUnlock is the documented
        // release. Run BEFORE CloseClipboard (callers drop this guard first).
        use windows_sys::Win32::System::Memory::GlobalUnlock;
        unsafe {
            let _ = GlobalUnlock(self.handle);
        }
    }
}

fn read_unicode_text() -> Option<String> {
    use windows_sys::Win32::System::DataExchange::{CloseClipboard, GetClipboardData, OpenClipboard};
    use windows_sys::Win32::System::Ole::CF_UNICODETEXT;
    unsafe {
        if OpenClipboard(std::ptr::null_mut()) == 0 {
            return None;
        }
        // RAII: ensures CloseClipboard runs even on early return.
        struct ClipGuard;
        impl Drop for ClipGuard {
            fn drop(&mut self) {
                unsafe { CloseClipboard() };
            }
        }
        let _clip = ClipGuard;

        let h = GetClipboardData(CF_UNICODETEXT as u32);
        if h.is_null() {
            return None;
        }
        let (ptr, _size) = LockedClipData::lock(h)?; // _locked dropped (unlock) before _clip
        let _locked = LockedClipData { handle: h };
        // Read until a NUL u16 (UTF-16 code unit). ptr is *const u8; step by u16.
        let u16ptr = ptr as *const u16;
        let mut len = 0usize;
        // SAFETY: u16ptr points to the locked CF_UNICODETEXT buffer; reading sequentially
        // until a 0 u16 (the NUL terminator). The buffer is GlobalSize'd so the read stays
        // in bounds (a well-formed CF_UNICODETEXT blob is NUL-terminated).
        while *u16ptr.add(len) != 0 {
            len += 1;
        }
        let slice = std::slice::from_raw_parts(u16ptr, len);
        drop(_locked); // unlock BEFORE _clip (CloseClipboard) drops
        Some(String::from_utf16_lossy(slice))
    }
}

/// Read CF_DIBV5: returns (header bytes, pixel bytes) from the real clipboard. Used for
/// the full pixel round-trip (P2 #6): assert header + the 4 BGRA pixels after the system
/// clipboard round-trip.
fn read_dibv5() -> Option<(Vec<u8>, Vec<u8>)> {
    use windows_sys::Win32::System::DataExchange::{CloseClipboard, GetClipboardData, OpenClipboard};
    use windows_sys::Win32::System::Ole::CF_DIBV5;
    const HDR_SIZE: usize = std::mem::size_of::<windows_sys::Win32::Graphics::Gdi::BITMAPV5HEADER>();
    unsafe {
        if OpenClipboard(std::ptr::null_mut()) == 0 {
            return None;
        }
        struct ClipGuard;
        impl Drop for ClipGuard {
            fn drop(&mut self) {
                unsafe { CloseClipboard() };
            }
        }
        let _clip = ClipGuard;

        let h = GetClipboardData(CF_DIBV5 as u32);
        if h.is_null() {
            return None;
        }
        let (ptr, size) = LockedClipData::lock(h)?;
        let _locked = LockedClipData { handle: h };
        if size < HDR_SIZE {
            drop(_locked);
            return None;
        }
        // Header + pixel bytes (the blob is header followed by pixel data; total = size).
        let header = std::slice::from_raw_parts(ptr, HDR_SIZE).to_vec();
        let pixels = std::slice::from_raw_parts(ptr.add(HDR_SIZE), size - HDR_SIZE).to_vec();
        drop(_locked); // unlock BEFORE _clip (CloseClipboard) drops
        Some((header, pixels))
    }
}

// --- The four cardinality cases ---

#[test]
#[ignore = "touches the REAL system clipboard (Win-only); run via `cargo test --test clipboard_win -- --ignored --test-threads=1` (CI does this)"]
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
#[ignore = "touches the REAL system clipboard (Win-only); run via `cargo test --test clipboard_win -- --ignored --test-threads=1` (CI does this)"]
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
#[ignore = "touches the REAL system clipboard (Win-only); run via `cargo test --test clipboard_win -- --ignored --test-threads=1` (CI does this)"]
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
#[ignore = "touches the REAL system clipboard (Win-only); run via `cargo test --test clipboard_win -- --ignored --test-threads=1` (CI does this)"]
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

    // Text round-trip.
    assert_eq!(read_unicode_text().as_deref(), Some("hi"));

    // CF_DIBV5: FULL pixel round-trip (round-14 P2 #6). Read header + pixels back from the
    // REAL clipboard via GlobalSize/Lock/copy/Unlock, then assert the header + each BGRA
    // pixel position. A wrong stride, a bottom-up flip, or a channel-swap would scramble
    // these distinct per-position values (this is the test the prev version skipped).
    assert!(clipboard_has(CF_DIBV5 as u32));
    let (header, pixels) = read_dibv5().expect("read CF_DIBV5 back");
    assert_eq!(header.len(), std::mem::size_of::<BITMAPV5HEADER>(), "header is 124 bytes");
    // Parse the header via ptr::read_unaligned (the buffer came from GlobalLock, alignment
    // not guaranteed for a raw byte slice). Check the load-bearing fields.
    let hdr: BITMAPV5HEADER = unsafe { std::ptr::read_unaligned(header.as_ptr() as *const BITMAPV5HEADER) };
    assert_eq!(hdr.bV5Size, 124);
    assert_eq!(hdr.bV5Width, 2);
    assert_eq!(hdr.bV5Height, -2, "negated ⇒ top-down");
    assert_eq!(hdr.bV5BitCount, 32);
    assert_eq!(hdr.bV5Compression, windows_sys::Win32::Graphics::Gdi::BI_BITFIELDS);
    assert_eq!(hdr.bV5RedMask, 0x00FF_0000);
    assert_eq!(hdr.bV5AlphaMask, 0xFF00_0000);
    // 4 pixels × 4 bytes = 16 bytes of BGRA, in top-down order TL,TR,BL,BR.
    assert_eq!(pixels.len(), 16, "2×2×4 bytes of pixel data");
    // Expected BGRA per source RGBA (r,g,b,a)→(b,g,r,a):
    let expected: [[u8; 4]; 4] = [
        [0, 0, 255, 255],     // (0,0) red   RGBA(255,0,0,255)
        [0, 255, 0, 255],     // (1,0) green RGBA(0,255,0,255)
        [255, 0, 0, 255],     // (0,1) blue  RGBA(0,0,255,255)
        [0, 255, 255, 255],   // (1,1) yellow RGBA(255,255,0,255)
    ];
    for (i, want) in expected.iter().enumerate() {
        let got = &pixels[i * 4..i * 4 + 4];
        assert_eq!(
            got, want,
            "pixel {i} BGRA mismatch (got {:?}, want {:?}) — stride/flip/channel-swap check",
            got, want
        );
    }
}
