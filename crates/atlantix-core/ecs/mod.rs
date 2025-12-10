pub mod components;
pub mod systems;
pub mod resources;

use bevy_ecs::prelude::*;

/// Initialize the ECS world with default systems
pub fn build_resistor_world() -> World {
    let mut world = World::new();
    
    // Register resources
    world.insert_resource(resources::GeneratorConfig::default());
    
    world
}

/// Run the resistor generation pipeline
pub fn run_generation_pipeline(world: &mut World) {
    let mut schedule = Schedule::default();
    
    // Add systems in order
    schedule.add_systems((
        systems::generate_eseries_values,
        systems::assign_package_attributes,
        systems::calculate_tolerances,
        systems::generate_manufacturer_parts,
        systems::format_outputs,
    ));
    
    schedule.run(world);
}