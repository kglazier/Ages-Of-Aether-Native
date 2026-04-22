use bevy::prelude::*;
use bevy::animation::{AnimationTarget, AnimationTargetId, VariableCurve};
use std::collections::HashMap;
use crate::components::*;
use crate::data::*;
use crate::resources::*;

/// Tracks which animation is currently playing and stores graph node indices.
#[derive(Component)]
pub struct HeroAnimState {
    pub idle_node: AnimationNodeIndex,
    pub walk_node: AnimationNodeIndex,
    pub attack_node: AnimationNodeIndex,
    pub current: HeroAnimKind,
    pub player_entity: Entity,
    /// Set by hero_auto_attack when an attack fires, cleared by update_hero_animations.
    pub attack_triggered: bool,
    /// The mixamorig:Hips bone entity — used to cancel root motion from animation clips.
    pub hips_entity: Option<Entity>,
    /// Bind-pose position of the Hips bone (captured at setup). Root motion cancellation
    /// resets the Hips to this position each frame instead of (0,0,0).
    pub hips_bind_pos: Vec3,
    /// Handle to the run animation clip — used to strip root motion curves after loading.
    pub run_clip_handle: Handle<AnimationClip>,
    /// Whether we've already stripped root motion curves from the run clip.
    pub run_clip_stripped: bool,
    /// When set, ALL bone translations/scales are reset to these bind-pose values every frame,
    /// keeping only rotation from the animation. Used for models with bone-scale mismatch
    /// (e.g. Northern Outsider whose bones are ~100x larger than Mixamo animation expects).
    pub bone_bind_poses: Option<HashMap<Entity, (Vec3, Vec3)>>,
    /// Skip Hips root-motion stripping and per-frame Hips reset. Needed when the
    /// animation's Hips values are compatible but the model's bind-pose Hips are not.
    pub skip_hips_reset: bool,
    /// All clip handles that need non-rotation curves stripped (translation + scale removed).
    pub rotation_only_clips: Vec<Handle<AnimationClip>>,
    /// Whether non-rotation curves have been stripped from the clips.
    pub rotation_only_stripped: bool,
    /// Armature entity — used for post-animation rotation correction.
    pub armature_entity: Option<Entity>,
    /// Target rotation to SET on the armature AFTER animation, in PostUpdate.
    /// Computed as: correction_quat * bind_pose_rotation.
    /// Fixes models exported with wrong up-axis (e.g. Pharaoh).
    pub armature_rotation_fix: Option<Quat>,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum HeroAnimKind {
    Idle,
    Walk,
    Attack,
}

/// Spawns the hero at level start.
pub fn spawn_hero(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    active_hero: Res<ActiveHeroType>,
    existing_heroes: Query<Entity, With<Hero>>,
    current_level: Res<crate::resources::CurrentLevel>,
    save_data: Option<Res<crate::save::SaveData>>,
    no_hero: Res<crate::resources::NoHeroSelected>,
) {
    // Skip entirely in towers-only mode
    if no_hero.0 {
        return;
    }
    // Don't spawn if hero already exists
    if !existing_heroes.is_empty() {
        return;
    }

    let stats = hero_stats(active_hero.0);
    let spawn_pos = level_hero_spawn(current_level.0);
    let scene = asset_server.load(format!("{}#Scene0", stats.model_path));

    // Apply TacticalMastery cooldown reduction
    let cd_mult = save_data.as_ref().map(|s| s.cooldown_mult()).unwrap_or(1.0);

    commands.spawn((
        SceneRoot(scene),
        // y_offset raises the model visually; blocking/attack use XZ-only distance.
        Transform::from_translation(spawn_pos + Vec3::new(0.0, stats.model_y_offset, 0.0))
            .with_scale(Vec3::splat(stats.model_scale)),
        Hero,
        HeroModelSetup { y_offset: stats.model_y_offset, rotation_x: stats.model_rotation_x },
        Health {
            current: stats.hp,
            max: stats.hp,
        },
        HeroMoveTarget(None),
        HeroAttackTimer {
            cooldown: 1.0 / stats.attack_speed,
            elapsed: 0.0,
        },
        HeroAttackRange(stats.attack_range),
        HeroAttackDamage(stats.damage),
        HeroMoveSpeed(stats.move_speed),
        HeroNeedsAnimation,
        GameWorldEntity,
        {
            let defs = hero_abilities(active_hero.0);
            HeroAbilities {
                cooldowns: [0.0; 3],
                max_cooldowns: [
                    defs[0].cooldown * cd_mult,
                    defs[1].cooldown * cd_mult,
                    defs[2].cooldown * cd_mult,
                ],
            }
        },
    ));

    info!("Hero spawned: {}", stats.name);
}

/// Consumes the HeroMoveCommand resource and applies it to the hero.
pub fn hero_consume_move_command(
    mut move_cmd: ResMut<HeroMoveCommand>,
    mut hero_q: Query<&mut HeroMoveTarget, (With<Hero>, Without<HeroRespawnTimer>)>,
) {
    if let Some(target) = move_cmd.0.take() {
        for mut move_target in &mut hero_q {
            move_target.0 = Some(target);
        }
    }
}

/// Moves the hero toward its move target.
pub fn hero_movement(
    mut hero_q: Query<
        (&mut Transform, &mut HeroMoveTarget, &HeroMoveSpeed, Option<&HeroVisualOffset>),
        (With<Hero>, Without<HeroRespawnTimer>),
    >,
    time: Res<Time>,
) {
    for (mut transform, mut move_target, speed, visual_offset) in &mut hero_q {
        let Some(target) = move_target.0 else {
            continue;
        };

        let current = Vec3::new(transform.translation.x, 0.0, transform.translation.z);
        let goal = Vec3::new(target.x, 0.0, target.z);
        let diff = goal - current;
        let dist = diff.length();

        if dist < 0.3 {
            // Arrived
            move_target.0 = None;
            continue;
        }

        let move_delta = diff.normalize() * speed.0 * time.delta_secs();
        if move_delta.length() >= dist {
            transform.translation.x = target.x;
            transform.translation.z = target.z;
            move_target.0 = None;
        } else {
            transform.translation.x += move_delta.x;
            transform.translation.z += move_delta.z;
        }

        // Face movement direction, compensating entity position so the visual
        // center (Hips) stays in place during rotation (prevents jump for offset models)
        let offset = visual_offset.map_or(Vec3::ZERO, |o| o.0);
        let old_world_offset = transform.rotation * offset;

        let look_target = Vec3::new(
            transform.translation.x + diff.x,
            transform.translation.y,
            transform.translation.z + diff.z,
        );
        transform.look_at(look_target, Vec3::Y);
        transform.rotate_y(std::f32::consts::PI);

        if offset != Vec3::ZERO {
            let new_world_offset = transform.rotation * offset;
            let shift = old_world_offset - new_world_offset;
            transform.translation.x += shift.x;
            transform.translation.z += shift.z;
        }
    }
}

/// Hero auto-attacks the nearest enemy within range.
/// Only fires when the hero is stationary (no active move target).
pub fn hero_auto_attack(
    mut hero_q: Query<
        (&Transform, &mut HeroAttackTimer, &HeroAttackRange, &HeroAttackDamage, &HeroMoveTarget, Option<&mut HeroAnimState>),
        (With<Hero>, Without<HeroRespawnTimer>),
    >,
    mut enemies: Query<(Entity, &Transform, &mut Health, &Armor, Option<&GolemBlocked>), With<Enemy>>,
    time: Res<Time>,
) {
    for (hero_tf, mut timer, range, damage, move_target, anim_state) in &mut hero_q {
        timer.elapsed += time.delta_secs();
        if timer.elapsed < timer.cooldown {
            continue;
        }

        // Don't auto-attack while moving — attack anim would be immediately
        // interrupted by the active move command.
        if move_target.0.is_some() {
            continue;
        }

        // Find nearest enemy: prefer blocked enemies in a wide range, fall back to any in attack range
        let mut best: Option<(Entity, f32)> = None;
        let mut best_unblocked: Option<(Entity, f32)> = None;
        let hero_xz = Vec3::new(hero_tf.translation.x, 0.0, hero_tf.translation.z);
        for (entity, enemy_tf, _, _, blocked) in &enemies {
            let enemy_xz = Vec3::new(enemy_tf.translation.x, 0.0, enemy_tf.translation.z);
            let dist = hero_xz.distance(enemy_xz);
            if blocked.is_some() && dist <= 4.0 {
                if best.is_none() || dist < best.unwrap().1 {
                    best = Some((entity, dist));
                }
            } else if dist <= range.0 {
                if best_unblocked.is_none() || dist < best_unblocked.unwrap().1 {
                    best_unblocked = Some((entity, dist));
                }
            }
        }
        let best = best.or(best_unblocked);

        if let Some((target, _dist)) = best {
            timer.elapsed = 0.0;

            // Trigger attack animation
            if let Some(mut anim) = anim_state {
                anim.attack_triggered = true;
            }

            // Hero deals physical damage (reduced by armor)
            if let Ok((_, _, mut health, armor, _)) = enemies.get_mut(target) {
                let reduction = armor.physical / (armor.physical + 100.0);
                let reduced = damage.0 * (1.0 - reduction);
                health.current -= reduced;
            }
        }
    }
}

/// Checks if hero HP <= 0 and starts respawn timer.
pub fn hero_death_check(
    mut commands: Commands,
    hero_q: Query<(Entity, &Health, &Transform), (With<Hero>, Without<HeroRespawnTimer>)>,
    active_hero: Res<ActiveHeroType>,
) {
    for (entity, health, transform) in &hero_q {
        if health.current <= 0.0 {
            let stats = hero_stats(active_hero.0);
            commands.entity(entity).insert((
                HeroRespawnTimer {
                    remaining: stats.respawn_time,
                    total: stats.respawn_time,
                    death_pos: transform.translation,
                },
                Visibility::Hidden,
            ));
            info!("Hero died! Respawning in {:.0}s", stats.respawn_time);
        }
    }
}

/// Ticks down the respawn timer and revives the hero.
pub fn hero_respawn_tick(
    mut commands: Commands,
    mut hero_q: Query<(Entity, &mut Health, &mut HeroRespawnTimer, &mut Transform), With<Hero>>,
    active_hero: Res<ActiveHeroType>,
    time: Res<Time>,
) {
    for (entity, mut health, mut timer, mut transform) in &mut hero_q {
        timer.remaining -= time.delta_secs();
        if timer.remaining <= 0.0 {
            let stats = hero_stats(active_hero.0);
            health.current = stats.hp;
            health.max = stats.hp;
            // Respawn at the position where the hero died
            transform.translation = timer.death_pos;

            commands.entity(entity).remove::<HeroRespawnTimer>();
            commands.entity(entity).insert(Visibility::Visible);
            info!("Hero respawned!");
        }
    }
}

/// Applies visual rotation and Y offset to the hero's scene child once it loads.
/// This keeps the hero entity's transform clean for look_at and distance checks,
/// while the scene child handles model-specific corrections (Z-up models, etc).
pub fn apply_hero_model_offset(
    mut commands: Commands,
    hero_q: Query<(Entity, &Children, &HeroModelSetup), With<Hero>>,
    mut transforms: Query<&mut Transform>,
) {
    for (entity, children, setup) in &hero_q {
        // Apply to first child (the scene root)
        if let Some(&child) = children.iter().next() {
            if let Ok(mut tf) = transforms.get_mut(child) {
                if setup.rotation_x != 0.0 {
                    tf.rotate_x(setup.rotation_x);
                }
                // Y offset is in world space, but child is scaled by parent.
                // Convert: world_offset / parent_scale
                // Parent scale is uniform, so we can read it from the parent entity.
                // For now, just apply directly — the offset was designed for entity-level.
                // We skip child-level Y offset since model_y_offset is on the entity transform.
                commands.entity(entity).remove::<HeroModelSetup>();
            }
        }
    }
}

/// Spawns 3D health bar and selection ring for the hero once it appears.
pub fn spawn_hero_visuals(
    mut commands: Commands,
    hero_q: Query<Entity, Added<Hero>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for _entity in &hero_q {
        // Hero health bar background
        commands.spawn((
            Mesh3d(meshes.add(Rectangle::new(1.2, 0.14))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgba(0.0, 0.0, 0.0, 0.7),
                alpha_mode: AlphaMode::Blend,
                unlit: true,
                double_sided: true,
                ..default()
            })),
            Transform::default(),
            HeroHealthBarBg3d,
            GameWorldEntity,
        ));

