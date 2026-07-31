//! Windows compound-clipboard restore adapter (Phase 4 Task 2b, milestone 2).
//!
//! `#[cfg(windows)]` ONLY. Holds every Windows-specific piece of the compound write:
//! `Handle` (newtype over `HGLOBAL`), the `GlobalMemOps` trait + the real `alloc_global`
//! helper (leak-safe via a local RAII guard disarmed only on success — covers
//! GlobalAlloc-ok/GlobalLock-fail), `Win32ClipOps` (impls the platform-neutral
//! `super::fsm::ClipOps`), and `build_blobs` (maps `(Option<&str>, Option<&ImageBlob>)`
//! to 0/1/2 prepared formats). Windows-only unit tests cover `build_blobs` cardinality
//! and the real `alloc_global` leak path (injected `GlobalMemOps` fake).
//!
//! The PUBLIC `restore_snapshot` wrapper + cfg-split + the HWND callchain + the real
//! clipboard integration test land in milestone 3. Until then these items are unused
//! outside tests → `#[allow(dead_code)]`.

use super::fsm::ClipOps;
use crate::selection_engine::ImageBlob;
use std::os::windows::ffi::OsStrExt;

/// A Win32 movable-memory handle owning one prepared blob. Newtype so the FSM's
/// `type Handle` is well-defined; `.0` unwraps the raw `HGLOBAL` in `set`/`free`.
pub(super) struct Handle(pub windows_sys::Win32::Foundation::HGLOBAL);

/// Injectable low-level Win32 memory ops, so the REAL `alloc_global` helper is unit-
/// tested with an injected lock failure (round-13 review P1: a `ClipOps`-level fake
/// sees `alloc` as one black box and can't catch the adapter forgetting `GlobalFree`).
trait GlobalMemOps {
    /// `GlobalAlloc(flags, bytes)` → raw handle; null = failure.
    fn alloc(&mut self, flags: u32, bytes: usize) -> *mut core::ffi::c_void;
    /// `GlobalLock(h)` → pointer; null = failure.
    fn lock(&mut self, h: *mut core::ffi::c_void) -> *mut core::ffi::c_void;
    /// `GlobalUnlock(h)` (best-effort).
    unsafe fn unlock(&mut self, h: *mut core::ffi::c_void) -> i32;
    /// `GlobalFree(h)`.
    unsafe fn free(&mut self, h: *mut core::ffi::c_void);
}

/// Real `GlobalMemOps` over the Win32 Local/Global heap.
struct RealGlobalMem;

impl GlobalMemOps for RealGlobalMem {
    fn alloc(&mut self, flags: u32, bytes: usize) -> *mut core::ffi::c_void {
        unsafe { windows_sys::Win32::System::Memory::GlobalAlloc(flags, bytes) }
    }
    fn lock(&mut self, h: *mut core::ffi::c_void) -> *mut core::ffi::c_void {
        unsafe { windows_sys::Win32::System::Memory::GlobalLock(h) }
    }
    unsafe fn unlock(&mut self, h: *mut core::ffi::c_void) -> i32 {
        unsafe { windows_sys::Win32::System::Memory::GlobalUnlock(h) }
    }
    unsafe fn free(&mut self, h: *mut core::ffi::c_void) {
        unsafe { windows_sys::Win32::Foundation::GlobalFree(h) };
    }
}

/// Allocate a `GMEM_MOVEABLE` blob of `bytes.len()`, copy `bytes` in, return a `Handle`.
/// Generic over `GlobalMemOps` so a Windows unit test injects a `lock_fails` fake and
/// asserts `free` runs on the alloc'd handle (the leak the round-12 design couldn't
/// catch). Postcondition honored: on Err, NO app-owned handle is live — the local RAII
/// guard `GlobalGuard` owns the raw handle and `GlobalFree`s in `Drop`; it is
/// `mem::forget`-disarmed ONLY on the success path (right before returning the Handle).
fn alloc_global<M: GlobalMemOps>(m: &mut M, bytes: &[u8]) -> Result<Handle, String> {
    use windows_sys::Win32::System::Memory::GMEM_MOVEABLE;

    let raw = m.alloc(GMEM_MOVEABLE, bytes.len());
    if raw.is_null() {
        return Err("GlobalAlloc failed".into());
    }
    // RAII: owns `raw`, Drop calls m.free(raw). Disarmed via mem::forget on success.
    struct GlobalGuard<'a, M: GlobalMemOps>(&'a mut M, *mut core::ffi::c_void);
    impl<M: GlobalMemOps> Drop for GlobalGuard<'_, M> {
        fn drop(&mut self) {
            // SAFETY: self.1 was returned by alloc and not yet freed.
            unsafe { self.0.free(self.1) };
        }
    }
    let g = GlobalGuard(m, raw);
    let ptr = g.0.lock(raw);
    if ptr.is_null() {
        return Err("GlobalLock failed".into()); // g.Drop frees raw ✓
    }
    // SAFETY: ptr is a valid locked pointer to a buffer of bytes.len(); src is a valid
    // &[u8] of the same length. Both alive for the copy.
    unsafe { std::ptr::copy_nonoverlapping(bytes.as_ptr(), ptr as *mut u8, bytes.len()) };
    let _ = unsafe { g.0.unlock(raw) }; // best-effort
    std::mem::forget(g); // disarm: ownership transfers to the caller via Handle
    Ok(Handle(raw))
}

