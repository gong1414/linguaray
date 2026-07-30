//! macOS system dictionary lookup (spec §E: word definitions where LLMs are weak).
//! Uses DCSCopyTextDefinition (CoreServices). Returns plain text or None.
//! Non-macOS: returns None (no system dict).

#[cfg(target_os = "macos")]
pub fn lookup(word: &str) -> Option<String> {
    use core_foundation::base::{TCFType, CFRange};
    use core_foundation::string::{CFString, CFStringRef};

    #[link(name = "CoreServices", kind = "framework")]
    extern "C" {
        fn DCSCopyTextDefinition(
            dict: *const std::ffi::c_void,
            text: CFStringRef,
            range: CFRange,
        ) -> CFStringRef;
    }

    unsafe {
        let cf_word = CFString::new(word);
        // DCSCopyTextDefinition interprets `range` in terms of the CFString's
        // length (UTF-16 code units, as returned by `char_len`). We pass the
        // whole string; pass a null dict to let the system pick the active
        // dictionary (the documented default behavior).
        let range = CFRange {
            location: 0,
            length: cf_word.char_len(),
        };
        let result = DCSCopyTextDefinition(std::ptr::null(), cf_word.as_concrete_TypeRef(), range);
        if result.is_null() {
            return None;
        }
        // Create-rule ownership: wrap_under_create_rule consumes the retain
        // produced by the Copy-function and releases it on drop.
        let def = CFString::wrap_under_create_rule(result).to_string();
        if def.is_empty() {
            None
        } else {
            Some(def)
        }
    }
}

#[cfg(not(target_os = "macos"))]
pub fn lookup(_word: &str) -> Option<String> {
    None
}