        // Hero health bar fill
        commands.spawn((
            Mesh3d(meshes.add(Rectangle::new(1.1, 0.1))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgb(0.2, 0.9, 0.2),
                unlit: true,
                double_sided: true,
                ..default()
            })),
            Transform::default(),
            HeroHealthBar3d,
            GameWorldEntity,
        ));

        // Glowing selection ring on ground
        commands.spawn((
            Mesh3d(meshes.add(Annulus::new(0.9, 1.1))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgba(0.5, 0.8, 1.0, 0.4),
                emissive: LinearRgba::new(0.5, 0.8, 1.0, 1.0),
                alpha_mode: AlphaMode::Blend,
                unlit: true,
                double_sided: true,
                ..default()
            })),
            Transform::default(),
            HeroSelectionRing,
            GameWorldEntity,
        ));
    }
}

/// Updates hero 3D health bar position and the selection ring.
pub fn update_hero_visuals(
    hero_q: Query<(&Transform, &GlobalTransform, &Health, Option<&HeroRespawnTimer>, Option<&HeroAnimState>), With<Hero>>,
    mut bar_q: Query<
        (&mut Transform, &MeshMaterial3d<StandardMaterial>, &HeroHealthBar3d),
        (Without<Hero>, Without<HeroHealthBarBg3d>, Without<HeroSelectionRing>),
    >,
    mut bg_q: Query<
        &mut Transform,
        (With<HeroHealthBarBg3d>, Without<Hero>, Without<HeroHealthBar3d>, Without<HeroSelectionRing>),
    >,
    mut ring_q: Query<
        &mut Transform,
        (With<HeroSelectionRing>, Without<Hero>, Without<HeroHealthBar3d>, Without<HeroHealthBarBg3d>),
    >,
    camera_q: Query<
        &Transform,
        (With<Camera3d>, Without<Hero>, Without<HeroHealthBar3d>, Without<HeroHealthBarBg3d>, Without<HeroSelectionRing>),
    >,
    global_transforms: Query<&GlobalTransform, Without<Hero>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    selection: Res<Selection>,
    time: Res<Time>,
) {
    let Ok((_hero_tf, hero_global, health, respawn, anim_state)) = hero_q.get_single() else {
        return;
    };
    let Ok(cam_tf) = camera_q.get_single() else {
        return;
    };

    let is_dead = respawn.is_some();

    // Track the Hips bone's world position when available — this gives us the model's
    // actual visual center, accounting for scene-root rotation and bone offsets.
    // Falls back to entity position for heroes without anim state yet.
    let world_pos = hero_global.translation();
    let visual_xz = anim_state
        .and_then(|a| a.hips_entity)
        .and_then(|hips| global_transforms.get(hips).ok())
        .map(|gt| {
            let p = gt.translation();
            (p.x, p.z)
        })
        .unwrap_or((world_pos.x, world_pos.z));

    let bar_y = (world_pos.y + 1.8).max(2.5);
    let bar_pos = Vec3::new(visual_xz.0, bar_y, visual_xz.1);

    // Update health bar fill
    for (mut transform, mat_handle, _) in &mut bar_q {
        if is_dead {
            transform.translation = Vec3::new(0.0, -100.0, 0.0); // hide off-screen
        } else {
            let hp_pct = (health.current / health.max).clamp(0.0, 1.0);
            let to_cam = (cam_tf.translation - bar_pos).normalize_or_zero();
            // Anchor bar to the left: offset by half the missing width in camera-right direction
            let cam_right = cam_tf.rotation * Vec3::X;
            let bar_width = 1.1; // match Rectangle width in spawn_hero_visuals
            let left_offset = cam_right * (bar_width * 0.5 * (hp_pct - 1.0));
            transform.translation = bar_pos + to_cam * 0.02 + left_offset;
            transform.rotation = cam_tf.rotation;
            transform.scale = Vec3::new(hp_pct, 1.0, 1.0);

            // Color
            if let Some(mat) = materials.get_mut(&mat_handle.0) {
                mat.base_color = if hp_pct > 0.5 {
                    let t = (1.0 - hp_pct) * 2.0;
                    Color::srgb(t, 1.0, 0.0)
                } else {
                    let t = hp_pct * 2.0;
                    Color::srgb(1.0, t, 0.0)
                };
            }
        }
    }

    // Update health bar background
    for mut transform in &mut bg_q {
        if is_dead {
            transform.translation = Vec3::new(0.0, -100.0, 0.0);
        } else {
            transform.translation = bar_pos;
            transform.rotation = cam_tf.rotation;
        }
    }

    // Update selection ring — only visible when hero is selected
    let hero_selected = matches!(*selection, Selection::Hero);
    for mut transform in &mut ring_q {
        if is_dead || !hero_selected {
            transform.translation = Vec3::new(0.0, -100.0, 0.0);
        } else {
            transform.translation = Vec3::new(
                visual_xz.0,
                0.05,
                visual_xz.1,
            );
            // Flat on ground
            transform.rotation = Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2);
            // Gentle pulse
            let pulse = 1.0 + (time.elapsed_secs() * 2.0).sin() * 0.1;
            transform.scale = Vec3::splat(pulse);
        }
    }
}

