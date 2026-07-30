//! macOS Accessibility (AX) — §B hybrid capture: read the focused element's
//! selected text directly via AX FIRST (no clipboard touch), falling back to the
//! sentinel simulate-copy path in `selection.rs` when AX is untrusted or the app
//! doesn't expose a selection.
//!
//! Uses accessibility-sys (raw FFI) + core-foundation, mirroring the C flow:
//! AXUIElementCreateSystemWide → kAXFocusedUIElementAttribute → kAXSelectedTextAttribute.
//! Non-macOS: AX is unavailable; `read_selection` returns None (caller uses copy-fallback).

#[cfg(target_os = "macos")]
mod imp {
    use accessibility_sys::*;
    use core_foundation::base::{CFRelease, FromVoid, TCFType};
    use core_foundation::string::{CFString, CFStringRef};
    use core_foundation_sys::base::{CFGetTypeID, CFTypeRef};
    use core_foundation_sys::string::CFStringGetTypeID;

    // Attribute name constants as CFStrings (created per call; cheap enough).
    fn attr(name: &str) -> CFString {
        CFString::new(name)
    }

    /// Is this process trusted (Accessibility granted)? AXIsProcessTrustOptions is
    /// newer; AXIsProcessTrusted is the classic call and exists in ApplicationServices.
    pub fn enabled() -> bool {
        unsafe { AXIsProcessTrusted() }
    }

    /// Read the system-wide focused element's selected text via AX. None if AX is
    /// unavailable, untrusted, or the focused element exposes no selection.
    pub fn read_selection() -> Option<String> {
        unsafe {
            let system = AXUIElementCreateSystemWide();
            if system.is_null() {
                return None;
            }
            // focused element
            let focused_attr = attr("AXFocusedUIElement").as_concrete_TypeRef();
            let mut focused: CFTypeRef = std::ptr::null_mut();
            let r1 = AXUIElementCopyAttributeValue(system, focused_attr, &mut focused);
            if r1 != 0 || focused.is_null() {
                CFRelease(system.cast());
                return None;
            }
            // selected text on the focused element
            let sel_attr = attr("AXSelectedText").as_concrete_TypeRef();
            let mut value: CFTypeRef = std::ptr::null_mut();
            let r2 = AXUIElementCopyAttributeValue(focused as AXUIElementRef, sel_attr, &mut value);
            CFRelease(focused);
            CFRelease(system.cast());
            if r2 != 0 || value.is_null() {
                return None;
            }
            // value should be a CFString
            let type_id = CFGetTypeID(value);
            let str_type_id = CFStringGetTypeID();
            let out = if type_id == str_type_id {
                let s = CFString::wrap_under_get_rule(value as CFStringRef);
                Some(s.to_string())
            } else {
                None
            };
            CFRelease(value);
            out.filter(|s| !s.is_empty())
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

/// Try the AX direct-read. None ⇒ caller falls back to the sentinel copy path.
/// (spec §B: AX-first, copy-fallback.)
pub fn read_selection() -> Option<String> {
    if !enabled() {
        return None;
    }
    imp::read_selection()
}
