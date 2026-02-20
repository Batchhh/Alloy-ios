//! Utils
use objc2::rc::Retained;
use objc2::{msg_send, ClassType};
use objc2_ui_kit::UIColor;
use once_cell::sync::Lazy;
use std::sync::Mutex;

use crate::ui::pref;
use crate::ui::utils::feedback::UISelectionFeedbackGenerator;

pub static FEEDBACK_GEN: Lazy<Mutex<Option<Retained<UISelectionFeedbackGenerator>>>> =
    Lazy::new(|| Mutex::new(None));

/// Triggers haptic feedback for UI interactions
pub fn trigger_feedback() {
    let mut gen = FEEDBACK_GEN.lock().unwrap();
    if gen.is_none() {
        *gen = Some(UISelectionFeedbackGenerator::new());
        if let Some(g) = gen.as_ref() {
            g.prepare();
        }
    }
    if let Some(g) = gen.as_ref() {
        g.selectionChanged();
        g.prepare();
    }
}

/// Converts a hex color string (e.g., "#FF0000" or "FF0000") to a `UIColor`
pub fn hex_to_color(hex: &str) -> Option<Retained<UIColor>> {
    let hex = hex.trim_start_matches('#');
    if hex.len() != 6 {
        return None;
    }
    let r = u8::from_str_radix(&hex[0..2], 16).ok()? as f64 / 255.0;
    let g = u8::from_str_radix(&hex[2..4], 16).ok()? as f64 / 255.0;
    let b = u8::from_str_radix(&hex[4..6], 16).ok()? as f64 / 255.0;
    Some(unsafe { msg_send![UIColor::class(), colorWithRed: r, green: g, blue: b, alpha: 1.0] })
}

/// Gets the current value of a slider
pub fn get_slider_value(key: &str) -> f32 {
    pref::Preferences::get_float(key)
}

/// Gets the current value of a text input
pub fn get_input_value(key: &str) -> String {
    pref::Preferences::get_string(key)
}

/// Gets the current selection index of a dropdown
pub fn get_dropdown_value(key: &str) -> i32 {
    pref::Preferences::get_int(key)
}

/// Gets the current state of a toggle
pub fn get_toggle_value(key: &str) -> bool {
    pref::Preferences::get_bool(key)
}
