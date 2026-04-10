//! # Alloy — iOS Modding Framework
//!
//! High-performance iOS modding framework built in Rust.
//!
//! ## Quick Start
//!
//! 1. Configure your target in `src/config.rs`
//! 2. Register UI elements and set up hooks below in `init()`
//! 3. Build & deploy with `make deploy`
//!
//! For full documentation, see the `docs/` directory:
//! - [`docs/getting-started.md`](../docs/getting-started.md) — Setup, building, deploying
//! - [`docs/ui.md`](../docs/ui.md) — Menu system & UI components
//! - [`docs/memory.md`](../docs/memory.md) — Hooking, patching, shellcode (via [`specter-mem`](https://crates.io/crates/specter-mem))
//! - [`docs/architecture.md`](../docs/architecture.md) — Project structure & build pipeline

mod config;
mod ui;
mod utils;

pub use utils::logger;

use dispatch::Queue;

// Entry point
#[ctor::ctor]
fn init() {
    logger::info("Library initializing...");

    Queue::main().exec_async(|| {
        init_ui();
    });
}

// See docs/ui.md for the full UI API reference.
// See docs/memory.md for hooking, patching, and shellcode via specter-mem.
// use specter::memory;
fn init_ui() {
    let page_id = 1;

    ui::add_tab("Main Settings", page_id);
    ui::add_section_header(page_id, "General Configurations");

    ui::add_toggle(
        page_id,
        "Enable Feature",
        "feature_enabled",
        false,
        Some(|state: bool| {
            logger::info(&format!("Feature: {}", state));
        }),
    );

    ui::add_slider_with_options(
        page_id,
        "Speed Multiplier",
        "speed_value",
        0.0,
        100.0,
        50.0,
        ui::SliderOptions::new().with_toggle(
            ui::ToggleOptions::new("speed_enabled", true).with_callback(|state: bool| {
                logger::info(&format!("Speed enabled: {}", state));
            }),
        ),
        Some(|val: f32| {
            logger::info(&format!("Speed: {}", val));
        }),
    );

    ui::add_input_with_options(
        page_id,
        "Player Name",
        "player_name",
        "Enter name",
        "",
        ui::InputOptions::new().with_toggle(
            ui::ToggleOptions::new("name_enabled", false).with_callback(|state: bool| {
                logger::info(&format!("Name enabled: {}", state));
            }),
        ),
        Some(|text: String| {
            logger::info(&format!("Name: {}", text));
        }),
    );

    ui::native::init_overlay();
}
