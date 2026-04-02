use bevy::prelude::*;
use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use crate::components::*;
use crate::resources::*;

/// Marker for the debug overlay UI root.
#[derive(Component)]
pub struct DebugOverlay;

/// Marker for the debug stats text.
#[derive(Component)]
pub struct DebugStatsText;

/// Resource tracking whether the debug overlay is visible.
#[derive(Resource)]
pub struct DebugState {
    pub show_overlay: bool,
}

impl Default for DebugState {
    fn default() -> Self {
        Self { show_overlay: false }
    }
}

// Admin panel button markers
#[derive(Component)]
pub struct DebugToggleButton;
#[derive(Component)]
pub struct DebugGoldButton;
#[derive(Component)]
pub struct DebugLivesButton;
#[derive(Component)]
pub struct DebugKillButton;
#[derive(Component)]
pub struct DebugSkipButton;
#[derive(Component)]
pub struct DebugHealHeroButton;
#[derive(Component)]
pub struct DebugNextLevelButton;

/// Admin hotkeys for testing (desktop).
pub fn debug_hotkeys(
    keys: Res<ButtonInput<KeyCode>>,
    mut game: ResMut<GameData>,
    mut wave: ResMut<WaveState>,
    mut enemies: Query<&mut Health, With<Enemy>>,
    mut debug_state: ResMut<DebugState>,
) {
    if keys.just_pressed(KeyCode::KeyG) {
        game.gold += 1000;
        info!("DEBUG: +1000 gold (total: {})", game.gold);
    }
    if keys.just_pressed(KeyCode::KeyL) {
        game.lives += 10;
        info!("DEBUG: +10 lives (total: {})", game.lives);
    }
    if keys.just_pressed(KeyCode::KeyK) {
        let mut count = 0;
        for mut health in &mut enemies {
            health.current = 0.0;
            count += 1;
        }
        info!("DEBUG: killed {} enemies", count);
    }
    if keys.just_pressed(KeyCode::KeyN) {
        for mut health in &mut enemies {
            health.current = 0.0;
        }
        // Mark current/next wave as completed
        if game.wave_number < game.max_waves {
            game.wave_number += 1;
        }
        wave.phase = WavePhase::Idle;
        wave.active_enemies = 0;
        info!("DEBUG: wave skipped (now wave {})", game.wave_number);
    }
    if keys.just_pressed(KeyCode::F1) {
        debug_state.show_overlay = !debug_state.show_overlay;
    }
}

/// Spawns or despawns the debug overlay based on debug_state.
pub fn manage_debug_overlay(
    mut commands: Commands,
    debug_state: Res<DebugState>,
    overlay: Query<Entity, With<DebugOverlay>>,
) {
    if !debug_state.is_changed() {
        return;
    }

    // Remove old overlay
    for entity in &overlay {
        commands.entity(entity).despawn_recursive();
    }

    if debug_state.show_overlay {
        commands
            .spawn((
                DebugOverlay,
                Node {
                    position_type: PositionType::Absolute,
                    right: Val::Px(16.0),
                    bottom: Val::Px(80.0),
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(6.0),
                    padding: UiRect::all(Val::Px(10.0)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.85)),
                BorderRadius::all(Val::Px(8.0)),
                GlobalZIndex(20),
            ))
            .with_children(|parent| {
                // Stats text
                parent.spawn((
                    Text::new("Debug"),
                    TextFont { font_size: 14.0, ..default() },
                    TextColor(Color::srgb(0.0, 1.0, 0.0)),
                    DebugStatsText,
                ));

                // Admin buttons
                spawn_debug_button(parent, "+1000 Gold", Color::srgb(1.0, 0.85, 0.0), DebugGoldButton);
                spawn_debug_button(parent, "+10 Lives", Color::srgb(1.0, 0.4, 0.4), DebugLivesButton);
                spawn_debug_button(parent, "Kill All", Color::srgb(1.0, 0.2, 0.2), DebugKillButton);
                spawn_debug_button(parent, "Skip Wave", Color::srgb(0.4, 0.8, 1.0), DebugSkipButton);
                spawn_debug_button(parent, "Heal Hero", Color::srgb(0.4, 1.0, 0.4), DebugHealHeroButton);
                spawn_debug_button(parent, "Next Level", Color::srgb(0.8, 0.6, 1.0), DebugNextLevelButton);
            });
    }
}

