//! In-Game//! User interface modules

pub mod theme;
pub mod components;
pub mod menu;
pub mod window;
pub mod pref;

// Re-export public API for backward compatibility
pub mod native {
    pub use super::window::init_overlay;
}

// Re-export menu building API
pub use menu::{add_action_button, add_button, add_button_with_nav, add_input, add_label, add_page, add_slider, add_toggle};
