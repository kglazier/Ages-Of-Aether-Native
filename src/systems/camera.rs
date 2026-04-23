use bevy::prelude::*;
use bevy::input::mouse::{MouseScrollUnit, MouseWheel};

/// Tracks the point on the ground the camera is looking at.
/// WASD moves this point, camera follows at a fixed offset.
#[derive(Resource)]
pub struct CameraFocus {
    pub target: Vec3,
    pub distance: f32,
}

impl Default for CameraFocus {
    fn default() -> Self {
        Self {
            target: Vec3::ZERO,
            distance: 27.0,
        }
    }
}

/// Tracks active touch state for pan/pinch gestures.
#[derive(Resource, Default)]
pub struct TouchState {
    /// Previous frame's single-finger position (for panning)
    prev_touch: Option<Vec2>,
    /// Previous frame's pinch distance (for zooming)
    prev_pinch_distance: Option<f32>,
}

/// WASD/arrow keys + touch to pan, scroll/pinch to zoom.
/// Camera maintains 45-degree angle looking at focus point.
pub fn camera_control(
    mut camera_q: Query<&mut Transform, With<Camera3d>>,
    mut focus: ResMut<CameraFocus>,
    keys: Res<ButtonInput<KeyCode>>,
    mut scroll_events: EventReader<MouseWheel>,
    touches: Res<Touches>,
    time: Res<Time>,
    mut touch_state: Local<TouchState>,
    mut shake: ResMut<CameraShake>,
    intro: Res<CameraIntro>,
) {
    let Ok(mut transform) = camera_q.get_single_mut() else {
        return;
    };

    // During the level-start zoom intro, only the intro system drives focus.
    // We still write the camera transform below so the animation renders.
    let skip_input = intro.active;
    if skip_input {
        // Drain scroll events so they don't accumulate behind the intro.
        scroll_events.clear();
    }

    let speed = 15.0 * time.delta_secs();

    // --- Keyboard pan: move the focus point along screen-relative X/Z ---
    let mut pan = Vec3::ZERO;
    if !skip_input && (keys.pressed(KeyCode::KeyW) || keys.pressed(KeyCode::ArrowUp)) {
        pan.z -= 1.0;
    }
    if !skip_input && (keys.pressed(KeyCode::KeyS) || keys.pressed(KeyCode::ArrowDown)) {
        pan.z += 1.0;
    }
    if !skip_input && (keys.pressed(KeyCode::KeyA) || keys.pressed(KeyCode::ArrowLeft)) {
        pan.x -= 1.0;
    }
    if !skip_input && (keys.pressed(KeyCode::KeyD) || keys.pressed(KeyCode::ArrowRight)) {
        pan.x += 1.0;
    }
    if pan != Vec3::ZERO {
        focus.target += pan.normalize() * speed;
    }

    // --- Scroll wheel zoom (desktop) ---
    if !skip_input {
        for ev in scroll_events.read() {
            let zoom_delta = match ev.unit {
                MouseScrollUnit::Line => ev.y * 2.0,
                MouseScrollUnit::Pixel => ev.y * 0.05,
            };
            focus.distance = (focus.distance - zoom_delta).clamp(10.0, 45.0);
        }
    }

    // --- Touch input (mobile) ---
    let active_touches: Vec<&bevy::input::touch::Touch> = if skip_input {
        Vec::new()
    } else {
        touches.iter().collect()
    };

    match active_touches.len() {
        1 => {
            // Single finger drag → pan
            let pos = active_touches[0].position();
            if let Some(prev) = touch_state.prev_touch {
                let delta = pos - prev;
                // Scale touch delta to world units — negative because dragging
                // right should move the world left (camera focus moves left)
                let pan_scale = focus.distance * 0.003;
                focus.target.x -= delta.x * pan_scale;
                focus.target.z -= delta.y * pan_scale;
            }
            touch_state.prev_touch = Some(pos);
            touch_state.prev_pinch_distance = None;
        }
        2 => {
            // Two finger pinch → zoom
            let p1 = active_touches[0].position();
            let p2 = active_touches[1].position();
            let dist = p1.distance(p2);

            if let Some(prev_dist) = touch_state.prev_pinch_distance {
                let zoom_delta = (dist - prev_dist) * 0.05;
                focus.distance = (focus.distance - zoom_delta).clamp(10.0, 45.0);
            }
            touch_state.prev_pinch_distance = Some(dist);
            touch_state.prev_touch = None;
        }
        _ => {
            // No touches or 3+ fingers — reset state
            touch_state.prev_touch = None;
            touch_state.prev_pinch_distance = None;
        }
    }

    // --- Clamp focus to map boundaries ---
    // Tighten bounds as camera zooms out so the visible area stays on the map.
    // At min zoom (10) the focus can move freely; at max zoom (45) it's tight.
    let zoom_t = ((focus.distance - 10.0) / 35.0).clamp(0.0, 1.0); // 0 at close, 1 at far
    let half_x = 18.0 - zoom_t * 10.0;  // 18 close → 8 far
    let half_z = 9.0 - zoom_t * 5.0;    // 9 close → 4 far
    focus.target.x = focus.target.x.clamp(-half_x, half_x);
    focus.target.z = focus.target.z.clamp(-half_z, half_z);

    // --- Position camera at 45-degree angle above focus point ---
    let offset = Vec3::new(0.0, focus.distance * 0.7, focus.distance * 0.65);
    transform.translation = focus.target + offset;

    // Apply camera shake
    if shake.remaining > 0.0 {
        shake.remaining -= time.delta_secs();
        let decay = (shake.remaining / shake.duration).max(0.0);
        let t = (shake.duration - shake.remaining) * 30.0;
        let shake_x = (t * 1.3).sin() * shake.intensity * decay;
        let shake_y = (t * 1.7).cos() * shake.intensity * decay * 0.5;
        transform.translation.x += shake_x;
        transform.translation.y += shake_y;
    }

    transform.look_at(focus.target, Vec3::Y);
}

