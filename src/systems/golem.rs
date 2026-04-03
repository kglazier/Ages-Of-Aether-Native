use bevy::prelude::*;
use bevy::animation::{AnimationTargetId, VariableCurve};
use crate::components::*;

/// Marker for golems that need their animation set up after scene loads.
#[derive(Component)]
pub struct GolemNeedsAnimation;

/// Golem animation clips that need root motion stripped.
#[derive(Component)]
pub struct GolemClipsNeedStrip {
    pub handles: Vec<Handle<AnimationClip>>,
    pub stripped: bool,
}

/// Timer placed on a tower when its golems die. Prevents instant respawn.
#[derive(Component)]
pub struct GolemRespawnTimer(pub f32);

/// Tracks which animation is currently playing and stores graph node indices.
#[derive(Component)]
pub struct GolemAnimState {
    pub idle_node: AnimationNodeIndex,
    pub walk_node: AnimationNodeIndex,
    pub attack_node: AnimationNodeIndex,
    pub current: GolemAnimKind,
    pub player_entity: Entity,
}

#[derive(Clone, Copy, PartialEq)]
pub enum GolemAnimKind {
    Idle,
    Walk,
    Attack,
}

/// When an Earth tower exists without golems and its respawn timer is up, spawn golems.
/// Mountain King spec: 1 elite golem (3x HP, 2x damage).
/// Bramble Grove spec: no golems at all (uses aura).
pub fn spawn_golems(
    mut commands: Commands,
    earth_towers: Query<(Entity, &Element, &AttackDamage, &BuildSpotRef, Option<&GolemRespawnTimer>, Option<&TowerSpec>, Option<&TowerRallyPoint>), With<Tower>>,
    existing_golems: Query<&GolemOwner, With<Golem>>,
    spots: Query<&Transform, With<BuildSpot>>,
    asset_server: Res<AssetServer>,
    time: Res<Time>,
    level_path: Res<crate::resources::LevelPath>,
) {
    for (tower_entity, element, damage, spot_ref, respawn_timer, spec, tower_rally) in &earth_towers {
        if *element != Element::Earth {
            continue;
        }

        // Bramble Grove doesn't spawn golems
        if spec.is_some_and(|s| s.0 == crate::data::TowerSpecialization::BrambleGrove) {
            continue;
        }

        let has_golems = existing_golems.iter().any(|owner| owner.0 == tower_entity);
        if has_golems {
            continue;
        }

        // Tick or start respawn timer
        if let Some(timer) = respawn_timer {
            if timer.0 > 0.0 {
                // Still waiting — tick the timer
                commands.entity(tower_entity).insert(GolemRespawnTimer(timer.0 - time.delta_secs()));
                continue;
            }
            // Timer done — remove it and spawn
            commands.entity(tower_entity).remove::<GolemRespawnTimer>();
        }

        let Ok(spot_transform) = spots.get(spot_ref.0) else {
            continue;
        };
        let tower_pos = spot_transform.translation;

        // Use player-set rally point if available, otherwise find nearest path point
        let nearest_path_pos = if let Some(rally) = tower_rally {
            rally.0
        } else {
            let path = &level_path.0;
            let mut best_pos = path[0];
            let mut best_dist = f32::MAX;
            for i in 0..path.len() - 1 {
                let a = path[i];
                let b = path[i + 1];
                let ab = b - a;
                let ap = tower_pos - a;
                let t = (ap.dot(ab) / ab.dot(ab)).clamp(0.0, 1.0);
                let closest = a + ab * t;
                let dist = tower_pos.distance(closest);
                if dist < best_dist {
                    best_dist = dist;
                    best_pos = closest;
                }
            }
            best_pos
        };

        let is_mountain_king = spec.is_some_and(|s| s.0 == crate::data::TowerSpecialization::MountainKing);
        let golem_count = if is_mountain_king { 1 } else { 2 };
        let golem_hp = if is_mountain_king { 300.0 } else { 100.0 };
        let golem_dmg = if is_mountain_king { damage.0 * 2.0 } else { damage.0 };

        for i in 0..golem_count {
            // Spread golems slightly along the path direction
            let offset = if golem_count == 1 {
                Vec3::ZERO
            } else if i == 0 {
                Vec3::new(0.8, 0.0, 0.0)
            } else {
                Vec3::new(-0.8, 0.0, 0.0)
            };
            let rally = nearest_path_pos + offset;

            // Spawn the skinned mesh scene — animation system will drive the bones
            let scene = asset_server.load("models/golems/golem.glb#Scene0");
            let scale = if is_mountain_king { 80.0 } else { 60.0 };

            commands.spawn((
                SceneRoot(scene),
                Transform::from_translation(rally).with_scale(Vec3::splat(scale)),
                Golem,
                GolemNeedsAnimation,
                GolemOwner(tower_entity),
                GolemRallyPoint(rally),
                BlockingEnemy(None),
                GolemAttack {
                    damage: golem_dmg,
                    cooldown: 1.0,
                    elapsed: 0.0,
                },
                Health {
                    current: golem_hp,
                    max: golem_hp,
                },
                crate::components::GameWorldEntity,
            ));
        }
    }
}