/// The Win32 clipboard adapter. Stores the owner HWND (resolved in the callchain);
/// `open` passes it to `OpenClipboard`.
pub(super) struct Win32ClipOps {
    owner: windows_sys::Win32::Foundation::HWND,
}

impl Win32ClipOps {
    pub(super) fn new(owner: windows_sys::Win32::Foundation::HWND) -> Self {
        Win32ClipOps { owner }
    }
}

impl ClipOps for Win32ClipOps {
    type Handle = Handle;

    fn open(&mut self) -> Result<(), String> {
        // SAFETY: self.owner is a valid HWND (the Tauri main window's, milestone 3).
        // OpenClipboard returns 0 on failure.
        let ok = unsafe {
            windows_sys::Win32::System::DataExchange::OpenClipboard(self.owner)
        };
        if ok == 0 {
            return Err("OpenClipboard failed".into());
        }
        Ok(())
    }
    fn close(&mut self) {
        // SAFETY: called only after a successful OpenClipboard (the OpenClip guard is
        // constructed only then). CloseClipboard is the documented release.
        unsafe {
            windows_sys::Win32::System::DataExchange::CloseClipboard();
        }
    }
    fn empty(&mut self) -> Result<(), String> {
        // SAFETY: clipboard is open (called via the OpenClip guard). EmptyClipboard
        // assigns ownership to the window that has the clipboard open + frees prior data.
        let ok = unsafe {
            windows_sys::Win32::System::DataExchange::EmptyClipboard()
        };
        if ok == 0 {
            return Err("EmptyClipboard failed".into());
        }
        Ok(())
    }
    fn set(&mut self, fmt: u32, h: Handle) -> Result<(), (Handle, String)> {
        // SetClipboardData(fmt, h.0): on Ok ownership transfers to the system (do NOT
        // free); on failure it returns null and ownership does NOT transfer → return the
        // handle so the caller frees it.
        // SAFETY: h.0 is a valid GMEM_MOVEABLE HGLOBAL (from alloc_global); fmt is a real
        // clipboard format id (CF_UNICODETEXT/CF_DIBV5 from build_blobs).
        let r = unsafe {
            windows_sys::Win32::System::DataExchange::SetClipboardData(fmt, h.0)
        };
        if r.is_null() {
            return Err((h, "SetClipboardData failed".into()));
        }
        Ok(())
    }
    fn alloc(&mut self, bytes: &[u8]) -> Result<Handle, String> {
        alloc_global(&mut RealGlobalMem, bytes)
    }
    fn free(&mut self, h: Handle) {
        // SAFETY: h.0 is an app-owned HGLOBAL not submitted to the system.
        unsafe { windows_sys::Win32::Foundation::GlobalFree(h.0) };
    }
}

