/// macOS notification implementation using UNUserNotificationCenter
///
/// This provides native macOS notifications with click handling via
/// UNUserNotificationCenterDelegate.

use super::{ClickAction, NotificationClick, NotificationManager};
use std::sync::Arc;

#[cfg(target_os = "macos")]
use cocoa::base::{id, nil, BOOL, YES};
#[cfg(target_os = "macos")]
use cocoa::foundation::{NSString, NSAutoreleasePool, NSInteger, NSUInteger};
#[cfg(target_os = "macos")]
use objc::declare::ClassDecl;
#[cfg(target_os = "macos")]
use objc::runtime::{Class, Object, Sel};
#[cfg(target_os = "macos")]
use objc::{class, msg_send, sel, sel_impl};
#[cfg(target_os = "macos")]
use block::ConcreteBlock;
#[cfg(target_os = "macos")]
use std::sync::mpsc;

// UNAuthorizationOptions
#[cfg(target_os = "macos")]
const UN_AUTHORIZATION_OPTION_BADGE: NSUInteger = 1 << 0;
#[cfg(target_os = "macos")]
const UN_AUTHORIZATION_OPTION_SOUND: NSUInteger = 1 << 1;
#[cfg(target_os = "macos")]
const UN_AUTHORIZATION_OPTION_ALERT: NSUInteger = 1 << 2;

// UNAuthorizationStatus
#[cfg(target_os = "macos")]
const UN_AUTHORIZATION_STATUS_NOT_DETERMINED: NSInteger = 0;
#[cfg(target_os = "macos")]
const UN_AUTHORIZATION_STATUS_DENIED: NSInteger = 1;
#[cfg(target_os = "macos")]
const UN_AUTHORIZATION_STATUS_AUTHORIZED: NSInteger = 2;

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

/// Request notification permission from macOS
/// This will show the system permission dialog if permission hasn't been determined yet.
/// Returns "granted" or "denied".
#[cfg(target_os = "macos")]
pub fn request_notification_permission() -> Result<String, String> {
    unsafe {
        let _pool = NSAutoreleasePool::new(nil);

        // Get UNUserNotificationCenter
        let center_class = class!(UNUserNotificationCenter);
        let center: id = msg_send![center_class, currentNotificationCenter];

        // Create channel to receive result from completion handler
        let (tx, rx) = mpsc::channel::<(bool, Option<String>)>();

        // Request authorization with alert, sound, and badge options
        let options: NSUInteger = UN_AUTHORIZATION_OPTION_ALERT
            | UN_AUTHORIZATION_OPTION_SOUND
            | UN_AUTHORIZATION_OPTION_BADGE;

        let block = ConcreteBlock::new(move |granted: BOOL, error: id| {
            let error_msg = if error != nil {
                let error_desc: id = msg_send![error, localizedDescription];
                Some(nsstring_to_string(error_desc))
            } else {
                None
            };
            let _ = tx.send((granted == YES, error_msg));
        });
        let block = block.copy();

        println!("🔔 Requesting notification permission...");
        let _: () = msg_send![center, requestAuthorizationWithOptions: options completionHandler: &*block];

        // Use timeout instead of blocking forever - the completion handler may not fire
        // if the main run loop isn't pumping
        match rx.recv_timeout(std::time::Duration::from_secs(30)) {
            Ok((granted, error_msg)) => {
                if let Some(err) = error_msg {
                    println!("❌ Permission request error: {}", err);
                    return Err(err);
                }
                let status = if granted { "granted" } else { "denied" };
                println!("🔔 Notification permission: {}", status);
                Ok(status.to_string())
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {
                println!("⚠️ Permission request timed out - checking current status");
                // Fall back to checking current authorization status
                get_current_authorization_status()
            }
            Err(e) => {
                println!("❌ Failed to receive permission result: {}", e);
                Err(format!("Failed to receive permission result: {}", e))
            }
        }
    }
}

/// Get current notification authorization status without prompting
#[cfg(target_os = "macos")]
fn get_current_authorization_status() -> Result<String, String> {
    unsafe {
        let _pool = NSAutoreleasePool::new(nil);

        let center_class = class!(UNUserNotificationCenter);
        let center: id = msg_send![center_class, currentNotificationCenter];

        let (tx, rx) = mpsc::channel::<NSInteger>();

        let block = ConcreteBlock::new(move |settings: id| {
            let status: NSInteger = msg_send![settings, authorizationStatus];
            let _ = tx.send(status);
        });
        let block = block.copy();

        let _: () = msg_send![center, getNotificationSettingsWithCompletionHandler: &*block];

        match rx.recv_timeout(std::time::Duration::from_secs(5)) {
            Ok(status) => {
                let status_str = match status {
                    UN_AUTHORIZATION_STATUS_NOT_DETERMINED => {
                        println!("🔔 Authorization status: not_determined (permission dialog didn't show)");
                        "denied" // Treat as denied if we couldn't prompt
                    }
                    UN_AUTHORIZATION_STATUS_DENIED => {
                        println!("🔔 Authorization status: denied");
                        "denied"
                    }
                    UN_AUTHORIZATION_STATUS_AUTHORIZED => {
                        println!("🔔 Authorization status: granted");
                        "granted"
                    }
                    _ => {
                        println!("🔔 Authorization status: unknown ({})", status);
                        "denied"
                    }
                };
                Ok(status_str.to_string())
            }
            Err(_) => {
                println!("❌ Failed to get notification settings (timed out)");
                Err("Failed to get notification settings".to_string())
            }
        }
    }
}

#[cfg(not(target_os = "macos"))]
pub fn request_notification_permission() -> Result<String, String> {
    // On non-macOS platforms, return granted (Linux doesn't require permission)
    Ok("granted".to_string())
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
