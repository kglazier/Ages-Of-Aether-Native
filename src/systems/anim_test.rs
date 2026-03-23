use bevy::prelude::*;
use bevy::animation::{AnimationTarget, AnimationTargetId};

// ---------------------------------------------------------------------------
// Components
// ---------------------------------------------------------------------------

#[derive(Component)]
pub struct AnimTestModel;

#[derive(Component)]
pub struct AnimTestNeedsSetup;

/// Stores all the animation state for the test screen.
#[derive(Component)]
pub struct AnimTestState {
    pub clip_names: Vec<String>,
    pub node_indices: Vec<AnimationNodeIndex>,
    pub current_clip: usize,
    pub player_entity: Entity,
}

#[derive(Component)]
pub struct AnimTestUI;

/// Marker for the reference model (run.glb spawned as its own scene).
#[derive(Component)]
pub struct AnimTestRefModel;

/// Marker for the reference model's anim setup.
#[derive(Component)]
pub struct AnimTestRefNeedsAnim;

/// Marker for reference model anim started.
#[derive(Component)]
pub struct AnimTestRefAnimStarted;

// ---------------------------------------------------------------------------
// Setup
// ---------------------------------------------------------------------------

pub fn setup_anim_test(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    // Camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 3.0, 6.0).looking_at(Vec3::new(0.0, 1.0, 0.0), Vec3::Y),
    ));

    // Lighting
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 800.0,
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
    commands.insert_resource(ClearColor(Color::srgb(0.1, 0.1, 0.15)));

    // Ground
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(20.0, 20.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.2, 0.2, 0.25),
            ..default()
        })),
    ));

    // Sacred Maiden model (center) — needs manual animation setup
    let maiden_scene = asset_server.load("models/heroes/sacred-maiden.glb#Scene0");
    commands.spawn((
        SceneRoot(maiden_scene),
        Transform::from_translation(Vec3::new(0.0, 0.0, 0.0))
            .with_scale(Vec3::splat(2.0)),
        AnimTestModel,
        AnimTestNeedsSetup,
    ));

    // Reference: run.glb spawned as its own scene (right side)
    // This should auto-animate on its own skeleton — confirms the clip works.
    let run_scene = asset_server.load("models/enemies/anims/run.glb#Scene0");
    commands.spawn((
        SceneRoot(run_scene),
        Transform::from_translation(Vec3::new(4.0, 0.0, 0.0))
            .with_scale(Vec3::splat(2.0)),
        AnimTestRefModel,
        AnimTestRefNeedsAnim,
    ));

    // Reference: maiden-idle.glb spawned as its own scene (left side)
    let idle_scene = asset_server.load("models/heroes/anims/maiden-idle.glb#Scene0");
    commands.spawn((
        SceneRoot(idle_scene),
        Transform::from_translation(Vec3::new(-4.0, 0.0, 0.0))
            .with_scale(Vec3::splat(2.0)),
        AnimTestRefModel,
        AnimTestRefNeedsAnim,
    ));

    // UI instructions
    commands.spawn((
        Text::new("AnimTest: 1=idle 2=run 3=walk 4=attack 5=crouch-walk 6=maiden-idle-fast\nCenter=Maiden  Left=idle.glb ref  Right=run.glb ref"),
        TextFont { font_size: 16.0, ..default() },
        TextColor(Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            left: Val::Px(10.0),
            ..default()
        },
        AnimTestUI,
    ));

    info!("AnimTest: screen set up");
}

// ---------------------------------------------------------------------------
// Skeleton setup for the Sacred Maiden (same as hero.rs)
// ---------------------------------------------------------------------------

