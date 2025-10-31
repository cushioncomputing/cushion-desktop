/// Notification commands
use tauri_plugin_notification::{NotificationExt, PermissionState};
use tauri::{Emitter, Manager};
use std::sync::Mutex;
use std::collections::HashMap;
use serde_json::json;

// Global state to store notification ID -> URL mappings
lazy_static::lazy_static! {
    static ref NOTIFICATION_URLS: Mutex<HashMap<String, String>> = Mutex::new(HashMap::new());
}

#[tauri::command]
pub async fn show_notification(
    app: tauri::AppHandle,
    title: String,
    body: String,
    url: Option<String>
) -> Result<(), String> {
    println!("📱 Rust: Attempting to show notification: '{}' - '{}' (url: {:?})", title, body, url);

    // Generate unique notification identifier
    let notification_id = format!("cushion-notif-{}", chrono::Utc::now().timestamp_millis());

    // If URL is provided, store the mapping for click handling
    if let Some(notification_url) = url.clone() {
        if let Ok(mut urls) = NOTIFICATION_URLS.lock() {
            urls.insert(notification_id.clone(), notification_url.clone());
            println!("📝 Rust: Stored URL mapping: '{}' -> '{}'", notification_id, notification_url);
        }
    }

    // Create and send the notification with identifier
    let notification_builder = app.notification().builder()
        .title(title.clone())
        .body(body.clone())
        .identifier(&notification_id);

    match notification_builder.show() {
        Ok(_) => {
            println!("✅ Rust: Notification sent successfully with ID: {}", notification_id);
            Ok(())
        },
        Err(e) => {
            println!("❌ Rust: Failed to send notification: {}", e);
            println!("❌ Rust: Error type: {:?}", std::any::type_name_of_val(&e));
            Err(format!("Notification error: {}", e))
        }
    }
}

/// Set up global notification click handler using notification plugin's onAction
/// This should be called once during app setup in lib.rs
pub fn setup_notification_click_handler(app: &tauri::AppHandle) {
    let app_clone = app.clone();

    // Listen for notification action events from the plugin
    // When a notification is clicked, the plugin emits an action event
    app.notification().on_action(move |notification_id| {
        println!("🔔 Rust: Notification clicked with ID: {}", notification_id);

        // Look up the URL associated with this notification ID
        if let Ok(mut urls) = NOTIFICATION_URLS.lock() {
            if let Some(url) = urls.remove(&notification_id) {
                println!("🔗 Rust: Found URL for notification '{}': {}", notification_id, url);

                // Convert HTTPS URL to cushion:// deep link scheme
                // Example: https://app.cushion.so/team/post/123 -> cushion://team/post/123
                let deep_link = if url.starts_with("https://app.cushion.so/") {
                    url.replace("https://app.cushion.so/", "cushion://")
                } else if url.starts_with("http://localhost:3000/") {
                    // Development URLs
                    url.replace("http://localhost:3000/", "cushion://")
                } else {
                    // Already a deep link or relative path
                    if url.starts_with("cushion://") {
                        url.clone()
                    } else {
                        format!("cushion://{}", url.trim_start_matches('/'))
                    }
                };

                println!("🚀 Rust: Emitting deep link: {}", deep_link);

                // Emit deep-link event (same pattern as lib.rs deep link handler)
                let _ = app_clone.emit("deep-link", deep_link);

                // Show and focus the window
                if let Some(window) = app_clone.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                    let _ = window.unminimize();
                    println!("✅ Rust: Window shown and focused");
                }
            } else {
                println!("⚠️  Rust: No URL found for notification ID: {}", notification_id);
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
