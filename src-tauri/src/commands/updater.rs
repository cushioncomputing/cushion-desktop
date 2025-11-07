use tauri::AppHandle;
use tauri_plugin_updater::UpdaterExt;

#[tauri::command]
pub async fn check_for_updates(app: AppHandle) -> Result<Option<String>, String> {
    println!("🔄 Checking for updates...");

    match app.updater_builder().build() {
        Ok(updater) => {
            match updater.check().await {
                Ok(Some(update)) => {
                    let version = update.version.clone();
                    println!("✅ Update available: {}", version);
                    Ok(Some(version))
                }
                Ok(None) => {
                    println!("✅ App is up to date");
                    Ok(None)
                }
                Err(e) => {
                    println!("❌ Error checking for updates: {}", e);
                    Err(format!("Update check failed: {}", e))
                }
            }
        }
        Err(e) => {
            println!("❌ Failed to build updater: {}", e);
            Err(format!("Failed to initialize updater: {}", e))
        }
    }
}

#[tauri::command]
pub async fn install_update(app: AppHandle) -> Result<(), String> {
    println!("📥 Installing update...");

    match app.updater_builder().build() {
        Ok(updater) => {
            match updater.check().await {
                Ok(Some(update)) => {
                    println!("⬇️  Downloading update: {}", update.version);

                    // Download and install the update
                    match update.download_and_install(|chunk_length, content_length| {
                        if let Some(total) = content_length {
                            let percentage = (chunk_length as f64 / total as f64) * 100.0;
                            println!("📊 Download progress: {:.1}%", percentage);
                        }
                    }, || {
                        println!("✅ Update downloaded, installing...");
                    }).await {
                        Ok(_) => {
                            println!("🎉 Update installed successfully!");
                            println!("🔄 Restarting application...");
                            Ok(())
                        }
                        Err(e) => {
                            println!("❌ Failed to install update: {}", e);
                            Err(format!("Failed to install update: {}", e))
                        }
                    }
                }
                Ok(None) => {
                    println!("ℹ️  No update available");
                    Err("No update available".to_string())
                }
                Err(e) => {
                    println!("❌ Error checking for updates: {}", e);
                    Err(format!("Update check failed: {}", e))
                }
            }
        }
        Err(e) => {
            println!("❌ Failed to build updater: {}", e);
            Err(format!("Failed to initialize updater: {}", e))
        }
    }
}

#[tauri::command]
pub fn get_app_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}
