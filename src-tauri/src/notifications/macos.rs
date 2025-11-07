/// macOS notification implementation using UNUserNotificationCenter
///
/// This provides native macOS notifications with click handling via
/// UNUserNotificationCenterDelegate.

use super::{ClickAction, NotificationClick, NotificationManager};
use std::sync::Arc;

#[cfg(target_os = "macos")]
use cocoa::base::{id, nil};
#[cfg(target_os = "macos")]
use cocoa::foundation::{NSString, NSAutoreleasePool};
#[cfg(target_os = "macos")]
use objc::declare::ClassDecl;
#[cfg(target_os = "macos")]
use objc::runtime::{Class, Object, Sel};
#[cfg(target_os = "macos")]
use objc::{class, msg_send, sel, sel_impl};
#[cfg(target_os = "macos")]
use block::ConcreteBlock;

#[cfg(target_os = "macos")]
static mut NOTIFICATION_MANAGER: Option<Arc<NotificationManager>> = None;

/// Setup macOS notification handling
#[cfg(target_os = "macos")]
pub fn setup(manager: Arc<NotificationManager>) {
    unsafe {
        NOTIFICATION_MANAGER = Some(manager);

        // Get UNUserNotificationCenter
        let center_class = class!(UNUserNotificationCenter);
        let center: id = msg_send![center_class, currentNotificationCenter];

        // Create and set our delegate
        let delegate = create_notification_delegate();
        let _: () = msg_send![center, setDelegate: delegate];

        println!("🍎 macOS notification system initialized with delegate");
    }
}

/// Create the UNUserNotificationCenterDelegate
#[cfg(target_os = "macos")]
unsafe fn create_notification_delegate() -> id {
    // Create delegate class if it doesn't exist
    let superclass = class!(NSObject);
    let mut decl = ClassDecl::new("CushionNotificationDelegate", superclass)
        .expect("Failed to create CushionNotificationDelegate class");

    // Add the delegate method for handling notification responses
    extern "C" fn did_receive_response(
        _self: &Object,
        _cmd: Sel,
        _center: id,
        response: id,
        completion_handler: id,
    ) {
        unsafe {
            // Get the notification response details
            let notification: id = msg_send![response, notification];
            let request: id = msg_send![notification, request];
            let identifier_ns: id = msg_send![request, identifier];
            let identifier = nsstring_to_string(identifier_ns);

            let action_identifier_ns: id = msg_send![response, actionIdentifier];
            let action_identifier = nsstring_to_string(action_identifier_ns);

            println!("🍎 Notification response - ID: {}, Action: {}", identifier, action_identifier);

            // Determine the action type
            let action = if action_identifier == "com.apple.UNNotificationDefaultActionIdentifier" {
                ClickAction::Body
            } else if action_identifier == "com.apple.UNNotificationDismissActionIdentifier" {
                ClickAction::Dismiss
            } else {
                ClickAction::Button(action_identifier)
            };

            // Get the URL from metadata if stored
            let url = if let Some(ref manager) = NOTIFICATION_MANAGER {
                manager.get_metadata(&identifier)
            } else {
                None
            };

            let click = NotificationClick {
                id: identifier,
                url,
                action,
            };

            // Call the notification manager's click handler
            if let Some(ref manager) = NOTIFICATION_MANAGER {
                manager.handle_click(click);
            }

            // Call the completion handler block properly
            // The completion handler is a block with signature: void (^)(void)
            let block: &block::Block<(), ()> = &*(completion_handler as *const _);
            block.call(());
        }
    }

    // Add method to class
    decl.add_method(
        sel!(userNotificationCenter:didReceiveNotificationResponse:withCompletionHandler:),
        did_receive_response as extern "C" fn(&Object, Sel, id, id, id),
    );

    // Register the class
    let delegate_class = decl.register();

    // Create an instance
    let delegate: id = msg_send![delegate_class, new];
    delegate
}

/// Show a notification on macOS
#[cfg(target_os = "macos")]
pub fn show_notification(
    id: String,
    title: String,
    body: String,
    _url: Option<String>,
) -> Result<(), String> {
    unsafe {
        let _pool = NSAutoreleasePool::new(nil);

        // Get UNUserNotificationCenter
        let center_class = class!(UNUserNotificationCenter);
        let center: id = msg_send![center_class, currentNotificationCenter];

        // Create notification content
        let content_class = class!(UNMutableNotificationContent);
        let content: id = msg_send![content_class, new];

        // Set title
        let title_ns = NSString::alloc(nil).init_str(&title);
        let _: () = msg_send![content, setTitle: title_ns];

        // Set body
        let body_ns = NSString::alloc(nil).init_str(&body);
        let _: () = msg_send![content, setBody: body_ns];

        // Create notification request
        let request_class = class!(UNNotificationRequest);
        let id_ns = NSString::alloc(nil).init_str(&id);
        let request: id = msg_send![
            request_class,
            requestWithIdentifier: id_ns
            content: content
            trigger: nil
        ];

        // Add notification request with completion handler
        let block = ConcreteBlock::new(|error: id| {
            if error != nil {
                let error_desc: id = msg_send![error, localizedDescription];
                let error_str = nsstring_to_string(error_desc);
                println!("❌ Failed to show macOS notification: {}", error_str);
            } else {
                println!("✅ macOS notification shown successfully");
            }
        });
        let block = block.copy();

        let _: () = msg_send![center, addNotificationRequest: request withCompletionHandler: &*block];

        Ok(())
    }
}

/// Helper to convert NSString to Rust String
#[cfg(target_os = "macos")]
unsafe fn nsstring_to_string(ns_string: id) -> String {
    if ns_string == nil {
        return String::new();
    }
    let c_str: *const i8 = msg_send![ns_string, UTF8String];
    if c_str.is_null() {
        return String::new();
    }
    std::ffi::CStr::from_ptr(c_str)
        .to_string_lossy()
        .into_owned()
}

#[cfg(not(target_os = "macos"))]
pub fn setup(_manager: Arc<NotificationManager>) {}

#[cfg(not(target_os = "macos"))]
pub fn show_notification(
    _id: String,
    _title: String,
    _body: String,
    _url: Option<String>,
) -> Result<(), String> {
    Err("macOS notifications not supported on this platform".to_string())
}
