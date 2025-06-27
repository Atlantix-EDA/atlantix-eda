use bevy_ecs::prelude::*;
use crate::ecs::components::*;
use crate::ecs::resources::*;

/// Generate E-series values for resistors
pub fn generate_eseries_values(
    mut commands: Commands,
    config: Res<GeneratorConfig>,
    mut eseries_cache: ResMut<ESeriesCache>,
    query: Query<(Entity, &ESeries, &Package), Without<ResistorValue>>,
) {
    for (entity, series, package) in &query {
        let base_values = eseries_cache.get_or_calculate(series.0);
        
        // Generate values for all decades
        for decade in &config.decades {
            for base_value in &base_values {
                let ohms = base_value * (*decade as f64);
                let formatted = format_resistance(ohms);
                
                // Spawn a new resistor entity for each value
                commands.spawn(ResistorBundle {
                    value: ResistorValue { ohms, formatted: formatted.clone() },
                    package: package.clone(),
                    tolerance: Tolerance(get_tolerance_from_series(series.0)),
                    power: PowerRating(get_power_from_package(&package.name)),
                    description: Description(String::new()), // Will be filled by another system
                    part_number: PartNumber(format!("R{}_{}", package.name, formatted)),
                    manufacturers: ManufacturerParts::default(),
                });
            }
        }
        
        // Remove the template entity
        commands.entity(entity).despawn();
    }
}

/// Assign package-specific attributes
pub fn assign_package_attributes(
    mut query: Query<(&mut Description, &ResistorValue, &Package, &Tolerance, &PowerRating), Added<ResistorValue>>,
) {
    for (mut description, value, package, tolerance, power) in &mut query {
        description.0 = format!(
            "RES SMT {}ohms, {}, {}, {}",
            value.formatted,
            package.name,
            tolerance.0,
            power.0
        );
    }
}

/// Calculate tolerances based on E-series
pub fn calculate_tolerances(
    query: Query<(Entity, &ESeries), Without<Tolerance>>,
    mut commands: Commands,
) {
    for (entity, series) in &query {
        let tolerance = get_tolerance_from_series(series.0);
        commands.entity(entity).insert(Tolerance(tolerance));
    }
}

/// Generate manufacturer-specific part numbers
pub fn generate_manufacturer_parts(
    mut query: Query<(&mut ManufacturerParts, &ResistorValue, &Package)>,
    config: Res<GeneratorConfig>,
) {
    for (mut mfr_parts, value, package) in &mut query {
        let mut parts = Vec::new();
        
        for manufacturer in &config.manufacturers {
            match manufacturer.as_str() {
                "Vishay" => {
                    parts.push(ManufacturerPart {
                        manufacturer: "Vishay".to_string(),
                        mpn: generate_vishay_mpn(&value.ohms, &package.name),
                        distributor: "Digikey".to_string(),
                        distributor_pn: generate_vishay_digikey_pn(&value.formatted, &package.name),
                    });
                }
                "Yageo" => {
                    parts.push(ManufacturerPart {
                        manufacturer: "Yageo".to_string(),
                        mpn: generate_yageo_mpn(&value.ohms, &package.name),
                        distributor: "Mouser".to_string(),
                        distributor_pn: generate_yageo_mouser_pn(&value.formatted, &package.name),
                    });
                }
                "KOA" => {
                    parts.push(ManufacturerPart {
                        manufacturer: "KOA Speer".to_string(),
                        mpn: generate_koa_mpn(&value.ohms, &package.name),
                        distributor: "Digikey".to_string(),
                        distributor_pn: generate_koa_digikey_pn(&value.ohms, &package.name),
                    });
                }
                _ => {}
            }
        }
        
        mfr_parts.0 = parts;
    }
}

/// Format outputs based on configuration
pub fn format_outputs(
    query: Query<(&ResistorValue, &Package, &Description, &PartNumber, &ManufacturerParts)>,
    config: Res<GeneratorConfig>,
    mut commands: Commands,
) {
    for (value, package, description, part_number, mfr_parts) in &query {
        for format in &config.output_formats {
            match format {
                OutputFormat::KicadSymbols => {
                    // Generate KiCad symbol with manufacturer fields
                    let symbol = generate_kicad_symbol_with_mfrs(
                        &part_number.0,
                        &value.formatted,
                        &format!("Atlantix_Resistors:R_{}_{}", package.imperial, package.metric),
                        &description.0,
                        &mfr_parts.0,
                    );
                    // In a real implementation, we'd collect these for file output
                }
                OutputFormat::Altium => {
                    // Generate Altium CSV line
                    if let Some(first_mfr) = mfr_parts.0.first() {
                        let csv_line = format!(
                            "{},{},{},{},{},{},{},Atlantix_R.SchLib,Res1,Atlantix_R.PcbLib,RES{},Atlantix EDA,=Description",
                            part_number.0,
                            description.0,
                            value.formatted,
                            package.name,
                            get_power_from_package(&package.name),
                            first_mfr.distributor,
                            first_mfr.distributor_pn,
                            package.name
                        );
                        // In a real implementation, we'd collect these for file output
                    }
                }
                _ => {}
            }
        }
    }
}