// ---------------------------------------------------------------------------
// Level-start zoom intro
// ---------------------------------------------------------------------------
//
// On entering Playing, the camera starts pulled back showing most of the map,
// then eases in over ~1.2s to the hero spawn area. Any input (tap, touch,
// keyboard, scroll) cancels the intro immediately.

#[derive(Resource, Default)]
pub struct CameraIntro {
    pub active: bool,
    pub elapsed: f32,
    pub duration: f32,
    pub from_target: Vec3,
    pub from_distance: f32,
    pub to_target: Vec3,
    pub to_distance: f32,
}

impl CameraIntro {
    pub fn start(&mut self, from_target: Vec3, from_distance: f32, to_target: Vec3, to_distance: f32) {
        self.active = true;
        self.elapsed = 0.0;
        self.duration = 1.2;
        self.from_target = from_target;
        self.from_distance = from_distance;
        self.to_target = to_target;
        self.to_distance = to_distance;
    }

    pub fn finish(&mut self, focus: &mut CameraFocus) {
        if self.active {
            focus.target = self.to_target;
            focus.distance = self.to_distance;
            self.active = false;
        }
    }
}

/// Seed the camera intro on level entry. Starts from a wide shot (distance 42)
/// centered over the map, eases to the default focus near the hero spawn.
pub fn start_level_intro(
    mut intro: ResMut<CameraIntro>,
    mut focus: ResMut<CameraFocus>,
    current_level: Res<crate::resources::CurrentLevel>,
) {
    let hero_spawn = crate::data::level_hero_spawn(current_level.0);
    // Start shot: centered over map, far back
    let wide_target = Vec3::new(0.0, 0.0, 0.0);
    let wide_distance = 42.0;
    // End shot: near hero spawn, default distance
    let close_target = Vec3::new(hero_spawn.x * 0.5, 0.0, hero_spawn.z * 0.5);
    let close_distance = 27.0;

    // Snap focus to wide so camera_control doesn't render a frame at the old state.
    focus.target = wide_target;
    focus.distance = wide_distance;
    intro.start(wide_target, wide_distance, close_target, close_distance);
}

/// Drives the zoom intro. Runs before `camera_control` and overrides focus.
pub fn tick_level_intro(
    time: Res<Time>,
    mut intro: ResMut<CameraIntro>,
    mut focus: ResMut<CameraFocus>,
    touches: Res<Touches>,
    keys: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
) {
    if !intro.active {
        return;
    }

    // Skip on any input.
    let any_input = touches.iter().next().is_some()
        || keys.get_pressed().next().is_some()
        || mouse.any_just_pressed([MouseButton::Left, MouseButton::Right]);
    if any_input {
        intro.finish(&mut focus);
        return;
    }

    intro.elapsed += time.delta_secs();
    let t = (intro.elapsed / intro.duration).clamp(0.0, 1.0);
    // easeOutCubic
    let e = 1.0 - (1.0 - t).powi(3);

    focus.target = intro.from_target.lerp(intro.to_target, e);
    focus.distance = intro.from_distance + (intro.to_distance - intro.from_distance) * e;

    if t >= 1.0 {
        intro.finish(&mut focus);
    }
}

/// Resource for camera shake — any system can write to trigger a shake.
#[derive(Resource, Default)]
pub struct CameraShake {
    pub intensity: f32,
    pub duration: f32,
    pub remaining: f32,
}

impl CameraShake {
    pub fn trigger(&mut self, intensity: f32, duration: f32) {
        // Only override if this shake is stronger than the current one
        if intensity > self.intensity * (self.remaining / self.duration.max(0.001)) {
            self.intensity = intensity;
            self.duration = duration;
            self.remaining = duration;
        }
    }
}
