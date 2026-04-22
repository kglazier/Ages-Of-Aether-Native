use bevy::prelude::*;
use crate::components::*;
use crate::data::*;
use crate::resources::*;

/// Each tower finds the nearest enemy in range and fires a projectile.
pub fn tower_targeting(
    mut commands: Commands,
    mut towers: Query<
        (&Transform, &mut AttackTimer, &AttackRange, &AttackDamage, &Element, Option<&TowerSpec>),
        With<Tower>,
    >,
    enemies: Query<(Entity, &Transform, &PathFollower), With<Enemy>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    time: Res<Time>,
) {
    use crate::data::TowerSpecialization;

    for (tower_pos, mut timer, range, damage, element, spec) in &mut towers {
        // Bramble Grove doesn't fire projectiles (uses aura only)
        if let Some(s) = spec {
            if matches!(s.0, TowerSpecialization::BrambleGrove) {
                continue;
            }
        }

        timer.elapsed += time.delta_secs();
        if timer.elapsed < timer.cooldown {
            continue;
        }

        // Target "first" enemy — furthest along the path within range
        let mut best: Option<(Entity, f32)> = None; // (entity, path_progress)
        for (enemy_entity, enemy_pos, follower) in &enemies {
            let dist = tower_pos.translation.distance(enemy_pos.translation);
            if dist <= range.0 {
                let progress = follower.segment as f32 + follower.progress;
                if best.is_none() || progress > best.unwrap().1 {
                    best = Some((enemy_entity, progress));
                }
            }
        }

        if let Some((target, _)) = best {
            timer.elapsed = 0.0;

            let color = element_color(*element);
            let emissive = element_emissive(*element);
            let proj_pos = tower_pos.translation + Vec3::Y * 1.5;

            // Scale projectile size for heavy hitters
            let proj_radius = if spec.is_some_and(|s| matches!(s.0, TowerSpecialization::Railgun | TowerSpecialization::MeteorTower)) {
                0.3
            } else {
                0.15
            };

            let mut proj = commands.spawn((
                Mesh3d(meshes.add(Sphere::new(proj_radius))),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: color,
                    emissive,
                    ..default()
                })),
                Transform::from_translation(proj_pos),
                Projectile {
                    damage: damage.0,
                    speed: 15.0,
                    target,
                    element: *element,
                },
            ));

            // Fire projectiles have AoE splash
            if *element == Element::Fire {
                proj.insert(AoeSplash(2.5));
            }

            // Propagate specialization to projectile
            if let Some(s) = spec {
                proj.insert(ProjectileSpec(s.0));
            }

            // Muzzle flash at the tower's firing point
            commands.spawn((
                Mesh3d(meshes.add(Sphere::new(0.25))),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: Color::srgba(1.0, 0.9, 0.7, 0.9),
                    emissive: emissive * 2.0,
                    alpha_mode: AlphaMode::Blend,
                    unlit: true,
                    ..default()
                })),
                Transform::from_translation(proj_pos),
                MuzzleFlash {
                    lifetime: 0.15,
                    elapsed: 0.0,
                    element: *element,
                },
            ));
        }
    }
}

