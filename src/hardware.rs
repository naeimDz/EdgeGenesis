use bevy::prelude::*;
use serde::Deserialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub enum HardwareType {
    ESP32,
    RaspberryPi4,
    JetsonNano,
}

/// Hardware specification component
#[derive(Component, Debug, Clone, Copy)]
pub struct HardwareSpec {
    pub hardware_type: HardwareType,
    pub battery_capacity_wh: f32,
    pub idle_power_w: f32,
    pub max_solar_input_w: f32,
}

impl HardwareSpec {
    pub fn new(hw_type: HardwareType) -> Self {
        match hw_type {
            HardwareType::ESP32 => Self {
                hardware_type: HardwareType::ESP32,
                battery_capacity_wh: 1.5, // Tiny LiPo/Capacitor
                idle_power_w: 0.1,        // Ultra-low power
                max_solar_input_w: 2.0,   // Tiny 2W panel
            },
            HardwareType::RaspberryPi4 => Self {
                hardware_type: HardwareType::RaspberryPi4,
                battery_capacity_wh: 11.1, // UPS HAT
                idle_power_w: 2.5,         // Standard idle
                max_solar_input_w: 20.0,   // 20W Panel
            },
            HardwareType::JetsonNano => Self {
                hardware_type: HardwareType::JetsonNano,
                battery_capacity_wh: 20.0, // Larger battery
                idle_power_w: 5.0,         // GPU idle
                max_solar_input_w: 40.0,   // 40W Panel
            },
        }
    }

    pub fn name(&self) -> &'static str {
        match self.hardware_type {
            HardwareType::ESP32 => "ESP32",
            HardwareType::RaspberryPi4 => "RPi4",
            HardwareType::JetsonNano => "Jetson",
        }
    }
}
