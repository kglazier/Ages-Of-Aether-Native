use bevy::prelude::*;
use crate::components::*;
use crate::data::*;
use crate::resources::*;

/// Tick global ability cooldowns each frame.
pub fn tick_player_ability_cooldowns(
    mut abilities: ResMut<PlayerAbilities>,
    time: Res<Time>,
) {
    let dt = time.delta_secs();
    if abilities.meteor_cooldown > 0.0 {
        abilities.meteor_cooldown = (abilities.meteor_cooldown - dt).max(0.0);
    }
    if abilities.reinforcement_cooldown > 0.0 {
        abilities.reinforcement_cooldown = (abilities.reinforcement_cooldown - dt).max(0.0);
    }
}

/// Execute a player ability at a target location.
pub fn execute_player_ability(
    mut commands: Commands,
    mut targeting: ResMut<PlayerAbilityTargeting>,
    mut move_cmd: ResMut<HeroMoveCommand>,
    mut abilities: ResMut<PlayerAbilities>,
    mut enemies: Query<(Entity, &Transform, &mut Health, &Armor), With<Enemy>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let Some(ability_type) = targeting.0 else { return };

    // We consume the hero move command as our target position
    let Some(target_pos) = move_cmd.0.take() else { return };

    match ability_type {
        PlayerAbilityType::Meteor => {
            if abilities.meteor_cooldown > 0.0 { return; }
            let def = player_ability_def(PlayerAbilityType::Meteor);
            abilities.meteor_cooldown = def.cooldown;

            // Deal damage to enemies in radius
            for (_entity, enemy_tf, mut health, armor) in &mut enemies {
                let dist = target_pos.distance(enemy_tf.translation);
                if dist <= METEOR_RADIUS {
                    let reduction = armor.physical / (armor.physical + 100.0);
                    health.current -= METEOR_DAMAGE * (1.0 - reduction);
                }
            }

            // VFX — expanding ring
            commands.spawn((
                Mesh3d(meshes.add(Annulus::new(METEOR_RADIUS * 0.7, METEOR_RADIUS))),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: Color::srgba(1.0, 0.4, 0.1, 0.8),
                    emissive: LinearRgba::new(2.0, 0.8, 0.2, 1.0),
                    alpha_mode: AlphaMode::Blend,
                    unlit: true,
                    double_sided: true,
                    ..default()
                })),
                Transform::from_translation(Vec3::new(target_pos.x, 0.3, target_pos.z))
                    .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
                DeathEffect { lifetime: 0.8, elapsed: 0.0 },
            ));

            info!("Meteor struck at {:?}", target_pos);
        }
        PlayerAbilityType::Reinforcements => {
            if abilities.reinforcement_cooldown > 0.0 { return; }
            let def = player_ability_def(PlayerAbilityType::Reinforcements);
            abilities.reinforcement_cooldown = def.cooldown;

            // Spawn soldiers around target position
            for i in 0..REINFORCEMENT_COUNT {
                let offset = Vec3::new(
                    if i % 2 == 0 { -1.0 } else { 1.0 },
                    0.0,
                    0.0,
                );
                let pos = target_pos + offset;
                commands.spawn((
                    Mesh3d(meshes.add(Capsule3d::new(0.3, 1.0))),
                    MeshMaterial3d(materials.add(StandardMaterial {
                        base_color: Color::srgb(0.3, 0.7, 0.3),
                        emissive: LinearRgba::new(0.3, 0.5, 0.2, 1.0),
                        ..default()
                    })),
                    Transform::from_translation(Vec3::new(pos.x, 0.7, pos.z)),
                    Health { current: REINFORCEMENT_HP, max: REINFORCEMENT_HP },
                    AttackDamage(REINFORCEMENT_DAMAGE),
                    AttackRange(2.0),
                    AttackTimer { cooldown: 1.0, elapsed: 0.0 },
                    Golem,
                    ReinforcementSoldier { remaining: REINFORCEMENT_DURATION },
                ));
            }

            // VFX
            commands.spawn((
                Mesh3d(meshes.add(Annulus::new(1.5, 2.0))),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: Color::srgba(0.3, 0.7, 0.3, 0.6),
                    emissive: LinearRgba::new(0.4, 1.0, 0.4, 1.0),
                    alpha_mode: AlphaMode::Blend,
                    unlit: true,
                    double_sided: true,
                    ..default()
                })),
                Transform::from_translation(Vec3::new(target_pos.x, 0.2, target_pos.z))
                    .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
                DeathEffect { lifetime: 0.6, elapsed: 0.0 },
            ));

            info!("Reinforcements deployed at {:?}", target_pos);
        }
    }

    targeting.0 = None;
}

/// Tick reinforcement soldier lifetimes and despawn when expired.
pub fn tick_reinforcements(
    mut commands: Commands,
    mut soldiers: Query<(Entity, &mut ReinforcementSoldier)>,
    time: Res<Time>,
) {
    let dt = time.delta_secs();
    for (entity, mut soldier) in &mut soldiers {
        soldier.remaining -= dt;
        if soldier.remaining <= 0.0 {
            commands.entity(entity).despawn_recursive();
        }
    }
}
