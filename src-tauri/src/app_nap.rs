/// macOS App Nap Prevention Module
///
/// This module prevents macOS from throttling the app (App Nap) when it's
/// backgrounded or hidden. This is critical for maintaining WebSocket connections
/// and keeping JavaScript timers (like keepalive intervals) running reliably.
///
/// Uses NSProcessInfo.beginActivityWithOptions with the correct flag to prevent
/// App Nap while allowing the Mac to sleep normally (same approach as Slack).

#[cfg(target_os = "macos")]
use cocoa::base::{id, nil};
#[cfg(target_os = "macos")]
use cocoa::foundation::NSString;
#[cfg(target_os = "macos")]
use objc::{class, msg_send, sel, sel_impl};

// NSActivityOptions flag values from NSProcessInfo.h:
// NSActivityUserInitiatedAllowingIdleSystemSleep = 0x00EFFFFF
// = (NSActivityUserInitiated & ~NSActivityIdleSystemSleepDisabled)
//
// This flag is EXPLICITLY DESIGNED by Apple to:
// - Prevent App Nap and keep app responsive
// - Allow idle system sleep (does NOT include NSActivityIdleSystemSleepDisabled bit 20)
// - Allow idle display sleep (does NOT include NSActivityIdleDisplaySleepDisabled bit 40)
//
// Apple docs: "NSActivityUserInitiatedAllowingIdleSystemSleep allows the Mac to go to
// sleep according to the user's Energy Saver System Preferences"
#[cfg(target_os = "macos")]
const NS_ACTIVITY_USER_INITIATED_ALLOWING_IDLE_SYSTEM_SLEEP: u64 = 0x00EFFFFF;

/// Prevents the app from being put to sleep by macOS App Nap.
///
/// Uses NSProcessInfo.beginActivityWithOptions with NSActivityUserInitiatedAllowingIdleSystemSleep,
/// which is Apple's recommended flag for preventing App Nap while allowing the Mac to sleep
/// normally according to Energy Saver settings.
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

        // NSActivityUserInitiatedAllowingIdleSystemSleep (0x00EFFFFF):
        // Apple's official flag for preventing App Nap while allowing system sleep.
        // From NSProcessInfo.h: (NSActivityUserInitiated & ~NSActivityIdleSystemSleepDisabled)
        // - Prevents App Nap throttling (keeps JavaScript timers and WebSocket alive)
        // - Does NOT include bit 20 (NSActivityIdleSystemSleepDisabled) - allows system sleep
        // - Does NOT include bit 40 (NSActivityIdleDisplaySleepDisabled) - allows display sleep
        let options = NS_ACTIVITY_USER_INITIATED_ALLOWING_IDLE_SYSTEM_SLEEP;

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
        println!("[AppNap]   Flag: NSActivityUserInitiatedAllowingIdleSystemSleep (0x00EFFFFF)");
        println!("[AppNap]   This prevents App Nap while allowing display sleep and system sleep");
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
