//! macOS menu setup
//!
//! Creates the application menu bar for macOS.

use tauri::menu::{Menu, MenuItem, PredefinedMenuItem, Submenu};

use crate::updater;

/// Setup the application menu bar
pub fn setup_menu(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let check_updates = MenuItem::with_id(app, "check-for-updates", "Check for Updates...", true, None::<&str>)?;

    let app_submenu = Submenu::with_items(
        app,
        "Cushion",
        true,
        &[
            &PredefinedMenuItem::about(app, Some("About Cushion"), None)?,
            &PredefinedMenuItem::separator(app)?,
            &check_updates,
            &PredefinedMenuItem::separator(app)?,
            &PredefinedMenuItem::hide(app, Some("Hide Cushion"))?,
            &PredefinedMenuItem::hide_others(app, Some("Hide Others"))?,
            &PredefinedMenuItem::show_all(app, Some("Show All"))?,
            &PredefinedMenuItem::separator(app)?,
            &PredefinedMenuItem::quit(app, Some("Quit Cushion"))?,
        ],
    )?;

    let edit_submenu = Submenu::with_items(
        app,
        "Edit",
        true,
        &[
            &PredefinedMenuItem::undo(app, None)?,
            &PredefinedMenuItem::redo(app, None)?,
            &PredefinedMenuItem::separator(app)?,
            &PredefinedMenuItem::cut(app, None)?,
            &PredefinedMenuItem::copy(app, None)?,
            &PredefinedMenuItem::paste(app, None)?,
            &PredefinedMenuItem::separator(app)?,
            &PredefinedMenuItem::select_all(app, None)?,
        ],
    )?;

    let view_submenu = Submenu::with_items(
        app,
        "View",
        true,
        &[
            &PredefinedMenuItem::fullscreen(app, None)?,
        ],
    )?;

    let window_submenu = Submenu::with_items(
        app,
        "Window",
        true,
        &[
            &PredefinedMenuItem::minimize(app, None)?,
            &PredefinedMenuItem::maximize(app, None)?,
            &PredefinedMenuItem::separator(app)?,
            &PredefinedMenuItem::close_window(app, None)?,
        ],
    )?;

    let menu = Menu::with_items(app, &[&app_submenu, &edit_submenu, &view_submenu, &window_submenu])?;
    app.set_menu(menu)?;

    Ok(())
}

/// Setup menu event handlers
pub fn setup_menu_events(app: &mut tauri::App) {
    let handle = app.handle().clone();
    app.on_menu_event(move |_app, event| {
        if event.id().as_ref() == "check-for-updates" {
            updater::show_update_dialog(&handle, true);
        }
    });
}
