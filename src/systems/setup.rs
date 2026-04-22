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
    // Reset sky color and ambient light so they don't bleed into menu screens
    commands.insert_resource(ClearColor(Color::srgb(0.05, 0.05, 0.1)));
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 100.0,
        ..default()
    });
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
    difficulty: Res<crate::resources::Difficulty>,
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
    let diff_gold = difficulty.gold_mult();
    let lives = difficulty.starting_lives();

    // Populate GameData from level config
    game_data.gold = (config.starting_gold as f32 * gold_bonus * diff_gold) as u32;
    game_data.lives = lives;
    game_data.max_lives = lives;
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
    scatter_scenery(&mut commands, &asset_server, &path, &spots, level);

    // --- Level landmarks ---
    spawn_landmarks(&mut commands, &asset_server, &mut meshes, &mut materials, level);
}

/// Spawns level-specific landmark assets (volcano, coliseum, castle) as background decoration.
fn spawn_landmarks(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    level: u32,
) {
    use crate::components::GameWorldEntity;

    struct Landmark {
        model: &'static str,
        scale: f32,
        pos: Vec3,
        rotation_y: f32,
        is_volcano: bool,
    }

    let landmarks: Vec<Landmark> = match level {
        // Prehistoric — large volcano with lava stream
        3 => vec![
            Landmark { model: "models/environment/volcano.glb#Scene0", scale: 10.5, pos: Vec3::new(-2.0, 0.0, -5.5), rotation_y: 0.3, is_volcano: true },
        ],
        4 => vec![
            Landmark { model: "models/environment/volcano.glb#Scene0", scale: 12.0, pos: Vec3::new(14.5, 0.0, -6.5), rotation_y: 0.1, is_volcano: true },
        ],
        // Stone Age — procedural mountains (handled below)
        5 | 6 => vec![],
        // Ancient — coliseum backdrop
        7 => vec![
            Landmark { model: "models/environment/coliseum.glb#Scene0", scale: 0.0056, pos: Vec3::new(-4.0, 0.0, -7.0), rotation_y: 0.0, is_volcano: false },
        ],
        8 => vec![
            Landmark { model: "models/environment/coliseum.glb#Scene0", scale: 0.004, pos: Vec3::new(-2.0, 0.0, 6.5), rotation_y: 0.0, is_volcano: false },
        ],
        // Medieval — castle backdrop
        9 => vec![
            Landmark { model: "models/environment/castle.glb#Scene0", scale: 2.0, pos: Vec3::new(20.0, 0.0, 2.5), rotation_y: 0.5, is_volcano: false },
        ],
        10 => vec![
            Landmark { model: "models/environment/castle.glb#Scene0", scale: 5.0, pos: Vec3::new(-4.5, 0.0, -9.5), rotation_y: -0.3, is_volcano: false },
        ],
        _ => vec![],
    };

    for lm in &landmarks {
        let mut entity_cmd = commands.spawn((
            SceneRoot(asset_server.load(lm.model)),
            Transform::from_translation(lm.pos)
                .with_rotation(Quat::from_rotation_y(lm.rotation_y))
                .with_scale(Vec3::splat(lm.scale)),
            GameWorldEntity,
        ));
        if lm.is_volcano {
            entity_cmd.insert(VolcanoModel);
        }
    }

    // Snow-capped mountains for stone age levels (5, 6)
    if level == 5 || level == 6 {
        let mountain_body = materials.add(StandardMaterial {
            base_color: Color::srgb(0.15, 0.18, 0.35),
            perceptual_roughness: 0.9,
            ..default()
        });
        let snow_cap = materials.add(StandardMaterial {
            base_color: Color::srgb(0.92, 0.95, 1.0),
            perceptual_roughness: 0.6,
            ..default()
        });

        // (position, body_height, body_radius, snow_height, snow_radius)
        let mountains: &[(Vec3, f32, f32, f32, f32)] = &[
            (Vec3::new(-18.0, 0.0, -14.0), 6.0, 3.0, 1.5, 1.3),
            (Vec3::new(-6.0, 0.0, -18.0), 8.0, 3.5, 2.0, 1.5),
            (Vec3::new(8.0, 0.0, -17.0), 7.0, 3.0, 1.8, 1.3),
            (Vec3::new(20.0, 0.0, -14.0), 5.0, 2.5, 1.3, 1.1),
        ];

        for (pos, body_h, body_r, snow_h, snow_r) in mountains {
            // Mountain body — dark blue cone
            commands.spawn((
                Mesh3d(meshes.add(Cone::new(*body_r, *body_h))),
                MeshMaterial3d(mountain_body.clone()),
                Transform::from_translation(*pos + Vec3::Y * body_h * 0.5),
                GameWorldEntity,
            ));
            // Snow cap — white cone on top
            commands.spawn((
                Mesh3d(meshes.add(Cone::new(*snow_r, *snow_h))),
                MeshMaterial3d(snow_cap.clone()),
                Transform::from_translation(*pos + Vec3::Y * (body_h - snow_h * 0.3)),
                GameWorldEntity,
            ));
        }
    }

    // Lava stream for prehistoric levels (3, 4) — flows from volcano base
    if level == 3 || level == 4 {
        let lava_material = materials.add(StandardMaterial {
            base_color: Color::srgb(0.9, 0.3, 0.0),
            emissive: bevy::color::LinearRgba::new(3.0, 0.8, 0.1, 1.0),
            unlit: true,
            ..default()
        });

        // Stream segments: series of stretched quads from volcano base outward
        // Each segment: (start_pos, end_pos, width)
        let stream_segments: &[(Vec3, Vec3, f32)] = if level == 3 {
            &[
                (Vec3::new(-2.0, 0.03, -3.0), Vec3::new(-2.5, 0.03, 2.0), 1.2),
                (Vec3::new(-2.5, 0.03, 2.0), Vec3::new(-4.0, 0.03, 8.0), 1.0),
                (Vec3::new(-4.0, 0.03, 8.0), Vec3::new(-5.0, 0.03, 15.0), 0.8),
            ]
        } else {
            &[
                (Vec3::new(14.5, 0.03, -4.0), Vec3::new(13.0, 0.03, 2.0), 1.2),
                (Vec3::new(13.0, 0.03, 2.0), Vec3::new(11.0, 0.03, 8.0), 1.0),
                (Vec3::new(11.0, 0.03, 8.0), Vec3::new(10.0, 0.03, 15.0), 0.8),
            ]
        };

        for (i, (start, end, width)) in stream_segments.iter().enumerate() {
            let center = (*start + *end) / 2.0;
            let diff = *end - *start;
            let length = diff.length();
            let angle = diff.z.atan2(diff.x);

            commands.spawn((
                Mesh3d(meshes.add(Cuboid::new(length, 0.02, *width))),
                MeshMaterial3d(lava_material.clone()),
                Transform::from_translation(center)
                    .with_rotation(Quat::from_rotation_y(-angle)),
                GameWorldEntity,
                crate::components::LavaStream { phase: i as f32 * 1.5 },
            ));
        }
    }
}

