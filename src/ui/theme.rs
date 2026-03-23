//! Theme colors for the mod menu interface
//!
//! A sleek, minimal design system with refined aesthetics
//! Colors are cached on first access since the theme is a compile-time constant.

use objc2::rc::Retained;
use objc2_ui_kit::UIColor;
use once_cell::sync::Lazy;

/// Theme colors for the menu
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ThemeVariant {
    /// Obsidian - Deep black with violet accents
    Default,
    /// Midnight - Rich navy with electric blue highlights
    DeepBlue,
    /// Aurora - Dark with vibrant gradient accents
    Sunset,
    /// Carbon - Pure dark with neon green
    DarkForest,
    /// Cyberpunk - Neon Yellow accent on dark gunmetal background
    Cyberpunk,
    /// Dracula - Purple/Pink accent on dark blue-grey background
    Dracula,
    /// Monokai - Green/Yellow accent on brown-grey background
    Monokai,
    /// Nord - Ice Blue accent on polar night grey background
    Nord,
    /// Oceanic - Teal accent on deep ocean blue background
    Oceanic,
    /// Vampire - Blood Red accent on pitch black background
    Vampire,
    /// Void - White/Grey accent on pure black background (minimalist)
    Void,
    /// Royal - Gold accent on deep purple background
    Royal,
    /// Matrix - Hacker Green accent on black background
    Matrix,
    /// Solarized - Cyan/Blue accent on solarized dark background
    Solarized,
}

// -- Cached theme color statics (computed once on first access) --

static BACKGROUND: Lazy<Retained<UIColor>> = Lazy::new(|| match Theme::current() {
    ThemeVariant::Default => UIColor::colorWithRed_green_blue_alpha(0.04, 0.04, 0.05, 0.92),
    ThemeVariant::DeepBlue => UIColor::colorWithRed_green_blue_alpha(0.02, 0.03, 0.08, 0.94),
    ThemeVariant::Sunset => UIColor::colorWithRed_green_blue_alpha(0.06, 0.03, 0.06, 0.94),
    ThemeVariant::Cyberpunk => UIColor::colorWithRed_green_blue_alpha(0.10, 0.10, 0.12, 0.94),
    ThemeVariant::Dracula => UIColor::colorWithRed_green_blue_alpha(0.16, 0.17, 0.21, 0.94),
    ThemeVariant::Monokai => UIColor::colorWithRed_green_blue_alpha(0.15, 0.14, 0.12, 0.94),
    ThemeVariant::Nord => UIColor::colorWithRed_green_blue_alpha(0.18, 0.20, 0.25, 0.94),
    ThemeVariant::Oceanic => UIColor::colorWithRed_green_blue_alpha(0.05, 0.10, 0.15, 0.94),
    ThemeVariant::Vampire => UIColor::colorWithRed_green_blue_alpha(0.02, 0.00, 0.00, 0.94),
    ThemeVariant::Void => UIColor::colorWithRed_green_blue_alpha(0.00, 0.00, 0.00, 0.94),
    ThemeVariant::Royal => UIColor::colorWithRed_green_blue_alpha(0.10, 0.05, 0.15, 0.94),
    ThemeVariant::Matrix => UIColor::colorWithRed_green_blue_alpha(0.00, 0.02, 0.00, 0.94),
    ThemeVariant::Solarized => UIColor::colorWithRed_green_blue_alpha(0.00, 0.17, 0.21, 0.94),
    ThemeVariant::DarkForest => UIColor::colorWithRed_green_blue_alpha(0.02, 0.05, 0.03, 0.94),
});

static GRADIENT_START: Lazy<Retained<UIColor>> = Lazy::new(|| match Theme::current() {
    ThemeVariant::Default => UIColor::colorWithRed_green_blue_alpha(0.06, 0.05, 0.08, 0.98),
    ThemeVariant::DeepBlue => UIColor::colorWithRed_green_blue_alpha(0.02, 0.04, 0.12, 0.98),
    ThemeVariant::Sunset => UIColor::colorWithRed_green_blue_alpha(0.08, 0.04, 0.10, 0.98),
    ThemeVariant::Cyberpunk => UIColor::colorWithRed_green_blue_alpha(0.12, 0.12, 0.14, 0.98),
    ThemeVariant::Dracula => UIColor::colorWithRed_green_blue_alpha(0.20, 0.21, 0.25, 0.98),
    ThemeVariant::Monokai => UIColor::colorWithRed_green_blue_alpha(0.18, 0.17, 0.15, 0.98),
    ThemeVariant::Nord => UIColor::colorWithRed_green_blue_alpha(0.22, 0.24, 0.29, 0.98),
    ThemeVariant::Oceanic => UIColor::colorWithRed_green_blue_alpha(0.08, 0.14, 0.20, 0.98),
    ThemeVariant::Vampire => UIColor::colorWithRed_green_blue_alpha(0.05, 0.00, 0.00, 0.98),
    ThemeVariant::Void => UIColor::colorWithRed_green_blue_alpha(0.02, 0.02, 0.02, 0.98),
    ThemeVariant::Royal => UIColor::colorWithRed_green_blue_alpha(0.14, 0.08, 0.20, 0.98),
    ThemeVariant::Matrix => UIColor::colorWithRed_green_blue_alpha(0.00, 0.05, 0.00, 0.98),
    ThemeVariant::Solarized => UIColor::colorWithRed_green_blue_alpha(0.02, 0.20, 0.25, 0.98),
    ThemeVariant::DarkForest => UIColor::colorWithRed_green_blue_alpha(0.03, 0.08, 0.04, 0.98),
});

