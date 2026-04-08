use bevy::prelude::*;
use bevy::animation::{AnimationTarget, AnimationTargetId, VariableCurve};
use std::collections::HashMap;

/// Marker for the showcase scene root entities.
#[derive(Component)]
pub struct ShowcaseModel;

/// Marker for showcase labels.
#[derive(Component)]
pub struct ShowcaseLabel;

/// Marker for models needing animation setup, stores the model path for loading its own animation.
#[derive(Component)]
pub struct ShowcaseNeedsAnim(pub String);

/// Marker for animation players already started.
#[derive(Component)]
pub struct ShowcaseAnimStarted;

/// Marker: this model needs bind-pose resets (rotation-only animation).
/// After animation applies, translation and scale are reset to bind-pose values,
/// keeping only the rotation from the animation clip.
#[derive(Component)]
pub struct RotationOnlyAnim;

/// Stores bind-pose translation and scale for every bone entity.
/// Inserted after the scene loads and armature is found.
#[derive(Component)]
pub struct BoneBindPoses(pub HashMap<Entity, (Vec3, Vec3)>);

/// Cavalry debug: different attack animation styles to compare.
#[derive(Component)]
pub enum CavalryDebugStyle {
    /// Knight Z-rotation (current approach)
    KnightLean,
    /// Knight X-rotation (forward/back thrust)
    KnightThrust,
    /// Horse rocks forward/back, knight stays
    HorseRock,
    /// Both: horse rocks + knight thrusts
    Combined,
}

/// Marker: this model's animation clips need curve stripping (keep only rotation).
/// Stores the clip handle for stripping.
#[derive(Component)]
pub struct NeedsCurveStrip(pub Handle<AnimationClip>);

/// Tracks clip handles that have already been stripped so we don't re-strip.
#[derive(Component)]
pub struct CurveStripDone;

/// Marker for the debug screen back button.
#[derive(Component)]
pub struct DebugBackButton;

/// Oscillates a debug model back and forth along +X so you can see which way it faces.
#[derive(Component)]
pub struct DebugMover {
    pub origin: Vec3,
    pub speed: f32,
}

struct ModelEntry {
    path: &'static str,
    label: &'static str,
    scale: f32,
    y_offset: f32, // raise model above ground
    anim: &'static str, // animation source GLB path (empty = no animation)
}

