use bevy::prelude::*;
use bevy::animation::{AnimationTarget, AnimationTargetId, VariableCurve};
use crate::components::*;
use crate::data::*;

/// Tracks clip handles for root motion stripping. All 4 clips need stripping.
#[derive(Component)]
pub struct EnemyClipsNeedStrip {
    pub handles: Vec<Handle<AnimationClip>>,
    pub stripped: bool,
}

/// Tracks clip handles that need bone name remapping (e.g. Mixamo → custom rig).
#[derive(Component)]
pub struct NeedsBoneRemap {
    pub handles: Vec<Handle<AnimationClip>>,
    pub bone_map: &'static [(&'static str, &'static str)],
    pub remapped: bool,
}

/// After enemy scene loads, find AnimationPlayer and build animation graph.
/// Supports two modes:
/// - Embedded animations (blobs): loads clips by index from the GLTF file
/// - External animations (skinned humanoids): loads clips from separate GLB files
/// - No animations (animals/dinosaurs): falls back to ProceduralWalkAnim
pub fn setup_enemy_animations(
    mut commands: Commands,
    enemies: Query<(Entity, &Children, &EnemyTypeId), With<EnemyNeedsAnimation>>,
    children_q: Query<&Children>,
    anim_players: Query<Entity, With<AnimationPlayer>>,
    names: Query<&Name>,
    asset_server: Res<AssetServer>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
) {
    for (enemy_entity, children, type_id) in &enemies {
        // Walk hierarchy to find AnimationPlayer (auto-created by Bevy for GLTFs with animations)
        let mut stack: Vec<Entity> = children.iter().copied().collect();
        let mut player_entity = None;

        while let Some(entity) = stack.pop() {
            if anim_players.get(entity).is_ok() {
                player_entity = Some(entity);
                break;
            }
            if let Ok(grandchildren) = children_q.get(entity) {
                stack.extend(grandchildren.iter());
            }
        }

        let stats = enemy_stats(type_id.0);

        // Check if anim_indices are configured (255 = unconfigured sentinel)
        let has_configured_indices = stats.anim_indices != [255; 4];

        if let Some(anim_files) = stats.anim_files {
            // External animation mode (humanoid Mixamo rigs)
            // Need an AnimationPlayer — insert one manually if not found
            let player_entity = if let Some(pe) = player_entity {
                pe
            } else {
                // Find the Armature entity by name
                let mut armature = None;
                let mut search: Vec<Entity> = children.iter().copied().collect();
                while let Some(e) = search.pop() {
                    if let Ok(name) = names.get(e) {
                        if name.as_str() == "Armature" || name.as_str().contains("CharacterArmature") {
                            armature = Some(e);
                            break;
                        }
                    }
                    if let Ok(gc) = children_q.get(e) {
                        search.extend(gc.iter());
                    }
                }
                if let Some(armature) = armature {
                    // Insert AnimationPlayer + AnimationTarget on armature and all bones
                    commands.entity(armature).insert(AnimationPlayer::default());
                    let armature_name = names.get(armature)
                        .map(|n| n.clone())
                        .unwrap_or_else(|_| Name::new("Armature"));
                    let root_path = vec![armature_name.clone()];
                    commands.entity(armature).insert(AnimationTarget {
                        id: AnimationTargetId::from_names(root_path.iter()),
                        player: armature,
                    });
                    if let Ok(arm_children) = children_q.get(armature) {
                        for &child in arm_children.iter() {
                            insert_enemy_anim_targets(
                                &mut commands, child, armature,
                                &root_path, &children_q, &names,
                            );
                        }
                    }
                    info!("Inserted AnimationTarget on enemy bone hierarchy for {:?}", type_id.0);
                    armature
                } else {
                    // Fallback: use first child
                    let host = children.iter().next().copied();
                    let Some(host) = host else { continue };
                    commands.entity(host).insert(AnimationPlayer::default());
                    host
                }
            };

            let walk_clip: Handle<AnimationClip> = asset_server.load(format!("{}#Animation0", anim_files[0]));
            let idle_clip: Handle<AnimationClip> = asset_server.load(format!("{}#Animation0", anim_files[1]));
            let attack_clip: Handle<AnimationClip> = asset_server.load(format!("{}#Animation0", anim_files[2]));
            let death_clip: Handle<AnimationClip> = asset_server.load(format!("{}#Animation0", anim_files[3]));

            // Store handles for root motion stripping after clips load
            let strip_handles = vec![
                walk_clip.clone(), idle_clip.clone(),
                attack_clip.clone(), death_clip.clone(),
            ];

            let mut graph = AnimationGraph::new();
            let walk_node = graph.add_clip(walk_clip, 1.0, graph.root);
            let idle_node = graph.add_clip(idle_clip, 1.0, graph.root);
            let attack_node = graph.add_clip(attack_clip, 1.0, graph.root);
            let death_node = graph.add_clip(death_clip, 1.0, graph.root);
            let graph_handle = graphs.add(graph);

            commands.entity(player_entity).insert(AnimationGraphHandle(graph_handle));

            commands.entity(enemy_entity).insert((
                EnemyAnimState {
                    walk_node,
                    idle_node,
                    attack_node,
                    death_node,
                    current: EnemyAnimKind::Walk,
                    player_entity,
                },
                EnemyClipsNeedStrip { handles: strip_handles, stripped: false },
            ));
        } else if has_configured_indices && player_entity.is_some() {
            // Embedded animation mode (animals/dinosaurs with built-in clips)
            let pe = player_entity.unwrap();
            let [walk_idx, idle_idx, attack_idx, death_idx] = stats.anim_indices;

            // Stop any auto-playing animation from the GLB load
            commands.entity(pe).insert(AnimationPlayer::default());

            let walk_clip = asset_server.load(format!("{}#Animation{}", stats.model_path, walk_idx));
            let idle_clip = asset_server.load(format!("{}#Animation{}", stats.model_path, idle_idx));
            let attack_clip = asset_server.load(format!("{}#Animation{}", stats.model_path, attack_idx));
            let death_clip = asset_server.load(format!("{}#Animation{}", stats.model_path, death_idx));

            let mut graph = AnimationGraph::new();
            let walk_node = graph.add_clip(walk_clip, 1.0, graph.root);
            let idle_node = graph.add_clip(idle_clip, 1.0, graph.root);
            let attack_node = graph.add_clip(attack_clip, 1.0, graph.root);
            let death_node = graph.add_clip(death_clip, 1.0, graph.root);
            let graph_handle = graphs.add(graph);

            commands.entity(pe).insert(AnimationGraphHandle(graph_handle));

            commands.entity(enemy_entity).insert(EnemyAnimState {
                walk_node,
                idle_node,
                attack_node,
                death_node,
                current: EnemyAnimKind::Walk,
                player_entity: pe,
            });
            info!("Embedded anim setup: {:?} indices={:?} walk={:?} atk={:?}",
                  type_id.0, stats.anim_indices, walk_node, attack_node);
        } else if has_configured_indices {
            // Embedded anim indices configured but AnimationPlayer not found yet —
            // scene is still loading. Retry next frame.
            info!("Waiting for AnimationPlayer: {:?}", type_id.0);
            continue;
        } else if children.len() > 0 {
            // Animal/dinosaur model with no configured animations —
            // use procedural walk animation with leg bones.
            // Replace any auto-created AnimationPlayer with a fresh stopped one
            // so the GLB's default animation doesn't play.
            if let Some(pe) = player_entity {
                commands.entity(pe).insert(AnimationPlayer::default());
            }
            commands.entity(enemy_entity).insert((
                ProceduralWalkAnim { phase: 0.0 },
                NeedsLegDiscovery,
            ));
        } else {
            // Scene not loaded yet — try again next frame
            continue;
        }

        commands.entity(enemy_entity).remove::<EnemyNeedsAnimation>();
    }
}

