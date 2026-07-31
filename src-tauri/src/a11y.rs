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

#[path = "../vendor/get-selected-text-ax/macos_ax.rs"]
mod vendored_ax;

#[cfg(target_os = "macos")]
mod imp {
    /// Is this process trusted (Accessibility granted)? AXIsProcessTrusted from
    /// ApplicationServices. Used for the onboarding banner, not for the read itself.
    pub fn enabled() -> bool {
        unsafe { accessibility_sys::AXIsProcessTrusted() }
    }

    /// AX DIRECT-READ ONLY (vendored from yetone/get-selected-text, see
    /// vendor/get-selected-text-ax/). The upstream's AppleScript copy-fallback is
    /// deliberately NOT included — it clobbers the clipboard and bypasses §B's
    /// sentinel/restore discipline. Our §B copy-fallback lives in selection.rs.
    /// None if AX errs/empty ⇒ selection.rs does the sentinel copy-fallback.
    pub fn read_selection() -> Option<String> {
        super::vendored_ax::read_selected_text_ax()
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