pub fn setup_showcase(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    // Camera — looking down at the grid
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 30.0, 35.0).looking_at(Vec3::new(0.0, 0.0, 5.0), Vec3::Y),
    ));

    // Lighting
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 600.0,
        ..default()
    });
    commands.spawn((
        DirectionalLight {
            illuminance: 8000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.8, 0.4, 0.0)),
    ));
    commands.insert_resource(ClearColor(Color::srgb(0.15, 0.15, 0.2)));

    // Ground plane
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(200.0, 200.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.2, 0.2, 0.25),
            ..default()
        })),
    ));

    // Animation sources
    let _idle = "models/enemies/anims/idle.glb";
    let golem_anim = "models/golems/golem.glb"; // golem's own embedded anim
    let _maiden_idle = "models/heroes/anims/maiden-idle.glb";
    let _mutant_idle = "models/heroes/anims/mutant-idle.glb";

    let models: Vec<ModelEntry> = vec![
        // === ROW 1: Lightning towers L0, L1, L2, Spec ===
        ModelEntry { path: "models/towers/hive-turret.glb", label: "Spark Tower (L0)", scale: 0.75, y_offset: 0.0, anim: "" },
        ModelEntry { path: "models/towers/hive-turret.glb", label: "Bolt Tower (L1)", scale: 0.85, y_offset: 0.0, anim: "" },
        ModelEntry { path: "models/towers/hive-turret.glb", label: "Storm Tower (L2)", scale: 0.9, y_offset: 0.0, anim: "" },
        ModelEntry { path: "models/towers/hive-turret.glb", label: "Storm Spire (Spec)", scale: 0.9, y_offset: 0.0, anim: "" },
        // === ROW 1 cont: Earth towers L0, L1, L2, Spec ===
        ModelEntry { path: "models/towers/tower-earth.glb", label: "Clay Barracks (L0)", scale: 1.2, y_offset: 0.0, anim: "" },
        ModelEntry { path: "models/towers/tower-earth.glb", label: "Stone Barracks (L1)", scale: 1.35, y_offset: 0.0, anim: "" },
        ModelEntry { path: "models/towers/tower-earth.glb", label: "Golem Fort (L2)", scale: 1.5, y_offset: 0.0, anim: "" },
        ModelEntry { path: "models/towers/tower-earth.glb", label: "Mountain King (Spec)", scale: 1.5, y_offset: 0.0, anim: "" },
        // === ROW 2: Ice towers L0, L1, L2, Spec ===
        ModelEntry { path: "models/towers/tower-ice.glb", label: "Frost Tower (L0)", scale: 1.5, y_offset: 0.0, anim: "" },
        ModelEntry { path: "models/towers/tower-ice.glb", label: "Ice Spire (L1)", scale: 1.65, y_offset: 0.0, anim: "" },
        ModelEntry { path: "models/towers/tower-ice.glb", label: "Blizzard Tower (L2)", scale: 1.8, y_offset: 0.0, anim: "" },
        ModelEntry { path: "models/towers/tower-ice.glb", label: "Shatter Mage (Spec)", scale: 1.8, y_offset: 0.0, anim: "" },
        // === ROW 2 cont: Fire towers L0, L1, L2, Spec ===
        ModelEntry { path: "models/towers/tower-lightning.glb", label: "Ember Cannon (L0)", scale: 2.25, y_offset: 0.0, anim: "" },
        ModelEntry { path: "models/towers/tower-lightning.glb", label: "Flame Mortar (L1)", scale: 2.4, y_offset: 0.0, anim: "" },
        ModelEntry { path: "models/towers/tower-lightning.glb", label: "Inferno (L2)", scale: 2.55, y_offset: 0.0, anim: "" },
        ModelEntry { path: "models/towers/tower-lightning.glb", label: "Meteor Tower (Spec)", scale: 2.55, y_offset: 0.0, anim: "" },

        // === ROW 3: Golem ===
        ModelEntry { path: "models/golems/golem.glb", label: "Golem", scale: 90.0, y_offset: 0.0, anim: golem_anim },
    ];

    let cols = 4; // L0, L1, L2, Spec per row
    let spacing = 10.0;

    for (idx, entry) in models.iter().enumerate() {
        let col = idx % cols;
        let row = idx / cols;
        let x = (col as f32 - cols as f32 / 2.0 + 0.5) * spacing;
        let z = row as f32 * spacing;

        // Scene model
        let scene = asset_server.load(format!("{}#Scene0", entry.path));
        let mut entity_cmds = commands.spawn((
            SceneRoot(scene),
            Transform::from_translation(Vec3::new(x, entry.y_offset, z))
                .with_scale(Vec3::splat(entry.scale)),
            ShowcaseModel,
        ));
        if !entry.anim.is_empty() {
            entity_cmds.insert(ShowcaseNeedsAnim(format!("{}#Animation0", entry.anim)));
        }

        // Label pedestal
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(spacing * 0.8, 0.05, spacing * 0.8))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgb(0.12, 0.12, 0.15),
                ..default()
            })),
            Transform::from_translation(Vec3::new(x, 0.01, z)),
        ));

        // Text label above with number
        commands.spawn((
            Text2d::new(format!("#{} {}", idx + 1, entry.label)),
            TextFont { font_size: 18.0, ..default() },
            TextColor(Color::WHITE),
            Transform::from_translation(Vec3::new(x, 3.0, z - spacing * 0.35))
                .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_4)),
            ShowcaseLabel,
        ));
    }

    info!("Showcase: spawned {} models in a {}x{} grid", models.len(), cols, (models.len() + cols - 1) / cols);
}

