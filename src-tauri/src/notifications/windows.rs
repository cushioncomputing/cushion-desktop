/// Windows notification implementation using tauri-winrt-notification
///
/// This provides native Windows toast notifications with click handling.

use super::{ClickAction, NotificationClick, NotificationManager};
use std::sync::Arc;
use tauri::AppHandle;

#[cfg(target_os = "windows")]
use tauri_winrt_notification::{Duration, Sound, Toast};

/// Setup Windows notification handling
#[cfg(target_os = "windows")]
pub fn setup(_manager: Arc<NotificationManager>) {
    println!("🪟 Windows notification system initialized");
    // No global setup needed for Windows - we handle clicks per-notification
}

/// Show a notification on Windows
#[cfg(target_os = "windows")]
pub fn show_notification(
    app: AppHandle,
    id: String,
    title: String,
    body: String,
    url: Option<String>,
) -> Result<(), String> {
    // Get the app's bundle identifier for the toast
    let app_id = app.config()
        .identifier
        .clone();

    // Get the manager to call handle_click
    let manager = NotificationManager::get()
        .ok_or_else(|| "Notification manager not initialized".to_string())?;

    let notification_id = id.clone();
    let notification_url = url.clone();

    // Create and show the toast
    Toast::new(&app_id)
        .title(&title)
        .text1(&body)
        .sound(Some(Sound::Default))
        .duration(Duration::Short)
        .on_activated(move |action| {
            println!("🪟 Windows toast activated: {:?}", action);

            let click = NotificationClick {
                id: notification_id.clone(),
                url: notification_url.clone(),
                action: match action {
                    None => ClickAction::Body,
                    Some(btn) => ClickAction::Button(btn),
                },
            };

            manager.handle_click(click);
        })
        .show()
        .map_err(|e| format!("Failed to show Windows notification: {}", e))?;

    Ok(())
}

#[cfg(not(target_os = "windows"))]
pub fn setup(_manager: Arc<NotificationManager>) {}

#[cfg(not(target_os = "windows"))]
pub fn show_notification(
    _app: AppHandle,
    _id: String,
    _title: String,
    _body: String,
    _url: Option<String>,
) -> Result<(), String> {
    Err("Windows notifications not supported on this platform".to_string())
}
