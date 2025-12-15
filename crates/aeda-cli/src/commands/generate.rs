//! Generate component libraries

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// E-series base values
fn get_e_series(series: &str) -> Result<Vec<f64>, String> {
    match series.to_uppercase().as_str() {
        "E96" => Ok(vec![
            1.00, 1.02, 1.05, 1.07, 1.10, 1.13, 1.15, 1.18, 1.21, 1.24,
            1.27, 1.30, 1.33, 1.37, 1.40, 1.43, 1.47, 1.50, 1.54, 1.58,
            1.62, 1.65, 1.69, 1.74, 1.78, 1.82, 1.87, 1.91, 1.96, 2.00,
            2.05, 2.10, 2.15, 2.21, 2.26, 2.32, 2.37, 2.43, 2.49, 2.55,
            2.61, 2.67, 2.74, 2.80, 2.87, 2.94, 3.01, 3.09, 3.16, 3.24,
            3.32, 3.40, 3.48, 3.57, 3.65, 3.74, 3.83, 3.92, 4.02, 4.12,
            4.22, 4.32, 4.42, 4.53, 4.64, 4.75, 4.87, 4.99, 5.11, 5.23,
            5.36, 5.49, 5.62, 5.76, 5.90, 6.04, 6.19, 6.34, 6.49, 6.65,
            6.81, 6.98, 7.15, 7.32, 7.50, 7.68, 7.87, 8.06, 8.25, 8.45,
            8.66, 8.87, 9.09, 9.31, 9.53, 9.76,
        ]),
        "E48" => Ok(vec![
            1.00, 1.05, 1.10, 1.15, 1.21, 1.27, 1.33, 1.40, 1.47, 1.54,
            1.62, 1.69, 1.78, 1.87, 1.96, 2.05, 2.15, 2.26, 2.37, 2.49,
            2.61, 2.74, 2.87, 3.01, 3.16, 3.32, 3.48, 3.65, 3.83, 4.02,
            4.22, 4.42, 4.64, 4.87, 5.11, 5.36, 5.62, 5.90, 6.19, 6.49,
            6.81, 7.15, 7.50, 7.87, 8.25, 8.66, 9.09, 9.53,
        ]),
        "E24" => Ok(vec![
            1.0, 1.1, 1.2, 1.3, 1.5, 1.6, 1.8, 2.0, 2.2, 2.4, 2.7, 3.0,
            3.3, 3.6, 3.9, 4.3, 4.7, 5.1, 5.6, 6.2, 6.8, 7.5, 8.2, 9.1,
        ]),
        "E12" => Ok(vec![
            1.0, 1.2, 1.5, 1.8, 2.2, 2.7, 3.3, 3.9, 4.7, 5.6, 6.8, 8.2,
        ]),
        "E6" => Ok(vec![1.0, 1.5, 2.2, 3.3, 4.7, 6.8]),
        _ => Err(format!("Unknown E-series: {}", series)),
    }
}

fn get_tolerance(series: &str) -> &'static str {
    match series.to_uppercase().as_str() {
        "E96" => "1%",
        "E48" => "2%",
        "E24" => "5%",
        "E12" => "10%",
        "E6" => "20%",
        _ => "1%",
    }
}

fn get_power_rating(package: &str) -> &'static str {
    match package {
        "0201" => "1/20W",
        "0402" => "1/16W",
        "0603" => "1/10W",
        "0805" => "1/8W",
        "1206" => "1/4W",
        "1210" => "1/2W",
        "2010" => "3/4W",
        "2512" => "1W",
        _ => "1/10W",
    }
}

fn get_metric_suffix(package: &str) -> &'static str {
    match package {
        "0201" => "_0603Metric",
        "0402" => "_1005Metric",
        "0603" => "_1608Metric",
        "0805" => "_2012Metric",
        "1206" => "_3216Metric",
        "1210" => "_3225Metric",
        "2010" => "_5025Metric",
        "2512" => "_6332Metric",
        _ => "_Metric",
    }
}

#[derive(Serialize)]
struct ResistorLibrary {
    name: String,
    #[serde(rename = "type")]
    component_type: String,
    description: String,
    package: String,
    footprint: String,
    tolerance: String,
    power_rating: String,
    series: String,
    pins: Vec<String>,
    prefix: String,
    base_values: Vec<f64>,
    multipliers: HashMap<String, f64>,
    methods: LibraryMethods,
}

#[derive(Serialize)]
struct CapacitorLibrary {
    name: String,
    #[serde(rename = "type")]
    component_type: String,
    description: String,
    package: String,
    footprint: String,
    dielectric: String,
    voltage_rating: String,
    tolerance: String,
    pins: Vec<String>,
    prefix: String,
    values: Vec<String>,
    value_suffixes: HashMap<String, f64>,
    methods: LibraryMethods,
}

#[derive(Serialize)]
struct LibraryMethods {
    after_factory: Vec<String>,
    after_value: Vec<String>,
}

impl Default for LibraryMethods {
    fn default() -> Self {
        Self {
            after_factory: vec![
                "and_value".into(),
                "at".into(),
                "located_at".into(),
                "on_layer".into(),
                "rotated".into(),
                "place".into(),
            ],
            after_value: vec![
                "at".into(),
                "located_at".into(),
                "on_layer".into(),
                "rotated".into(),
                "place".into(),
            ],
        }
    }
}

#[derive(Serialize, Deserialize)]
struct Manifest {
    name: String,
    version: String,
    description: String,
    libraries: HashMap<String, HashMap<String, String>>,
}

