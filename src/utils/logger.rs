//! Logging via Apple Unified Logging System

use log::LevelFilter;
use oslog::OsLogger;
use std::sync::Once;

use crate::config;

static INIT: Once = Once::new();

#[cfg(dev_release)]
fn ensure_initialized() {
    INIT.call_once(|| {
        OsLogger::new("com.batch.alloy")
            .level_filter(LevelFilter::Debug)
            .init()
            .ok();
    });
}

/// Logs an informational message
pub fn info(msg: &str) {
    #[cfg(dev_release)]
    {
        ensure_initialized();
        log::info!("{}", msg);
    }
}

/// Logs a debug message
pub fn debug(msg: &str) {
    #[cfg(dev_release)]
    {
        ensure_initialized();
        log::debug!("{}", msg);
    }
}

/// Logs a warning message
pub fn warning(msg: &str) {
    #[cfg(dev_release)]
    {
        ensure_initialized();
        log::warn!("{}", msg);
    }
}

/// Logs an error message
pub fn error(msg: &str) {
    #[cfg(dev_release)]
    {
        ensure_initialized();
        log::error!("{}", msg);
    }
}
