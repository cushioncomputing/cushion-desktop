//! Update dialog handling
//!
//! Provides UI for update prompts and installation.

use tauri_plugin_dialog::{DialogExt, MessageDialogButtons};
use tauri_plugin_updater::UpdaterExt;

/// Show update dialog and handle installation
///
/// # Arguments
/// * `handle` - The Tauri app handle
/// * `show_up_to_date` - If true, shows a dialog when already up to date (for manual checks)
pub fn show_update_dialog(handle: &tauri::AppHandle, show_up_to_date: bool) {
    let app = handle.clone();

    tauri::async_runtime::spawn(async move {
        match app.updater_builder().build() {
            Ok(updater) => {
                match updater.check().await {
                    Ok(Some(update)) => {
                        prompt_and_install_update(&app, &update).await;
                    }
                    Ok(None) => {
                        println!("✅ App is up to date");
                        if show_up_to_date {
                            show_up_to_date_dialog(&app).await;
                        }
                    }
                    Err(e) => {
                        println!("❌ Error checking for updates: {}", e);
                        if show_up_to_date {
                            show_error_dialog(&app, "Could not check for updates. Please check your internet connection and try again.").await;
                        }
                    }
                }
            }
            Err(e) => {
                println!("❌ Failed to build updater: {}", e);
                if show_up_to_date {
                    show_error_dialog(&app, "Could not check for updates. Please try again later.").await;
                }
            }
        }
    });
}

/// Prompt user to install update and handle the installation
async fn prompt_and_install_update(app: &tauri::AppHandle, update: &tauri_plugin_updater::Update) {
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
                show_error_dialog(app, "Failed to install update. Please try again later.").await;
            }
        }
    } else {
        println!("ℹ️  User declined update installation");
    }
}

/// Show "up to date" dialog
async fn show_up_to_date_dialog(app: &tauri::AppHandle) {
    let app_clone = app.clone();
    let _ = tauri::async_runtime::spawn_blocking(move || {
        app_clone.dialog()
            .message("You're up to date!")
            .title("Software Update")
            .blocking_show()
    }).await;
}

/// Show error dialog
async fn show_error_dialog(app: &tauri::AppHandle, message: &str) {
    let app_clone = app.clone();
    let msg = message.to_string();
    let _ = tauri::async_runtime::spawn_blocking(move || {
        app_clone.dialog()
            .message(msg)
            .title("Update Error")
            .blocking_show()
    }).await;
}
