//! # Platform-Specific Operations
//!
//! This module contains low-level primitives for interacting with the operating system (iOS/macOS).
//! It handles:
//! - Mach exception handling for hardware breakpoints
//! - Thread suspension/resumption for safe memory patching
//! - Task port management

pub mod breakpoint;
pub mod thread;
