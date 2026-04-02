use bevy::prelude::*;
use crate::components::*;
use crate::data::*;

/// Waits until the render surface is available (window has valid dimensions).
/// On Android, the native window is null until after Event::Resumed.
pub fn wait_for_window(
    windows: Query<&Window>,
    mut next_state: ResMut<NextState<crate::states::AppState>>,
    needs: Res<crate::resources::NeedsFreshSetup>,
) {
    for window in &windows {
        if window.physical_width() > 0 && window.physical_height() > 0 {
            if needs.0 {
                // Restarting or switching levels — go straight to Playing
                info!("Window ready, resuming Playing (fresh setup)");
                next_state.set(crate::states::AppState::Playing);
            } else {
                info!("Window ready ({}x{}), transitioning to MainMenu", window.physical_width(), window.physical_height());
                next_state.set(crate::states::AppState::MainMenu);
            }
            return;
        }
    }
}

/// Despawns all game-world entities so a fresh level can be set up.
/// Only despawns entities tagged with GameWorldEntity, not Bevy internals.
pub fn cleanup_game_world(
    mut commands: Commands,
    game_entities: Query<Entity, With<crate::components::GameWorldEntity>>,
    mut debug_state: ResMut<crate::systems::debug::DebugState>,
) {
    for entity in &game_entities {
        commands.entity(entity).despawn_recursive();
    }
    debug_state.show_overlay = false;
}

/// Runs once when entering Playing state.
/// Sets up the 3D scene: camera, lights, ground, path, build spots.
pub fn setup_level(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
    current_level: Res<crate::resources::CurrentLevel>,
    mut level_path_res: ResMut<crate::resources::LevelPath>,
    mut game_data: ResMut<crate::resources::GameData>,
    mut auto_wave: ResMut<crate::resources::AutoWave>,
    save_data: Option<Res<crate::save::SaveData>>,
) {
    // Reset auto-wave at the start of each new level
    auto_wave.enabled = false;
    auto_wave.countdown = 0.0;
    let level = current_level.0;
    let theme = level_theme(level);
    let config = level_start_config(level);

    // Apply WarChest bonus to starting gold
    let war_chest_level = if let Some(save) = save_data.as_ref() {
        let idx = crate::data::upgrade_index(crate::data::UpgradeKind::WarChest);
        if idx < save.upgrade_levels.len() { save.upgrade_levels[idx] } else { 0 }
    } else { 0 };
    let gold_bonus = 1.0 + 0.10 * war_chest_level as f32;

    // Populate GameData from level config
    game_data.gold = (config.starting_gold as f32 * gold_bonus) as u32;
    game_data.lives = config.lives;
    game_data.max_lives = config.lives;
    game_data.max_waves = config.max_waves;
    game_data.wave_number = 0;

    // Cache level path for other systems
    let path = level_path(level);
    level_path_res.0 = path.clone();

    use crate::components::GameWorldEntity;

    // --- Camera: 45-degree top-down view ---
    // Msaa::Off: MSAA causes panics/rendering failures on Android (Adreno GPUs)
    commands.spawn((
        Camera3d::default(),
        Msaa::Off,
        Transform::from_xyz(0.0, 20.0, 18.0).looking_at(Vec3::ZERO, Vec3::Y),
        bevy::core_pipeline::bloom::Bloom {
            intensity: 0.15,
            low_frequency_boost: 0.5,
            ..default()
        },
        // Bloom requires HDR tonemapping
        bevy::core_pipeline::tonemapping::Tonemapping::TonyMcMapface,
        GameWorldEntity,
    ));

    // --- Lighting ---
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 300.0,
        ..default()
    });

    commands.spawn((
        DirectionalLight {
            illuminance: 5000.0,
            // Shadow maps cause segfaults on Android — disable on mobile
            shadows_enabled: !cfg!(target_os = "android"),
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.8, 0.4, 0.0)),
        GameWorldEntity,
    ));

    // --- Sky color (clear color) ---
    commands.insert_resource(ClearColor(theme.sky));

    // --- Ground plane ---
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(50.0, 50.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: theme.ground,
            perceptual_roughness: 0.9,
            ..default()
        })),
        GameWorldEntity,
    ));

    // --- Path segments (stretched cubes along waypoints) ---
    let path_material = materials.add(StandardMaterial {
        base_color: theme.path,
        perceptual_roughness: 0.9,
        ..default()
    });

    for i in 0..path.len() - 1 {
        let start = path[i];
        let end = path[i + 1];
        let center = (start + end) / 2.0;
        let diff = end - start;
        let length = diff.length();
        // Rotate the box to align with the path direction
        let angle = diff.z.atan2(diff.x);

        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(length, 0.05, 1.5))),
            MeshMaterial3d(path_material.clone()),
            Transform::from_translation(center + Vec3::Y * 0.01)
                .with_rotation(Quat::from_rotation_y(-angle)),
            GameWorldEntity,
        ));
    }

    // --- Build spots (cylinders the player can click to place towers) ---
    let spots = level_build_spots(level);
    let spot_material = materials.add(StandardMaterial {
        base_color: Color::srgba(0.3, 0.3, 0.3, 0.8),
        perceptual_roughness: 0.5,
        ..default()
    });

    for (i, pos) in spots.iter().enumerate() {
        commands.spawn((
            Mesh3d(meshes.add(Cylinder::new(0.8, 0.1))),
            MeshMaterial3d(spot_material.clone()),
            Transform::from_translation(*pos + Vec3::Y * 0.06),
            BuildSpot {
                id: i,
                occupied: false,
            },
            GameWorldEntity,
        ));
    }

    // --- Environment scenery ---
    scatter_scenery(&mut commands, &asset_server, &path, &spots);
}

