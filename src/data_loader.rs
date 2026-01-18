use crate::models::RealModelType;
/// Data loading module for CSV parsing of power and solar profiles
/// CSV data is used to OVERRIDE defaults from models.rs when available
use serde::Deserialize;
use std::error::Error;
use std::fs::File;

/// Power profile loaded from CSV - optional override for models.rs defaults
#[derive(Debug, Deserialize, Clone)]
#[allow(dead_code)]
pub struct PowerProfileCSV {
    pub model_name: String,
    pub idle_power_w: f32,
    pub inference_power_w: f32,
    pub avg_inference_time_ms: f32,
    pub model_size_mb: f32,
    pub accuracy_percent: f32,
    pub parameters_millions: f32,
}

/// Solar irradiance profile for a specific hour
#[derive(Debug, Deserialize, Clone)]
#[allow(dead_code)]
pub struct SolarProfile {
    pub hour: u8,
    pub avg_irradiance_w_m2: f32,
    pub panel_efficiency: f32,
}

impl SolarProfile {
    /// Calculate power output from a 20W rated panel (IoT Scale)
    pub fn power_output_100w_panel(&self) -> f32 {
        let panel_area_m2 = 0.12; // Small IoT Panel (~20W)
        self.avg_irradiance_w_m2 * panel_area_m2 * self.panel_efficiency
    }
}

/// Helper to get power data with CSV override capability
pub fn get_model_power(
    model: RealModelType,
    csv_overrides: Option<&std::collections::HashMap<String, PowerProfileCSV>>,
) -> (f32, f32) {
    // Try CSV first
    if let Some(overrides) = csv_overrides {
        if let Some(csv_data) = overrides.get(model.name()) {
            return (csv_data.idle_power_w, csv_data.inference_power_w);
        }
    }

    // Fallback to models.rs (always reliable)
    // Standard RPi4 Idle
    (2.5, model.inference_power_w())
}

/// Load power profiles from CSV (optional)
pub fn load_power_profiles(path: &str) -> Result<Vec<PowerProfileCSV>, Box<dyn Error>> {
    let file = File::open(path)?;
    let mut reader = csv::Reader::from_reader(file);
    let mut profiles = Vec::new();

    for result in reader.deserialize() {
        let profile: PowerProfileCSV = result?;
        profiles.push(profile);
    }

    Ok(profiles)
}

/// Load solar profiles from CSV
pub fn load_solar_profiles(path: &str) -> Result<Vec<SolarProfile>, Box<dyn Error>> {
    let file = File::open(path)?;
    let mut reader = csv::Reader::from_reader(file);
    let mut profiles = Vec::new();

    for result in reader.deserialize() {
        let profile: SolarProfile = result?;
        profiles.push(profile);
    }

    Ok(profiles)
}