/// Set up animations for skinned showcase models.
/// If a model has no embedded AnimationPlayer (animations were stripped),
/// manually inserts one on the Armature node + AnimationTargets on all bones.
pub fn showcase_setup_anims(
    mut commands: Commands,
    models: Query<(Entity, &Children, &ShowcaseNeedsAnim)>,
    children_q: Query<&Children>,
    anim_players: Query<Entity, With<AnimationPlayer>>,
    names: Query<&Name>,
    asset_server: Res<AssetServer>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
) {
    for (model_entity, children, needs_anim) in &models {
        let mut stack: Vec<Entity> = children.iter().copied().collect();
        let mut found_player = None;
        let mut armature_entity = None;

        while let Some(entity) = stack.pop() {
            if anim_players.get(entity).is_ok() {
                found_player = Some(entity);
                break;
            }
            if let Ok(name) = names.get(entity) {
                let n = name.as_str();
                if n == "Armature" || n.contains("CharacterArmature") {
                    armature_entity = Some(entity);
                }
            }
            if let Ok(grandchildren) = children_q.get(entity) {
                stack.extend(grandchildren.iter());
            }
        }

        // If no AnimationPlayer found, try to create one on the Armature
        let player_entity = if let Some(e) = found_player {
            e
        } else if let Some(armature) = armature_entity {
            commands.entity(armature).insert(AnimationPlayer::default());

            // Insert AnimationTarget on armature and all bones
            let armature_name = names.get(armature)
                .map(|n| n.clone())
                .unwrap_or_else(|_| Name::new("Armature"));
            let root_path = vec![armature_name];
            commands.entity(armature).insert(AnimationTarget {
                id: AnimationTargetId::from_names(root_path.iter()),
                player: armature,
            });
            if let Ok(armature_children) = children_q.get(armature) {
                for &child in armature_children.iter() {
                    insert_showcase_anim_targets(
                        &mut commands, child, armature,
                        &root_path, &children_q, &names,
                    );
                }
            }
            info!("Showcase: manually inserted AnimationPlayer + targets on {:?}",
                  names.get(armature).map(|n| n.as_str()).unwrap_or("?"));
            armature
        } else {
            continue;
        };

        let clip: Handle<AnimationClip> = asset_server.load(&needs_anim.0);
        let (graph, _node) = AnimationGraph::from_clip(clip);
        let graph_handle = graphs.add(graph);

        commands.entity(player_entity).insert(AnimationGraphHandle(graph_handle));
        commands.entity(model_entity).remove::<ShowcaseNeedsAnim>();
    }
}

/// Recursively insert AnimationTarget on all bone children for showcase models.
fn insert_showcase_anim_targets(
    commands: &mut Commands,
    entity: Entity,
    player: Entity,
    parent_path: &[Name],
    children_q: &Query<&Children>,
    names: &Query<&Name>,
) {
    let Ok(name) = names.get(entity) else { return };
    let mut path = parent_path.to_vec();
    path.push(name.clone());

    commands.entity(entity).insert(AnimationTarget {
        id: AnimationTargetId::from_names(path.iter()),
        player,
    });

    if let Ok(children) = children_q.get(entity) {
        for &child in children.iter() {
            insert_showcase_anim_targets(commands, child, player, &path, children_q, names);
        }
    }
}

/// Start playback on showcase AnimationPlayers.
pub fn showcase_play_anims(
    mut commands: Commands,
    mut players: Query<
        (Entity, &mut AnimationPlayer, &AnimationGraphHandle),
        Without<ShowcaseAnimStarted>,
    >,
) {
    for (entity, mut player, _) in &mut players {
        let node = AnimationNodeIndex::new(1);
        player.play(node).repeat();
        commands.entity(entity).insert(ShowcaseAnimStarted);
    }
}

// ===========================================================================
// Northern Outsider Debug Screen
// ===========================================================================

#[derive(Component)]
pub struct ModelDebugEntity;

