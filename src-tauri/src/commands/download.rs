//! File download commands
//!
//! Handles downloading files from URLs and saving to disk.

use base64::{engine::general_purpose, Engine as _};
use futures_util::StreamExt;
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::{AppHandle, Manager};
use tokio::io::AsyncWriteExt;

/// Pending download info stored when context menu is shown
pub struct PendingDownload {
    pub url: Mutex<Option<String>>,
    pub media_type: Mutex<Option<String>>,
}

impl PendingDownload {
    pub fn new() -> Self {
        Self {
            url: Mutex::new(None),
            media_type: Mutex::new(None),
        }
    }
}

impl Default for PendingDownload {
    fn default() -> Self {
        Self::new()
    }
}

/// Download a file from a URL to the Downloads folder
///
/// # Arguments
/// * `url` - The URL to download from (must be http/https)
/// * `suggested_filename` - Optional suggested filename
#[tauri::command]
pub async fn download_file(
    app: AppHandle,
    url: String,
    suggested_filename: Option<String>,
) -> Result<String, String> {
    use tauri_plugin_dialog::{DialogExt, FilePath};

    // Validate URL
    let parsed_url = url::Url::parse(&url).map_err(|e| format!("Invalid URL: {}", e))?;

    // Only allow http/https
    if !matches!(parsed_url.scheme(), "http" | "https") {
        return Err(format!("Unsupported URL scheme: {}", parsed_url.scheme()));
    }

    // Determine suggested filename
    let filename = suggested_filename
        .or_else(|| extract_filename_from_url(&parsed_url))
        .unwrap_or_else(|| format!("download-{}", chrono_timestamp()));

    // Show save file dialog
    let downloads_dir = dirs::download_dir();
    let file_dialog = app.dialog().file();

    let file_dialog = if let Some(ref dir) = downloads_dir {
        file_dialog.set_directory(dir)
    } else {
        file_dialog
    };

    let save_path = file_dialog
        .set_file_name(&filename)
        .blocking_save_file();

    let save_path = match save_path {
        Some(FilePath::Path(path)) => path,
        Some(FilePath::Url(url)) => url
            .to_file_path()
            .map_err(|_| "Invalid file path".to_string())?,
        None => return Err("Save cancelled".to_string()),
    };

    // Download the file (may return different path if extension was added)
    let final_path = download_to_path(&url, &save_path).await?;

    Ok(final_path.to_string_lossy().to_string())
}

/// Show native context menu for media download
///
/// # Arguments
/// * `url` - The media URL
/// * `media_type` - Either "image" or "video"
/// * `x` - X coordinate for menu position
/// * `y` - Y coordinate for menu position
#[tauri::command]
pub async fn show_media_context_menu(
    app: AppHandle,
    url: String,
    media_type: String,
    x: f64,
    y: f64,
) -> Result<(), String> {
    use tauri::menu::{ContextMenu, Menu, MenuItem, PredefinedMenuItem};
    use tauri::Position;

    // Store the pending download info
    let pending = app.state::<PendingDownload>();
    *pending.url.lock().unwrap() = Some(url);
    *pending.media_type.lock().unwrap() = Some(media_type.clone());

    // Create menu item labels based on type
    let save_label = if media_type == "video" {
        "Save Video…"
    } else {
        "Save Image…"
    };

    let copy_label = if media_type == "video" {
        "Copy Video URL"
    } else {
        "Copy Image URL"
    };

    // Create the context menu with both items
    let save_item = MenuItem::with_id(&app, "save_media", save_label, true, None::<&str>)
        .map_err(|e| e.to_string())?;
    let copy_item = MenuItem::with_id(&app, "copy_media_url", copy_label, true, None::<&str>)
        .map_err(|e| e.to_string())?;
    let separator = PredefinedMenuItem::separator(&app)
        .map_err(|e| e.to_string())?;

    let menu = Menu::with_items(&app, &[&save_item, &separator, &copy_item])
        .map_err(|e| e.to_string())?;

    // Get the window and show popup menu at position
    if let Some(webview_window) = app.get_webview_window("main") {
        let position = Position::Logical(tauri::LogicalPosition::new(x, y));
        // Get the underlying Window from WebviewWindow
        let window = webview_window.as_ref().window();
        menu.popup_at(window, position)
            .map_err(|e| e.to_string())?;
    }

    Ok(())
}