/// Shows/hides a move target marker on the ground where the hero is heading.
pub fn update_hero_move_marker(
    mut commands: Commands,
    hero_q: Query<&HeroMoveTarget, With<Hero>>,
    mut marker_q: Query<(Entity, &mut Transform, &mut Visibility), With<HeroMoveMarker>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    time: Res<Time>,
) {
    let target = hero_q.iter().next().and_then(|mt| mt.0);

    if let Some(target_pos) = target {
        if let Ok((_entity, mut transform, mut vis)) = marker_q.get_single_mut() {
            transform.translation = Vec3::new(target_pos.x, 0.08, target_pos.z);
            let pulse = 0.8 + (time.elapsed_secs() * 4.0).sin() * 0.2;
            transform.scale = Vec3::splat(pulse);
            *vis = Visibility::Visible;
        } else {
            // Spawn marker
            commands.spawn((
                Mesh3d(meshes.add(Annulus::new(0.2, 0.35))),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: Color::srgba(0.4, 1.0, 0.4, 0.6),
                    emissive: LinearRgba::new(0.3, 0.8, 0.3, 1.0),
                    alpha_mode: AlphaMode::Blend,
                    unlit: true,
                    double_sided: true,
                    ..default()
                })),
                Transform::from_translation(Vec3::new(target_pos.x, 0.08, target_pos.z))
                    .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
                HeroMoveMarker,
                GameWorldEntity,
            ));
        }
    } else {
        // Hide marker when not moving
        for (_, _, mut vis) in &mut marker_q {
            *vis = Visibility::Hidden;
        }
    }
}

