//! Logging via Apple Unified Logging System 

use log::LevelFilter;
use oslog::OsLogger;
use std::sync::Once;

use crate::config;

static INIT: Once = Once::new();

fn ensure_initialized() {
    INIT.call_once(|| {
        let level = if config::DEBUG {
            LevelFilter::Debug
        } else {
            LevelFilter::Warn
        };
        OsLogger::new("com.rust_tweak")
            .level_filter(level)
            .init()
            .ok();
    });
}

pub fn info(msg: &str) {
    ensure_initialized();
    log::info!("[RGG] {}", msg);
}


pub fn debug(msg: &str) {
    ensure_initialized();
    log::debug!("[RGG] {}", msg);
}

pub fn warning(msg: &str) {
    ensure_initialized();
    log::warn!("[RGG] {}", msg);
}

pub fn error(msg: &str) {
    ensure_initialized();
    log::error!("[RGG] {}", msg);
}
