/// Bevy systems for simulating solar-powered edge AI devices
/// All power/solar data is read from CSV files at runtime
use bevy::prelude::*;
use bevy::time::common_conditions::on_timer;
use rand::prelude::*;
use std::time::Duration;

use crate::components::*;

const GRID_SIZE: i32 = 10;
const GRID_SPACING: f32 = 50.0;
const BATTERY_CAPACITY_WH: f32 = 5.0; // Small capacity (requires management)
const SOLAR_EFFICIENCY_PENALTY: f32 = 0.15; // Dim light (15% efficiency)
const BASE_POWER_DRAIN_W: f32 = 1.0; // Low base drain (model choice matters)

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
            };

            commands.spawn(EdgeNodeBundle {
                battery: Battery(BATTERY_CAPACITY_WH * 0.8), // Start at 80%
                gene,
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
    mut query: Query<(&mut Battery, &mut SurvivalScore, &mut Status, &Gene)>,
) {
    let mut rng = rand::rng();
    let dt = time.delta_secs();

    // Update simulation hour
    metrics.current_hour = (metrics.current_hour + dt / 60.0) % 24.0;

    // Get solar output for current hour
    let current_hour_index = metrics.current_hour as usize % 24;
    let solar_output_w = solar_profiles
        .0
        .get(current_hour_index)
        .map(|p| p.power_output_100w_panel())
        .unwrap_or(0.0);

    for (mut battery, mut score, mut status, gene) in query.iter_mut() {
        if *status == Status::Dead {
            continue;
        }

        // Get power using hybrid system (CSV override or models.rs default)
        let (idle_power, inference_power) =
            crate::data_loader::get_model_power(gene.model_type, power_overrides.0.as_ref());

        let power_w = BASE_POWER_DRAIN_W
            + if rng.random_bool(gene.inference_frequency as f64) {
                inference_power
            } else {
                idle_power
            };

        let drain_wh = (power_w * dt) / 3600.0f32;
        battery.0 -= drain_wh;

        // Solar recharge using CSV data (with harsh environment penalty)
        let recharge_w = solar_output_w * gene.solar_efficiency_factor * SOLAR_EFFICIENCY_PENALTY;
        let recharge_wh = (recharge_w * dt) / 3600.0f32;
        battery.0 += recharge_wh;

        // Track metrics
        metrics.total_energy_consumed += drain_wh;
        metrics.total_energy_harvested += recharge_wh;

        // Cap battery
        battery.0 = battery.0.clamp(0.0, BATTERY_CAPACITY_WH);

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
    query: Query<(&Transform, &Battery, &Gene, &Status)>,
) {
    for (transform, battery, gene, status) in query.iter() {
        let position = transform.translation.truncate();
        // Radius based on model size (larger models = bigger circles)
        let radius = (gene.model_type.size_mb() / 10.0).clamp(3.0, 20.0);

        let color = if *status == Status::Dead {
            Color::srgb(0.5, 0.5, 0.5) // Gray
        } else {
            let charge_ratio = (battery.0 / BATTERY_CAPACITY_WH).clamp(0.0, 1.0);
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
    query: Query<(Entity, &Status, &SurvivalScore, &Gene)>,
) {
    println!("\n=== EPOCH {} ===", epoch_count.0);
    epoch_count.0 += 1;
    metrics.generation = epoch_count.0;

    let mut survivors: Vec<(f32, Gene)> = Vec::new();
    let mut entities_to_despawn = Vec::new();

    for (entity, status, score, gene) in query.iter() {
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

    // Print best gene info using RealModelType methods
    let best_gene = &elites[0].1;
    let best_model_name = best_gene.model_type.name();
    let accuracy = best_gene.model_type.accuracy_percent();

    println!("📊 Population: {} alive", survivors.len());
    println!("🏆 Top fitness: {:.2}s", elites[0].0);
    println!(
        "📈 Best model: {} ({}% accuracy)",
        best_model_name, accuracy
    );
    println!(
        "Average fitness: {:.2}s",
        survivors.iter().map(|(f, _)| f).sum::<f32>() / survivors.len() as f32
    );

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

            // Mutation 2: Solar efficiency (±5%)
            new_gene.solar_efficiency_factor =
                (new_gene.solar_efficiency_factor + rng.random_range(-0.05..0.05)).clamp(0.7, 1.3);

            // Mutation 3: Model type (10% chance)
            if rng.random_bool(0.10) {
                new_gene.model_type = all_models[rng.random_range(0..all_models.len())];
            }

            commands.spawn(EdgeNodeBundle {
                battery: Battery(BATTERY_CAPACITY_WH * 0.8),
                gene: new_gene,
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
