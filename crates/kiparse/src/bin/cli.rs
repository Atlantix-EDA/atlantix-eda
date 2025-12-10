use clap::{Parser, Subcommand};
use kiparse::{pcb, symbol, Result};
use prettytable::{row, Table};
use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "kpx")]
#[command(about = "KiCad file parser and analyzer", long_about = None)]
#[command(version)]
struct Cli {
    /// The KiCad file to analyze
    file: PathBuf,

    #[command(subcommand)]
    command: Commands,

    /// Output in JSON format
    #[arg(short, long)]
    json: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Get detailed PCB information
    Details,
    
    /// Extract layer information
    Layers,
    
    /// Analyze 3D model coverage
    #[command(name = "3d")]
    ThreeDModels,
    
    /// Extract component positions
    Positions,
    
    /// Parse symbol libraries
    Symbols,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    
    let content = fs::read_to_string(&cli.file)?;
    let filename = cli.file.to_str().unwrap_or("unknown");
    
    match cli.command {
        Commands::Details => {
            if filename.ends_with(".kicad_pcb") {
                handle_pcb_details(&content, cli.json)?;
            } else {
                eprintln!("Details command requires a .kicad_pcb file");
                std::process::exit(1);
            }
        }
        Commands::Layers => {
            if filename.ends_with(".kicad_pcb") {
                handle_layers(&content, cli.json)?;
            } else {
                eprintln!("Layers command requires a .kicad_pcb file");
                std::process::exit(1);
            }
        }
        Commands::ThreeDModels => {
            if filename.ends_with(".kicad_pcb") {
                handle_3d_models(&content, cli.json)?;
            } else {
                eprintln!("3d command requires a .kicad_pcb file");
                std::process::exit(1);
            }
        }
        Commands::Positions => {
            if filename.ends_with(".kicad_pcb") {
                handle_positions(&content, cli.json)?;
            } else {
                eprintln!("Positions command requires a .kicad_pcb file");
                std::process::exit(1);
            }
        }
        Commands::Symbols => {
            if filename.ends_with(".kicad_sym") {
                handle_symbols(&content, cli.json)?;
            } else {
                eprintln!("Symbols command requires a .kicad_sym file");
                std::process::exit(1);
            }
        }
    }
    
    Ok(())
}

fn handle_pcb_details(content: &str, json_output: bool) -> Result<()> {
    let pcb = pcb::parse_layers_only(content)?;
    
    // Extract board dimensions from Edge.Cuts layer
    let edge_cuts_re = Regex::new(
        r#"(?s)\(gr_line\s*\(start\s+([\d.-]+)\s+([\d.-]+)\)\s*\(end\s+([\d.-]+)\s+([\d.-]+)\).*?\(layer\s+"Edge\.Cuts"\)"#
    ).unwrap();
    
    let mut min_x = f64::MAX;
    let mut min_y = f64::MAX;
    let mut max_x = f64::MIN;
    let mut max_y = f64::MIN;
    
    for cap in edge_cuts_re.captures_iter(content) {
        let x1: f64 = cap[1].parse().unwrap_or(0.0);
        let y1: f64 = cap[2].parse().unwrap_or(0.0);
        let x2: f64 = cap[3].parse().unwrap_or(0.0);
        let y2: f64 = cap[4].parse().unwrap_or(0.0);
        
        min_x = min_x.min(x1).min(x2);
        min_y = min_y.min(y1).min(y2);
        max_x = max_x.max(x1).max(x2);
        max_y = max_y.max(y1).max(y2);
    }
    
    let board_width_mm = if max_x > min_x { max_x - min_x } else { 0.0 };
    let board_height_mm = if max_y > min_y { max_y - min_y } else { 0.0 };
    let board_width_mils = board_width_mm * 39.3701; // 1mm = 39.3701 mils
    let board_height_mils = board_height_mm * 39.3701;
    let board_area_mm2 = board_width_mm * board_height_mm;
    let board_area_sq_in = board_area_mm2 / 645.16; // 1 sq inch = 645.16 mm²
    
    if json_output {
        #[cfg(feature = "json")]
        {
            let output = serde_json::json!({
                "layers": pcb.layers.len(),
                "signal_layers": pcb.layers.values().filter(|l| l.layer_type == "signal").count(),
                "file_size_kb": content.len() as f64 / 1024.0,
                "complexity": estimate_complexity(content),
                "board_size": {
                    "width_mm": board_width_mm,
                    "height_mm": board_height_mm,
                    "width_mils": board_width_mils,
                    "height_mils": board_height_mils,
                    "area_mm2": board_area_mm2,
                    "area_sq_in": board_area_sq_in,
                },
                "components": content.matches("(footprint").count(),
                "tracks": content.matches("(segment").count(),
                "vias": content.matches("(via").count(),
            });
            println!("{}", serde_json::to_string_pretty(&output)?);
        }
        #[cfg(not(feature = "json"))]
        {
            eprintln!("JSON output requires the 'json' feature");
            std::process::exit(1);
        }
    } else {
        println!("KiCad PCB Analysis");
        println!("==================");
        println!("Layers: {}", pcb.layers.len());
        println!("Signal layers: {}", pcb.layers.values().filter(|l| l.layer_type == "signal").count());
        println!("File size: {:.2} KB", content.len() as f64 / 1024.0);
        println!("Complexity: {}", estimate_complexity(content));
        
        // Board dimensions
        if board_width_mm > 0.0 && board_height_mm > 0.0 {
            println!("\nBoard Dimensions:");
            println!("  Width:  {:.2} mm ({:.0} mils)", board_width_mm, board_width_mils);
            println!("  Height: {:.2} mm ({:.0} mils)", board_height_mm, board_height_mils);
            println!("  Area:   {:.2} mm² ({:.2} sq in)", board_area_mm2, board_area_sq_in);
        }
        
        // Component count
        let footprint_count = content.matches("(footprint").count();
        let track_count = content.matches("(segment").count();
        let via_count = content.matches("(via").count();
        
        println!("\nBoard Statistics:");
        println!("  Components: {}", footprint_count);
        println!("  Tracks: {}", track_count);
        println!("  Vias: {}", via_count);
        
        if board_area_mm2 > 0.0 && footprint_count > 0 {
            let density = (footprint_count as f64 / board_area_mm2) * 645.16; // components per sq inch
            println!("  Density: {:.1} components/sq inch", density);
        }
    }
    
    Ok(())
}

