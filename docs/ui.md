# UI System

Alloy provides a native iOS mod menu built with `objc2` bindings. The UI system handles window management, theming, navigation, and user interaction — all from Rust.

## Architecture

```
ui/
├── window.rs        # Window management, overlay initialization
├── theme.rs         # Theme definitions and color system
├── pref.rs          # Persistent preferences (NSUserDefaults)
├── menu/
│   ├── registry.rs  # Menu item registration and storage
│   ├── handler.rs   # Event handling
│   ├── items.rs     # Item rendering
│   ├── view.rs      # Menu view construction
│   └── utils.rs     # Menu helpers
├── components/
│   ├── widgets.rs   # Reusable UI elements
│   ├── floating.rs  # Floating button overlay
│   ├── toast.rs     # Toast notifications
│   └── file_picker.rs
├── assets/
│   └── icons.rs     # Icon assets
└── utils/
    ├── delegate.rs  # UIKit delegates
    ├── feedback.rs  # Haptic feedback
    ├── wrappers.rs  # iOS type wrappers
    └── animations.rs
```

## Initializing the Menu

Call `ui::native::init_overlay()` to create the floating menu overlay. This is typically done in `entry.rs` on the main dispatch queue:

```rust
use dispatch::Queue;

Queue::main().exec_async(|| {
    // Register tabs and items first, then init
    ui::native::init_overlay();
});
```

## Pages & Tabs

The menu is organized into **pages** identified by integer IDs. Each page holds a list of menu items. Tabs provide top-level navigation between pages.

```rust
// Register a tab pointing to page 1
ui::add_tab("Main Settings", 1);

// Dynamically create a new page (returns its ID)
let page_id = ui::menu::registry::add_page("Advanced");
```

## Menu Items

All items are registered to a page by its ID. Items support optional callbacks that fire on user interaction.

### Toggle

```rust
ui::add_toggle(
    page_id,
    "Enable Feature",     // display name
    "feature_enabled",    // preference key
    false,                // default value
    Some(|state: bool| {
        logger::info(&format!("Feature: {}", state));
    }),
);
```

### Slider

```rust
ui::add_slider(
    page_id,
    "Intensity",          // display name
    "intensity_level",    // preference key
    0.0,                  // min
    100.0,                // max
    50.0,                 // default
    Some(|val: f32| {
        logger::info(&format!("Intensity: {}", val));
    }),
);
```

### Slider With Toggle

```rust
ui::add_slider_with_options(
    page_id,
    "Slider Option",
    "slider_option_value",
    0.0,
    100.0,
    50.0,
    ui::SliderOptions::new().with_toggle(
        ui::ToggleOptions::new("slider_option_enabled", true).with_callback(|state: bool| {
            logger::info(&format!("Slider enabled: {}", state));
        }),
    ),
    Some(|val: f32| {
        logger::info(&format!("Slider value: {}", val));
    }),
);
```

The toggle state is stored separately from the slider value. Tapping the card
(outside the slider track) flips the toggle — the card border highlights with
the accent color when on, and reverts to the default border when off.

### Text Input

```rust
ui::add_input(
    page_id,
    "Username",           // display name
    "username",           // preference key
    "Enter your username",// placeholder
    "",                   // default
    Some(|text: String| {
        logger::info(&format!("Username: {}", text));
    }),
);
```

### Input With Toggle

```rust
ui::add_input_with_options(
    page_id,
    "Input Option",
    "input_option_value",
    "100",
    "100",
    ui::InputOptions::new().with_toggle(
        ui::ToggleOptions::new("input_option_enabled", false).with_callback(
            |state: bool| {
                logger::info(&format!("Input enabled: {}", state));
            },
        ),
    ),
    Some(|text: String| {
        logger::info(&format!("Input value: {}", text));
    }),
);
```

The toggle state is stored separately from the input text. Tapping the card
(outside the text field) flips the toggle — the card border highlights with
the accent color when on, and reverts to the default border when off.

### Dropdown

```rust
ui::add_dropdown(
    page_id,
    "Theme",
    "theme_selection",
    vec!["Light".into(), "Dark".into(), "Auto".into()],
    1,                    // default index
    Some(|idx: i32| {
        logger::info(&format!("Selected: {}", idx));
    }),
);
```

### Buttons

```rust
// Simple button
ui::add_button(page_id, "Apply Changes", Some(|| {
    logger::info("Applying...");
}));

// Action button (styled differently, for one-off actions)
ui::add_action_button(page_id, "Reset Defaults", Some(|| {
    logger::info("Resetting...");
}));

// Navigation button (navigates to another page)
ui::add_button_with_nav(page_id, "Advanced Settings", 2, Some(|| {
    logger::info("Navigating...");
}));
```

### Labels & Section Headers

```rust
ui::add_label(page_id, "Status: Active", 14.0, true, Some("#00FF00"));
ui::add_section_header(page_id, "General Configurations");
```

## Reading Values

Retrieve current values for stateful items:

```rust
let enabled: bool   = ui::get_toggle_value("feature_enabled");
let level: f32      = ui::get_slider_value("intensity_level");
let name: String    = ui::get_input_value("username");
let theme_idx: i32  = ui::get_dropdown_value("theme_selection");
```

## Toasts & Loading

```rust
// Show a toast notification
ui::show_toast("Settings saved!", ui::ToastStatus::Success);

// Show a loading indicator
ui::show_loading("Loading...");
```

## Alerts

```rust
ui::alert("Title", "Message body");
```

## Preferences

All stateful menu items (toggles, sliders, inputs, dropdowns) automatically persist their values via `NSUserDefaults` using the `modmenu.<key>` prefix. You can also use the `Preferences` API directly:

```rust
use crate::ui::pref::Preferences;

Preferences::set_bool("custom_key", true);
let val = Preferences::get_bool("custom_key");

Preferences::set_float("speed", 1.5);
Preferences::set_string("name", "Alloy");
Preferences::set_int("count", 42);
```

## Themes

Alloy ships with 14 built-in themes. Set the active theme in `src/config.rs`:

```rust
pub const SELECTED_THEME: ThemeVariant = ThemeVariant::Nord;
```

Available themes:

| Theme | Description |
|-------|-------------|
| `Default` | Deep black with violet accents |
| `DeepBlue` | Rich navy with electric blue |
| `Sunset` | Dark with coral pink accents |
| `DarkForest` | Dark green with neon mint |
| `Cyberpunk` | Gunmetal with neon yellow |
| `Dracula` | Blue-grey with purple/pink |
| `Monokai` | Brown-grey with green/yellow |
| `Nord` | Polar night with ice blue |
| `Oceanic` | Deep ocean with teal |
| `Vampire` | Pitch black with blood red |
| `Void` | Pure black with white/grey (minimalist) |
| `Royal` | Deep purple with gold |
| `Matrix` | Black with hacker green |
| `Solarized` | Solarized dark with cyan |
