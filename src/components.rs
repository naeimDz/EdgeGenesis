use crate::models::RealModelType;
use bevy::prelude::*;
use std::collections::HashMap;

/// Battery component - stores energy level in Wh (Watt-hours)
#[derive(Component)]
pub struct Battery(pub f32);

/// Gene component - evolutionary configuration
#[derive(Component, Clone, Copy, Debug)]
pub struct Gene {
    /// Model type from models.rs (type-safe, documented)
    pub model_type: RealModelType,

    /// Inference frequency ratio (0.0 - 1.0)
    pub inference_frequency: f32,

    /// Solar panel efficiency multiplier (0.8 - 1.2)
    pub solar_efficiency_factor: f32,
}

/// Survival score - fitness metric
#[derive(Component, Clone, Copy)]
pub struct SurvivalScore(pub f32);

/// Node status
#[derive(Component, PartialEq, Eq)]
pub enum Status {
    Alive,
    Dead,
}

/// Bundle for edge node entity
#[derive(Bundle)]
pub struct EdgeNodeBundle {
    pub battery: Battery,
    pub gene: Gene,
    pub survival_score: SurvivalScore,
    pub status: Status,
    pub transform: Transform,
}

#[derive(Resource)]
pub struct EpochCount(pub u32);

/// Global simulation metrics
#[derive(Resource)]
pub struct SimulationMetrics {
    pub total_energy_consumed: f32,
    pub total_energy_harvested: f32,
    pub total_inferences: u64,

    #[allow(dead_code)]
    pub avg_node_lifetime: f32,

    pub current_hour: f32,
    pub generation: u32,
}

impl Default for SimulationMetrics {
    fn default() -> Self {
        Self {
            total_energy_consumed: 0.0,
            total_energy_harvested: 0.0,
            total_inferences: 0,
            avg_node_lifetime: 0.0,
            current_hour: 6.0,
            generation: 0,
        }
    }
}

/// Optional CSV overrides for power data
#[derive(Resource)]
pub struct PowerOverrides(pub Option<HashMap<String, crate::data_loader::PowerProfileCSV>>);

/// Solar profile data
#[derive(Resource)]
pub struct LoadedSolarProfiles(pub Vec<crate::data_loader::SolarProfile>);