/// Moves projectiles toward their target. On hit, applies damage + effects.
/// Elemental synergies:
///   Ice+Lightning — slowed enemies take +50% bonus true damage from lightning
///   Ice+Fire     — slowed enemies take 40% of fire damage as bonus true damage
pub fn move_projectiles(
    mut commands: Commands,
    mut projectiles: Query<(Entity, &mut Transform, &Projectile, Option<&AoeSplash>, Option<&ProjectileSpec>)>,
    mut enemies: Query<
        (Entity, &Transform, &mut Health, &Armor, Option<&SlowDebuff>),
        (With<Enemy>, Without<Projectile>),
    >,
    time: Res<Time>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (proj_entity, mut proj_transform, projectile, aoe, proj_spec) in &mut projectiles {
        // If target was already despawned, remove projectile
        let target_pos = {
            let Ok((_, enemy_transform, _, _, _)) = enemies.get(projectile.target) else {
                commands.entity(proj_entity).despawn();
                continue;
            };
            enemy_transform.translation
        };

        let direction = target_pos - proj_transform.translation;
        let distance = direction.length();

        if distance < 0.5 {
            // --- HIT ---
            let damage = projectile.damage;
            let element = projectile.element;

            // Apply damage to primary target (with synergy bonus)
            if let Ok((_, _, mut health, armor, slow)) = enemies.get_mut(projectile.target) {
                let reduced = apply_damage_reduction(damage, element, &armor);
                health.current -= reduced;

                // Elemental synergy: bonus true damage on slowed enemies
                if slow.is_some() {
                    match element {
                        Element::Lightning => {
                            // Ice+Lightning: +50% bonus true damage (ignores armor)
                            health.current -= damage * 0.5;
                        }
                        Element::Fire => {
                            // Ice+Fire: 40% of fire damage as bonus true damage
                            health.current -= damage * 0.4;
                        }
                        _ => {}
                    }
                }
            }

            // AoE splash (fire): damage nearby enemies too
            if let Some(splash) = aoe {
                for (entity, etransform, mut health, armor, slow) in &mut enemies {
                    if entity == projectile.target {
                        continue;
                    }
                    let dist = target_pos.distance(etransform.translation);
                    if dist <= splash.0 {
                        let reduced = apply_damage_reduction(damage * 0.35, element, &armor);
                        health.current -= reduced;

                        // Synergy on splash targets too
                        if slow.is_some() && element == Element::Fire {
                            health.current -= damage * 0.35 * 0.4;
                        }
                    }
                }
            }

            // Specialization effects on hit
            if let Some(proj_spec) = proj_spec {
                use crate::data::TowerSpecialization;
                match proj_spec.0 {
                    TowerSpecialization::StormSpire => {
                        // Chain lightning: jump to 2 nearby enemies for 40% damage
                        let mut chains = 0;
                        for (entity, etransform, mut health, armor, _) in &mut enemies {
                            if entity == projectile.target || chains >= 2 { continue; }
                            let dist = target_pos.distance(etransform.translation);
                            if dist <= 4.0 {
                                let chain_dmg = apply_damage_reduction(damage * 0.4, element, &armor);
                                health.current -= chain_dmg;
                                chains += 1;
                            }
                        }
                    }
                    TowerSpecialization::ShatterMage => {
                        // Already slowed enemies take 3x total damage (2x bonus since base already applied)
                        if let Ok((_, _, mut health, armor, slow)) = enemies.get_mut(projectile.target) {
                            if slow.is_some() {
                                let bonus = apply_damage_reduction(damage * 2.0, element, &armor);
                                health.current -= bonus;
                            }
                        }
                    }
                    TowerSpecialization::InfernoCannon => {
                        // Spawn a burning ground zone at impact
                        commands.spawn((
                            Mesh3d(meshes.add(Cylinder::new(3.0, 0.02))),
                            MeshMaterial3d(materials.add(StandardMaterial {
                                base_color: Color::srgba(0.55, 0.13, 0.0, 0.12),
                                emissive: LinearRgba::new(0.12, 0.03, 0.0, 1.0),
                                alpha_mode: AlphaMode::Blend,
                                unlit: true,
                                ..default()
                            })),
                            Transform::from_translation(Vec3::new(target_pos.x, 0.05, target_pos.z)),
                            BurnZone { radius: 3.0, dps: 5.0, remaining: 4.0 },
                            GameWorldEntity,
                        ));
                    }
                    _ => {}
                }
            }

            // Apply debuffs based on element
            match element {
                Element::Ice => {
                    // Apply slow: 50% speed reduction for 2 seconds
                    commands.entity(projectile.target).insert(SlowDebuff {
                        factor: 0.5,
                        remaining: 2.0,
                    });
                }
                Element::Fire => {
                    // Apply burn: 3 DPS for 3 seconds
                    commands.entity(projectile.target).insert(BurnDebuff {
                        dps: 3.0,
                        remaining: 3.0,
                    });
                }
                _ => {}
            }

            commands.entity(proj_entity).despawn();
        } else {
            let dir_norm = direction.normalize();
            proj_transform.translation += dir_norm * projectile.speed * time.delta_secs();

            // Stretch projectile along movement direction for trail effect
            let forward = dir_norm;
            let up = Vec3::Y;
            let right = forward.cross(up).normalize_or_zero();
            let actual_up = right.cross(forward).normalize_or_zero();
            if right.length_squared() > 0.001 {
                proj_transform.rotation = Quat::from_mat3(&Mat3::from_cols(right, actual_up, -forward));
            }
            // Stretch 2.5x along Z (forward), keep X/Y normal
            proj_transform.scale = Vec3::new(1.0, 1.0, 2.5);
        }
    }
}

/// Calculates damage after armor/magic resist reduction.
fn apply_damage_reduction(damage: f32, element: Element, armor: &Armor) -> f32 {
    match element {
        // Lightning and Ice deal magic damage → reduced by magic_resist
        Element::Lightning | Element::Ice => damage * (1.0 - armor.magic_resist),
        // Earth and Fire deal physical damage → reduced by armor
        Element::Earth | Element::Fire => {
            let reduction = armor.physical / (armor.physical + 100.0);
            damage * (1.0 - reduction)
        }
    }
}

/// Tick burn debuff: apply DPS damage and decrement timer.
pub fn tick_debuffs(
    mut commands: Commands,
    mut burn_q: Query<(Entity, &mut Health, &mut BurnDebuff)>,
    mut slow_q: Query<(Entity, &mut SlowDebuff)>,
    time: Res<Time>,
) {
    let dt = time.delta_secs();

    for (entity, mut health, mut burn) in &mut burn_q {
        health.current -= burn.dps * dt;
        burn.remaining -= dt;
        if burn.remaining <= 0.0 {
            commands.entity(entity).remove::<BurnDebuff>();
        }
    }

    for (entity, mut slow) in &mut slow_q {
        slow.remaining -= dt;
        if slow.remaining <= 0.0 {
            commands.entity(entity).remove::<SlowDebuff>();
        }
    }
}

