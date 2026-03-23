use bevy::prelude::*;
use crate::components::*;
use crate::data::*;
use crate::resources::*;

/// Minimum pause between pulses (seconds).
const PULSE_MIN_PAUSE: f32 = 2.0;
/// Maximum wait between pulses — starts next pulse even if enemies remain.
const PULSE_MAX_WAIT: f32 = 8.0;
/// Start next pulse when active enemies drop to this threshold.
const PULSE_ENEMY_THRESHOLD: u32 = 2;

/// Heal hero to full HP at the start of each wave.
pub fn heal_hero_on_wave_start(
    mut hero_q: Query<&mut Health, (With<Hero>, Without<HeroRespawnTimer>)>,
    _wave: Res<WaveState>,
    mut last_wave: Local<u32>,
    game: Res<GameData>,
) {
    // Detect wave number change (wave just started)
    if game.wave_number != *last_wave && game.wave_number > 0 {
        *last_wave = game.wave_number;
        for mut health in &mut hero_q {
            health.current = health.max;
        }
    }
}

/// Handles wave lifecycle: Idle → Spawning → PulsePause → Spawning → ... → Active → Idle.
pub fn wave_spawner(
    mut commands: Commands,
    mut wave: ResMut<WaveState>,
    mut game: ResMut<GameData>,
    asset_server: Res<AssetServer>,
    time: Res<Time>,
    mut wave_btn: ResMut<WaveButtonPressed>,
    level_path: Res<crate::resources::LevelPath>,
    current_level: Res<crate::resources::CurrentLevel>,
) {
    // Consume the button press
    let btn_pressed = wave_btn.0;
    wave_btn.0 = false;

    match wave.phase {
        WavePhase::Idle => {
            if !btn_pressed {
                return;
            }
            if game.wave_number >= game.max_waves {
                return;
            }

            game.wave_number += 1;
            start_wave(&mut wave, &game, current_level.0);
        }

        WavePhase::Spawning => {
            let dt = time.delta_secs();
            wave.spawn_timer += dt;
            wave.wave_elapsed += dt;

            let wave_num = game.wave_number as f32;
            let config = level_start_config(current_level.0);
            let hp_mult = 1.0 + config.wave_hp_scale * (wave_num - 1.0);
            let speed_mult = 1.0 + config.wave_speed_scale * (wave_num - 1.0);
            let current_pulse = wave.current_pulse;

            // Try to spawn from the current group
            if let Some(group) = wave.groups.get(wave.current_group) {
                // If this group belongs to a future pulse, pause between pulses
                if group.pulse > current_pulse {
                    wave.phase = WavePhase::PulsePause;
                    wave.pulse_pause_timer = 0.0;
                    return;
                }

                let delay = group.delay;
                let interval = group.interval;
                let count = group.count;
                let enemy_type = group.enemy_type;

                // Wait for delay (relative to pulse start)
                if wave.wave_elapsed < delay {
                    return;
                }

                if wave.spawned_in_group < count && wave.spawn_timer >= interval {
                    wave.spawn_timer = 0.0;
                    wave.spawned_in_group += 1;
                    wave.active_enemies += 1;

                    spawn_enemy(
                        &mut commands,
                        &asset_server,
                        enemy_type,
                        hp_mult,
                        speed_mult,
                        &level_path,
                        wave.active_enemies,
                    );

                    if wave.spawned_in_group >= count {
                        wave.current_group += 1;
                        wave.spawned_in_group = 0;
                        wave.spawn_timer = 0.0;
                    }
                }
            } else {
                // All groups spawned
                wave.phase = WavePhase::Active;
            }
        }

        WavePhase::PulsePause => {
            let dt = time.delta_secs();
            wave.pulse_pause_timer += dt;

            let enemies_cleared = wave.active_enemies <= PULSE_ENEMY_THRESHOLD;
            let min_pause_met = wave.pulse_pause_timer >= PULSE_MIN_PAUSE;
            let timeout = wave.pulse_pause_timer >= PULSE_MAX_WAIT;

            if (enemies_cleared && min_pause_met) || timeout {
                wave.current_pulse += 1;
                wave.wave_elapsed = 0.0;
                wave.spawn_timer = 0.0;
                wave.phase = WavePhase::Spawning;
            }
        }

        WavePhase::Active => {
            if wave.active_enemies == 0 {
                wave.phase = WavePhase::Idle;
            }

            // Call Early: tap button during active wave for bonus gold
            if btn_pressed
                && game.wave_number < game.max_waves
            {
                let waves = level_waves(current_level.0);
                let wave_idx = (game.wave_number as usize).min(waves.len() - 1);
                let bonus = waves[wave_idx].early_call_bonus;
                game.gold += bonus;

                game.wave_number += 1;
                start_wave(&mut wave, &game, current_level.0);

                info!("Called early! +{}g bonus", bonus);
            }
        }
    }
}

