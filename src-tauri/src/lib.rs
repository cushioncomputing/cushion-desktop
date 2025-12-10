#![allow(unexpected_cfgs)]

// Module declarations
mod commands;
mod theme;
mod notifications;
mod app_nap;
mod updater;
mod window;
mod menu;

// Imports
use tauri::{Emitter, Listener, Manager};

#[cfg(target_os = "macos")]
use cocoa::base::id;

/// Wrapper struct to store the macOS activity assertion
/// This keeps the App Nap prevention active for the app lifetime
#[cfg(target_os = "macos")]
struct AppNapActivity(id);

// Safety: The activity object is created once and never modified.
// NSProcessInfo is thread-safe, and we only keep the object alive without accessing it.
#[cfg(target_os = "macos")]
unsafe impl Send for AppNapActivity {}
#[cfg(target_os = "macos")]
unsafe impl Sync for AppNapActivity {}

/// Main Tauri application entry point
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_deep_link::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_dialog::init())
        .plugin(
            tauri_plugin_window_state::Builder::default()
                .skip_initial_state("main")
                .build()
        )
        .manage(updater::PendingUpdate::new())
        .setup(setup_app)
        .on_window_event(window::handle_window_event)
        .invoke_handler(tauri::generate_handler![
            commands::system::greet,
            commands::notification::show_notification,
            commands::notification::check_notification_permission,
            commands::system::get_user_agent,
            commands::window::show_main_window,
            commands::window::is_window_visible,
            commands::window::is_window_focused,
            commands::window::is_window_minimized,
            commands::window::set_zoom_level,
            commands::system::open_url,
            commands::updater::check_for_updates,
            commands::updater::install_update,
            commands::updater::get_app_version,
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(window::handle_run_event);
}

/// Setup function for the Tauri application
fn setup_app(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    // Setup application menu (macOS only)
    #[cfg(target_os = "macos")]
    {
        menu::setup_menu(app)?;
        menu::setup_menu_events(app);
    }

    // Create the main window programmatically
    let win_builder = window::create_window_builder(app);
    let window = win_builder.build()?;

    // Ensure window is shown to attach to display before layer manipulation
    let _ = window.show();

    // Restore window state with manual adjustment
    use tauri_plugin_window_state::{WindowExt, StateFlags};
    let _ = window.restore_state(StateFlags::all());

    // Apply window effects on macOS
    #[cfg(target_os = "macos")]
    {
        let ns_window = window.ns_window().unwrap() as id;
        // Set background color based on system theme
        theme::update_window_background_for_theme(ns_window);
    }

    // Setup deep link handling
    setup_deep_links(app.handle());

    // Setup notification system
    notifications::setup(app.handle());

    // Prevent macOS App Nap to keep WebSocket connections alive
    #[cfg(target_os = "macos")]
    {
        if let Some(activity) = app_nap::prevent_app_nap() {
            // Store the activity in app state to keep the assertion active
            // If we don't store it, it will be dropped and the assertion will end
            app.manage(AppNapActivity(activity));
        }
    }

    // Check for updates on startup
    updater::setup_auto_update_check(app.handle());

    // Handle dock icon clicks (macOS)
    #[cfg(target_os = "macos")]
    {
        app.set_activation_policy(tauri::ActivationPolicy::Regular);
    }

    Ok(())
}

/// Setup deep link event handling
fn setup_deep_links(handle: &tauri::AppHandle) {
    let handle_clone = handle.clone();

    // Set up deep link handler - tauri-plugin-deep-link emits "deep-link://new-url" events
    // when a cushion:// URL is opened
    handle.listen("deep-link://new-url", move |event| {
        // In Tauri v2, payload is a String
        let payload = event.payload();
        println!("Received deep link payload: {}", payload);

        // Try to parse as JSON array
        if let Ok(urls) = serde_json::from_str::<Vec<String>>(payload) {
            if let Some(url) = urls.first() {
                // Validate the URL scheme before emitting
                if let Ok(parsed) = url::Url::parse(url) {
                    if parsed.scheme() == "cushion" || parsed.scheme() == "cushion-dev" {
                        println!("Received deep link: {}", url);
                        let _ = handle_clone.emit("deep-link", url);

                        // Show the window when a deep link is received
                        if let Some(window) = handle_clone.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                            let _ = window.unminimize();
                        }
                    } else {
                        eprintln!(
                            "Rejected deep link with invalid scheme: {}",
                            parsed.scheme()
                        );
                    }
                } else {
                    eprintln!("Failed to parse deep link URL: {}", url);
                }
            }
        }
    });
}