/// Scatters decorative models around the map, avoiding the path and build spots.
fn scatter_scenery(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    path: &[Vec3],
    build_spots: &[Vec3],
) {
    // Scenery definitions: (model_path, scale, count)
    let scenery = [
        ("models/environment/rock.glb#Scene0", 0.6, 12),
        ("models/environment/fern.glb#Scene0", 1.2, 10),
        ("models/environment/tree-palm.glb#Scene0", 0.8, 6),
    ];

    // Simple deterministic pseudo-random scatter using a seed
    let mut seed: u32 = 42;
    let next_rand = |s: &mut u32| -> f32 {
        *s = s.wrapping_mul(1103515245).wrapping_add(12345);
        ((*s >> 16) & 0x7FFF) as f32 / 32767.0
    };

    for (model_path, scale, count) in scenery {
        let scene = asset_server.load(model_path);
        for _ in 0..count {
            // Try to find a valid position (not too close to path or build spots)
            let mut attempts = 0;
            loop {
                attempts += 1;
                if attempts > 20 {
                    break;
                }

                let x = next_rand(&mut seed) * 40.0 - 20.0;
                let z = next_rand(&mut seed) * 30.0 - 15.0;
                let pos = Vec3::new(x, 0.0, z);

                // Check distance from path segments
                let mut too_close_to_path = false;
                for i in 0..path.len() - 1 {
                    let a = path[i];
                    let b = path[i + 1];
                    let ab = b - a;
                    let ap = pos - a;
                    let t = (ap.dot(ab) / ab.dot(ab)).clamp(0.0, 1.0);
                    let closest = a + ab * t;
                    if pos.distance(closest) < 2.5 {
                        too_close_to_path = true;
                        break;
                    }
                }
                if too_close_to_path {
                    continue;
                }

                // Check distance from build spots
                let too_close_to_spot = build_spots
                    .iter()
                    .any(|spot| pos.distance(*spot) < 2.5);
                if too_close_to_spot {
                    continue;
                }

                let rotation = Quat::from_rotation_y(next_rand(&mut seed) * std::f32::consts::TAU);
                let scale_vary = scale * (0.7 + next_rand(&mut seed) * 0.6);

                commands.spawn((
                    SceneRoot(scene.clone()),
                    Transform::from_translation(pos)
                        .with_rotation(rotation)
                        .with_scale(Vec3::splat(scale_vary)),
                    crate::components::GameWorldEntity,
                ));
                break;
            }
        }
    }
}