/// After the golem scene loads, find the AnimationPlayer child and attach a multi-clip animation graph.
pub fn setup_golem_animations(
    mut commands: Commands,
    golems: Query<(Entity, &Children), With<GolemNeedsAnimation>>,
    children_q: Query<&Children>,
    anim_players: Query<Entity, With<AnimationPlayer>>,
    asset_server: Res<AssetServer>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
) {
    for (golem_entity, children) in &golems {
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

        let Some(player_entity) = found_player else {
            continue;
        };

        // Build graph with idle, walk, and attack clips
        let idle_clip: Handle<AnimationClip> = asset_server.load("models/golems/golem-idle.glb#Animation0");
        let walk_clip: Handle<AnimationClip> = asset_server.load("models/golems/golem-walk.glb#Animation0");
        let attack_clip: Handle<AnimationClip> = asset_server.load("models/golems/golem-attack.glb#Animation0");

        let strip_handles = vec![idle_clip.clone(), walk_clip.clone(), attack_clip.clone()];

        let mut graph = AnimationGraph::new();
        let idle_node = graph.add_clip(idle_clip, 1.0, graph.root);
        let walk_node = graph.add_clip(walk_clip, 1.0, graph.root);
        let attack_node = graph.add_clip(attack_clip, 1.0, graph.root);
        let graph_handle = graphs.add(graph);

        commands.entity(player_entity).insert(AnimationGraphHandle(graph_handle));

        // Store animation state on the golem entity
        commands.entity(golem_entity).insert((
            GolemAnimState {
                idle_node,
                walk_node,
                attack_node,
                current: GolemAnimKind::Idle,
                player_entity,
            },
            GolemClipsNeedStrip { handles: strip_handles, stripped: false },
        ));
        commands.entity(golem_entity).remove::<GolemNeedsAnimation>();

        info!("Golem {:?} animation graph attached (idle/walk/attack)", golem_entity);
    }
}

/// Start idle playback on newly set up golems.
pub fn play_golem_animations(
    golems: Query<&GolemAnimState, Added<GolemAnimState>>,
    mut players: Query<&mut AnimationPlayer>,
) {
    for anim_state in &golems {
        if let Ok(mut player) = players.get_mut(anim_state.player_entity) {
            player.play(anim_state.idle_node).repeat();
        }
    }
}

/// Switch golem animation based on current behavior (idle, walking, attacking).
pub fn update_golem_animations(
    mut golems: Query<(&mut GolemAnimState, &BlockingEnemy, &GolemAttack, &Transform, &GolemRallyPoint), With<Golem>>,
    enemies: Query<&Transform, (With<Enemy>, Without<Golem>)>,
    mut players: Query<&mut AnimationPlayer>,
) {
    for (mut anim_state, blocking, _attack, golem_transform, rally) in &mut golems {
        let dist_to_rally = golem_transform.translation.distance(rally.0);

        // Walking to rally point takes priority over combat animations
        let desired = if dist_to_rally > 0.3 {
            GolemAnimKind::Walk
        } else if let Some(blocked_entity) = blocking.0 {
            if enemies.get(blocked_entity).is_ok() {
                // Stay in attack animation while fighting — don't flip to idle between swings
                GolemAnimKind::Attack
            } else {
                GolemAnimKind::Idle
            }
        } else {
            GolemAnimKind::Idle
        };

        if desired != anim_state.current {
            anim_state.current = desired;
            if let Ok(mut player) = players.get_mut(anim_state.player_entity) {
                // Stop all clips before starting the new one
                player.stop_all();
                let node = match desired {
                    GolemAnimKind::Idle => anim_state.idle_node,
                    GolemAnimKind::Walk => anim_state.walk_node,
                    GolemAnimKind::Attack => anim_state.attack_node,
                };
                player.play(node).repeat();
            }
        }
    }
}

