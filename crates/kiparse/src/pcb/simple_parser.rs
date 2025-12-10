//! Simple parser for KiCad PCB files that extracts only layer information.
//! 
//! This parser is designed to quickly extract layer definitions from a KiCad PCB file
//! without parsing the entire file structure. It is often used as the first stage parser 
//! in combination with a more complex second stage parser that handles detailed parsing
//! of what is on each layer.
//! 
//! An example of a layer line in a KiCad PCB file is:
//! ```kicad
//! (layers
//!     (0 "F.Cu" "Copper" "Front Copper Layer")
//!     (1 "B.Cu" "Copper" "Back Copper Layer")
//!     ....
//! ``` 

//! and so on. The layer lines typically start with an ID, followed by the layer name, type, and optionally a user-defined name.
//! 
use super::types::*;
use crate::error::Result;

/// Parse Layers
/// 
/// This function reads a KiCad PCB file content and extracts only the layer definitions.
/// The result type is a `PcbFile` containing the layers found in the file. The PcbFile type is
/// ```rust
///  use std::collections::HashMap;
///  use serde::{Serialize, Deserialize};
///  use kiparse::pcb::types::{Layer, Footprint, Track, Via, Zone, Text, Graphic};
/// #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
///  pub struct PcbFile {
///    pub version         : String,
///    pub generator       : String,
///    pub board_thickness : Option<f64>,
///    pub paper_size      : Option<String>,
///    pub layers          : HashMap<i32, Layer>, // seeking this information with this parse_layers_only function
///    pub footprints      : Vec<Footprint>,
///    pub tracks          : Vec<Track>,
///    pub vias            : Vec<Via>,
///    pub zones           : Vec<Zone>,
///    pub texts           : Vec<Text>,
///    pub graphics        : Vec<Graphic>,
///  }
/// ```
pub fn parse_layers_only(content: &str) -> Result<PcbFile> {
    let mut pcb = PcbFile::new();
    pcb.version = "unknown".to_string();
    pcb.generator = "simple_parser".to_string();
    
    if let Some(layers_start) = content.find("(layers") {
        let layers_section = &content[layers_start..];
        
        let lines: Vec<&str> = layers_section.lines().collect();
        
        for line in lines {
            let line = line.trim();
            if line.starts_with('(') && line.contains('"') && !line.starts_with("(layers") {
                // Try to parse layer line
                if let Some(layer) = parse_layer_line(line) {
                    pcb.layers.insert(layer.id, layer);
                }
            } else if line.starts_with(')') && pcb.layers.len() > 0 {
                break;
            }
        }
    }
    eprintln!("Simple parser found {} layers", pcb.layers.len());
    Ok(pcb)
}

/// Parse Layer Line
/// 
/// This function parses a single layer line from the KiCad PCB file.
/// It expects the line to be in the format: (ID "Name" "Type" [User Name])
/// Returns an Option<Layer> if the line is valid, or None if it cannot be parsed.
fn parse_layer_line(line: &str) -> Option<Layer> {

    let line = line.trim_start_matches('(').trim_end_matches(')');
    let parts: Vec<&str> = line.split_whitespace().collect();
    
    if parts.len() >= 3 {
        if let Ok(id) = parts[0].parse::<i32>() {
            let name = parts[1].trim_matches('"').to_string();
            
            let layer_type = parts[2].trim_matches('"').to_string();
            
            let user_name = if parts.len() > 3 {
                Some(parts[3..].join(" ").trim_matches('"').to_string())
            } else {
                None
            };
            
            return Some(Layer {
                id,
                name,
                layer_type,
                user_name,
            });
        }
    }
    
    None
}