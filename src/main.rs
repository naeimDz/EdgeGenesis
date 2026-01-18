mod components;
mod data_loader;
mod models;
mod systems;

use bevy::prelude::*;
use components::EpochCount;

fn main() {
    let mut app = App::new();

    // Load power profiles from CSV
    let power_profiles_vec = data_loader::load_power_profiles(
        "data/power_profiles/raspberry_pi_4.csv",
    )
    .unwrap_or_else(|e| {
        eprintln!("⚠️ Could not load power profiles: {}. Using defaults.", e);
        Vec::new()
    });

    // Load solar profiles from CSV
    let solar_profiles_vec = data_loader::load_solar_profiles(
        "data/solar_profiles/algiers_solar.csv",
    )
    .unwrap_or_else(|e| {
        eprintln!("⚠️ Could not load solar profiles: {}. Using defaults.", e);
        Vec::new()
    });

    // Convert power profiles to HashMap for fast lookup by model name
    let mut power_map = std::collections::HashMap::new();
    for p in power_profiles_vec {
        println!(
            "📦 Loaded model: {} ({}W inference, {}% accuracy)",
            p.model_name, p.inference_power_w, p.accuracy_percent
        );
        power_map.insert(p.model_name.clone(), p);
    }

    println!("☀️ Loaded {} solar profile hours", solar_profiles_vec.len());

    app.add_plugins(DefaultPlugins)
        .insert_resource(EpochCount(1))
        .insert_resource(components::LoadedPowerProfiles(power_map))
        .insert_resource(components::LoadedSolarProfiles(solar_profiles_vec));

    systems::register_systems(&mut app);

    app.run();
}