fn handle_layers(content: &str, json_output: bool) -> Result<()> {
    let pcb = pcb::parse_layers_only(content)?;
    
    if json_output {
        #[cfg(feature = "json")]
        {
            println!("{}", serde_json::to_string_pretty(&pcb)?);
        }
        #[cfg(not(feature = "json"))]
        {
            eprintln!("JSON output requires the 'json' feature");
            std::process::exit(1);
        }
    } else {
        println!("KiCad PCB Layer Information");
        println!("===========================");
        println!("Total layers: {}", pcb.layers.len());
        
        let mut table = Table::new();
        table.add_row(row!["ID", "Name", "Type", "User Name"]);
        
        let mut sorted_layers: Vec<_> = pcb.layers.iter().collect();
        sorted_layers.sort_by_key(|(id, _)| *id);
        
        for (id, layer) in sorted_layers {
            table.add_row(row![
                id,
                layer.name,
                layer.layer_type,
                layer.user_name.as_deref().unwrap_or("-")
            ]);
        }
        
        table.printstd();
    }
    
    Ok(())
}

fn handle_3d_models(content: &str, json_output: bool) -> Result<()> {
    let pcb = pcb::parse_layers_only(content)?;
    
    // Extract 3D model information
    let footprint_re = Regex::new(
        r#"(?s)\(footprint\s+"([^"]+)".*?\(property\s+"Reference"\s+"([^"]+)".*?\(model\s+"([^"]+)""#
    ).unwrap();
    
    let all_refs_re = Regex::new(
        r#"(?s)\(footprint\s+"[^"]+?".*?\(property\s+"Reference"\s+"([^"]+)""#
    ).unwrap();
    
    let mut components_with_models = Vec::new();
    let mut model_types: HashMap<String, usize> = HashMap::new();
    
    for cap in footprint_re.captures_iter(content) {
        let reference = cap[2].to_string();
        let model_path = &cap[3];
        
        components_with_models.push(reference);
        
        let model_type = if model_path.ends_with(".wrl") {
            "WRL"
        } else if model_path.ends_with(".step") || model_path.ends_with(".stp") {
            "STEP"
        } else {
            "Other"
        };
        *model_types.entry(model_type.to_string()).or_insert(0) += 1;
    }
    
    let mut all_refs = HashSet::new();
    for cap in all_refs_re.captures_iter(content) {
        all_refs.insert(cap[1].to_string());
    }
    
    let total = all_refs.len();
    let with_models = components_with_models.len();
    let without_models = total - with_models;
    let coverage = if total > 0 { (with_models as f64 / total as f64) * 100.0 } else { 0.0 };
    
    if json_output {
        #[cfg(feature = "json")]
        {
            let output = serde_json::json!({
                "total_components": total,
                "with_3d_models": with_models,
                "without_3d_models": without_models,
                "coverage_percent": coverage,
                "model_types": model_types,
            });
            println!("{}", serde_json::to_string_pretty(&output)?);
        }
        #[cfg(not(feature = "json"))]
        {
            eprintln!("JSON output requires the 'json' feature");
            std::process::exit(1);
        }
    } else {
        println!("3D Model Coverage Analysis");
        println!("==========================");
        println!("Total components: {}", total);
        println!("With 3D models: {} ({:.1}%)", with_models, coverage);
        println!("Without 3D models: {} ({:.1}%)", without_models, 100.0 - coverage);
        
        if !model_types.is_empty() {
            println!("\nModel Types:");
            for (model_type, count) in &model_types {
                println!("  {}: {}", model_type, count);
            }
        }
    }
    
    Ok(())
}