/// Start walk animation on newly set up enemies.
pub fn play_enemy_walk_anim(
    enemies: Query<&EnemyAnimState, Added<EnemyAnimState>>,
    mut players: Query<&mut AnimationPlayer>,
) {
    for anim_state in &enemies {
        if let Ok(mut player) = players.get_mut(anim_state.player_entity) {
            player.play(anim_state.walk_node).repeat();
        }
    }
}

/// Switch enemy animation based on state:
/// - Walking: walk loop
/// - Blocked: attack loop (replaces procedural headbutt)
/// - Dying: death animation plays once
pub fn update_enemy_animations(
    mut enemies: Query<(&mut EnemyAnimState, Option<&GolemBlocked>, Option<&EnemyDying>)>,
    mut players: Query<&mut AnimationPlayer>,
) {
    for (mut anim_state, blocked, dying) in &mut enemies {
        let desired = if dying.is_some() {
            EnemyAnimKind::Death
        } else if blocked.is_some() {
            EnemyAnimKind::Attack
        } else {
            EnemyAnimKind::Walk
        };

        if desired != anim_state.current {
            info!("Anim transition: {:?} -> {:?} (player={:?}, atk_node={:?})",
                  anim_state.current, desired, anim_state.player_entity, anim_state.attack_node);
            anim_state.current = desired;
            if let Ok(mut player) = players.get_mut(anim_state.player_entity) {
                player.stop_all();
                match desired {
                    EnemyAnimKind::Walk => { player.play(anim_state.walk_node).repeat(); }
                    EnemyAnimKind::Idle => { player.play(anim_state.idle_node).repeat(); }
                    EnemyAnimKind::Attack => { player.play(anim_state.attack_node).repeat(); }
                    EnemyAnimKind::Death => { player.play(anim_state.death_node); }
                }
            }
        }
    }
}

