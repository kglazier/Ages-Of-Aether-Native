use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use crate::data::{UpgradeKind, upgrade_index};

/// Persistent player progress saved to disk.
#[derive(Resource, Serialize, Deserialize, Clone)]
pub struct SaveData {
    /// Best star rating per level (0 = not beaten). Index 0 = level 1.
    pub level_stars: Vec<u8>,
    /// Meta-currency earned from completing levels.
    pub aether_gems: u32,
    /// Upgrade levels for each meta-progression upgrade (0 = not purchased). Index maps to UpgradeKind ordinal.
    pub upgrade_levels: Vec<u8>,
    /// Whether the first-play tutorial has been completed (or skipped).
    #[serde(default)]
    pub tutorial_completed: bool,
}

impl Default for SaveData {
    fn default() -> Self {
        Self {
            level_stars: vec![0; 10],
            aether_gems: 0,
            upgrade_levels: vec![0; 5],
            tutorial_completed: false,
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn save_file_path() -> String {
    #[cfg(target_os = "android")]
    {
        "/data/data/com.agesofaether/files/save.json".to_string()
    }
    #[cfg(not(target_os = "android"))]
    {
        "save.json".to_string()
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn read_save_string() -> Option<String> {
    std::fs::read_to_string(&save_file_path()).ok()
}

#[cfg(not(target_arch = "wasm32"))]
fn write_save_string(json: &str) {
    if let Err(e) = std::fs::write(&save_file_path(), json) {
        warn!("Failed to write save file: {}", e);
    }
}

#[cfg(target_arch = "wasm32")]
const WEB_SAVE_KEY: &str = "ages-of-aether/save";

#[cfg(target_arch = "wasm32")]
fn read_save_string() -> Option<String> {
    let storage = web_sys::window()?.local_storage().ok()??;
    storage.get_item(WEB_SAVE_KEY).ok()?
}

#[cfg(target_arch = "wasm32")]
fn write_save_string(json: &str) {
    let Some(window) = web_sys::window() else { return };
    let Ok(Some(storage)) = window.local_storage() else { return };
    if let Err(e) = storage.set_item(WEB_SAVE_KEY, json) {
        warn!("Failed to write save to localStorage: {:?}", e);
    }
}

/// Loads save data on startup. Inserts default if missing or corrupt.
pub fn load_save_on_startup(mut commands: Commands) {
    let mut data = read_save_string()
        .and_then(|s| serde_json::from_str::<SaveData>(&s).ok())
        .unwrap_or_default();
    data.level_stars.resize(10, 0);
    data.upgrade_levels.resize(5, 0);
    commands.insert_resource(data);
}

impl SaveData {
    /// Get the level of a specific upgrade (0 if not purchased).
    pub fn upgrade_level(&self, kind: UpgradeKind) -> u8 {
        let idx = upgrade_index(kind);
        if idx < self.upgrade_levels.len() { self.upgrade_levels[idx] } else { 0 }
    }

    /// Tower damage multiplier from ElementalFury upgrade.
    pub fn tower_damage_mult(&self) -> f32 {
        1.0 + 0.05 * self.upgrade_level(UpgradeKind::ElementalFury) as f32
    }

    /// Tower range multiplier from FarSight upgrade.
    pub fn tower_range_mult(&self) -> f32 {
        1.0 + 0.05 * self.upgrade_level(UpgradeKind::FarSight) as f32
    }

    /// Sell refund rate including SalvageExpert bonus.
    pub fn sell_refund_rate(&self) -> f32 {
        crate::data::SELL_REFUND_RATE + 0.05 * self.upgrade_level(UpgradeKind::SalvageExpert) as f32
    }

    /// Hero ability cooldown multiplier from TacticalMastery upgrade.
    pub fn cooldown_mult(&self) -> f32 {
        1.0 - 0.05 * self.upgrade_level(UpgradeKind::TacticalMastery) as f32
    }
}

/// Writes the save data. Call after updating any persistent field.
pub fn write_save(save: &SaveData) {
    if let Ok(json) = serde_json::to_string_pretty(save) {
        write_save_string(&json);
    }
}

/// Saves progress when entering GameOver state (only on victory).
pub fn save_on_level_complete(
    outcome: Res<crate::states::GameOutcome>,
    current_level: Res<crate::resources::CurrentLevel>,
    mut save: ResMut<SaveData>,
    mut newly_unlocked: ResMut<crate::resources::NewlyUnlockedHero>,
    mut active_hero: ResMut<crate::resources::ActiveHeroType>,
    mut no_hero: ResMut<crate::resources::NoHeroSelected>,
) {
    if !outcome.victory || outcome.stars == 0 {
        return;
    }

    let idx = (current_level.0 as usize).saturating_sub(1);
    if idx >= save.level_stars.len() {
        return;
    }

    // Detect first-time hero unlocks: a hero whose unlock-level == this level
    // and whose level entry was 0 stars before this completion.
    let was_first_clear = save.level_stars[idx] == 0;
    let mut just_unlocked: Option<crate::data::HeroType> = None;
    if was_first_clear {
        for hero in crate::data::ALL_HERO_TYPES {
            if crate::data::hero_unlock_level(hero) == current_level.0 {
                just_unlocked = Some(hero);
                break;
            }
        }
    }

    // Only update if new star count is better
    let old_stars = save.level_stars[idx];
    if outcome.stars > old_stars {
        let new_stars = outcome.stars - old_stars;
        save.level_stars[idx] = outcome.stars;
        // Award gems: 10 per star earned (only for new stars)
        let gems = new_stars as u32 * 10;
        save.aether_gems += gems;
        info!("Level {} complete! {} stars (+{} gems)", current_level.0, outcome.stars, gems);
    }

    if let Ok(json) = serde_json::to_string_pretty(&*save) {
        write_save_string(&json);
    }

    // Auto-select the newly earned hero so "Next Level" picks them up,
    // and stash it for the GameOver screen's notification.
    if let Some(hero) = just_unlocked {
        active_hero.0 = hero;
        no_hero.0 = false;
        newly_unlocked.0 = Some(hero);
        info!("Hero unlocked: {:?}", hero);
    }
}