pub fn setup_model_debug(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    // Camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 8.0, 15.0).looking_at(Vec3::new(0.0, 1.0, 0.0), Vec3::Y),
        ModelDebugEntity,
    ));

    // Lighting
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 800.0,
        ..default()
    });
    commands.spawn((
        DirectionalLight {
            illuminance: 10000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.8, 0.4, 0.0)),
        ModelDebugEntity,
    ));
    commands.insert_resource(ClearColor(Color::srgb(0.15, 0.15, 0.2)));

    // Ground
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(100.0, 100.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.2, 0.2, 0.25),
            ..default()
        })),
        ModelDebugEntity,
    ));

    let pi2 = std::f32::consts::FRAC_PI_2;
    let pi = std::f32::consts::PI;
    let mino = "models/enemies/minotaur-mixamo.glb";
    let mino_idle = "models/enemies/anims/idle.glb#Animation0";
    let mino_walk = "models/enemies/anims/crouch-walk.glb#Animation0";
    let mino_atk = "models/enemies/anims/attack.glb#Animation0";
    let mino_die = "models/enemies/anims/die.glb#Animation0";

    // (label, model, scale, anim_clip_path, rotation_x, rotation_y, strip_curves)
    // strip_curves = true strips translation+scale from Mixamo clips (keeps rotation only)
    let variants: Vec<(&str, &str, f32, &str, f32, f32, bool)> = vec![
        // Row 1: Minotaur anim variants (stripped)
        ("Mino idle",       mino, 1.2, mino_idle, 0.0, 0.0, true),
        ("Mino walk",       mino, 1.2, mino_walk, 0.0, 0.0, true),
        ("Mino atk",        mino, 1.2, mino_atk,  0.0, 0.0, true),
        ("Mino die",        mino, 1.2, mino_die,  0.0, 0.0, true),
        // Row 2: No anim + scale variants
        ("Mino no anim",    mino, 1.2, "",         0.0, 0.0, false),
        ("Mino small",      mino, 0.5, mino_idle,  0.0, 0.0, true),
        ("Mino big",        mino, 2.0, mino_idle,  0.0, 0.0, true),
        ("Mino no anim 2x", mino, 2.0, "",         0.0, 0.0, false),
        // Row 3: Rotation variants (stripped)
        ("Mino +90X",       mino, 1.2, mino_idle,  pi2, 0.0, true),
        ("Mino -90X",       mino, 1.2, mino_idle, -pi2, 0.0, true),
        ("Mino rotY=90",    mino, 1.2, mino_idle,  0.0, pi2, true),
        ("Mino rotY=180",   mino, 1.2, mino_idle,  0.0, pi,  true),
    ];

    let cols = 4;
    let spacing = 12.0;

    for (idx, (label, model, scale, anim, rot_x, rot_y, strip_curves)) in variants.iter().enumerate() {
        let col = idx % cols;
        let row = idx / cols;
        let x = (col as f32 - cols as f32 / 2.0 + 0.5) * spacing;
        let z = row as f32 * -spacing;

        let scene = asset_server.load(format!("{}#Scene0", model));
        let mut transform = Transform::from_translation(Vec3::new(x, 0.0, z))
            .with_scale(Vec3::splat(*scale));
        if *rot_x != 0.0 {
            transform.rotate_x(*rot_x);
        }
        if *rot_y != 0.0 {
            transform.rotate_y(*rot_y);
        }

        let mut entity_cmds = commands.spawn((
            SceneRoot(scene),
            transform,
            ShowcaseModel,
            ModelDebugEntity,
        ));
        if !anim.is_empty() {
            let clip_path = anim.to_string();
            let clip_handle: Handle<AnimationClip> = asset_server.load(&clip_path);
            entity_cmds.insert(ShowcaseNeedsAnim(clip_path));
            if *strip_curves {
                entity_cmds.insert(NeedsCurveStrip(clip_handle));
            }
        }

        // Pedestal
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(spacing * 0.8, 0.05, spacing * 0.8))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgb(0.12, 0.12, 0.15),
                ..default()
            })),
            Transform::from_translation(Vec3::new(x, 0.01, z)),
            ModelDebugEntity,
        ));

        // Label
        commands.spawn((
            Text2d::new(*label),
            TextFont { font_size: 16.0, ..default() },
            TextColor(Color::WHITE),
            Transform::from_translation(Vec3::new(x, 3.5, z - spacing * 0.35))
                .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_4)),
            ModelDebugEntity,
        ));
    }

    // Cavalry attack animation variants (after minotaur rows)
    let horse_model = "models/enemies/cavalry-horse.glb";
    let knight_model = "models/enemies/cavalry-knight.glb";
    let horse_anim = "models/enemies/cavalry-horse.glb#Animation0"; // Walk
    let cavalry_styles = [
        ("Cav: Knight lean Z", CavalryDebugStyle::KnightLean),
        ("Cav: Knight thrust X", CavalryDebugStyle::KnightThrust),
        ("Cav: Horse rock", CavalryDebugStyle::HorseRock),
        ("Cav: Combined", CavalryDebugStyle::Combined),
    ];
    for (ci, (label, style)) in cavalry_styles.into_iter().enumerate() {
        let idx = variants.len() + ci;
        let col = idx % cols;
        let row = idx / cols;
        let x = (col as f32 - cols as f32 / 2.0 + 0.5) * spacing;
        let z = row as f32 * -spacing;

        let horse_scene = asset_server.load(format!("{}#Scene0", horse_model));
        let transform = Transform::from_translation(Vec3::new(x, 0.0, z))
            .with_scale(Vec3::splat(0.5));

        let mut entity_cmds = commands.spawn((
            SceneRoot(horse_scene),
            transform,
            ShowcaseModel,
            ModelDebugEntity,
            style,
            ShowcaseNeedsAnim(horse_anim.to_string()),
        ));

        // Mount the knight
        let knight_scene = asset_server.load(format!("{}#Scene0", knight_model));
        entity_cmds.with_child((
            SceneRoot(knight_scene),
            Transform::from_translation(Vec3::new(0.0, 1.0, 0.0))
                .with_scale(Vec3::splat(0.013)),
            crate::components::CavalryKnight,
        ));

        // Pedestal
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(spacing * 0.8, 0.05, spacing * 0.8))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgb(0.12, 0.12, 0.15),
                ..default()
            })),
            Transform::from_translation(Vec3::new(x, 0.01, z)),
            ModelDebugEntity,
        ));
        // Label
        commands.spawn((
            Text2d::new(label),
            TextFont { font_size: 16.0, ..default() },
            TextColor(Color::WHITE),
            Transform::from_translation(Vec3::new(x, 3.5, z - spacing * 0.35))
                .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_4)),
            ModelDebugEntity,
        ));
    }

    // Back button (2D UI overlay)
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(16.0),
            top: Val::Px(16.0),
            padding: UiRect::axes(Val::Px(20.0), Val::Px(10.0)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.2, 0.2, 0.3, 0.9)),
        BorderRadius::all(Val::Px(8.0)),
        DebugBackButton,
        ModelDebugEntity,
        Button,
    )).with_child((
        Text::new("< Back"),
        TextFont { font_size: 20.0, ..default() },
        TextColor(Color::WHITE),
    ));

    info!("Model debug: spawned {} Voltra variants", variants.len());
}