/// Fix golem materials — the GLB has near-black base_color (0.1, 0.1, 0.1).
pub fn fix_golem_materials(
    golems: Query<&Children, With<Golem>>,
    children_q: Query<&Children>,
    mesh_q: Query<&MeshMaterial3d<StandardMaterial>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut fixed: Local<bool>,
) {
    if *fixed {
        return;
    }
    for children in &golems {
        let mut stack: Vec<Entity> = children.iter().copied().collect();
        while let Some(entity) = stack.pop() {
            if let Ok(mat_handle) = mesh_q.get(entity) {
                if let Some(material) = materials.get_mut(&mat_handle.0) {
                    material.base_color = Color::srgb(0.55, 0.45, 0.35);
                    *fixed = true;
                }
            }
            if let Ok(grandchildren) = children_q.get(entity) {
                stack.extend(grandchildren.iter());
            }
        }
    }
}

/// Assign each golem's BlockingEnemy target (nearest blocked enemy).
/// Blocking/unblocking is now handled by the unified block_enemies system in hero.rs.
pub fn golem_assign_targets(
    mut golems: Query<(Entity, &Transform, &mut BlockingEnemy, &GolemOwner), With<Golem>>,
    enemies: Query<(Entity, &Transform, Option<&Flying>), (With<Enemy>, With<GolemBlocked>)>,
) {
    let engage_range = 3.0;

    // First pass: find nearest blocked enemy for each golem
    let mut golem_targets: Vec<(Entity, Entity, Option<(Entity, f32)>)> = Vec::new();
    for (golem_entity, golem_transform, _blocking, owner) in &golems {
        let golem_pos = golem_transform.translation;
        let mut nearest: Option<(Entity, f32)> = None;
        for (enemy_entity, enemy_transform, flying) in &enemies {
            if flying.is_some() { continue; }
            let dist = golem_pos.distance(enemy_transform.translation);
            if dist <= engage_range {
                if nearest.is_none() || dist < nearest.unwrap().1 {
                    nearest = Some((enemy_entity, dist));
                }
            }
        }
        golem_targets.push((golem_entity, owner.0, nearest));
    }

    // Sibling coordination: if one golem has a target, the other picks nearest too
    let mut tower_has_target: std::collections::HashMap<Entity, bool> = std::collections::HashMap::new();
    for &(_golem, owner, ref target) in &golem_targets {
        if target.is_some() {
            tower_has_target.insert(owner, true);
        }
    }

    for (golem_entity, golem_transform, mut blocking, owner) in &mut golems {
        let own_target = golem_targets.iter().find(|(e, _, _)| *e == golem_entity);
        let has_own = own_target.map(|(_, _, t)| t.is_some()).unwrap_or(false);

        if has_own {
            blocking.0 = own_target.unwrap().2.map(|(e, _)| e);
        } else if tower_has_target.contains_key(&owner.0) {
            // Sibling is engaged — pick nearest blocked enemy
            let golem_pos = golem_transform.translation;
            let mut best: Option<(Entity, f32)> = None;
            for (enemy_entity, enemy_transform, flying) in &enemies {
                if flying.is_some() { continue; }
                let dist = golem_pos.distance(enemy_transform.translation);
                if best.is_none() || dist < best.unwrap().1 {
                    best = Some((enemy_entity, dist));
                }
            }
            blocking.0 = best.map(|(e, _)| e);
        } else {
            blocking.0 = None;
        }
    }
}

