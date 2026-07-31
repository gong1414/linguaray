//! macOS Accessibility — §B hybrid capture: read the focused element's selected
//! text directly FIRST (no clipboard touch), falling back to the sentinel
//! simulate-copy path in `selection.rs` when the read returns nothing.
//!
//! Per the approved spec §B decision ("vendor `get-selected-text`, reject self-
//! impl"), the AX read is vendored from yetone/get-selected-text (MIT, see
//! vendor/get-selected-text-ax/LICENSE-MIT). The vendored module exposes ONLY the
//! AX direct-read; the upstream's AppleScript copy-fallback is deliberately
//! excluded — §B's copy-fallback (with sentinel state machine + clipboard restore)
//! lives in selection.rs, which we own.
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
