/// Notification commands
use tauri_plugin_notification::NotificationExt;

#[tauri::command]
pub async fn show_notification(app: tauri::AppHandle, title: String, body: String) -> Result<(), String> {
    println!("📱 Rust: Attempting to show notification: '{}' - '{}'", title, body);

    // Send the notification
    let notification_builder = app.notification().builder();

    match notification_builder
        .title(title)
        .body(body)
        .show() {
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
