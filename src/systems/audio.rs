use bevy::prelude::*;
use bevy::window::WindowFocused;
use crate::components::*;
use crate::resources::*;

/// Preloaded audio handles for quick playback.
#[derive(Resource)]
pub struct AudioAssets {
    pub tower_build: Handle<AudioSource>,
    pub tower_sell: Handle<AudioSource>,
    pub tower_upgrade: Handle<AudioSource>,
    pub enemy_death: Handle<AudioSource>,
    pub enemy_leak: Handle<AudioSource>,
    pub wave_start: Handle<AudioSource>,
    pub wave_complete: Handle<AudioSource>,
    pub tower_lightning: Handle<AudioSource>,
    pub tower_fire: Handle<AudioSource>,
    pub tower_ice: Handle<AudioSource>,
    pub tower_earth: Handle<AudioSource>,
    pub victory: Handle<AudioSource>,
    pub defeat: Handle<AudioSource>,
    pub battle_music: Handle<AudioSource>,
    /// Whether all audio assets have finished loading.
    pub all_loaded: bool,
}

impl AudioAssets {
    /// Returns true once all audio handles are loaded.
    fn check_loaded(&self, sources: &Assets<AudioSource>) -> bool {
        sources.get(&self.tower_build).is_some()
            && sources.get(&self.enemy_death).is_some()
            && sources.get(&self.tower_lightning).is_some()
            && sources.get(&self.wave_start).is_some()
            && sources.get(&self.battle_music).is_some()
    }

}

/// Marker for the background music entity.
#[derive(Component)]
pub struct BgMusic;

pub fn load_audio(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(AudioAssets {
        tower_build: asset_server.load("audio/sfx/tower-build.wav"),
        tower_sell: asset_server.load("audio/sfx/tower-sell.wav"),
        tower_upgrade: asset_server.load("audio/sfx/tower-upgrade.wav"),
        enemy_death: asset_server.load("audio/sfx/enemy-death.wav"),
        enemy_leak: asset_server.load("audio/sfx/enemy-leak.wav"),
        wave_start: asset_server.load("audio/sfx/wave-start.wav"),
        wave_complete: asset_server.load("audio/sfx/wave-complete.wav"),
        tower_lightning: asset_server.load("audio/sfx/tower-lightning-attack.wav"),
        tower_fire: asset_server.load("audio/sfx/tower-fire-attack.wav"),
        tower_ice: asset_server.load("audio/sfx/tower-ice-attack.wav"),
        tower_earth: asset_server.load("audio/sfx/tower-earth-attack.wav"),
        victory: asset_server.load("audio/sfx/victory.wav"),
        defeat: asset_server.load("audio/sfx/defeat.wav"),
        battle_music: asset_server.load("audio/music/battle-theme.ogg"),
        all_loaded: false,
    });
}

/// Polls until all audio assets are loaded. Prevents playback before ready.
pub fn check_audio_loaded(
    audio_assets: Option<ResMut<AudioAssets>>,
    sources: Res<Assets<AudioSource>>,
) {
    let Some(mut audio) = audio_assets else { return };
    if !audio.all_loaded && audio.check_loaded(&sources) {
        audio.all_loaded = true;
        info!("All audio assets loaded");
    }
}

/// Helper: only spawn an AudioPlayer if assets are loaded.
fn try_play(commands: &mut Commands, handle: &Handle<AudioSource>, audio: &AudioAssets) {
    if !audio.all_loaded { return; }
    commands.spawn((
        AudioPlayer(handle.clone()),
        PlaybackSettings::DESPAWN,
    ));
}

fn try_play_volume(commands: &mut Commands, handle: &Handle<AudioSource>, audio: &AudioAssets, vol: f32) {
    if !audio.all_loaded { return; }
    commands.spawn((
        AudioPlayer(handle.clone()),
        PlaybackSettings {
            volume: bevy::audio::Volume::new(vol),
            ..PlaybackSettings::DESPAWN
        },
    ));
}

