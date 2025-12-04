/// Window management commands
use tauri::Manager;
use tauri_plugin_store::StoreExt;

const PREFERENCES_STORE: &str = "preferences.json";
const ZOOM_LEVEL_KEY: &str = "zoom_level";

#[tauri::command]
pub fn show_main_window(app: tauri::AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("main") {
        window.show().map_err(|e| e.to_string())?;
        window.set_focus().map_err(|e| e.to_string())?;
        window.unminimize().map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
pub fn is_window_visible(app: tauri::AppHandle) -> Result<bool, String> {
    if let Some(window) = app.get_webview_window("main") {
        window.is_visible().map_err(|e| e.to_string())
    } else {
        Ok(false)
    }
}

#[tauri::command]
pub fn is_window_focused(app: tauri::AppHandle) -> Result<bool, String> {
    if let Some(window) = app.get_webview_window("main") {
        window.is_focused().map_err(|e| e.to_string())
    } else {
        Ok(false)
    }
}

#[tauri::command]
pub fn is_window_minimized(app: tauri::AppHandle) -> Result<bool, String> {
    if let Some(window) = app.get_webview_window("main") {
        window.is_minimized().map_err(|e| e.to_string())
    } else {
        Ok(false)
    }
}

#[tauri::command]
pub fn set_zoom_level(app: tauri::AppHandle, zoom: f64) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("main") {
        window.set_zoom(zoom).map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
pub fn save_zoom_preference(app: tauri::AppHandle, zoom: f64) -> Result<(), String> {
    let store = app.store(PREFERENCES_STORE).map_err(|e| e.to_string())?;
    store.set(ZOOM_LEVEL_KEY, serde_json::json!(zoom));
    store.save().map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn get_zoom_preference(app: tauri::AppHandle) -> Result<f64, String> {
    let store = app.store(PREFERENCES_STORE).map_err(|e| e.to_string())?;
    match store.get(ZOOM_LEVEL_KEY) {
        Some(value) => value.as_f64().ok_or_else(|| "Invalid zoom value".to_string()),
        None => Ok(1.0), // Default zoom level
    }
}
