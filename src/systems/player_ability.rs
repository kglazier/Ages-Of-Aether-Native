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
    asset_server: Res<AssetServer>,
    mut shake: ResMut<super::camera::CameraShake>,
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

            // VFX — very subtle orange ground circle (no grow, fixed size)
            commands.spawn((
                Mesh3d(meshes.add(Circle::new(METEOR_RADIUS * 0.5))),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: Color::srgba(0.7, 0.4, 0.15, 0.04),
                    emissive: LinearRgba::NONE,
                    alpha_mode: AlphaMode::Blend,
                    unlit: true,
                    double_sided: true,
                    ..default()
                })),
                Transform::from_translation(Vec3::new(target_pos.x, 0.1, target_pos.z))
                    .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
                TimedDespawn { remaining: 1.5 },
            ));

            // VFX — falling rock (dark, no glow)
            commands.spawn((
                Mesh3d(meshes.add(Sphere::new(0.4))),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: Color::srgb(0.3, 0.2, 0.1),
                    emissive: LinearRgba::NONE,
                    ..default()
                })),
                Transform::from_translation(Vec3::new(target_pos.x, 15.0, target_pos.z))
                    .with_scale(Vec3::new(1.0, 0.7, 1.0)),
                MeteorFalling {
                    target_y: 0.3,
                    speed: 45.0,
                },
            ));

            // Camera shake on impact
            shake.trigger(0.4, 0.25);

            info!("Meteor struck at {:?}", target_pos);
        }
        PlayerAbilityType::Reinforcements => {
            if abilities.reinforcement_cooldown > 0.0 { return; }
            let def = player_ability_def(PlayerAbilityType::Reinforcements);
            abilities.reinforcement_cooldown = def.cooldown;

            // Spawn soldiers using golem model with mossy green tint
            let golem_scene = asset_server.load("models/golems/golem.glb#Scene0");
            for i in 0..REINFORCEMENT_COUNT {
                let offset = Vec3::new(
                    if i % 2 == 0 { -1.0 } else { 1.0 },
                    0.0,
                    0.0,
                );
                let pos = target_pos + offset;
                let soldier_id = commands.spawn((
                    SceneRoot(golem_scene.clone()),
                    Transform::from_translation(Vec3::new(pos.x, super::golem::GOLEM_Y_OFFSET, pos.z))
                        .with_scale(Vec3::splat(60.0)),
                    Health { current: REINFORCEMENT_HP, max: REINFORCEMENT_HP },
                    AttackDamage(REINFORCEMENT_DAMAGE),
                    AttackRange(2.0),
                    AttackTimer { cooldown: 1.0, elapsed: 0.0 },
                    Golem,
                    super::golem::GolemNeedsAnimation,
                    BlockingEnemy(None),
                    GolemAttack { damage: REINFORCEMENT_DAMAGE, cooldown: 1.0, elapsed: 0.0 },
                    GolemRallyPoint(Vec3::new(pos.x, 0.0, pos.z)),
                    ReinforcementSoldier { remaining: REINFORCEMENT_DURATION },
                    crate::components::GameWorldEntity,
                )).id();
                // GolemOwner is required by golem_assign_targets — self-own for reinforcements
                commands.entity(soldier_id).insert(GolemOwner(soldier_id));
            }

            // VFX — subtle green placement ring
            commands.spawn((
                Mesh3d(meshes.add(Annulus::new(1.3, 1.5))),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: Color::srgba(0.3, 0.6, 0.3, 0.15),
                    emissive: LinearRgba::new(0.1, 0.3, 0.1, 1.0),
                    alpha_mode: AlphaMode::Blend,
                    unlit: true,
                    double_sided: true,
                    ..default()
                })),
                Transform::from_translation(Vec3::new(target_pos.x, 0.1, target_pos.z))
                    .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
                TimedDespawn { remaining: 0.5 },
            ));

            info!("Reinforcements deployed at {:?}", target_pos);
        }
    }

    targeting.0 = None;
}

/// Falling meteor rock that animates downward then despawns.
#[derive(Component)]
pub struct MeteorFalling {
    pub target_y: f32,
    pub speed: f32,
}

/// Marker for the targeting ring visual.
#[derive(Component)]
pub struct AbilityTargetRing;

/// Show/update/hide targeting ring that follows cursor when targeting an ability.
pub fn update_targeting_ring(
    mut commands: Commands,
    targeting: Res<PlayerAbilityTargeting>,
    ring_q: Query<Entity, With<AbilityTargetRing>>,
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    touches: Res<Touches>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    if targeting.0.is_none() {
        // No targeting — despawn ring if it exists
        for entity in &ring_q {
            commands.entity(entity).despawn_recursive();
        }
        return;
    }

    let ability = targeting.0.unwrap();
    let radius = match ability {
        PlayerAbilityType::Meteor => METEOR_RADIUS,
        PlayerAbilityType::Reinforcements => 2.0,
    };
    let color = match ability {
        PlayerAbilityType::Meteor => Color::srgba(1.0, 0.4, 0.1, 0.4),
        PlayerAbilityType::Reinforcements => Color::srgba(0.3, 0.8, 0.3, 0.4),
    };

    // Get cursor/touch position projected to ground
    let Ok(window) = windows.get_single() else { return };
    let screen_pos = if let Some(pos) = window.cursor_position() {
        pos
    } else if let Some(touch) = touches.iter().next() {
        touch.position()
    } else {
        return;
    };

    let Ok((camera, cam_transform)) = camera_query.get_single() else { return };
    let Ok(ray) = camera.viewport_to_world(cam_transform, screen_pos) else { return };
    let Some(distance) = ray.intersect_plane(Vec3::ZERO, InfinitePlane3d::new(Vec3::Y)) else {
        return;
    };
    let world_pos = ray.get_point(distance);

    if let Ok(entity) = ring_q.get_single() {
        // Update existing ring position
        commands.entity(entity).insert(
            Transform::from_translation(Vec3::new(world_pos.x, 0.15, world_pos.z))
                .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2))
        );
    } else {
        // Spawn ring
        commands.spawn((
            Mesh3d(meshes.add(Annulus::new(radius * 0.85, radius))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: color,
                emissive: LinearRgba::new(0.5, 0.3, 0.1, 0.5),
                alpha_mode: AlphaMode::Blend,
                unlit: true,
                double_sided: true,
                ..default()
            })),
            Transform::from_translation(Vec3::new(world_pos.x, 0.15, world_pos.z))
                .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
            AbilityTargetRing,
        ));
    }
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

/// Tick TimedDespawn entities — just remove them when time's up.
pub fn tick_timed_despawns(
    mut commands: Commands,
    mut q: Query<(Entity, &mut TimedDespawn)>,
    time: Res<Time>,
) {
    let dt = time.delta_secs();
    for (entity, mut td) in &mut q {
        td.remaining -= dt;
        if td.remaining <= 0.0 {
            commands.entity(entity).despawn_recursive();
        }
    }
}

/// Animate meteor rocks falling from the sky, despawn on landing.
pub fn animate_meteor_falling(
    mut commands: Commands,
    mut meteors: Query<(Entity, &mut Transform, &MeteorFalling)>,
    time: Res<Time>,
) {
    let dt = time.delta_secs();
    for (entity, mut transform, meteor) in &mut meteors {
        transform.translation.y -= meteor.speed * dt;
        if transform.translation.y <= meteor.target_y {
            commands.entity(entity).despawn();
        }
    }
}