// Helper functions
fn format_resistance(ohms: f64) -> String {
    match ohms {
        o if o < 10.0 => format!("{:.2}", o),
        o if o < 100.0 => format!("{:.1}", o),
        o if o < 1000.0 => format!("{:.0}", o),
        o if o < 10000.0 => format!("{:.2}K", o / 1000.0),
        o if o < 100000.0 => format!("{:.1}K", o / 1000.0),
        o if o < 1000000.0 => format!("{:.0}K", o / 1000.0),
        _ => format!("{:.2}M", ohms / 1000000.0),
    }
}

fn get_tolerance_from_series(series: usize) -> String {
    match series {
        192 => "0.5%",
        96 => "1%",
        48 => "2%",
        24 => "5%",
        12 => "10%",
        6 => "20%",
        _ => "1%",
    }.to_string()
}

fn get_power_from_package(package: &str) -> String {
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
    }.to_string()
}

fn generate_vishay_mpn(ohms: &f64, package: &str) -> String {
    // Simplified - real implementation would be more complex
    format!("CRCW{}{:04.0}FKEA", package, ohms)
}

fn generate_vishay_digikey_pn(formatted: &str, package: &str) -> String {
    format!("541-{}CT-ND", formatted)
}

fn generate_yageo_mpn(ohms: &f64, package: &str) -> String {
    format!("RC{}FR-07{}L", package, format_resistance(*ohms))
}

fn generate_yageo_mouser_pn(formatted: &str, package: &str) -> String {
    format!("603-RC{}FR-07{}", package, formatted)
}

fn generate_koa_mpn(ohms: &f64, package: &str) -> String {
    // KOA Speer part numbering: RK73H[size][tolerance]TD[value][tolerance_letter]
    // RK73H = Thick film chip resistor series
    // Size codes: 1E = 0402, 1J = 0603, 2A = 0805, 2B = 1206, 2E = 1210, 3A = 2010, 3E = 2512
    let size_code = match package {
        "0402" => "1E",
        "0603" => "1J",
        "0805" => "2A",
        "1206" => "2B",
        "1210" => "2E",
        "2010" => "3A",
        "2512" => "3E",
        _ => "1J",
    };
    
    // Convert resistance to KOA format (4 digits)
    let value_code = format_koa_resistance(*ohms);
    
    // TTD = Thin Thick Film, F = 1% tolerance
    format!("RK73H{}TTD{}F", size_code, value_code)
}

fn generate_koa_digikey_pn(ohms: &f64, package: &str) -> String {
    // Generate Digikey part number for KOA parts
    let mpn = generate_koa_mpn(ohms, package);
    format!("{}-ND", mpn)
}

fn format_koa_resistance(ohms: f64) -> String {
    // KOA uses a 4-digit code system
    // Examples: 1001 = 1.00K, 4701 = 4.70K, 1000 = 100Ω, 10R0 = 10.0Ω
    match ohms {
        o if o < 10.0 => {
            // For values less than 10 ohms, use R notation
            let value = (o * 10.0).round() as i32;
            format!("{:02}R{}", value / 10, value % 10)
        }
        o if o < 100.0 => {
            // 10-99 ohms: multiply by 10 to get 3 digits + 0
            format!("{:03}0", (o * 10.0).round() as i32)
        }
        o if o < 1000.0 => {
            // 100-999 ohms: use value + 1 as multiplier
            format!("{:03}1", o.round() as i32)
        }
        o if o < 10000.0 => {
            // 1K-9.99K: divide by 10
            format!("{:03}2", (o / 10.0).round() as i32)
        }
        o if o < 100000.0 => {
            // 10K-99.9K: divide by 100
            format!("{:03}3", (o / 100.0).round() as i32)
        }
        o if o < 1000000.0 => {
            // 100K-999K: divide by 1000
            format!("{:03}4", (o / 1000.0).round() as i32)
        }
        _ => {
            // 1M and above: divide by 10000
            format!("{:03}5", (ohms / 10000.0).round() as i32)
        }
    }
}

fn generate_kicad_symbol_with_mfrs(
    name: &str,
    value: &str,
    footprint: &str,
    description: &str,
    manufacturers: &[ManufacturerPart],
) -> String {
    // Simplified - would generate full KiCad symbol with manufacturer fields
    format!("(symbol \"{}\" ...)", name)
}