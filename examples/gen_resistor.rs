extern crate component;
extern crate clap;
use clap::{Parser, ValueEnum};
use std::fs;

#[derive(Debug, Clone, ValueEnum, PartialEq)]
enum OutputFormat {
    Altium,
    Kicad,
}

#[derive(Parser)]
#[command(name = "gen_resistor")]
#[command(about = "Generate resistor libraries for PCB design")]
#[command(version = "0.2.0")]
struct Args {
    /// Output format: altium or kicad
    #[arg(long, default_value = "altium")]
    format: OutputFormat,
    
    /// Package sizes to generate (comma-separated)
    #[arg(long, default_value = "0402,0603,0805,1206")]
    packages: String,
    
    /// Output directory
    #[arg(long, default_value = "outputs")]
    output_dir: String,
    
    /// E-series (24, 48, 96)
    #[arg(long, default_value = "96")]
    series: usize,
    
    /// KiCad target library directory (for --format kicad only)
    #[arg(long)]
    kicad_target_lib: Option<String>,
    
    /// Manufacturer (currently only Vishay is supported)
    #[arg(long, default_value = "Vishay")]
    manufacturer: String,
    
    /// Resistor symbol style (for --format kicad only)
    #[arg(long, default_value = "european")]
    symbol_style: String,
}

fn main() {
    let args = Args::parse();
    
    println!("Atlantix EDA Resistor Library Generator v0.2.0");
    println!("Format: {:?}", args.format);
    println!("Series: E-{}", args.series);
    
    let packages: Vec<&str> = args.packages.split(',').map(|s| s.trim()).collect();
    println!("Packages: {:?}", packages);
    
    if args.manufacturer != "Vishay" {
        eprintln!("Error: Currently only Vishay is supported as a manufacturer");
        std::process::exit(1);
    }
    println!("Manufacturer: {}", args.manufacturer);
    
    if args.symbol_style != "european" && args.symbol_style != "american" {
        eprintln!("Error: Symbol style must be 'european' or 'american'");
        std::process::exit(1);
    }
    if args.format == OutputFormat::Kicad {
        println!("Symbol style: {}", args.symbol_style);
    }
    
    let decades = vec![1, 10, 100, 1000, 10000, 100000];
    
    match args.format {
        OutputFormat::Altium => generate_altium_libraries(&packages, &args.output_dir, args.series, &decades),
        OutputFormat::Kicad => generate_kicad_libraries(&packages, &args.output_dir, args.series, &decades, args.kicad_target_lib.as_deref(), &args.symbol_style),
    }
}

fn generate_altium_libraries(packages: &[&str], output_dir: &str, series: usize, decades: &[u32]) {
    println!("\nGenerating Altium CSV libraries...");
    
    fs::create_dir_all(output_dir).expect("Failed to create output directory");
    
    for package in packages {
        println!("Generating {} package...", package);
        
        let mut resistor = component::Resistor::new(series, package.to_string());
        let mut full_series = String::new();
        
        for decade in decades {
            let series_data = resistor.generate(*decade);
            full_series.push_str(&series_data);
        }
        
        let filename = format!("{}/resistors_{}.csv", output_dir, package);
        let csv_header = "Part,Description,Value,Case,Power,Supplier 1,Supplier Part Number 1,Library Path,Library Ref,Footprint Path,Footprint Ref,Company,Comment\r\n";
        let full_content = format!("{}{}", csv_header, full_series);
        
        match fs::write(&filename, full_content) {
            Ok(()) => println!("Successfully generated {}", filename),
            Err(e) => eprintln!("Error generating {}: {}", filename, e),
        }
    }
    
    println!("\nAltium library generation complete!");
    println!("Files generated in: {}/", output_dir);
    println!("Import these CSV files into Altium Designer's Database Library.");
}

fn generate_kicad_libraries(packages: &[&str], output_dir: &str, series: usize, decades: &[u32], kicad_target_lib: Option<&str>, symbol_style: &str) {
    println!("\nGenerating KiCad libraries...");
    
    let (symbols_dir, footprints_dir) = if let Some(root) = kicad_target_lib {
        (
            format!("{}/symbols", root),
            format!("{}/footprints/Atlantix_Resistors.pretty", root)
        )
    } else {
        (
            format!("{}/kicad/symbols", output_dir),
            format!("{}/kicad/Atlantix_Resistors.pretty", output_dir)
        )
    };
    
    fs::create_dir_all(&symbols_dir).expect("Failed to create symbols directory");
    fs::create_dir_all(&footprints_dir).expect("Failed to create footprints directory");
    
    // Generate symbols for each package
    for package in packages {
        println!("Generating symbols for {} package...", package);
        
        let mut resistor = component::Resistor::new(series, package.to_string());
        let symbol_file = format!("{}/Atlantix_R_{}.kicad_sym", symbols_dir, package);
        
        match resistor.generate_kicad_symbols(decades.to_vec(), &symbol_file, symbol_style) {
            Ok(()) => println!("Successfully generated {}", symbol_file),
            Err(e) => eprintln!("Error generating symbols for {}: {}", package, e),
        }
    }
    
    // Generate footprints
    println!("Generating footprints...");
    let resistor = component::Resistor::new(series, "0603".to_string());
    
    match resistor.generate_kicad_footprints(packages.to_vec(), &footprints_dir) {
        Ok(()) => println!("Successfully generated footprints"),
        Err(e) => eprintln!("Error generating footprints: {}", e),
    }
    
    println!("\nKiCad library generation complete!");
    println!("Files generated:");
    println!("  Symbols: {}/Atlantix_R_*.kicad_sym", symbols_dir);
    println!("  Footprints: {}/*.kicad_mod", footprints_dir);
    if kicad_target_lib.is_some() {
        println!("  Libraries installed to your KiCad target library!");
    }
    println!("");
    if kicad_target_lib.is_some() {
        println!("To use in KiCad:");
        println!("1. Add 'Atlantix_Resistors' footprint library in Footprint Library Manager");
        println!("2. Add symbol libraries in Symbol Library Manager");
        println!("3. Libraries are ready to use in your schematics!");
    } else {
        println!("To use in KiCad:");
        println!("1. Copy symbol files to your KiCad project or global library");
        println!("2. Copy footprint .pretty directory to your KiCad footprint libraries");
        println!("3. Add libraries to your project in Symbol Library Manager and Footprint Library Manager");
    }
}