/// Marks dead enemies as dying (plays death animation) and awards gold + effects.
/// Enemies with EnemyDying are no longer Enemy — they don't block, move, or get targeted.
pub fn check_enemy_death(
    mut commands: Commands,
    query: Query<(Entity, &Health, &GoldReward, &Transform), (With<Enemy>, Without<EnemyDying>)>,
    mut game: ResMut<GameData>,
    mut wave: ResMut<WaveState>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (entity, health, gold, transform) in &query {
        if health.current <= 0.0 {
            game.gold += gold.0;
            wave.active_enemies = wave.active_enemies.saturating_sub(1);
            let pos = transform.translation;

            // Death burst — expanding sphere
            commands.spawn((
                Mesh3d(meshes.add(Sphere::new(0.3))),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: Color::srgba(1.0, 0.3, 0.1, 0.8),
                    emissive: LinearRgba::new(2.0, 0.6, 0.2, 1.0),
                    alpha_mode: AlphaMode::Blend,
                    unlit: true,
                    ..default()
                })),
                Transform::from_translation(pos),
                DeathEffect {
                    lifetime: 0.4,
                    elapsed: 0.0,
                },
            ));

            // Floating gold indicator
            commands.spawn((
                Mesh3d(meshes.add(Sphere::new(0.15))),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: Color::srgba(1.0, 0.85, 0.0, 0.9),
                    emissive: LinearRgba::new(2.0, 1.7, 0.0, 1.0),
                    alpha_mode: AlphaMode::Blend,
                    unlit: true,
                    ..default()
                })),
                Transform::from_translation(pos + Vec3::Y * 0.5),
                GoldPopup {
                    lifetime: 0.8,
                    elapsed: 0.0,
                    start_y: pos.y + 0.5,
                },
            ));

            // Remove Enemy so it's no longer targeted/blocked/moved.
            // Insert EnemyDying to play death animation then despawn.
            commands.entity(entity).remove::<Enemy>();
            commands.entity(entity).remove::<GolemBlocked>();
            commands.entity(entity).remove::<BlockOffset>();
            commands.entity(entity).insert(EnemyDying { timer: 1.2 });
        }
    }
}

/// Animates death burst effects — scale up and fade out.
pub fn animate_death_effects(
    mut commands: Commands,
    mut effects: Query<(Entity, &mut DeathEffect, &mut Transform, &MeshMaterial3d<StandardMaterial>)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    time: Res<Time>,
) {
    for (entity, mut effect, mut transform, mat_handle) in &mut effects {
        effect.elapsed += time.delta_secs();
        let t = (effect.elapsed / effect.lifetime).min(1.0);

        // Scale up from 1x to 3x
        let scale = 1.0 + t * 2.0;
        transform.scale = Vec3::splat(scale);

        // Fade out
        if let Some(material) = materials.get_mut(&mat_handle.0) {
            material.base_color = Color::srgba(1.0, 0.3, 0.1, 0.8 * (1.0 - t));
        }

        if effect.elapsed >= effect.lifetime {
            commands.entity(entity).despawn();
        }
    }
}

/// Animates floating gold popups — rise and fade out.
pub fn animate_gold_popups(
    mut commands: Commands,
    mut popups: Query<(Entity, &mut GoldPopup, &mut Transform, &MeshMaterial3d<StandardMaterial>)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    time: Res<Time>,
) {
    for (entity, mut popup, mut transform, mat_handle) in &mut popups {
        popup.elapsed += time.delta_secs();
        let t = (popup.elapsed / popup.lifetime).min(1.0);

        // Float upward
        transform.translation.y = popup.start_y + t * 2.5;

        // Fade out in second half
        if let Some(material) = materials.get_mut(&mat_handle.0) {
            let alpha = if t < 0.5 { 0.9 } else { 0.9 * (1.0 - (t - 0.5) * 2.0) };
            material.base_color = Color::srgba(1.0, 0.85, 0.0, alpha.max(0.0));
        }

        if popup.elapsed >= popup.lifetime {
            commands.entity(entity).despawn();
        }
    }
}

/// Animates muzzle flash — quick scale up and fade.
pub fn animate_muzzle_flashes(
    mut commands: Commands,
    mut flashes: Query<(Entity, &mut MuzzleFlash, &mut Transform, &MeshMaterial3d<StandardMaterial>)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    time: Res<Time>,
) {
    for (entity, mut flash, mut transform, mat_handle) in &mut flashes {
        flash.elapsed += time.delta_secs();
        let t = (flash.elapsed / flash.lifetime).min(1.0);

        // Scale up quickly then shrink
        let scale = if t < 0.3 { 1.0 + t * 3.0 } else { (1.0 - t) * 2.0 };
        transform.scale = Vec3::splat(scale.max(0.01));

        if let Some(material) = materials.get_mut(&mat_handle.0) {
            material.base_color = Color::srgba(1.0, 0.9, 0.7, 0.9 * (1.0 - t));
        }

        if flash.elapsed >= flash.lifetime {
            commands.entity(entity).despawn();
        }
    }
}

