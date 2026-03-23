use bevy::prelude::*;
use crate::components::*;
use crate::resources::*;

/// Handles mouse clicks and touch taps on the game world.
/// - Tap hero → select hero
/// - Tap empty build spot → select it (opens build menu in UI)
/// - Tap existing tower → select it (opens upgrade panel in UI)
/// - Tap empty ground while hero selected → move hero
/// - Tap empty ground otherwise → deselect
/// - Escape → deselect
pub fn handle_world_click(
    mouse: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    touches: Res<Touches>,
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    spots: Query<(Entity, &BuildSpot, &Transform)>,
    towers: Query<(Entity, &Transform), With<Tower>>,
    hero_q: Query<&Transform, (With<Hero>, Without<Tower>, Without<BuildSpot>, Without<HeroRespawnTimer>)>,
    mut selection: ResMut<Selection>,
    mut hero_move_cmd: ResMut<crate::resources::HeroMoveCommand>,
    // Don't process world clicks if a UI button is being hovered/pressed
    ui_interactions: Query<&Interaction, With<Button>>,
) {
    // Don't handle world clicks while setting rally point
    if matches!(*selection, Selection::SettingRallyPoint(_)) {
        return;
    }

    // Escape clears selection
    if keys.just_pressed(KeyCode::Escape) {
        *selection = Selection::None;
        return;
    }

    // Get the click/tap position — mouse click or touch tap
    let click_pos = if mouse.just_pressed(MouseButton::Left) {
        let Ok(window) = windows.get_single() else { return };
        window.cursor_position()
    } else if let Some(touch) = touches.iter_just_pressed().next() {
        Some(touch.position())
    } else {
        return;
    };

    let Some(screen_pos) = click_pos else { return };

    // Skip world click if any UI button is being interacted with
    for interaction in &ui_interactions {
        if *interaction != Interaction::None {
            return;
        }
    }

    // On touch devices, also skip if tap is in the HUD area (top 60px)
    if screen_pos.y < 60.0 {
        return;
    }

    let Ok((camera, cam_transform)) = camera_query.get_single() else { return };
    let Ok(ray) = camera.viewport_to_world(cam_transform, screen_pos) else { return };
    let Some(distance) = ray.intersect_plane(Vec3::ZERO, InfinitePlane3d::new(Vec3::Y)) else {
        return;
    };
    let world_pos = ray.get_point(distance);

    // Check hero first (tap to select hero) — skip if already selected so
    // that clicking near the hero commands a short move instead of re-selecting.
    if !matches!(*selection, Selection::Hero) {
        for hero_tf in &hero_q {
            let dist = world_pos.distance(hero_tf.translation);
            if dist < 2.0 {
                *selection = Selection::Hero;
                return;
            }
        }
    }

    // Check towers (they sit on top of build spots)
    for (tower_entity, tower_transform) in &towers {
        let dist = world_pos.distance(tower_transform.translation);
        if dist < 1.5 {
            *selection = Selection::Tower(tower_entity);
            return;
        }
    }

    // Then check build spots
    for (spot_entity, spot, spot_transform) in &spots {
        if spot.occupied {
            continue;
        }
        let dist = world_pos.distance(spot_transform.translation);
        if dist < 1.2 {
            *selection = Selection::BuildSpot(spot_entity);
            return;
        }
    }

    // Clicked empty ground
    if matches!(*selection, Selection::Hero) {
        // Hero is selected — move hero there, deselect immediately
        hero_move_cmd.0 = Some(world_pos);
        *selection = Selection::None;
    } else {
        // Deselect
        *selection = Selection::None;
    }
}
