//! Theme colors for the mod menu interface

use objc2::rc::Retained;
use objc2_ui_kit::UIColor;

/// Theme colors for the menu
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ThemeVariant {
    Default,
    DeepBlue,
    Sunset,
    Light,
}

pub struct Theme;

impl Theme {
    fn current() -> ThemeVariant {
        crate::config::SELECTED_THEME
    }

    pub fn background() -> Retained<UIColor> {
        match Self::current() {
            ThemeVariant::Default => UIColor::clearColor(),
            ThemeVariant::DeepBlue => UIColor::colorWithRed_green_blue_alpha(0.05, 0.05, 0.1, 0.95),
            ThemeVariant::Sunset => UIColor::colorWithRed_green_blue_alpha(0.1, 0.05, 0.05, 0.95),
            ThemeVariant::Light => UIColor::colorWithRed_green_blue_alpha(1.0, 1.0, 1.0, 0.95),
        }
    }
    
    pub fn header() -> Retained<UIColor> {
        match Self::current() {
            ThemeVariant::Default => UIColor::colorWithRed_green_blue_alpha(0.0, 0.0, 0.0, 0.3),
            ThemeVariant::DeepBlue => UIColor::colorWithRed_green_blue_alpha(0.0, 0.0, 0.2, 0.8),
            ThemeVariant::Sunset => UIColor::colorWithRed_green_blue_alpha(0.2, 0.0, 0.0, 0.8),
            ThemeVariant::Light => UIColor::colorWithRed_green_blue_alpha(0.95, 0.95, 0.95, 0.9),
        }
    }
    
    pub fn accent() -> Retained<UIColor> {
        match Self::current() {
            ThemeVariant::Default => UIColor::whiteColor(),
            ThemeVariant::DeepBlue => UIColor::cyanColor(),
            ThemeVariant::Sunset => UIColor::orangeColor(),
            ThemeVariant::Light => UIColor::systemBlueColor(),
        }
    }
    
    pub fn text() -> Retained<UIColor> {
        match Self::current() {
            ThemeVariant::Default | ThemeVariant::DeepBlue | ThemeVariant::Sunset => UIColor::whiteColor(),
            ThemeVariant::Light => UIColor::blackColor(),
        }
    }
    
    pub fn text_secondary() -> Retained<UIColor> {
        match Self::current() {
            ThemeVariant::Default => UIColor::lightGrayColor(),
            ThemeVariant::DeepBlue => UIColor::colorWithRed_green_blue_alpha(0.7, 0.8, 1.0, 1.0),
            ThemeVariant::Sunset => UIColor::colorWithRed_green_blue_alpha(1.0, 0.8, 0.7, 1.0),
            ThemeVariant::Light => UIColor::darkGrayColor(),
        }
    }
    
    pub fn toggle_off() -> Retained<UIColor> {
        match Self::current() {
            ThemeVariant::Default => UIColor::darkGrayColor(),
            ThemeVariant::DeepBlue => UIColor::colorWithRed_green_blue_alpha(0.2, 0.2, 0.4, 1.0),
            ThemeVariant::Sunset => UIColor::colorWithRed_green_blue_alpha(0.4, 0.2, 0.2, 1.0),
            ThemeVariant::Light => UIColor::lightGrayColor(),
        }
    }

    pub fn container_background() -> Retained<UIColor> {
        match Self::current() {
            ThemeVariant::Default => UIColor::colorWithRed_green_blue_alpha(1.0, 1.0, 1.0, 0.05),
            ThemeVariant::DeepBlue => UIColor::colorWithRed_green_blue_alpha(0.0, 0.2, 0.4, 0.2),
            ThemeVariant::Sunset => UIColor::colorWithRed_green_blue_alpha(0.4, 0.1, 0.1, 0.2),
            ThemeVariant::Light => UIColor::colorWithRed_green_blue_alpha(0.0, 0.0, 0.0, 0.05),
        }
    }