/// Unified blocking system — every frame, re-evaluates which enemies are blocked
/// by any blocker (hero or golem). Matches the original Three.js approach:
/// clear blocked, then set blocked for enemies in range of any blocker.
/// Applies a random spread offset when an enemy first becomes blocked.
pub fn block_enemies(
    mut commands: Commands,
    hero_q: Query<&Transform, (With<Hero>, Without<HeroRespawnTimer>, Without<Enemy>, Without<Golem>)>,
    golems: Query<&Transform, (With<Golem>, Without<Hero>, Without<Enemy>)>,
    mut enemies: Query<(Entity, &mut Transform, Option<&Flying>, Option<&GolemBlocked>, Option<&BlockOffset>), With<Enemy>>,
    level_path: Res<crate::resources::LevelPath>,
) {
    let hero_range = 1.5;
    let golem_range = 1.8;

    // Collect blocker positions + ranges
    let mut blockers: Vec<(Vec3, f32)> = Vec::new();
    if let Ok(hero_tf) = hero_q.get_single() {
        // Only block if hero is near the path (use XZ distance to ignore model Y offset)
        let path = &level_path.0;
        let hero_xz = Vec3::new(hero_tf.translation.x, 0.0, hero_tf.translation.z);
        let hero_near_path = (0..path.len() - 1).any(|i| {
            let a = Vec3::new(path[i].x, 0.0, path[i].z);
            let b = Vec3::new(path[i + 1].x, 0.0, path[i + 1].z);
            let ab = b - a;
            let ap = hero_xz - a;
            let t = (ap.dot(ab) / ab.dot(ab)).clamp(0.0, 1.0);
            hero_xz.distance(a + ab * t) <= 2.5
        });
        if hero_near_path {
            blockers.push((hero_xz, hero_range));
        }
    }
    for golem_tf in &golems {
        blockers.push((golem_tf.translation, golem_range));
    }

    for (entity, mut transform, flying, was_blocked, block_offset) in &mut enemies {
        if flying.is_some() { continue; }

        // Check if in range of any blocker (XZ distance to ignore Y offsets)
        // Use hysteresis: smaller range to enter blocked state, larger to exit
        let enemy_xz = Vec3::new(transform.translation.x, 0.0, transform.translation.z);
        let already_blocked = was_blocked.is_some();
        let hysteresis = if already_blocked { 0.3 } else { 0.0 }; // small buffer to prevent flicker
        let in_range = blockers.iter().any(|(pos, range)| {
            let blocker_xz = Vec3::new(pos.x, 0.0, pos.z);
            blocker_xz.distance(enemy_xz) <= *range + hysteresis
        });

        if in_range && !already_blocked {
            // Newly blocked — apply random spread offset
            let angle = (entity.index() as f32 * 2.3571) % (std::f32::consts::PI * 2.0);
            let dist = 0.3 + ((entity.index() as f32 * 1.7321) % 0.5);
            let offset = Vec3::new(angle.cos() * dist, 0.0, angle.sin() * dist);
            transform.translation.x += offset.x;
            transform.translation.z += offset.z;
            commands.entity(entity).insert((GolemBlocked, BlockOffset(offset)));
        } else if !in_range && already_blocked {
            // Unblocked — remove offset and GolemBlocked
            if let Some(offset) = block_offset {
                transform.translation.x -= offset.0.x;
                transform.translation.z -= offset.0.z;
            }
            commands.entity(entity).remove::<GolemBlocked>();
            commands.entity(entity).remove::<BlockOffset>();
        }
    }
}

