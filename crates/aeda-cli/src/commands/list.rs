//! List available component libraries

use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Deserialize)]
struct Manifest {
    name: String,
    version: String,
    libraries: HashMap<String, HashMap<String, String>>,
}

pub fn run(data_dir: &Path, component_type: &str) -> Result<(), String> {
    let manifest_path = data_dir.join("libraries/manifest.json");

    if !manifest_path.exists() {
        return Err(format!(
            "Manifest not found at {}. Run 'aeda init' first.",
            manifest_path.display()
        ));
    }

    let content = fs::read_to_string(&manifest_path)
        .map_err(|e| format!("Failed to read manifest: {}", e))?;

    let manifest: Manifest = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse manifest: {}", e))?;

    println!("Atlantix EDA Libraries ({})", manifest.name);
    println!("Version: {}\n", manifest.version);

    let filter_all = component_type == "all";

    for (category, items) in &manifest.libraries {
        if !filter_all && category != component_type {
            continue;
        }

        if items.is_empty() {
            println!("{}/ (empty - run 'aeda generate')", category);
        } else {
            println!("{}/", category);
            for (name, path) in items {
                println!("  {}::{} -> {}", category, name, path);
            }
        }
        println!();
    }

    if manifest.libraries.values().all(|v| v.is_empty()) {
        println!("No libraries generated yet.");
        println!("\nGenerate libraries with:");
        println!("  aeda generate resistors --series E96 --packages 0603,0805");
        println!("  aeda generate capacitors --dielectric X7R --packages 0603");
    }

    Ok(())
}
