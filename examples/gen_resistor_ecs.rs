extern crate component;
extern crate bevy_ecs;

use bevy_ecs::prelude::*;
use component::ecs::{components::*, resources::*, systems};

fn main() {
    println!("Atlantix EDA - Bevy ECS Resistor Generator Demo");
    
    // Create the ECS world
    let mut world = World::new();
    
    // Add resources
    world.insert_resource(GeneratorConfig {
        output_formats: vec![
            OutputFormat::KicadSymbols,
            OutputFormat::KicadFootprints,
            OutputFormat::Altium,
        ],
        manufacturers: vec!["Vishay".to_string(), "Yageo".to_string(), "KOA".to_string()],
        decades: vec![1, 10, 100, 1000, 10000, 100000],
    });
    world.insert_resource(ESeriesCache::default());
    
    // Spawn template entities for each package
    let packages = vec!["0603", "0805", "1206"];
    for package_name in packages {
        world.spawn((
            ESeries(96),
            Package {
                name: package_name.to_string(),
                imperial: package_name.to_string(),
                metric: get_metric_name(package_name),
            },
        ));
    }
    
    println!("Spawned {} package templates", world.query::<&Package>().iter(&world).count());
    
    // Create and run the generation schedule
    let mut schedule = Schedule::default();
    
    // Note: Systems run in the order they're added
    schedule.add_systems((
        systems::generate_eseries_values,
        systems::assign_package_attributes,
        systems::generate_manufacturer_parts,
    ));
    
    println!("Running generation pipeline...");
    schedule.run(&mut world);
    
    // Run the assignment and manufacturer systems again to ensure all data is filled
    // (This is a workaround for the ordering issue with spawned entities)
    let mut post_generation_schedule = Schedule::default();
    post_generation_schedule.add_systems((
        systems::assign_package_attributes,
        systems::generate_manufacturer_parts,
    ));
    post_generation_schedule.run(&mut world);
    
    // Query results
    let resistor_count = world.query::<&ResistorValue>().iter(&world).count();
    println!("Generated {} resistors", resistor_count);
    
    // Show a sample of generated resistors
    println!("\nSample resistors:");
    let mut query = world.query::<(&PartNumber, &Description, &ManufacturerParts)>();
    for (i, (part_num, desc, mfrs)) in query.iter(&world)
        .filter(|(pn, _, _)| pn.0.contains("100") || pn.0.contains("1.00K") || pn.0.contains("10.0K"))
        .enumerate() {
        if i >= 3 { break; }
        println!("  {}: {}", part_num.0, desc.0);
        if mfrs.0.is_empty() {
            println!("    No manufacturers assigned");
        } else {
            for mfr in &mfrs.0 {
                println!("    - {}: {} ({})", mfr.manufacturer, mfr.mpn, mfr.distributor_pn);
            }
        }
    }
    
    // Debug: Check manufacturer generation
    let with_manufacturers = world.query::<&ManufacturerParts>()
        .iter(&world)
        .filter(|mfrs| !mfrs.0.is_empty())
        .count();
    println!("\nDebug: {} resistors have manufacturers assigned", with_manufacturers);
    
    // Demonstrate the power of ECS queries
    println!("\nECS Query Examples:");
    
    // Query all 1% resistors
    let mut tolerance_query = world.query_filtered::<&PartNumber, With<Tolerance>>();
    let one_percent_count = tolerance_query.iter(&world)
        .filter(|_| true) // In real impl, would check tolerance value
        .count();
    println!("  1% tolerance resistors: {}", one_percent_count);
    
    // Query all 0603 resistors
    let package_0603_count = world.query::<(&Package, &ResistorValue)>()
        .iter(&world)
        .filter(|(pkg, _)| pkg.name == "0603")
        .count();
    println!("  0603 package resistors: {}", package_0603_count);
    
    // Query resistors with manufacturer alternates
    let yageo_count = world.query::<&ManufacturerParts>()
        .iter(&world)
        .filter(|mfrs| mfrs.0.iter().any(|m| m.manufacturer == "Yageo"))
        .count();
    println!("  Resistors with Yageo alternates: {}", yageo_count);
    
    let koa_count = world.query::<&ManufacturerParts>()
        .iter(&world)
        .filter(|mfrs| mfrs.0.iter().any(|m| m.manufacturer == "KOA Speer"))
        .count();
    println!("  Resistors with KOA Speer alternates: {}", koa_count);
}

fn get_metric_name(package: &str) -> String {
    match package {
        "0402" => "1005Metric",
        "0603" => "1608Metric",
        "0805" => "2012Metric",
        "1206" => "3216Metric",
        "1210" => "3225Metric",
        "2512" => "6332Metric",
        _ => "UnknownMetric",
    }.to_string()
}