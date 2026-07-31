//! Vendored AX-only selection read, adapted from yetone/get-selected-text (MIT OR
//! Apache-2.0; see LICENSE-MIT / LICENSE-APACHE in this directory).
#![allow(clippy::doc_lazy_continuation)]
//!
//! IMPORTANT: this is the AX DIRECT-READ ONLY. The upstream `get_selected_text()`
//! falls back to an AppleScript copy that clobbers the clipboard; we deliberately
//! do NOT include that here. §B's copy-fallback (with the sentinel state machine
//! + clipboard restore) lives in selection.rs / selection_engine.rs, which we own.
//! AX-only here means the clipboard is never touched on the AX-success path.
//!
//! Adapted changes: stripped the clipboard/AppleScript fallback + the active-window
//! cache (we don't need per-app caching); kept the AXUIElement focused/selected
//! read verbatim.

#[cfg(target_os = "macos")]
pub fn read_selected_text_ax() -> Option<String> {
    use accessibility_ng::{AXAttribute, AXUIElement};
    use accessibility_sys_ng::{kAXFocusedUIElementAttribute, kAXSelectedTextAttribute};
    use core_foundation::string::CFString;

    let system_element = AXUIElement::system_wide();
    let focused = system_element
        .attribute(&AXAttribute::new(&CFString::from_static_string(
            kAXFocusedUIElementAttribute,
        )))
        .ok()?
        .downcast_into::<AXUIElement>()?;
    let text = focused
        .attribute(&AXAttribute::new(&CFString::from_static_string(
            kAXSelectedTextAttribute,
        )))
        .ok()?
        .downcast_into::<CFString>()?;
    let s = text.to_string();
    if s.is_empty() { None } else { Some(s) }
}

#[cfg(not(target_os = "macos"))]
pub fn read_selected_text_ax() -> Option<String> {
    None
}
