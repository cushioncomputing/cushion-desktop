use tauri::{Emitter, Listener, Manager, RunEvent, WebviewUrl, WebviewWindowBuilder, WindowEvent};

#[cfg(target_os = "macos")]
use tauri::TitleBarStyle;
use tauri_plugin_notification::NotificationExt;
use std::fs;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
struct WindowState {
    width: f64,
    height: f64,
}

impl Default for WindowState {
    fn default() -> Self {
        Self {
            width: 1200.0,
            height: 800.0,
        }
    }
}

fn get_state_file_path(app_handle: &tauri::AppHandle) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let app_dir = app_handle.path().app_data_dir()
        .map_err(|e| format!("Failed to get app data dir: {}", e))?;

    if !app_dir.exists() {
        fs::create_dir_all(&app_dir)
            .map_err(|e| format!("Failed to create app data dir: {}", e))?;
    }

    Ok(app_dir.join("window-state.json"))
}

fn save_window_state(app_handle: &tauri::AppHandle, state: &WindowState) -> Result<(), Box<dyn std::error::Error>> {
    let state_path = get_state_file_path(app_handle)?;
    let state_json = serde_json::to_string_pretty(state)
        .map_err(|e| format!("Failed to serialize window state: {}", e))?;

    fs::write(&state_path, state_json)
        .map_err(|e| format!("Failed to write window state: {}", e))?;

    println!("Saved window state: {}x{} to {:?}", state.width, state.height, state_path);
    Ok(())
}

fn load_window_state(app_handle: &tauri::AppHandle) -> WindowState {
    match get_state_file_path(app_handle) {
        Ok(state_path) => {
            match fs::read_to_string(&state_path) {
                Ok(state_json) => {
                    match serde_json::from_str::<WindowState>(&state_json) {
                        Ok(state) => {
                            println!("Loaded window state: {}x{} from {:?}", state.width, state.height, state_path);
                            state
                        }
                        Err(e) => {
                            println!("Failed to parse window state: {}, using default", e);
                            WindowState::default()
                        }
                    }
                }
                Err(e) => {
                    println!("Failed to read window state file: {}, using default", e);
                    WindowState::default()
                }
            }
        }
        Err(e) => {
            println!("Failed to get state file path: {}, using default", e);
            WindowState::default()
        }
    }
}

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
async fn show_notification(app: tauri::AppHandle, title: String, body: String) -> Result<(), String> {
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
async fn check_notification_permission(app: tauri::AppHandle) -> Result<String, String> {
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

#[tauri::command]
fn is_window_visible(app: tauri::AppHandle) -> Result<bool, String> {
    if let Some(window) = app.get_webview_window("main") {
        window.is_visible().map_err(|e| e.to_string())
    } else {
        Ok(false)
    }
}

#[tauri::command]
fn is_window_focused(app: tauri::AppHandle) -> Result<bool, String> {
    if let Some(window) = app.get_webview_window("main") {
        window.is_focused().map_err(|e| e.to_string())
    } else {
        Ok(false)
    }
}

#[tauri::command]
fn is_window_minimized(app: tauri::AppHandle) -> Result<bool, String> {
    if let Some(window) = app.get_webview_window("main") {
        window.is_minimized().map_err(|e| e.to_string())
    } else {
        Ok(false)
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_deep_link::init())
        .setup(|app| {
            // Load saved window state
            let saved_state = load_window_state(&app.handle());

            // Create the main window programmatically with saved size
            let win_builder = WebviewWindowBuilder::new(app, "main", WebviewUrl::default())
                .title("Cushion")
                .inner_size(saved_state.width, saved_state.height)
                .min_inner_size(800.0, 600.0)
                .resizable(true)
                .center()
                .hidden_title(true);

            #[cfg(target_os = "macos")]
            let win_builder = win_builder
                .title_bar_style(TitleBarStyle::Overlay)
                .traffic_light_position(tauri::LogicalPosition::new(15.0, 20.0));

            let _window = win_builder.build().unwrap();

            // Handle deep link events
            let handle = app.handle().clone();

            // Set up deep link handler
            app.listen("deep-link://new-url", move |event| {
                let payload = event.payload();
                println!("Received deep link: {}", payload);
                let _ = handle.emit("deep-link", payload);
            });

            // Handle dock icon clicks (macOS)
            #[cfg(target_os = "macos")]
            {
                app.set_activation_policy(tauri::ActivationPolicy::Regular);
            }

            Ok(())
        })
        .on_window_event(|window, event| {
            match event {
                WindowEvent::Resized(size) => {
                    // Save window size when resized
                    let app_handle = window.app_handle();
                    let state = WindowState {
                        width: size.width as f64,
                        height: size.height as f64,
                    };

                    if let Err(e) = save_window_state(&app_handle, &state) {
                        println!("Failed to save window state on resize: {}", e);
                    }
                }
                WindowEvent::CloseRequested { api, .. } => {
                    println!("Close requested - hiding window instead of closing");

                    // Save window size before hiding
                    let app_handle = window.app_handle();
                    if let Ok(size) = window.inner_size() {
                        let state = WindowState {
                            width: size.width as f64,
                            height: size.height as f64,
                        };

                        if let Err(e) = save_window_state(&app_handle, &state) {
                            println!("Failed to save window state on hide: {}", e);
                        }
                    }

                    // Prevent the window from closing and hide it instead
                    api.prevent_close();
                    let _ = window.hide();
                }
                _ => {}
            }
        })
        .invoke_handler(tauri::generate_handler![greet, show_notification, check_notification_permission, get_user_agent, show_main_window, is_window_visible, is_window_focused, is_window_minimized])
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|app_handle, event| {
            match event {
                RunEvent::ExitRequested { api, .. } => {
                    // Prevent the app from exiting when the last window closes
                    println!("Exit requested - preventing exit to keep app alive");
                    api.prevent_exit();
                }
                #[cfg(target_os = "macos")]
                RunEvent::Reopen { .. } => {
                    println!("Reopen event received (dock icon clicked)");
                    // Show the existing hidden window when dock icon is clicked
                    if let Some(window) = app_handle.get_webview_window("main") {
                        // Window should always exist since we hide instead of destroy
                        println!("Showing existing hidden window");
                        let _ = window.show();
                        let _ = window.set_focus();
                        let _ = window.unminimize();
                    } else {
                        // This shouldn't happen with the new approach, but fallback just in case
                        println!("Warning: Window doesn't exist, this shouldn't happen!");
                        let saved_state = load_window_state(app_handle);

                        let win_builder = WebviewWindowBuilder::new(
                            app_handle,
                            "main",
                            WebviewUrl::default()
                        )
                        .title("Cushion")
                        .inner_size(saved_state.width, saved_state.height)
                        .min_inner_size(800.0, 600.0)
                        .resizable(true)
                        .center()
                        .hidden_title(true);

                        #[cfg(target_os = "macos")]
                        let win_builder = win_builder
                            .title_bar_style(TitleBarStyle::Overlay)
                            .traffic_light_position(tauri::LogicalPosition::new(15.0, 20.0));

                        let _ = win_builder.build();
                    }
                }
                _ => {}
            }
        });
}