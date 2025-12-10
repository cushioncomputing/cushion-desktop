/// Minimal cross-platform notification system with click handling
///
/// This module provides native notification support with click handlers
/// for macOS, Windows, and Linux without heavy dependencies.

use tauri::{AppHandle, Emitter, Manager};
use std::sync::{Arc, Mutex, OnceLock};
use std::collections::HashMap;

#[cfg(target_os = "macos")]
pub mod macos;

#[cfg(target_os = "windows")]
mod windows;

#[cfg(target_os = "linux")]
mod linux;

/// Callback function type for notification clicks
type NotificationCallback = Box<dyn Fn(NotificationClick) + Send + Sync + 'static>;

/// Data about a notification click
#[derive(Debug, Clone)]
pub struct NotificationClick {
    /// The notification identifier
    pub id: String,
    /// Optional deep link URL associated with the notification
    pub url: Option<String>,
    /// The type of click action
    pub action: ClickAction,
}

#[derive(Debug, Clone)]
pub enum ClickAction {
    /// User clicked the notification body
    Body,
    /// User dismissed the notification
    Dismiss,
    /// User clicked a custom action button
    Button(String),
}

/// Global notification manager instance
static NOTIFICATION_MANAGER: OnceLock<Arc<NotificationManager>> = OnceLock::new();

/// Cross-platform notification manager
pub struct NotificationManager {
    app: AppHandle,
    callback: Mutex<Option<NotificationCallback>>,
    /// Store notification metadata (id -> url mapping)
    metadata: Mutex<HashMap<String, String>>,
}

impl NotificationManager {
    /// Initialize the notification manager
    pub fn init(app: AppHandle) -> Arc<Self> {
        let manager = Arc::new(NotificationManager {
            app,
            callback: Mutex::new(None),
            metadata: Mutex::new(HashMap::new()),
        });

        // Set up platform-specific handlers
        #[cfg(target_os = "macos")]
        macos::setup(manager.clone());

        #[cfg(target_os = "windows")]
        windows::setup(manager.clone());

        #[cfg(target_os = "linux")]
        linux::setup(manager.clone());

        NOTIFICATION_MANAGER.set(manager.clone()).ok();
        manager
    }

    /// Get the global notification manager instance
    pub fn get() -> Option<Arc<NotificationManager>> {
        NOTIFICATION_MANAGER.get().cloned()
    }

    /// Set the callback for notification clicks
    pub fn set_callback<F>(&self, callback: F)
    where
        F: Fn(NotificationClick) + Send + Sync + 'static,
    {
        let mut cb = self.callback.lock().unwrap();
        *cb = Some(Box::new(callback));
    }

    /// Called by platform implementations when a notification is clicked
    pub fn handle_click(&self, click: NotificationClick) {
        println!("🔔 Notification clicked: {:?}", click);

        if let Some(callback) = self.callback.lock().unwrap().as_ref() {
            callback(click);
        }
    }

    /// Store metadata for a notification
    pub fn store_metadata(&self, id: String, url: String) {
        let mut metadata = self.metadata.lock().unwrap();
        metadata.insert(id, url);
    }

    /// Get metadata for a notification
    pub fn get_metadata(&self, id: &str) -> Option<String> {
        let metadata = self.metadata.lock().unwrap();
        metadata.get(id).cloned()
    }

    /// Show a notification
    pub fn show_notification(
        &self,
        id: String,
        title: String,
        body: String,
        url: Option<String>,
    ) -> Result<(), String> {
        println!("📱 Showing notification: '{}' - '{}'", title, body);

        // Store metadata if URL provided
        if let Some(ref notification_url) = url {
            self.store_metadata(id.clone(), notification_url.clone());
        }

        // Platform-specific notification display
        #[cfg(target_os = "macos")]
        return macos::show_notification(id, title, body, url);

        #[cfg(target_os = "windows")]
        return windows::show_notification(self.app.clone(), id, title, body, url);

        #[cfg(target_os = "linux")]
        return linux::show_notification(id, title, body, url);

        #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
        {
            Err("Notifications not supported on this platform".to_string())
        }
    }

    /// Get app handle
    pub fn app(&self) -> &AppHandle {
        &self.app
    }
}

/// Validate that a URL is safe to emit as a deep link
fn is_valid_notification_url(url: &str) -> bool {
    if let Ok(parsed) = url::Url::parse(url) {
        // Allow cushion:// and cushion-dev:// deep links
        if parsed.scheme() == "cushion" || parsed.scheme() == "cushion-dev" {
            return true;
        }
        // Allow HTTPS URLs to cushion.so domains
        if parsed.scheme() == "https" {
            if let Some(host) = parsed.host_str() {
                return host == "app.cushion.so" || host.ends_with(".cushion.so");
            }
        }
    }
    false
}

/// Setup notification system with default click handler
pub fn setup(app: &AppHandle) -> Arc<NotificationManager> {
    let manager = NotificationManager::init(app.clone());
    let app_clone = app.clone();

    // Set up default click handler
    manager.set_callback(move |click| {
        println!("🔔 Default handler: {:?}", click);

        match click.action {
            ClickAction::Body | ClickAction::Button(_) => {
                // Emit deep link event if URL exists and is valid
                if let Some(ref url) = click.url {
                    if is_valid_notification_url(url) {
                        println!("🔗 Emitting deep link: {}", url);
                        let _ = app_clone.emit("deep-link", url.clone());
                    } else {
                        eprintln!("🚫 Rejected invalid notification URL: {}", url);
                    }
                }

                // Focus the window
                if let Some(window) = app_clone.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                    let _ = window.unminimize();
                }
            }
            ClickAction::Dismiss => {
                println!("Notification dismissed");
            }
        }
    });

    manager
}
