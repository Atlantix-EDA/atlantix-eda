//! 3D Model extraction example
//! 
//! This example demonstrates how to extract 3D model information from KiCad PCB files.
//! It identifies which components have 3D models (STEP, STP, or WRL files) attached.

use kiparse::prelude::*;
use regex::Regex;
use std::collections::{HashMap, HashSet};

fn main() -> Result<()> {
    // Use the included FPGA board example
    let content = include_str!("../assets/fpga.kicad_pcb");
    
    // Parse basic structure
    let pcb = parse_layers_only(content)?;
    
    println!("KiCad PCB 3D Model Analysis");
    println!("===========================");
    println!("Found {} layers", pcb.layers.len());
    println!();
    
    // Extract footprints with their 3D model information
    let footprint_re = Regex::new(
        r#"(?s)\(footprint\s+"([^"]+)".*?\(property\s+"Reference"\s+"([^"]+)".*?\(model\s+"([^"]+)""#
    ).unwrap();
    
    let mut components_with_models = Vec::new();
    let mut components_without_models = Vec::new();
    let mut model_types: HashMap<String, usize> = HashMap::new();
    let mut model_libraries: HashMap<String, usize> = HashMap::new();
    
    // Find components with 3D models
    for cap in footprint_re.captures_iter(content) {
        let footprint = &cap[1];
        let reference = &cap[2];
        let model_path = &cap[3];
        
        components_with_models.push((reference.to_string(), footprint.to_string(), model_path.to_string()));
        
        // Analyze model type
        let model_type = if model_path.ends_with(".wrl") {
            "WRL (VRML)"
        } else if model_path.ends_with(".step") || model_path.ends_with(".stp") {
            "STEP"
        } else if model_path.ends_with(".igs") || model_path.ends_with(".iges") {
            "IGES"
        } else {
            "Other"
        };
        *model_types.entry(model_type.to_string()).or_insert(0) += 1;
        
        // Extract library name
        if let Some(lib_start) = model_path.find("3dshapes/") {
            if let Some(lib_end) = model_path[..lib_start].rfind('/') {
                let lib_name = &model_path[lib_end + 1..lib_start - 1];
                *model_libraries.entry(lib_name.to_string()).or_insert(0) += 1;
            }
        }
    }
    
    // Find all unique references
    let all_refs_re = Regex::new(
        r#"(?s)\(footprint\s+"[^"]+?".*?\(property\s+"Reference"\s+"([^"]+)""#
    ).unwrap();
    
    let mut all_refs = HashSet::new();
    for cap in all_refs_re.captures_iter(content) {
        all_refs.insert(cap[1].to_string());
    }
    
    // Find components without models
    let refs_with_models: HashSet<_> = components_with_models.iter()
        .map(|(r, _, _)| r.clone())
        .collect();
    
    for reference in &all_refs {
        if !refs_with_models.contains(reference) {
            components_without_models.push(reference.clone());
        }
    }
    
    // Sort for consistent output
    components_with_models.sort_by(|a, b| natural_sort(&a.0, &b.0));
    components_without_models.sort();
    
    // Display summary
    let total_components = all_refs.len();
    let with_models = components_with_models.len();
    let without_models = components_without_models.len();
    let coverage_percent = if total_components > 0 {
        (with_models as f64 / total_components as f64) * 100.0
    } else {
        0.0
    };
    
    println!("📊 3D MODEL COVERAGE:");
    println!("  Total components: {}", total_components);
    println!("  With 3D models: {} ({:.1}%)", with_models, coverage_percent);
    println!("  Without 3D models: {} ({:.1}%)", without_models, 100.0 - coverage_percent);
    println!();
    
    // Model type distribution
    if !model_types.is_empty() {
        println!("📁 MODEL TYPES:");
        for (model_type, count) in &model_types {
            println!("  {}: {}", model_type, count);
        }
        println!();
    }
    
    // Library usage
    if !model_libraries.is_empty() {
        println!("📚 3D LIBRARIES USED:");
        let mut sorted_libs: Vec<_> = model_libraries.iter().collect();
        sorted_libs.sort_by(|a, b| b.1.cmp(a.1));
        
        for (lib, count) in sorted_libs.iter().take(10) {
            println!("  {}: {} models", lib, count);
        }
        if sorted_libs.len() > 10 {
            println!("  ... and {} more libraries", sorted_libs.len() - 10);
        }
        println!();
    }
    
    // Group components by type
    let mut component_groups: HashMap<String, Vec<(String, bool)>> = HashMap::new();
    
    for (reference, _, _) in &components_with_models {
        let prefix = extract_component_prefix(reference);
        component_groups.entry(prefix).or_insert_with(Vec::new).push((reference.clone(), true));
    }
    
    for reference in &components_without_models {
        let prefix = extract_component_prefix(reference);
        component_groups.entry(prefix).or_insert_with(Vec::new).push((reference.clone(), false));
    }
    
    // Display component groups
    println!("🔧 COMPONENT TYPES:");
    let mut sorted_groups: Vec<_> = component_groups.iter().collect();
    sorted_groups.sort_by(|a, b| a.0.cmp(b.0));
    
    for (prefix, components) in sorted_groups {
        let total = components.len();
        let with_3d = components.iter().filter(|(_, has_3d)| *has_3d).count();
        let coverage = (with_3d as f64 / total as f64) * 100.0;
        
        println!("  {} - Total: {}, With 3D: {} ({:.0}%)", 
                 get_component_description(prefix), total, with_3d, coverage);
    }
    println!();
    
    // List components without models
    if without_models > 0 {
        println!("⚠️  COMPONENTS WITHOUT 3D MODELS ({}):", without_models);
        for reference in &components_without_models {
            println!("  - {}", reference);
        }
    }
    
    // Example: Show a few components with their 3D models
    if !components_with_models.is_empty() {
        println!("\n📋 SAMPLE COMPONENTS WITH 3D MODELS:");
        for (reference, footprint, model) in components_with_models.iter().take(5) {
            let model_file = model.split('/').last().unwrap_or(model);
            println!("  {} ({}): {}", reference, footprint, model_file);
        }
        if components_with_models.len() > 5 {
            println!("  ... and {} more", components_with_models.len() - 5);
        }
    }
    
    Ok(())
}

