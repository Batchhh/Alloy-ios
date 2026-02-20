//! # Memory Manipulation
//!
//! This module provides tools for modifying process memory at runtime.
//!
//! ## Features
//!
//! - **Hooks**: Intercept function calls using trampolines and code caves
//! - **Patches**: Modify code or data at specific addresses
//! - **Read/Write**: Safe abstractions for reading and writing process memory
//! - **Checksum**: Self-integrity verification for installed hooks

pub mod checksum;
pub mod hook;
pub mod patch;
pub mod rw;