/// Download a file from blob data (base64 encoded)
///
/// # Arguments
/// * `data` - Base64 encoded file data
/// * `filename` - Suggested filename
/// * `_mime_type` - MIME type of the file (currently unused, reserved for future use)
#[tauri::command]
pub async fn download_blob(
    data: String,
    filename: String,
    _mime_type: Option<String>,
) -> Result<String, String> {
    // Decode base64 data
    let bytes = general_purpose::STANDARD
        .decode(&data)
        .map_err(|e| format!("Failed to decode base64: {}", e))?;

    // Get save path with unique filename
    let save_path = get_unique_download_path(&filename)?;

    // Write to file
    std::fs::write(&save_path, bytes).map_err(|e| format!("Failed to write file: {}", e))?;

    Ok(save_path.to_string_lossy().to_string())
}

/// Extract filename from URL path
fn extract_filename_from_url(url: &url::Url) -> Option<String> {
    // First, check if this is a Next.js image URL with the actual URL in query params
    if url.path().contains("/_next/image") {
        if let Some(actual_url) = url.query_pairs().find(|(k, _)| k == "url") {
            if let Ok(decoded) = urlencoding::decode(&actual_url.1) {
                if let Ok(inner_url) = url::Url::parse(&decoded) {
                    return extract_filename_from_url(&inner_url);
                }
                // If it's a relative path, extract filename from it
                let path = decoded.to_string();
                if let Some(filename) = path.split('/').last() {
                    if !filename.is_empty() && filename.contains('.') {
                        return Some(filename.to_string());
                    }
                }
            }
        }
    }

    url.path_segments()?
        .last()
        .filter(|s| !s.is_empty() && s.contains('.'))
        .map(|s| {
            // URL decode the filename
            urlencoding::decode(s).unwrap_or_else(|_| s.into()).to_string()
        })
}

/// Generate a simple timestamp string for fallback filenames
fn chrono_timestamp() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs().to_string())
        .unwrap_or_else(|_| "unknown".to_string())
}

/// Get the Downloads folder path with a unique filename
fn get_unique_download_path(filename: &str) -> Result<PathBuf, String> {
    let downloads =
        dirs::download_dir().ok_or_else(|| "Could not find Downloads folder".to_string())?;

    let mut path = downloads.join(filename);

    // If file doesn't exist, return as-is
    if !path.exists() {
        return Ok(path);
    }

    // Generate unique filename: "file (1).ext", "file (2).ext", etc.
    let stem = std::path::Path::new(filename)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("download");
    let extension = std::path::Path::new(filename)
        .extension()
        .and_then(|s| s.to_str());

    let mut counter = 1;
    loop {
        let new_name = match extension {
            Some(ext) => format!("{} ({}).{}", stem, counter, ext),
            None => format!("{} ({})", stem, counter),
        };
        path = downloads.join(new_name);

        if !path.exists() {
            return Ok(path);
        }
        counter += 1;

        // Safety limit
        if counter > 1000 {
            return Err("Too many files with same name".to_string());
        }
    }
}

/// Download a file from URL to the specified path using streaming
/// Returns the actual path used (may differ if extension was added)
async fn download_to_path(url: &str, path: &PathBuf) -> Result<PathBuf, String> {
    let client = reqwest::Client::new();

    let response = client
        .get(url)
        .send()
        .await
        .map_err(|e| format!("Request failed: {}", e))?;

    if !response.status().is_success() {
        return Err(format!(
            "Download failed with status: {}",
            response.status()
        ));
    }

    // Check if we need to add an extension based on content-type
    let final_path = if path.extension().is_none() {
        let content_type = response
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");

        let ext = match content_type {
            ct if ct.contains("image/png") => ".png",
            ct if ct.contains("image/jpeg") || ct.contains("image/jpg") => ".jpg",
            ct if ct.contains("image/gif") => ".gif",
            ct if ct.contains("image/webp") => ".webp",
            ct if ct.contains("image/svg") => ".svg",
            ct if ct.contains("video/mp4") => ".mp4",
            ct if ct.contains("video/webm") => ".webm",
            ct if ct.contains("video/quicktime") => ".mov",
            ct if ct.contains("image/") => ".png", // fallback for images
            ct if ct.contains("video/") => ".mp4", // fallback for videos
            _ => "",
        };

        if !ext.is_empty() {
            let mut new_path = path.clone();
            let new_name = format!("{}{}", path.file_name().unwrap_or_default().to_string_lossy(), ext);
            new_path.set_file_name(new_name);
            new_path
        } else {
            path.clone()
        }
    } else {
        path.clone()
    };

    // Create file
    let mut file = tokio::fs::File::create(&final_path)
        .await
        .map_err(|e| format!("Failed to create file: {}", e))?;

    // Stream the response body to disk
    let mut stream = response.bytes_stream();

    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| format!("Stream error: {}", e))?;
        file.write_all(&chunk)
            .await
            .map_err(|e| format!("Write error: {}", e))?;
    }

    file.flush()
        .await
        .map_err(|e| format!("Flush error: {}", e))?;

    Ok(final_path)
}