/// Applies rocking/tilt overlay on enemies whose all anim_indices are the same
/// (single embedded clip used for all states, e.g. sabertooth).
/// Rocks side-to-side when blocked (attack), tilts forward when dying.
pub fn rock_single_clip_enemies(
    enemies: Query<(&EnemyAnimState, &EnemyTypeId, &Children, Option<&GolemBlocked>, Option<&EnemyDying>, Option<&Flying>)>,
    mut transforms: Query<&mut Transform>,
    time: Res<Time>,
) {
    for (anim_state, type_id, children, blocked, dying, flying) in &enemies {
        // Skip flying enemies — their child rotation is set by EnemyModelRotation
        if flying.is_some() { continue; }
        let stats = crate::data::enemy_stats(type_id.0);
        // Only apply to enemies whose walk/idle/attack/death all point to the same clip
        let [w, i, a, d] = stats.anim_indices;
        if w != i || w != a || w != d { continue; }
        if w == 255 { continue; } // procedural, not embedded

        let Some(&child) = children.iter().next() else { continue };
        let Ok(mut t) = transforms.get_mut(child) else { continue };

        if dying.is_some() {
            // Tilt forward on death
            t.rotation = Quat::from_rotation_x(0.5);
        } else if blocked.is_some() {
            // Rock back and forth when attacking
            let rock = (time.elapsed_secs() * 6.0).sin() * 0.3;
            t.rotation = Quat::from_rotation_x(rock);
        } else {
            // Walking normally — reset rotation
            t.rotation = Quat::IDENTITY;
        }
    }
}

/// Discovers quadruped leg bones by name after the GLTF scene loads.
/// Looks for bones named FrontLeg.L/R, BackLeg.L/R (or Mixamo equivalents).
/// Captures bind pose Euler Y/Z so the walk oscillation only replaces the X component.
/// Retries each frame until the scene hierarchy is populated.
pub fn discover_leg_bones(
    mut commands: Commands,
    enemies: Query<(Entity, &Children), With<NeedsLegDiscovery>>,
    children_q: Query<&Children>,
    names: Query<&Name>,
    transforms: Query<&Transform>,
) {
    for (enemy_entity, children) in &enemies {
        let mut leg_bones: Vec<(Entity, f32, f32, f32)> = Vec::new();
        let mut foot_bones: Vec<(Entity, f32, Quat, Vec3)> = Vec::new();
        let mut named_count = 0u32;
        let mut stack: Vec<Entity> = children.iter().copied().collect();
        while let Some(entity) = stack.pop() {
            if let Ok(name) = names.get(entity) {
                named_count += 1;
                let n = name.as_str().to_lowercase();

                // Leg bones (hip/shoulder joints) — Euler X replacement
                let leg_phase = if n == "frontleg.l" { Some(std::f32::consts::PI) }
                    else if n == "frontleg.r" { Some(0.0) }
                    else if n == "backleg.l" { Some(0.0) }
                    else if n == "backleg.r" { Some(std::f32::consts::PI) }
                    else if n.ends_with("leftupleg") { Some(0.0) }
                    else if n.ends_with("rightupleg") { Some(std::f32::consts::PI) }
                    else if n.ends_with("leftarm") && !n.contains("fore") { Some(std::f32::consts::PI) }
                    else if n.ends_with("rightarm") && !n.contains("fore") { Some(0.0) }
                    // Minotaur-style: Leg1.L/R (upper leg = hip joint)
                    else if n.starts_with("leg1.l") { Some(0.0) }
                    else if n.starts_with("leg1.r") { Some(std::f32::consts::PI) }
                    else { None };

                if let Some(phase_offset) = leg_phase {
                    let (ez, ey, _ex) = transforms
                        .get(entity)
                        .map(|t| t.rotation.to_euler(bevy::math::EulerRot::ZYX))
                        .unwrap_or((0.0, 0.0, 0.0));
                    leg_bones.push((entity, phase_offset, ez, ey));
                }

                // Foot IK-target bones — delta rotation to follow legs
                let foot_phase = if n == "frontfoot.l" { Some(std::f32::consts::PI) }
                    else if n == "frontfoot.r" { Some(0.0) }
                    else if n == "backfoot.l" { Some(0.0) }
                    else if n == "backfoot.r" { Some(std::f32::consts::PI) }
                    else { None };

                if let Some(phase_offset) = foot_phase {
                    let (bind_quat, bind_translation) = transforms
                        .get(entity)
                        .map(|t| (t.rotation, t.translation))
                        .unwrap_or((Quat::IDENTITY, Vec3::ZERO));
                    foot_bones.push((entity, phase_offset, bind_quat, bind_translation));
                }
            }
            if let Ok(gc) = children_q.get(entity) {
                stack.extend(gc.iter());
            }
        }
        if leg_bones.len() >= 2 {
            info!("Discovered {} leg + {} foot bones for procedural walk", leg_bones.len(), foot_bones.len());
            commands.entity(enemy_entity).insert(QuadLegBones { legs: leg_bones, feet: foot_bones });
            commands.entity(enemy_entity).remove::<NeedsLegDiscovery>();
        } else if named_count >= 4 {
            commands.entity(enemy_entity).remove::<NeedsLegDiscovery>();
        }
    }
}