static GRADIENT_END: Lazy<Retained<UIColor>> = Lazy::new(|| match Theme::current() {
    ThemeVariant::Default => UIColor::colorWithRed_green_blue_alpha(0.02, 0.02, 0.03, 0.98),
    ThemeVariant::DeepBlue => UIColor::colorWithRed_green_blue_alpha(0.01, 0.01, 0.04, 0.98),
    ThemeVariant::Sunset => UIColor::colorWithRed_green_blue_alpha(0.04, 0.02, 0.05, 0.98),
    ThemeVariant::Cyberpunk => UIColor::colorWithRed_green_blue_alpha(0.08, 0.08, 0.10, 0.98),
    ThemeVariant::Dracula => UIColor::colorWithRed_green_blue_alpha(0.14, 0.15, 0.18, 0.98),
    ThemeVariant::Monokai => UIColor::colorWithRed_green_blue_alpha(0.12, 0.11, 0.10, 0.98),
    ThemeVariant::Nord => UIColor::colorWithRed_green_blue_alpha(0.15, 0.17, 0.21, 0.98),
    ThemeVariant::Oceanic => UIColor::colorWithRed_green_blue_alpha(0.03, 0.07, 0.12, 0.98),
    ThemeVariant::Vampire => UIColor::colorWithRed_green_blue_alpha(0.00, 0.00, 0.00, 0.98),
    ThemeVariant::Void => UIColor::colorWithRed_green_blue_alpha(0.00, 0.00, 0.00, 0.98),
    ThemeVariant::Royal => UIColor::colorWithRed_green_blue_alpha(0.08, 0.03, 0.12, 0.98),
    ThemeVariant::Matrix => UIColor::colorWithRed_green_blue_alpha(0.00, 0.01, 0.00, 0.98),
    ThemeVariant::Solarized => UIColor::colorWithRed_green_blue_alpha(0.00, 0.15, 0.18, 0.98),
    ThemeVariant::DarkForest => UIColor::colorWithRed_green_blue_alpha(0.01, 0.03, 0.02, 0.98),
});

static HEADER: Lazy<Retained<UIColor>> = Lazy::new(|| match Theme::current() {
    ThemeVariant::Default => UIColor::colorWithRed_green_blue_alpha(1.0, 1.0, 1.0, 0.03),
    ThemeVariant::DeepBlue => UIColor::colorWithRed_green_blue_alpha(0.3, 0.5, 0.9, 0.08),
    ThemeVariant::Sunset => UIColor::colorWithRed_green_blue_alpha(0.8, 0.3, 0.5, 0.08),
    ThemeVariant::Cyberpunk => UIColor::colorWithRed_green_blue_alpha(0.10, 0.10, 0.12, 0.08),
    ThemeVariant::Dracula => UIColor::colorWithRed_green_blue_alpha(0.16, 0.17, 0.21, 0.08),
    ThemeVariant::Monokai => UIColor::colorWithRed_green_blue_alpha(0.15, 0.14, 0.12, 0.08),
    ThemeVariant::Nord => UIColor::colorWithRed_green_blue_alpha(0.18, 0.20, 0.25, 0.08),
    ThemeVariant::Oceanic => UIColor::colorWithRed_green_blue_alpha(0.05, 0.10, 0.15, 0.08),
    ThemeVariant::Vampire => UIColor::colorWithRed_green_blue_alpha(0.10, 0.00, 0.00, 0.08),
    ThemeVariant::Void => UIColor::colorWithRed_green_blue_alpha(0.10, 0.10, 0.10, 0.08),
    ThemeVariant::Royal => UIColor::colorWithRed_green_blue_alpha(0.14, 0.08, 0.20, 0.08),
    ThemeVariant::Matrix => UIColor::colorWithRed_green_blue_alpha(0.00, 0.10, 0.00, 0.08),
    ThemeVariant::Solarized => UIColor::colorWithRed_green_blue_alpha(0.00, 0.17, 0.21, 0.08),
    ThemeVariant::DarkForest => UIColor::colorWithRed_green_blue_alpha(0.2, 0.8, 0.3, 0.06),
});

