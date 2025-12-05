//! Window management module
//!
//! Handles window creation, configuration, and lifecycle events.

mod events;

use tauri::{WebviewUrl, WebviewWindowBuilder};

#[cfg(target_os = "macos")]
use tauri::TitleBarStyle;

pub use events::{handle_run_event, handle_window_event};

/// Create the main window builder with all configuration
pub fn create_window_builder<R: tauri::Runtime, M: tauri::Manager<R>>(app: &M) -> WebviewWindowBuilder<'_, R, M> {
    let mut win_builder = WebviewWindowBuilder::new(app, "main", WebviewUrl::App(Default::default()))
        .title("Cushion")
        .inner_size(1200.0, 800.0)
        .min_inner_size(800.0, 600.0)
        .resizable(true)
        .center()
        .hidden_title(true)
        .transparent(true)
        // Disable Tauri's built-in drag/drop handler to pass events through to the WebView.
        // This allows the web-app's react-dropzone implementation to handle file uploads directly.
        .disable_drag_drop_handler();

    #[cfg(target_os = "macos")]
    {
        win_builder = win_builder
            .title_bar_style(TitleBarStyle::Overlay)
            .traffic_light_position(tauri::LogicalPosition::new(15.0, 20.0))
            .initialization_script(get_initialization_script());
    }

    win_builder
}

/// Get the JavaScript initialization script for the webview
#[cfg(target_os = "macos")]
fn get_initialization_script() -> &'static str {
    r#"
        // Disable swipe navigation and overscroll bounce
        document.addEventListener('DOMContentLoaded', function() {
            document.body.style.overscrollBehaviorX = 'none';
            document.body.style.overscrollBehaviorY = 'none';
            document.body.style.overflowX = 'hidden';
            document.documentElement.style.overscrollBehaviorX = 'none';
            document.documentElement.style.overscrollBehaviorY = 'none';
            document.documentElement.style.overflowX = 'hidden';

            // Disable spellcheck and text prediction on all inputs
            const disableInputPrediction = (el) => {
                el.setAttribute('spellcheck', 'false');
                el.setAttribute('autocomplete', 'off');
                el.setAttribute('autocorrect', 'off');
                el.setAttribute('autocapitalize', 'off');
                el.setAttribute('writingsuggestions', 'false');
            };

            // Apply to existing inputs
            document.querySelectorAll('input, textarea, [contenteditable="true"]').forEach(disableInputPrediction);

            // Apply to dynamically added inputs via MutationObserver
            const observer = new MutationObserver((mutations) => {
                mutations.forEach((mutation) => {
                    mutation.addedNodes.forEach((node) => {
                        if (node.nodeType === 1) {
                            if (node.matches && node.matches('input, textarea, [contenteditable="true"]')) {
                                disableInputPrediction(node);
                            }
                            node.querySelectorAll && node.querySelectorAll('input, textarea, [contenteditable="true"]').forEach(disableInputPrediction);
                        }
                    });
                });
            });
            observer.observe(document.body, { childList: true, subtree: true });
        });

        // Enable zoom controls with webview zoom (persisted to localStorage)
        let zoomLevel = parseFloat(localStorage.getItem('cushion_zoom_level')) || 1.0;

        // Apply saved zoom on startup
        if (zoomLevel !== 1.0) {
            window.__TAURI__.core.invoke('set_zoom_level', { zoom: zoomLevel });
        }

        document.addEventListener('keydown', async function(e) {
            if ((e.metaKey || e.ctrlKey) && e.key === '=') {
                e.preventDefault();
                zoomLevel = Math.min(zoomLevel + 0.1, 3.0);
                localStorage.setItem('cushion_zoom_level', zoomLevel.toString());
                await window.__TAURI__.core.invoke('set_zoom_level', { zoom: zoomLevel });
            } else if ((e.metaKey || e.ctrlKey) && e.key === '-') {
                e.preventDefault();
                zoomLevel = Math.max(zoomLevel - 0.1, 0.5);
                localStorage.setItem('cushion_zoom_level', zoomLevel.toString());
                await window.__TAURI__.core.invoke('set_zoom_level', { zoom: zoomLevel });
            } else if ((e.metaKey || e.ctrlKey) && e.key === '0') {
                e.preventDefault();
                zoomLevel = 1.0;
                localStorage.setItem('cushion_zoom_level', zoomLevel.toString());
                await window.__TAURI__.core.invoke('set_zoom_level', { zoom: zoomLevel });
            }
        });

        // Handle link clicks
        document.addEventListener('click', async function(e) {
            let target = e.target;
            while (target && target.tagName !== 'A') {
                target = target.parentElement;
            }

            if (target && target.tagName === 'A' && target.href) {
                const url = new URL(target.href);
                const isInternal = url.origin === window.location.origin
                    || url.hostname === 'cushion.so'
                    || url.hostname.endsWith('.cushion.so')
                    || url.hostname === 'localhost';

                if (isInternal) {
                    // Remove _blank target so internal links navigate in same window
                    if (target.target === '_blank') {
                        target.removeAttribute('target');
                    }
                } else {
                    // External link - open in default browser
                    e.preventDefault();
                    await window.__TAURI__.core.invoke('open_url', { url: target.href });
                }
            }
        }, true);
    "#
}

/// Recreate the main window (fallback for when window is unexpectedly destroyed)
#[cfg(target_os = "macos")]
pub fn recreate_window(app_handle: &tauri::AppHandle) {
    let win_builder = WebviewWindowBuilder::new(
        app_handle,
        "main",
        WebviewUrl::App(Default::default())
    )
    .title("Cushion")
    .inner_size(1200.0, 800.0)
    .min_inner_size(800.0, 600.0)
    .resizable(true)
    .center()
    .hidden_title(true)
    .transparent(true)
    .visible(true) // Ensure window is visible to attach to display
    .title_bar_style(TitleBarStyle::Overlay)
    .traffic_light_position(tauri::LogicalPosition::new(15.0, 20.0));

    let _ = win_builder.build();
}