/// Procedural walk animation for animals/dinosaurs.
/// If leg bones were discovered, oscillates them for a walking gait.
/// Otherwise falls back to gentle bob + lean on the scene root.
pub fn animate_procedural_walk(
    mut enemies: Query<(&Children, &mut ProceduralWalkAnim, Option<&QuadLegBones>, Option<&PathFollower>, Option<&EnemyDying>, Option<&GolemBlocked>)>,
    mut transforms: Query<&mut Transform>,
    time: Res<Time>,
) {
    let dt = time.delta_secs();
    for (children, mut anim, quad_legs, follower, dying, blocked) in &mut enemies {
        if dying.is_some() {
            if let Some(&child) = children.iter().next() {
                if let Ok(mut t) = transforms.get_mut(child) {
                    t.rotation = Quat::from_rotation_x(0.5);
                }
            }
            continue;
        }

        // Headbutt animation when blocked by golem
        if blocked.is_some() {
            anim.phase += dt * 6.0;
            let headbutt = (anim.phase * 4.0).sin() * 0.25;
            if let Some(&child) = children.iter().next() {
                if let Ok(mut t) = transforms.get_mut(child) {
                    t.rotation = Quat::from_rotation_x(headbutt);
                }
            }
            continue;
        }

        // Three.js uses a global walkTime shared across all enemies.
        // Speed determines frequency: sin(walkTime * speed + phase).
        let move_speed = follower.map(|f| f.speed).unwrap_or(2.0);
        // Keep per-enemy phase for bob/lean fallback
        anim.phase += dt * move_speed * 4.0;

        let walk_time = time.elapsed_secs();
        let speed = move_speed * 4.0;

        // Animate leg bones — replace outermost X rotation with oscillation (ZYX order).
        // In glam ZYX, X is outermost = rotation around parent's X axis,
        // matching Three.js's bone.rotation.x (forward/backward leg swing).
        if let Some(quad) = quad_legs {
            let amplitude = 0.45;

            // Leg bones: Euler X replacement (ZYX order so X is outermost = parent-axis swing)
            for &(bone_entity, phase_offset, bind_ez, bind_ey) in &quad.legs {
                if let Ok(mut t) = transforms.get_mut(bone_entity) {
                    let osc_x = (walk_time * speed + phase_offset).sin() * amplitude;
                    t.rotation = Quat::from_euler(
                        bevy::math::EulerRot::ZYX,
                        bind_ez,
                        bind_ey,
                        osc_x,
                    );
                }
            }

            // Foot IK-target bones: very small vertical lift to reduce "stuck foot" look.
            for &(bone_entity, phase_offset, _bind_quat, bind_pos) in &quad.feet {
                if let Ok(mut t) = transforms.get_mut(bone_entity) {
                    let swing = (walk_time * speed + phase_offset).sin();
                    let lift = swing.abs() * 0.015;
                    t.translation = bind_pos + Vec3::new(0.0, lift, 0.0);
                }
            }

            // Body bob
            if let Some(&child) = children.iter().next() {
                if let Ok(mut t) = transforms.get_mut(child) {
                    let bob = (walk_time * speed * 2.0).sin().abs() * 0.15;
                    t.translation.y = bob;
                }
            }
        }

        // Bob + lean on the scene root — only when no leg bones
        if quad_legs.is_none() {
            let bob = (anim.phase).sin() * 0.06;
            let lean = (anim.phase * 0.5).sin() * 0.03;
            if let Some(&child) = children.iter().next() {
                if let Ok(mut t) = transforms.get_mut(child) {
                    t.translation.y = bob;
                    t.rotation = Quat::from_rotation_z(lean);
                }
            }
        }
    }
}