/// Enemies within melee range attack the hero.
pub fn enemies_attack_hero(
    mut hero_q: Query<(&Transform, &mut Health, Option<&HeroDamageReduction>), (With<Hero>, Without<HeroRespawnTimer>, Without<Enemy>)>,
    enemies: Query<(&Transform, Option<&GolemBlocked>), With<Enemy>>,
    time: Res<Time>,
) {
    let Ok((hero_tf, mut hero_health, dr_buff)) = hero_q.get_single_mut() else {
        return;
    };

    // All blocked enemies near the hero deal damage (blocked = in melee)
    let dps_per_enemy = 5.0;
    let attack_range = 3.0;
    let dr_factor = dr_buff.map(|b| b.factor).unwrap_or(1.0);
    for (enemy_tf, blocked) in &enemies {
        if blocked.is_none() { continue; }
        let dist_xz = Vec3::new(hero_tf.translation.x, 0.0, hero_tf.translation.z)
            .distance(Vec3::new(enemy_tf.translation.x, 0.0, enemy_tf.translation.z));
        if dist_xz < attack_range {
            hero_health.current -= dps_per_enemy * dr_factor * time.delta_secs();
        }
    }
}

/// Passive HP regeneration: hero heals slowly when not taking damage.
/// Tracks a "last hit" timer — regen starts after 3 seconds of no damage.
pub fn hero_passive_regen(
    mut hero_q: Query<(&mut Health, Option<&HeroRespawnTimer>), With<Hero>>,
    enemies: Query<(&Transform, Option<&GolemBlocked>), With<Enemy>>,
    hero_pos_q: Query<&Transform, (With<Hero>, Without<Enemy>)>,
    time: Res<Time>,
    mut no_damage_timer: Local<f32>,
) {
    let Ok((mut health, respawn)) = hero_q.get_single_mut() else { return };
    if respawn.is_some() { return; }
    if health.current >= health.max { return; }

    // Check if hero is being attacked (blocked enemies nearby)
    let is_being_attacked = if let Ok(hero_tf) = hero_pos_q.get_single() {
        enemies.iter().any(|(enemy_tf, blocked)| {
            blocked.is_some() && hero_tf.translation.distance(enemy_tf.translation) < 3.0
        })
    } else {
        false
    };

    if is_being_attacked {
        *no_damage_timer = 0.0;
    } else {
        *no_damage_timer += time.delta_secs();
    }

    // Start regen after 3s of not being attacked: 5% max HP per second
    if *no_damage_timer >= 3.0 {
        let regen = health.max * 0.05 * time.delta_secs();
        health.current = (health.current + regen).min(health.max);
    }
}

