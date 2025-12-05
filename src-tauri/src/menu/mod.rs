//! Application menu module
//!
//! Handles menu creation and event handling for different platforms.

#[cfg(target_os = "macos")]
mod macos;

#[cfg(target_os = "macos")]
pub use macos::{setup_menu, setup_menu_events};
