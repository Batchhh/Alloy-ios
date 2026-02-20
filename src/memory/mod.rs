//! # Memory Module
//!
//! This module provides comprehensive utilities for memory manipulation, introspection, and analysis.
//! It serves as the core foundation for runtime modifications, offering safe abstractions over
//! low-level memory operations.
//!
//! ## Submodules
//!
//! - [`allocation`] - Memory allocation utilities and shellcode management
//! - [`ffi`] - Foreign Function Interface bindings and exception handling
//! - [`info`] - Information gathering (symbols, images, protection, etc.)
//! - [`manipulation`] - Active memory manipulation (hooks, patches, R/W)
//! - [`platform`] - Platform-specific primitives (threads, breakpoints)

pub mod allocation;
pub mod ffi;
pub mod info;
pub mod manipulation;
pub mod platform;

pub use allocation::shellcode;
pub use info::{code_cave, image, protection, scan};
pub use manipulation::{checksum, hook, patch, rw};
pub use platform::{breakpoint, thread};