/// Initialize wave state for a new wave.
fn start_wave(wave: &mut WaveState, game: &GameData, level: u32) {
    let waves = level_waves(level);
    let wave_idx = (game.wave_number as usize - 1).min(waves.len() - 1);
    let wave_def = &waves[wave_idx];

    wave.groups = wave_def
        .groups
        .iter()
        .map(|g| crate::resources::SpawnGroup {
            enemy_type: g.enemy_type,
            count: g.count,
            interval: g.interval,
            delay: g.delay,
            pulse: g.pulse,
        })
        .collect();

    wave.phase = WavePhase::Spawning;
    wave.spawn_timer = 0.0;
    wave.wave_elapsed = 0.0;
    wave.current_group = 0;
    wave.spawned_in_group = 0;
    wave.current_pulse = 0;
    wave.pulse_pause_timer = 0.0;
}

fn spawn_enemy(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    enemy_type: EnemyType,
    hp_mult: f32,
    speed_mult: f32,
    level_path: &crate::resources::LevelPath,
    spawn_index: u32,
) {
    let stats = enemy_stats(enemy_type);
    let spawn_pos = level_path.0[0];
    let scaled_speed = stats.speed * speed_mult;
    let y_offset = if stats.is_flying { 2.0 } else { 0.0 };

    let scene = asset_server.load(format!("{}#Scene0", stats.model_path));

    let mut transform = Transform::from_translation(spawn_pos + Vec3::Y * y_offset)
        .with_scale(Vec3::splat(stats.model_scale));
    if stats.rotation_y != 0.0 {
        transform.rotate_y(stats.rotation_y);
    }

    let mut entity_commands = commands.spawn((
        SceneRoot(scene),
        transform,
        Enemy,
        EnemyTypeId(enemy_type),
        Health {
            current: stats.hp * hp_mult,
            max: stats.hp * hp_mult,
        },
        PathFollower {
            segment: 0,
            progress: 0.0,
            speed: scaled_speed,
            base_speed: scaled_speed,
            // Stagger laterally so health bars don't merge into one long bar
            lateral_offset: {
                let hash = spawn_index.wrapping_mul(1103515245).wrapping_add(12345);
                let norm = ((hash >> 16) & 0x7FFF) as f32 / 32767.0; // 0..1
                (norm - 0.5) * 0.8 // -0.4..+0.4
            },
        },
        ModelScale(stats.model_scale),
        GoldReward(stats.gold_reward),
        Armor {
            physical: stats.armor,
            magic_resist: stats.magic_resist,
        },
        GameWorldEntity,
        EnemyNeedsAnimation,
    ));

    // Apply tint if this enemy type needs recoloring
    if let Some([r, g, b]) = stats.tint {
        entity_commands.insert(EnemyNeedsTint(Color::srgb(r, g, b)));
    }

    // Flying enemies can't be blocked by golems
    if stats.is_flying {
        entity_commands.insert(Flying);
    }

    // Healer enemies get an aura
    if stats.is_healer {
        entity_commands.insert(HealerAura {
            radius: 4.0,
            heal_per_second: 5.0,
        });
    }
}
