/// System-level commands

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
    use tauri_plugin_opener::OpenerExt;

    // Parse and validate URL
    let parsed = url::Url::parse(&url).map_err(|e| format!("Invalid URL: {}", e))?;

    // Only allow safe protocols
    match parsed.scheme() {
        "http" | "https" | "mailto" => {}
        scheme => return Err(format!("Protocol '{}' not allowed", scheme)),
    }

    app.opener()
        .open_url(&url, None::<&str>)
        .map_err(|e| e.to_string())?;
    Ok(())
}
