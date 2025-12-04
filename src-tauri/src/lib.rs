#![allow(unexpected_cfgs)]

// Module declarations
mod commands;
mod theme;
mod notifications;
mod app_nap;

// Imports
use std::sync::Mutex;
use std::time::Duration;
use tauri::{Emitter, Listener, Manager, RunEvent, WebviewUrl, WebviewWindowBuilder, WindowEvent};

/// State to track pending updates that should be shown when window gains focus
struct PendingUpdate(Mutex<Option<String>>);

#[cfg(target_os = "macos")]
use tauri::menu::{Menu, MenuItem, PredefinedMenuItem, Submenu};

#[cfg(target_os = "macos")]
use tauri::TitleBarStyle;

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
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(
            tauri_plugin_window_state::Builder::default()
                .skip_initial_state("main")
                .build()
        )
        .manage(PendingUpdate(Mutex::new(None)))
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
            commands::window::save_zoom_preference,
            commands::window::get_zoom_preference,
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
    // Setup application menu (macOS only)
    #[cfg(target_os = "macos")]
    {
        let check_updates = MenuItem::with_id(app, "check-for-updates", "Check for Updates...", true, None::<&str>)?;

        let app_submenu = Submenu::with_items(
            app,
            "Cushion",
            true,
            &[
                &PredefinedMenuItem::about(app, Some("About Cushion"), None)?,
                &PredefinedMenuItem::separator(app)?,
                &check_updates,
                &PredefinedMenuItem::separator(app)?,
                &PredefinedMenuItem::hide(app, Some("Hide Cushion"))?,
                &PredefinedMenuItem::hide_others(app, Some("Hide Others"))?,
                &PredefinedMenuItem::show_all(app, Some("Show All"))?,
                &PredefinedMenuItem::separator(app)?,
                &PredefinedMenuItem::quit(app, Some("Quit Cushion"))?,
            ],
        )?;

        let edit_submenu = Submenu::with_items(
            app,
            "Edit",
            true,
            &[
                &PredefinedMenuItem::undo(app, None)?,
                &PredefinedMenuItem::redo(app, None)?,
                &PredefinedMenuItem::separator(app)?,
                &PredefinedMenuItem::cut(app, None)?,
                &PredefinedMenuItem::copy(app, None)?,
                &PredefinedMenuItem::paste(app, None)?,
                &PredefinedMenuItem::separator(app)?,
                &PredefinedMenuItem::select_all(app, None)?,
            ],
        )?;

        let view_submenu = Submenu::with_items(
            app,
            "View",
            true,
            &[
                &PredefinedMenuItem::fullscreen(app, None)?,
            ],
        )?;

        let window_submenu = Submenu::with_items(
            app,
            "Window",
            true,
            &[
                &PredefinedMenuItem::minimize(app, None)?,
                &PredefinedMenuItem::maximize(app, None)?,
                &PredefinedMenuItem::separator(app)?,
                &PredefinedMenuItem::close_window(app, None)?,
            ],
        )?;

        let menu = Menu::with_items(app, &[&app_submenu, &edit_submenu, &view_submenu, &window_submenu])?;
        app.set_menu(menu)?;
    }

    // Setup menu event handler
    #[cfg(target_os = "macos")]
    {
        let handle = app.handle().clone();
        app.on_menu_event(move |_app, event| {
            if event.id().as_ref() == "check-for-updates" {
                trigger_update_check(&handle);
            }
        });
    }

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
    setup_auto_update_check(app.handle());

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

            // Disable spellcheck and text prediction on all inputs
            const disableInputPrediction = (el) => {
                el.setAttribute('spellcheck', 'false');
                el.setAttribute('autocomplete', 'off');
                el.setAttribute('autocorrect', 'off');
                el.setAttribute('autocapitalize', 'off');
                el.setAttribute('writingsuggestions', 'false');
            };

            // Apply to existing inputs
            document.querySelectorAll('input, textarea, [contenteditable="true"]').forEach(disableInputPrediction);

            // Apply to dynamically added inputs via MutationObserver
            const observer = new MutationObserver((mutations) => {
                mutations.forEach((mutation) => {
                    mutation.addedNodes.forEach((node) => {
                        if (node.nodeType === 1) {
                            if (node.matches && node.matches('input, textarea, [contenteditable="true"]')) {
                                disableInputPrediction(node);
                            }
                            node.querySelectorAll && node.querySelectorAll('input, textarea, [contenteditable="true"]').forEach(disableInputPrediction);
                        }
                    });
                });
            });
            observer.observe(document.body, { childList: true, subtree: true });
        });

        // Enable zoom controls with webview zoom
        let zoomLevel = 1.0;

        // Load saved zoom preference on startup
        (async function() {
            try {
                const savedZoom = await window.__TAURI__.core.invoke('get_zoom_preference');
                if (savedZoom && savedZoom >= 0.5 && savedZoom <= 3.0) {
                    zoomLevel = savedZoom;
                    await window.__TAURI__.core.invoke('set_zoom_level', { zoom: zoomLevel });
                }
            } catch (e) {
                console.log('Could not load zoom preference:', e);
            }
        })();

        document.addEventListener('keydown', async function(e) {
            if ((e.metaKey || e.ctrlKey) && e.key === '=') {
                e.preventDefault();
                zoomLevel = Math.min(zoomLevel + 0.1, 3.0);
                await window.__TAURI__.core.invoke('set_zoom_level', { zoom: zoomLevel });
                await window.__TAURI__.core.invoke('save_zoom_preference', { zoom: zoomLevel });
            } else if ((e.metaKey || e.ctrlKey) && e.key === '-') {
                e.preventDefault();
                zoomLevel = Math.max(zoomLevel - 0.1, 0.5);
                await window.__TAURI__.core.invoke('set_zoom_level', { zoom: zoomLevel });
                await window.__TAURI__.core.invoke('save_zoom_preference', { zoom: zoomLevel });
            } else if ((e.metaKey || e.ctrlKey) && e.key === '0') {
                e.preventDefault();
                zoomLevel = 1.0;
                await window.__TAURI__.core.invoke('set_zoom_level', { zoom: zoomLevel });
                await window.__TAURI__.core.invoke('save_zoom_preference', { zoom: zoomLevel });
            }
        });

        // Handle link clicks
        document.addEventListener('click', async function(e) {
            let target = e.target;
            while (target && target.tagName !== 'A') {
                target = target.parentElement;
            }

            if (target && target.tagName === 'A' && target.href) {
                const url = new URL(target.href);
                const isInternal = url.origin === window.location.origin
                    || url.hostname === 'cushion.so'
                    || url.hostname.endsWith('.cushion.so')
                    || url.hostname === 'localhost';

                if (isInternal) {
                    // Remove _blank target so internal links navigate in same window
                    if (target.target === '_blank') {
                        target.removeAttribute('target');
                    }
                } else {
                    // External link - open in default browser
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

/// Check for updates silently without showing any UI
/// Returns the version string if an update is available
async fn check_for_update_silent(app: &tauri::AppHandle) -> Option<String> {
    use tauri_plugin_updater::UpdaterExt;

    println!("🔄 Checking for updates...");

    match app.updater_builder().build() {
        Ok(updater) => {
            match updater.check().await {
                Ok(Some(update)) => {
                    println!("✅ Update available: {}", update.version);
                    Some(update.version.clone())
                }
                Ok(None) => {
                    println!("✅ App is up to date");
                    None
                }
                Err(e) => {
                    println!("❌ Error checking for updates: {}", e);
                    None
                }
            }
        }
        Err(e) => {
            println!("❌ Failed to build updater: {}", e);
            None
        }
    }
}

/// Show update dialog and handle installation
fn show_update_dialog(handle: &tauri::AppHandle) {
    use tauri_plugin_updater::UpdaterExt;
    use tauri_plugin_dialog::{DialogExt, MessageDialogButtons};

    let app = handle.clone();

    tauri::async_runtime::spawn(async move {
        // Re-check to get the actual update object for installation
        match app.updater_builder().build() {
            Ok(updater) => {
                match updater.check().await {
                    Ok(Some(update)) => {
                        let app_for_dialog = app.clone();
                        let confirmed = tauri::async_runtime::spawn_blocking(move || {
                            app_for_dialog.dialog()
                                .message("There's a new version available. Would you like to update?")
                                .title("Software Update")
                                .buttons(MessageDialogButtons::OkCancelCustom("Install Update".into(), "Not Now".into()))
                                .blocking_show()
                        }).await.unwrap_or(false);

                        if confirmed {
                            println!("✅ User confirmed update installation");
                            println!("⬇️  Installing update: {}", update.version);

                            match update.download_and_install(|chunk_length, content_length| {
                                if let Some(total) = content_length {
                                    let percentage = (chunk_length as f64 / total as f64) * 100.0;
                                    println!("📊 Download progress: {:.1}%", percentage);
                                }
                            }, || {
                                println!("✅ Update downloaded, installing...");
                            }).await {
                                Ok(_) => {
                                    println!("🎉 Update installed! Restarting...");
                                    app.restart();
                                }
                                Err(e) => {
                                    println!("❌ Failed to install update: {}", e);
                                    let app_for_error = app.clone();
                                    let _ = tauri::async_runtime::spawn_blocking(move || {
                                        app_for_error.dialog()
                                            .message("Failed to install update. Please try again later.")
                                            .title("Update Error")
                                            .blocking_show()
                                    }).await;
                                }
                            }
                        } else {
                            println!("ℹ️  User declined update installation");
                        }
                    }
                    Ok(None) => {
                        println!("ℹ️  Update no longer available");
                    }
                    Err(e) => {
                        println!("❌ Error checking for updates: {}", e);
                    }
                }
            }
            Err(e) => {
                println!("❌ Failed to build updater: {}", e);
            }
        }
    });
}

/// Setup automatic update checking - runs periodically every 4 hours
/// Updates are checked silently; dialog only shows when window gains focus
fn setup_auto_update_check(handle: &tauri::AppHandle) {
    let app = handle.clone();

    tauri::async_runtime::spawn(async move {
        loop {
            // Check for updates silently
            if let Some(version) = check_for_update_silent(&app).await {
                // Store pending update version to show dialog on next focus
                if let Some(state) = app.try_state::<PendingUpdate>() {
                    *state.0.lock().unwrap() = Some(version);
                }
            }

            // Wait 4 hours before next check
            tokio::time::sleep(Duration::from_secs(4 * 60 * 60)).await;
        }
    });
}

/// Trigger update check from menu (shows dialog for both available and up-to-date)
#[cfg(target_os = "macos")]
fn trigger_update_check(handle: &tauri::AppHandle) {
    use tauri_plugin_updater::UpdaterExt;
    use tauri_plugin_dialog::{DialogExt, MessageDialogButtons};

    let app = handle.clone();

    tauri::async_runtime::spawn(async move {
        println!("🔄 Checking for updates (manual trigger)...");

        match app.updater_builder().build() {
            Ok(updater) => {
                match updater.check().await {
                    Ok(Some(update)) => {
                        println!("✅ Update available: {}", update.version);

                        let app_for_dialog = app.clone();
                        let confirmed = tauri::async_runtime::spawn_blocking(move || {
                            app_for_dialog.dialog()
                                .message("There's a new version available. Would you like to update?")
                                .title("Software Update")
                                .buttons(MessageDialogButtons::OkCancelCustom("Install Update".into(), "Not Now".into()))
                                .blocking_show()
                        }).await.unwrap_or(false);

                        if confirmed {
                            println!("✅ User confirmed update installation");
                            println!("⬇️  Installing update: {}", update.version);

                            match update.download_and_install(|chunk_length, content_length| {
                                if let Some(total) = content_length {
                                    let percentage = (chunk_length as f64 / total as f64) * 100.0;
                                    println!("📊 Download progress: {:.1}%", percentage);
                                }
                            }, || {
                                println!("✅ Update downloaded, installing...");
                            }).await {
                                Ok(_) => {
                                    println!("🎉 Update installed! Restarting...");
                                    app.restart();
                                }
                                Err(e) => {
                                    println!("❌ Failed to install update: {}", e);
                                    let app_for_error = app.clone();
                                    let _ = tauri::async_runtime::spawn_blocking(move || {
                                        app_for_error.dialog()
                                            .message("Failed to install update. Please try again later.")
                                            .title("Update Error")
                                            .blocking_show()
                                    }).await;
                                }
                            }
                        } else {
                            println!("ℹ️  User declined update installation");
                        }
                    }
                    Ok(None) => {
                        println!("✅ App is up to date");
                        // Show "up to date" dialog for manual checks
                        let _ = tauri::async_runtime::spawn_blocking(move || {
                            app.dialog()
                                .message("You're up to date!")
                                .title("Software Update")
                                .blocking_show()
                        }).await;
                    }
                    Err(e) => {
                        println!("❌ Error checking for updates: {}", e);
                        let _ = tauri::async_runtime::spawn_blocking(move || {
                            app.dialog()
                                .message("Could not check for updates. Please check your internet connection and try again.")
                                .title("Update Error")
                                .blocking_show()
                        }).await;
                    }
                }
            }
            Err(e) => {
                println!("❌ Failed to build updater: {}", e);
                let _ = tauri::async_runtime::spawn_blocking(move || {
                    app.dialog()
                        .message("Could not check for updates. Please try again later.")
                        .title("Update Error")
                        .blocking_show()
                }).await;
            }
        }
    });
}

/// Handle window events
fn handle_window_event(window: &tauri::Window, event: &WindowEvent) {
    match event {
        WindowEvent::Focused(true) => {
            // Check if there's a pending update to show when window gains focus
            if let Some(state) = window.app_handle().try_state::<PendingUpdate>() {
                let pending = state.0.lock().unwrap().take();
                if pending.is_some() {
                    println!("🔔 Showing pending update dialog on window focus");
                    show_update_dialog(window.app_handle());
                }
            }
        }
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
