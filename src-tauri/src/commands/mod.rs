/// Tauri command modules
///
/// Commands are organized into separate modules by category:
/// - `download`: File download commands
/// - `notification`: System notification commands
/// - `system`: System-level commands (user agent, URL handling, etc.)
/// - `window`: Window management commands
/// - `updater`: App update checking and installation commands
pub mod download;
pub mod notification;
pub mod system;
pub mod updater;
pub mod window;
