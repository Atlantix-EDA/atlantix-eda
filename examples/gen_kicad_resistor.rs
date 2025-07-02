extern crate component;
use std::fs;

fn main() {
    println!("Generating KiCad resistor libraries...");
    
    let decades = vec![1, 10, 100, 1000, 10000, 100000];
    let packages = vec!["0402", "0603", "0805", "1206"];
    
    // Create output directories
    fs::create_dir_all("outputs/kicad/symbols").expect("Failed to create symbols directory");
    fs::create_dir_all("outputs/kicad/footprints.pretty").expect("Failed to create footprints directory");
    
    // Generate symbols for each package
    for package in &packages {
        println!("Generating symbols for {} package...", package);
        
        let mut resistor = component::Resistor::new(96, package.to_string());
        let symbol_file = format!("outputs/kicad/symbols/resistors_{}.kicad_sym", package);
        
        match resistor.generate_kicad_symbols(decades.clone(), &symbol_file) {
            Ok(()) => println!("Successfully generated {}", symbol_file),
            Err(e) => eprintln!("Error generating symbols for {}: {}", package, e),
        }
    }
    
    // Generate footprints
    println!("Generating footprints...");
    let resistor = component::Resistor::new(96, "0603".to_string());
    
    match resistor.generate_kicad_footprints(packages, "outputs/kicad/footprints.pretty") {
        Ok(()) => println!("Successfully generated footprints"),
        Err(e) => eprintln!("Error generating footprints: {}", e),
    }
    
    println!("KiCad library generation complete!");
    println!("Files generated:");
    println!("  Symbols: outputs/kicad/symbols/resistors_*.kicad_sym");
    println!("  Footprints: outputs/kicad/footprints.pretty/*.kicad_mod");
    println!("");
    println!("To use in KiCad:");
    println!("1. Copy symbol files to your KiCad project or global library");
    println!("2. Copy footprint .pretty directory to your KiCad footprint libraries");
    println!("3. Add libraries to your project in Symbol Library Manager and Footprint Library Manager");
}