//! Initialize data directory structure

use std::fs;
use std::path::Path;

pub fn run(data_dir: &Path) -> Result<(), String> {
    println!("Initializing Atlantix EDA data directory: {}", data_dir.display());

    // Create directory structure
    let dirs = [
        "libraries/resistor",
        "libraries/capacitor",
        "libraries/inductor",
        "libraries/diode",
        "libraries/ic",
        "footprints",
        "symbols",
        "3d_models",
        "cache",
    ];

    for dir in &dirs {
        let path = data_dir.join(dir);
        fs::create_dir_all(&path)
            .map_err(|e| format!("Failed to create {}: {}", path.display(), e))?;
        println!("  Created: {}", dir);
    }

    // Create default config.toml
    let config_path = data_dir.join("config.toml");
    if !config_path.exists() {
        let default_config = r#"# Atlantix EDA Configuration

[general]
# Default output format: kicad, altium, stencil
default_format = "kicad"

[paths]
# Override default paths (uncomment to customize)
# footprints = "/custom/path/to/footprints"
# symbols = "/custom/path/to/symbols"

[generation]
# Default series for resistor generation
default_resistor_series = "E96"
# Default packages for generation
default_packages = ["0603", "0805", "1206"]

[stencil]
# Path where Stencil looks for libraries
# This should match library_manager base_path in stencil-bd
library_path = "libraries"
"#;
        fs::write(&config_path, default_config)
            .map_err(|e| format!("Failed to write config: {}", e))?;
        println!("  Created: config.toml");
    }

    // Create manifest.json for libraries
    let manifest_path = data_dir.join("libraries/manifest.json");
    if !manifest_path.exists() {
        let default_manifest = r#"{
  "name": "atlantix_eda",
  "version": "1.0.0",
  "description": "Atlantix EDA Component Libraries",
  "libraries": {
    "resistor": {},
    "capacitor": {},
    "inductor": {},
    "diode": {},
    "ic": {}
  }
}
"#;
        fs::write(&manifest_path, default_manifest)
            .map_err(|e| format!("Failed to write manifest: {}", e))?;
        println!("  Created: libraries/manifest.json");
    }

    println!("\nInitialization complete!");
    println!("\nNext steps:");
    println!("  aeda generate resistors --series E96 --packages 0603,0805,1206");
    println!("  aeda export stencil");
    println!("  aeda list");

    Ok(())
}
