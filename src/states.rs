use bevy::prelude::*;

#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum AppState {
    ModelShowcase,
    AnimTest,
    /// Wait for the native render surface before spawning 3D entities.
    /// On Android, the native window is null until Event::Resumed.
    #[default]
    WaitingForWindow,
    MainMenu,
    LevelSelect,
    HeroSelect,
    Logbook,
    UpgradeShop,
    ModelDebug,
    Credits,
    Playing,
    Paused,
    GameOver,
}

/// Tracks whether the player won or lost (set before entering GameOver state).
#[derive(Resource, Default)]
pub struct GameOutcome {
    pub victory: bool,
    pub stars: u8, // 1-3 based on lives remaining
}
