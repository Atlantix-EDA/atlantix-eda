//! Symbol library parsing example
//! 
//! This example demonstrates how to parse KiCad symbol library files
//! and extract symbol information and descriptions.

use kiparse::{symbol, Result};
use std::env;

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        eprintln!("Usage: {} <file.kicad_sym>", args[0]);
        eprintln!("Example: {} assets/example_components.kicad_sym", args[0]);
        std::process::exit(1);
    }
    
    let filename = &args[1];
    let content = std::fs::read_to_string(filename)?;
    
    // Parse the symbol library
    let symbols = symbol::parse_symbol_lib(&content)?;
    
    println!("KiCad Symbol Library Analysis");
    println!("=============================");
    println!("File: {}", filename);
    println!("Total symbols: {}\n", symbols.len());
    
    if symbols.is_empty() {
        println!("No symbols found in the library.");
        return Ok(());
    }
    
    // Categorize symbols by type
    let mut categories = std::collections::HashMap::new();
    
    for symbol in &symbols {
        let category = categorize_symbol(&symbol.name);
        categories.entry(category).or_insert_with(Vec::new).push(symbol);
    }
    
    // Display symbols by category
    for (category, symbol_list) in categories {
        println!("📦 {} ({} symbols):", category.to_uppercase(), symbol_list.len());
        
        for symbol in symbol_list {
            let description = if symbol.description.is_empty() {
                "No description".to_string()
            } else {
                symbol.description.clone()
            };
            
            println!("  • {:20} | {}", symbol.name, description);
        }
        println!();
    }
    
    // Statistics
    println!("STATISTICS:");
    println!("===========");
    
    let symbols_with_desc = symbols.iter()
        .filter(|s| !s.description.is_empty())
        .count();
    
    println!("Symbols with descriptions: {} / {} ({:.1}%)", 
             symbols_with_desc, 
             symbols.len(),
             (symbols_with_desc as f64 / symbols.len() as f64) * 100.0);
    
    // Find the longest description
    if let Some(longest) = symbols.iter()
        .max_by_key(|s| s.description.len()) {
        if !longest.description.is_empty() {
            println!("Longest description: {} ({} chars)", 
                     longest.name, 
                     longest.description.len());
        }
    }
    
    // Look for common component types
    println!("\nComponent type distribution:");
    let mut type_counts = std::collections::HashMap::new();
    
    for symbol in &symbols {
        let component_type = extract_component_type(&symbol.name);
        *type_counts.entry(component_type).or_insert(0) += 1;
    }
    
    let mut sorted_types: Vec<_> = type_counts.iter().collect();
    sorted_types.sort_by(|a, b| b.1.cmp(a.1)); // Sort by count descending
    
    for (component_type, count) in sorted_types.iter().take(10) {
        println!("  {:15}: {}", component_type, count);
    }
    
    Ok(())
}

fn categorize_symbol(name: &str) -> &'static str {
    let name_lower = name.to_lowercase();
    
    if name_lower.contains("resistor") || name_lower.starts_with("r_") {
        "resistors"
    } else if name_lower.contains("capacitor") || name_lower.starts_with("c_") {
        "capacitors"
    } else if name_lower.contains("inductor") || name_lower.starts_with("l_") {
        "inductors"
    } else if name_lower.contains("diode") || name_lower.starts_with("d_") {
        "diodes"
    } else if name_lower.contains("transistor") || name_lower.contains("mosfet") || 
              name_lower.starts_with("q_") || name_lower.starts_with("m_") {
        "transistors"
    } else if name_lower.contains("ic") || name_lower.contains("mcu") || 
              name_lower.contains("cpu") || name_lower.starts_with("u_") {
        "integrated_circuits"
    } else if name_lower.contains("connector") || name_lower.starts_with("j_") || 
              name_lower.starts_with("p_") {
        "connectors"
    } else if name_lower.contains("crystal") || name_lower.contains("oscillator") || 
              name_lower.starts_with("y_") {
        "crystals_oscillators"
    } else if name_lower.contains("switch") || name_lower.contains("button") || 
              name_lower.starts_with("sw_") {
        "switches"
    } else if name_lower.contains("led") || name_lower.contains("light") {
        "leds_displays"
    } else {
        "other"
    }
}

fn extract_component_type(name: &str) -> String {
    // Extract the base component type from the symbol name
    if let Some(underscore_pos) = name.find('_') {
        name[..underscore_pos].to_string()
    } else {
        name.to_string()
    }
}