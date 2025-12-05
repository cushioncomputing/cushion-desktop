//! Window and application event handlers
//!
//! Handles window lifecycle events and application run events.

use tauri::{Manager, RunEvent, WindowEvent};

#[cfg(target_os = "macos")]
use cocoa::base::id;

use crate::updater;

/// Handle window events
pub fn handle_window_event(window: &tauri::Window, event: &WindowEvent) {
    match event {
        WindowEvent::Focused(true) => {
            // Check if there's a pending update to show when window gains focus
            if let Some(state) = window.app_handle().try_state::<updater::PendingUpdate>() {
                if state.take().is_some() {
                    println!("🔔 Showing pending update dialog on window focus");
                    updater::show_update_dialog(window.app_handle(), false);
                }
            }
        }
        WindowEvent::ThemeChanged(theme) => {
            println!("Theme changed to: {:?}", theme);

            #[cfg(target_os = "macos")]
            {
                let ns_window = window.ns_window().unwrap() as id;
                crate::theme::update_window_background_for_theme(ns_window);
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
pub fn handle_run_event(app_handle: &tauri::AppHandle, event: RunEvent) {
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
                super::recreate_window(app_handle);
            }
        }
        _ => {}
    }
}
