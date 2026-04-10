//! # Native UI Implementation
//!
//! This module implements the native iOS UI overlay for the mod menu.
//! It handles:
//! - Window management and overlay initialization
//! - UI Components (Buttons, Toggles, Sliders)
//! - Menu structure and navigation
//! - Theming and Preferences
//! - Interactions (Touch, Drag, etc.)
pub mod assets;
pub mod components;
pub mod menu;
pub mod pref;
#[allow(dead_code)]
pub mod theme;
#[allow(dead_code)]
pub mod utils;
#[allow(dead_code)]
pub mod window;
pub mod native {
    pub use super::window::init_overlay;
}
#[allow(unused_imports)]
pub use components::toast::{show_toast, ToastStatus};
#[allow(unused_imports)]
pub use menu::utils::{get_dropdown_value, get_input_value, get_slider_value, get_toggle_value};
#[allow(unused_imports)]
pub use menu::{
    add_action_button, add_button, add_button_with_nav, add_dropdown, add_input, add_label,
    add_section_header, add_slider, add_tab, add_toggle,
};
#[allow(unused_imports)]
pub use window::alert;
