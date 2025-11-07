/// Notification commands
use tauri_plugin_notification::NotificationExt;
use tauri::{Emitter, Listener, Manager};
use std::sync::Mutex;
use std::collections::HashMap;

// Global state to store notification title -> URL mappings
lazy_static::lazy_static! {
    static ref NOTIFICATION_URLS: Mutex<HashMap<String, String>> = Mutex::new(HashMap::new());
}

/// Get the URL associated with a notification title (if any)
/// This is used by the native macOS notification handler
pub fn get_notification_url(title: &str) -> Option<String> {
    if let Ok(urls) = NOTIFICATION_URLS.lock() {
        urls.get(title).cloned()
    } else {
        None
    }
}

#[tauri::command]
pub async fn show_notification(
    app: tauri::AppHandle,
    title: String,
    body: String,
    url: Option<String>
) -> Result<(), String> {
    println!("📱 Rust: Attempting to show notification: '{}' - '{}' (url: {:?})", title, body, url);

    // If URL is provided, store the mapping for later retrieval when notification is clicked
    if let Some(notification_url) = url.clone() {
        if let Ok(mut urls) = NOTIFICATION_URLS.lock() {
            urls.insert(title.clone(), notification_url.clone());
            println!("📝 Rust: Stored URL mapping: '{}' -> '{}'", title, notification_url);
        }
    }

    // Create and send the notification with sound
    let notification_builder = app.notification()
        .builder()
        .title(title.clone())
        .body(body.clone())
        .sound("default");

    match notification_builder.show() {
        Ok(_) => {
            println!("✅ Rust: Notification sent successfully!");
            Ok(())
        },
        Err(e) => {
            println!("❌ Rust: Failed to send notification: {}", e);
            println!("❌ Rust: Error type: {:?}", std::any::type_name_of_val(&e));
            Err(format!("Notification error: {}", e))
        }
    }
}

/// Set up global notification click handler
/// This should be called once during app setup in lib.rs
pub fn setup_notification_click_handler(app: &tauri::AppHandle) {
    let app_clone = app.clone();

    // Listen for notification click events
    // Note: The exact event name may vary based on tauri-plugin-notification version
    app.listen("notification://clicked", move |event| {
        let payload = event.payload();
        println!("🔔 Rust: Notification clicked, payload: {}", payload);

        // Try to extract the title from the payload
        // The payload format may vary, so we'll try to match against stored titles
        if let Ok(urls) = NOTIFICATION_URLS.lock() {
            for (title, url) in urls.iter() {
                if payload.contains(title) {
                    println!("🔗 Rust: Matched notification '{}', emitting deep link: {}", title, url);

                    // Emit deep-link event (same as deep link handler in lib.rs)
                    let _ = app_clone.emit("deep-link", url.clone());

                    // Show and focus the window
                    if let Some(window) = app_clone.get_webview_window("main") {
                        let _ = window.show();
                        let _ = window.set_focus();
                        let _ = window.unminimize();
                    }

                    break;
                }
            }
        }
    });
}

#[tauri::command]
pub async fn check_notification_permission(app: tauri::AppHandle) -> Result<String, String> {
    println!("🔍 Rust: Checking notification permission...");

    // Try to send a test notification to see if permission is granted
    match app.notification()
        .builder()
        .title("Permission Check")
        .body("Testing notification permissions")
        .show() {
        Ok(_) => {
            println!("✅ Rust: Permission check successful!");
            Ok("granted".to_string())
        },
        Err(e) => {
            println!("❌ Rust: Permission check failed: {}", e);
            Ok("denied".to_string())
        }
    }
}
