use bevy::prelude::*;
use crate::components::*;
use crate::data::*;

/// Resource: which specialization was chosen from the UI.
#[derive(Resource, Default)]
pub struct SpecializationChosen(pub Option<TowerSpecialization>);

/// Apply specialization to the selected tower: modify stats, spawn aura if needed.
pub fn apply_specialization(
    mut commands: Commands,
    mut spec_chosen: ResMut<SpecializationChosen>,
    mut selection: ResMut<crate::resources::Selection>,
    mut towers: Query<(
        Entity,
        &Element,
        &TowerLevel,
        &mut TowerInvestment,
        &mut AttackDamage,
        &mut AttackRange,
        &mut AttackTimer,
    ), (With<Tower>, Without<TowerSpec>)>,
    mut game: ResMut<crate::resources::GameData>,
    golems: Query<(Entity, &crate::components::GolemOwner), With<Golem>>,
    audio_assets: Option<Res<super::audio::AudioAssets>>,
) {
    let Some(spec) = spec_chosen.0.take() else { return };

    let crate::resources::Selection::Tower(tower_entity) = *selection else { return };

    let Ok((entity, element, level, mut investment, mut damage, mut range, mut timer)) =
        towers.get_mut(tower_entity)
    else {
        return;
    };

    // Must be max level
    if level.0 < 2 { return; }

    // Check cost
    let specs = element_specializations(*element);
    let def = specs.iter().find(|(s, _)| *s == spec).map(|(_, d)| d);
    let Some(def) = def else { return };
    if game.gold < def.cost { return; }

    game.gold -= def.cost;
    investment.0 += def.cost;

    // Apply stat modifications
    match spec {
        TowerSpecialization::Railgun => {
            damage.0 *= 2.5;
            range.0 += 3.0;
            timer.cooldown /= 0.35; // Much slower
        }
        TowerSpecialization::MeteorTower => {
            damage.0 *= 3.0;
            range.0 += 2.0;
            timer.cooldown /= 0.3;
        }
        TowerSpecialization::ShatterMage => {
            // Stats stay the same, effect is in projectile hit logic
        }
        TowerSpecialization::StormSpire => {
            // Stats stay the same, effect is chain lightning on hit
        }
        TowerSpecialization::InfernoCannon => {
            damage.0 *= 1.5;
        }
        TowerSpecialization::MountainKing => {
            // Kill existing golems — they'll respawn with boosted stats
            for (golem_entity, owner) in &golems {
                if owner.0 == entity {
                    commands.entity(golem_entity).despawn_recursive();
                }
            }
        }
        TowerSpecialization::BrambleGrove => {
            // Kill existing golems — this spec replaces them with an aura
            for (golem_entity, owner) in &golems {
                if owner.0 == entity {
                    commands.entity(golem_entity).despawn_recursive();
                }
            }
            // Spawn aura visual (flat ring on ground)
            // The tick system handles the actual effect
        }
        TowerSpecialization::BlizzardTower => {
            // This spec replaces projectile firing with a constant slow field
            // The tick system handles the actual effect
        }
    }

    commands.entity(entity).insert(TowerSpec(spec));

    // Play upgrade SFX
    if let Some(audio) = audio_assets {
        if audio.all_loaded {
            commands.spawn((
                AudioPlayer(audio.tower_upgrade.clone()),
                PlaybackSettings::DESPAWN,
            ));
        }
    }

    // Refresh tower panel by re-triggering selection change detection
    *selection = crate::resources::Selection::Tower(tower_entity);

    info!("Tower specialized: {:?}", spec);
}

/// Tick aura effects for Bramble Grove (slow+damage) and Blizzard Tower (slow field).
pub fn tick_tower_auras(
    mut commands: Commands,
    towers: Query<(&Transform, &TowerSpec, &AttackRange), With<Tower>>,
    mut enemies: Query<(Entity, &Transform, &mut Health, Option<&SlowDebuff>, Option<&BurnDebuff>), With<Enemy>>,
    time: Res<Time>,
) {
    let dt = time.delta_secs();
    if dt == 0.0 { return; }

    for (tower_tf, spec, range) in &towers {
        match spec.0 {
            TowerSpecialization::BlizzardTower => {
                // Constant AoE slow field
                let radius = range.0;
                for (entity, enemy_tf, _, slow, _) in &enemies {
                    if slow.is_some() { continue; }
                    let dist = tower_tf.translation.distance(enemy_tf.translation);
                    if dist <= radius {
                        commands.entity(entity).insert(SlowDebuff {
                            factor: 0.4,
                            remaining: 0.5, // Re-applied every frame
                        });
                    }
                }
            }
            TowerSpecialization::BrambleGrove => {
                // Slow + damage aura
                let radius = 5.0;
                for (entity, enemy_tf, mut health, slow, _) in &mut enemies {
                    let dist = tower_tf.translation.distance(enemy_tf.translation);
                    if dist <= radius {
                        // Damage
                        health.current -= 3.0 * dt;
                        // Slow
                        if slow.is_none() {
                            commands.entity(entity).insert(SlowDebuff {
                                factor: 0.6,
                                remaining: 0.5,
                            });
                        }
                    }
                }
            }
            _ => {}
        }
    }
}

/// Tick burn zones (from Inferno Cannon impacts) — damage enemies standing in them.
pub fn tick_burn_zones(
    mut commands: Commands,
    mut zones: Query<(Entity, &Transform, &mut BurnZone)>,
    mut enemies: Query<(&Transform, &mut Health), With<Enemy>>,
    time: Res<Time>,
) {
    let dt = time.delta_secs();
    for (zone_entity, zone_tf, mut zone) in &mut zones {
        zone.remaining -= dt;
        if zone.remaining <= 0.0 {
            commands.entity(zone_entity).despawn();
            continue;
        }

        for (enemy_tf, mut health) in &mut enemies {
            let dist = zone_tf.translation.distance(enemy_tf.translation);
            if dist <= zone.radius {
                health.current -= zone.dps * dt;
            }
        }
    }
}
