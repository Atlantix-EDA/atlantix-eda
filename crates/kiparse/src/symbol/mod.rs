//! Symbol library parsing module for KiCad .kicad_sym files
//! 
//! This module provides parsing capabilities for KiCad symbol library files,
//! extracting symbol definitions, descriptions, and metadata.
//! 
//! ## Usage Example
//! 
//! ```rust
//! use kiparse::symbol::symbol_parser::parse_symbol_lib;
//! 
//! let content = r#"
//!   (symbol "Resistor" 
//!     (property "Description" "Basic resistor component")
//!   )
//! "#;
//! let symbols = parse_symbol_lib(content)?;
//! 
//! for symbol in &symbols {
//!     println!("Symbol: {} - {}", symbol.name, symbol.description);
//! }
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

pub mod types;
pub mod symbol_parser;

// Re-export commonly used items
pub use types::*;
pub use symbol_parser::parse_symbol_lib;