/// Starts battle music once all audio is loaded.
pub fn start_battle_music(
    mut commands: Commands,
    audio_assets: Option<Res<AudioAssets>>,
    music_q: Query<Entity, With<BgMusic>>,
) {
    let Some(audio) = audio_assets else { return };
    if !audio.all_loaded { return; }
    if !music_q.is_empty() { return; }

    commands.spawn((
        AudioPlayer(audio.battle_music.clone()),
        PlaybackSettings {
            mode: bevy::audio::PlaybackMode::Loop,
            volume: bevy::audio::Volume::new(0.3),
            ..default()
        },
        BgMusic,
    ));
}

/// Plays SFX when enemies die (triggered by DeathEffect spawning).
pub fn play_death_sfx(
    mut commands: Commands,
    new_deaths: Query<Entity, Added<DeathEffect>>,
    audio_assets: Option<Res<AudioAssets>>,
) {
    let Some(audio) = audio_assets else { return };
    if new_deaths.iter().next().is_some() {
        try_play(&mut commands, &audio.enemy_death, &audio);
    }
}

/// Plays SFX when towers fire (triggered by MuzzleFlash spawning).
/// Uses per-element sounds.
pub fn play_tower_attack_sfx(
    mut commands: Commands,
    new_flashes: Query<&MuzzleFlash, Added<MuzzleFlash>>,
    audio_assets: Option<Res<AudioAssets>>,
) {
    let Some(audio) = audio_assets else { return };
    for flash in &new_flashes {
        let handle = match flash.element {
            crate::components::Element::Lightning => &audio.tower_lightning,
            crate::components::Element::Fire => &audio.tower_fire,
            crate::components::Element::Ice => &audio.tower_ice,
            crate::components::Element::Earth => &audio.tower_earth,
        };
        try_play_volume(&mut commands, handle, &audio, 0.4);
        break; // Only play one SFX per frame to avoid overlap
    }
}

/// Pauses all audio when app loses focus (minimized), resumes on regain.
/// Uses both WindowFocused (desktop) and AppLifecycle (Android) events.
pub fn pause_audio_on_focus(
    mut focus_events: EventReader<WindowFocused>,
    mut lifecycle_events: EventReader<bevy::window::AppLifecycle>,
    music_q: Query<&AudioSink, With<BgMusic>>,
) {
    // Desktop: window focus/blur
    for event in focus_events.read() {
        for sink in &music_q {
            if event.focused {
                sink.play();
            } else {
                sink.pause();
            }
        }
    }

    // Android: app lifecycle suspend/resume
    for event in lifecycle_events.read() {
        use bevy::window::AppLifecycle;
        match event {
            AppLifecycle::Suspended | AppLifecycle::WillSuspend => {
                for sink in &music_q {
                    sink.pause();
                }
            }
            AppLifecycle::Running => {
                for sink in &music_q {
                    sink.play();
                }
            }
            _ => {}
        }
    }
}

/// Plays SFX on wave state transitions. Uses Local to track last phase.
pub fn play_wave_sfx(
    mut commands: Commands,
    wave: Res<WaveState>,
    audio_assets: Option<Res<AudioAssets>>,
    mut last_phase: Local<Option<u8>>,
) {
    let Some(audio) = audio_assets else { return };

    let current: u8 = match wave.phase {
        WavePhase::Idle => 0,
        WavePhase::Spawning | WavePhase::PulsePause => 1,
        WavePhase::Active => 2,
    };

    let prev = last_phase.unwrap_or(255);
    if prev == current {
        return;
    }

    let was_active = prev == 2;
    *last_phase = Some(current);

    match wave.phase {
        WavePhase::Spawning => {
            try_play_volume(&mut commands, &audio.wave_start, &audio, 0.5);
        }
        WavePhase::Idle if was_active => {
            try_play_volume(&mut commands, &audio.wave_complete, &audio, 0.5);
        }
        _ => {}
    }
}