/// Healer enemies (Sporebloom) heal nearby allies.
pub fn healer_aura_tick(
    healers: Query<(&Transform, &HealerAura), With<Enemy>>,
    mut enemies: Query<(&Transform, &mut Health), With<Enemy>>,
    time: Res<Time>,
) {
    let dt = time.delta_secs();

    let healer_info: Vec<(Vec3, f32, f32)> = healers
        .iter()
        .map(|(t, aura)| (t.translation, aura.radius, aura.heal_per_second))
        .collect();

    for (enemy_transform, mut health) in &mut enemies {
        if health.current <= 0.0 || health.current >= health.max {
            continue;
        }
        for (healer_pos, radius, hps) in &healer_info {
            let dist = enemy_transform.translation.distance(*healer_pos);
            if dist <= *radius {
                health.current = (health.current + hps * dt).min(health.max);
                break; // Only heal from one healer at a time
            }
        }
    }
}

/// Elemental synergies: golems near allied elemental towers apply debuffs to blocked enemies.
///   Earth+Ice  — golems within 8 units of an ice tower apply slow to blocked enemies
///   Earth+Fire — golems within 8 units of a fire tower apply burn to blocked enemies
const SYNERGY_RADIUS: f32 = 8.0;

pub fn golem_elemental_synergy(
    mut commands: Commands,
    golems: Query<&Transform, With<Golem>>,
    towers: Query<(&Transform, &Element), With<Tower>>,
    blocked_enemies: Query<(Entity, &Transform, Option<&SlowDebuff>, Option<&BurnDebuff>), (With<Enemy>, With<GolemBlocked>)>,
    time: Res<Time>,
) {
    let dt = time.delta_secs();
    if dt == 0.0 { return; }

    // Collect tower positions by element (only ice and fire)
    let ice_positions: Vec<Vec3> = towers.iter()
        .filter(|(_, e)| **e == Element::Ice)
        .map(|(t, _)| t.translation)
        .collect();
    let fire_positions: Vec<Vec3> = towers.iter()
        .filter(|(_, e)| **e == Element::Fire)
        .map(|(t, _)| t.translation)
        .collect();

    if ice_positions.is_empty() && fire_positions.is_empty() {
        return;
    }

    // For each golem, check if it's near an ice or fire tower
    let golem_positions: Vec<Vec3> = golems.iter().map(|t| t.translation).collect();

    for (enemy_entity, enemy_tf, has_slow, has_burn) in &blocked_enemies {
        // Check if this blocked enemy is near a golem that is near an elemental tower
        for golem_pos in &golem_positions {
            let enemy_near_golem = golem_pos.distance(enemy_tf.translation) < 3.0;
            if !enemy_near_golem { continue; }

            // Earth+Ice synergy: golem near ice tower applies slow
            if has_slow.is_none() {
                let golem_near_ice = ice_positions.iter()
                    .any(|tp| golem_pos.distance(*tp) <= SYNERGY_RADIUS);
                if golem_near_ice {
                    commands.entity(enemy_entity).insert(SlowDebuff {
                        factor: 0.5,
                        remaining: 1.5,
                    });
                }
            }

            // Earth+Fire synergy: golem near fire tower applies burn
            if has_burn.is_none() {
                let golem_near_fire = fire_positions.iter()
                    .any(|tp| golem_pos.distance(*tp) <= SYNERGY_RADIUS);
                if golem_near_fire {
                    commands.entity(enemy_entity).insert(BurnDebuff {
                        dps: 1.0,
                        remaining: 1.0,
                    });
                }
            }

            break; // Only need one golem proximity check
        }
    }
}

/// Manages the 3D range indicator circle when a tower is selected.
pub fn update_range_indicator(
    mut commands: Commands,
    selection: Res<Selection>,
    towers: Query<(&Transform, &AttackRange), With<Tower>>,
    indicators: Query<Entity, With<RangeIndicator>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    current_level: Res<crate::resources::CurrentLevel>,
) {
    // Always despawn old indicators
    for entity in &indicators {
        commands.entity(entity).despawn();
    }

    // Show if a tower is selected or we're setting rally point
    let tower_entity = match *selection {
        Selection::Tower(e) | Selection::SettingRallyPoint(e) => e,
        _ => return,
    };

    let Ok((tower_transform, range)) = towers.get(tower_entity) else {
        return;
    };

    // Frozen tundra (levels 5/6) has a near-white background — invert the ring
    // so it stays visible. Other themes use the default white ring.
    let ring_color = match current_level.0 {
        5 | 6 => Color::srgba(0.05, 0.1, 0.25, 0.55),
        _ => Color::srgba(1.0, 1.0, 1.0, 0.25),
    };

    // Spawn a thin ring using an annulus (flat ring shape)
    let pos = tower_transform.translation;
    let inner = range.0 - 0.05;
    let outer = range.0 + 0.05;
    commands.spawn((
        Mesh3d(meshes.add(Annulus::new(inner, outer))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: ring_color,
            alpha_mode: AlphaMode::Blend,
            unlit: true,
            double_sided: true,
            ..default()
        })),
        Transform::from_translation(Vec3::new(pos.x, 0.15, pos.z))
            .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
        RangeIndicator,
        GameWorldEntity,
    ));
}

