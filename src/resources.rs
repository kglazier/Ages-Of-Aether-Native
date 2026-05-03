use bevy::prelude::*;
use crate::data::EnemyType;

#[derive(Resource, Clone, Copy, PartialEq, Eq, Debug)]
pub enum Difficulty {
    Easy,
    Normal,
    Hard,
}

impl Default for Difficulty {
    fn default() -> Self { Difficulty::Easy }
}

impl Difficulty {
    pub fn label(self) -> &'static str {
        match self {
            Difficulty::Easy => "Easy",
            Difficulty::Normal => "Normal",
            Difficulty::Hard => "Hard",
        }
    }
    /// Starting lives — same across all levels, varies by difficulty.
    pub fn starting_lives(self) -> u32 {
        match self { Difficulty::Easy => 20, Difficulty::Normal => 15, Difficulty::Hard => 10 }
    }
    pub fn gold_mult(self) -> f32 {
        match self { Difficulty::Easy => 1.25, Difficulty::Normal => 1.0, Difficulty::Hard => 0.85 }
    }
    pub fn enemy_hp_mult(self) -> f32 {
        match self { Difficulty::Easy => 0.85, Difficulty::Normal => 1.0, Difficulty::Hard => 1.20 }
    }
    pub fn enemy_speed_mult(self) -> f32 {
        match self { Difficulty::Easy => 0.95, Difficulty::Normal => 1.0, Difficulty::Hard => 1.10 }
    }
}

#[derive(Resource)]
pub struct GameData {
    pub gold: u32,
    pub lives: u32,
    pub max_lives: u32,
    pub wave_number: u32,
    pub max_waves: u32,
}

impl Default for GameData {
    fn default() -> Self {
        Self {
            gold: 220,
            lives: 20,
            max_lives: 20,
            wave_number: 0,
            max_waves: 10,
        }
    }
}

/// Tracks wave spawning progress.
#[derive(Resource, Default)]
pub struct WaveState {
    pub phase: WavePhase,
    pub spawn_timer: f32,
    pub wave_elapsed: f32, // total time since pulse started (for group delays)
    pub groups: Vec<SpawnGroup>,
    pub current_group: usize,
    pub spawned_in_group: u32,
    pub active_enemies: u32,
    pub current_pulse: u32,
    pub pulse_pause_timer: f32,
}

#[derive(Default, PartialEq)]
pub enum WavePhase {
    #[default]
    Idle,
    Spawning,
    PulsePause,
    Active,
}

pub struct SpawnGroup {
    pub enemy_type: EnemyType,
    pub count: u32,
    pub interval: f32,
    pub delay: f32, // seconds before this group starts (relative to pulse start)
    pub pulse: u32, // which pulse this group belongs to
}

/// Set by the UI "Send Wave" button; consumed by wave_spawner each frame.
#[derive(Resource, Default)]
pub struct WaveButtonPressed(pub bool);

/// Auto-wave state: when enabled, automatically starts the next wave after a countdown.
#[derive(Resource, Default)]
pub struct AutoWave {
    pub enabled: bool,
    /// Countdown timer (seconds) before auto-starting the next wave.
    /// Only ticks when phase is Idle and enabled is true.
    pub countdown: f32,
}

/// Delay before auto-wave starts the next wave (seconds).
pub const AUTO_WAVE_DELAY: f32 = 5.0;

/// Game speed multiplier (1.0, 2.0, or 3.0).
#[derive(Resource)]
pub struct GameSpeed(pub f32);

impl Default for GameSpeed {
    fn default() -> Self {
        Self(1.0)
    }
}

/// Guards OnEnter(Playing) so cleanup+setup only runs on fresh start / restart,
/// not when resuming from pause.
#[derive(Resource)]
pub struct NeedsFreshSetup(pub bool);

impl Default for NeedsFreshSetup {
    fn default() -> Self {
        Self(false)
    }
}

/// Set by input system when player taps ground — consumed by hero movement.
#[derive(Resource, Default)]
pub struct HeroMoveCommand(pub Option<Vec3>);

/// Which hero type is currently active. Only matters when NoHeroSelected is false.
#[derive(Resource)]
pub struct ActiveHeroType(pub crate::data::HeroType);

impl Default for ActiveHeroType {
    fn default() -> Self {
        Self(crate::data::HeroType::IceHulk) // first unlock; placeholder until player picks
    }
}

/// True = "Towers Only" mode (no hero spawns, no hero HUD).
/// Default true so a fresh save starts with no hero — players earn the first one
/// by beating level 1.
#[derive(Resource)]
pub struct NoHeroSelected(pub bool);

impl Default for NoHeroSelected {
    fn default() -> Self { Self(true) }
}

/// Volume settings for music and sound effects (0.0–1.0).
#[derive(Resource)]
pub struct VolumeSettings {
    pub music: f32,
    pub sfx: f32,
}

impl Default for VolumeSettings {
    fn default() -> Self {
        Self { music: 0.5, sfx: 0.5 }
    }
}

/// Global player ability cooldowns (Meteor & Reinforcements).
#[derive(Resource)]
pub struct PlayerAbilities {
    pub meteor_cooldown: f32,
    pub reinforcement_cooldown: f32,
}

impl Default for PlayerAbilities {
    fn default() -> Self {
        Self { meteor_cooldown: 0.0, reinforcement_cooldown: 0.0 }
    }
}

/// When set, the next ground tap should target this ability instead of moving the hero.
#[derive(Resource, Default)]
pub struct PlayerAbilityTargeting(pub Option<crate::data::PlayerAbilityType>);

/// Admin/debug unlock flags (runtime only, not persisted).
#[derive(Resource, Default)]
pub struct AdminUnlocks {
    pub all_levels: bool,
    pub all_heroes: bool,
}

/// Which level is currently being played (1-indexed).
#[derive(Resource)]
pub struct CurrentLevel(pub u32);

impl Default for CurrentLevel {
    fn default() -> Self {
        Self(1)
    }
}

/// Cached path waypoints for the current level — avoids re-allocating every frame.
#[derive(Resource, Default)]
pub struct LevelPath(pub Vec<Vec3>);

/// What the player has currently selected in the UI.
#[derive(Resource, Default)]
pub enum Selection {
    #[default]
    None,
    /// Player clicked an empty build spot — show tower build menu.
    BuildSpot(Entity),
    /// Player clicked an existing tower — show upgrade/sell panel.
    Tower(Entity),
    /// Player is setting a rally point for an earth tower's golems.
    SettingRallyPoint(Entity),
    /// Player clicked on the hero.
    Hero,
}
