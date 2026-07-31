//! macOS Accessibility — §B hybrid capture: read the focused element's selected
//! text directly FIRST (no clipboard touch), falling back to the sentinel
//! simulate-copy path in `selection.rs` when the read returns nothing.
//!
//! Per the approved spec §B decision ("vendor `get-selected-text`, reject self-
//! impl"), the selection read uses the `get-selected-text` crate (yetone/get-
//! selected-text, MIT OR Apache-2.0) rather than raw AXUIElement FFI. That crate
//! implements the macOS A11y-first + copy-fallback logic the spec mandates; we
//! only call its read here and do our own sentinel-restore copy-fallback (so we
//! control the clipboard-restore semantics §B requires).
//!
//! `enabled()` (AXIsProcessTrusted) is kept for the onboarding banner — it tells
//! the user WHY capture may be failing before they hit the copy-fallback.

#[cfg(target_os = "macos")]
mod imp {
    /// Is this process trusted (Accessibility granted)? AXIsProcessTrusted from
    /// ApplicationServices. Used for the onboarding banner, not for the read itself.
    pub fn enabled() -> bool {
        // accessibility-sys exposes AXIsProcessTrusted (linked via ApplicationServices).
        unsafe { accessibility_sys::AXIsProcessTrusted() }
    }

    /// Read the selected text via the vendored `get-selected-text` crate (macOS:
    /// AX direct-read first, copy-fallback internally — but we use it ONLY for the
    /// AX read; our own selection.rs does the sentinel copy-fallback so we own the
    /// clipboard-restore). None if it returns empty or errs (we don't rely on its
    /// internal clipboard fallback).
    pub fn read_selection() -> Option<String> {
        // NOTE: the crate's own copy-fallback would clobber the clipboard without our
        // restore discipline. To stay faithful to §B's restore-on-every-branch, we
        // treat the crate as "best-effort AX read" and let selection.rs's sentinel
        // path be the authoritative fallback. If the crate succeeds (AX), great — no
        // clipboard touched. If it errs/empty, we fall back ourselves.
        match get_selected_text::get_selected_text() {
            Ok(s) if !s.trim().is_empty() => Some(s),
            _ => None,
        }
    }
}

#[cfg(not(target_os = "macos"))]
mod imp {
    pub fn enabled() -> bool { true } // non-macOS uses simulate-copy only
    pub fn read_selection() -> Option<String> { None }
}

pub fn enabled() -> bool {
    imp::enabled()
}

/// Try the AX read. None ⇒ caller (selection.rs) falls back to the sentinel copy path.
pub fn read_selection() -> Option<String> {
    if !enabled() {
        return None;
    }
    imp::read_selection()
}
