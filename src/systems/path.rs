use bevy::prelude::*;
use crate::components::*;

/// Moves each enemy along the level's waypoint path.
/// Respects slow debuff by reducing effective speed.
/// Adds procedural animation: bobbing, squash/stretch while moving.
pub fn move_enemies(
    mut query: Query<(Entity, &mut Transform, &mut PathFollower, &ModelScale, Option<&SlowDebuff>, Option<&GolemBlocked>, Option<&QuadLegBones>, Option<&EnemyAnimState>), With<Enemy>>,
    time: Res<Time>,
    level_path: Res<crate::resources::LevelPath>,
) {
    let path = &level_path.0;
    let last_segment = path.len() - 1;
    let t = time.elapsed_secs();

    for (entity, mut transform, mut follower, model_scale, slow, golem_blocked, quad_legs, anim_state) in &mut query {
        if follower.segment >= last_segment {
            continue;
        }

        let has_skeletal = anim_state.is_some();

        // Golem-blocked enemies don't move along the path
        if golem_blocked.is_some() {
            follower.speed = 0.0;

            // Only do procedural headbutt bob for enemies without skeletal or quad-leg animations
            if !has_skeletal && quad_legs.is_none() {
                let phase = entity.index() as f32 * 1.7;
                let base_scale = model_scale.0;
                let cycle = (t * 5.0 + phase).sin();
                let jab = cycle.max(0.0);
                let s = jab * 0.12;
                transform.scale = Vec3::new(
                    base_scale * (1.0 + s),
                    base_scale * (1.0 - s),
                    base_scale * (1.0 + s),
                );
                transform.translation.y = follower.y_offset + jab * 0.1;
            }

            continue;
        }

        // Apply slow debuff if present
        let effective_speed = match slow {
            Some(debuff) => follower.base_speed * debuff.factor,
            None => follower.base_speed,
        };
        follower.speed = effective_speed;

        let start = path[follower.segment];
        let end = path[follower.segment + 1];
        let segment_length = (end - start).length();

        follower.progress += (effective_speed * time.delta_secs()) / segment_length;

        if follower.progress >= 1.0 {
            follower.progress = 0.0;
            follower.segment += 1;
            if follower.segment >= last_segment {
                transform.translation = path[last_segment];
                continue;
            }
        }

        let seg_start = path[follower.segment];
        let seg_end = path[follower.segment + 1];
        let pos = seg_start.lerp(seg_end, follower.progress);

        // Apply lateral offset perpendicular to path direction
        let dir = (seg_end - seg_start).normalize();
        let lateral = Vec3::new(-dir.z, 0.0, dir.x) * follower.lateral_offset;

        if has_skeletal {
            // Skeletal animation handles visual quality — just set position
            transform.translation = Vec3::new(pos.x + lateral.x, follower.y_offset, pos.z + lateral.z);
        } else if quad_legs.is_some() {
            // Quad-leg enemies: leg bones handle animation, no squash/stretch (causes distortion)
            transform.translation = Vec3::new(pos.x + lateral.x, follower.y_offset, pos.z + lateral.z);
        } else {
            // Procedural animation: bob + squash/stretch
            let phase = entity.index() as f32 * 1.7;
            let bob_speed = effective_speed * 2.5;
            let bob_y = (t * bob_speed + phase).sin() * 0.15;
            transform.translation = Vec3::new(pos.x + lateral.x, follower.y_offset + bob_y, pos.z + lateral.z);

            let base_scale = model_scale.0;
            let squash = (t * bob_speed + phase).sin() * 0.06;
            transform.scale = Vec3::new(
                base_scale * (1.0 + squash),
                base_scale * (1.0 - squash),
                base_scale * (1.0 + squash),
            );
        }

        // Face movement direction
        let direction = (seg_end - seg_start).normalize();
        if direction.length_squared() > 0.001 {
            let target_rot = Quat::from_rotation_y(f32::atan2(direction.x, direction.z));
            transform.rotation = transform.rotation.slerp(target_rot, 0.15);
        }
    }
}
