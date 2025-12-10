//! # KiParse
//! 
//! A KiCad file format parser for PCB and symbol files.
//! 
//! KiParse provides parsing capabilities for KiCad's native file formats through
//! a pragmatic approach focusing on what actually works with real-world files.
//! 
//! ## Features
//! 
//! - **Layer Extraction**: Fast and reliable layer information parsing from PCB files
//! - **Symbol Parsing**: Complete symbol library parsing with metadata
//! - **Robust Error Handling**: Detailed error messages with context
//! - **Memory Efficient**: Optimized for large files
//! - **Well Tested**: Works with real-world KiCad files
//! 
//! ## Quick Start
//! 
//! ```rust
//! use kiparse::prelude::*;
//! 
//! let pcb_content = r#"(kicad_pcb
//!   (version "20240108")
//!   (generator "pcbnew")
//!   (layers
//!     (0 "F.Cu" signal)
//!     (31 "B.Cu" signal)
//!   )
//! )"#;
//! 
//! let pcb = parse_layers_only(pcb_content)?;
//! 
//! println!("Found {} layers", pcb.layers.len());
//! for (id, layer) in &pcb.layers {
//!     println!("Layer {}: {} ({})", id, layer.name, layer.layer_type);
//! }
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//! 
//! ## Module Organization
//! 
//! - [`pcb`] - PCB file layer extraction (.kicad_pcb)
//! - [`symbol`] - Symbol library parsing (.kicad_sym) 
//! - [`error`] - Error types and handling
//! 
//! ## Performance Characteristics
//! 
//! KiParse is designed for practical use with real PCB files:
//! 
//! - **Layer extraction**: ~10MB/s
//! - **Symbol parsing**: ~15MB/s
//! - **Memory usage**: ~1.5x file size during parsing

pub mod pcb;
pub mod symbol;
pub mod error;
pub mod prelude;

// Re-export commonly used types at the crate root
pub use error::{KicadError, Result};

// Re-export the main parsing functions for convenience
pub use pcb::parse_layers_only;
pub use pcb::detail_parser::DetailParser;
pub use symbol::symbol_parser::parse_symbol_lib;

// Re-export PCB data types with module prefix to avoid conflicts
pub use pcb::types::{
    PcbFile, Layer, Track, Footprint, Pad, Via, Zone, Text, Graphic,
    Point, Rect, Arc
};

// Re-export Symbol types with explicit naming to avoid conflicts
pub use symbol::types::Symbol;

/// Library version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Returns version information for the library
pub fn version() -> &'static str {
    VERSION
}

#[cfg(test)]
mod integration_tests {
    use super::*;
    
    #[test]
    fn test_version() {
        assert!(!version().is_empty());
    }
}