static ACCENT: Lazy<Retained<UIColor>> = Lazy::new(|| match Theme::current() {
    ThemeVariant::Default => UIColor::colorWithRed_green_blue_alpha(0.65, 0.45, 1.0, 1.0),
    ThemeVariant::DeepBlue => UIColor::colorWithRed_green_blue_alpha(0.35, 0.65, 1.0, 1.0),
    ThemeVariant::Sunset => UIColor::colorWithRed_green_blue_alpha(1.0, 0.45, 0.55, 1.0),
    ThemeVariant::Cyberpunk => UIColor::colorWithRed_green_blue_alpha(1.0, 0.9, 0.0, 1.0),
    ThemeVariant::Dracula => UIColor::colorWithRed_green_blue_alpha(0.74, 0.46, 1.0, 1.0),
    ThemeVariant::Monokai => UIColor::colorWithRed_green_blue_alpha(0.9, 0.86, 0.45, 1.0),
    ThemeVariant::Nord => UIColor::colorWithRed_green_blue_alpha(0.53, 0.75, 0.81, 1.0),
    ThemeVariant::Oceanic => UIColor::colorWithRed_green_blue_alpha(0.3, 0.8, 0.8, 1.0),
    ThemeVariant::Vampire => UIColor::colorWithRed_green_blue_alpha(0.8, 0.0, 0.0, 1.0),
    ThemeVariant::Void => UIColor::colorWithRed_green_blue_alpha(0.9, 0.9, 0.9, 1.0),
    ThemeVariant::Royal => UIColor::colorWithRed_green_blue_alpha(1.0, 0.84, 0.0, 1.0),
    ThemeVariant::Matrix => UIColor::colorWithRed_green_blue_alpha(0.0, 1.0, 0.0, 1.0),
    ThemeVariant::Solarized => UIColor::colorWithRed_green_blue_alpha(0.16, 0.63, 0.6, 1.0),
    ThemeVariant::DarkForest => UIColor::colorWithRed_green_blue_alpha(0.2, 1.0, 0.6, 1.0),
});

static ACCENT_SOFT: Lazy<Retained<UIColor>> = Lazy::new(|| match Theme::current() {
    ThemeVariant::Default => UIColor::colorWithRed_green_blue_alpha(0.65, 0.45, 1.0, 0.15),
    ThemeVariant::DeepBlue => UIColor::colorWithRed_green_blue_alpha(0.35, 0.65, 1.0, 0.15),
    ThemeVariant::Sunset => UIColor::colorWithRed_green_blue_alpha(1.0, 0.45, 0.55, 0.15),
    ThemeVariant::Cyberpunk => UIColor::colorWithRed_green_blue_alpha(1.0, 0.9, 0.0, 0.15),
    ThemeVariant::Dracula => UIColor::colorWithRed_green_blue_alpha(0.74, 0.46, 1.0, 0.15),
    ThemeVariant::Monokai => UIColor::colorWithRed_green_blue_alpha(0.9, 0.86, 0.45, 0.15),
    ThemeVariant::Nord => UIColor::colorWithRed_green_blue_alpha(0.53, 0.75, 0.81, 0.15),
    ThemeVariant::Oceanic => UIColor::colorWithRed_green_blue_alpha(0.3, 0.8, 0.8, 0.15),
    ThemeVariant::Vampire => UIColor::colorWithRed_green_blue_alpha(0.8, 0.0, 0.0, 0.15),
    ThemeVariant::Void => UIColor::colorWithRed_green_blue_alpha(0.9, 0.9, 0.9, 0.15),
    ThemeVariant::Royal => UIColor::colorWithRed_green_blue_alpha(1.0, 0.84, 0.0, 0.15),
    ThemeVariant::Matrix => UIColor::colorWithRed_green_blue_alpha(0.0, 1.0, 0.0, 0.15),
    ThemeVariant::Solarized => UIColor::colorWithRed_green_blue_alpha(0.16, 0.63, 0.6, 0.15),
    ThemeVariant::DarkForest => UIColor::colorWithRed_green_blue_alpha(0.2, 1.0, 0.6, 0.12),
});