/// Build the prepared-format list from the `(text, image)` Options. Cardinality:
/// `(None, None)` → empty Vec (restore_with clears only — the §B empty-original case);
/// `(Some(t), None)` → one `[CF_UNICODETEXT, utf16-NUL]`; `(None, Some(img))` → one
/// `[CF_DIBV5, header+BGRA]`; `(Some, Some)` → two, order `[text, dib]`.
///
/// Order matters (the FSM is format-agnostic, so a swap would slip through without the
/// `build_blobs` cardinality tests). Checked conversions mirror the macOS preflight.
fn build_blobs(
    text: Option<&str>,
    image: Option<&ImageBlob>,
) -> Result<Vec<(u32, Vec<u8>)>, String> {
    use windows_sys::Win32::Graphics::Gdi::{BI_BITFIELDS, BITMAPV5HEADER};
    use windows_sys::Win32::System::Ole::{CF_DIBV5, CF_UNICODETEXT};
    use windows_sys::Win32::UI::ColorSystem::LCS_sRGB;
    // LCS_GM_IMAGES is in Graphics::Gdi (verified); render intent = perceptual/picture.
    const LCS_GM_IMAGES: u32 = 4;

    let mut out: Vec<(u32, Vec<u8>)> = Vec::with_capacity(2);

    // (a) UTF-16 NUL-terminated text. encode_wide is on OsStr (via OsStrExt), so go
    // through OsStr — str: AsRef<OsStr>.
    if let Some(t) = text {
        let mut u16s: Vec<u16> = std::ffi::OsStr::new(t).encode_wide().collect();
        u16s.push(0); // NUL terminator
        let bytes: Vec<u8> = u16s.iter().flat_map(|w| w.to_le_bytes()).collect();
        out.push((CF_UNICODETEXT as u32, bytes));
    }

    // (b) CF_DIBV5 = BITMAPV5HEADER + BGRA pixels (top-down, BI_BITFIELDS, masks).
    if let Some(img) = image {
        // Preflight (mirrors the macOS restore_compound_to checks; the Win32 fields are
        // i32/u32 so wrapping casts would be UB at the FFI boundary).
        if img.width == 0 || img.height == 0 {
            return Err(format!(
                "image has zero dimension ({}x{})",
                img.width, img.height
            ));
        }
        let width = i32::try_from(img.width)
            .map_err(|_| format!("image width {} exceeds i32 range", img.width))?;
        let height = i32::try_from(img.height)
            .map_err(|_| format!("image height {} exceeds i32 range", img.height))?;
        // bV5Height is negated for top-down; i32::MIN can't be negated.
        let neg_height = height
            .checked_neg()
            .ok_or_else(|| "image height == i32::MIN (cannot negate for top-down)".to_string())?;
        let total_u64 = (img.width as u64)
            .checked_mul(img.height as u64)
            .and_then(|n| n.checked_mul(4))
            .ok_or_else(|| "image byte count overflows u64".to_string())?;
        let expected = usize::try_from(total_u64)
            .map_err(|_| "image byte count exceeds usize".to_string())?;
        if img.bytes.len() != expected {
            return Err(format!(
                "image bytes {} != width*height*4 ({})",
                img.bytes.len(),
                expected
            ));
        }

        // Header: zeroed, then set the fields that matter. CIEXYZTRIPLE endpoints stay
        // zeroed (sRGB doesn't need them).
        let mut hdr: BITMAPV5HEADER = unsafe { std::mem::zeroed() };
        hdr.bV5Size = std::mem::size_of::<BITMAPV5HEADER>() as u32; // 124
        hdr.bV5Width = width;
        hdr.bV5Height = neg_height; // negative ⇒ top-down, origin upper-left (no vertical flip)
        hdr.bV5Planes = 1;
        hdr.bV5BitCount = 32;
        hdr.bV5Compression = BI_BITFIELDS; // 3 — required to honor the masks below
        // bV5SizeImage is u32; total_u64 may exceed it for huge images → checked conversion.
        hdr.bV5SizeImage = u32::try_from(total_u64)
            .map_err(|_| "image byte count exceeds u32 (bV5SizeImage)".to_string())?;
        // BGRA masks (Windows native byte order: B,G,R,A per pixel).
        hdr.bV5RedMask = 0x00FF_0000;
        hdr.bV5GreenMask = 0x0000_FF00;
        hdr.bV5BlueMask = 0x0000_00FF;
        hdr.bV5AlphaMask = 0xFF00_0000;
        hdr.bV5CSType = LCS_sRGB as u32; // sRGB color space
        hdr.bV5Intent = LCS_GM_IMAGES; // perceptual/picture rendering intent

        // Pixel buffer: convert each source RGBA pixel (r,g,b,a) → BGRA bytes (b,g,r,a).
        // Row stride = width*4 (no padding at 32bpp). Channel swap is what the test asserts.
        let mut bytes = Vec::with_capacity(std::mem::size_of::<BITMAPV5HEADER>() + img.bytes.len());
        let hdr_bytes: &[u8] = unsafe {
            std::slice::from_raw_parts(
                &hdr as *const BITMAPV5HEADER as *const u8,
                std::mem::size_of::<BITMAPV5HEADER>(),
            )
        };
        bytes.extend_from_slice(hdr_bytes);
        for px in img.bytes.chunks_exact(4) {
            // px = [r, g, b, a] → emit [b, g, r, a]
            bytes.push(px[2]); // B
            bytes.push(px[1]); // G
            bytes.push(px[0]); // R
            bytes.push(px[3]); // A
        }
        out.push((CF_DIBV5 as u32, bytes));
    }

    Ok(out)
}