fn spawn_debug_button<M: Component>(parent: &mut ChildBuilder, label: &str, text_color: Color, marker: M) {
    parent
        .spawn((
            Button,
            marker,
            Node {
                width: Val::Px(120.0),
                height: Val::Px(32.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.2, 0.2, 0.2, 0.9)),
            BorderRadius::all(Val::Px(4.0)),
        ))
        .with_children(|btn| {
            btn.spawn((
                Text::new(label),
                TextFont { font_size: 14.0, ..default() },
                TextColor(text_color),
            ));
        });
}

/// Handles debug button interactions.
pub fn handle_debug_buttons(
    mut commands: Commands,
    toggle_q: Query<&Interaction, (Changed<Interaction>, With<DebugToggleButton>)>,
    gold_q: Query<&Interaction, (Changed<Interaction>, With<DebugGoldButton>)>,
    lives_q: Query<&Interaction, (Changed<Interaction>, With<DebugLivesButton>)>,
    kill_q: Query<&Interaction, (Changed<Interaction>, With<DebugKillButton>)>,
    skip_q: Query<&Interaction, (Changed<Interaction>, With<DebugSkipButton>)>,
    heal_q: Query<&Interaction, (Changed<Interaction>, With<DebugHealHeroButton>)>,
    next_level_q: Query<&Interaction, (Changed<Interaction>, With<DebugNextLevelButton>)>,
    mut debug_state: ResMut<DebugState>,
    mut game: ResMut<GameData>,
    mut wave: ResMut<WaveState>,
    mut enemies: Query<&mut Health, (With<Enemy>, Without<Hero>)>,
    mut hero_q: Query<(Entity, &mut Health), (With<Hero>, Without<Enemy>)>,
    mut current_level: ResMut<CurrentLevel>,
    mut needs_fresh: ResMut<NeedsFreshSetup>,
    mut next_state: ResMut<NextState<crate::states::AppState>>,
) {
    for interaction in &toggle_q {
        if *interaction == Interaction::Pressed {
            debug_state.show_overlay = !debug_state.show_overlay;
        }
    }
    for interaction in &gold_q {
        if *interaction == Interaction::Pressed {
            game.gold += 1000;
            info!("DEBUG: +1000 gold");
        }
    }
    for interaction in &lives_q {
        if *interaction == Interaction::Pressed {
            game.lives += 10;
            info!("DEBUG: +10 lives");
        }
    }
    for interaction in &kill_q {
        if *interaction == Interaction::Pressed {
            for mut health in &mut enemies {
                health.current = 0.0;
            }
            info!("DEBUG: killed all enemies");
        }
    }
    for interaction in &skip_q {
        if *interaction == Interaction::Pressed {
            for mut health in &mut enemies {
                health.current = 0.0;
            }
            // Advance wave number even if wave hasn't started yet
            if game.wave_number < game.max_waves {
                game.wave_number += 1;
            }
            wave.phase = WavePhase::Idle;
            wave.active_enemies = 0;
            info!("DEBUG: wave skipped (now wave {})", game.wave_number);
        }
    }
    for interaction in &heal_q {
        if *interaction == Interaction::Pressed {
            for (entity, mut health) in &mut hero_q {
                health.current = health.max;
                // Also revive if dead
                commands.entity(entity).remove::<HeroRespawnTimer>();
                commands.entity(entity).insert(Visibility::Visible);
            }
            info!("DEBUG: hero healed");
        }
    }
    for interaction in &next_level_q {
        if *interaction == Interaction::Pressed {
            let next = if current_level.0 >= crate::data::MAX_LEVELS { 1 } else { current_level.0 + 1 };
            current_level.0 = next;
            needs_fresh.0 = true;
            *wave = WaveState::default();
            // Go through WaitingForWindow — it checks NeedsFreshSetup to re-enter Playing
            next_state.set(crate::states::AppState::WaitingForWindow);
            info!("DEBUG: switching to level {}", next);
        }
    }
}

