//! Second stage detail parser for PCB elements
//! 
//! This module provides the second stage of the two-stage parsing process.
//! After the simple parser extracts layers, this parser extracts detailed
//! information about specific PCB elements using optimized regex patterns.

use regex::Regex;
use once_cell::sync::Lazy;
use std::collections::HashMap;
use crate::error::Result;

/// Component information extracted from footprints
#[derive(Debug, Clone)]
pub struct ComponentInfo {
    pub reference: String,
    pub footprint: String,
    pub position: (f64, f64),
    pub rotation: f64,
    pub layer: String,
    pub value: Option<String>,
    pub description: Option<String>,
}

/// 3D model information
#[derive(Debug, Clone)]
pub struct Model3DInfo {
    pub reference: String,
    pub footprint: String,
    pub model_path: String,
    pub model_type: ModelType,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ModelType {
    Wrl,
    Step,
    Iges,
    Other,
}

/// Track/trace information
#[derive(Debug, Clone)]
pub struct TrackInfo {
    pub start: (f64, f64),
    pub end: (f64, f64),
    pub width: f64,
    pub layer: String,
    pub net: Option<i32>,
}

/// Via information
#[derive(Debug, Clone)]
pub struct ViaInfo {
    pub position: (f64, f64),
    pub size: f64,
    pub drill: f64,
    pub layers: (String, String),
    pub net: Option<i32>,
}

/// Board outline from Edge.Cuts
#[derive(Debug, Clone)]
pub struct BoardOutline {
    pub min_x: f64,
    pub min_y: f64,
    pub max_x: f64,
    pub max_y: f64,
    pub width_mm: f64,
    pub height_mm: f64,
}

// Lazy static regex patterns for efficient parsing
static COMPONENT_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r#"(?s)\(footprint\s+"([^"]+)".*?\(at\s+([\d.-]+)\s+([\d.-]+)(?:\s+([\d.-]+))?\).*?\(property\s+"Reference"\s+"([^"]+)""#
    ).unwrap()
});

static COMPONENT_WITH_VALUE_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r#"(?s)\(footprint\s+"([^"]+)".*?\(at\s+([\d.-]+)\s+([\d.-]+)(?:\s+([\d.-]+))?\).*?\(property\s+"Reference"\s+"([^"]+)".*?\(property\s+"Value"\s+"([^"]+)""#
    ).unwrap()
});

// Regex to extract description from within a footprint block
static DESCRIPTION_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r#"\(property\s+"Description"\s+"([^"]+)""#
    ).unwrap()
});


static MODEL_3D_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r#"(?s)\(footprint\s+"([^"]+)".*?\(property\s+"Reference"\s+"([^"]+)".*?\(model\s+"([^"]+)""#
    ).unwrap()
});

static TRACK_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r#"\(segment\s*\(start\s+([\d.-]+)\s+([\d.-]+)\)\s*\(end\s+([\d.-]+)\s+([\d.-]+)\)\s*\(width\s+([\d.-]+)\)\s*\(layer\s+"([^"]+)"\)(?:\s*\(net\s+(\d+)\))?"#
    ).unwrap()
});

static VIA_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r#"\(via\s*\(at\s+([\d.-]+)\s+([\d.-]+)\)\s*\(size\s+([\d.-]+)\)\s*\(drill\s+([\d.-]+)\)\s*\(layers\s+"([^"]+)"\s+"([^"]+)"\)(?:\s*\(net\s+(\d+)\))?"#
    ).unwrap()
});

static EDGE_CUTS_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r#"(?s)\(gr_line\s*\(start\s+([\d.-]+)\s+([\d.-]+)\)\s*\(end\s+([\d.-]+)\s+([\d.-]+)\).*?\(layer\s+"Edge\.Cuts"\)"#
    ).unwrap()
});

/// Detail parser for extracting specific PCB elements
pub struct DetailParser<'a> {
    content: &'a str,
}

impl<'a> DetailParser<'a> {
    /// Create a new detail parser for the given PCB content
    pub fn new(content: &'a str) -> Self {
        Self { content }
    }