/// Handle back button press on the debug screen.
pub fn debug_back_button(
    interaction: Query<&Interaction, (Changed<Interaction>, With<DebugBackButton>)>,
    mut next_state: ResMut<NextState<crate::states::AppState>>,
) {
    for &inter in &interaction {
        if inter == Interaction::Pressed {
            next_state.set(crate::states::AppState::MainMenu);
        }
    }
}

/// Oscillates DebugMover entities back and forth along +X from their origin.
pub fn debug_mover_tick(
    mut movers: Query<(&DebugMover, &mut Transform)>,
    time: Res<Time>,
) {
    let t = time.elapsed_secs();
    for (mover, mut transform) in &mut movers {
        // Triangle wave: moves +X for 2s, resets to origin, repeat
        let cycle = (t * 0.5) % 1.0; // 0..1 over 2 seconds
        let offset_x = cycle * mover.speed * 2.0;
        transform.translation = mover.origin + Vec3::new(offset_x, 0.0, 0.0);
    }
}

/// Animates cavalry debug variants with different attack styles.
pub fn cavalry_debug_tick(
    cavalry_q: Query<(Entity, &CavalryDebugStyle, &Children)>,
    mut transforms: Query<&mut Transform>,
    knight_q: Query<Entity, With<crate::components::CavalryKnight>>,
    children_q: Query<&Children>,
    time: Res<Time>,
) {
    let t = time.elapsed_secs();
    for (entity, style, children) in &cavalry_q {
        // Find the knight child entity
        let knight_entity = children.iter()
            .find(|c| knight_q.get(**c).is_ok())
            .copied();

        // Find the first scene-root child (horse mesh root) — it's the child that ISN'T the knight
        let horse_child = children.iter()
            .find(|c| knight_q.get(**c).is_err())
            .copied();

        match style {
            CavalryDebugStyle::KnightLean => {
                // Current approach: knight Z-rotation lean
                if let Some(ke) = knight_entity {
                    if let Ok(mut tf) = transforms.get_mut(ke) {
                        let swing = (t * 4.0).sin() * 0.2;
                        tf.rotation = Quat::from_rotation_z(swing);
                    }
                }
            }
            CavalryDebugStyle::KnightThrust => {
                // Knight thrusts forward/back (X rotation)
                if let Some(ke) = knight_entity {
                    if let Ok(mut tf) = transforms.get_mut(ke) {
                        let thrust = (t * 5.0).sin() * 0.25;
                        tf.rotation = Quat::from_rotation_x(thrust);
                    }
                }
            }
            CavalryDebugStyle::HorseRock => {
                // Horse scene root rocks forward/back
                if let Some(hc) = horse_child {
                    if let Ok(mut tf) = transforms.get_mut(hc) {
                        let rock = (t * 3.0).sin() * 0.08;
                        tf.rotation = Quat::from_rotation_x(rock);
                    }
                }
            }
            CavalryDebugStyle::Combined => {
                // Horse rocks + knight thrusts
                if let Some(hc) = horse_child {
                    if let Ok(mut tf) = transforms.get_mut(hc) {
                        let rock = (t * 3.0).sin() * 0.06;
                        tf.rotation = Quat::from_rotation_x(rock);
                    }
                }
                if let Some(ke) = knight_entity {
                    if let Ok(mut tf) = transforms.get_mut(ke) {
                        let thrust = (t * 5.0).sin() * 0.2;
                        tf.rotation = Quat::from_rotation_x(thrust);
                    }
                }
            }
        }
    }
}