static ACCENT_SHADOW: Lazy<Retained<UIColor>> = Lazy::new(|| match Theme::current() {
    ThemeVariant::Default => UIColor::colorWithRed_green_blue_alpha(0.65, 0.45, 1.0, 0.3),
    ThemeVariant::DeepBlue => UIColor::colorWithRed_green_blue_alpha(0.35, 0.65, 1.0, 0.3),
    ThemeVariant::Sunset => UIColor::colorWithRed_green_blue_alpha(1.0, 0.45, 0.55, 0.3),
    ThemeVariant::Cyberpunk => UIColor::colorWithRed_green_blue_alpha(1.0, 0.9, 0.0, 0.3),
    ThemeVariant::Dracula => UIColor::colorWithRed_green_blue_alpha(0.74, 0.46, 1.0, 0.3),
    ThemeVariant::Monokai => UIColor::colorWithRed_green_blue_alpha(0.9, 0.86, 0.45, 0.3),
    ThemeVariant::Nord => UIColor::colorWithRed_green_blue_alpha(0.53, 0.75, 0.81, 0.3),
    ThemeVariant::Oceanic => UIColor::colorWithRed_green_blue_alpha(0.3, 0.8, 0.8, 0.3),
    ThemeVariant::Vampire => UIColor::colorWithRed_green_blue_alpha(0.8, 0.0, 0.0, 0.3),
    ThemeVariant::Void => UIColor::colorWithRed_green_blue_alpha(0.9, 0.9, 0.9, 0.3),
    ThemeVariant::Royal => UIColor::colorWithRed_green_blue_alpha(1.0, 0.84, 0.0, 0.3),
    ThemeVariant::Matrix => UIColor::colorWithRed_green_blue_alpha(0.0, 1.0, 0.0, 0.3),
    ThemeVariant::Solarized => UIColor::colorWithRed_green_blue_alpha(0.16, 0.63, 0.6, 0.3),
    ThemeVariant::DarkForest => UIColor::colorWithRed_green_blue_alpha(0.2, 1.0, 0.6, 0.25),
});

static TEXT: Lazy<Retained<UIColor>> =
    Lazy::new(|| UIColor::colorWithRed_green_blue_alpha(0.98, 0.98, 1.0, 1.0));

static TEXT_SECONDARY: Lazy<Retained<UIColor>> =
    Lazy::new(|| UIColor::colorWithRed_green_blue_alpha(0.6, 0.6, 0.65, 1.0));

static TEXT_TERTIARY: Lazy<Retained<UIColor>> =
    Lazy::new(|| UIColor::colorWithRed_green_blue_alpha(0.4, 0.4, 0.45, 1.0));

static TOGGLE_OFF: Lazy<Retained<UIColor>> =
    Lazy::new(|| UIColor::colorWithRed_green_blue_alpha(0.2, 0.2, 0.22, 1.0));

static KNOB_ON: Lazy<Retained<UIColor>> =
    Lazy::new(|| UIColor::colorWithRed_green_blue_alpha(1.0, 1.0, 1.0, 1.0));

static KNOB_OFF: Lazy<Retained<UIColor>> =
    Lazy::new(|| UIColor::colorWithRed_green_blue_alpha(0.55, 0.55, 0.6, 1.0));

static SLIDER_TRACK_INACTIVE: Lazy<Retained<UIColor>> =
    Lazy::new(|| UIColor::colorWithRed_green_blue_alpha(1.0, 1.0, 1.0, 0.08));

static CONTAINER_BACKGROUND: Lazy<Retained<UIColor>> =
    Lazy::new(|| UIColor::colorWithRed_green_blue_alpha(1.0, 1.0, 1.0, 0.04));

static CONTAINER_ACTIVE: Lazy<Retained<UIColor>> =
    Lazy::new(|| UIColor::colorWithRed_green_blue_alpha(1.0, 1.0, 1.0, 0.08));

static CONTAINER_BORDER: Lazy<Retained<UIColor>> =
    Lazy::new(|| UIColor::colorWithRed_green_blue_alpha(1.0, 1.0, 1.0, 0.06));

static MENU_BORDER: Lazy<Retained<UIColor>> =
    Lazy::new(|| UIColor::colorWithRed_green_blue_alpha(1.0, 1.0, 1.0, 0.08));

static TOGGLE_BUTTON_BACKGROUND: Lazy<Retained<UIColor>> =
    Lazy::new(|| UIColor::colorWithRed_green_blue_alpha(0.08, 0.08, 0.1, 0.85));