    /// Extract all component information
    pub fn extract_components(&self) -> Result<Vec<ComponentInfo>> {
        let mut components = Vec::new();

        // Find all footprint block start positions
        static FOOTPRINT_START: Lazy<Regex> = Lazy::new(|| {
            Regex::new(r#"\n\s*\(footprint\s+""#).unwrap()
        });

        // Regexes for extracting properties within a block
        static VALUE_REGEX: Lazy<Regex> = Lazy::new(|| {
            Regex::new(r#"\(property\s+"Value"\s+"([^"]+)""#).unwrap()
        });
        static FOOTPRINT_INFO_REGEX: Lazy<Regex> = Lazy::new(|| {
            Regex::new(r#"(?s)\(footprint\s+"([^"]+)".*?\(at\s+([\d.-]+)\s+([\d.-]+)(?:\s+([\d.-]+))?\).*?\(property\s+"Reference"\s+"([^"]+)""#).unwrap()
        });

        // Find footprint block boundaries
        let starts: Vec<usize> = FOOTPRINT_START.find_iter(self.content)
            .map(|m| m.start())
            .collect();

        for (i, &start) in starts.iter().enumerate() {
            // Block ends at next footprint or end of content
            let end = starts.get(i + 1).copied().unwrap_or(self.content.len());
            let block = &self.content[start..end];

            // Extract basic info from block
            if let Some(cap) = FOOTPRINT_INFO_REGEX.captures(block) {
                let footprint = cap[1].to_string();
                let x: f64 = cap[2].parse().unwrap_or(0.0);
                let y: f64 = cap[3].parse().unwrap_or(0.0);
                let rotation: f64 = cap.get(4).map_or(0.0, |m| m.as_str().parse().unwrap_or(0.0));
                let reference = cap[5].to_string();

                // Extract Value property from this block
                let value = VALUE_REGEX.captures(block).map(|c| c[1].to_string());

                // Extract Description property from this block
                let description = DESCRIPTION_REGEX.captures(block).map(|c| c[1].to_string());

                // Determine layer from footprint context
                let layer = self.extract_component_layer(&footprint, x, y)
                    .unwrap_or_else(|| "F.Cu".to_string());

                components.push(ComponentInfo {
                    reference,
                    footprint,
                    position: (x, y),
                    rotation,
                    layer,
                    value,
                    description,
                });
            }
        }

        // Fallback: Also capture components using simpler regex if block method missed any
        for cap in COMPONENT_REGEX.captures_iter(self.content) {
            let reference = &cap[5];
            // Skip if we already have this component
            if components.iter().any(|c| c.reference == reference) {
                continue;
            }

            let footprint = cap[1].to_string();
            let x: f64 = cap[2].parse().unwrap_or(0.0);
            let y: f64 = cap[3].parse().unwrap_or(0.0);
            let rotation: f64 = cap.get(4).map_or(0.0, |m| m.as_str().parse().unwrap_or(0.0));
            let layer = self.extract_component_layer(&footprint, x, y)
                .unwrap_or_else(|| "F.Cu".to_string());

            components.push(ComponentInfo {
                reference: reference.to_string(),
                footprint,
                position: (x, y),
                rotation,
                layer,
                value: None,
                description: None,
            });
        }

        Ok(components)
    }

    /// Extract 3D model information
    pub fn extract_3d_models(&self) -> Result<Vec<Model3DInfo>> {
        let mut models = Vec::new();
        
        for cap in MODEL_3D_REGEX.captures_iter(self.content) {
            let footprint = cap[1].to_string();
            let reference = cap[2].to_string();
            let model_path = cap[3].to_string();
            
            let model_type = if model_path.ends_with(".wrl") {
                ModelType::Wrl
            } else if model_path.ends_with(".step") || model_path.ends_with(".stp") {
                ModelType::Step
            } else if model_path.ends_with(".igs") || model_path.ends_with(".iges") {
                ModelType::Iges
            } else {
                ModelType::Other
            };
            
            models.push(Model3DInfo {
                reference,
                footprint,
                model_path,
                model_type,
            });
        }
        
        Ok(models)
    }

    /// Extract track/trace information
    pub fn extract_tracks(&self) -> Result<Vec<TrackInfo>> {
        let mut tracks = Vec::new();
        
        for cap in TRACK_REGEX.captures_iter(self.content) {
            let start_x: f64 = cap[1].parse().unwrap_or(0.0);
            let start_y: f64 = cap[2].parse().unwrap_or(0.0);
            let end_x: f64 = cap[3].parse().unwrap_or(0.0);
            let end_y: f64 = cap[4].parse().unwrap_or(0.0);
            let width: f64 = cap[5].parse().unwrap_or(0.0);
            let layer = cap[6].to_string();
            let net = cap.get(7).and_then(|m| m.as_str().parse().ok());
            
            tracks.push(TrackInfo {
                start: (start_x, start_y),
                end: (end_x, end_y),
                width,
                layer,
                net,
            });
        }
        
        Ok(tracks)
    }

    /// Extract via information
    pub fn extract_vias(&self) -> Result<Vec<ViaInfo>> {
        let mut vias = Vec::new();
        
        for cap in VIA_REGEX.captures_iter(self.content) {
            let x: f64 = cap[1].parse().unwrap_or(0.0);
            let y: f64 = cap[2].parse().unwrap_or(0.0);
            let size: f64 = cap[3].parse().unwrap_or(0.0);
            let drill: f64 = cap[4].parse().unwrap_or(0.0);
            let layer1 = cap[5].to_string();
            let layer2 = cap[6].to_string();
            let net = cap.get(7).and_then(|m| m.as_str().parse().ok());
            
            vias.push(ViaInfo {
                position: (x, y),
                size,
                drill,
                layers: (layer1, layer2),
                net,
            });
        }
        
        Ok(vias)
    }

    /// Extract board outline from Edge.Cuts layer
    pub fn extract_board_outline(&self) -> Result<Option<BoardOutline>> {
        let mut min_x = f64::MAX;
        let mut min_y = f64::MAX;
        let mut max_x = f64::MIN;
        let mut max_y = f64::MIN;
        let mut found_edge = false;
        
        for cap in EDGE_CUTS_REGEX.captures_iter(self.content) {
            found_edge = true;
            let x1: f64 = cap[1].parse().unwrap_or(0.0);
            let y1: f64 = cap[2].parse().unwrap_or(0.0);
            let x2: f64 = cap[3].parse().unwrap_or(0.0);
            let y2: f64 = cap[4].parse().unwrap_or(0.0);
            
            min_x = min_x.min(x1).min(x2);
            min_y = min_y.min(y1).min(y2);
            max_x = max_x.max(x1).max(x2);
            max_y = max_y.max(y1).max(y2);
        }
        
        if !found_edge {
            return Ok(None);
        }
        
        let width_mm = max_x - min_x;
        let height_mm = max_y - min_y;
        
        Ok(Some(BoardOutline {
            min_x,
            min_y,
            max_x,
            max_y,
            width_mm,
            height_mm,
        }))
    }

    /// Extract component counts by type
    pub fn extract_component_summary(&self) -> Result<HashMap<String, usize>> {
        let components = self.extract_components()?;
        let mut summary = HashMap::new();
        
        for comp in components {
            let prefix = extract_component_prefix(&comp.reference);
            *summary.entry(prefix).or_insert(0) += 1;
        }
        
        Ok(summary)
    }

    /// Helper to extract component layer (simplified - could be enhanced)
    fn extract_component_layer(&self, _footprint: &str, _x: f64, _y: f64) -> Option<String> {
        // This is a simplified version - in reality, you'd need to look at the
        // footprint definition to determine if it's on F.Cu or B.Cu
        // For now, return None to use default
        None
    }
}

/// Extract component reference prefix (R, C, U, etc.)
fn extract_component_prefix(reference: &str) -> String {
    reference.chars()
        .take_while(|c| c.is_alphabetic())
        .collect()
}

/// Convert millimeters to mils
pub fn mm_to_mils(mm: f64) -> f64 {
    mm * 39.3701
}

/// Convert square millimeters to square inches
pub fn mm2_to_sq_in(mm2: f64) -> f64 {
    mm2 / 645.16
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_component_extraction() {
        let content = r#"
        (footprint "Resistor_SMD:R_0603_1608Metric"
            (at 100.5 50.25 90)
            (property "Reference" "R1"
                (at 0 0 90)
            )
            (property "Value" "10k"
                (at 0 0 90)
            )
            (property "Description" "Thick Film Resistor 10k 1% 0603"
                (at 0 0 90)
            )
        )
        "#;

        let parser = DetailParser::new(content);
        let components = parser.extract_components().unwrap();

        assert_eq!(components.len(), 1);
        assert_eq!(components[0].reference, "R1");
        assert_eq!(components[0].position, (100.5, 50.25));
        assert_eq!(components[0].rotation, 90.0);
        assert_eq!(components[0].value, Some("10k".to_string()));
        assert_eq!(components[0].description, Some("Thick Film Resistor 10k 1% 0603".to_string()));
    }

    #[test]
    fn test_3d_model_extraction() {
        let content = r#"
        (footprint "Capacitor_SMD:C_0805_2012Metric"
            (property "Reference" "C1")
            (model "${KICAD8_3DMODEL_DIR}/Capacitor_SMD.3dshapes/C_0805_2012Metric.wrl"
                (offset (xyz 0 0 0))
            )
        )
        "#;
        
        let parser = DetailParser::new(content);
        let models = parser.extract_3d_models().unwrap();
        
        assert_eq!(models.len(), 1);
        assert_eq!(models[0].reference, "C1");
        assert_eq!(models[0].model_type, ModelType::Wrl);
    }

    #[test]
    fn test_board_outline() {
        let content = r#"
        (gr_line (start 0 0) (end 100 0) (layer "Edge.Cuts"))
        (gr_line (start 100 0) (end 100 50) (layer "Edge.Cuts"))
        (gr_line (start 100 50) (end 0 50) (layer "Edge.Cuts"))
        (gr_line (start 0 50) (end 0 0) (layer "Edge.Cuts"))
        "#;
        
        let parser = DetailParser::new(content);
        let outline = parser.extract_board_outline().unwrap().unwrap();
        
        assert_eq!(outline.width_mm, 100.0);
        assert_eq!(outline.height_mm, 50.0);
    }
}