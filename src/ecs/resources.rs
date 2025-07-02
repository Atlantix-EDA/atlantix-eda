use bevy_ecs::prelude::*;

/// Global configuration for the generator
#[derive(Resource, Debug, Clone)]
pub struct GeneratorConfig {
    pub output_formats: Vec<OutputFormat>,
    pub manufacturers: Vec<String>,
    pub decades: Vec<u32>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum OutputFormat {
    Altium,
    KicadSymbols,
    KicadFootprints,
}

impl Default for GeneratorConfig {
    fn default() -> Self {
        Self {
            output_formats: vec![OutputFormat::KicadSymbols, OutputFormat::KicadFootprints],
            manufacturers: vec!["Vishay".to_string()],
            decades: vec![1, 10, 100, 1000, 10000, 100000],
        }
    }
}

/// Cache for E-series values to avoid recalculation
#[derive(Resource, Debug, Default)]
pub struct ESeriesCache {
    pub cache: std::collections::HashMap<usize, Vec<f64>>,
}

impl ESeriesCache {
    pub fn get_or_calculate(&mut self, series: usize) -> Vec<f64> {
        self.cache.entry(series).or_insert_with(|| {
            let mut values = vec![0.0; series];
            for index in 0..series {
                let gamma: f64 = f64::powf(10.0, index as f64 / series as f64);
                values[index] = (gamma * 100.0).round() / 100.0;
            }
            values
        }).clone()
    }
}