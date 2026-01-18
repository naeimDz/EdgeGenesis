use bevy::prelude::*;
use std::collections::HashMap;

/// Battery component - stores energy level in Wh (Watt-hours)
/// Range: 0.0 to 50.0 Wh (represents real Raspberry Pi 4 battery packs)
#[derive(Component)]
pub struct Battery(pub f32);

/// Model type enum - maps to model names in CSV
/// Each variant corresponds to a row in power_profiles CSV
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ModelType {
    YOLOv8Nano,
    YOLOv8Small,
    MobileNetV2,
    EfficientNetB0,
    TinyBERT,
    EfficientNetB1,
    MobileNetV3Small,
    DistilBERT,
}

impl ModelType {
    /// Get the CSV model name for lookup
    pub fn csv_name(&self) -> &'static str {
        match self {
            ModelType::YOLOv8Nano => "YOLOv8-nano",
            ModelType::YOLOv8Small => "YOLOv8-small",
            ModelType::MobileNetV2 => "MobileNetV2",
            ModelType::EfficientNetB0 => "EfficientNetB0",
            ModelType::TinyBERT => "TinyBERT",
            ModelType::EfficientNetB1 => "EfficientNetB1",
            ModelType::MobileNetV3Small => "MobileNetV3-Small",
            ModelType::DistilBERT => "DistilBERT",
        }
    }

    /// Get visual radius for rendering
    pub fn radius(&self) -> f32 {
        match self {
            ModelType::YOLOv8Nano | ModelType::MobileNetV3Small => 5.0,
            ModelType::MobileNetV2 => 8.0,
            ModelType::EfficientNetB0 | ModelType::EfficientNetB1 => 10.0,
            ModelType::YOLOv8Small => 12.0,
            ModelType::TinyBERT | ModelType::DistilBERT => 15.0,
        }
    }

    /// Get all model types for random selection
    pub fn all() -> &'static [ModelType] {
        &[
            ModelType::YOLOv8Nano,
            ModelType::YOLOv8Small,
            ModelType::MobileNetV2,
            ModelType::EfficientNetB0,
            ModelType::TinyBERT,
            ModelType::EfficientNetB1,
            ModelType::MobileNetV3Small,
            ModelType::DistilBERT,
        ]
    }
}

/// Gene component - contains the genetic configuration of a node
#[derive(Component, Clone, Copy, Debug)]
pub struct Gene {
    /// Model type selection - determines power/accuracy tradeoff
    pub model_type: ModelType,

    /// Inference frequency ratio (0.0 - 1.0)
    /// How often to perform inference: 1.0 = continuous, 0.5 = every other cycle
    pub inference_frequency: f32,

    /// Solar panel efficiency multiplier (0.8 - 1.2)
    /// Represents panel quality/orientation adaptation
    pub solar_efficiency_factor: f32,
}

/// Survival score - tracks fitness metric
#[derive(Component, Clone, Copy)]
pub struct SurvivalScore(pub f32);

/// Status component - indicates if the node is alive or dead
#[derive(Component, PartialEq, Eq)]
pub enum Status {
    Alive,
    Dead,
}

/// Bundle representing a complete Edge Node entity
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

/// Resource to track global simulation metrics
#[derive(Resource)]
pub struct SimulationMetrics {
    /// Total energy consumed by all nodes (Wh)
    pub total_energy_consumed: f32,

    /// Total energy harvested from solar (Wh)
    pub total_energy_harvested: f32,

    /// Number of successful inferences completed
    pub total_inferences: u64,

    /// Average node lifetime in seconds
    pub avg_node_lifetime: f32,

    /// Current time of day (0-24)
    pub current_hour: f32,

    /// Current generation/epoch
    pub generation: u32,
}

impl Default for SimulationMetrics {
    fn default() -> Self {
        Self {
            total_energy_consumed: 0.0,
            total_energy_harvested: 0.0,
            total_inferences: 0,
            avg_node_lifetime: 0.0,
            current_hour: 6.0, // Start at dawn
            generation: 0,
        }
    }
}

/// Resource holding loaded power profiles from CSV
#[derive(Resource)]
pub struct LoadedPowerProfiles(pub HashMap<String, crate::data_loader::PowerProfile>);

/// Resource holding loaded solar profiles from CSV
#[derive(Resource)]
pub struct LoadedSolarProfiles(pub Vec<crate::data_loader::SolarProfile>);
