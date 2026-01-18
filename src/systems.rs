/// Bevy systems for simulating solar-powered edge AI devices
/// All power/solar data is read from CSV files at runtime
use bevy::prelude::*;
use bevy::time::common_conditions::on_timer;
use rand::prelude::*;
use std::time::Duration;

use crate::components::*;
use crate::hardware::{HardwareSpec, HardwareType};
use crate::policies::PowerPolicy;

const GRID_SIZE: i32 = 10;
const GRID_SPACING: f32 = 50.0;
// Note: BATTERY_CAPACITY and BASE_DRAIN are now per-node in HardwareSpec
const SOLAR_EFFICIENCY_PENALTY: f32 = 1.0; // Real efficiency
const SIMULATION_SPEEDUP: f32 = 180.0; // 1 real sec = 3 sim minutes

/// Setup camera
pub fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d::default());
}

/// Setup system - spawns initial population of edge nodes
pub fn setup_grid(mut commands: Commands) {
    let mut rng = rand::rng();
    let offset = (GRID_SIZE as f32 * GRID_SPACING) / 2.0;

    // All available models from models.rs
    let all_models = [
        crate::models::RealModelType::YOLOv8Nano,
        crate::models::RealModelType::YOLOv8Small,
        crate::models::RealModelType::MobileNetV2,
        crate::models::RealModelType::EfficientNetB0,
        crate::models::RealModelType::TinyBERT,
        crate::models::RealModelType::EfficientNetB1,
        crate::models::RealModelType::MobileNetV3Small,
        crate::models::RealModelType::DistilBERT,
    ];

    for x in 0..GRID_SIZE {
        for y in 0..GRID_SIZE {
            let model_type = all_models[rng.random_range(0..all_models.len())];

            let gene = Gene {
                model_type,
                inference_frequency: rng.random_range(0.3..1.0),
                solar_efficiency_factor: rng.random_range(0.8..1.2),
                // Assign random policy initially
                policy: match rng.random_range(0..3) {
                    0 => PowerPolicy::Aggressive,
                    1 => PowerPolicy::Conservative,
                    _ => PowerPolicy::SmartAdaptive,
                },
            };

            // Assign Random Hardware
            let hw_type = match rng.random_range(0..3) {
                0 => HardwareType::ESP32,
                1 => HardwareType::JetsonNano,
                _ => HardwareType::RaspberryPi4,
            };
            let hardware = HardwareSpec::new(hw_type);

            commands.spawn(EdgeNodeBundle {
                battery: Battery(hardware.battery_capacity_wh * 0.8), // Start at 80%
                gene,
                hardware,
                survival_score: SurvivalScore(0.0),
                status: Status::Alive,
                transform: Transform::from_xyz(
                    x as f32 * GRID_SPACING - offset,
                    y as f32 * GRID_SPACING - offset,
                    0.0,
                ),
            });
        }
    }

    commands.insert_resource(SimulationMetrics::default());
    commands.insert_resource(EpochCount(0));
}

/// Physics system - uses models.rs with optional CSV overrides
pub fn resource_physics_system(
    time: Res<Time>,
    power_overrides: Res<PowerOverrides>,
    solar_profiles: Res<LoadedSolarProfiles>,
    mut metrics: ResMut<SimulationMetrics>,
    mut query: Query<(
        &mut Battery,
        &mut SurvivalScore,
        &mut Status,
        &Gene,
        &HardwareSpec,
    )>,
) {
    let mut rng = rand::rng();
    let dt = time.delta_secs();

    // Update simulation hour (synced with SIMULATION_SPEEDUP)
    metrics.current_hour = (metrics.current_hour + dt * SIMULATION_SPEEDUP / 3600.0) % 24.0;

    // Get solar output for current hour
    let current_hour_index = metrics.current_hour as usize % 24;
    let solar_output_w = solar_profiles
        .0
        .get(current_hour_index)
        .map(|p| p.power_output_100w_panel())
        .unwrap_or(0.0);

    for (mut battery, mut score, mut status, gene, hardware) in query.iter_mut() {
        if *status == Status::Dead {
            continue;
        }

        // Get power using hybrid system (CSV override or models.rs default)
        let (idle_power, inference_power) =
            crate::data_loader::get_model_power(gene.model_type, power_overrides.0.as_ref());

        // POLICY-BASED INFERENCE DECISION
        let should_infer =
            gene.policy
                .should_infer(battery.0, solar_output_w, gene.inference_frequency);

        let power_w = hardware.idle_power_w
            + if should_infer {
                inference_power
            } else {
                0.0 // Idle power is already added as baseline
            };

        // Solar recharge using CSV data (with harsh environment penalty)
        let recharge_w = solar_output_w * gene.solar_efficiency_factor * SOLAR_EFFICIENCY_PENALTY;
        let recharge_wh = (recharge_w * dt * SIMULATION_SPEEDUP) / 3600.0f32;
        battery.0 += recharge_wh;

        // Apply physics with time scaling
        let drain_wh = (power_w * dt * SIMULATION_SPEEDUP) / 3600.0f32;
        battery.0 -= drain_wh;

        // Track metrics
        metrics.total_energy_consumed += drain_wh;
        metrics.total_energy_harvested += recharge_wh;

        // Cap battery based on HARDWARE LIMIT
        battery.0 = battery.0.clamp(0.0, hardware.battery_capacity_wh);

        // Death condition
        if battery.0 <= 0.0 {
            if *status != Status::Dead {
                // println!("💀 Node died! (Battery depleted)"); // Optional: Uncomment for per-node death logs
                *status = Status::Dead;
            }
        } else {
            score.0 += dt;
            metrics.total_inferences += 1;
        }
    }
}