/// After an enemy's scene loads, recolor its materials by cloning each material,
/// removing the texture, and setting a tinted solid color. Each mesh part gets
/// the tint blended with its original base color so body sections stay distinct.
/// Applies species-colored tinting using the "keepColors + lightness spread" algorithm.
/// 1. Collects per-mesh-part lightness from the original material
/// 2. Remaps lightness to [0.32, 0.76] range for visual contrast
/// 3. Applies species hue with per-part hue variation
pub fn apply_enemy_tints(
    mut commands: Commands,
    tinted: Query<(Entity, &Children, &EnemyNeedsTint)>,
    children_q: Query<&Children>,
    mesh_q: Query<(Entity, &MeshMaterial3d<StandardMaterial>)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (entity, children, tint) in &tinted {
        let tint_srgba = tint.0.to_srgba();

        // Convert species color to HSL for hue/saturation
        let species_hue = rgb_to_hsl(tint_srgba.red, tint_srgba.green, tint_srgba.blue);

        // Pass 1: collect mesh entities and their original lightness values
        struct MeshPart {
            entity: Entity,
            handle: Handle<StandardMaterial>,
            lightness: f32,
        }
        let mut parts: Vec<MeshPart> = Vec::new();
        let mut stack: Vec<Entity> = children.iter().copied().collect();
        while let Some(child) = stack.pop() {
            if let Ok((mesh_entity, mat_handle)) = mesh_q.get(child) {
                if let Some(original) = materials.get(&mat_handle.0) {
                    let orig = original.base_color.to_srgba();
                    let (_, _, l) = rgb_to_hsl(orig.red, orig.green, orig.blue);
                    parts.push(MeshPart {
                        entity: mesh_entity,
                        handle: mat_handle.0.clone(),
                        lightness: l,
                    });
                }
            }
            if let Ok(grandchildren) = children_q.get(child) {
                stack.extend(grandchildren.iter());
            }
        }

        if parts.is_empty() {
            continue; // Scene meshes not loaded yet
        }

        // Pass 2: normalize lightness and apply species hue
        let min_l = parts.iter().map(|p| p.lightness).fold(f32::INFINITY, f32::min);
        let max_l = parts.iter().map(|p| p.lightness).fold(f32::NEG_INFINITY, f32::max);
        let range = (max_l - min_l).max(0.001);

        for (i, part) in parts.iter().enumerate() {
            if let Some(original) = materials.get(&part.handle) {
                let mut new_mat = original.clone();
                // Normalize lightness to [0, 1]
                let t = if parts.len() <= 1 {
                    0.5
                } else if range < 0.01 {
                    // All parts same lightness — fallback to index-based spread
                    i as f32 / (parts.len() as f32 - 1.0).max(1.0)
                } else {
                    ((part.lightness - min_l) / range).clamp(0.0, 1.0)
                };

                // Remap to target lightness range [0.32, 0.76]
                let target_l = 0.32 + t * 0.44;
                // Per-part hue variation
                let hue_shift = (t - 0.5) * 0.06;
                let h = species_hue.0 + hue_shift;
                let s = species_hue.1 * 0.85;

                let (r, g, b) = hsl_to_rgb(h, s, target_l);
                new_mat.base_color = Color::srgb(r, g, b);
                new_mat.base_color_texture = None;
                new_mat.perceptual_roughness = 1.0;
                new_mat.metallic = 0.0;
                new_mat.alpha_mode = AlphaMode::Opaque;
                // Subtle emissive so color reads in shadow
                new_mat.emissive = LinearRgba::new(r * 0.15, g * 0.15, b * 0.15, 1.0);

                let new_handle = materials.add(new_mat);
                commands.entity(part.entity).insert(MeshMaterial3d(new_handle));
            }
        }

        commands.entity(entity).remove::<EnemyNeedsTint>();
    }
}

