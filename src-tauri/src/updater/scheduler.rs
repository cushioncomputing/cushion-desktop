//! Automatic update scheduling
//!
//! Handles periodic background update checks.

use std::time::Duration;
use tauri::Manager;

use super::{check_for_update_silent, PendingUpdate};

/// Setup automatic update checking - runs periodically every 4 hours
/// Updates are checked silently; dialog only shows when window gains focus
pub fn setup_auto_update_check(handle: &tauri::AppHandle) {
    let app = handle.clone();

    tauri::async_runtime::spawn(async move {
        loop {
            // Check for updates silently
            if let Some(version) = check_for_update_silent(&app).await {
                // Store pending update version to show dialog on next focus
                if let Some(state) = app.try_state::<PendingUpdate>() {
                    state.set(version);
                }
            }

            // Wait 4 hours before next check
            tokio::time::sleep(Duration::from_secs(4 * 60 * 60)).await;
        }
    });
}
