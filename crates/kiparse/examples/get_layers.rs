//! Layer extraction example for CAM workflows
//! 
//! This example demonstrates how to quickly extract layer information
//! from KiCad PCB files for CAM (Computer-Aided Manufacturing) workflows.

use kiparse::{pcb, Result};
use std::env;

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    
    let filename: &str;
    // Check if a filename was provided as a command line argument
    // If not, use a default file for demonstration purposes
    // This is useful for quick testing without needing to specify a file every time.
    
    if args.len() < 2 {
        eprintln!("Usage: {} <file.kicad_pcb>", args[0]);
        eprintln!("Example: {} assets/fpga.kicad_pcb", args[0]);
        filename = "assets/fpga.kicad_pcb";
        eprintln!("Using default file: {}", filename);
    } else {
        eprintln!("Parsing file: {}", args[1]);
        filename = &args[1];
    }
    

    let content = std::fs::read_to_string(filename)?;
    
    // Use the fast simple parser for layer extraction
    let pcb = pcb::parse_layers_only(&content)?;
    
    println!("KiCad PCB Layer Information");
    println!("===========================");
    println!("File: {}", filename);
    println!("Version: {}", pcb.version);
    println!("Generator: {}", pcb.generator);
    println!("Total layers: {}\n", pcb.layers.len());
    
    // Categorize layers for CAM workflow
    let mut copper_layers = Vec::new();
    let mut technical_layers = Vec::new();
    let mut user_layers = Vec::new();
    
    for (id, layer) in &pcb.layers {
        match layer.layer_type.as_str() {
            "signal" => copper_layers.push((id, layer)),
            "user" => {
                if layer.name.contains("Cu") {
                    copper_layers.push((id, layer));
                } else {
                    technical_layers.push((id, layer));
                }
            }
            _ => user_layers.push((id, layer)),
        }
    }
    
    // Sort layers by ID
    copper_layers.sort_by_key(|(id, _)| **id);
    technical_layers.sort_by_key(|(id, _)| **id);
    user_layers.sort_by_key(|(id, _)| **id);
    
    // Display copper layers (critical for manufacturing)
    if !copper_layers.is_empty() {
        println!("🔶 COPPER LAYERS ({}):", copper_layers.len());
        for (id, layer) in copper_layers {
            let user_desc = layer.user_name.as_deref().unwrap_or("");
            println!("  Layer {:2}: {:12} | {}", id, layer.name, user_desc);
        }
        println!();
    }
    
    // Display technical layers (solder mask, paste, etc.)
    if !technical_layers.is_empty() {
        println!("⚙️  TECHNICAL LAYERS ({}):", technical_layers.len());
        for (id, layer) in technical_layers {
            let user_desc = layer.user_name.as_deref().unwrap_or("");
            println!("  Layer {:2}: {:12} | {}", id, layer.name, user_desc);
        }
        println!();
    }
    
    // Display user layers
    if !user_layers.is_empty() {
        println!("📋 USER LAYERS ({}):", user_layers.len());
        for (id, layer) in user_layers {
            let user_desc = layer.user_name.as_deref().unwrap_or("");
            println!("  Layer {:2}: {:12} | {}", id, layer.name, user_desc);
        }
        println!();
    }
    
    // Generate CAM-friendly layer mapping
    println!("CAM LAYER MAPPING:");
    println!("==================");
    
    // Standard PCB stackup detection
    let has_f_cu = pcb.layers.values().any(|l| l.name == "F.Cu");
    let has_b_cu = pcb.layers.values().any(|l| l.name == "B.Cu");
    let inner_layers: Vec<_> = pcb.layers.values()
        .filter(|l| l.name.starts_with("In") && l.name.ends_with(".Cu"))
        .collect();
    
    println!("PCB Stack-up:");
    if has_f_cu {
        println!("  - Top Copper (F.Cu)");
    }
    for layer in &inner_layers {
        println!("  - {} (Inner layer)", layer.name);
    }
    if has_b_cu {
        println!("  - Bottom Copper (B.Cu)");
    }
    
    let layer_count = (if has_f_cu { 1 } else { 0 }) + 
                     inner_layers.len() + 
                     (if has_b_cu { 1 } else { 0 });
    
    println!("\nTotal copper layers: {}", layer_count);
    
    // Check for common manufacturing layers
    println!("\nManufacturing layers present:");
    let required_layers = [
        ("F.Mask", "Front Solder Mask"),
        ("B.Mask", "Back Solder Mask"), 
        ("F.Paste", "Front Solder Paste"),
        ("B.Paste", "Back Solder Paste"),
        ("F.SilkS", "Front Silkscreen"),
        ("B.SilkS", "Back Silkscreen"),
        ("Edge.Cuts", "Board Outline"),
    ];
    
    for (layer_name, description) in required_layers {
        let present = pcb.layers.values().any(|l| l.name == layer_name);
        let status = if present { "✓" } else { "✗" };
        println!("  {} {}: {}", status, layer_name, description);
    }
    
    Ok(())
}