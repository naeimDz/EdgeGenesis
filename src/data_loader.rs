/// Data loading module for CSV parsing of power and solar profiles
/// Used to load real-world measurement data for simulation
use serde::Deserialize;
use std::error::Error;
use std::fs::File;

/// Power profile for a specific ML model on Raspberry Pi 4
/// Based on real-world measurements from academic papers and benchmarks
#[derive(Debug, Deserialize, Clone)]
pub struct PowerProfile {
    /// Model name (e.g., "YOLOv8-nano", "MobileNetV2")
    pub model_name: String,

    /// Idle power consumption in Watts (no inference running)
    pub idle_power_w: f32,

    /// Peak power consumption during inference in Watts
    pub inference_power_w: f32,

    /// Average inference latency in milliseconds
    pub avg_inference_time_ms: f32,

    /// Model size in MB (weights + architecture)
    pub model_size_mb: f32,

    /// Model accuracy as percentage (ImageNet or task-specific)
    pub accuracy_percent: f32,

    /// Number of parameters in millions
    pub parameters_millions: f32,
}

impl PowerProfile {
    /// Calculate energy per inference in Joules
    /// Energy = Power (Watts) × Time (seconds)
    pub fn energy_per_inference_j(&self) -> f32 {
        let time_seconds = self.avg_inference_time_ms / 1000.0;
        self.inference_power_w * time_seconds
    }

    /// Calculate efficiency metric: accuracy per watt
    /// Higher is better (accuracy per unit energy)
    pub fn efficiency_ratio(&self) -> f32 {
        self.accuracy_percent / self.inference_power_w
    }
}

/// Solar irradiance profile for a specific hour of the day
/// Matches CSV: hour,avg_irradiance_w_m2,panel_efficiency
#[derive(Debug, Deserialize, Clone)]
pub struct SolarProfile {
    /// Hour of day (0-23)
    pub hour: u8,

    /// Solar irradiance in W/m² (incident solar radiation)
    pub avg_irradiance_w_m2: f32,

    /// Panel efficiency as decimal (0.15-0.18 typical)
    pub panel_efficiency: f32,
}

impl SolarProfile {
    /// Calculate power output from a 100W rated panel
    /// Output = Irradiance × Panel_Area × Efficiency
    /// For 100W panel at 1000 W/m² STC, area ≈ 0.6 m²
    pub fn power_output_100w_panel(&self) -> f32 {
        let panel_area_m2 = 0.6; // Typical 100W panel
        self.avg_irradiance_w_m2 * panel_area_m2 * self.panel_efficiency
    }
}

/// Load power profiles from CSV file
pub fn load_power_profiles(path: &str) -> Result<Vec<PowerProfile>, Box<dyn Error>> {
    let file = File::open(path)?;
    let mut reader = csv::Reader::from_reader(file);
    let mut profiles = Vec::new();

    for result in reader.deserialize() {
        let profile: PowerProfile = result?;
        profiles.push(profile);
    }

    Ok(profiles)
}

/// Load solar profiles from CSV file
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_power_profile_energy_calculation() {
        let profile = PowerProfile {
            model_name: "YOLOv8-nano".to_string(),
            idle_power_w: 2.5,
            inference_power_w: 4.2,
            avg_inference_time_ms: 45.0,
            model_size_mb: 6.0,
            accuracy_percent: 80.4,
            parameters_millions: 3.2,
        };

        // 4.2W × 0.045s = 0.189J per inference
        let energy_j = profile.energy_per_inference_j();
        assert!((energy_j - 0.189).abs() < 0.001);
    }

    #[test]
    fn test_solar_profile_power_output() {
        let profile = SolarProfile {
            hour: 12,
            avg_irradiance_w_m2: 800.0,
            panel_efficiency: 0.18,
        };

        // 800 × 0.6 × 0.18 = 86.4W
        let output = profile.power_output_100w_panel();
        assert!((output - 86.4).abs() < 0.1);
    }
}