/// Public Windows compound-clipboard restore. NON-generic: builds the format list
/// (0/1/2 entries), constructs the real `Win32ClipOps` adapter (storing `owner`), and
/// runs the platform-neutral `restore_with` FSM. Re-exported from `clipboard/mod.rs`
/// so callers use the uniform `clipboard::restore_snapshot` path. The `owner` HWND
/// (`crate::selection::OwnerHwnd`) must belong to a thread running a message loop (the
/// app reuses the Tauri main window's HWND; a clipboard owner receives
/// `WM_DESTROYCLIPBOARD` even with eager rendering).
pub fn restore_snapshot(
    owner: crate::selection::OwnerHwnd,
    text: Option<&str>,
    image: Option<&crate::selection_engine::ImageBlob>,
) -> Result<(), String> {
    let formats = build_blobs(text, image)?;
    let mut ops = Win32ClipOps::new(owner);
    super::fsm::restore_with(&mut ops, &formats).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    //! Windows-only unit tests for the cfg(windows) adapter. `build_blobs` cardinality
    //! (all 4 Option combos → correct count/format-id/order) + the REAL `alloc_global`
    //! leak test (inject a `GlobalMemOps` fake that forces lock-fail; a ClipOps-level
    //! fake can't reach into the adapter's internals). The real Windows clipboard
    //! integration test lands in milestone 3.
    use super::*;

    #[test]
    fn build_blobs_empty_is_zero_entries() {
        let out = build_blobs(None, None).unwrap();
        assert!(out.is_empty(), "(None,None) → 0 entries (clear-only)");
    }

    #[test]
    fn build_blobs_text_only_is_one_unicode_text() {
        use windows_sys::Win32::System::Ole::CF_UNICODETEXT;
        let out = build_blobs(Some("hi"), None).unwrap();
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].0, CF_UNICODETEXT as u32);
        // Decode UTF-16-LE + NUL → "hi".
        let u16s: Vec<u16> = out[0]
            .1
            .chunks_exact(2)
            .map(|c| u16::from_le_bytes([c[0], c[1]]))
            .collect();
        let s = String::from_utf16_lossy(&u16s);
        assert_eq!(s, "hi\0");
    }

    #[test]
    fn build_blobs_image_only_is_one_dibv5() {
        use windows_sys::Win32::System::Ole::CF_DIBV5;
        let img = ImageBlob {
            width: 1,
            height: 1,
            bytes: vec![255, 0, 0, 255],
        };
        let out = build_blobs(None, Some(&img)).unwrap();
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].0, CF_DIBV5 as u32);
    }

    #[test]
    fn build_blobs_text_and_image_is_two_in_text_then_dib_order() {
        use windows_sys::Win32::System::Ole::{CF_DIBV5, CF_UNICODETEXT};
        let img = ImageBlob {
            width: 1,
            height: 1,
            bytes: vec![0, 0, 0, 0],
        };
        let out = build_blobs(Some("hi"), Some(&img)).unwrap();
        assert_eq!(out.len(), 2);
        // ORDER matters — the FSM is format-agnostic, so a swap would slip through
        // without this assertion.
        assert_eq!(out[0].0, CF_UNICODETEXT as u32, "text first");
        assert_eq!(out[1].0, CF_DIBV5 as u32, "dib second");
    }

    #[test]
    fn build_blobs_dibv5_header_and_first_pixel_bgra() {
        // Header fields + the RGBA→BGRA channel swap (top-down, masks). A 1×1 opaque-red
        // image: RGBA(255,0,0,255) → BGRA bytes (0,0,255,255). Asserts the header masks +
        // the pixel conversion (channel order) in one go.
        use windows_sys::Win32::Graphics::Gdi::BI_BITFIELDS;
        let img = ImageBlob {
            width: 1,
            height: 1,
            bytes: vec![255, 0, 0, 255], // RGBA red
        };
        let out = build_blobs(None, Some(&img)).unwrap();
        let blob = &out[0].1;
        // Header is 124 bytes.
        assert_eq!(std::mem::size_of::<windows_sys::Win32::Graphics::Gdi::BITMAPV5HEADER>(), 124);
        assert!(blob.len() > 124);
        let hdr_size = u32::from_le_bytes([blob[0], blob[1], blob[2], blob[3]]);
        assert_eq!(hdr_size, 124);
        // Read key header fields by offset (see BITMAPV5HEADER layout). This is fragile to
        // struct reordering, but the layout is a stable ABI; offset-based reads mirror how
        // an external reader would parse it. We re-derive offsets from the struct to avoid
        // magic numbers.
        let hdr: &windows_sys::Win32::Graphics::Gdi::BITMAPV5HEADER = unsafe {
            &*(blob.as_ptr() as *const windows_sys::Win32::Graphics::Gdi::BITMAPV5HEADER)
        };
        assert_eq!(hdr.bV5Width, 1);
        assert_eq!(hdr.bV5Height, -1, "negated ⇒ top-down");
        assert_eq!(hdr.bV5BitCount, 32);
        assert_eq!(hdr.bV5Compression, BI_BITFIELDS);
        assert_eq!(hdr.bV5RedMask, 0x00FF_0000);
        assert_eq!(hdr.bV5AlphaMask, 0xFF00_0000);
        // First (only) pixel at offset 124: BGRA = (0,0,255,255) for red RGBA.
        assert_eq!(&blob[124..128], &[0, 0, 255, 255], "BGRA for RGBA-red");
    }

    #[test]
    fn build_blobs_rejects_zero_dim_and_bad_len() {
        assert!(build_blobs(None, Some(&ImageBlob { width: 0, height: 4, bytes: vec![] })).is_err());
        assert!(build_blobs(None, Some(&ImageBlob { width: 2, height: 2, bytes: vec![0; 4] })).is_err());
    }

    // === The REAL alloc_global leak test (round-13 review P1) ===
    // Inject a GlobalMemOps fake that forces GlobalLock to fail; assert the helper frees
    // the alloc'd handle (the leak a ClipOps-level fake can't catch, since ClipOps::alloc
    // is one black box). A local RAII guard in alloc_global is the production equivalent.
    struct FakeGlobalMem {
        lock_fails: bool,
        log: std::cell::RefCell<Vec<&'static str>>,
    }
    impl GlobalMemOps for FakeGlobalMem {
        fn alloc(&mut self, _flags: u32, _bytes: usize) -> *mut core::ffi::c_void {
            self.log.borrow_mut().push("alloc");
            // Non-null sentinel pointer (never dereferenced: lock returns null).
            0xdead_beef as *mut _
        }
        fn lock(&mut self, _h: *mut core::ffi::c_void) -> *mut core::ffi::c_void {
            self.log.borrow_mut().push("lock");
            if self.lock_fails {
                std::ptr::null_mut()
            } else {
                // This branch is unreachable in real_alloc_helper_frees_on_lock_fail (which
                // always sets lock_fails: true). unreachable! (not unimplemented!) so clippy's
                // clippy::unimplemented lint doesn't fire under -D warnings.
                unreachable!("lock-ok path not used in this test")
            }
        }
        unsafe fn unlock(&mut self, _h: *mut core::ffi::c_void) -> i32 {
            self.log.borrow_mut().push("unlock");
            1
        }
        unsafe fn free(&mut self, _h: *mut core::ffi::c_void) {
            self.log.borrow_mut().push("free");
        }
    }

    #[test]
    fn real_alloc_helper_frees_on_lock_fail() {
        // GlobalAlloc ok, GlobalLock fails → helper must GlobalFree the handle before Err.
        let mut m = FakeGlobalMem {
            lock_fails: true,
            log: std::cell::RefCell::new(Vec::new()),
        };
        let r = alloc_global(&mut m, b"payload");
        assert!(r.is_err(), "alloc_global should return Err on lock fail");
        let log = m.log.borrow();
        assert!(log.contains(&"alloc"), "alloc called: {log:?}");
        assert!(log.contains(&"lock"), "lock called: {log:?}");
        assert!(
            log.contains(&"free"),
            "free MUST be called on lock fail (the leak this test guards): {log:?}"
        );
        assert!(!log.contains(&"unlock"), "no unlock on the failure path: {log:?}");
    }
}