/// Updates the debug stats text.
pub fn update_debug_overlay(
    debug_state: Res<DebugState>,
    game: Res<GameData>,
    wave: Res<WaveState>,
    enemies: Query<Entity, With<Enemy>>,
    towers: Query<Entity, With<Tower>>,
    golems: Query<Entity, With<Golem>>,
    build_spots: Query<(&Transform, &BuildSpot)>,
    hero_q: Query<&Transform, With<Hero>>,
    diagnostics: Res<DiagnosticsStore>,
    mut text_q: Query<&mut Text, With<DebugStatsText>>,
    current_level: Res<CurrentLevel>,
    camera_q: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    windows: Query<&Window>,
) {
    if !debug_state.show_overlay {
        return;
    }

    let fps = diagnostics
        .get(&FrameTimeDiagnosticsPlugin::FPS)
        .and_then(|d| d.smoothed())
        .unwrap_or(0.0);

    let enemy_count = enemies.iter().count();
    let tower_count = towers.iter().count();
    let golem_count = golems.iter().count();
    let phase = match wave.phase {
        WavePhase::Idle => "Idle",
        WavePhase::Spawning => "Spawning",
        WavePhase::PulsePause => "Pause",
        WavePhase::Active => "Active",
    };

    // Raycast cursor to ground plane (Y=0) for world coordinates
    let mut cursor_world = String::new();
    if let Ok((camera, cam_tf)) = camera_q.get_single() {
        if let Ok(window) = windows.get_single() {
            if let Some(cursor_pos) = window.cursor_position() {
                if let Ok(ray) = camera.viewport_to_world(cam_tf, cursor_pos) {
                    // Intersect with Y=0 plane
                    if ray.direction.y.abs() > 0.001 {
                        let t = -ray.origin.y / ray.direction.y;
                        if t > 0.0 {
                            let hit = ray.origin + ray.direction * t;
                            cursor_world = format!("\nCursor: ({:.1}, {:.1})", hit.x, hit.z);
                        }
                    }
                }
            }
        }
    }

    // Hero position
    let hero_info = if let Ok(hero_tf) = hero_q.get_single() {
        format!("\nHero: ({:.1}, {:.1})", hero_tf.translation.x, hero_tf.translation.z)
    } else {
        String::new()
    };

    // Build spot positions
    let mut spots_info = String::new();
    let mut spots: Vec<_> = build_spots.iter().collect();
    spots.sort_by_key(|(_, s)| s.id);
    for (tf, spot) in &spots {
        spots_info.push_str(&format!(
            "\n  #{}: ({:.1}, {:.1}){}",
            spot.id, tf.translation.x, tf.translation.z,
            if spot.occupied { " [T]" } else { "" }
        ));
    }
    if !spots_info.is_empty() {
        spots_info = format!("\nSpots:{}", spots_info);
    }

    let info = format!(
        "FPS: {:.0} | Level: {}\n\
         Gold: {} | Lives: {}\n\
         Wave: {}/{} [{}]\n\
         E: {} | T: {} | G: {}{}{}{}",
        fps, current_level.0,
        game.gold, game.lives,
        (game.wave_number + 1).min(game.max_waves), game.max_waves, phase,
        enemy_count, tower_count, golem_count,
        cursor_world, hero_info, spots_info,
    );

    for mut text in &mut text_q {
        text.0 = info.clone();
    }
}