/// After the hero scene loads, find or create an AnimationPlayer and attach a multi-clip animation graph.
/// The hero GLB has no embedded animations, so Bevy won't auto-create an AnimationPlayer
/// or AnimationTarget components. We manually insert both.
pub fn setup_hero_animations(
    mut commands: Commands,
    heroes: Query<(Entity, &Children), With<HeroNeedsAnimation>>,
    children_q: Query<&Children>,
    anim_players: Query<Entity, With<AnimationPlayer>>,
    names: Query<&Name>,
    transforms: Query<&Transform>,
    asset_server: Res<AssetServer>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
    active_hero: Res<ActiveHeroType>,
) {
    for (hero_entity, children) in &heroes {
        // Walk the scene hierarchy looking for AnimationPlayer or the Armature node
        let mut stack: Vec<Entity> = children.iter().copied().collect();
        let mut found_player = None;
        let mut armature_entity = None;

        while let Some(entity) = stack.pop() {
            if let Ok(name) = names.get(entity) {
                let n = name.as_str();
                if n == "Armature" || n.contains("CharacterArmature") {
                    armature_entity = Some(entity);
                }
            }
            if anim_players.get(entity).is_ok() {
                found_player = Some(entity);
                break;
            }
            if let Ok(grandchildren) = children_q.get(entity) {
                stack.extend(grandchildren.iter());
            }
        }

        // Need either an existing player or the armature node
        let player_entity = if let Some(e) = found_player {
            e
        } else if let Some(e) = armature_entity {
            // Model has no animations → insert AnimationPlayer manually
            commands.entity(e).insert(AnimationPlayer::default());
            e
        } else {
            // Scene not loaded yet
            continue;
        };

        // If we created the AnimationPlayer (no embedded anims), also insert
        // AnimationTarget on every bone in the hierarchy so the animation
        // system knows which entities to drive.
        if found_player.is_none() {
            if let Some(armature) = armature_entity {
                let armature_name = names.get(armature)
                    .map(|n| n.clone())
                    .unwrap_or_else(|_| Name::new("Armature"));
                let root_path = vec![armature_name];

                // Insert AnimationTarget on the armature itself
                commands.entity(armature).insert(AnimationTarget {
                    id: AnimationTargetId::from_names(root_path.iter()),
                    player: armature,
                });

                // Recursively insert on all bone children
                if let Ok(armature_children) = children_q.get(armature) {
                    for &child in armature_children.iter() {
                        insert_anim_targets_recursive(
                            &mut commands, child, armature,
                            &root_path, &children_q, &names,
                        );
                    }
                }
                info!("Inserted AnimationTarget on hero bone hierarchy");
            }
        }

        // Find the mixamorig:Hips entity and capture its bind-pose position
        let mut hips_entity = None;
        let mut hips_bind_pos = Vec3::ZERO;
        if let Some(armature) = armature_entity {
            if let Ok(armature_children) = children_q.get(armature) {
                for &child in armature_children.iter() {
                    if let Ok(name) = names.get(child) {
                        if name.as_str() == "mixamorig:Hips" {
                            hips_entity = Some(child);
                            if let Ok(tf) = transforms.get(child) {
                                hips_bind_pos = tf.translation;
                                info!("Hips bind pose: ({:.2}, {:.2}, {:.2})", hips_bind_pos.x, hips_bind_pos.y, hips_bind_pos.z);
                            }
                            break;
                        }
                    }
                }
            }
        }

        let stats = hero_stats(active_hero.0);

        // Capture bind poses for ALL bones when rotation_only_anims is enabled.
        // This must happen BEFORE any animation plays so we get the original model poses.
        let bone_bind_poses = if stats.rotation_only_anims {
            if let Some(armature) = armature_entity {
                let mut poses = HashMap::new();
                fn capture_bone_poses(
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
                            capture_bone_poses(child, children_q, transforms, out);
                        }
                    }
                }
                capture_bone_poses(armature, &children_q, &transforms, &mut poses);
                info!("Captured bind poses for {} bones (rotation-only mode)", poses.len());
                Some(poses)
            } else {
                None
            }
        } else {
            None
        };

        // Build graph with idle, run, and attack clips
        let idle_clip: Handle<AnimationClip> = asset_server.load(format!("{}#Animation0", stats.idle_anim));
        let run_clip: Handle<AnimationClip> = asset_server.load(format!("{}#Animation0", stats.run_anim));
        let attack_clip: Handle<AnimationClip> = asset_server.load(format!("{}#Animation0", stats.attack_anim));

        // Collect unique clip handles that need rotation-only stripping
        let rotation_only_clips = if stats.rotation_only_anims {
            let mut clips = vec![idle_clip.clone(), run_clip.clone(), attack_clip.clone()];
            clips.dedup();
            clips
        } else {
            vec![]
        };

        // Keep clone of run clip handle so we can strip its root motion curves after loading
        let run_clip_handle = run_clip.clone();

        let mut graph = AnimationGraph::new();
        let idle_node = graph.add_clip(idle_clip, 1.0, graph.root);
        let walk_node = graph.add_clip(run_clip, 1.0, graph.root);
        let attack_node = graph.add_clip(attack_clip, 1.0, graph.root);
        let graph_handle = graphs.add(graph);

        commands.entity(player_entity).insert(AnimationGraphHandle(graph_handle));

        // Compute visual center offset: Hips position in entity-local space
        // (accounts for scene-root rotation and parent scale)
        if stats.model_rotation_x != 0.0 {
            let rx = Quat::from_rotation_x(stats.model_rotation_x);
            let rotated = rx * hips_bind_pos;
            let offset = rotated * stats.model_scale;
            commands.entity(hero_entity).insert(HeroVisualOffset(offset));
            info!("HeroVisualOffset: ({:.3}, {:.3}, {:.3})", offset.x, offset.y, offset.z);
        }

        commands.entity(hero_entity).insert(HeroAnimState {
            idle_node,
            walk_node,
            attack_node,
            current: HeroAnimKind::Idle,
            player_entity,
            attack_triggered: false,
            hips_entity,
            hips_bind_pos,
            run_clip_handle,
            run_clip_stripped: false,
            bone_bind_poses,
            skip_hips_reset: stats.skip_root_motion_cancel,
            rotation_only_clips,
            rotation_only_stripped: false,
            armature_entity: armature_entity.or(Some(player_entity)),
            // Rotation is now applied on the scene child at load time,
            // not on the armature in PostUpdate (which conflicts with look_at).
            armature_rotation_fix: None,
        });
        commands.entity(hero_entity).remove::<HeroNeedsAnimation>();

        info!("Hero {:?} anim graph: idle={}, attack={}, run={}, skip_hips={}, rot_only={}, armature={:?}",
              hero_entity, stats.idle_anim, stats.attack_anim, stats.run_anim,
              stats.skip_root_motion_cancel, stats.rotation_only_anims, armature_entity);

    }
}