/// Place newly blocked enemies into spread positions around the nearest blocker.
/// Uses deterministic lateral + backward offsets relative to the path direction.
pub fn spread_blocked_enemies(
    mut enemies: Query<(Entity, &mut Transform, &PathFollower, &GolemBlocked), With<Enemy>>,
    golems: Query<&Transform, (With<Golem>, Without<Enemy>)>,
    hero_q: Query<&Transform, (With<Hero>, Without<Golem>, Without<Enemy>, Without<crate::components::HeroRespawnTimer>)>,
    mut settled: Local<std::collections::HashSet<Entity>>,
    level_path: Res<crate::resources::LevelPath>,
) {
    // Collect blocker positions (golems + hero if near path)
    let mut blocker_positions: Vec<Vec3> = golems.iter().map(|t| t.translation).collect();
    if let Ok(hero_tf) = hero_q.get_single() {
        let path = &level_path.0;
        let hero_near_path = (0..path.len() - 1).any(|i| {
            let ab = path[i + 1] - path[i];
            let ap = hero_tf.translation - path[i];
            let t = (ap.dot(ab) / ab.dot(ab)).clamp(0.0, 1.0);
            hero_tf.translation.distance(path[i] + ab * t) <= 2.5
        });
        if hero_near_path {
            blocker_positions.push(hero_tf.translation);
        }
    }

    if blocker_positions.is_empty() {
        return;
    }

    // Collect all blocked enemies
    let all_blocked: Vec<(Entity, Vec3)> = enemies
        .iter()
        .map(|(e, t, _, _)| (e, t.translation))
        .collect();

    // Clean up settled set
    settled.retain(|e| all_blocked.iter().any(|(ent, _)| ent == e));

    // Identify new arrivals
    let new_entities: Vec<Entity> = all_blocked.iter()
        .filter(|(e, _)| !settled.contains(e))
        .map(|(e, _)| *e)
        .collect();

    if new_entities.is_empty() {
        return;
    }

    // Get the path for calculating spread direction
    let path = &level_path.0;

    // Count existing settled enemies to know the next slot index
    let mut slot = settled.len() as f32;

    // Collect positions of all settled (already repositioned) blocked enemies
    let settled_positions: Vec<Vec3> = all_blocked.iter()
        .filter(|(e, _)| settled.contains(e))
        .map(|(_, pos)| *pos)
        .collect();

    for new_entity in &new_entities {
        let Ok((_, mut transform, follower, _)) = enemies.get_mut(*new_entity) else {
            continue;
        };

        let enemy_pos = transform.translation;

        // Only reposition if too close to another blocked enemy (overlap)
        let min_spacing = 0.6;
        let overlapping = settled_positions.iter().any(|p| {
            let dx = enemy_pos.x - p.x;
            let dz = enemy_pos.z - p.z;
            (dx * dx + dz * dz).sqrt() < min_spacing
        });

        if overlapping {
            // Nudge backward along the path and slightly lateral — stay near the path
            let seg = follower.segment.min(path.len().saturating_sub(2));
            let path_dir = (path[seg + 1] - path[seg]).normalize_or_zero();
            let lateral = Vec3::new(-path_dir.z, 0.0, path_dir.x);
            let backward = -path_dir;

            let i = slot as i32;
            let side = if i % 2 == 0 { 1.0 } else { -1.0 };
            let lateral_offset = side * 0.4;
            let backward_offset = 0.4 + (i as f32 * 0.3);

            let new_pos = enemy_pos
                + backward * backward_offset
                + lateral * lateral_offset;

            transform.translation.x = new_pos.x;
            transform.translation.z = new_pos.z;
        }
        // Otherwise: enemy stays where it naturally stopped on the path

        settled.insert(*new_entity);
        slot += 1.0;
    }
}

/// Scatter enemies that were just unblocked so they don't blob together.
/// Gives each a unique progress offset and lateral offset for natural spacing.
pub fn scatter_unblocked_enemies(
    mut commands: Commands,
    mut query: Query<(Entity, &mut PathFollower, &NeedsUnblockScatter)>,
) {
    for (entity, mut follower, scatter) in &mut query {
        let i = scatter.0;
        // Stagger progress: push backward along path so they don't blob
        let progress_offset = -0.05 - ((i as f32 * 0.618) % 1.0) * 0.15; // -0.05 to -0.20
        follower.progress = (follower.progress + progress_offset).clamp(0.0, 0.99);

        // Lateral offset: spread perpendicular to path, alternating sides (stay near path)
        let side = if i % 2 == 0 { 1.0 } else { -1.0 };
        let magnitude = 0.4 + (i as f32 * 0.4) % 0.8; // 0.4 to 1.2
        follower.lateral_offset = side * magnitude;

        commands.entity(entity).remove::<NeedsUnblockScatter>();
    }
}

/// Golems attack their blocked enemy, dealing melee damage.
pub fn golem_melee_attack(
    mut golems: Query<(&BlockingEnemy, &mut GolemAttack, &Transform), With<Golem>>,
    mut enemies: Query<&mut Health, With<Enemy>>,
    time: Res<Time>,
) {
    for (blocking, mut attack, _golem_transform) in &mut golems {
        attack.elapsed += time.delta_secs();

        if let Some(blocked_entity) = blocking.0 {
            if attack.elapsed >= attack.cooldown {
                attack.elapsed = 0.0;
                if let Ok(mut health) = enemies.get_mut(blocked_entity) {
                    health.current -= attack.damage;
                }
            }
        }
    }
}

