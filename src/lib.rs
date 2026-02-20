#![allow(warnings)] // suppress warnings

mod config;
mod entry;
mod memory;
mod ui;
mod utils;

pub use utils::logger;

// Entry point
#[ctor::ctor]
fn init() {
    logger::info("Library initializing...");

    entry::init();
}
