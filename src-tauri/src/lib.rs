use tauri::{Emitter, Listener, Manager};
use tauri_plugin_notification::NotificationExt;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
async fn show_notification(app: tauri::AppHandle, title: String, body: String) -> Result<(), String> {
    app.notification()
        .builder()
        .title(title)
        .body(body)
        .show()
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn get_user_agent() -> String {
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36 CushionDesktop/0.1.0".to_string()
}

#[tauri::command]
fn show_main_window(app: tauri::AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("main") {
        window.show().map_err(|e| e.to_string())?;
        window.set_focus().map_err(|e| e.to_string())?;
        window.unminimize().map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_deep_link::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .setup(|app| {
            // Handle deep link events
            let handle = app.handle().clone();

            // Set up deep link handler
            app.listen("deep-link://new-url", move |event| {
                let payload = event.payload();
                println!("Received deep link: {}", payload);
                let _ = handle.emit("deep-link", payload);
            });

            // Handle window close events to hide instead of quit
            let main_window = app.get_webview_window("main").unwrap();
            let window_clone = main_window.clone();
            main_window.on_window_event(move |event| {
                if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                    // Prevent the default close behavior
                    api.prevent_close();
                    // Hide the window instead
                    let _ = window_clone.hide();
                }
            });

            // Create a simple way to restore the window via menu
            #[cfg(target_os = "macos")]
            {
                let window_for_menu = main_window.clone();
                let app_handle = app.handle().clone();

                // Add a Tauri command to show the window that can be called from frontend
                app.manage(window_for_menu);
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![greet, show_notification, get_user_agent, show_main_window])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
