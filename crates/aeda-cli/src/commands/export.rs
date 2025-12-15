//! Export libraries to different formats

use std::path::Path;

pub fn to_kicad(data_dir: &Path, output: Option<&Path>) -> Result<(), String> {
    let output_dir = output.unwrap_or_else(|| Path::new("./kicad_libs"));

    println!("Exporting to KiCad format...");
    println!("Output directory: {}", output_dir.display());

    // TODO: Implement KiCad symbol and footprint generation
    // This would use atlantix-core's KicadSymbol and KicadFootprint

    println!();
    println!("KiCad export not yet implemented.");
    println!("Use atlantix-core directly for now:");
    println!("  cargo run --example gen_kicad_resistor");

    Ok(())
}

pub fn to_stencil(data_dir: &Path, output: Option<&Path>) -> Result<(), String> {
    let default_output = data_dir.join("libraries");
    let output_dir = output.unwrap_or(&default_output);

    println!("Exporting to Stencil DSL format...");
    println!("Output directory: {}", output_dir.display());

    // Stencil format is already the native format in data/libraries/
    // This command just confirms the libraries are ready

    let manifest_path = output_dir.join("manifest.json");
    if manifest_path.exists() {
        println!();
        println!("Libraries already in Stencil format at: {}", output_dir.display());
        println!();
        println!("To use in Stencil Designer, ensure library_manager points to:");
        println!("  {}", output_dir.display());
        println!();
        println!("Example usage in .stencil file:");
        println!("  local r = library(\"resistor::E96_0603\")");
        println!("  local r1 = r(\"10k\").at(10, 10).place()");
    } else {
        println!();
        println!("No libraries found. Generate them first:");
        println!("  aeda generate resistors --series E96 --packages 0603,0805");
    }

    Ok(())
}

pub fn to_altium(data_dir: &Path, output: Option<&Path>) -> Result<(), String> {
    let output_dir = output.unwrap_or_else(|| Path::new("./altium_libs"));

    println!("Exporting to Altium format...");
    println!("Output directory: {}", output_dir.display());

    // TODO: Implement Altium export
    // Would generate .SchLib and .PcbLib files

    println!();
    println!("Altium export not yet implemented.");
    println!("This feature is planned for a future release.");

    Ok(())
}
