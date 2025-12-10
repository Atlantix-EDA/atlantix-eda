//! Simple component position extraction example
//! 
//! This example demonstrates basic position extraction using regex patterns
//! to work around current parser limitations with complex PCB files.

use kiparse::Result;
use std::collections::HashMap;
use regex::Regex;

#[derive(Debug, Clone)]
struct ComponentInfo {
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
    
    println!("Simple Component Position Extractor");
    println!("===================================");
    println!("PCB File: {}", pcb_file);
    
    // Extract components using regex patterns
    let components = extract_components(&content);
    println!("Total Components Found: {}\n", components.len());
    
    // Group components by type
    let mut component_groups: HashMap<String, Vec<&ComponentInfo>> = HashMap::new();
    
    for component in &components {
        let prefix = extract_prefix(&component.reference);
        component_groups
            .entry(prefix.to_string())
            .or_insert_with(Vec::new)
            .push(component);
    }
    
    // Display components by type
    println!("COMPONENT POSITIONS BY TYPE:");
    println!("============================\n");
    
    let mut sorted_types: Vec<_> = component_groups.keys().collect();
    sorted_types.sort();
    
    for component_type in sorted_types {
        let components = &component_groups[component_type];
        let type_desc = get_component_description(component_type);
        
        println!("{} - {} ({} components):", component_type, type_desc, components.len());
        println!("Reference   X(mm)     Y(mm)     Rotation  Layer");
        println!("----------- --------- --------- --------- -------");
        
        // Sort by reference
        let mut sorted = components.clone();
        sorted.sort_by(|a, b| natural_sort(&a.reference, &b.reference));
        
        for comp in sorted.iter().take(10) {  // Show first 10 of each type
            println!("{:11} {:9.3} {:9.3} {:9.1} {}",
                     comp.reference,
                     comp.x,
                     comp.y,
                     comp.rotation,
                     comp.layer);
        }
        
        if components.len() > 10 {
            println!("... and {} more", components.len() - 10);
        }
        println!();
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
        println!("  X: {:.3} to {:.3} mm (width: {:.3} mm)", 
                 min_x, max_x, max_x - min_x);
        println!("  Y: {:.3} to {:.3} mm (height: {:.3} mm)", 
                 min_y, max_y, max_y - min_y);
        
        let area = (max_x - min_x) * (max_y - min_y);
        let density = components.len() as f64 / (area / 100.0);
        println!("\nBoard Area: {:.2} cm²", area / 100.0);
        println!("Component Density: {:.2} components/cm²", density);
    }
    
    Ok(())
}

fn extract_components(content: &str) -> Vec<ComponentInfo> {
    let mut components = Vec::new();
    
    // Regex to find footprint blocks
    let footprint_re = Regex::new(r#"(?s)\(footprint\s+"([^"]+)".*?\(layer\s+"([^"]+)"\).*?\(at\s+([\d.-]+)\s+([\d.-]+)(?:\s+([\d.-]+))?\)"#).unwrap();
    
    // Regex to find reference property within a footprint
    let reference_re = Regex::new(r#"(?s)\(property\s+"Reference"\s+"([^"]+)""#).unwrap();
    
    for footprint_match in footprint_re.captures_iter(content) {
        let footprint_name = footprint_match.get(1).unwrap().as_str();
        let layer = footprint_match.get(2).unwrap().as_str();
        let x: f64 = footprint_match.get(3).unwrap().as_str().parse().unwrap_or(0.0);
        let y: f64 = footprint_match.get(4).unwrap().as_str().parse().unwrap_or(0.0);
        let rotation: f64 = footprint_match.get(5)
            .map(|m| m.as_str().parse().unwrap_or(0.0))
            .unwrap_or(0.0);
        
        // Try to find reference in the footprint block
        let footprint_text = footprint_match.get(0).unwrap().as_str();
        let reference = if let Some(ref_match) = reference_re.captures(footprint_text) {
            ref_match.get(1).unwrap().as_str().to_string()
        } else {
            "Unknown".to_string()
        };
        
        components.push(ComponentInfo {
            reference,
            footprint: footprint_name.to_string(),
            x,
            y,
            rotation,
            layer: layer.to_string(),
        });
    }
    
    components
}

fn extract_prefix(reference: &str) -> &str {
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
        _ => "Other Components"
    }
}

fn natural_sort(a: &str, b: &str) -> std::cmp::Ordering {
    let a_prefix = extract_prefix(a);
    let b_prefix = extract_prefix(b);
    
    match a_prefix.cmp(b_prefix) {
        std::cmp::Ordering::Equal => {
            let a_num: u32 = a[a_prefix.len()..].parse().unwrap_or(0);
            let b_num: u32 = b[b_prefix.len()..].parse().unwrap_or(0);
            a_num.cmp(&b_num)
        }
        other => other
    }
}