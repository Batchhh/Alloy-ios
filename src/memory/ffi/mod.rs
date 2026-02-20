//! # FFI Bindings
//!
//! This module contains Foreign Function Interface (FFI) bindings for platform-specific functionality
//! not exposed by standard crates.
//!
//! Currently, it primarily handles Mach exception ports for hardware breakpoint support.

pub mod mach_exc;
