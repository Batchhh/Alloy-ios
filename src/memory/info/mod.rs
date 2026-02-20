//! # Memory Information and Analysis
//!
//! This module provides tools for gathering information about the process memory state.
//! It includes:
//! - **Protection**: Querying and changing memory page permissions (RWX)
//! - **Image**: Finding loaded dynamic libraries and their base addresses
//! - **Symbol**: Resolving function names to addresses (dlsym)
//! - **Scan**: Searching for byte patterns or assembly sequences
//! - **Code Cave**: Finding unused memory regions for code injection

pub mod code_cave;
pub mod image;
pub mod protection;
pub mod scan;

pub mod symbol;