fn handle_positions(content: &str, json_output: bool) -> Result<()> {
    let _pcb = pcb::parse_layers_only(content)?;
    
    // Extract component positions
    let component_re = Regex::new(
        r#"(?s)\(footprint\s+"([^"]+)".*?\(at\s+([\d.-]+)\s+([\d.-]+)(?:\s+([\d.-]+))?\).*?\(property\s+"Reference"\s+"([^"]+)""#
    ).unwrap();
    
    let mut components = Vec::new();
    
    for cap in component_re.captures_iter(content) {
        let footprint = &cap[1];
        let x: f64 = cap[2].parse().unwrap_or(0.0);
        let y: f64 = cap[3].parse().unwrap_or(0.0);
        let rotation: f64 = cap.get(4).map_or(0.0, |m| m.as_str().parse().unwrap_or(0.0));
        let reference = &cap[5];
        
        components.push(serde_json::json!({
            "reference": reference,
            "footprint": footprint,
            "x": x,
            "y": y,
            "rotation": rotation,
        }));
    }
    
    if json_output {
        #[cfg(feature = "json")]
        {
            let output = serde_json::json!({
                "component_count": components.len(),
                "components": components,
            });
            println!("{}", serde_json::to_string_pretty(&output)?);
        }
        #[cfg(not(feature = "json"))]
        {
            eprintln!("JSON output requires the 'json' feature");
            std::process::exit(1);
        }
    } else {
        println!("Component Positions");
        println!("===================");
        println!("Total components: {}", components.len());
        
        if !components.is_empty() {
            let mut table = Table::new();
            table.add_row(row!["Reference", "X (mm)", "Y (mm)", "Rotation", "Footprint"]);
            
            // Sort by reference
            components.sort_by(|a, b| {
                let ref_a = a["reference"].as_str().unwrap_or("");
                let ref_b = b["reference"].as_str().unwrap_or("");
                natural_sort(ref_a, ref_b)
            });
            
            for comp in &components {
                table.add_row(row![
                    comp["reference"].as_str().unwrap_or("-"),
                    format!("{:.2}", comp["x"].as_f64().unwrap_or(0.0)),
                    format!("{:.2}", comp["y"].as_f64().unwrap_or(0.0)),
                    format!("{:.0}°", comp["rotation"].as_f64().unwrap_or(0.0)),
                    comp["footprint"].as_str().unwrap_or("-"),
                ]);
            }
            
            table.printstd();
        }
    }
    
    Ok(())
}

fn handle_symbols(content: &str, json_output: bool) -> Result<()> {
    let symbols = symbol::parse_symbol_lib(content)?;
    
    if json_output {
        #[cfg(feature = "json")]
        {
            println!("{}", serde_json::to_string_pretty(&symbols)?);
        }
        #[cfg(not(feature = "json"))]
        {
            eprintln!("JSON output requires the 'json' feature");
            std::process::exit(1);
        }
    } else {
        println!("Symbol Library Analysis");
        println!("=======================");
        println!("Total symbols: {}", symbols.len());
        
        if !symbols.is_empty() {
            let mut table = Table::new();
            table.add_row(row!["Symbol", "Description"]);
            
            for symbol in &symbols {
                table.add_row(row![
                    symbol.name,
                    if symbol.description.is_empty() {
                        "-"
                    } else {
                        &symbol.description
                    }
                ]);
            }
            
            table.printstd();
        }
    }
    
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

fn natural_sort(a: &str, b: &str) -> std::cmp::Ordering {
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