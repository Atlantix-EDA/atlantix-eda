//! Working component position extraction example
//! 
//! This example demonstrates extracting component positions using the simple parser
//! combined with targeted regex extraction for component data.

use kiparse::{pcb, Result};
use std::collections::HashMap;
use regex::Regex;

#[derive(Debug)]
struct ComponentData {
    reference: String,
    #[allow(dead_code)]
    footprint: String,
    x: f64,
    y: f64,
    rotation: f64,
    layer: String,
}

fn main() -> Result<()> {
    // Load the FPGA board PCB file
    let pcb_file = "assets/fpga.kicad_pcb";
    let content = std::fs::read_to_string(pcb_file)?;
    
    println!("Component Position Extraction Example");
    println!("=====================================");
    println!("PCB File: {}", pcb_file);
    
    // Use simple parser to get basic info
    let pcb_info = pcb::parse_layers_only(&content)?;
    println!("Layers found: {}", pcb_info.layers.len());
    
    // Extract component data using regex
    let components = extract_components_regex(&content);
    println!("Total Components: {}\n", components.len());
    
    // Group by component type
    let mut type_groups: HashMap<String, Vec<&ComponentData>> = HashMap::new();
    
    for component in &components {
        let comp_type = extract_component_type(&component.reference);
        type_groups
            .entry(comp_type.to_string())
            .or_insert_with(Vec::new)
            .push(component);
    }
    
    // Display results
    println!("COMPONENT POSITIONS BY TYPE:");
    println!("============================\n");
    
    let mut sorted_types: Vec<_> = type_groups.keys().collect();
    sorted_types.sort();
    
    for comp_type in sorted_types {
        let components = &type_groups[comp_type];
        let type_desc = get_component_description(comp_type);
        
        println!("{} - {} ({} components):", comp_type, type_desc, components.len());
        println!("┌─────────────┬──────────┬──────────┬────────┬─────────┐");
        println!("│ Reference   │ X (mm)   │ Y (mm)   │ Angle  │ Layer   │");
        println!("├─────────────┼──────────┼──────────┼────────┼─────────┤");
        
        // Sort by reference with natural sorting
        let mut sorted_comps = components.clone();
        sorted_comps.sort_by(|a, b| natural_sort(&a.reference, &b.reference));
        
        // Show first 5 of each type
        for (i, comp) in sorted_comps.iter().enumerate() {
            if i >= 5 && components.len() > 5 {
                println!("│ ... and {} more                                     │", 
                         components.len() - 5);
                break;
            }
            
            println!("│ {:11} │ {:8.3} │ {:8.3} │ {:6.1}° │ {:7} │",
                     comp.reference,
                     comp.x,
                     comp.y,
                     comp.rotation,
                     comp.layer);
        }
        println!("└─────────────┴──────────┴──────────┴────────┴─────────┘\n");
    }
    
    // Board statistics
    if !components.is_empty() {
        let min_x = components.iter().map(|c| c.x).fold(f64::INFINITY, f64::min);
        let max_x = components.iter().map(|c| c.x).fold(f64::NEG_INFINITY, f64::max);
        let min_y = components.iter().map(|c| c.y).fold(f64::INFINITY, f64::min);
        let max_y = components.iter().map(|c| c.y).fold(f64::NEG_INFINITY, f64::max);
        
        println!("BOARD STATISTICS:");
        println!("=================");
        println!("Component Bounding Box:");
        println!("  X: {:.3} to {:.3} mm (width: {:.3} mm)", min_x, max_x, max_x - min_x);
        println!("  Y: {:.3} to {:.3} mm (height: {:.3} mm)", min_y, max_y, max_y - min_y);
        
        let area = (max_x - min_x) * (max_y - min_y);
        println!("\nBoard Area: {:.2} cm²", area / 100.0);
        println!("Component Density: {:.2} components/cm²", components.len() as f64 / (area / 100.0));
    }
    
    println!("\n✅ This example demonstrates component position extraction");
    println!("   from a real FPGA board design using KiParse.");
    
    Ok(())
}

fn extract_components_regex(content: &str) -> Vec<ComponentData> {
    let mut components = Vec::new();
    
    // Match footprint blocks and extract data
    let footprint_re = Regex::new(
        r#"(?s)\(footprint\s+"([^"]+)".*?\(layer\s+"([^"]+)"\).*?\(at\s+([\d.-]+)\s+([\d.-]+)(?:\s+([\d.-]+))?\).*?\(property\s+"Reference"\s+"([^"]+)""#
    ).unwrap();
    
    for cap in footprint_re.captures_iter(content) {
        let footprint = cap.get(1).map(|m| m.as_str()).unwrap_or("").to_string();
        let layer = cap.get(2).map(|m| m.as_str()).unwrap_or("").to_string();
        let x = cap.get(3).and_then(|m| m.as_str().parse().ok()).unwrap_or(0.0);
        let y = cap.get(4).and_then(|m| m.as_str().parse().ok()).unwrap_or(0.0);
        let rotation = cap.get(5).and_then(|m| m.as_str().parse().ok()).unwrap_or(0.0);
        let reference = cap.get(6).map(|m| m.as_str()).unwrap_or("").to_string();
        
        if !reference.is_empty() {
            components.push(ComponentData {
                reference,
                footprint,
                x,
                y,
                rotation,
                layer,
            });
        }
    }
    
    components
}

fn extract_component_type(reference: &str) -> &str {
    let end = reference.find(|c: char| c.is_ascii_digit()).unwrap_or(reference.len());
    &reference[..end]
}

fn get_component_description(prefix: &str) -> &'static str {
    match prefix {
        "R" => "Resistors",
        "C" => "Capacitors",
        "L" => "Inductors",
        "D" => "Diodes",
        "Q" => "Transistors",
        "U" => "Integrated Circuits",
        "J" => "Connectors",
        "SW" => "Switches",
        "FB" => "Ferrite Beads",
        "TP" => "Test Points",
        "Y" => "Crystals",
        _ => "Other Components"
    }
}

fn natural_sort(a: &str, b: &str) -> std::cmp::Ordering {
    let a_prefix = extract_component_type(a);
    let b_prefix = extract_component_type(b);
    
    match a_prefix.cmp(b_prefix) {
        std::cmp::Ordering::Equal => {
            let a_num: u32 = a[a_prefix.len()..].parse().unwrap_or(0);
            let b_num: u32 = b[b_prefix.len()..].parse().unwrap_or(0);
            a_num.cmp(&b_num)
        }
        other => other
    }
}