/// Move golems toward their rally point. They stay there and fight from that position.
pub fn golem_movement(
    mut golems: Query<(&mut Transform, &GolemRallyPoint, &BlockingEnemy), With<Golem>>,
    enemies: Query<&Transform, (With<Enemy>, Without<Golem>)>,
    time: Res<Time>,
) {
    let speed = 3.0;

    for (mut golem_transform, rally, blocking) in &mut golems {
        // Always move toward rally point
        let target = rally.0;

        let direction = target - golem_transform.translation;
        let distance = direction.length();

        if distance > 0.3 {
            let step = speed * time.delta_secs();
            if step >= distance {
                // Snap to target to avoid oscillation
                golem_transform.translation = target;
            } else {
                golem_transform.translation += direction.normalize() * step;
            }

            let look_dir = direction.normalize();
            if look_dir.length_squared() > 0.001 {
                let target_rot =
                    Quat::from_rotation_y(f32::atan2(look_dir.x, look_dir.z));
                golem_transform.rotation =
                    golem_transform.rotation.slerp(target_rot, 0.15);
            }
        } else if let Some(blocked_entity) = blocking.0 {
            // At rally point — face the nearest blocked enemy
            if let Ok(enemy_transform) = enemies.get(blocked_entity) {
                let look_dir = (enemy_transform.translation - golem_transform.translation).normalize();
                if look_dir.length_squared() > 0.001 {
                    let target_rot =
                        Quat::from_rotation_y(f32::atan2(look_dir.x, look_dir.z));
                    golem_transform.rotation =
                        golem_transform.rotation.slerp(target_rot, 0.15);
                }
            }
        }
    }
}

/// Blocked enemies attack the golem back, dealing damage.
pub fn enemies_attack_golem(
    mut golems: Query<(&Transform, &BlockingEnemy, &mut Health), With<Golem>>,
    enemies: Query<&Transform, With<GolemBlocked>>,
    time: Res<Time>,
) {
    let dps = 2.0;
    let dt = time.delta_secs();

    for (golem_transform, _blocking, mut health) in &mut golems {
        let mut attackers = 0u32;
        for enemy_transform in &enemies {
            let dist = golem_transform.translation.distance(enemy_transform.translation);
            if dist < 3.0 {
                attackers += 1;
            }
        }

        if attackers > 0 {
            health.current -= dps * attackers as f32 * dt;
        }
    }
}

/// Despawn dead golems and start a 12s respawn timer on the tower.
pub fn check_golem_death(
    mut commands: Commands,
    golems: Query<(Entity, &Health, &GolemOwner), With<Golem>>,
    towers: Query<Option<&GolemRespawnTimer>, With<Tower>>,
) {
    for (entity, health, owner) in &golems {
        if health.current <= 0.0 {
            // Only start timer if tower doesn't already have one
            if let Ok(existing_timer) = towers.get(owner.0) {
                if existing_timer.is_none() {
                    commands.entity(owner.0).insert(GolemRespawnTimer(12.0));
                }
            }
            commands.entity(entity).despawn_recursive();
        }
    }
}

/// When an earth tower is sold, despawn its golems too.
pub fn cleanup_orphan_golems(
    mut commands: Commands,
    golems: Query<(Entity, &GolemOwner), With<Golem>>,
    towers: Query<Entity, With<Tower>>,
) {
    for (golem_entity, owner) in &golems {
        if towers.get(owner.0).is_err() {
            commands.entity(golem_entity).despawn_recursive();
        }
    }
}

/// Strips root motion (Hips translation) from golem animation clips.
pub fn strip_golem_root_motion(
    mut golems: Query<&mut GolemClipsNeedStrip>,
    mut clips: ResMut<Assets<AnimationClip>>,
) {
    let hips_id = AnimationTargetId::from_names(
        [Name::new("Armature"), Name::new("mixamorig:Hips")].iter(),
    );

    for mut strip in &mut golems {
        if strip.stripped { continue; }

        let all_loaded = strip.handles.iter().all(|h| clips.get(h).is_some());
        if !all_loaded { continue; }

        let handles: Vec<_> = strip.handles.clone();
        for handle in &handles {
            if let Some(clip) = clips.get_mut(handle) {
                for (target_id, curves) in clip.curves_mut().iter_mut() {
                    if *target_id == hips_id {
                        // Root bone: strip position tracks, keep rotation
                        if curves.len() >= 2 {
                            let rotation = VariableCurve(curves[1].0.clone_value());
                            curves.clear();
                            curves.push(rotation);
                        }
                    } else if curves.len() == 3 {
                        // Non-root: strip scale, keep translation + rotation
                        curves.truncate(2);
                    }
                }
            }
        }
        strip.stripped = true;
        info!("Golem root motion stripped");
    }
}