/// Scatters decorative models around the map, avoiding the path and build spots.
fn scatter_scenery(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    path: &[Vec3],
    build_spots: &[Vec3],
    _level: u32,
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

/// Animates lava stream segments and volcano lava materials with pulsing emissive.
pub fn animate_lava(
    time: Res<Time>,
    lava_q: Query<(&LavaStream, &MeshMaterial3d<StandardMaterial>)>,
    volcano_q: Query<Entity, With<VolcanoModel>>,
    children_q: Query<&Children>,
    mesh_mat_q: Query<&MeshMaterial3d<StandardMaterial>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let t = time.elapsed_secs();

    // Animate lava stream segments
    for (lava, mat_handle) in &lava_q {
        let Some(mat) = materials.get_mut(&mat_handle.0) else { continue };
        let wave = (t * 0.8 + lava.phase).sin() * 0.5 + 0.5; // slower
        let bright = 0.6 + wave * 0.4;
        mat.base_color = Color::srgb(0.9 * bright, 0.25 * bright, 0.0);
        mat.emissive = bevy::color::LinearRgba::new(
            3.0 * bright,
            0.6 + wave * 0.4,
            0.05 + wave * 0.15,
            1.0,
        );
    }

    // Animate volcano model's lava-colored materials to match the stream
    let volcano_wave = (t * 0.8).sin() * 0.5 + 0.5;
    let v_bright = 0.6 + volcano_wave * 0.4;
    for volcano_entity in &volcano_q {
        let mut stack = vec![volcano_entity];
        while let Some(entity) = stack.pop() {
            if let Ok(children) = children_q.get(entity) {
                stack.extend(children.iter());
            }
            let Ok(mat_handle) = mesh_mat_q.get(entity) else { continue };
            let Some(mat) = materials.get_mut(&mat_handle.0) else { continue };
            // Only modify warm-colored materials (lava parts of the volcano)
            let [r, g, b, _] = mat.base_color.to_srgba().to_f32_array();
            if r > 0.4 && r > g * 1.5 {
                mat.emissive = bevy::color::LinearRgba::new(
                    r * 2.0 * v_bright,
                    g * 1.5 * v_bright,
                    b * 0.5,
                    1.0,
                );
            }
        }
    }
}
