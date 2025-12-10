//! Basic usage example for KiParse
//! 
//! This example demonstrates how to parse a KiCad PCB file and extract
//! layer information and basic file statistics.

use kiparse::prelude::*;
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
    
    // Parse layers from the PCB file
    println!("=== KiCad PCB Layer Analysis ===");
    let pcb = parse_layers_only(&content)?;
    
    println!("File: {}", filename);
    println!("Version: {}", pcb.version);
    println!("Generator: {}", pcb.generator);
    println!("Found {} layers:", pcb.layers.len());
    
    // Group layers by type
    let mut signal_layers = Vec::new();
    let mut other_layers = Vec::new();
    
    for (id, layer) in &pcb.layers {
        if layer.layer_type == "signal" {
            signal_layers.push((id, layer));
        } else {
            other_layers.push((id, layer));
        }
    }
    
    // Display signal layers
    if !signal_layers.is_empty() {
        println!("\n📡 Signal Layers ({}):", signal_layers.len());
        for (id, layer) in signal_layers {
            println!("  Layer {}: {}", id, layer.name);
        }
    }
    
    // Display other layers
    if !other_layers.is_empty() {
        println!("\n⚙️  Other Layers ({}):", other_layers.len());
        for (id, layer) in other_layers {
            let user_desc = layer.user_name.as_ref()
                .map(|s| format!(" ({})", s))
                .unwrap_or_default();
            println!("  Layer {}: {} [{}]{}", id, layer.name, layer.layer_type, user_desc);
        }
    }
    
    // Basic file statistics
    println!("\n📊 File Statistics:");
    println!("  File size: {:.2} KB", content.len() as f64 / 1024.0);
    println!("  Estimated board complexity: {}", estimate_complexity(&content));
    
    Ok(())
}

fn estimate_complexity(content: &str) -> &'static str {
    let footprint_count = content.matches("(footprint").count();
    let track_count = content.matches("(segment").count();
    
    match (footprint_count, track_count) {
        (0..=10, 0..=50) => "Simple",
        (11..=100, 51..=500) => "Moderate", 
        (101..=500, 501..=2000) => "Complex",
        _ => "Very Complex"
    }
}