fn extract_component_prefix(reference: &str) -> String {
    let mut prefix = String::new();
    for ch in reference.chars() {
        if ch.is_alphabetic() {
            prefix.push(ch);
        } else {
            break;
        }
    }
    prefix
}

fn get_component_description(prefix: &str) -> String {
    match prefix {
        "R" => "Resistors",
        "C" => "Capacitors",
        "L" => "Inductors",
        "D" => "Diodes/LEDs",
        "Q" => "Transistors",
        "U" => "ICs",
        "J" => "Connectors",
        "P" => "Connectors",
        "SW" => "Switches",
        "Y" => "Crystals",
        "T" => "Transformers",
        "F" => "Fuses",
        "BT" => "Batteries",
        "JP" => "Jumpers",
        "TP" => "Test Points",
        "M" => "Motors",
        "K" => "Relays",
        _ => prefix,
    }.to_string()
}

fn natural_sort(a: &str, b: &str) -> std::cmp::Ordering {
    // Extract prefix and number for natural sorting
    let (a_prefix, a_num) = split_reference(a);
    let (b_prefix, b_num) = split_reference(b);
    
    match a_prefix.cmp(&b_prefix) {
        std::cmp::Ordering::Equal => a_num.cmp(&b_num),
        other => other,
    }
}

fn split_reference(reference: &str) -> (&str, u32) {
    let mut split_pos = 0;
    for (i, ch) in reference.chars().enumerate() {
        if ch.is_numeric() {
            split_pos = i;
            break;
        }
    }
    
    if split_pos > 0 {
        let (prefix, num_str) = reference.split_at(split_pos);
        let num = num_str.parse().unwrap_or(0);
        (prefix, num)
    } else {
        (reference, 0)
    }
}