/// Recursively insert AnimationTarget on bone entities so the animation system can drive them.
fn insert_anim_targets_recursive(
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

    commands.entity(entity).insert(AnimationTarget {
        id: AnimationTargetId::from_names(path.iter()),
        player: player_entity,
    });

    if let Ok(children) = children_q.get(entity) {
        for &child in children.iter() {
            insert_anim_targets_recursive(commands, child, player_entity, &path, children_q, names);
        }
    }
}

/// Start idle playback on newly set up heroes.
pub fn play_hero_animations(
    heroes: Query<&HeroAnimState, Added<HeroAnimState>>,
    mut players: Query<&mut AnimationPlayer>,
) {
    for anim_state in &heroes {
        if let Ok(mut player) = players.get_mut(anim_state.player_entity) {
            player.play(anim_state.idle_node).repeat();
        }
    }
}

/// Switch hero animation based on current behavior and face the nearest enemy when attacking.
/// Rules:
/// - Attack animation plays fully through; only a player move command can interrupt it.
/// - Between auto-attacks, hero holds the last attack frame while enemies are in range.
/// - Walk (run) loops while moving; Idle loops otherwise.
pub fn update_hero_animations(
    mut heroes: Query<
        (&mut HeroAnimState, &HeroMoveTarget, &HeroAttackRange, &mut Transform),
        (With<Hero>, Without<HeroRespawnTimer>),
    >,
    enemies: Query<&Transform, (With<Enemy>, Without<Hero>)>,
    mut players: Query<&mut AnimationPlayer>,
) {
    for (mut anim_state, move_target, range, mut hero_tf) in &mut heroes {
        // Find nearest enemy in range (for facing)
        let mut nearest_enemy: Option<(Vec3, f32)> = None;
        for enemy_tf in &enemies {
            let dist = hero_tf.translation.distance(enemy_tf.translation);
            if dist <= range.0 {
                if nearest_enemy.is_none() || dist < nearest_enemy.unwrap().1 {
                    nearest_enemy = Some((enemy_tf.translation, dist));
                }
            }
        }

        // --- If attack is playing and not finished, only interrupt for move command ---
        if anim_state.current == HeroAnimKind::Attack {
            let attack_finished = players.get(anim_state.player_entity)
                .map(|p| p.all_finished())
                .unwrap_or(true);

            if !attack_finished {
                if move_target.0.is_some() {
                    // Player move command interrupts attack
                    anim_state.attack_triggered = false;
                    anim_state.current = HeroAnimKind::Walk;
                    if let Ok(mut player) = players.get_mut(anim_state.player_entity) {
                        player.stop_all();
                        player.play(anim_state.walk_node).repeat();
                    }
                } else {
                    // Let attack play out, keep facing enemy
                    face_enemy(&mut hero_tf, nearest_enemy);
                }
                continue;
            }
            // Attack finished — fall through to process pending trigger
        }

        // --- Attack trigger: restart attack clip from beginning ---
        if anim_state.attack_triggered {
            anim_state.attack_triggered = false;
            if let Ok(mut player) = players.get_mut(anim_state.player_entity) {
                player.stop_all();
                player.play(anim_state.attack_node);
            }
            anim_state.current = HeroAnimKind::Attack;
            face_enemy(&mut hero_tf, nearest_enemy);
            continue;
        }

        // --- Determine desired state ---
        let desired = if move_target.0.is_some() {
            HeroAnimKind::Walk
        } else if nearest_enemy.is_some() {
            // Enemies in range: hold attack pose (last frame), wait for next attack_triggered
            HeroAnimKind::Attack
        } else {
            HeroAnimKind::Idle
        };

        // Face enemy when not walking
        if desired != HeroAnimKind::Walk {
            face_enemy(&mut hero_tf, nearest_enemy);
        }

        // Only change animation on actual state transition
        // (Attack → Attack is a no-op; the clip holds its last frame until attack_triggered)
        if desired != anim_state.current {
            anim_state.current = desired;
            if let Ok(mut player) = players.get_mut(anim_state.player_entity) {
                player.stop_all();
                match desired {
                    HeroAnimKind::Idle => { player.play(anim_state.idle_node).repeat(); }
                    HeroAnimKind::Walk => { player.play(anim_state.walk_node).repeat(); }
                    HeroAnimKind::Attack => { /* Hold last frame; attack_triggered restarts */ }
                };
            }
        }
    }
}

