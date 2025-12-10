use bevy_ecs::prelude::*;

// Core resistor components
#[derive(Component, Debug, Clone)]
pub struct ResistorValue {
    pub ohms: f64,
    pub formatted: String,  // "1.33K", "100", etc.
}

#[derive(Component, Debug, Clone, Copy)]
pub struct ESeries(pub usize);  // 24, 48, 96, 192

#[derive(Component, Debug, Clone)]
pub struct Package {
    pub name: String,       // "0603", "0805", etc.
    pub imperial: String,   // "0603"
    pub metric: String,     // "1608Metric"
}

#[derive(Component, Debug, Clone)]
pub struct Tolerance(pub String);  // "1%", "2%", "5%"

#[derive(Component, Debug, Clone)]
pub struct PowerRating(pub String);  // "1/10W", "1/4W"

// Manufacturer components
#[derive(Component, Debug, Clone)]
pub enum Manufacturer {
    Vishay,
    Yageo,
    KoaSpeer,
    Stackpole,
    Panasonic,
}

#[derive(Component, Debug, Clone)]
pub struct ManufacturerPart {
    pub manufacturer: String,
    pub mpn: String,              // Manufacturer Part Number
    pub distributor: String,      // "Digikey", "Mouser"
    pub distributor_pn: String,   // Distributor Part Number
}

// Allow multiple manufacturers per resistor
#[derive(Component, Debug, Clone, Default)]
pub struct ManufacturerParts(pub Vec<ManufacturerPart>);

// Output format components
#[derive(Component, Debug, Clone)]
pub struct AltiumData {
    pub csv_line: String,
}

#[derive(Component, Debug, Clone)]
pub struct KicadSymbol {
    pub name: String,
    pub symbol_content: String,
}

#[derive(Component, Debug, Clone)]
pub struct KicadFootprint {
    pub name: String,
    pub footprint_content: String,
}

// Metadata components
#[derive(Component, Debug, Clone)]
pub struct Description(pub String);  // "RES SMT 1.33Kohms, 0603, 1%, 1/10W"

#[derive(Component, Debug, Clone)]
pub struct PartNumber(pub String);  // "R0603_1.33K"

// Bundle for a complete resistor
#[derive(Bundle)]
pub struct ResistorBundle {
    pub value: ResistorValue,
    pub package: Package,
    pub tolerance: Tolerance,
    pub power: PowerRating,
    pub description: Description,
    pub part_number: PartNumber,
    pub manufacturers: ManufacturerParts,
}