/// Convert RGB [0,1] to HSL [0,1].
fn rgb_to_hsl(r: f32, g: f32, b: f32) -> (f32, f32, f32) {
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let l = (max + min) / 2.0;

    if (max - min).abs() < 0.001 {
        return (0.0, 0.0, l);
    }

    let d = max - min;
    let s = if l > 0.5 { d / (2.0 - max - min) } else { d / (max + min) };
    let h = if (max - r).abs() < 0.001 {
        let mut h = (g - b) / d;
        if g < b { h += 6.0; }
        h / 6.0
    } else if (max - g).abs() < 0.001 {
        ((b - r) / d + 2.0) / 6.0
    } else {
        ((r - g) / d + 4.0) / 6.0
    };

    (h, s, l)
}

/// Convert HSL [0,1] to RGB [0,1].
fn hsl_to_rgb(h: f32, s: f32, l: f32) -> (f32, f32, f32) {
    if s.abs() < 0.001 {
        return (l, l, l);
    }

    let q = if l < 0.5 { l * (1.0 + s) } else { l + s - l * s };
    let p = 2.0 * l - q;
    let r = hue_to_rgb(p, q, h + 1.0 / 3.0);
    let g = hue_to_rgb(p, q, h);
    let b = hue_to_rgb(p, q, h - 1.0 / 3.0);
    (r, g, b)
}

fn hue_to_rgb(p: f32, q: f32, mut t: f32) -> f32 {
    if t < 0.0 { t += 1.0; }
    if t > 1.0 { t -= 1.0; }
    if t < 1.0 / 6.0 { return p + (q - p) * 6.0 * t; }
    if t < 0.5 { return q; }
    if t < 2.0 / 3.0 { return p + (q - p) * (2.0 / 3.0 - t) * 6.0; }
    p
}

/// Forces BLEND materials on enemy scene children to OPAQUE.
/// Retries each frame until enough mesh children are found (scene fully loaded).
pub fn fix_blend_enemy_materials(
    mut commands: Commands,
    enemies: Query<(Entity, &Children), With<NeedsBlendFix>>,
    children_q: Query<&Children>,
    mesh_q: Query<&MeshMaterial3d<StandardMaterial>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (entity, children) in &enemies {
        let mut mesh_count = 0;
        let mut stack: Vec<Entity> = children.iter().copied().collect();
        while let Some(child) = stack.pop() {
            if let Ok(mat_handle) = mesh_q.get(child) {
                mesh_count += 1;
                if let Some(mat) = materials.get(&mat_handle.0) {
                    if mat.alpha_mode != AlphaMode::Opaque {
                        // Only call get_mut when actually needed
                        if let Some(mat_mut) = materials.get_mut(&mat_handle.0) {
                            mat_mut.alpha_mode = AlphaMode::Opaque;
                        }
                    }
                }
            }
            if let Ok(grandchildren) = children_q.get(child) {
                stack.extend(grandchildren.iter());
            }
        }
        if mesh_count >= 1 {
            commands.entity(entity).remove::<NeedsBlendFix>();
        }
    }
}

/// Hides ground-plane / stand meshes that some Sketchfab models include.
/// Matches materials with "ground" in their name (case-insensitive).
pub fn hide_ground_meshes(
    mut commands: Commands,
    new_enemies: Query<(Entity, &Children), Added<Enemy>>,
    children_q: Query<&Children>,
    mesh_q: Query<(Entity, &MeshMaterial3d<StandardMaterial>)>,
    materials: Res<Assets<StandardMaterial>>,
    names: Query<&Name>,
) {
    for (_enemy, children) in &new_enemies {
        let mut stack: Vec<Entity> = children.iter().copied().collect();
        while let Some(child) = stack.pop() {
            // Check by material name
            if let Ok((mesh_entity, mat_handle)) = mesh_q.get(child) {
                if let Some(mat) = materials.get(&mat_handle.0) {
                    let mat_name = format!("{:?}", mat.base_color); // fallback
                    // Check Name component on mesh entity for "ground" or "circle"
                    let should_hide = if let Ok(name) = names.get(mesh_entity) {
                        let n = name.as_str().to_lowercase();
                        n.contains("ground") || n.contains("circle") || n.contains("stand")
                    } else {
                        false
                    };
                    let _ = mat_name;
                    if should_hide {
                        commands.entity(mesh_entity).insert(Visibility::Hidden);
                    }
                }
            }
            // Also check by entity name without requiring a mesh
            if let Ok(name) = names.get(child) {
                let n = name.as_str().to_lowercase();
                if n.contains("ground") || n.contains("circle") || n.contains("stand") {
                    commands.entity(child).insert(Visibility::Hidden);
                }
            }
            if let Ok(grandchildren) = children_q.get(child) {
                stack.extend(grandchildren.iter());
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Health bars
// ---------------------------------------------------------------------------

const HP_BAR_WIDTH: f32 = 1.0;
const HP_BAR_HEIGHT: f32 = 0.1;
const HP_BAR_Y_OFFSET: f32 = 1.8;

/// Spawns HP bar meshes (background + fill) for newly added enemies.
pub fn spawn_health_bars(
    mut commands: Commands,
    new_enemies: Query<(Entity, Option<&HealerAura>), Added<Enemy>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (entity, healer) in &new_enemies {
        // Background (dark, slightly larger)
        commands.spawn((
            Mesh3d(meshes.add(Rectangle::new(HP_BAR_WIDTH + 0.06, HP_BAR_HEIGHT + 0.04))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgba(0.0, 0.0, 0.0, 0.7),
                alpha_mode: AlphaMode::Blend,
                unlit: true,
                double_sided: true,
                ..default()
            })),
            Transform::default(),
            HealthBarBg(entity),
            GameWorldEntity,
        ));

        // Fill (starts green)
        commands.spawn((
            Mesh3d(meshes.add(Rectangle::new(HP_BAR_WIDTH, HP_BAR_HEIGHT))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgb(0.0, 1.0, 0.0),
                unlit: true,
                double_sided: true,
                ..default()
            })),
            Transform::default(),
            HealthBar(entity),
            GameWorldEntity,
        ));

        // Thin green ring under healer enemies
        if healer.is_some() {
            commands.spawn((
                Mesh3d(meshes.add(Annulus::new(0.55, 0.65))),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: Color::srgba(0.2, 0.9, 0.3, 0.5),
                    emissive: LinearRgba::new(0.2, 0.8, 0.3, 1.0),
                    alpha_mode: AlphaMode::Blend,
                    unlit: true,
                    double_sided: true,
                    ..default()
                })),
                Transform::default(),
                HealerRing(entity),
                GameWorldEntity,
            ));
        }
    }
}

