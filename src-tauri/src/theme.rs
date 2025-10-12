/// Theme detection and styling for macOS
#[cfg(target_os = "macos")]
use cocoa::appkit::NSWindow;
#[cfg(target_os = "macos")]
use cocoa::base::id;
#[cfg(target_os = "macos")]
use objc::{class, msg_send, sel, sel_impl};

#[cfg(target_os = "macos")]
pub fn is_dark_mode() -> bool {
    use cocoa::base::nil;
    use cocoa::foundation::NSString;

    unsafe {
        let app: id = msg_send![class!(NSApplication), sharedApplication];
        let appearance: id = msg_send![app, effectiveAppearance];
        let appearance_name: id = msg_send![appearance, name];

        if appearance_name != nil {
            let name_str = NSString::UTF8String(appearance_name);
            let name = std::ffi::CStr::from_ptr(name_str).to_string_lossy();
            name.contains("Dark")
        } else {
            false
        }
    }
}

#[cfg(target_os = "macos")]
pub fn update_window_background_for_theme(ns_window: id) {
    let is_dark = is_dark_mode();

    unsafe {
        // Set background color based on theme
        // Light mode: hsl(0, 0%, 98%) = rgb(250, 250, 250) = 0.98
        // Dark mode: hsl(0, 0%, 6%) = rgb(15, 15, 15) = 0.06
        let (r, g, b) = if is_dark {
            (0.06, 0.06, 0.06)
        } else {
            (0.98, 0.98, 0.98)
        };

        let bg_color: id = msg_send![class!(NSColor), colorWithRed:r green:g blue:b alpha:1.0];
        let _: () = msg_send![ns_window, setBackgroundColor: bg_color];

        // Update border color for theme
        let content_view: id = ns_window.contentView();
        if !content_view.is_null() {
            let layer: id = msg_send![content_view, layer];
            if !layer.is_null() {
                let border_color: id = if is_dark {
                    msg_send![class!(NSColor), colorWithRed:1.0 green:1.0 blue:1.0 alpha:0.1]
                } else {
                    msg_send![class!(NSColor), colorWithRed:0.07 green:0.07 blue:0.07 alpha:0.1]
                };
                let cg_color: *mut std::ffi::c_void = msg_send![border_color, CGColor];
                let _: () = msg_send![layer, setBorderColor: cg_color];
            }
        }
    }
}

// Stub implementations for non-macOS platforms
#[cfg(not(target_os = "macos"))]
pub fn is_dark_mode() -> bool {
    false
}

#[cfg(not(target_os = "macos"))]
pub fn update_window_background_for_theme(_window: ()) {
    // No-op on non-macOS platforms
}