pub fn anim_test_setup_skeleton(
    mut commands: Commands,
    models: Query<(Entity, &Children), With<AnimTestNeedsSetup>>,
    children_q: Query<&Children>,
    anim_players: Query<Entity, With<AnimationPlayer>>,
    names: Query<&Name>,
    asset_server: Res<AssetServer>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
) {
    for (model_entity, children) in &models {
        // Walk scene hierarchy to find AnimationPlayer or Armature
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

        let player_entity = if let Some(e) = found_player {
            e
        } else if let Some(e) = armature_entity {
            commands.entity(e).insert(AnimationPlayer::default());
            e
        } else {
            continue; // Scene not loaded yet
        };

        // Insert AnimationTarget on every bone (same as hero.rs)
        if found_player.is_none() {
            if let Some(armature) = armature_entity {
                let armature_name = names.get(armature)
                    .map(|n| n.clone())
                    .unwrap_or_else(|_| Name::new("Armature"));
                let root_path = vec![armature_name];

                commands.entity(armature).insert(AnimationTarget {
                    id: AnimationTargetId::from_names(root_path.iter()),
                    player: armature,
                });

                // Log the root target ID
                let root_id = AnimationTargetId::from_names(root_path.iter());
                info!("AnimTest MAIDEN root target: {:?} (path: {:?})", root_id, root_path.iter().map(|n| n.as_str()).collect::<Vec<_>>());

                if let Ok(armature_children) = children_q.get(armature) {
                    for &child in armature_children.iter() {
                        insert_anim_targets_with_logging(
                            &mut commands, child, armature,
                            &root_path, &children_q, &names,
                        );
                    }
                }
                info!("AnimTest: inserted AnimationTargets on maiden bone hierarchy");
            }
        }

        // Build animation graph with all test clips
        let clip_defs: Vec<(&str, &str)> = vec![
            ("maiden-idle", "models/heroes/anims/maiden-idle.glb"),
            ("run", "models/enemies/anims/run.glb"),
            ("walk", "models/enemies/anims/walk.glb"),
            ("maiden-kick", "models/heroes/anims/maiden-melee-kick.glb"),
            ("crouch-walk", "models/enemies/anims/crouch-walk.glb"),
            ("maiden-idle-fast", "models/heroes/anims/maiden-idle.glb"),
        ];

        let mut graph = AnimationGraph::new();
        let mut clip_names = Vec::new();
        let mut node_indices = Vec::new();

        for (name, path) in &clip_defs {
            let clip: Handle<AnimationClip> = asset_server.load(format!("{}#Animation0", path));
            let speed = if *name == "maiden-idle-fast" { 3.0 } else { 1.0 };
            let node = graph.add_clip(clip, speed, graph.root);
            clip_names.push(name.to_string());
            node_indices.push(node);
            info!("AnimTest: loaded clip '{}' from {} -> node {:?}", name, path, node);
        }

        let graph_handle = graphs.add(graph);
        commands.entity(player_entity).insert(AnimationGraphHandle(graph_handle));

        commands.entity(model_entity).insert(AnimTestState {
            clip_names,
            node_indices,
            current_clip: 0,
            player_entity,
        });
        commands.entity(model_entity).remove::<AnimTestNeedsSetup>();

        // Start with idle
        info!("AnimTest: skeleton + graph ready, starting idle");
    }
}

/// Same as hero.rs but also logs each bone path and target ID.
fn insert_anim_targets_with_logging(
    commands: &mut Commands,
    entity: Entity,
    player_entity: Entity,
    parent_path: &[Name],
    children_q: &Query<&Children>,
    names: &Query<&Name>,
) {
    let Ok(name) = names.get(entity) else {
        return;
    };

    let mut path = parent_path.to_vec();
    path.push(name.clone());

    let id = AnimationTargetId::from_names(path.iter());

    // Log first few bones for diagnostics
    if path.len() <= 3 {
        info!("AnimTest MAIDEN bone: {:?} -> {:?}",
            path.iter().map(|n| n.as_str()).collect::<Vec<_>>(), id);
    }

    commands.entity(entity).insert(AnimationTarget {
        id,
        player: player_entity,
    });

    if let Ok(children) = children_q.get(entity) {
        for &child in children.iter() {
            insert_anim_targets_with_logging(commands, child, player_entity, &path, children_q, names);
        }
    }
}

// ---------------------------------------------------------------------------
// Start idle on first setup
// ---------------------------------------------------------------------------