/// Updates HP bar positions (billboard toward camera), scale, and color.
pub fn update_health_bars(
    mut commands: Commands,
    enemies: Query<(&Transform, &Health, Option<&BossEnemy>), With<Enemy>>,
    mut bars: Query<
        (Entity, &HealthBar, &mut Transform, &MeshMaterial3d<StandardMaterial>),
        (Without<Enemy>, Without<HealthBarBg>),
    >,
    mut bg_bars: Query<
        (Entity, &HealthBarBg, &mut Transform),
        (Without<Enemy>, Without<HealthBar>),
    >,
    mut materials: ResMut<Assets<StandardMaterial>>,
    camera_q: Query<
        &Transform,
        (With<Camera3d>, Without<Enemy>, Without<HealthBar>, Without<HealthBarBg>),
    >,
) {
    let Ok(cam_tf) = camera_q.get_single() else { return };

    // Update fill bars
    for (entity, bar, mut transform, mat_handle) in &mut bars {
        let Ok((enemy_tf, health, boss)) = enemies.get(bar.0) else {
            commands.entity(entity).despawn();
            continue;
        };

        let hp_pct = (health.current / health.max).clamp(0.0, 1.0);
        // Scale HP bar height with model size so it's always above the head
        let scale_factor = (enemy_tf.scale.x * 2.5).clamp(1.0, 2.5);
        let bar_pos = enemy_tf.translation + Vec3::Y * HP_BAR_Y_OFFSET * scale_factor;

        // Billboard: face camera
        transform.rotation = cam_tf.rotation;
        // Offset slightly toward camera to render in front of background
        let to_camera = (cam_tf.translation - bar_pos).normalize_or_zero();
        // Anchor the bar at the left: offset right by half the missing portion
        // so it shrinks from right to left
        let right = cam_tf.right();
        let offset = -right * (1.0 - hp_pct) * HP_BAR_WIDTH * 0.5;
        transform.translation = bar_pos + to_camera * 0.02 + offset;
        transform.scale = Vec3::new(hp_pct, 1.0, 1.0);

        // Color: bosses always reddish-orange, others green → yellow → red
        if let Some(mat) = materials.get_mut(&mat_handle.0) {
            mat.base_color = if boss.is_some() {
                Color::srgb(0.9, 0.35, 0.1)
            } else {
                hp_color(hp_pct)
            };
        }
    }

    // Update background bars
    for (entity, bg, mut transform) in &mut bg_bars {
        let Ok((enemy_tf, _, _)) = enemies.get(bg.0) else {
            commands.entity(entity).despawn();
            continue;
        };

        let scale_factor = (enemy_tf.scale.x * 2.5).clamp(1.0, 2.5);
        let bar_pos = enemy_tf.translation + Vec3::Y * HP_BAR_Y_OFFSET * scale_factor;
        transform.rotation = cam_tf.rotation;
        transform.translation = bar_pos;
    }
}

/// Updates healer ring position (flat on ground under the enemy).
pub fn update_healer_rings(
    mut commands: Commands,
    enemies: Query<&Transform, With<Enemy>>,
    mut rings: Query<(Entity, &HealerRing, &mut Transform), Without<Enemy>>,
) {
    for (ring_entity, ring, mut transform) in &mut rings {
        let Ok(enemy_tf) = enemies.get(ring.0) else {
            commands.entity(ring_entity).despawn();
            continue;
        };
        transform.translation = Vec3::new(enemy_tf.translation.x, 0.05, enemy_tf.translation.z);
        transform.rotation = Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2);
    }
}

