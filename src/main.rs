mod components;
mod data_loader;
mod models;
mod systems;

use bevy::prelude::*;
use components::EpochCount;

fn main() {
    let mut app = App::new();

    // Load CSV data (optional overrides)
    let power_csv = data_loader::load_power_profiles("data/power_profiles/raspberry_pi_4.csv")
        .ok()
        .map(|profiles| {
            let mut map = std::collections::HashMap::new();
            for p in profiles {
                println!(
                    "📦 CSV Override: {} ({}W)",
                    p.model_name, p.inference_power_w
                );
                map.insert(p.model_name.clone(), p);
            }
            map
        });

    let solar_profiles = data_loader::load_solar_profiles("data/solar_profiles/algiers_solar.csv")
        .unwrap_or_else(|e| {
            eprintln!("⚠️ Solar CSV not found: {}. Using synthetic data.", e);
            Vec::new()
        });

    println!("☀️ Loaded {} solar hours", solar_profiles.len());
    println!("🧬 Using models.rs as primary data source");

    app.add_plugins(DefaultPlugins)
        .insert_resource(EpochCount(1))
        .insert_resource(components::PowerOverrides(power_csv))
        .insert_resource(components::LoadedSolarProfiles(solar_profiles));

    systems::register_systems(&mut app);

    app.run();
}
