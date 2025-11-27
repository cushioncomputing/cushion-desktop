/// Notification commands using custom cross-platform notification system
use crate::notifications::NotificationManager;

#[tauri::command]
pub async fn show_notification(
    title: String,
    body: String,
    url: Option<String>
) -> Result<(), String> {
    println!("📱 Show notification command: '{}' - '{}'", title, body);

    let manager = NotificationManager::get()
        .ok_or_else(|| "Notification manager not initialized".to_string())?;

    // Generate a unique notification ID
    let id = format!("cushion-{}", uuid::Uuid::new_v4());

    manager.show_notification(id, title, body, url)
}

#[tauri::command]
pub async fn check_notification_permission() -> Result<String, String> {
    println!("🔍 Requesting notification permission...");

    #[cfg(target_os = "macos")]
    {
        crate::notifications::macos::request_notification_permission()
    }

    #[cfg(target_os = "windows")]
    {
        // Windows typically doesn't require permission prompts for desktop apps
        Ok("granted".to_string())
    }

    #[cfg(target_os = "linux")]
    {
        // Linux typically doesn't require permission prompts
        Ok("granted".to_string())
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
    {
        Ok("denied".to_string())
    }
}