fn face_enemy(hero_tf: &mut Transform, nearest_enemy: Option<(Vec3, f32)>) {
    if let Some((enemy_pos, _)) = nearest_enemy {
        let look = Vec3::new(enemy_pos.x, hero_tf.translation.y, enemy_pos.z);
        hero_tf.look_at(look, Vec3::Y);
        // Idle/attack clips still have Hips rotation that flips the model;
        // compensate so the hero faces the enemy.
        hero_tf.rotate_y(std::f32::consts::PI);
    }
}

/// Strips translation and scale curves from ALL animation clips for rotation-only heroes.
/// GLTF exports curves per bone in order: [translation, rotation, scale].
/// We keep only the rotation curve (index 1), removing translation and scale.
/// This is the Bevy equivalent of Three.js stripping .position/.scale tracks.
pub fn strip_hero_rotation_only_clips(
    mut heroes: Query<&mut HeroAnimState, With<Hero>>,
    mut clips: ResMut<Assets<AnimationClip>>,
) {
    for mut anim_state in &mut heroes {
        if anim_state.rotation_only_stripped || anim_state.rotation_only_clips.is_empty() {
            continue;
        }

        // Check if all clips are loaded
        let all_loaded = anim_state.rotation_only_clips.iter().all(|h| clips.get(h).is_some());
        if !all_loaded {
            continue;
        }

        let handles: Vec<_> = anim_state.rotation_only_clips.clone();
        for handle in &handles {
            if let Some(clip) = clips.get_mut(handle) {
                let mut stripped_count = 0;
                for (_target_id, curves) in clip.curves_mut().iter_mut() {
                    if curves.len() == 3 {
                        // GLTF order: [translation(0), rotation(1), scale(2)]
                        // Keep only rotation
                        let rotation = VariableCurve(curves[1].0.clone_value());
                        curves.clear();
                        curves.push(rotation);
                        stripped_count += 1;
                    } else if curves.len() == 2 {
                        // Likely [translation, rotation] — keep index 1
                        let rotation = VariableCurve(curves[1].0.clone_value());
                        curves.clear();
                        curves.push(rotation);
                        stripped_count += 1;
                    }
                    // len == 1: probably just rotation or morph weights, leave as-is
                }
                info!("Stripped translation/scale from {} bone targets in clip", stripped_count);
            }
        }
        anim_state.rotation_only_stripped = true;
    }
}

/// Strips root-motion curves from the run animation clip after it loads.
/// Only targets the run clip (which has forward-drift baked into Hips translation).
/// Idle and attack clips are left untouched so their full animation plays.
pub fn strip_hero_root_motion_clips(
    mut heroes: Query<&mut HeroAnimState, With<Hero>>,
    mut clips: ResMut<Assets<AnimationClip>>,
) {
    let hips_id = AnimationTargetId::from_names(
        [Name::new("Armature"), Name::new("mixamorig:Hips")].iter(),
    );

    for mut anim_state in &mut heroes {
        if anim_state.run_clip_stripped {
            continue;
        }
        // Skip stripping for heroes whose animations have compatible Hips values
        if anim_state.skip_hips_reset {
            anim_state.run_clip_stripped = true;
            continue;
        }

        let handle = anim_state.run_clip_handle.clone();
        if let Some(clip) = clips.get_mut(&handle) {
            if clip.curves_mut().remove(&hips_id).is_some() {
                info!("Stripped Hips root motion curves from run clip");
            }
            anim_state.run_clip_stripped = true;
        }
    }
}

/// Resets bone transforms after animation applies.
/// - For rotation-only heroes: resets ALL bone translations and scales to bind pose,
///   keeping only rotation from the animation (fixes bone-scale mismatch).
/// - For other heroes: resets only Hips X/Z to bind pose (cancels root motion drift).
pub fn cancel_hero_root_motion(
    heroes: Query<&HeroAnimState, With<Hero>>,
    mut transforms: Query<&mut Transform, Without<Hero>>,
) {
    for anim in &heroes {
        // Full bind-pose reset for rotation-only heroes (e.g. Northern Outsider)
        if let Some(ref poses) = anim.bone_bind_poses {
            for (&entity, &(translation, scale)) in poses {
                if let Ok(mut tf) = transforms.get_mut(entity) {
                    tf.translation = translation;
                    tf.scale = scale;
                }
            }
            continue;
        }

        // Standard root motion cancellation: reset Hips X/Z only
        // Skip for heroes whose animations have compatible Hips values
        if anim.skip_hips_reset {
            continue;
        }
        if let Some(hips) = anim.hips_entity {
            if let Ok(mut tf) = transforms.get_mut(hips) {
                tf.translation = anim.hips_bind_pos;
            }
        }

        // Armature rotation correction — applied AFTER animation writes bone rotations.
        // Fixes models exported with wrong up-axis (e.g. Pharaoh is Z-up instead of Y-up).
        // SET rather than multiply, since animations don't target the Armature node itself.
        // Also zero the armature translation so the rotation doesn't displace the model.
        if let (Some(armature), Some(fix_rot)) = (anim.armature_entity, anim.armature_rotation_fix) {
            if let Ok(mut tf) = transforms.get_mut(armature) {
                tf.rotation = fix_rot;
                tf.translation = Vec3::ZERO;
            }
        }

    }
}