pub fn cleanup_model_debug(
    mut commands: Commands,
    entities: Query<Entity, With<ModelDebugEntity>>,
) {
    for entity in &entities {
        commands.entity(entity).despawn_recursive();
    }
}

/// Captures bind-pose translations and scales for models marked with `RotationOnlyAnim`.
/// Runs every frame until it finds the armature; once captured, `BoneBindPoses` is inserted
/// and the component acts as the "done" flag.
pub fn capture_rotation_only_bind_poses(
    mut commands: Commands,
    models: Query<(Entity, &Children), (With<RotationOnlyAnim>, Without<BoneBindPoses>)>,
    children_q: Query<&Children>,
    names: Query<&Name>,
    transforms: Query<&Transform>,
) {
    for (model_entity, children) in &models {
        // Find armature
        let mut stack: Vec<Entity> = children.iter().copied().collect();
        let mut armature_entity = None;
        while let Some(entity) = stack.pop() {
            if let Ok(name) = names.get(entity) {
                let n = name.as_str();
                if n == "Armature" || n.contains("CharacterArmature") {
                    armature_entity = Some(entity);
                    break;
                }
            }
            if let Ok(grandchildren) = children_q.get(entity) {
                stack.extend(grandchildren.iter());
            }
        }

        let Some(armature) = armature_entity else { continue };

        // Walk the bone hierarchy and capture every bone's bind-pose translation + scale
        let mut bind_poses = HashMap::new();
        fn capture_recursive(
            entity: Entity,
            children_q: &Query<&Children>,
            transforms: &Query<&Transform>,
            out: &mut HashMap<Entity, (Vec3, Vec3)>,
        ) {
            if let Ok(tf) = transforms.get(entity) {
                out.insert(entity, (tf.translation, tf.scale));
            }
            if let Ok(children) = children_q.get(entity) {
                for &child in children.iter() {
                    capture_recursive(child, children_q, transforms, out);
                }
            }
        }
        capture_recursive(armature, &children_q, &transforms, &mut bind_poses);

        info!("Captured bind poses for {} bones on {:?}", bind_poses.len(), model_entity);
        commands.entity(model_entity).insert(BoneBindPoses(bind_poses));
    }
}

