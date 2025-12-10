//! # KiParse Prelude
//! 
//! This module provides a convenient way to import the most commonly used items
//! from the KiParse crate. Import everything you need with:
//! 
//! ```rust
//! use kiparse::prelude::*;
//! ```
//! 
//! ## What's included
//! 
//! - **Error types**: `Result`, `KicadError`
//! - **Main parsing functions**: `parse_layers_only`, `parse_symbol_lib`
//! - **Core data types**: `PcbFile`, `Layer`, `Symbol`, `Point`, etc.

// Re-export error types (most commonly used)
pub use crate::error::{KicadError, Result};

// Re-export main parsing functions
pub use crate::pcb::parse_layers_only;
pub use crate::pcb::detail_parser::DetailParser;
pub use crate::symbol::symbol_parser::parse_symbol_lib;

// Re-export core PCB types
pub use crate::pcb::types::{
    PcbFile, Layer, Track, Footprint, Pad, Via, Zone, Text, Graphic,
    Point, Rect, Arc
};

// Re-export Symbol types
pub use crate::symbol::types::Symbol;