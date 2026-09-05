#[cfg(target_os = "macos")]
mod macos {
    use std::ffi::{c_void, CString};
    use std::process::Command;

    use objc2::{msg_send, runtime::AnyObject};

    #[link(name = "ApplicationServices", kind = "framework")]
    extern "C" {
        fn AXIsProcessTrusted() -> bool;
        fn AXIsProcessTrustedWithOptions(options: *mut c_void) -> bool;
    }

    #[link(name = "CoreGraphics", kind = "framework")]
    extern "C" {
        fn CGPreflightScreenCaptureAccess() -> bool;
        fn CGRequestScreenCaptureAccess() -> bool;
    }

    unsafe fn cocoa_string(value: &str) -> *mut AnyObject {
        let bytes = CString::new(value).expect("privacy pane anchor contains no null bytes");
        msg_send![objc2::class!(NSString), stringWithUTF8String: bytes.as_ptr()]
    }

    fn show_privacy_settings(anchor: &str) {
        let url = format!("x-apple.systempreferences:com.apple.preference.security?{anchor}");
        let _ = Command::new("open").arg(url).spawn();
    }

    pub fn accessibility_granted() -> bool {
        unsafe { AXIsProcessTrusted() }
    }

    pub fn request_accessibility(open_settings_only: bool) {
        if !open_settings_only {
            unsafe {
                let prompt: *mut AnyObject =
                    msg_send![objc2::class!(NSNumber), numberWithBool: true];
                let options: *mut AnyObject = msg_send![
                    objc2::class!(NSDictionary),
                    dictionaryWithObject: prompt,
                    forKey: cocoa_string("AXTrustedCheckOptionPrompt")
                ];
                AXIsProcessTrustedWithOptions(options.cast());
            }
        }
        show_privacy_settings("Privacy_Accessibility");
    }

    pub fn screen_recording_granted() -> bool {
        unsafe { CGPreflightScreenCaptureAccess() }
    }

    pub fn request_screen_recording(open_settings_only: bool) {
        if !open_settings_only && !screen_recording_granted() {
            unsafe {
                CGRequestScreenCaptureAccess();
            }
        }
        show_privacy_settings("Privacy_ScreenCapture");
    }
}

pub fn is_accessibility_permission_granted() -> bool {
    #[cfg(target_os = "macos")]
    return macos::accessibility_granted();
    #[cfg(target_os = "windows")]
    return true;
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    false
}

pub fn request_accessibility_permission(open_settings_only: bool) {
    #[cfg(target_os = "macos")]
    macos::request_accessibility(open_settings_only);
    #[cfg(not(target_os = "macos"))]
    let _ = open_settings_only;
}

pub fn is_screen_recording_permission_granted() -> bool {
    #[cfg(target_os = "macos")]
    return macos::screen_recording_granted();
    #[cfg(target_os = "windows")]
    return true;
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    false
}

pub fn request_screen_recording_permission(open_settings_only: bool) {
    #[cfg(target_os = "macos")]
    macos::request_screen_recording(open_settings_only);
    #[cfg(not(target_os = "macos"))]
    let _ = open_settings_only;
}
