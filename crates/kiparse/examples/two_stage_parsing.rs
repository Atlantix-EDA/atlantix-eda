//! Two-stage parsing example
//! 
//! This example demonstrates the two-stage parsing approach:
//! 1. First stage: Quick layer extraction with simple_parser
//! 2. Second stage: Detailed element extraction with DetailParser

use kiparse::prelude::*;
use kiparse::pcb::detail_parser::DetailParser;

fn main() -> Result<()> {
    // Use the included FPGA board example
    let content = include_str!("../assets/fpga.kicad_pcb");
    
    println!("KiParse Two-Stage Parsing Demonstration");
    println!("======================================\n");
    
    // STAGE 1: Simple layer parsing
    println!("STAGE 1: Layer Extraction (Simple Parser)");
    println!("-----------------------------------------");
    let pcb = parse_layers_only(content)?;
    println!("✓ Found {} layers", pcb.layers.len());
    
    // Show layer summary
    let signal_layers = pcb.layers.values()
        .filter(|l| l.layer_type == "signal")
        .count();
    let user_layers = pcb.layers.values()
        .filter(|l| l.layer_type == "user")
        .count();
    
    println!("  - Signal layers: {}", signal_layers);
    println!("  - User layers: {}", user_layers);
    
    // STAGE 2: Detailed parsing
    println!("\nSTAGE 2: Detail Extraction (Detail Parser)");
    println!("------------------------------------------");
    let detail_parser = DetailParser::new(content);
    
    // Extract components
    let components = detail_parser.extract_components()?;
    println!("✓ Extracted {} components", components.len());
    
    // Show component summary
    let component_summary = detail_parser.extract_component_summary()?;
    let mut sorted_summary: Vec<_> = component_summary.iter().collect();
    sorted_summary.sort_by_key(|(k, _)| k.as_str());
    
    println!("\n  Component Types:");
    for (comp_type, count) in sorted_summary {
        println!("    {}: {}", comp_type, count);
    }
    
    // Extract board outline
    if let Some(outline) = detail_parser.extract_board_outline()? {
        println!("\n✓ Board Dimensions:");
        println!("  - Size: {:.1} × {:.1} mm", outline.width_mm, outline.height_mm);
        println!("  - Size: {:.0} × {:.0} mils", 
                 outline.width_mm * 39.3701, 
                 outline.height_mm * 39.3701);
    }
    
    // Extract 3D models
    let models = detail_parser.extract_3d_models()?;
    let model_coverage = if !components.is_empty() {
        (models.len() as f64 / components.len() as f64) * 100.0
    } else {
        0.0
    };
    println!("\n✓ 3D Model Coverage: {:.1}%", model_coverage);
    
    // Extract tracks and vias
    let tracks = detail_parser.extract_tracks()?;
    let vias = detail_parser.extract_vias()?;
    println!("\n✓ Routing Elements:");
    println!("  - Tracks: {}", tracks.len());
    println!("  - Vias: {}", vias.len());
    
    // Show sample component details
    println!("\n📋 Sample Component Details (first 5):");
    for comp in components.iter().take(5) {
        println!("  {} at ({:.2}, {:.2})mm, {}°", 
                 comp.reference, 
                 comp.position.0, 
                 comp.position.1,
                 comp.rotation);
        if let Some(value) = &comp.value {
            println!("     Value: {}", value);
        }
    }
    
    println!("\n✅ Two-stage parsing complete!");
    println!("\nThis demonstrates how KiParse uses:");
    println!("1. Simple parser for fast layer extraction");
    println!("2. Detail parser for comprehensive element analysis");
    
    Ok(())
}