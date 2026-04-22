use bevy::prelude::*;
use crate::components::*;
use crate::data::*;
use crate::resources::*;

/// Resource: which ability index (0-2) was just activated by the UI.
#[derive(Resource, Default)]
pub struct AbilityActivated(pub Option<usize>);

/// Tick ability cooldowns each frame.
pub fn tick_ability_cooldowns(
    mut hero_q: Query<&mut HeroAbilities, (With<Hero>, Without<HeroRespawnTimer>)>,
    time: Res<Time>,
) {
    let dt = time.delta_secs();
    for mut abilities in &mut hero_q {
        for cd in &mut abilities.cooldowns {
            if *cd > 0.0 {
                *cd = (*cd - dt).max(0.0);
            }
        }
    }
}

/// Tick damage reduction buff.
pub fn tick_hero_buffs(
    mut commands: Commands,
    mut hero_q: Query<(Entity, &mut HeroDamageReduction), With<Hero>>,
    time: Res<Time>,
) {
    for (entity, mut buff) in &mut hero_q {
        buff.remaining -= time.delta_secs();
        if buff.remaining <= 0.0 {
            commands.entity(entity).remove::<HeroDamageReduction>();
        }
    }
}

/// Execute the activated ability.
pub fn execute_ability(
    mut commands: Commands,
    mut ability_res: ResMut<AbilityActivated>,
    mut hero_q: Query<
        (Entity, &Transform, &mut Health, &HeroAttackDamage, &mut HeroAbilities),
        (With<Hero>, Without<HeroRespawnTimer>),
    >,
    mut enemies: Query<(Entity, &Transform, &mut Health, &Armor), (With<Enemy>, Without<Hero>)>,
    active_hero: Res<ActiveHeroType>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let Some(idx) = ability_res.0.take() else { return };

    let Ok((hero_entity, hero_tf, mut hero_health, hero_dmg, mut abilities)) = hero_q.get_single_mut() else {
        return;
    };

    if idx >= 3 || abilities.cooldowns[idx] > 0.0 {
        return;
    }

    let defs = hero_abilities(active_hero.0);
    let def = &defs[idx];
    let hero_pos = hero_tf.translation;

    // Start cooldown
    abilities.cooldowns[idx] = def.cooldown;

    // Execute effect
    match def.effect {
        AbilityEffect::AoeDamage { damage, radius } => {
            for (_, enemy_tf, mut health, armor) in &mut enemies {
                let dist = hero_pos.distance(enemy_tf.translation);
                if dist <= radius {
                    let reduction = armor.physical / (armor.physical + 100.0);
                    health.current -= damage * (1.0 - reduction);
                }
            }
            spawn_ability_vfx(&mut commands, &mut meshes, &mut materials, hero_pos, radius, def.color);
        }
        AbilityEffect::Heal { percent } => {
            let heal_amount = hero_health.max * percent;
            hero_health.current = (hero_health.current + heal_amount).min(hero_health.max);
            spawn_ability_vfx(&mut commands, &mut meshes, &mut materials, hero_pos, 1.5, def.color);
        }
        AbilityEffect::AoeSlow { factor, duration, radius } => {
            for (entity, enemy_tf, _, _) in &mut enemies {
                let dist = hero_pos.distance(enemy_tf.translation);
                if dist <= radius {
                    commands.entity(entity).insert(SlowDebuff {
                        factor,
                        remaining: duration,
                    });
                }
            }
            spawn_ability_vfx(&mut commands, &mut meshes, &mut materials, hero_pos, radius, def.color);
        }
        AbilityEffect::AoeDamageAndSlow { damage, radius, slow_factor, slow_duration } => {
            for (entity, enemy_tf, mut health, armor) in &mut enemies {
                let dist = hero_pos.distance(enemy_tf.translation);
                if dist <= radius {
                    let reduction = armor.physical / (armor.physical + 100.0);
                    health.current -= damage * (1.0 - reduction);
                    commands.entity(entity).insert(SlowDebuff {
                        factor: slow_factor,
                        remaining: slow_duration,
                    });
                }
            }
            spawn_ability_vfx(&mut commands, &mut meshes, &mut materials, hero_pos, radius, def.color);
        }
        AbilityEffect::AoeDamageAndBurn { damage, radius, burn_dps, burn_duration } => {
            for (entity, enemy_tf, mut health, armor) in &mut enemies {
                let dist = hero_pos.distance(enemy_tf.translation);
                if dist <= radius {
                    let reduction = armor.physical / (armor.physical + 100.0);
                    health.current -= damage * (1.0 - reduction);
                    commands.entity(entity).insert(BurnDebuff {
                        dps: burn_dps,
                        remaining: burn_duration,
                    });
                }
            }
            spawn_ability_vfx(&mut commands, &mut meshes, &mut materials, hero_pos, radius, def.color);
        }
        AbilityEffect::SingleTargetBurst { multiplier, range } => {
            // Find nearest enemy in range
            let mut best: Option<(Entity, f32)> = None;
            for (entity, enemy_tf, _, _) in &enemies {
                let dist = hero_pos.distance(enemy_tf.translation);
                if dist <= range {
                    if best.is_none() || dist < best.unwrap().1 {
                        best = Some((entity, dist));
                    }
                }
            }
            // Always show VFX so the button feels responsive even when no enemy is in range.
            spawn_ability_vfx(&mut commands, &mut meshes, &mut materials, hero_pos, 1.5, def.color);
            if let Some((target, _)) = best {
                if let Ok((_, _, mut health, armor)) = enemies.get_mut(target) {
                    let total_damage = hero_dmg.0 * multiplier;
                    let reduction = armor.physical / (armor.physical + 100.0);
                    health.current -= total_damage * (1.0 - reduction);
                }
            } else {
                // No target — refund cooldown so the player can try again immediately.
                abilities.cooldowns[idx] = 0.0;
            }
        }
        AbilityEffect::DamageReduction { factor, duration } => {
            commands.entity(hero_entity).insert(HeroDamageReduction {
                factor,
                remaining: duration,
            });
            spawn_ability_vfx(&mut commands, &mut meshes, &mut materials, hero_pos, 2.0, def.color);
        }
    }

    info!("Hero used ability: {}", def.name);
}

/// Spawn a quick expanding ring VFX at the ability's impact point.
fn spawn_ability_vfx(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    pos: Vec3,
    radius: f32,
    color: [f32; 3],
) {
    commands.spawn((
        Mesh3d(meshes.add(Annulus::new(radius * 0.8, radius))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgba(color[0], color[1], color[2], 0.7),
            emissive: LinearRgba::new(color[0] * 2.0, color[1] * 2.0, color[2] * 2.0, 1.0),
            alpha_mode: AlphaMode::Blend,
            unlit: true,
            double_sided: true,
            ..default()
        })),
        Transform::from_translation(Vec3::new(pos.x, 0.2, pos.z))
            .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
        DeathEffect { lifetime: 0.5, elapsed: 0.0 }, // reuse death effect for fade-out
    ));
}
