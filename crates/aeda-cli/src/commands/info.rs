//! Show information about a specific library

use serde::Deserialize;
use std::fs;
use std::path::Path;

#[derive(Deserialize, Debug)]
struct ComponentLibrary {
    name: String,
    #[serde(rename = "type")]
    component_type: String,
    description: String,
    package: String,
    footprint: String,
    #[serde(default)]
    tolerance: String,
    #[serde(default)]
    power_rating: String,
    pins: Vec<String>,
    prefix: String,
    #[serde(default)]
    base_values: Vec<f64>,
    #[serde(default)]
    values: Vec<String>,
}

pub fn run(data_dir: &Path, library: &str) -> Result<(), String> {
    // Parse library path like "resistor::E96_0603"
    let parts: Vec<&str> = library.split("::").collect();
    if parts.len() != 2 {
        return Err(format!(
            "Invalid library path '{}'. Expected format: category::name (e.g., resistor::E96_0603)",
            library
        ));
    }

    let category = parts[0];
    let name = parts[1];

    let lib_path = data_dir.join(format!("libraries/{}/{}.json", category, name));

    if !lib_path.exists() {
        return Err(format!("Library not found: {}", lib_path.display()));
    }

    let content = fs::read_to_string(&lib_path)
        .map_err(|e| format!("Failed to read library: {}", e))?;

    let lib: ComponentLibrary = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse library: {}", e))?;

    println!("Library: {}", library);
    println!("=========={}", "=".repeat(library.len()));
    println!();
    println!("Name:        {}", lib.name);
    println!("Type:        {}", lib.component_type);
    println!("Description: {}", lib.description);
    println!("Package:     {}", lib.package);
    println!("Footprint:   {}", lib.footprint);
    println!("Prefix:      {}", lib.prefix);
    println!("Pins:        {:?}", lib.pins);

    if !lib.tolerance.is_empty() {
        println!("Tolerance:   {}", lib.tolerance);
    }
    if !lib.power_rating.is_empty() {
        println!("Power:       {}", lib.power_rating);
    }

    println!();
    if !lib.base_values.is_empty() {
        println!("Base values: {} values in series", lib.base_values.len());
        println!("  First 10: {:?}", &lib.base_values[..lib.base_values.len().min(10)]);
    }

    if !lib.values.is_empty() {
        println!("Values: {} discrete values", lib.values.len());
        println!("  {:?}", &lib.values[..lib.values.len().min(10)]);
        if lib.values.len() > 10 {
            println!("  ... and {} more", lib.values.len() - 10);
        }
    }

    Ok(())
}