/// HP bar color: green at full → yellow at half → red at empty.
fn hp_color(pct: f32) -> Color {
    if pct > 0.5 {
        let t = (1.0 - pct) * 2.0;
        Color::srgb(t, 1.0, 0.0)
    } else {
        let t = pct * 2.0;
        Color::srgb(1.0, t, 0.0)
    }
}

/// Removes enemies that reached the end of the path and subtracts a life.
pub fn check_enemy_leak(
    mut commands: Commands,
    query: Query<(Entity, &PathFollower), With<Enemy>>,
    mut game: ResMut<GameData>,
    mut wave: ResMut<WaveState>,
    audio_assets: Option<Res<super::audio::AudioAssets>>,
    level_path: Res<crate::resources::LevelPath>,
    vol_settings: Res<VolumeSettings>,
) {
    let last_segment = level_path.0.len() - 1;
    let mut leaked = false;

    for (entity, follower) in &query {
        if follower.segment >= last_segment {
            game.lives = game.lives.saturating_sub(1);
            wave.active_enemies = wave.active_enemies.saturating_sub(1);
            commands.entity(entity).despawn_recursive();
            leaked = true;
        }
    }

    if leaked {
        if let Some(audio) = audio_assets {
            if audio.all_loaded {
                commands.spawn((
                    AudioPlayer(audio.enemy_leak.clone()),
                    PlaybackSettings {
                        volume: bevy::audio::Volume::new(vol_settings.sfx),
                        ..PlaybackSettings::DESPAWN
                    },
                ));
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Damage numbers
// ---------------------------------------------------------------------------

/// Adds LastHealth tracker to newly spawned enemies.
pub fn init_damage_tracking(
    mut commands: Commands,
    new_enemies: Query<(Entity, &Health), Added<Enemy>>,
) {
    for (entity, health) in &new_enemies {
        commands.entity(entity).insert(LastHealth(health.current));
    }
}

/// Detects health decreases and spawns floating damage numbers.
pub fn spawn_damage_numbers(
    mut commands: Commands,
    mut enemies: Query<(&Transform, &Health, &mut LastHealth), With<Enemy>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (transform, health, mut last) in &mut enemies {
        let diff = last.0 - health.current;
        if diff > 0.5 {
            let pos = transform.translation;
            // Deterministic horizontal jitter based on damage amount
            let jitter_x = ((diff * 73.0) % 1.0 - 0.5) * 0.8;

            // Color: white for small, yellow for medium, red for big hits
            let (color, emissive) = if diff >= 40.0 {
                (Color::srgb(1.0, 0.2, 0.1), LinearRgba::new(2.0, 0.4, 0.2, 1.0))
            } else if diff >= 15.0 {
                (Color::srgb(1.0, 0.9, 0.2), LinearRgba::new(2.0, 1.8, 0.4, 1.0))
            } else {
                (Color::WHITE, LinearRgba::new(2.0, 2.0, 2.0, 1.0))
            };

            // Scale sphere size by damage amount (small sphere = small hit)
            let size = (diff / 30.0).clamp(0.06, 0.2);

            let start_y = pos.y + 2.0;
            commands.spawn((
                Mesh3d(meshes.add(Sphere::new(size))),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: color,
                    emissive,
                    unlit: true,
                    ..default()
                })),
                Transform::from_translation(Vec3::new(pos.x + jitter_x, start_y, pos.z)),
                DamageNumber {
                    lifetime: 0.8,
                    elapsed: 0.0,
                    start_y,
                },
                GameWorldEntity,
            ));
        }
        last.0 = health.current;
    }
}

/// Animates damage numbers: rise upward and fade out.
pub fn animate_damage_numbers(
    mut commands: Commands,
    mut numbers: Query<(Entity, &mut Transform, &DamageNumber, &MeshMaterial3d<StandardMaterial>)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    time: Res<Time>,
) {
    let dt = time.delta_secs();
    for (entity, mut transform, number, mat_handle) in &mut numbers {
        let elapsed = number.elapsed + dt;
        if elapsed >= number.lifetime {
            commands.entity(entity).despawn();
            continue;
        }

        // Rise
        let progress = elapsed / number.lifetime;
        transform.translation.y = number.start_y + progress * 1.5;

        // Fade out
        if let Some(mat) = materials.get_mut(&mat_handle.0) {
            let alpha = 1.0 - progress;
            mat.base_color = mat.base_color.with_alpha(alpha);
            mat.emissive = mat.emissive * alpha;
        }
    }
}

/// Tick elapsed time on damage numbers (separate from animation for borrowing).
pub fn tick_damage_numbers(
    mut numbers: Query<&mut DamageNumber>,
    time: Res<Time>,
) {
    let dt = time.delta_secs();
    for mut number in &mut numbers {
        number.elapsed += dt;
    }
}
