//! PCB file parsing module for KiCad .kicad_pcb files
//! 
//! This module provides layer extraction from KiCad PCB files.
//! 
//! ## Usage Example
//! 
//! ### Layer Extraction
//! ```rust
//! use kiparse::pcb::parse_layers_only;
//! 
//! let content = r#"(kicad_pcb
//!   (version "20240108")
//!   (generator "pcbnew")
//!   (layers
//!     (0 "F.Cu" signal)
//!   )
//! )"#;
//! let pcb = parse_layers_only(content)?;
//! 
//! for (id, layer) in &pcb.layers {
//!     println!("Layer {}: {}", id, layer.name);
//! }
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

pub mod types;
pub mod simple_parser;
pub mod detail_parser;

#[cfg(test)]
mod tests {
    use super::*;
    
    // Test data for minimal valid KiCad PCB file
    const MINIMAL_PCB: &str = r#"(kicad_pcb
  (version "20240108")
  (generator "pcbnew")
  (layers
    (0 "F.Cu" signal)
    (31 "B.Cu" signal)
    (32 "B.Adhes" user "B.Adhesive")
  )
)"#;

    #[test]
    fn test_simple_parser_minimal_pcb() {
        let result = parse_layers_only(MINIMAL_PCB);
        assert!(result.is_ok());
        
        let pcb = result.unwrap();
        assert_eq!(pcb.version, "unknown");
        assert_eq!(pcb.generator, "simple_parser");
        assert_eq!(pcb.layers.len(), 3);
        
        // Check specific layers
        assert!(pcb.layers.contains_key(&0));
        assert!(pcb.layers.contains_key(&31));
        assert!(pcb.layers.contains_key(&32));
        
        let f_cu = pcb.layers.get(&0).unwrap();
        assert_eq!(f_cu.name, "F.Cu");
        assert_eq!(f_cu.layer_type, "signal");
        assert_eq!(f_cu.user_name, None);
        
        let b_adhes = pcb.layers.get(&32).unwrap();
        assert_eq!(b_adhes.name, "B.Adhes");
        assert_eq!(b_adhes.layer_type, "user");
        assert_eq!(b_adhes.user_name, Some("B.Adhesive".to_string()));
    }

    #[test]
    fn test_pcb_file_new() {
        let pcb = PcbFile::new();
        
        assert_eq!(pcb.version, "");
        assert_eq!(pcb.generator, "");
        assert_eq!(pcb.board_thickness, None);
        assert_eq!(pcb.paper_size, None);
        assert_eq!(pcb.layers.len(), 0);
        assert_eq!(pcb.footprints.len(), 0);
        assert_eq!(pcb.tracks.len(), 0);
        assert_eq!(pcb.vias.len(), 0);
        assert_eq!(pcb.zones.len(), 0);
        assert_eq!(pcb.texts.len(), 0);
        assert_eq!(pcb.graphics.len(), 0);
    }

    #[test]
    fn test_point_creation() {
        let point = Point { x: 10.5, y: -20.3 };
        assert_eq!(point.x, 10.5);
        assert_eq!(point.y, -20.3);
    }

    #[test]
    fn test_layer_creation() {
        let layer = Layer {
            id: 0,
            name: "F.Cu".to_string(),
            layer_type: "signal".to_string(),
            user_name: None,
        };
        
        assert_eq!(layer.id, 0);
        assert_eq!(layer.name, "F.Cu");
        assert_eq!(layer.layer_type, "signal");
        assert_eq!(layer.user_name, None);
    }
}

// Re-export commonly used items
pub use types::*;
pub use simple_parser::parse_layers_only;
pub use detail_parser::DetailParser;