/// Rendering system - visualizes node state
pub fn render_nodes_system(
    mut gizmos: Gizmos,
    query: Query<(&Transform, &Battery, &Gene, &Status, &HardwareSpec)>,
) {
    for (transform, battery, gene, status, hardware) in query.iter() {
        let position = transform.translation.truncate();
        // Radius based on model size (larger models = bigger circles)
        let radius = (gene.model_type.size_mb() / 10.0).clamp(3.0, 20.0);

        let color = if *status == Status::Dead {
            Color::srgb(0.5, 0.5, 0.5) // Gray
        } else {
            let charge_ratio = (battery.0 / hardware.battery_capacity_wh).clamp(0.0, 1.0);
            if charge_ratio > 0.75 {
                Color::srgb(0.0, 1.0, 0.0) // Green
            } else if charge_ratio > 0.25 {
                Color::srgb(1.0, 1.0, 0.0) // Yellow
            } else {
                Color::srgb(1.0, 0.0, 0.0) // Red
            }
        };

        gizmos.circle_2d(position, radius, color);
    }
}

/// Genetic epoch system - evolutionary selection and mutation
pub fn genetic_epoch_system(
    mut commands: Commands,
    mut epoch_count: ResMut<EpochCount>,
    mut metrics: ResMut<SimulationMetrics>,
    query: Query<(Entity, &Status, &SurvivalScore, &Gene, &Battery)>,
) {
    let _simulated_hours_passed = (epoch_count.0 as f32 * 30.0) / 60.0; // Assuming 1 real sec = 1 sim minute

    // Calculate average battery level
    let total_battery: f32 = query.iter().map(|(_, _, _, _, battery)| battery.0).sum();
    let avg_battery = if !query.is_empty() {
        total_battery / query.iter().count() as f32
    } else {
        0.0
    };

    println!("\n=== EPOCH {} ===", epoch_count.0);
    println!("⏰ Simulated Time: {:.1} hours", metrics.current_hour); // Current hour of day
    println!(
        "🔋 Avg Energy Consumed (Epoch): {:.2} Wh",
        metrics.total_energy_consumed / 100.0
    );
    println!("⚡ Avg Battery Level: {:.2} Wh", avg_battery);

    // Reset epoch metrics
    metrics.total_energy_consumed = 0.0;
    metrics.total_energy_harvested = 0.0;

    epoch_count.0 += 1;
    metrics.generation = epoch_count.0;

    let mut survivors: Vec<(f32, Gene)> = Vec::new();
    let mut entities_to_despawn = Vec::new();

    for (entity, status, score, gene, _battery) in query.iter() {
        entities_to_despawn.push(entity);
        if *status != Status::Dead {
            survivors.push((score.0, *gene));
        }
    }

    // Despawn all entities
    for entity in entities_to_despawn {
        commands.entity(entity).despawn();
    }

    if survivors.is_empty() {
        println!("🔴 EXTINCTION - Reseeding");
        setup_grid(commands);
        return;
    }

    // Sort by fitness (descending)
    survivors.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());

    // Elite selection: top 15%
    let elite_count = (survivors.len() as f32 * 0.15).ceil() as usize;
    let elites = &survivors[0..elite_count.max(1)];

    // --- DETAILED REPORTING START ---

    // 1. Dominant Model (Most Common)
    let mut model_counts: std::collections::HashMap<String, usize> =
        std::collections::HashMap::new();
    for (_, gene) in &survivors {
        *model_counts
            .entry(gene.model_type.name().to_string())
            .or_insert(0) += 1;
    }
    let dominant_model = model_counts.iter().max_by_key(|&(_, count)| count).unwrap();

    // 2. Elite Model (Highest Accuracy amongst survivors)
    let best_accuracy_survivor = survivors
        .iter()
        .max_by_key(|(_, gene)| (gene.model_type.accuracy_percent() * 100.0) as u32)
        .unwrap();

    // 3. Fittest Model (Longest Survival Duration) - already sorted in elites[0]
    let fittest_gene = &elites[0].1;

    println!("📊 Population: {} alive", survivors.len());

    // Report 1: The "King of the Jungle" (Most Numerous)
    println!(
        "🦁 Dominant Model: {} (Count: {}/{})",
        dominant_model.0,
        dominant_model.1,
        survivors.len()
    );

    // Report 1.5: Dominant Policy
    let mut policy_counts: std::collections::HashMap<String, usize> =
        std::collections::HashMap::new();
    for (_, gene) in &survivors {
        *policy_counts
            .entry(gene.policy.name().to_string())
            .or_insert(0) += 1;
    }
    if let Some(dom_policy) = policy_counts.iter().max_by_key(|&(_, count)| count) {
        println!(
            "📜 Dominant Policy: {} (Count: {})",
            dom_policy.0, dom_policy.1
        );
    }

    // Report 2: The "Scholar" (Highest Accuracy Survivor)
    println!(
        "🧠 Smartest Survivor: {} ({:.1}% acc)",
        best_accuracy_survivor.1.model_type.name(),
        best_accuracy_survivor.1.model_type.accuracy_percent()
    );

    // Report 3: The "Survivor" (Top Fitness Score)
    println!(
        "🏆 Top Fitness Specimen: {} (Score: {:.2}s)",
        fittest_gene.model_type.name(),
        elites[0].0
    );

    println!(
        "📉 Avg Generation Fitness: {:.2}s",
        survivors.iter().map(|(f, _)| f).sum::<f32>() / survivors.len() as f32
    );
    // --- DETAILED REPORTING END ---

    // Repopulation with mutation
    let mut rng = rand::rng();
    let offset = (GRID_SIZE as f32 * GRID_SPACING) / 2.0;
    let all_models = [
        crate::models::RealModelType::YOLOv8Nano,
        crate::models::RealModelType::YOLOv8Small,
        crate::models::RealModelType::MobileNetV2,
        crate::models::RealModelType::EfficientNetB0,
        crate::models::RealModelType::TinyBERT,
        crate::models::RealModelType::EfficientNetB1,
        crate::models::RealModelType::MobileNetV3Small,
        crate::models::RealModelType::DistilBERT,
    ];

    for x in 0..GRID_SIZE {
        for y in 0..GRID_SIZE {
            let parent = &elites[rng.random_range(0..elites.len())].1;
            let mut new_gene = *parent;

            // Mutation 1: Inference frequency (±10%)
            new_gene.inference_frequency =
                (new_gene.inference_frequency + rng.random_range(-0.1..0.1)).clamp(0.1, 1.0);

            // Mutation 1.5: Policy Switch (5% chance)
            if rng.random_bool(0.05) {
                new_gene.policy = match rng.random_range(0..3) {
                    0 => PowerPolicy::Aggressive,
                    1 => PowerPolicy::Conservative,
                    _ => PowerPolicy::SmartAdaptive,
                };
            }

            // Mutation 2: Solar efficiency (±5%)
            new_gene.solar_efficiency_factor =
                (new_gene.solar_efficiency_factor + rng.random_range(-0.05..0.05)).clamp(0.7, 1.3);

            // Mutation 3: Model type (10% chance)
            if rng.random_bool(0.10) {
                new_gene.model_type = all_models[rng.random_range(0..all_models.len())];
            }

            // Assign Random Hardware for new generation
            let hw_type = match rng.random_range(0..3) {
                0 => HardwareType::ESP32,
                1 => HardwareType::JetsonNano,
                _ => HardwareType::RaspberryPi4,
            };
            let new_hardware = HardwareSpec::new(hw_type);

            commands.spawn(EdgeNodeBundle {
                battery: Battery(new_hardware.battery_capacity_wh * 0.8),
                gene: new_gene,
                hardware: new_hardware,
                survival_score: SurvivalScore(0.0),
                status: Status::Alive,
                transform: Transform::from_xyz(
                    x as f32 * GRID_SPACING - offset,
                    y as f32 * GRID_SPACING - offset,
                    0.0,
                ),
            });
        }
    }

    println!("✅ New generation spawned ({})", GRID_SIZE * GRID_SIZE);
}

/// Register all systems with Bevy app
pub fn register_systems(app: &mut App) {
    app.add_systems(Startup, (setup_camera, setup_grid))
        .add_systems(
            Update,
            (
                resource_physics_system,
                render_nodes_system,
                genetic_epoch_system.run_if(on_timer(Duration::from_secs(30))),
            ),
        );
}
