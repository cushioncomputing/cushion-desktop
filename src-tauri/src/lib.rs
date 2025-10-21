#![allow(unexpected_cfgs)]

// Module declarations
mod commands;
mod theme;

// Imports
use tauri::{Emitter, Listener, Manager, RunEvent, WebviewUrl, WebviewWindowBuilder, WindowEvent};

#[cfg(target_os = "macos")]
use tauri::TitleBarStyle;

#[cfg(target_os = "macos")]
use cocoa::base::id;

/// Main Tauri application entry point
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_deep_link::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_dialog::init())
        .plugin(
            tauri_plugin_window_state::Builder::default()
                .skip_initial_state("main")
                .build()
        )
        .setup(setup_app)
        .on_window_event(handle_window_event)
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
        .run(handle_run_event);
}

/// Setup function for the Tauri application
fn setup_app(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    // Create the main window programmatically
    let win_builder = create_window_builder(app);
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

    // Setup notification click handling
    commands::notification::setup_notification_click_handler(app.handle());

    // Handle dock icon clicks (macOS)
    #[cfg(target_os = "macos")]
    {
        app.set_activation_policy(tauri::ActivationPolicy::Regular);
    }

    Ok(())
}

/// Create the main window builder with all configuration
fn create_window_builder<R: tauri::Runtime, M: tauri::Manager<R>>(app: &M) -> WebviewWindowBuilder<'_, R, M> {
    let mut win_builder = WebviewWindowBuilder::new(app, "main", WebviewUrl::App(Default::default()))
        .title("Cushion")
        .inner_size(1200.0, 800.0)
        .min_inner_size(800.0, 600.0)
        .resizable(true)
        .center()
        .hidden_title(true)
        .transparent(true)
        // Disable Tauri's built-in drag/drop handler to pass events through to the WebView.
        // This allows the web-app's react-dropzone implementation to handle file uploads directly.
        .disable_drag_drop_handler();

    #[cfg(target_os = "macos")]
    {
        win_builder = win_builder
            .title_bar_style(TitleBarStyle::Overlay)
            .traffic_light_position(tauri::LogicalPosition::new(15.0, 20.0))
            .initialization_script(get_initialization_script());
    }

    win_builder
}

/// Get the JavaScript initialization script for the webview
#[cfg(target_os = "macos")]
fn get_initialization_script() -> &'static str {
    r#"
        // Disable swipe navigation and overscroll bounce
        document.addEventListener('DOMContentLoaded', function() {
            document.body.style.overscrollBehaviorX = 'none';
            document.body.style.overscrollBehaviorY = 'none';
            document.body.style.overflowX = 'hidden';
            document.documentElement.style.overscrollBehaviorX = 'none';
            document.documentElement.style.overscrollBehaviorY = 'none';
            document.documentElement.style.overflowX = 'hidden';
        });

        // Enable zoom controls with webview zoom
        let zoomLevel = 1.0;
        document.addEventListener('keydown', async function(e) {
            if ((e.metaKey || e.ctrlKey) && e.key === '=') {
                e.preventDefault();
                zoomLevel = Math.min(zoomLevel + 0.1, 3.0);
                await window.__TAURI__.core.invoke('set_zoom_level', { zoom: zoomLevel });
            } else if ((e.metaKey || e.ctrlKey) && e.key === '-') {
                e.preventDefault();
                zoomLevel = Math.max(zoomLevel - 0.1, 0.5);
                await window.__TAURI__.core.invoke('set_zoom_level', { zoom: zoomLevel });
            } else if ((e.metaKey || e.ctrlKey) && e.key === '0') {
                e.preventDefault();
                zoomLevel = 1.0;
                await window.__TAURI__.core.invoke('set_zoom_level', { zoom: zoomLevel });
            }
        });

        // Open external links in default browser
        document.addEventListener('click', async function(e) {
            let target = e.target;
            while (target && target.tagName !== 'A') {
                target = target.parentElement;
            }

            if (target && target.tagName === 'A' && target.href) {
                const url = new URL(target.href);
                // Only open external links (not same origin)
                if (url.origin !== window.location.origin) {
                    e.preventDefault();
                    await window.__TAURI__.core.invoke('open_url', { url: target.href });
                }
            }
        }, true);
    "#
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
                println!("Received deep link: {}", url);
                let _ = handle_clone.emit("deep-link", url);

                // Show the window when a deep link is received
                if let Some(window) = handle_clone.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                    let _ = window.unminimize();
                }
            }
        }
    });
}

/// Handle window events
fn handle_window_event(window: &tauri::Window, event: &WindowEvent) {
    match event {
        WindowEvent::ThemeChanged(theme) => {
            println!("Theme changed to: {:?}", theme);

            #[cfg(target_os = "macos")]
            {
                let ns_window = window.ns_window().unwrap() as id;
                theme::update_window_background_for_theme(ns_window);
            }
        }
        WindowEvent::CloseRequested { api, .. } => {
            println!("Close requested - hiding window instead of closing");

            // Prevent the window from closing and hide it instead
            api.prevent_close();
            let _ = window.hide();
        }
        _ => {}
    }
}

/// Handle application run events
fn handle_run_event(app_handle: &tauri::AppHandle, event: RunEvent) {
    match event {
        RunEvent::ExitRequested { api, code, .. } => {
            // Only prevent exit on normal close (Cmd+Q or window close)
            // Let the app exit normally on force quit
            if code.is_none() {
                // Prevent the app from exiting when the last window closes
                println!("Exit requested - preventing exit to keep app alive");
                api.prevent_exit();
            }
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
                recreate_window(app_handle);
            }
        }
        _ => {}
    }
}

/// Recreate the main window (fallback for when window is unexpectedly destroyed)
#[cfg(target_os = "macos")]
fn recreate_window(app_handle: &tauri::AppHandle) {
    let win_builder = WebviewWindowBuilder::new(
        app_handle,
        "main",
        WebviewUrl::App(Default::default())
    )
    .title("Cushion")
    .inner_size(1200.0, 800.0)
    .min_inner_size(800.0, 600.0)
    .resizable(true)
    .center()
    .hidden_title(true)
    .transparent(true)
    .visible(true) // Ensure window is visible to attach to display
    .title_bar_style(TitleBarStyle::Overlay)
    .traffic_light_position(tauri::LogicalPosition::new(15.0, 20.0));

    let _ = win_builder.build();
}
