//! # Memory Allocation Utilities
//!
//! This module provides tools for allocating memory and executing code dynamically.
//! It includes:
//! - **Shellcode Loader**: Allocating executable pages, resolving symbols, and running raw machine code.
//! - **Position Independent Code**: Support for relocating code that depends on absolute addresses.

pub mod shellcode;