/// Strips root-bone position tracks and all scale tracks from enemy animation clips.
/// Root bone = bone whose parent is NOT a bone (i.e., Hips under Armature).
/// This prevents walk-forward drift while keeping limb position tracks for proper animation.
pub fn strip_enemy_clip_root_motion(
    mut enemies: Query<&mut EnemyClipsNeedStrip>,
    mut clips: ResMut<Assets<AnimationClip>>,
) {
    for mut strip in &mut enemies {
        if strip.stripped { continue; }

        let all_loaded = strip.handles.iter().all(|h| clips.get(h).is_some());
        if !all_loaded { continue; }

        let handles: Vec<_> = strip.handles.clone();
        for handle in &handles {
            if let Some(clip) = clips.get_mut(handle) {
                // For each bone target in the clip:
                for (_target_id, curves) in clip.curves_mut().iter_mut() {
                    // Strip all translation + scale, keep only rotation.
                    // GLTF order: [translation(0), rotation(1), scale(2)]
                    // This is necessary because external anim clips contain absolute
                    // bone positions from a different character's skeleton.
                    if curves.len() >= 2 {
                        let rotation = VariableCurve(curves[1].0.clone_value());
                        curves.clear();
                        curves.push(rotation);
                    }
                }
            }
        }
        info!("Stripped translation + scale from enemy clips (rotation-only mode)");
        strip.stripped = true;
    }
}

/// Programmatic sword-swing for the static knight model mounted on cavalry horses.
pub fn animate_cavalry_knight(
    mut knights: Query<(&Parent, &mut Transform), With<CavalryKnight>>,
    cavalry: Query<Option<&GolemBlocked>, With<Enemy>>,
    time: Res<Time>,
) {
    let t = time.elapsed_secs();
    for (parent, mut transform) in knights.iter_mut() {
        let blocked = cavalry.get(parent.get()).ok().flatten().is_some();
        if blocked {
            // Forward/back thrust (X rotation)
            let thrust = (t * 5.0).sin() * 0.25;
            transform.rotation = Quat::from_rotation_x(thrust);
        } else {
            // Gentle riding bob
            let bob = (t * 3.0).sin() * 0.05;
            transform.rotation = Quat::from_rotation_x(bob);
        }
    }
}

/// Recursively insert AnimationTarget on bone entities for external-anim enemies.
fn insert_enemy_anim_targets(
    commands: &mut Commands,
    entity: Entity,
    player: Entity,
    parent_path: &[Name],
    children_q: &Query<&Children>,
    names: &Query<&Name>,
) {
    let name = names.get(entity)
        .map(|n| n.clone())
        .unwrap_or_else(|_| Name::new(format!("bone_{}", entity.index())));
    let mut path = parent_path.to_vec();
    path.push(name);
    let id = AnimationTargetId::from_names(path.iter());
    commands.entity(entity).insert(AnimationTarget { id, player });
    if let Ok(children) = children_q.get(entity) {
        for &child in children.iter() {
            insert_enemy_anim_targets(commands, child, player, &path, children_q, names);
        }
    }
}

/// Despawn dying enemies after their death timer expires.
pub fn tick_dying_enemies(
    mut commands: Commands,
    mut dying: Query<(Entity, &mut EnemyDying)>,
    time: Res<Time>,
) {
    for (entity, mut d) in &mut dying {
        d.timer -= time.delta_secs();
        if d.timer <= 0.0 {
            commands.entity(entity).despawn_recursive();
        }
    }
}
