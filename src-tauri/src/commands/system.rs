/// System-level commands

/// Allowed URL schemes for opening external URLs
const ALLOWED_URL_SCHEMES: &[&str] = &["http", "https", "mailto"];

/// Validates that a URL uses an allowed scheme
fn validate_url_scheme(url: &str) -> Result<(), String> {
    // Parse the URL to extract the scheme
    let scheme = url.split(':').next().unwrap_or("").to_lowercase();

    if ALLOWED_URL_SCHEMES.contains(&scheme.as_str()) {
        Ok(())
    } else {
        Err(format!(
            "URL scheme '{}' is not allowed. Allowed schemes: {:?}",
            scheme, ALLOWED_URL_SCHEMES
        ))
    }
}

#[tauri::command]
pub fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
pub fn get_user_agent(app: tauri::AppHandle) -> String {
    // Get app identifier to determine if dev or prod
    let config = app.config();
    let identifier = &config.identifier;

    let app_type = if identifier.ends_with(".dev") {
        "CushionDesktop/dev"
    } else {
        "CushionDesktop/prod"
    };

    format!("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36 {}", app_type)
}

#[tauri::command]
pub async fn open_url(app: tauri::AppHandle, url: String) -> Result<(), String> {
    // Validate URL scheme before opening
    validate_url_scheme(&url)?;

    use tauri_plugin_opener::OpenerExt;
    app.opener().open_url(&url, None::<&str>).map_err(|e| e.to_string())?;
    Ok(())
}
