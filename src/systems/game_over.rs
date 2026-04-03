use bevy::prelude::*;
use crate::resources::*;
use crate::states::*;
use super::audio::AudioAssets;

/// Check for victory (all waves complete, no enemies left) or defeat (0 lives).
pub fn check_game_over(
    mut commands: Commands,
    game: Res<GameData>,
    wave: Res<WaveState>,
    mut next_state: ResMut<NextState<AppState>>,
    mut outcome: ResMut<GameOutcome>,
    audio_assets: Option<Res<AudioAssets>>,
    vol_settings: Res<VolumeSettings>,
) {
    // Defeat: ran out of lives
    if game.lives == 0 {
        outcome.victory = false;
        outcome.stars = 0;
        if let Some(audio) = &audio_assets {
            if audio.all_loaded {
                commands.spawn((
                    AudioPlayer(audio.defeat.clone()),
                    PlaybackSettings {
                        volume: bevy::audio::Volume::new(vol_settings.sfx),
                        ..PlaybackSettings::DESPAWN
                    },
                ));
            }
        }
        next_state.set(AppState::GameOver);
        return;
    }

    // Victory: all waves done and no enemies remaining
    if game.wave_number >= game.max_waves && wave.phase == WavePhase::Idle {
        outcome.victory = true;
        // Star rating based on lives remaining
        let pct = if game.max_lives > 0 { game.lives as f32 / game.max_lives as f32 } else { 0.0 };
        outcome.stars = if pct >= 0.9 { 3 } else if pct >= 0.5 { 2 } else { 1 };
        if let Some(audio) = &audio_assets {
            if audio.all_loaded {
                commands.spawn((
                    AudioPlayer(audio.victory.clone()),
                    PlaybackSettings {
                        volume: bevy::audio::Volume::new(vol_settings.sfx),
                        ..PlaybackSettings::DESPAWN
                    },
                ));
            }
        }
        next_state.set(AppState::GameOver);
    }
}
