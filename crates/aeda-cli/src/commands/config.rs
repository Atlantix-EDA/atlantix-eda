//! Show configuration and paths

use std::path::Path;

pub fn run(data_dir: &Path) -> Result<(), String> {
    println!("Atlantix EDA Configuration");
    println!("==========================\n");

    println!("Data directory: {}", data_dir.display());
    println!();

    println!("Directory structure:");
    let dirs = [
        ("libraries/", "Component library manifests (JSON)"),
        ("footprints/", "KiCad footprint files (.kicad_mod)"),
        ("symbols/", "KiCad symbol files (.kicad_sym)"),
        ("3d_models/", "3D models (STEP, WRL)"),
        ("cache/", "Downloaded/temporary files"),
    ];

    for (dir, desc) in &dirs {
        let path = data_dir.join(dir);
        let status = if path.exists() { "✓" } else { "✗" };
        println!("  {} {} - {}", status, dir, desc);
    }

    println!();

    // Check config file
    let config_path = data_dir.join("config.toml");
    if config_path.exists() {
        println!("Config file: {} (exists)", config_path.display());
    } else {
        println!("Config file: {} (not found - run 'aeda init')", config_path.display());
    }

    // Check manifest
    let manifest_path = data_dir.join("libraries/manifest.json");
    if manifest_path.exists() {
        println!("Library manifest: {} (exists)", manifest_path.display());
    } else {
        println!("Library manifest: {} (not found - run 'aeda init')", manifest_path.display());
    }

    println!();
    println!("Environment:");
    println!("  HOME: {}", std::env::var("HOME").unwrap_or_else(|_| "(not set)".into()));

    Ok(())
}
