/// macOS App Nap Prevention Module
///
/// This module prevents macOS from throttling the app (App Nap) when it's
/// backgrounded or hidden. This is critical for maintaining WebSocket connections
/// and keeping JavaScript timers (like keepalive intervals) running reliably.
///
/// Uses NSProcessInfo.beginActivityWithOptions with NSActivityUserInitiated flag.
/// This prevents App Nap but DOES NOT prevent the Mac from sleeping normally.

#[cfg(target_os = "macos")]
use cocoa::base::{id, nil};
#[cfg(target_os = "macos")]
use cocoa::foundation::NSString;
#[cfg(target_os = "macos")]
use objc::{class, msg_send, sel, sel_impl};

#[cfg(target_os = "macos")]
const NS_ACTIVITY_USER_INITIATED: u64 = 0x00FFFFFF;
#[cfg(target_os = "macos")]
const NS_ACTIVITY_LATENCY_CRITICAL: u64 = 0xFF00000000;

/// Prevents the app from being put to sleep by macOS App Nap.
///
/// Returns an activity object that must be kept alive for the duration of the app.
/// If dropped, the activity assertion is released and App Nap may resume.
///
/// # Returns
/// - `Some(id)` - Activity object on macOS (must be stored to keep assertion active)
/// - `None` - On non-macOS platforms
#[cfg(target_os = "macos")]
pub fn prevent_app_nap() -> Option<id> {
    unsafe {
        // Get the shared NSProcessInfo instance
        let process_info_class = class!(NSProcessInfo);
        let process_info: id = msg_send![process_info_class, processInfo];

        if process_info == nil {
            eprintln!("[AppNap] Failed to get NSProcessInfo");
            return None;
        }

        // Create reason string
        let reason = NSString::alloc(nil).init_str(
            "Maintaining WebSocket connection for real-time notifications"
        );

        // Activity options: User-initiated + latency-critical (prevents App Nap, allows Mac to sleep)
        // NSActivityLatencyCritical is needed to fully prevent App Nap in Activity Monitor
        let options = NS_ACTIVITY_USER_INITIATED | NS_ACTIVITY_LATENCY_CRITICAL;

        // Begin activity
        let activity: id = msg_send![
            process_info,
            beginActivityWithOptions: options
            reason: reason
        ];

        if activity == nil {
            eprintln!("[AppNap] Failed to begin activity - assertion not created");
            return None;
        }

        println!("[AppNap] ✓ Activity assertion started successfully");
        println!("[AppNap]   Options: NSActivityUserInitiated | NSActivityLatencyCritical");
        println!("[AppNap]   This prevents App Nap but allows Mac to sleep normally");
        println!("[AppNap]   Reason: Maintaining WebSocket connection for real-time notifications");

        // Return the activity object - must be stored to keep assertion active!
        Some(activity)
    }
}

/// No-op for non-macOS platforms
#[cfg(not(target_os = "macos"))]
pub fn prevent_app_nap() -> Option<()> {
    println!("[AppNap] Not on macOS - App Nap prevention not needed");
    None
}

/// Ends the activity assertion (allows App Nap to resume).
///
/// Note: This is called automatically when the activity object is dropped,
/// so explicit calls are usually not needed.
#[cfg(target_os = "macos")]
pub fn end_activity(activity: id) {
    if activity == nil {
        return;
    }

    unsafe {
        let process_info_class = class!(NSProcessInfo);
        let process_info: id = msg_send![process_info_class, processInfo];

        if process_info != nil {
            let _: () = msg_send![process_info, endActivity: activity];
            println!("[AppNap] Activity assertion ended");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(target_os = "macos")]
    fn test_prevent_app_nap() {
        let activity = prevent_app_nap();
        assert!(activity.is_some(), "Should create activity on macOS");
    }

    #[test]
    #[cfg(not(target_os = "macos"))]
    fn test_prevent_app_nap_non_macos() {
        let result = prevent_app_nap();
        assert!(result.is_none(), "Should return None on non-macOS");
    }
}