pub fn anim_test_start_idle(
    models: Query<&AnimTestState, Added<AnimTestState>>,
    mut players: Query<&mut AnimationPlayer>,
) {
    for state in &models {
        if let Ok(mut player) = players.get_mut(state.player_entity) {
            if !state.node_indices.is_empty() {
                player.play(state.node_indices[0]).repeat();
                info!("AnimTest: playing '{}'", state.clip_names[0]);
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Reference model auto-animation (for run.glb and idle.glb spawned as scenes)
// ---------------------------------------------------------------------------

pub fn anim_test_ref_setup(
    mut commands: Commands,
    models: Query<(Entity, &Children), With<AnimTestRefNeedsAnim>>,
    children_q: Query<&Children>,
    anim_players: Query<Entity, With<AnimationPlayer>>,
) {
    for (model_entity, children) in &models {
        let mut stack: Vec<Entity> = children.iter().copied().collect();
        let mut found_player = None;

        while let Some(entity) = stack.pop() {
            if anim_players.get(entity).is_ok() {
                found_player = Some(entity);
                break;
            }
            if let Ok(grandchildren) = children_q.get(entity) {
                stack.extend(grandchildren.iter());
            }
        }

        if found_player.is_some() {
            commands.entity(model_entity).remove::<AnimTestRefNeedsAnim>();
            info!("AnimTest: ref model {:?} has AnimationPlayer", model_entity);
        }
    }
}

pub fn anim_test_ref_play(
    mut commands: Commands,
    ref_models: Query<Entity, (With<AnimTestRefModel>, Without<AnimTestRefNeedsAnim>, Without<AnimTestRefAnimStarted>)>,
    children_q: Query<&Children>,
    mut players: Query<(Entity, &mut AnimationPlayer, Option<&AnimationGraphHandle>)>,
) {
    for model_entity in &ref_models {
        // Walk children to find the animation player
        let mut stack = vec![model_entity];
        while let Some(entity) = stack.pop() {
            if let Ok((player_entity, mut player, graph)) = players.get_mut(entity) {
                if graph.is_some() {
                    // Has a graph — play node 1 (first clip)
                    let node = AnimationNodeIndex::new(1);
                    player.play(node).repeat();
                    commands.entity(model_entity).insert(AnimTestRefAnimStarted);
                    info!("AnimTest: ref model playing via graph on {:?}", player_entity);
                }
                break;
            }
            if let Ok(children) = children_q.get(entity) {
                stack.extend(children.iter());
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Keyboard input to switch clips
// ---------------------------------------------------------------------------

pub fn anim_test_input(
    keys: Res<ButtonInput<KeyCode>>,
    mut models: Query<&mut AnimTestState>,
    mut players: Query<&mut AnimationPlayer>,
    mut ui_text: Query<&mut Text, With<AnimTestUI>>,
) {
    let pressed = if keys.just_pressed(KeyCode::Digit1) { Some(0) }
        else if keys.just_pressed(KeyCode::Digit2) { Some(1) }
        else if keys.just_pressed(KeyCode::Digit3) { Some(2) }
        else if keys.just_pressed(KeyCode::Digit4) { Some(3) }
        else if keys.just_pressed(KeyCode::Digit5) { Some(4) }
        else if keys.just_pressed(KeyCode::Digit6) { Some(5) }
        else { None };

    let Some(idx) = pressed else { return; };

    for mut state in &mut models {
        if idx >= state.node_indices.len() {
            continue;
        }

        state.current_clip = idx;
        if let Ok(mut player) = players.get_mut(state.player_entity) {
            player.stop_all();
            player.play(state.node_indices[idx]).repeat();
            info!("AnimTest: switched to '{}' (node {:?})",
                state.clip_names[idx], state.node_indices[idx]);
        }

        for mut text in &mut ui_text {
            text.0 = format!(
                "AnimTest: 1=idle 2=run 3=walk 4=attack 5=crouch-walk 6=maiden-idle-fast\n\
                 Center=Maiden  Left=idle.glb ref  Right=run.glb ref\n\
                 >>> Playing: {} <<<",
                state.clip_names[idx],
            );
        }
    }
}

// ---------------------------------------------------------------------------
// Diagnostic: log hierarchy of reference models to compare root names
// ---------------------------------------------------------------------------

pub fn anim_test_log_ref_hierarchy(
    mut logged: Local<bool>,
    ref_models: Query<(Entity, &Children), (With<AnimTestRefModel>, Without<AnimTestRefNeedsAnim>)>,
    children_q: Query<&Children>,
    names: Query<&Name>,
    anim_targets: Query<&AnimationTarget>,
) {
    if *logged {
        return;
    }

    // Wait until at least one ref model is ready
    if ref_models.is_empty() {
        return;
    }

    *logged = true;

    for (model_entity, children) in &ref_models {
        info!("AnimTest: --- Ref model {:?} hierarchy ---", model_entity);
        let mut stack: Vec<(Entity, usize)> = children.iter().map(|&e| (e, 0)).collect();
        while let Some((entity, depth)) = stack.pop() {
            let name = names.get(entity).map(|n| n.as_str().to_string()).unwrap_or_else(|_| "???".to_string());
            let target_info = anim_targets.get(entity)
                .map(|t| format!("target={:?}", t.id))
                .unwrap_or_default();
            let indent = "  ".repeat(depth);
            if depth < 4 {
                info!("AnimTest:   {}{} {}", indent, name, target_info);
            }
            if let Ok(grandchildren) = children_q.get(entity) {
                for &child in grandchildren.iter() {
                    stack.push((child, depth + 1));
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Simple camera orbit
// ---------------------------------------------------------------------------

pub fn anim_test_camera(
    keys: Res<ButtonInput<KeyCode>>,
    mut camera: Query<&mut Transform, With<Camera3d>>,
    time: Res<Time>,
    mut angle: Local<f32>,
    mut distance: Local<f32>,
    mut height: Local<f32>,
) {
    if *distance == 0.0 {
        *distance = 6.0;
        *height = 3.0;
    }

    let speed = 2.0 * time.delta_secs();
    if keys.pressed(KeyCode::KeyA) { *angle -= speed; }
    if keys.pressed(KeyCode::KeyD) { *angle += speed; }
    if keys.pressed(KeyCode::KeyW) { *distance = (*distance - speed * 2.0).max(2.0); }
    if keys.pressed(KeyCode::KeyS) { *distance = (*distance + speed * 2.0).min(20.0); }
    if keys.pressed(KeyCode::KeyQ) { *height += speed; }
    if keys.pressed(KeyCode::KeyE) { *height -= speed; }

    if let Ok(mut tf) = camera.get_single_mut() {
        tf.translation = Vec3::new(
            angle.cos() * *distance,
            *height,
            angle.sin() * *distance,
        );
        tf.look_at(Vec3::new(0.0, 1.0, 0.0), Vec3::Y);
    }
}