/// Resets bone translations and scales to bind-pose after animation applies.
/// This keeps only rotation from the animation, fixing models with bone-scale mismatch.
/// Must run after animate_targets but before TransformPropagate.
pub fn reset_rotation_only_bones(
    models: Query<&BoneBindPoses>,
    mut transforms: Query<&mut Transform>,
) {
    for poses in &models {
        for (&entity, &(translation, scale)) in &poses.0 {
            if let Ok(mut tf) = transforms.get_mut(entity) {
                tf.translation = translation;
                tf.scale = scale;
            }
        }
    }
}

/// Strips translation/scale curves from animation clips for models marked with NeedsCurveStrip.
/// Keeps only rotation curves (index 1 in GLTF bone curve ordering: [translation, rotation, scale]).
pub fn strip_debug_rotation_only_clips(
    mut commands: Commands,
    models: Query<(Entity, &NeedsCurveStrip), Without<CurveStripDone>>,
    mut clips: ResMut<Assets<AnimationClip>>,
) {
    for (entity, needs_strip) in &models {
        let Some(clip) = clips.get_mut(&needs_strip.0) else { continue };

        let mut stripped_count = 0;
        for (_target_id, curves) in clip.curves_mut().iter_mut() {
            if curves.len() >= 2 {
                let rotation = VariableCurve(curves[1].0.clone_value());
                curves.clear();
                curves.push(rotation);
                stripped_count += 1;
            }
        }
        info!("Debug: stripped translation/scale from {} bone targets", stripped_count);
        commands.entity(entity).insert(CurveStripDone);
    }
}

/// Camera pan for showcase — WASD + scroll zoom.
pub fn showcase_camera(
    mut focus: Local<Vec3>,
    mut distance: Local<f32>,
    keys: Res<ButtonInput<KeyCode>>,
    mut scroll: EventReader<bevy::input::mouse::MouseWheel>,
    mut camera: Query<&mut Transform, With<Camera3d>>,
    time: Res<Time>,
) {
    if *distance == 0.0 {
        *distance = 60.0;
        *focus = Vec3::new(0.0, 0.0, 20.0);
    }

    let speed = 25.0 * time.delta_secs();
    if keys.pressed(KeyCode::KeyW) { focus.z -= speed; }
    if keys.pressed(KeyCode::KeyS) { focus.z += speed; }
    if keys.pressed(KeyCode::KeyA) { focus.x -= speed; }
    if keys.pressed(KeyCode::KeyD) { focus.x += speed; }

    for ev in scroll.read() {
        *distance = (*distance - ev.y * 5.0).clamp(5.0, 500.0);
    }

    if let Ok(mut transform) = camera.get_single_mut() {
        let offset = Vec3::new(0.0, *distance * 0.7, *distance * 0.6);
        transform.translation = *focus + offset;
        transform.look_at(*focus, Vec3::Y);
    }
}