static SHADOW: Lazy<Retained<UIColor>> =
    Lazy::new(|| UIColor::colorWithRed_green_blue_alpha(0.0, 0.0, 0.0, 1.0));

static INPUT_BACKGROUND: Lazy<Retained<UIColor>> =
    Lazy::new(|| UIColor::colorWithRed_green_blue_alpha(0.0, 0.0, 0.0, 0.25));

static SUCCESS: Lazy<Retained<UIColor>> =
    Lazy::new(|| UIColor::colorWithRed_green_blue_alpha(0.2, 0.9, 0.5, 1.0));

static WARNING: Lazy<Retained<UIColor>> =
    Lazy::new(|| UIColor::colorWithRed_green_blue_alpha(1.0, 0.75, 0.2, 1.0));

static ERROR: Lazy<Retained<UIColor>> =
    Lazy::new(|| UIColor::colorWithRed_green_blue_alpha(1.0, 0.35, 0.4, 1.0));

/// Theme manager for UI colors and styling
pub struct Theme;

impl Theme {
    pub fn current() -> ThemeVariant {
        crate::config::SELECTED_THEME
    }

    /// Main background color
    pub fn background() -> Retained<UIColor> {
        BACKGROUND.clone()
    }

    /// Gradient start
    pub fn gradient_start() -> Retained<UIColor> {
        GRADIENT_START.clone()
    }

    /// Gradient end
    pub fn gradient_end() -> Retained<UIColor> {
        GRADIENT_END.clone()
    }

    /// Header background
    pub fn header() -> Retained<UIColor> {
        HEADER.clone()
    }

    /// Primary accent
    pub fn accent() -> Retained<UIColor> {
        ACCENT.clone()
    }

    /// Accent glow
    pub fn accent_soft() -> Retained<UIColor> {
        ACCENT_SOFT.clone()
    }

    /// Primary text
    pub fn text() -> Retained<UIColor> {
        TEXT.clone()
    }

    /// Secondary text
    pub fn text_secondary() -> Retained<UIColor> {
        TEXT_SECONDARY.clone()
    }

    /// Tertiary text
    pub fn text_tertiary() -> Retained<UIColor> {
        TEXT_TERTIARY.clone()
    }

    /// Toggle off state
    pub fn toggle_off() -> Retained<UIColor> {
        TOGGLE_OFF.clone()
    }

    /// Toggle knob color when active
    pub fn knob_on() -> Retained<UIColor> {
        KNOB_ON.clone()
    }

    /// Toggle knob when off
    pub fn knob_off() -> Retained<UIColor> {
        KNOB_OFF.clone()
    }

    /// Slider inactive track
    pub fn slider_track_inactive() -> Retained<UIColor> {
        SLIDER_TRACK_INACTIVE.clone()
    }

    /// Container background
    pub fn container_background() -> Retained<UIColor> {
        CONTAINER_BACKGROUND.clone()
    }

    /// Container on hover/active
    pub fn container_active() -> Retained<UIColor> {
        CONTAINER_ACTIVE.clone()
    }

    /// Container border
    pub fn container_border() -> Retained<UIColor> {
        CONTAINER_BORDER.clone()
    }

    /// Menu window border
    pub fn menu_border() -> Retained<UIColor> {
        MENU_BORDER.clone()
    }

    /// Toggle button background
    pub fn toggle_button_background() -> Retained<UIColor> {
        TOGGLE_BUTTON_BACKGROUND.clone()
    }

    /// Toggle button border
    pub fn toggle_button_border() -> Retained<UIColor> {
        Self::container_border()
    }

    /// Shadow color
    pub fn shadow() -> Retained<UIColor> {
        SHADOW.clone()
    }

    /// Accent shadow
    pub fn accent_shadow() -> Retained<UIColor> {
        ACCENT_SHADOW.clone()
    }

    /// Input background
    pub fn input_background() -> Retained<UIColor> {
        INPUT_BACKGROUND.clone()
    }

    /// Input border
    pub fn input_border() -> Retained<UIColor> {
        Self::container_border()
    }

    /// Input placeholder
    pub fn input_placeholder_background() -> Retained<UIColor> {
        UIColor::clearColor()
    }

    /// Muted arrow color
    pub fn arrow_muted() -> Retained<UIColor> {
        Self::text_tertiary()
    }

    /// Success state
    pub fn success() -> Retained<UIColor> {
        SUCCESS.clone()
    }

    /// Warning state
    pub fn warning() -> Retained<UIColor> {
        WARNING.clone()
    }

    /// Error state
    pub fn error() -> Retained<UIColor> {
        ERROR.clone()
    }
}