    pub fn container_border() -> Retained<UIColor> {
        match Self::current() {
            ThemeVariant::Default => UIColor::colorWithRed_green_blue_alpha(1.0, 1.0, 1.0, 0.1),
            ThemeVariant::DeepBlue => UIColor::colorWithRed_green_blue_alpha(0.0, 0.5, 1.0, 0.3),
            ThemeVariant::Sunset => UIColor::colorWithRed_green_blue_alpha(1.0, 0.5, 0.0, 0.3),
            ThemeVariant::Light => UIColor::colorWithRed_green_blue_alpha(0.0, 0.0, 0.0, 0.1),
        }
    }
    
    pub fn menu_border() -> Retained<UIColor> {
        match Self::current() {
            ThemeVariant::Default => UIColor::colorWithRed_green_blue_alpha(1.0, 1.0, 1.0, 0.15),
            ThemeVariant::DeepBlue => UIColor::colorWithRed_green_blue_alpha(0.0, 0.8, 1.0, 0.5),
            ThemeVariant::Sunset => UIColor::colorWithRed_green_blue_alpha(1.0, 0.6, 0.0, 0.5),
            ThemeVariant::Light => UIColor::colorWithRed_green_blue_alpha(0.8, 0.8, 0.8, 1.0),
        }
    }

    pub fn toggle_button_background() -> Retained<UIColor> {
        match Self::current() {
            ThemeVariant::Default => UIColor::colorWithRed_green_blue_alpha(0.0, 0.0, 0.0, 0.6),
            ThemeVariant::DeepBlue => UIColor::colorWithRed_green_blue_alpha(0.0, 0.1, 0.3, 0.8),
            ThemeVariant::Sunset => UIColor::colorWithRed_green_blue_alpha(0.3, 0.0, 0.0, 0.8),
            ThemeVariant::Light => UIColor::whiteColor(),
        }
    }

    pub fn toggle_button_border() -> Retained<UIColor> {
        match Self::current() {
            ThemeVariant::Default => UIColor::colorWithRed_green_blue_alpha(1.0, 1.0, 1.0, 0.2),
            ThemeVariant::DeepBlue => UIColor::colorWithRed_green_blue_alpha(0.2, 0.6, 1.0, 0.5),
            ThemeVariant::Sunset => UIColor::colorWithRed_green_blue_alpha(1.0, 0.4, 0.2, 0.5),
            ThemeVariant::Light => UIColor::systemBlueColor(),
        }
    }

    pub fn shadow() -> Retained<UIColor> {
        UIColor::blackColor()
    }
    
    pub fn knob_on() -> Retained<UIColor> {
        match Self::current() {
            ThemeVariant::Light => UIColor::whiteColor(),
            _ => UIColor::blackColor(),
        }
    }

    pub fn slider_track_inactive() -> Retained<UIColor> {
        match Self::current() {
            ThemeVariant::Light => UIColor::colorWithRed_green_blue_alpha(0.0, 0.0, 0.0, 0.1),
            _ => UIColor::colorWithRed_green_blue_alpha(1.0, 1.0, 1.0, 0.3),
        }
    }

    pub fn arrow_muted() -> Retained<UIColor> {
        match Self::current() {
            ThemeVariant::Light => UIColor::colorWithRed_green_blue_alpha(0.0, 0.0, 0.0, 0.3),
            _ => UIColor::colorWithRed_green_blue_alpha(1.0, 1.0, 1.0, 0.3),
        }
    }

    pub fn input_background() -> Retained<UIColor> {
        match Self::current() {
            ThemeVariant::Light => UIColor::colorWithRed_green_blue_alpha(0.0, 0.0, 0.0, 0.05),
            _ => UIColor::colorWithRed_green_blue_alpha(1.0, 1.0, 1.0, 0.1),
        }
    }
    
    pub fn input_border() -> Retained<UIColor> {
        match Self::current() {
            ThemeVariant::Light => UIColor::colorWithRed_green_blue_alpha(0.0, 0.0, 0.0, 0.2),
             _ => UIColor::colorWithRed_green_blue_alpha(1.0, 1.0, 1.0, 0.2)
        }
    }
    
    pub fn input_placeholder_background() -> Retained<UIColor> {
        match Self::current() {
            ThemeVariant::Light => UIColor::colorWithRed_green_blue_alpha(0.0, 0.0, 0.0, 0.05),
            _ => UIColor::colorWithRed_green_blue_alpha(0.0, 0.0, 0.0, 0.2),
        }
    }
}
