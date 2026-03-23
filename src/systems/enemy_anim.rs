use bevy::prelude::*;
use bevy::animation::{AnimationTargetId};
use crate::components::*;
use crate::data::*;

/// Tracks walk clip handle for root motion stripping.
#[derive(Component)]
pub struct EnemyWalkClip {
    pub handle: Handle<AnimationClip>,
    pub stripped: bool,
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

        // Check if anim_indices are all zero (unconfigured) — force procedural animation
        let has_configured_indices = stats.anim_indices != [0; 4];

        if let Some(anim_files) = stats.anim_files {
            // External animation mode (humanoid Mixamo rigs)
            // Need an AnimationPlayer — insert one manually if not found
            let player_entity = if let Some(pe) = player_entity {
                pe
            } else {
                // Find a suitable host entity in the hierarchy
                let host = children.iter().next().copied();
                let Some(host) = host else { continue };
                let mut candidate = host;
                let mut sub_stack: Vec<Entity> = vec![host];
                while let Some(e) = sub_stack.pop() {
                    candidate = e;
                    if let Ok(gc) = children_q.get(e) {
                        if !gc.is_empty() {
                            candidate = e;
                            break;
                        }
                        sub_stack.extend(gc.iter());
                    }
                }
                commands.entity(candidate).insert(AnimationPlayer::default());
                candidate
            };

            let walk_clip: Handle<AnimationClip> = asset_server.load(format!("{}#Animation0", anim_files[0]));
            let idle_clip = asset_server.load(format!("{}#Animation0", anim_files[1]));
            let attack_clip = asset_server.load(format!("{}#Animation0", anim_files[2]));
            let death_clip = asset_server.load(format!("{}#Animation0", anim_files[3]));

            let walk_clip_handle = walk_clip.clone();

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
                EnemyWalkClip { handle: walk_clip_handle, stripped: false },
            ));
        } else if has_configured_indices && player_entity.is_some() {
            // Embedded animation mode (blobs with built-in clips)
            let pe = player_entity.unwrap();
            let [walk_idx, idle_idx, attack_idx, death_idx] = stats.anim_indices;

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
        } else if children.len() > 0 {
            // Animal/dinosaur model — use procedural walk animation with leg bones.
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

/// Discovers quadruped leg bones by name after the GLTF scene loads.
/// Looks for bones named FrontLeg.L/R, BackLeg.L/R (or Mixamo equivalents).
pub fn discover_leg_bones(
    mut commands: Commands,
    enemies: Query<(Entity, &Children), With<NeedsLegDiscovery>>,
    children_q: Query<&Children>,
    names: Query<&Name>,
) {
    for (enemy_entity, children) in &enemies {
        let mut leg_bones: Vec<(Entity, f32)> = Vec::new();
        let mut stack: Vec<Entity> = children.iter().copied().collect();
        while let Some(entity) = stack.pop() {
            if let Ok(name) = names.get(entity) {
                let n = name.as_str().to_lowercase();
                // Non-Mixamo rig (stegosaurus, triceratops, etc.)
                if n.ends_with("frontleg.l") { leg_bones.push((entity, std::f32::consts::PI)); }
                else if n.ends_with("frontleg.r") { leg_bones.push((entity, 0.0)); }
                else if n.ends_with("backleg.l") { leg_bones.push((entity, 0.0)); }
                else if n.ends_with("backleg.r") { leg_bones.push((entity, std::f32::consts::PI)); }
                // Mixamo rig (used as quadruped)
                else if n.ends_with("leftupleg") { leg_bones.push((entity, 0.0)); }
                else if n.ends_with("rightupleg") { leg_bones.push((entity, std::f32::consts::PI)); }
                else if n.ends_with("leftarm") { leg_bones.push((entity, std::f32::consts::PI)); }
                else if n.ends_with("rightarm") { leg_bones.push((entity, 0.0)); }
            }
            if let Ok(gc) = children_q.get(entity) {
                stack.extend(gc.iter());
            }
        }
        if leg_bones.len() >= 4 {
            commands.entity(enemy_entity).insert(QuadLegBones(leg_bones));
        }
        commands.entity(enemy_entity).remove::<NeedsLegDiscovery>();
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

        // Scale animation speed with movement speed (stop when blocked)
        let move_speed = if blocked.is_some() {
            0.0
        } else {
            follower.map(|f| f.speed).unwrap_or(2.0)
        };
        anim.phase += dt * move_speed * 4.0;

        // Animate leg bones if discovered
        if let Some(legs) = quad_legs {
            let amplitude = 0.35; // ~20 degrees swing
            for &(bone_entity, phase_offset) in &legs.0 {
                if let Ok(mut t) = transforms.get_mut(bone_entity) {
                    t.rotation = Quat::from_rotation_x(
                        (anim.phase + phase_offset).sin() * amplitude
                    );
                }
            }
        }

        // Bob + lean on the scene root
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

/// Cancel root motion for humanoid enemies using external Mixamo walk animations.
/// The walk clip moves the character forward — we zero only X/Z of the armature
/// root each frame, preserving Y so the model doesn't sink into the ground.
pub fn cancel_enemy_root_motion(
    enemies: Query<(&Children, &EnemyAnimState), Without<EnemyDying>>,
    children_q: Query<&Children>,
    mut transforms: Query<&mut Transform>,
) {
    for (children, _anim_state) in &enemies {
        // The GLTF scene root is the first child; the armature root is inside it.
        // Zero out X/Z to prevent walk-forward drift, keep Y for ground placement.
        for &child in children.iter() {
            if let Ok(grandchildren) = children_q.get(child) {
                for &gc in grandchildren.iter() {
                    if let Ok(mut t) = transforms.get_mut(gc) {
                        t.translation.x = 0.0;
                        t.translation.z = 0.0;
                    }
                }
            }
        }
    }
}

/// Strips root-bone position tracks from enemy walk clips after they load.
/// This prevents root motion drift (the walk animation moving the character forward).
/// Only translation curves are removed — rotation curves are kept for limb movement.
pub fn strip_enemy_walk_root_motion(
    mut enemies: Query<&mut EnemyWalkClip>,
    mut clips: ResMut<Assets<AnimationClip>>,
) {
    // The Hips bone is the root motion source in Mixamo animations
    let hips_id = AnimationTargetId::from_names(
        [Name::new("Armature"), Name::new("mixamorig:Hips")].iter(),
    );

    for mut walk_clip in &mut enemies {
        if walk_clip.stripped { continue; }
        let handle = walk_clip.handle.clone();
        if let Some(clip) = clips.get_mut(&handle) {
            // Remove the Hips bone curves entirely (translation + rotation + scale)
            // This prevents the walk animation from moving the model
            if clip.curves_mut().remove(&hips_id).is_some() {
                info!("Stripped Hips root motion from enemy walk clip");
            }
            walk_clip.stripped = true;
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
