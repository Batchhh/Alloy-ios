//! # Menu System
//!
//! This module handles the core menu logic, including:
//! - Item registry and page management
//! - Interaction handling (taps, gestures)
//! - Menu rendering and UI updates
pub mod handler;
pub mod items;
pub mod registry;
pub mod utils;
pub mod view;

pub use handler::*;
pub use items::*;
pub use registry::*;
pub use utils::*;
pub use view::*;
