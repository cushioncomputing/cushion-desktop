/// macOS-specific notification handling using UNUserNotificationCenter
/// This module provides native notification click handling to bring the app to foreground
use cocoa::base::{id, nil};
use cocoa::foundation::NSString;
use objc::declare::ClassDecl;
use objc::runtime::{Class, Object, Sel};
use objc::{class, msg_send, sel, sel_impl};
use std::sync::Once;
use tauri::{AppHandle, Manager};

type CompletionBlock = *mut block::Block<(), ()>;

static INIT_DELEGATE: Once = Once::new();

/// Setup the UNUserNotificationCenter delegate to handle notification clicks
pub fn setup_notification_delegate(app_handle: &AppHandle) {
    unsafe {
        // Get UNUserNotificationCenter singleton
        let center_class = class!(UNUserNotificationCenter);
        let center: id = msg_send![center_class, currentNotificationCenter];

        if center == nil {
            println!("❌ Failed to get UNUserNotificationCenter");
            return;
        }

        // Create and set our custom delegate
        let delegate = create_delegate(app_handle.clone());
        let _: () = msg_send![center, setDelegate: delegate];

        println!("✅ UNUserNotificationCenter delegate set up successfully");
    }
}

/// Create a custom delegate class that implements UNUserNotificationCenterDelegate
unsafe fn create_delegate(app_handle: AppHandle) -> id {
    INIT_DELEGATE.call_once(|| {
        // Define a new Objective-C class
        let superclass = class!(NSObject);
        let mut decl = ClassDecl::new("CushionNotificationDelegate", superclass).unwrap();

        // Add the app_handle as an instance variable
        decl.add_ivar::<*mut std::ffi::c_void>("_app_handle");

        // Implement userNotificationCenter:didReceiveNotificationResponse:withCompletionHandler:
        extern "C" fn did_receive_response(
            this: &Object,
            _cmd: Sel,
            _center: id,
            _response: id,
            completion_handler: id,
        ) {
            unsafe {
                println!("🔔 Notification clicked! Bringing app to foreground...");

                // Get the app handle from the instance variable
                let app_handle_ptr: *mut std::ffi::c_void = *this.get_ivar("_app_handle");
                if !app_handle_ptr.is_null() {
                    let app_handle = &*(app_handle_ptr as *const AppHandle);

                    // Activate the app and bring it to the foreground
                    activate_app();

                    // Show and focus the main window
                    if let Some(window) = app_handle.get_webview_window("main") {
                        let _ = window.show();
                        let _ = window.unminimize();
                        let _ = window.set_focus();
                        println!("✅ Window brought to foreground");
                    }
                }

                // Call the completion handler block
                if completion_handler != nil {
                    let block = completion_handler as CompletionBlock;
                    (*block).call(());
                }
            }
        }

        // Implement userNotificationCenter:willPresentNotification:withCompletionHandler:
        // This allows notifications to be shown even when the app is in the foreground
        extern "C" fn will_present_notification(
            _this: &Object,
            _cmd: Sel,
            _center: id,
            _notification: id,
            completion_handler: id,
        ) {
            unsafe {
                // UNNotificationPresentationOptionBanner = 1 << 0 = 1
                // UNNotificationPresentationOptionSound = 1 << 1 = 2
                // UNNotificationPresentationOptionBadge = 1 << 2 = 4
                // Combined: 1 | 2 = 3 (banner + sound)
                let options: u64 = 3; // Show banner and play sound

                if completion_handler != nil {
                    // The completion handler for willPresent takes UNNotificationPresentationOptions (u64)
                    type PresentBlock = *mut block::Block<(u64,), ()>;
                    let block = completion_handler as PresentBlock;
                    (*block).call((options,));
                }
            }
        }

        unsafe {
            decl.add_method(
                sel!(userNotificationCenter:didReceiveNotificationResponse:withCompletionHandler:),
                did_receive_response as extern "C" fn(&Object, Sel, id, id, id),
            );

            decl.add_method(
                sel!(userNotificationCenter:willPresentNotification:withCompletionHandler:),
                will_present_notification as extern "C" fn(&Object, Sel, id, id, id),
            );
        }

        decl.register();
    });

    // Create an instance of our delegate class
    let delegate_class = class!(CushionNotificationDelegate);
    let delegate: id = msg_send![delegate_class, alloc];
    let delegate: id = msg_send![delegate, init];

    // Store the app_handle in the instance variable
    let app_handle_box = Box::new(app_handle);
    let app_handle_ptr = Box::into_raw(app_handle_box) as *mut std::ffi::c_void;
    (*delegate).set_ivar("_app_handle", app_handle_ptr);

    delegate
}

/// Activate the macOS application and bring it to the foreground
unsafe fn activate_app() {
    let ns_app_class = class!(NSApplication);
    let ns_app: id = msg_send![ns_app_class, sharedApplication];

    if ns_app != nil {
        // NSApplicationActivationPolicyRegular = 0
        let policy: i64 = 0;
        let _: () = msg_send![ns_app, setActivationPolicy: policy];

        // Activate the app, ignoring other apps
        let _: () = msg_send![ns_app, activateIgnoringOtherApps: true];

        println!("✅ App activated");
    }
}
