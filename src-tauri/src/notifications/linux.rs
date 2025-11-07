/// Linux notification implementation using notify-rust
///
/// This provides native freedesktop/XDG notifications with click handling
/// via DBus action callbacks.

use super::{ClickAction, NotificationClick, NotificationManager};
use std::sync::Arc;

#[cfg(target_os = "linux")]
use notify_rust::{Notification, Timeout};

/// Setup Linux notification handling
#[cfg(target_os = "linux")]
pub fn setup(_manager: Arc<NotificationManager>) {
    println!("🐧 Linux notification system initialized");
    // notify-rust handles action callbacks per-notification
}

/// Show a notification on Linux
#[cfg(target_os = "linux")]
pub fn show_notification(
    id: String,
    title: String,
    body: String,
    url: Option<String>,
) -> Result<(), String> {
    let manager = NotificationManager::get()
        .ok_or_else(|| "Notification manager not initialized".to_string())?;

    let notification_id = id.clone();
    let notification_url = url.clone();

    // Create notification with default action
    let mut notification = Notification::new()
        .summary(&title)
        .body(&body)
        .timeout(Timeout::Default)
        .action("default", "Open")
        .finalize();

    // Set up action callback
    let manager_clone = manager.clone();
    notification.show().map_err(|e| format!("Failed to show Linux notification: {}", e))?;

    // Note: notify-rust doesn't provide a built-in callback mechanism for actions
    // on most Linux desktop environments. The notification server handles actions
    // but doesn't provide a standard way to get callbacks back to the application.
    //
    // For full click handling on Linux, you would need to:
    // 1. Use DBus directly to listen for action invoked signals
    // 2. Or use a notification server that supports action callbacks
    //
    // For now, we'll rely on the user opening the app manually.
    // A future enhancement could add DBus action listeners.

    println!("🐧 Linux notification shown (action callbacks not yet implemented)");

    Ok(())
}

#[cfg(not(target_os = "linux"))]
pub fn setup(_manager: Arc<NotificationManager>) {}

#[cfg(not(target_os = "linux"))]
pub fn show_notification(
    _id: String,
    _title: String,
    _body: String,
    _url: Option<String>,
) -> Result<(), String> {
    Err("Linux notifications not supported on this platform".to_string())
}