fn update_manifest(data_dir: &Path, category: &str, name: &str, path: &str) -> Result<(), String> {
    let manifest_path = data_dir.join("libraries/manifest.json");

    let mut manifest: Manifest = if manifest_path.exists() {
        let content = fs::read_to_string(&manifest_path)
            .map_err(|e| format!("Failed to read manifest: {}", e))?;
        serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse manifest: {}", e))?
    } else {
        Manifest {
            name: "atlantix_eda".into(),
            version: "1.0.0".into(),
            description: "Atlantix EDA Component Libraries".into(),
            libraries: HashMap::new(),
        }
    };

    manifest
        .libraries
        .entry(category.to_string())
        .or_insert_with(HashMap::new)
        .insert(name.to_string(), path.to_string());

    let content = serde_json::to_string_pretty(&manifest)
        .map_err(|e| format!("Failed to serialize manifest: {}", e))?;

    fs::write(&manifest_path, content)
        .map_err(|e| format!("Failed to write manifest: {}", e))?;

    Ok(())
}

pub fn resistors(data_dir: &Path, series: &str, packages: &str) -> Result<(), String> {
    let base_values = get_e_series(series)?;
    let tolerance = get_tolerance(series);
    let packages: Vec<&str> = packages.split(',').map(|s| s.trim()).collect();

    println!("Generating {} resistor libraries...", series);

    // Ensure directory exists
    let resistor_dir = data_dir.join("libraries/resistor");
    fs::create_dir_all(&resistor_dir)
        .map_err(|e| format!("Failed to create directory: {}", e))?;

    for package in &packages {
        let name = format!("{}_{}", series, package);
        let metric = get_metric_suffix(package);
        let footprint = format!("Resistor_SMD:R_{}{}", package, metric);
        let power = get_power_rating(package);

        let library = ResistorLibrary {
            name: name.clone(),
            component_type: "resistor".into(),
            description: format!("{} Resistors in {} package", series, package),
            package: package.to_string(),
            footprint,
            tolerance: tolerance.into(),
            power_rating: power.into(),
            series: series.into(),
            pins: vec!["1".into(), "2".into()],
            prefix: "R".into(),
            base_values: base_values.clone(),
            multipliers: [
                ("".into(), 1.0),
                ("k".into(), 1000.0),
                ("K".into(), 1000.0),
                ("M".into(), 1_000_000.0),
            ]
            .into_iter()
            .collect(),
            methods: LibraryMethods::default(),
        };

        let lib_path = resistor_dir.join(format!("{}.json", name));
        let content = serde_json::to_string_pretty(&library)
            .map_err(|e| format!("Failed to serialize library: {}", e))?;

        fs::write(&lib_path, content)
            .map_err(|e| format!("Failed to write library: {}", e))?;

        // Update manifest
        update_manifest(
            data_dir,
            "resistor",
            &name,
            &format!("resistor/{}.json", name),
        )?;

        println!("  Created: resistor::{} ({} base values)", name, base_values.len());
    }

    println!("\nDone! Libraries available at: {}", resistor_dir.display());
    Ok(())
}

pub fn capacitors(data_dir: &Path, dielectric: &str, packages: &str) -> Result<(), String> {
    let packages: Vec<&str> = packages.split(',').map(|s| s.trim()).collect();

    println!("Generating {} capacitor libraries...", dielectric);

    // Ensure directory exists
    let capacitor_dir = data_dir.join("libraries/capacitor");
    fs::create_dir_all(&capacitor_dir)
        .map_err(|e| format!("Failed to create directory: {}", e))?;

    // Standard capacitor values
    let values = vec![
        "10pF", "22pF", "47pF", "100pF", "220pF", "470pF",
        "1nF", "2.2nF", "4.7nF", "10nF", "22nF", "47nF",
        "100nF", "220nF", "470nF", "1uF", "2.2uF", "4.7uF", "10uF",
    ];

    for package in &packages {
        let name = format!("{}_{}", dielectric, package);
        let metric = get_metric_suffix(package);
        let footprint = format!("Capacitor_SMD:C_{}{}", package, metric);

        let library = CapacitorLibrary {
            name: name.clone(),
            component_type: "capacitor".into(),
            description: format!("{} MLCC Capacitors in {} package", dielectric, package),
            package: package.to_string(),
            footprint,
            dielectric: dielectric.into(),
            voltage_rating: "16V".into(),
            tolerance: "10%".into(),
            pins: vec!["1".into(), "2".into()],
            prefix: "C".into(),
            values: values.iter().map(|s| s.to_string()).collect(),
            value_suffixes: [
                ("pF".into(), 1e-12),
                ("nF".into(), 1e-9),
                ("uF".into(), 1e-6),
                ("µF".into(), 1e-6),
            ]
            .into_iter()
            .collect(),
            methods: LibraryMethods::default(),
        };

        let lib_path = capacitor_dir.join(format!("{}.json", name));
        let content = serde_json::to_string_pretty(&library)
            .map_err(|e| format!("Failed to serialize library: {}", e))?;

        fs::write(&lib_path, content)
            .map_err(|e| format!("Failed to write library: {}", e))?;

        // Update manifest
        update_manifest(
            data_dir,
            "capacitor",
            &name,
            &format!("capacitor/{}.json", name),
        )?;

        println!("  Created: capacitor::{} ({} values)", name, values.len());
    }

    println!("\nDone! Libraries available at: {}", capacitor_dir.display());
    Ok(())
}
