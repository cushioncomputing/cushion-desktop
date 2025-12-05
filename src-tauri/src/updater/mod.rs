//! Update management module
//!
//! Handles update checking, dialogs, and scheduling for the Cushion desktop app.

mod dialog;
mod scheduler;

use std::sync::Mutex;
use tauri_plugin_updater::UpdaterExt;

pub use dialog::show_update_dialog;
pub use scheduler::setup_auto_update_check;

/// State to track pending updates that should be shown when window gains focus
pub struct PendingUpdate(pub Mutex<Option<String>>);

impl PendingUpdate {
    pub fn new() -> Self {
        Self(Mutex::new(None))
    }

    /// Set a pending update version
    pub fn set(&self, version: String) {
        *self.0.lock().unwrap() = Some(version);
    }

    /// Take the pending update (clears it)
    pub fn take(&self) -> Option<String> {
        self.0.lock().unwrap().take()
    }
}

impl Default for PendingUpdate {
    fn default() -> Self {
        Self::new()
    }
}

/// Check for updates silently without showing any UI
/// Returns the version string if an update is available
pub async fn check_for_update_silent(app: &tauri::AppHandle) -> Option<String> {
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
